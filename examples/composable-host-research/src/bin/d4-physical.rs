//! Finite private D4 application qualification on two explicit IPv4 endpoints.
//! Reuses the D3 single C1 owner and YTP1 replica; no SDK API, proxy or GUI.
use std::{
    error::Error,
    fmt::Write,
    io,
    net::{SocketAddrV4, UdpSocket},
    thread,
    time::{Duration, Instant},
};
use yetlet_composable_host_research::{
    protocol::{Message, WireSnapshot, decode, encode},
    simulation::Direction,
    udp_headless_d3::{
        HOST_GENERATION, HeadlessNetworkClient, HeadlessNetworkHost, INITIAL_EPOCH,
        INITIAL_SESSION, INPUT_EXPIRY, PEER_TIMEOUT,
    },
};

const USAGE: &str = "Usage: d4-physical host|client --bind IPv4:PORT --peer IPv4:PORT";
const HOST_LIMIT: Duration = Duration::from_secs(45);
const CLIENT_LIMIT: Duration = Duration::from_secs(20);
const STEP: Duration = Duration::from_nanos(1_000_000_000 / 60);
const POLL_PAUSE: Duration = Duration::from_millis(3);
const WEST_DURATION: Duration = Duration::from_millis(600);
const SILENCE_DURATION: Duration = Duration::from_millis(3600);
const EAST_DURATION: Duration = Duration::from_secs(1);
const STILL_DURATION: Duration = Duration::from_millis(500);
const INITIAL_NONCE: u64 = 100;
const INITIAL_REMOTE_X: i32 = 20;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Role {
    Host,
    Client,
}

impl Role {
    fn name(self) -> &'static str {
        match self {
            Self::Host => "host",
            Self::Client => "client",
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct Config {
    role: Role,
    bind: SocketAddrV4,
    peer: SocketAddrV4,
}

fn endpoint(text: &str) -> Result<SocketAddrV4, String> {
    let address: SocketAddrV4 = text
        .parse()
        .map_err(|_| format!("invalid IPv4 endpoint: {text}"))?;
    let first = address.ip().octets()[0];
    if address.port() == 0 || first == 0 || first >= 224 {
        return Err(format!(
            "endpoint must be concrete unicast with a nonzero port: {text}"
        ));
    }
    Ok(address)
}

fn parse_args(args: &[String]) -> Result<Config, String> {
    let role = match args.first().map(String::as_str) {
        Some("host") => Role::Host,
        Some("client") => Role::Client,
        _ => return Err("one host or client role is required".into()),
    };
    let mut bind = None;
    let mut peer = None;
    let mut remaining = &args[1..];
    while let Some(flag) = remaining.first() {
        let slot = match flag.as_str() {
            "--bind" => &mut bind,
            "--peer" => &mut peer,
            _ => return Err(format!("unknown argument: {flag}")),
        };
        if slot.is_some() {
            return Err(format!("duplicate argument: {flag}"));
        }
        let value = remaining
            .get(1)
            .ok_or_else(|| format!("missing value for {flag}"))?;
        *slot = Some(endpoint(value)?);
        remaining = &remaining[2..];
    }
    let bind = bind.ok_or("missing --bind")?;
    let peer = peer.ok_or("missing --peer")?;
    if bind == peer {
        return Err("bind and peer must be distinct endpoints".into());
    }
    Ok(Config { role, bind, peer })
}

fn require(condition: bool, message: &str) -> io::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(io::Error::other(message))
    }
}

fn codec_error(error: impl std::fmt::Debug) -> io::Error {
    io::Error::other(format!("D4 canonical snapshot: {error:?}"))
}

fn hex(bytes: &[u8]) -> String {
    let mut result = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut result, "{byte:02x}").expect("writing to String cannot fail");
    }
    result
}

fn wire_fields(snapshot: &WireSnapshot) -> String {
    format!(
        "session={} epoch={} generation={} revision={} tick={} x1={} y1={} x2={} y2={} held1={:?} held2={:?} paused={}",
        snapshot.session,
        snapshot.epoch,
        snapshot.generation,
        snapshot.revision,
        snapshot.tick,
        snapshot.positions[0].x,
        snapshot.positions[0].y,
        snapshot.positions[1].x,
        snapshot.positions[1].y,
        snapshot.held[0],
        snapshot.held[1],
        snapshot.paused,
    )
}

fn log_wire(prefix: &str, snapshot: &WireSnapshot, bytes: &[u8]) {
    println!(
        "{prefix} {} canonical_hex={}",
        wire_fields(snapshot),
        hex(bytes)
    );
}

