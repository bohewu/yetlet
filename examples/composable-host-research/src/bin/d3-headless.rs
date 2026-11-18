//! D3 private, display-free, 127.0.0.1-only two application-process demo.
//! The Client process composes NetworkClient + bounded FaultProxy: no process
//! for each Host, no winit/wgpu/HTTP or background network thread.
use std::{
    error::Error,
    net::{Ipv4Addr, SocketAddrV4},
    thread,
    time::{Duration, Instant},
};
use yetlet_composable_host_research::{
    fault_d3::{FAULT_SEED, FaultProxy},
    simulation::Direction,
    udp_headless_d3::{
        HeadlessNetworkClient, HeadlessNetworkHost, INITIAL_EPOCH, INITIAL_SESSION, INPUT_EXPIRY,
        PEER_TIMEOUT,
    },
};

type MainResult<T> = Result<T, Box<dyn Error>>;
const STEP: Duration = Duration::from_nanos(1_000_000_000 / 60);
const HOST_LIMIT: u64 = 500;
const CLIENT_DURATION: Duration = Duration::from_millis(7200);

fn host_main() -> MainResult<()> {
    let start = Instant::now();
    let mut host = HeadlessNetworkHost::bind(start)?;
    println!(
        "D3_HOST_READY pid={} bind={} owner=single_C1 headless=true gpu=false window=false http=false",
        std::process::id(),
        host.endpoint()
    );
    let mut next_tick = start + STEP;
    let mut last_timeout = 0;
    let mut last_reconnect = 0;
    let mut last_expiry = 0;
    let mut last_logged = 0;
    loop {
        let now = Instant::now();
        host.poll(now)?;
        if host.stats.timed_out > last_timeout {
            println!(
                "D3_PEER_TIMEOUT duration_ms={} count={} session={} epoch={}",
                PEER_TIMEOUT.as_millis(),
                host.stats.timed_out,
                host.session(),
                host.epoch()
            );
            last_timeout = host.stats.timed_out;
        }
        if host.stats.reconnects > last_reconnect {
            println!(
                "D3_RECONNECTED session={} epoch={} same_owner_generation={} tick={}",
                host.session(),
                host.epoch(),
                host.snapshot().generation(),
                host.snapshot().tick()
            );
            last_reconnect = host.stats.reconnects;
        }
        let mut due = 0;
        while now >= next_tick && due < 4 && host.snapshot().tick() < HOST_LIMIT {
            host.step(now)?;
            due += 1;
            next_tick += STEP;
        }
        if now >= next_tick && due == 4 {
            // Debt management mirrors C1/C2 foreground max catch-up:
            // no unbounded replay of ticks under slow OS callbacks.
            next_tick = now + STEP;
        }
        if host.stats.expiry_release > last_expiry {
            println!(
                "D3_LOST_RELEASE_EXPIRED threshold_ms={} count={} tick={} held_remote={:?}",
                INPUT_EXPIRY.as_millis(),
                host.stats.expiry_release,
                host.snapshot().tick(),
                host.snapshot().held()[1]
            );
            last_expiry = host.stats.expiry_release;
        }
        let current = host.snapshot();
        if current.tick() != last_logged && current.tick().is_multiple_of(3) {
            println!(
                "D3_HOST_SNAPSHOT session={} epoch={} revision={} tick={} x1={} x2={}",
                host.session(),
                host.epoch(),
                current.revision(),
                current.tick(),
                current.positions()[0].x,
                current.positions()[1].x
            );
            last_logged = current.tick();
        }
        if current.tick() >= HOST_LIMIT {
            break;
        }
        thread::sleep(Duration::from_millis(3));
    }
    let latest = host.snapshot();
    let stats = host.stats;
    host.stop();
    if !host.socket_closed() {
        return Err("Host socket still open".into());
    }
    if stats.timed_out == 0 || stats.expiry_release == 0 || stats.reconnects == 0 {
        return Err(format!("D3 Host missed timeout/release/reconnect: {stats:?}").into());
    }
    println!(
        "D3_HOST_FINAL pid={} closed=true tick={} revision={} session={} epoch={} x1={} x2={} recv={} sent={} inputs={} stale={} rate_limited={} oversize={} malformed={} expired={} timed_out={} reconnects={} max_received_poll={}",
        std::process::id(),
        latest.tick(),
        latest.revision(),
        host.session(),
        host.epoch(),
        latest.positions()[0].x,
        latest.positions()[1].x,
        stats.received,
        stats.sent,
        stats.accepted_inputs,
        stats.stale,
        stats.rate_limited,
        stats.oversize,
        stats.malformed,
        stats.expiry_release,
        stats.timed_out,
        stats.reconnects,
        stats.max_received_one_poll,
    );
    Ok(())
}