#[derive(Default)]
struct TransientErrors {
    reset: u64,
    refused: u64,
}

impl TransientErrors {
    fn observe(
        &mut self,
        result: io::Result<()>,
        role: Role,
        operation: &'static str,
    ) -> io::Result<()> {
        match result {
            Ok(()) => Ok(()),
            Err(error)
                if matches!(
                    error.kind(),
                    io::ErrorKind::ConnectionReset | io::ErrorKind::ConnectionRefused
                ) =>
            {
                match error.kind() {
                    io::ErrorKind::ConnectionReset => self.reset += 1,
                    io::ErrorKind::ConnectionRefused => self.refused += 1,
                    _ => unreachable!(),
                }
                let count = self.reset + self.refused;
                if count <= 4 {
                    println!(
                        "D4_SOCKET_TRANSIENT role={} operation={operation} kind={:?} count={count}",
                        role.name(),
                        error.kind(),
                    );
                }
                Ok(())
            }
            Err(error) => Err(error),
        }
    }
}

#[derive(Default)]
struct HostProgress {
    initial_movement: bool,
    expiry_neutral: bool,
    first_timeout: Option<(u64, u64)>,
    reconnected: bool,
    reconnect_x: i32,
    reconnect_input_baseline: u64,
    replacement_movement: bool,
    replacement_timeout: bool,
    observed_inputs: u64,
    observed_expiry: u64,
    observed_timeouts: u64,
    observed_reconnects: u64,
    last_publication: Option<Vec<u8>>,
}

impl HostProgress {
    fn complete(&self) -> bool {
        self.initial_movement
            && self.expiry_neutral
            && self.first_timeout.is_some()
            && self.reconnected
            && self.replacement_movement
            && self.replacement_timeout
    }

    fn observe(&mut self, host: &HeadlessNetworkHost, elapsed: Duration) -> io::Result<()> {
        let current = host.snapshot();
        let session = host.session();
        let epoch = host.epoch();
        require(
            current.generation() == HOST_GENERATION,
            "authoritative generation changed",
        )?;
        if session == INITIAL_SESSION
            && epoch == INITIAL_EPOCH
            && host.stats.accepted_inputs > 0
            && current.positions()[1].x < INITIAL_REMOTE_X
            && !self.initial_movement
        {
            self.initial_movement = true;
            println!(
                "D4_HOST_INITIAL_MOVEMENT elapsed_ms={} tick={} x2={} accepted_inputs={}",
                elapsed.as_millis(),
                current.tick(),
                current.positions()[1].x,
                host.stats.accepted_inputs,
            );
        }
        if host.stats.expiry_release > self.observed_expiry {
            require(
                current.held()[1] == Direction::Still,
                "expired remote input was not neutral in the authoritative publication",
            )?;
            if session == INITIAL_SESSION && epoch == INITIAL_EPOCH {
                self.expiry_neutral = true;
            }
            println!(
                "D4_HOST_INPUT_EXPIRED session={session} epoch={epoch} tick={} held2={:?} threshold_ms={} count={}",
                current.tick(),
                current.held()[1],
                INPUT_EXPIRY.as_millis(),
                host.stats.expiry_release,
            );
            self.observed_expiry = host.stats.expiry_release;
        }
        if host.stats.reconnects > self.observed_reconnects {
            let (timeout_tick, timeout_generation) = self
                .first_timeout
                .ok_or_else(|| io::Error::other("reconnect preceded the initial peer timeout"))?;
            require(
                session == INITIAL_SESSION + 1
                    && epoch == INITIAL_EPOCH + 1
                    && current.generation() == timeout_generation
                    && current.tick() > timeout_tick
                    && !self.reconnected,
                "replacement session did not preserve the progressing original authority",
            )?;
            self.reconnected = true;
            self.reconnect_x = current.positions()[1].x;
            // Include any replacement input received in the same bounded poll.
            self.reconnect_input_baseline = self.observed_inputs;
            self.observed_reconnects = host.stats.reconnects;
            println!(
                "D4_HOST_RECONNECTED session={session} epoch={epoch} generation={} tick={} first_timeout_tick={timeout_tick} baseline_x2={} input_baseline={}",
                current.generation(),
                current.tick(),
                self.reconnect_x,
                self.reconnect_input_baseline,
            );
        }
        if self.reconnected
            && session == INITIAL_SESSION + 1
            && epoch == INITIAL_EPOCH + 1
            && host.stats.accepted_inputs > self.reconnect_input_baseline
            && current.positions()[1].x > self.reconnect_x
            && !self.replacement_movement
        {
            self.replacement_movement = true;
            println!(
                "D4_HOST_REPLACEMENT_MOVEMENT session={session} epoch={epoch} tick={} x2={} accepted_since_reconnect={}",
                current.tick(),
                current.positions()[1].x,
                host.stats.accepted_inputs - self.reconnect_input_baseline,
            );
        }
        if host.stats.timed_out > self.observed_timeouts {
            require(!host.connected(), "timed-out peer remained connected")?;
            require(
                current.held()[1] == Direction::Still,
                "timed-out peer retained held input",
            )?;
            if session == INITIAL_SESSION && epoch == INITIAL_EPOCH && self.first_timeout.is_none()
            {
                require(
                    self.initial_movement && self.expiry_neutral,
                    "initial timeout lacked remote movement or neutral expiry evidence",
                )?;
                self.first_timeout = Some((current.tick(), current.generation()));
                println!(
                    "D4_HOST_FIRST_TIMEOUT session={session} epoch={epoch} generation={} tick={} threshold_ms={}",
                    current.generation(),
                    current.tick(),
                    PEER_TIMEOUT.as_millis(),
                );
            } else {
                require(
                    session == INITIAL_SESSION + 1
                        && epoch == INITIAL_EPOCH + 1
                        && self.reconnected
                        && self.replacement_movement
                        && !self.replacement_timeout,
                    "timeout was not the qualified replacement peer's timeout",
                )?;
                self.replacement_timeout = true;
                println!(
                    "D4_HOST_REPLACEMENT_TIMEOUT session={session} epoch={epoch} generation={} tick={} threshold_ms={}",
                    current.generation(),
                    current.tick(),
                    PEER_TIMEOUT.as_millis(),
                );
            }
            self.observed_timeouts = host.stats.timed_out;
        }
        self.observed_inputs = host.stats.accepted_inputs;
        if let Some(bytes) = host.last_sent_snapshot_frame()
            && self.last_publication.as_deref() != Some(bytes)
        {
            let Message::Snapshot(snapshot) = decode(bytes).map_err(codec_error)? else {
                return Err(io::Error::other("retained publication was not a Snapshot"));
            };
            log_wire("D4_PUBLICATION", &snapshot, bytes);
            self.last_publication = Some(bytes.to_vec());
        }
        Ok(())
    }
}