fn client_main(host_endpoint: SocketAddrV4) -> MainResult<()> {
    let start = Instant::now();
    let placeholder = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 1);
    let mut client = HeadlessNetworkClient::bind(placeholder, INITIAL_SESSION, INITIAL_EPOCH, 100)?;
    let mut proxy = FaultProxy::bind(host_endpoint, client.endpoint(), FAULT_SEED)?;
    client.select_initial_proxy(proxy.endpoint())?;
    println!(
        "D3_CLIENT_READY pid={} local={} proxy={} host={} seed={} fault_capacity=32 headless=true gpu=false window=false http=false",
        std::process::id(),
        client.endpoint(),
        proxy.endpoint(),
        host_endpoint,
        FAULT_SEED
    );
    let mut faults_started = false;
    let mut silence_started = false;
    let mut faults_ended = false;
    let mut reconnect_started = false;
    let mut stale_sent = false;
    let mut old_snapshot = None;
    let mut last_logged = 0;
    let mut saw_new_epoch = false;
    let mut saw_old_epoch = false;
    loop {
        let now = Instant::now();
        let elapsed = now.duration_since(start);
        if !faults_started && elapsed >= Duration::from_millis(420) {
            proxy.set_faults(true);
            faults_started = true;
            println!(
                "D3_FAULTS_ON seed={} policy=drop10_duplicate5_reorder100ms",
                FAULT_SEED
            );
        }
        if !silence_started && elapsed >= Duration::from_millis(1150) {
            client.set_silence(true); // deliberate lost-release
            silence_started = true;
            println!(
                "D3_CLIENT_INPUT_SILENCED elapsed_ms={} intent=simulate_lost_release",
                elapsed.as_millis()
            );
        }
        if !faults_ended && elapsed >= Duration::from_millis(4450) {
            proxy.set_faults(false);
            faults_ended = true;
            println!(
                "D3_FAULTS_OFF elapsed_ms={} data_seen={} drops={} duplicates={} reordered={}",
                elapsed.as_millis(),
                proxy.counters().data_seen,
                proxy.counters().dropped,
                proxy.counters().duplicated,
                proxy.counters().reordered
            );
        }
        if !reconnect_started && elapsed >= Duration::from_millis(4780) {
            if client.snapshot().is_none() {
                return Err("no initial Client Snapshot".into());
            }
            old_snapshot = client.snapshot();
            client.reconnect(INITIAL_SESSION + 1, INITIAL_EPOCH + 1, 101)?;
            client.set_direction(Direction::Still);
            client.set_silence(false);
            reconnect_started = true;
            println!(
                "D3_CLIENT_RECONNECT_SENT session={} epoch={} nonce=101 old_snapshot_retained=true",
                INITIAL_SESSION + 1,
                INITIAL_EPOCH + 1
            );
        }
        proxy.poll(now)?;
        client.poll(now)?;
        proxy.poll(now)?;
        if let Some(snapshot) = client.snapshot() {
            if snapshot.session == INITIAL_SESSION {
                saw_old_epoch = true;
            }
            if snapshot.session == INITIAL_SESSION + 1 {
                saw_new_epoch = true;
            }
            if snapshot.revision != last_logged {
                println!(
                    "D3_CLIENT_SNAPSHOT session={} epoch={} revision={} tick={} x1={} x2={}",
                    snapshot.session,
                    snapshot.epoch,
                    snapshot.revision,
                    snapshot.tick,
                    snapshot.positions[0].x,
                    snapshot.positions[1].x
                );
                last_logged = snapshot.revision;
            }
        }
        if saw_new_epoch && !stale_sent {
            client.send_old_marker_qa()?;
            stale_sent = true;
            println!(
                "D3_OLD_MARKERS_SENT kind=Hello+HeldInput+Leave old_session={} old_epoch={}",
                INITIAL_SESSION, INITIAL_EPOCH
            );
        }
        if elapsed >= CLIENT_DURATION {
            break;
        }
        thread::sleep(Duration::from_millis(3));
    }
    // Stop data-plane faults and give previously queued control packets one
    // bounded flush opportunity before explicitly closing only owned sockets.
    proxy.set_faults(false);
    let count = proxy.counters();
    let latest = client.snapshot();
    let client_stats = client.stats;
    proxy.stop();
    client.stop();
    if !proxy.stopped() || !client.socket_closed() {
        return Err("Client/proxy sockets not closed".into());
    }
    if !faults_started
        || !faults_ended
        || !silence_started
        || !reconnect_started
        || !stale_sent
        || !saw_old_epoch
        || !saw_new_epoch
        || count.fault_eligible < 40
        || count.dropped == 0
        || count.duplicated == 0
        || client_stats.snapshots_received < 10
    {
        return Err(format!(
            "missing bounded D3 real fault or reconnect proof: client={client_stats:?} faults={count:?} new={saw_new_epoch} old={saw_old_epoch}"
        ).into());
    }
    let state = latest.ok_or("Client ended without trusted Snapshot")?;
    if state.session != INITIAL_SESSION + 1 || state.epoch != INITIAL_EPOCH + 1 {
        return Err("Client latest Snapshot was not fresh new session".into());
    }
    if old_snapshot
        .as_ref()
        .is_none_or(|old| old.session != INITIAL_SESSION)
    {
        return Err("Retained old snapshot changed".into());
    }
    println!(
        "D3_CLIENT_FINAL pid={} closed=true session={} epoch={} tick={} revision={} x1={} x2={} recv={} sent={} accepted_snapshots={} stale={} fault_seed={} eligible={} drops={} duplicate={} reordered={} delayed={} proxy_forwarded={} proxy_queue_full={} proxy_max_pending={} proxy_foreign={} no_socket_workers=true",
        std::process::id(),
        state.session,
        state.epoch,
        state.tick,
        state.revision,
        state.positions[0].x,
        state.positions[1].x,
        client_stats.received,
        client_stats.sent,
        client_stats.snapshots_received,
        client_stats.stale,
        FAULT_SEED,
        count.fault_eligible,
        count.dropped,
        count.duplicated,
        count.reordered,
        count.delayed,
        count.forwarded,
        count.queue_full,
        count.max_pending,
        count.foreign,
    );
    Ok(())
}
fn main() -> MainResult<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    match args.as_slice() {
        [role] if role == "host" => host_main(),
        [role, endpoint] if role == "client" => {
            let endpoint: SocketAddrV4 = endpoint.parse()?;
            if endpoint.ip() != &Ipv4Addr::LOCALHOST || endpoint.port() == 0 {
                return Err("Explicit 127.0.0.1:<port> only".into());
            }
            client_main(endpoint)
        }
        _ => Err("Usage: d3-headless host | client 127.0.0.1:<actual-host-port>".into()),
    }
}