fn host_scenario(
    host: &mut HeadlessNetworkHost,
    started: Instant,
    progress: &mut HostProgress,
    transient: &mut TransientErrors,
) -> io::Result<()> {
    let mut next_tick = started + STEP;
    loop {
        let now = Instant::now();
        require(
            now.duration_since(started) < HOST_LIMIT,
            "host deadline expired",
        )?;
        transient.observe(host.poll(now), Role::Host, "poll")?;
        progress.observe(host, now.duration_since(started))?;
        let mut due = 0;
        while now >= next_tick && due < 4 {
            transient.observe(host.step(now), Role::Host, "step")?;
            progress.observe(host, now.duration_since(started))?;
            next_tick += STEP;
            due += 1;
        }
        if now >= next_tick && due == 4 {
            next_tick = now + STEP;
        }
        if progress.complete() {
            return Ok(());
        }
        // Tolerated socket errors never bypass stepping, the deadline or sleep.
        thread::sleep(POLL_PAUSE);
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ClientPhase {
    #[default]
    AwaitInitial,
    West,
    Silent,
    AwaitReplacement,
    East,
    Still,
}

#[derive(Default)]
struct ClientProgress {
    phase: ClientPhase,
    initial_movement: bool,
    silence_neutral: bool,
    replacement_movement: bool,
    replacement_still: bool,
    last_replica: Option<Vec<u8>>,
}

fn client_scenario(
    client: &mut HeadlessNetworkClient,
    started: Instant,
    progress: &mut ClientProgress,
    transient: &mut TransientErrors,
) -> io::Result<()> {
    let mut phase_started = started;
    let mut silence_revision = 0;
    let mut replacement_x = 0;
    let mut still_revision = 0;
    client.set_direction(Direction::West);
    loop {
        let now = Instant::now();
        require(
            now.duration_since(started) < CLIENT_LIMIT,
            "client deadline expired",
        )?;
        transient.observe(client.poll(now), Role::Client, "poll")?;
        let latest = client.snapshot();
        if let Some(snapshot) = latest.as_deref() {
            let bytes = encode(&Message::Snapshot(snapshot.clone())).map_err(codec_error)?;
            if progress.last_replica.as_deref() != Some(bytes.as_slice()) {
                log_wire("D4_REPLICA", snapshot, &bytes);
                progress.last_replica = Some(bytes);
            }
            require(
                snapshot.generation == HOST_GENERATION,
                "replica generation changed",
            )?;
        }
        match progress.phase {
            ClientPhase::AwaitInitial => {
                if let Some(snapshot) = latest.as_deref() {
                    require(
                        snapshot.session == INITIAL_SESSION
                            && snapshot.epoch == INITIAL_EPOCH
                            && snapshot.positions[1].x == INITIAL_REMOTE_X,
                        "initial replica did not identify the initial remote-player state",
                    )?;
                    progress.phase = ClientPhase::West;
                    phase_started = now;
                    println!(
                        "D4_CLIENT_INITIAL_SNAPSHOT {} phase=West",
                        wire_fields(snapshot),
                    );
                }
            }
            ClientPhase::West => {
                if let Some(snapshot) = latest.as_deref()
                    && snapshot.session == INITIAL_SESSION
                    && snapshot.epoch == INITIAL_EPOCH
                    && snapshot.positions[1].x < INITIAL_REMOTE_X
                {
                    progress.initial_movement = true;
                }
                if now.duration_since(phase_started) >= WEST_DURATION {
                    require(
                        progress.initial_movement,
                        "initial remote movement was not observed",
                    )?;
                    silence_revision = latest
                        .as_ref()
                        .ok_or_else(|| io::Error::other("initial replica missing"))?
                        .revision;
                    client.set_silence(true);
                    progress.phase = ClientPhase::Silent;
                    // The final input poll happened immediately before this entry.
                    phase_started = now;
                    println!(
                        "D4_CLIENT_SILENCE session={INITIAL_SESSION} epoch={INITIAL_EPOCH} revision={silence_revision} elapsed_ms={} duration_ms={}",
                        now.duration_since(started).as_millis(),
                        SILENCE_DURATION.as_millis(),
                    );
                }
            }
            ClientPhase::Silent => {
                if let Some(snapshot) = latest.as_deref()
                    && snapshot.session == INITIAL_SESSION
                    && snapshot.epoch == INITIAL_EPOCH
                    && snapshot.revision > silence_revision
                    && snapshot.held[1] == Direction::Still
                {
                    if !progress.silence_neutral {
                        println!("D4_CLIENT_NEUTRAL_REPLICA {}", wire_fields(snapshot));
                    }
                    progress.silence_neutral = true;
                }
                if now.duration_since(phase_started) >= SILENCE_DURATION {
                    require(
                        progress.silence_neutral,
                        "silence did not yield a newer neutral initial-session replica",
                    )?;
                    client.reconnect(INITIAL_SESSION + 1, INITIAL_EPOCH + 1, INITIAL_NONCE + 1)?;
                    client.set_direction(Direction::East);
                    client.set_silence(false);
                    progress.phase = ClientPhase::AwaitReplacement;
                    phase_started = now;
                    println!(
                        "D4_CLIENT_RECONNECT_REQUEST local={} session={} epoch={} nonce={} same_socket=true",
                        client.endpoint(),
                        INITIAL_SESSION + 1,
                        INITIAL_EPOCH + 1,
                        INITIAL_NONCE + 1,
                    );
                }
            }
            ClientPhase::AwaitReplacement => {
                if let Some(snapshot) = latest.as_deref() {
                    require(
                        snapshot.session == INITIAL_SESSION + 1
                            && snapshot.epoch == INITIAL_EPOCH + 1,
                        "replacement replica used the wrong session",
                    )?;
                    replacement_x = snapshot.positions[1].x;
                    progress.phase = ClientPhase::East;
                    phase_started = now;
                    println!(
                        "D4_CLIENT_REPLACEMENT_SNAPSHOT {} phase=East baseline_x2={replacement_x}",
                        wire_fields(snapshot),
                    );
                }
            }
            ClientPhase::East => {
                if let Some(snapshot) = latest.as_deref()
                    && snapshot.session == INITIAL_SESSION + 1
                    && snapshot.epoch == INITIAL_EPOCH + 1
                    && snapshot.positions[1].x > replacement_x
                {
                    progress.replacement_movement = true;
                }
                if now.duration_since(phase_started) >= EAST_DURATION {
                    require(
                        progress.replacement_movement,
                        "replacement session did not yield actual eastward movement",
                    )?;
                    still_revision = latest
                        .as_ref()
                        .ok_or_else(|| io::Error::other("replacement replica missing"))?
                        .revision;
                    client.set_direction(Direction::Still);
                    progress.phase = ClientPhase::Still;
                    phase_started = now;
                    println!(
                        "D4_CLIENT_STILL_REQUEST session={} epoch={} after_revision={still_revision}",
                        INITIAL_SESSION + 1,
                        INITIAL_EPOCH + 1,
                    );
                }
            }
            ClientPhase::Still => {
                if let Some(snapshot) = latest.as_deref()
                    && snapshot.session == INITIAL_SESSION + 1
                    && snapshot.epoch == INITIAL_EPOCH + 1
                    && snapshot.revision > still_revision
                    && snapshot.held[1] == Direction::Still
                {
                    progress.replacement_still = true;
                }
                if now.duration_since(phase_started) >= STILL_DURATION {
                    require(
                        progress.replacement_still,
                        "replacement Still input was not published to the replica",
                    )?;
                    return Ok(());
                }
            }
        }
        thread::sleep(POLL_PAUSE);
    }
}

struct Cleanup {
    closed: bool,
    rebound: bool,
    error: Option<io::Error>,
}

fn verify_cleanup(local: SocketAddrV4, closed: bool) -> Cleanup {
    // This exact-address bind is also attempted when closed=false, preserving
    // both observations. The temporary verifier socket never outlives this call.
    let rebound = UdpSocket::bind(local);
    let (rebound_ok, rebind_error) = match rebound {
        Ok(socket) => {
            drop(socket);
            (true, None)
        }
        Err(error) => (false, Some(error)),
    };
    let error = if !closed {
        Some(io::Error::other(
            "original application socket was not closed",
        ))
    } else {
        rebind_error
    };
    Cleanup {
        closed,
        rebound: rebound_ok,
        error,
    }
}

fn finish(scenario: io::Result<()>, cleanup: Cleanup, role: Role) -> io::Result<()> {
    match (scenario, cleanup.error) {
        (Err(primary), Some(secondary)) => {
            eprintln!(
                "D4_CLEANUP_ERROR role={} secondary={secondary:?}",
                role.name()
            );
            Err(primary)
        }
        (Err(primary), None) => Err(primary),
        (Ok(()), Some(error)) => Err(error),
        (Ok(()), None) => Ok(()),
    }
}

fn run_host(config: Config) -> io::Result<()> {
    let started = Instant::now();
    let mut host = HeadlessNetworkHost::bind_on(started, config.bind, config.peer)?;
    println!(
        "D4_READY role=host pid={} local={} peer={} runtime_bound_ms={} owner=single_C1 headless=true",
        std::process::id(),
        host.endpoint(),
        config.peer,
        HOST_LIMIT.as_millis(),
    );
    let mut progress = HostProgress::default();
    let mut transient = TransientErrors::default();
    let scenario = host_scenario(&mut host, started, &mut progress, &mut transient);
    let final_state = host.snapshot();
    let stats = host.stats;
    host.stop();
    let cleanup = verify_cleanup(config.bind, host.socket_closed());
    println!(
        "D4_HOST_FINAL pid={} passed={} elapsed_ms={} closed={} rebind={} session={} epoch={} generation={} revision={} tick={} x2={} held2={:?} initial_movement={} expiry_neutral={} first_timeout={} reconnected={} replacement_movement={} replacement_timeout={} received={} sent={} accepted_inputs={} snapshots_sent={} expired={} timeouts={} reconnects={} max_received_poll={} connection_reset={} connection_refused={}",
        std::process::id(),
        scenario.is_ok() && cleanup.error.is_none(),
        started.elapsed().as_millis(),
        cleanup.closed,
        cleanup.rebound,
        host.session(),
        host.epoch(),
        final_state.generation(),
        final_state.revision(),
        final_state.tick(),
        final_state.positions()[1].x,
        final_state.held()[1],
        progress.initial_movement,
        progress.expiry_neutral,
        progress.first_timeout.is_some(),
        progress.reconnected,
        progress.replacement_movement,
        progress.replacement_timeout,
        stats.received,
        stats.sent,
        stats.accepted_inputs,
        stats.snapshots_sent,
        stats.expiry_release,
        stats.timed_out,
        stats.reconnects,
        stats.max_received_one_poll,
        transient.reset,
        transient.refused,
    );
    finish(scenario, cleanup, Role::Host)
}

fn run_client(config: Config) -> io::Result<()> {
    let started = Instant::now();
    let mut client = HeadlessNetworkClient::bind_on(
        config.bind,
        config.peer,
        INITIAL_SESSION,
        INITIAL_EPOCH,
        INITIAL_NONCE,
    )?;
    println!(
        "D4_READY role=client pid={} local={} peer={} runtime_bound_ms={} replica=read_only headless=true",
        std::process::id(),
        client.endpoint(),
        config.peer,
        CLIENT_LIMIT.as_millis(),
    );
    let mut progress = ClientProgress::default();
    let mut transient = TransientErrors::default();
    let scenario = client_scenario(&mut client, started, &mut progress, &mut transient);
    let selected = client
        .snapshot()
        .map(|snapshot| wire_fields(&snapshot))
        .unwrap_or_else(|| "snapshot=none".into());
    let stats = client.stats;
    client.stop();
    let cleanup = verify_cleanup(config.bind, client.socket_closed());
    println!(
        "D4_CLIENT_FINAL pid={} passed={} elapsed_ms={} closed={} rebind={} {} phase={:?} initial_movement={} silence_neutral={} replacement_movement={} replacement_still={} received={} sent={} snapshots_received={} welcomes={} joins={} stale={} max_received_poll={} connection_reset={} connection_refused={}",
        std::process::id(),
        scenario.is_ok() && cleanup.error.is_none(),
        started.elapsed().as_millis(),
        cleanup.closed,
        cleanup.rebound,
        selected,
        progress.phase,
        progress.initial_movement,
        progress.silence_neutral,
        progress.replacement_movement,
        progress.replacement_still,
        stats.received,
        stats.sent,
        stats.snapshots_received,
        stats.welcomes,
        stats.joins,
        stats.stale,
        stats.max_received_one_poll,
        transient.reset,
        transient.refused,
    );
    finish(scenario, cleanup, Role::Client)
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let config = parse_args(&args).map_err(|error| {
        io::Error::new(io::ErrorKind::InvalidInput, format!("{error}\n{USAGE}"))
    })?;
    match config.role {
        Role::Host => run_host(config)?,
        Role::Client => run_client(config)?,
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse(arguments: &[&str]) -> Result<Config, String> {
        parse_args(
            &arguments
                .iter()
                .map(|arg| (*arg).into())
                .collect::<Vec<_>>(),
        )
    }

    #[test]
    fn explicit_endpoint_selection_accepts_both_flag_orders() {
        let host = parse(&[
            "host",
            "--bind",
            "198.51.100.10:40001",
            "--peer",
            "198.51.100.20:40002",
        ])
        .unwrap();
        let client = parse(&[
            "client",
            "--peer",
            "198.51.100.10:40001",
            "--bind",
            "198.51.100.20:40002",
        ])
        .unwrap();
        assert_eq!(host.bind, client.peer);
        assert_eq!(host.peer, client.bind);
        assert_eq!(host.role, Role::Host);
        assert_eq!(client.role, Role::Client);
    }

    #[test]
    fn missing_duplicate_unknown_and_extra_arguments_fail_before_acquisition() {
        for arguments in [
            vec![],
            vec!["host"],
            vec!["host", "--bind"],
            vec!["host", "--bind", "127.0.0.1:1"],
            vec!["host", "--bind", "127.0.0.1:1", "--bind", "127.0.0.1:2"],
            vec![
                "host",
                "--bind",
                "127.0.0.1:1",
                "--peer",
                "127.0.0.1:2",
                "--other",
                "x",
            ],
            vec![
                "host",
                "--bind",
                "127.0.0.1:1",
                "--peer",
                "127.0.0.1:2",
                "extra",
            ],
            vec!["unknown", "--bind", "127.0.0.1:1", "--peer", "127.0.0.1:2"],
        ] {
            assert!(parse(&arguments).is_err(), "{arguments:?}");
        }
    }

    #[test]
    fn wildcard_multicast_reserved_zero_port_ipv6_and_self_peers_are_rejected() {
        for invalid in [
            "0.0.0.0:42",
            "0.1.2.3:42",
            "224.0.0.1:42",
            "240.0.0.1:42",
            "255.255.255.255:42",
            "198.51.100.10:0",
            "[::1]:42",
        ] {
            assert!(endpoint(invalid).is_err(), "{invalid}");
        }
        assert!(parse(&["host", "--bind", "127.0.0.1:42", "--peer", "127.0.0.1:42"]).is_err());
    }
}
