//! Private D3/D4 bounded HeadlessHost+NetworkHost and NetworkClient over UDP.
//! Existing constructors and proxy selection retain their loopback-only defaults;
//! explicit endpoints pin one exact peer before decoding or session admission.
//! No GPU, Window, HTTP, background thread or public Yetlet API.
//! Session/epoch markers and selected source addresses are NOT authentication.
use crate::{
    protocol::{
        Binding, EndpointId, HostFixture, MAX_WIRE_BYTES, Message, ReplicaFixture, WireSnapshot,
        decode, encode,
    },
    simulation::{Direction, Player, Snapshot},
};
use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    rc::Rc,
    time::{Duration, Instant},
};

pub const MAX_RECEIVE: usize = 8;
pub const MAX_FRAME_STORAGE: usize = MAX_WIRE_BYTES + 1;
pub const INPUT_EXPIRY: Duration = Duration::from_millis(250);
pub const PEER_TIMEOUT: Duration = Duration::from_secs(3);
pub const JOIN_RETRY: Duration = Duration::from_millis(250);
pub const MAX_JOIN_RETRIES: u32 = 3;
pub const MAX_ACCEPTED_INPUT_PER_SECOND: u32 = 30;
pub const MAX_JOIN_REPLIES_PER_SECOND: u32 = 3;
pub const MAX_SNAPSHOT_PER_SECOND: u32 = 20;
pub const SNAPSHOT_MIN_INTERVAL: Duration =
    Duration::from_millis(1000 / MAX_SNAPSHOT_PER_SECOND as u64);
pub const MAX_TICK_LAG: u64 = 12;
pub const HOST_GENERATION: u64 = 1;
pub const INITIAL_SESSION: u64 = 7101;
pub const INITIAL_EPOCH: u64 = 1;
pub const REMOTE_PEER: u64 = 22;

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq)]
pub struct NetworkStats {
    pub received: u64,
    pub sent: u64,
    pub oversize: u64,
    pub foreign: u64,
    pub malformed: u64,
    pub rejected: u64,
    pub stale: u64,
    pub accepted_inputs: u64,
    pub snapshots_sent: u64,
    pub snapshots_received: u64,
    pub welcomes: u64,
    pub joins: u64,
    pub rate_limited: u64,
    pub expiry_release: u64,
    pub timed_out: u64,
    pub reconnects: u64,
    pub max_received_one_poll: usize,
}
fn err(s: &str) -> io::Error {
    io::Error::other(s)
}
fn loopback(address: SocketAddr) -> Option<SocketAddrV4> {
    match address {
        SocketAddr::V4(ip) if ip.ip() == &Ipv4Addr::LOCALHOST && ip.port() != 0 => Some(ip),
        _ => None,
    }
}
fn bind() -> io::Result<(UdpSocket, SocketAddrV4)> {
    let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))?;
    socket.set_nonblocking(true)?;
    let addr = loopback(socket.local_addr()?).ok_or_else(|| err("IPv4 loopback bind failed"))?;
    Ok((socket, addr))
}
fn concrete_endpoint(address: SocketAddrV4) -> bool {
    address.port() != 0
        && !address.ip().is_unspecified()
        && !address.ip().is_multicast()
        && !address.ip().is_broadcast()
}
fn validate_selected_pair(local: SocketAddrV4, peer: SocketAddrV4) -> io::Result<()> {
    if !concrete_endpoint(local) || !concrete_endpoint(peer) || local == peer {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "selected endpoints require distinct concrete IPv4 addresses and nonzero ports",
        ));
    }
    Ok(())
}
fn bind_selected(local: SocketAddrV4) -> io::Result<(UdpSocket, SocketAddrV4)> {
    let socket = UdpSocket::bind(local)?;
    socket.set_nonblocking(true)?;
    if socket.local_addr()? != SocketAddr::V4(local) {
        return Err(err(
            "selected IPv4 bind did not retain the requested endpoint",
        ));
    }
    Ok((socket, local))
}
fn send(sock: &UdpSocket, to: SocketAddrV4, frame: &[u8]) -> io::Result<bool> {
    if !concrete_endpoint(to) || frame.len() > MAX_WIRE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "D3/D4 concrete destination and bounded frame required",
        ));
    }
    match sock.send_to(frame, to) {
        Ok(len) if len == frame.len() => Ok(true),
        Ok(_) => Err(io::Error::new(io::ErrorKind::WriteZero, "partial datagram")),
        Err(e) if e.kind() == io::ErrorKind::WouldBlock => Ok(false),
        Err(e) => Err(e),
    }
}

pub struct HeadlessNetworkHost {
    socket: Option<UdpSocket>,
    local: SocketAddrV4,
    owner: HostFixture,
    allowed_peer: Option<SocketAddrV4>,
    peer: Option<SocketAddrV4>,
    nonce: Option<u64>,
    session: u64,
    epoch: u64,
    last_peer: Option<Instant>,
    last_input: Option<Instant>,
    remote_direction: Direction,
    last_wire_tick: u64,
    last_input_sequence: u64,
    last_snapshot_at: Option<Instant>,
    last_sent_snapshot_frame: Option<Vec<u8>>,
    rate_window: Instant,
    inputs_in_window: u32,
    joins_in_window: u32,
    pub stats: NetworkStats,
}
impl HeadlessNetworkHost {
    pub fn bind(now: Instant) -> io::Result<Self> {
        let (socket, local) = bind()?;
        Self::from_bound(socket, local, None, now)
    }
    /// Explicit private-application route; only this exact peer may begin or
    /// replace a session. The allowance survives peer timeout and Leave.
    pub fn bind_on(
        now: Instant,
        local: SocketAddrV4,
        exact_peer: SocketAddrV4,
    ) -> io::Result<Self> {
        validate_selected_pair(local, exact_peer)?;
        let (socket, local) = bind_selected(local)?;
        Self::from_bound(socket, local, Some(exact_peer), now)
    }
    fn from_bound(
        socket: UdpSocket,
        local: SocketAddrV4,
        allowed_peer: Option<SocketAddrV4>,
        now: Instant,
    ) -> io::Result<Self> {
        let owner = HostFixture::new(
            HOST_GENERATION,
            [
                (
                    EndpointId(1001),
                    Binding {
                        peer: 11,
                        session: INITIAL_SESSION,
                        epoch: INITIAL_EPOCH,
                        player: Player::One,
                    },
                ),
                (
                    EndpointId(1002),
                    Binding {
                        peer: REMOTE_PEER,
                        session: INITIAL_SESSION,
                        epoch: INITIAL_EPOCH,
                        player: Player::Two,
                    },
                ),
            ],
        )
        .map_err(|e| io::Error::other(format!("D3 owner: {e:?}")))?;
        Ok(Self {
            socket: Some(socket),
            local,
            owner,
            allowed_peer,
            peer: None,
            nonce: None,
            session: INITIAL_SESSION,
            epoch: INITIAL_EPOCH,
            last_peer: None,
            last_input: None,
            remote_direction: Direction::Still,
            last_wire_tick: 0,
            last_input_sequence: 0,
            last_snapshot_at: None,
            last_sent_snapshot_frame: None,
            rate_window: now,
            inputs_in_window: 0,
            joins_in_window: 0,
            stats: NetworkStats::default(),
        })
    }
    pub fn endpoint(&self) -> SocketAddrV4 {
        self.local
    }
    pub fn connected(&self) -> bool {
        self.peer.is_some()
    }
    pub fn session(&self) -> u64 {
        self.session
    }
    pub fn epoch(&self) -> u64 {
        self.epoch
    }
    pub fn snapshot(&self) -> Rc<Snapshot> {
        self.owner.snapshot()
    }
    /// One canonical Snapshot frame from the last complete OS submission.
    /// This is not a delivery acknowledgement. It may identify the prior epoch
    /// until a newer frame is submitted and remains historical after stop.
    pub fn last_sent_snapshot_frame(&self) -> Option<&[u8]> {
        self.last_sent_snapshot_frame.as_deref()
    }
    pub fn socket_closed(&self) -> bool {
        self.socket.is_none()
    }
    pub fn stop(&mut self) {
        self.owner.stop();
        self.socket.take();
        self.peer = None;
    }

    fn rate_reset(&mut self, now: Instant) {
        if now.saturating_duration_since(self.rate_window) >= Duration::from_secs(1) {
            self.rate_window = now;
            self.inputs_in_window = 0;
            self.joins_in_window = 0;
        }
    }
    /// The one authoritative reducer keeps its current generation/state
    /// during reconnect. Only the remote slot is re-bound and released.
    fn on_hello(&mut self, source: SocketAddrV4, nonce: u64, now: Instant) -> io::Result<()> {
        if self.joins_in_window >= MAX_JOIN_REPLIES_PER_SECOND {
            self.stats.rate_limited += 1;
            return Ok(());
        }
        if self.peer.is_some() && self.peer != Some(source) {
            self.stats.foreign += 1;
            return Ok(());
        }
        // A timed-out assignment cannot be revived by an old Join retry.
        // Only a strictly newer attempt nonce may enter a new Epoch.
        if self.peer.is_none() && self.nonce == Some(nonce) {
            self.stats.stale += 1;
            return Ok(());
        }
        let new_session = match self.nonce {
            None => false,
            Some(old) if nonce == old => false, // bounded retry of same attempt
            Some(old) if nonce > old => true,
            Some(_) => {
                self.stats.stale += 1;
                return Ok(());
            }
        };
        if new_session {
            let session = self
                .session
                .checked_add(1)
                .ok_or_else(|| err("session overflow"))?;
            let epoch = self
                .epoch
                .checked_add(1)
                .ok_or_else(|| err("epoch overflow"))?;
            self.owner
                .rotate_remote(
                    EndpointId(1002),
                    Binding {
                        peer: REMOTE_PEER,
                        session,
                        epoch,
                        player: Player::Two,
                    },
                )
                .map_err(|e| io::Error::other(format!("D3 rotate: {e:?}")))?;
            self.session = session;
            self.epoch = epoch;
            self.last_wire_tick = 0;
            self.last_input_sequence = 0;
            self.remote_direction = Direction::Still;
            self.last_input = None;
            self.stats.reconnects += 1;
        }
        self.peer = Some(source);
        self.nonce = Some(nonce);
        self.last_peer = Some(now);
        self.joins_in_window += 1;
        let bytes = encode(&Message::Welcome {
            peer: REMOTE_PEER,
            session: self.session,
            epoch: self.epoch,
            generation: HOST_GENERATION,
            tick: self.owner.snapshot().tick(),
            player: Player::Two,
        })
        .map_err(|e| io::Error::other(format!("welcome: {e:?}")))?;
        let Some(sock) = self.socket.as_ref() else {
            return Ok(());
        };
        if send(sock, source, &bytes)? {
            self.stats.sent += 1;
            self.stats.welcomes += 1;
            self.send_snapshot(now)?;
        }
        Ok(())
    }
    fn on_input(
        &mut self,
        source: SocketAddrV4,
        bytes: &[u8],
        msg: Message,
        now: Instant,
    ) -> io::Result<()> {
        let Message::HeldInput {
            peer,
            session,
            epoch,
            sequence,
            tick,
            direction,
        } = msg
        else {
            self.stats.rejected += 1;
            return Ok(());
        };
        if self.peer != Some(source) {
            self.stats.foreign += 1;
            return Ok(());
        }
        // Account for ALL well-formed attempts from the bound Socket,
        // including obsolete-session and wrong-peer replays.
        if self.inputs_in_window >= MAX_ACCEPTED_INPUT_PER_SECOND {
            self.stats.rate_limited += 1;
            return Ok(());
        }
        self.inputs_in_window += 1;
        if peer != REMOTE_PEER || session != self.session || epoch != self.epoch {
            self.stats.stale += 1;
            return Ok(());
        }
        let next_tick = self
            .owner
            .next_tick()
            .map_err(|e| io::Error::other(format!("tick: {e:?}")))?;
        // A lossy/100ms-reordered datagram cannot reliably specify the
        // exact *server* tick. Only accept recent strictly fenced client
        // timestamps, then stamp the *server's* next Tick for D1 admission.
        // D1's own exact-Tick and per-peer sequence checks stay unchanged.
        if tick <= self.last_wire_tick
            || tick > next_tick.saturating_add(1)
            || next_tick.saturating_sub(tick) > MAX_TICK_LAG
        {
            self.stats.stale += 1;
            return Ok(());
        }
        let canonical = encode(&Message::HeldInput {
            peer,
            session,
            epoch,
            sequence,
            tick: next_tick,
            direction,
        })
        .map_err(|e| io::Error::other(format!("input rebase: {e:?}")))?;
        if self.owner.admit(EndpointId(1002), &canonical).is_err() {
            self.stats.stale += 1;
            return Ok(());
        }
        debug_assert!(bytes.len() <= MAX_WIRE_BYTES);
        self.last_wire_tick = tick;
        self.last_input_sequence = sequence;
        self.remote_direction = direction;
        self.last_input = Some(now);
        self.last_peer = Some(now);
        self.stats.accepted_inputs += 1;
        Ok(())
    }
    pub fn poll(&mut self, now: Instant) -> io::Result<()> {
        if self.socket.is_none() {
            return Ok(());
        }
        self.rate_reset(now);
        self.check_peer_timeout(now)?;
        let mut buffer = [0u8; MAX_FRAME_STORAGE];
        let mut processed = 0;
        for _ in 0..MAX_RECEIVE {
            let recv = self.socket.as_ref().expect("bound").recv_from(&mut buffer);
            let (length, address) = match recv {
                Ok(data) => data,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            };
            processed += 1;
            self.stats.received += 1;
            if length > MAX_WIRE_BYTES {
                self.stats.oversize += 1;
                continue;
            }
            let source = match self.allowed_peer {
                Some(expected) if address == SocketAddr::V4(expected) => Some(expected),
                Some(_) => None,
                None => loopback(address),
            };
            let Some(source) = source else {
                self.stats.foreign += 1;
                continue;
            };
            let Ok(msg) = decode(&buffer[..length]) else {
                self.stats.malformed += 1;
                continue;
            };
            match msg {
                Message::Hello { nonce } => self.on_hello(source, nonce, now)?,
                Message::HeldInput { .. } => self.on_input(source, &buffer[..length], msg, now)?,
                Message::Leave {
                    peer,
                    session,
                    epoch,
                    sequence,
                } => {
                    // Advisory: old Leave may not delete the replacement.
                    if self.peer == Some(source)
                        && peer == REMOTE_PEER
                        && session == self.session
                        && epoch == self.epoch
                        && sequence != u64::MAX
                        && sequence > self.last_input_sequence
                    {
                        self.peer = None;
                        self.last_input_sequence = sequence;
                        self.last_input = None;
                        self.remote_direction = Direction::Still;
                        self.owner
                            .admit_local(Player::Two, Direction::Still)
                            .map_err(|e| io::Error::other(format!("leave release: {e:?}")))?;
                    } else {
                        self.stats.stale += 1;
                    }
                }
                _ => self.stats.rejected += 1,
            }
        }
        self.stats.max_received_one_poll = self.stats.max_received_one_poll.max(processed);
        Ok(())
    }
    fn check_peer_timeout(&mut self, now: Instant) -> io::Result<()> {
        if self.peer.is_some()
            && self
                .last_peer
                .is_some_and(|last| now.saturating_duration_since(last) >= PEER_TIMEOUT)
        {
            self.peer = None;
            self.last_input = None;
            self.remote_direction = Direction::Still;
            self.owner
                .admit_local(Player::Two, Direction::Still)
                .map_err(|e| io::Error::other(format!("peer timeout: {e:?}")))?;
            self.stats.timed_out += 1;
        }
        Ok(())
    }
    pub fn step(&mut self, now: Instant) -> io::Result<()> {
        self.check_peer_timeout(now)?;
        if self.remote_direction != Direction::Still
            && self
                .last_input
                .is_some_and(|last| now.saturating_duration_since(last) >= INPUT_EXPIRY)
        {
            self.owner
                .admit_local(Player::Two, Direction::Still)
                .map_err(|e| io::Error::other(format!("lost-release expiry: {e:?}")))?;
            self.remote_direction = Direction::Still;
            self.stats.expiry_release += 1;
        }
        let next = self
            .owner
            .next_tick()
            .map_err(|e| io::Error::other(format!("next tick: {e:?}")))?;
        self.owner
            .admit_local(
                Player::One,
                if next < 42 {
                    Direction::East
                } else {
                    Direction::Still
                },
            )
            .map_err(|e| io::Error::other(format!("local: {e:?}")))?;
        let result = self
            .owner
            .advance()
            .map_err(|e| io::Error::other(format!("C1 owner: {e:?}")))?;
        if result.tick().is_multiple_of(3) {
            self.send_snapshot(now)?;
        }
        Ok(())
    }
    fn send_snapshot(&mut self, now: Instant) -> io::Result<()> {
        // Join's immediate full snapshot and periodic publications share
        // one monotonic per-peer budget, never two independent 20Hz paths.
        if self
            .last_snapshot_at
            .is_some_and(|last| now.saturating_duration_since(last) < SNAPSHOT_MIN_INTERVAL)
        {
            return Ok(());
        }
        let (Some(peer), Some(sock)) = (self.peer, self.socket.as_ref()) else {
            return Ok(());
        };
        let bytes = self
            .owner
            .snapshot_frame(self.session, self.epoch)
            .map_err(|e| io::Error::other(format!("D3 snapshot encode: {e:?}")))?;
        if send(sock, peer, &bytes)? {
            self.stats.sent += 1;
            self.stats.snapshots_sent += 1;
            self.last_snapshot_at = Some(now);
            self.last_sent_snapshot_frame = Some(bytes);
        }
        Ok(())
    }
}

pub struct HeadlessNetworkClient {
    socket: Option<UdpSocket>,
    local: SocketAddrV4,
    expected_proxy: SocketAddrV4,
    initial_proxy_selectable: bool,
    expected_session: u64,
    expected_epoch: u64,
    nonce: u64,
    hello_count: u32,
    last_hello: Option<Instant>,
    replica: Option<ReplicaFixture>,
    latest: Option<Rc<WireSnapshot>>,
    last_input_at: Option<Instant>,
    next_sequence: u64,
    silence: bool,
    direction: Direction,
    pub stats: NetworkStats,
}
impl HeadlessNetworkClient {
    pub fn bind(
        expected_proxy: SocketAddrV4,
        session: u64,
        epoch: u64,
        nonce: u64,
    ) -> io::Result<Self> {
        if expected_proxy.ip() != &Ipv4Addr::LOCALHOST
            || expected_proxy.port() == 0
            || session == 0
            || epoch == 0
            || nonce == 0
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "trusted D3 client markers",
            ));
        }
        let (socket, local) = bind()?;
        Ok(Self::from_bound(
            socket,
            local,
            expected_proxy,
            session,
            epoch,
            nonce,
            true,
        ))
    }
    /// Explicit private-application route with an immutable exact host.
    pub fn bind_on(
        local: SocketAddrV4,
        exact_host: SocketAddrV4,
        session: u64,
        epoch: u64,
        nonce: u64,
    ) -> io::Result<Self> {
        validate_selected_pair(local, exact_host)?;
        if session == 0 || epoch == 0 || nonce == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "trusted D4 client markers",
            ));
        }
        let (socket, local) = bind_selected(local)?;
        Ok(Self::from_bound(
            socket, local, exact_host, session, epoch, nonce, false,
        ))
    }
    fn from_bound(
        socket: UdpSocket,
        local: SocketAddrV4,
        expected_proxy: SocketAddrV4,
        session: u64,
        epoch: u64,
        nonce: u64,
        initial_proxy_selectable: bool,
    ) -> Self {
        Self {
            socket: Some(socket),
            local,
            expected_proxy,
            initial_proxy_selectable,
            expected_session: session,
            expected_epoch: epoch,
            nonce,
            hello_count: 0,
            last_hello: None,
            replica: None,
            latest: None,
            last_input_at: None,
            next_sequence: 0,
            silence: false,
            direction: Direction::West,
            stats: NetworkStats::default(),
        }
    }
    /// The client Socket may bind before the separately owned in-process
    /// fault relay gets its ephemeral port. This selection is legal only
    /// before any Hello or trusted Replica; it is not a remote retarget API.
    pub fn select_initial_proxy(&mut self, proxy: SocketAddrV4) -> io::Result<()> {
        if !self.initial_proxy_selectable
            || proxy.ip() != &Ipv4Addr::LOCALHOST
            || proxy.port() == 0
            || self.hello_count != 0
            || self.replica.is_some()
            || self.latest.is_some()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "D3 initial proxy select",
            ));
        }
        self.expected_proxy = proxy;
        Ok(())
    }
    pub fn endpoint(&self) -> SocketAddrV4 {
        self.local
    }
    pub fn bound_host(&self) -> SocketAddrV4 {
        self.expected_proxy
    }
    pub fn socket_closed(&self) -> bool {
        self.socket.is_none()
    }
    pub fn snapshot(&self) -> Option<Rc<WireSnapshot>> {
        self.latest.as_ref().map(Rc::clone)
    }
    pub fn connected(&self) -> bool {
        self.replica.is_some()
    }
    pub fn set_silence(&mut self, enabled: bool) {
        self.silence = enabled;
    }
    pub fn set_direction(&mut self, direction: Direction) {
        self.direction = direction;
    }
    pub fn stop(&mut self) {
        self.socket.take();
    }
    /// Explicit D3 negative fixture: send obsolete Hello/Input/Leave from
    /// the same currently bound Client Socket after a newer session exists.
    /// The Host must reject all three without deleting its replacement.
    pub fn send_old_marker_qa(&mut self) -> io::Result<()> {
        let session = self
            .expected_session
            .checked_sub(1)
            .filter(|id| *id != 0)
            .ok_or_else(|| err("old session absent"))?;
        let epoch = self
            .expected_epoch
            .checked_sub(1)
            .filter(|id| *id != 0)
            .ok_or_else(|| err("old epoch absent"))?;
        let nonce = self
            .nonce
            .checked_sub(1)
            .filter(|id| *id != 0)
            .ok_or_else(|| err("old nonce absent"))?;
        let Some(sock) = self.socket.as_ref() else {
            return Err(err("client stopped"));
        };
        let stale = [
            Message::Hello { nonce },
            Message::HeldInput {
                peer: REMOTE_PEER,
                session,
                epoch,
                sequence: 42,
                tick: self.latest.as_ref().map_or(1, |s| s.tick.saturating_add(1)),
                direction: Direction::West,
            },
            Message::Leave {
                peer: REMOTE_PEER,
                session,
                epoch,
                sequence: 43,
            },
        ];
        for message in stale {
            let bytes =
                encode(&message).map_err(|e| io::Error::other(format!("old fixture: {e:?}")))?;
            if send(sock, self.expected_proxy, &bytes)? {
                self.stats.sent += 1;
            }
        }
        Ok(())
    }

    pub fn reconnect(&mut self, session: u64, epoch: u64, nonce: u64) -> io::Result<()> {
        if session <= self.expected_session || epoch <= self.expected_epoch || nonce <= self.nonce {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "must rotate Session/Epoch/nonce",
            ));
        }
        self.expected_session = session;
        self.expected_epoch = epoch;
        self.nonce = nonce;
        self.hello_count = 0;
        self.last_hello = None;
        self.replica = None;
        self.latest = None;
        self.next_sequence = 0;
        self.last_input_at = None;
        Ok(())
    }
    pub fn poll(&mut self, now: Instant) -> io::Result<()> {
        let Some(sock) = self.socket.as_ref() else {
            return Ok(());
        };
        if self.replica.is_none()
            && self.hello_count < MAX_JOIN_RETRIES
            && self
                .last_hello
                .is_none_or(|last| now.saturating_duration_since(last) >= JOIN_RETRY)
        {
            let bytes = encode(&Message::Hello { nonce: self.nonce })
                .map_err(|e| io::Error::other(format!("hello: {e:?}")))?;
            if send(sock, self.expected_proxy, &bytes)? {
                self.hello_count += 1;
                self.stats.joins += 1;
                self.stats.sent += 1;
                self.last_hello = Some(now);
            }
        }
        let mut buf = [0u8; MAX_FRAME_STORAGE];
        let mut handled = 0;
        for _ in 0..MAX_RECEIVE {
            let (len, from) = match sock.recv_from(&mut buf) {
                Ok(item) => item,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => break,
                Err(e) => return Err(e),
            };
            handled += 1;
            self.stats.received += 1;
            if len > MAX_WIRE_BYTES {
                self.stats.oversize += 1;
                continue;
            }
            if from != SocketAddr::V4(self.expected_proxy) {
                self.stats.foreign += 1;
                continue;
            }
            let Ok(msg) = decode(&buf[..len]) else {
                self.stats.malformed += 1;
                continue;
            };
            match msg {
                Message::Welcome {
                    peer,
                    session,
                    epoch,
                    generation,
                    player,
                    ..
                } => {
                    if peer != REMOTE_PEER
                        || session != self.expected_session
                        || epoch != self.expected_epoch
                        || generation != HOST_GENERATION
                        || player != Player::Two
                        || self.replica.is_some()
                    {
                        self.stats.stale += 1;
                        continue;
                    }
                    self.replica = Some(
                        ReplicaFixture::new(EndpointId(9001), session, epoch, generation)
                            .map_err(|e| io::Error::other(format!("replica: {e:?}")))?,
                    );
                    self.stats.welcomes += 1;
                }
                Message::Snapshot(_) => {
                    let Some(replica) = self.replica.as_mut() else {
                        self.stats.rejected += 1;
                        continue;
                    };
                    let Ok(latest) = replica.admit_snapshot(EndpointId(9001), &buf[..len]) else {
                        self.stats.stale += 1;
                        continue;
                    };
                    self.latest = Some(latest);
                    self.stats.snapshots_received += 1;
                }
                _ => self.stats.rejected += 1,
            }
        }
        self.stats.max_received_one_poll = self.stats.max_received_one_poll.max(handled);
        if !self.silence
            && self.replica.is_some()
            && let Some(latest) = self.latest.as_ref()
            && self
                .last_input_at
                .is_none_or(|last| now.saturating_duration_since(last) >= Duration::from_millis(50))
        {
            let Some(sequence) = self.next_sequence.checked_add(1) else {
                return Err(err("client sequence exhausted"));
            };
            let Some(tick) = latest.tick.checked_add(1) else {
                return Err(err("client tick exhausted"));
            };
            let payload = encode(&Message::HeldInput {
                peer: REMOTE_PEER,
                session: self.expected_session,
                epoch: self.expected_epoch,
                sequence,
                tick,
                direction: self.direction,
            })
            .map_err(|e| io::Error::other(format!("input encode: {e:?}")))?;
            if send(sock, self.expected_proxy, &payload)? {
                self.stats.sent += 1;
                self.next_sequence = sequence;
                self.last_input_at = Some(now);
                self.stats.accepted_inputs += 1; // locally emitted, not Host admission
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d3_stale_same_session_leave_cannot_unbind_newer_validated_input() {
        let now = Instant::now();
        let mut host = HeadlessNetworkHost::bind(now).unwrap();
        let peer = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        peer.send_to(
            &encode(&Message::Hello { nonce: 100 }).unwrap(),
            host.endpoint(),
        )
        .unwrap();
        host.poll(now).unwrap();
        assert!(host.connected());
        let moving = encode(&Message::HeldInput {
            peer: REMOTE_PEER,
            session: INITIAL_SESSION,
            epoch: INITIAL_EPOCH,
            sequence: 10,
            tick: 1,
            direction: Direction::West,
        })
        .unwrap();
        peer.send_to(&moving, host.endpoint()).unwrap();
        host.poll(now + Duration::from_millis(1)).unwrap();
        assert_eq!(host.stats.accepted_inputs, 1);
        assert_eq!(host.last_input_sequence, 10);
        assert_eq!(host.owner.pending_count(), 1);
        let old = host.snapshot();
        let stale = encode(&Message::Leave {
            peer: REMOTE_PEER,
            session: INITIAL_SESSION,
            epoch: INITIAL_EPOCH,
            sequence: 9,
        })
        .unwrap();
        peer.send_to(&stale, host.endpoint()).unwrap();
        host.poll(now + Duration::from_millis(2)).unwrap();
        assert!(
            host.connected(),
            "replayed current-session Leave must not release live binding"
        );
        assert_eq!(host.last_input_sequence, 10);
        assert_eq!(host.owner.pending_count(), 1);
        assert!(host.stats.stale >= 1);
        assert!(Rc::ptr_eq(&old, &host.snapshot()));
        let fresh = encode(&Message::Leave {
            peer: REMOTE_PEER,
            session: INITIAL_SESSION,
            epoch: INITIAL_EPOCH,
            sequence: 11,
        })
        .unwrap();
        peer.send_to(&fresh, host.endpoint()).unwrap();
        host.poll(now + Duration::from_millis(3)).unwrap();
        assert!(!host.connected());
        assert_eq!(host.last_input_sequence, 11);
        assert_eq!(host.remote_direction, Direction::Still);
        assert!(Rc::ptr_eq(&old, &host.snapshot()));
        host.step(now + Duration::from_millis(4)).unwrap();
        assert_eq!(host.snapshot().held()[1], Direction::Still);
        host.stop();
    }

    #[test]
    fn d3_join_reply_and_periodic_snapshots_share_a_single_20hz_budget() {
        let now = Instant::now();
        let mut host = HeadlessNetworkHost::bind(now).unwrap();
        let peer = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        peer.set_read_timeout(Some(Duration::from_millis(150)))
            .unwrap();
        let hello = encode(&Message::Hello { nonce: 100 }).unwrap();
        peer.send_to(&hello, host.endpoint()).unwrap();
        host.poll(now).unwrap();
        assert_eq!(host.stats.welcomes, 1);
        assert_eq!(host.stats.snapshots_sent, 1);
        let mut recv = [0u8; MAX_FRAME_STORAGE];
        let kinds: Vec<_> = (0..2)
            .map(|_| {
                let (len, from) = peer.recv_from(&mut recv).unwrap();
                assert_eq!(loopback(from), Some(host.endpoint()));
                decode(&recv[..len]).unwrap()
            })
            .collect();
        assert!(kinds.iter().any(|m| matches!(m, Message::Welcome { .. })));
        assert!(kinds.iter().any(|m| matches!(m, Message::Snapshot(_))));
        peer.send_to(&hello, host.endpoint()).unwrap();
        host.poll(now + Duration::from_millis(1)).unwrap();
        assert_eq!(host.stats.welcomes, 2);
        assert_eq!(
            host.stats.snapshots_sent, 1,
            "join retry must not add another full frame before interval"
        );
        for tick in 1..=90 {
            host.step(now + Duration::from_millis(tick * 10)).unwrap();
        }
        assert!(host.stats.snapshots_sent > 1);
        assert!(
            host.stats.snapshots_sent <= u64::from(MAX_SNAPSHOT_PER_SECOND),
            "the combined Join+periodic stream must not exceed the 20/s budget within 900ms"
        );
        host.stop();
    }

    #[test]
    fn d3_expired_epoch_inputs_from_bound_socket_consume_same_30_attempt_budget() {
        let now = Instant::now();
        let mut host = HeadlessNetworkHost::bind(now).unwrap();
        let peer = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        peer.send_to(
            &encode(&Message::Hello { nonce: 100 }).unwrap(),
            host.endpoint(),
        )
        .unwrap();
        host.poll(now).unwrap();
        peer.send_to(
            &encode(&Message::Hello { nonce: 101 }).unwrap(),
            host.endpoint(),
        )
        .unwrap();
        host.poll(now + Duration::from_millis(1)).unwrap();
        assert_eq!(host.session(), INITIAL_SESSION + 1);
        let old = host.snapshot();
        let staged = host.owner.pending_count();
        for seq in 1..=u64::from(MAX_ACCEPTED_INPUT_PER_SECOND + 10) {
            peer.send_to(
                &encode(&Message::HeldInput {
                    peer: REMOTE_PEER,
                    session: INITIAL_SESSION,
                    epoch: INITIAL_EPOCH,
                    sequence: seq,
                    tick: 1,
                    direction: Direction::West,
                })
                .unwrap(),
                host.endpoint(),
            )
            .unwrap();
        }
        for _ in 0..6 {
            host.poll(now + Duration::from_millis(2)).unwrap();
        }
        assert_eq!(host.inputs_in_window, MAX_ACCEPTED_INPUT_PER_SECOND);
        assert!(host.stats.rate_limited >= 10);
        assert_eq!(host.stats.accepted_inputs, 0);
        assert!(host.stats.stale >= u64::from(MAX_ACCEPTED_INPUT_PER_SECOND));
        assert!(host.connected());
        assert_eq!(host.owner.pending_count(), staged);
        assert!(Rc::ptr_eq(&old, &host.snapshot()));
        host.stop();
    }

    #[test]
    fn d3_lost_release_expires_after_250ms_without_replaying_or_restarting_c1() {
        let now = Instant::now();
        let mut host = HeadlessNetworkHost::bind(now).unwrap();
        host.owner
            .admit_local(Player::Two, Direction::West)
            .unwrap();
        host.remote_direction = Direction::West;
        host.last_input = Some(now);
        host.step(now + Duration::from_millis(249)).unwrap();
        assert_eq!(host.snapshot().held()[1], Direction::West);
        let before = host.snapshot();
        host.step(now + INPUT_EXPIRY).unwrap();
        assert_eq!(host.snapshot().held()[1], Direction::Still);
        assert_eq!(host.stats.expiry_release, 1);
        assert_eq!(host.snapshot().positions()[1], before.positions()[1]);
        assert_eq!(host.snapshot().generation(), before.generation());
        host.step(now + Duration::from_millis(500)).unwrap();
        assert_eq!(host.stats.expiry_release, 1);
        host.stop();
        assert!(host.socket_closed());
    }

    #[test]
    fn d3_peer_timeout_3s_and_new_epoch_stale_packet_fenced_with_single_owner() {
        let start = Instant::now();
        let mut host = HeadlessNetworkHost::bind(start).unwrap();
        let client = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        client
            .send_to(
                &encode(&Message::Hello { nonce: 100 }).unwrap(),
                host.endpoint(),
            )
            .unwrap();
        host.poll(start).unwrap();
        assert!(host.connected());
        let generation = host.snapshot().generation();
        host.poll(start + PEER_TIMEOUT + Duration::from_millis(1))
            .unwrap();
        assert!(!host.connected());
        assert_eq!(host.stats.timed_out, 1);
        client
            .send_to(
                &encode(&Message::Hello { nonce: 101 }).unwrap(),
                host.endpoint(),
            )
            .unwrap();
        host.poll(start + PEER_TIMEOUT + Duration::from_millis(20))
            .unwrap();
        assert!(host.connected());
        assert_eq!(host.session(), INITIAL_SESSION + 1);
        assert_eq!(host.epoch(), INITIAL_EPOCH + 1);
        assert_eq!(host.stats.reconnects, 1);
        assert_eq!(host.snapshot().generation(), generation);
        let old = encode(&Message::HeldInput {
            peer: REMOTE_PEER,
            session: INITIAL_SESSION,
            epoch: INITIAL_EPOCH,
            sequence: 1,
            tick: 1,
            direction: Direction::West,
        })
        .unwrap();
        let before = host.snapshot();
        client.send_to(&old, host.endpoint()).unwrap();
        host.poll(start + PEER_TIMEOUT + Duration::from_millis(21))
            .unwrap();
        assert!(host.stats.stale >= 1);
        assert!(Rc::ptr_eq(&before, &host.snapshot()));
        // A stale Leave must not delete the replacement.
        let leave = encode(&Message::Leave {
            peer: REMOTE_PEER,
            session: INITIAL_SESSION,
            epoch: INITIAL_EPOCH,
            sequence: 2,
        })
        .unwrap();
        client.send_to(&leave, host.endpoint()).unwrap();
        host.poll(start + PEER_TIMEOUT + Duration::from_millis(22))
            .unwrap();
        assert!(host.connected());
        host.stop();
    }

    #[test]
    fn d3_remote_flood_rate_limit_does_not_mutate_rejected_c1_publication() {
        let now = Instant::now();
        let mut host = HeadlessNetworkHost::bind(now).unwrap();
        let client = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        client
            .send_to(
                &encode(&Message::Hello { nonce: 100 }).unwrap(),
                host.endpoint(),
            )
            .unwrap();
        host.poll(now).unwrap();
        let old = host.snapshot();
        for i in 1..=60 {
            let payload = encode(&Message::HeldInput {
                peer: REMOTE_PEER,
                session: INITIAL_SESSION,
                epoch: INITIAL_EPOCH,
                sequence: i,
                tick: 1,
                direction: Direction::West,
            })
            .unwrap();
            client.send_to(&payload, host.endpoint()).unwrap();
        }
        for _ in 0..10 {
            host.poll(now + Duration::from_millis(1)).unwrap();
        }
        assert!(host.stats.rate_limited > 0);
        assert!(host.stats.accepted_inputs <= MAX_ACCEPTED_INPUT_PER_SECOND as u64);
        assert!(host.stats.max_received_one_poll <= MAX_RECEIVE);
        assert!(Rc::ptr_eq(&old, &host.snapshot()));
        host.stop();
    }

    #[test]
    fn d3_old_nonce_cannot_reopen_expired_assignment_but_new_epoch_can() {
        let now = Instant::now();
        let mut host = HeadlessNetworkHost::bind(now).unwrap();
        let client = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let first = encode(&Message::Hello { nonce: 100 }).unwrap();
        client.send_to(&first, host.endpoint()).unwrap();
        host.poll(now).unwrap();
        assert!(host.connected());
        host.poll(now + PEER_TIMEOUT + Duration::from_millis(1))
            .unwrap();
        assert!(!host.connected());
        let old = host.snapshot();
        client.send_to(&first, host.endpoint()).unwrap();
        host.poll(now + PEER_TIMEOUT + Duration::from_millis(2))
            .unwrap();
        assert!(!host.connected());
        assert_eq!(host.stats.stale, 1);
        assert!(Rc::ptr_eq(&old, &host.snapshot()));
        client
            .send_to(
                &encode(&Message::Hello { nonce: 101 }).unwrap(),
                host.endpoint(),
            )
            .unwrap();
        host.poll(now + PEER_TIMEOUT + Duration::from_millis(3))
            .unwrap();
        assert!(host.connected());
        assert_eq!(host.session(), INITIAL_SESSION + 1);
        assert_eq!(host.epoch(), INITIAL_EPOCH + 1);
        assert_eq!(host.stats.reconnects, 1);
        host.stop();
    }

    #[test]
    fn d3_oversized_and_foreign_peer_frames_never_touch_game_authority() {
        let now = Instant::now();
        let mut host = HeadlessNetworkHost::bind(now).unwrap();
        let actual = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let foreign = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        actual
            .send_to(
                &encode(&Message::Hello { nonce: 100 }).unwrap(),
                host.endpoint(),
            )
            .unwrap();
        host.poll(now).unwrap();
        assert!(host.connected());
        let old = host.snapshot();
        actual
            .send_to(&[0u8; MAX_WIRE_BYTES + 1], host.endpoint())
            .unwrap();
        foreign
            .send_to(
                &encode(&Message::HeldInput {
                    peer: REMOTE_PEER,
                    session: INITIAL_SESSION,
                    epoch: INITIAL_EPOCH,
                    sequence: 1,
                    tick: 1,
                    direction: Direction::West,
                })
                .unwrap(),
                host.endpoint(),
            )
            .unwrap();
        host.poll(now + Duration::from_millis(1)).unwrap();
        assert_eq!(host.stats.oversize, 1);
        assert_eq!(host.stats.foreign, 1);
        assert_eq!(host.stats.accepted_inputs, 0);
        assert!(Rc::ptr_eq(&old, &host.snapshot()));
        host.stop();
    }

    #[test]
    fn d3_old_snapshot_after_new_epoch_welcome_cannot_replace_replica() {
        let now = Instant::now();
        let host_sock = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let SocketAddr::V4(source) = host_sock.local_addr().unwrap() else {
            panic!()
        };
        let mut client =
            HeadlessNetworkClient::bind(source, INITIAL_SESSION + 1, INITIAL_EPOCH + 1, 101)
                .unwrap();
        let state = HeadlessNetworkHost::bind(now).unwrap();
        let welcome = encode(&Message::Welcome {
            peer: REMOTE_PEER,
            session: INITIAL_SESSION + 1,
            epoch: INITIAL_EPOCH + 1,
            generation: HOST_GENERATION,
            tick: 0,
            player: Player::Two,
        })
        .unwrap();
        host_sock.send_to(&welcome, client.endpoint()).unwrap();
        host_sock
            .send_to(
                &state
                    .owner
                    .snapshot_frame(INITIAL_SESSION, INITIAL_EPOCH)
                    .unwrap(),
                client.endpoint(),
            )
            .unwrap();
        client.poll(now).unwrap();
        assert!(client.connected());
        assert!(client.snapshot().is_none());
        assert_eq!(client.stats.stale, 1);
        let new = state
            .owner
            .snapshot_frame(INITIAL_SESSION + 1, INITIAL_EPOCH + 1)
            .unwrap();
        host_sock.send_to(&new, client.endpoint()).unwrap();
        client.poll(now + Duration::from_millis(2)).unwrap();
        let fresh = client.snapshot().unwrap();
        assert_eq!(fresh.session, INITIAL_SESSION + 1);
        assert_eq!(fresh.epoch, INITIAL_EPOCH + 1);
        host_sock.send_to(&new, client.endpoint()).unwrap();
        client.poll(now + Duration::from_millis(3)).unwrap();
        assert!(Rc::ptr_eq(&fresh, &client.snapshot().unwrap()));
        client.stop();
    }
    fn d4_reserved_loopback() -> (UdpSocket, SocketAddrV4) {
        let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
        let address = loopback(socket.local_addr().unwrap()).unwrap();
        (socket, address)
    }

    #[test]
    fn d4_selected_configuration_preserves_loopback_and_proxy_defaults() {
        let now = Instant::now();
        let local = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 32001);
        let peer = SocketAddrV4::new(Ipv4Addr::LOCALHOST, 32002);
        for invalid in [
            SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 32003),
            SocketAddrV4::new(Ipv4Addr::BROADCAST, 32003),
            SocketAddrV4::new(Ipv4Addr::new(239, 1, 2, 3), 32003),
            SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0),
        ] {
            assert_eq!(
                HeadlessNetworkHost::bind_on(now, invalid, peer)
                    .err()
                    .unwrap()
                    .kind(),
                io::ErrorKind::InvalidInput,
            );
            assert_eq!(
                HeadlessNetworkHost::bind_on(now, local, invalid)
                    .err()
                    .unwrap()
                    .kind(),
                io::ErrorKind::InvalidInput,
            );
            assert_eq!(
                HeadlessNetworkClient::bind_on(invalid, peer, INITIAL_SESSION, INITIAL_EPOCH, 100)
                    .err()
                    .unwrap()
                    .kind(),
                io::ErrorKind::InvalidInput,
            );
            assert_eq!(
                HeadlessNetworkClient::bind_on(local, invalid, INITIAL_SESSION, INITIAL_EPOCH, 100)
                    .err()
                    .unwrap()
                    .kind(),
                io::ErrorKind::InvalidInput,
            );
        }
        assert!(HeadlessNetworkHost::bind_on(now, local, local).is_err());
        assert!(
            HeadlessNetworkClient::bind_on(local, local, INITIAL_SESSION, INITIAL_EPOCH, 100)
                .is_err()
        );
        for (session, epoch, nonce) in [(0, 1, 100), (7101, 0, 100), (7101, 1, 0)] {
            assert_eq!(
                HeadlessNetworkClient::bind_on(local, peer, session, epoch, nonce)
                    .err()
                    .unwrap()
                    .kind(),
                io::ErrorKind::InvalidInput,
            );
        }
        let mut host = HeadlessNetworkHost::bind(now).unwrap();
        assert_eq!(host.endpoint().ip(), &Ipv4Addr::LOCALHOST);
        assert_ne!(host.endpoint().port(), 0);
        assert!(host.last_sent_snapshot_frame().is_none());
        assert!(
            HeadlessNetworkClient::bind(
                SocketAddrV4::new(Ipv4Addr::new(192, 0, 2, 1), 32002),
                INITIAL_SESSION,
                INITIAL_EPOCH,
                100,
            )
            .is_err()
        );
        let mut legacy =
            HeadlessNetworkClient::bind(peer, INITIAL_SESSION, INITIAL_EPOCH, 100).unwrap();
        assert_eq!(legacy.endpoint().ip(), &Ipv4Addr::LOCALHOST);
        legacy.select_initial_proxy(host.endpoint()).unwrap();
        assert_eq!(legacy.bound_host(), host.endpoint());
        legacy.stop();
        host.stop();
    }

    #[test]
    fn d4_selected_peer_is_fenced_before_hello_and_decode_after_timeout() {
        let now = Instant::now();
        let (peer, peer_address) = d4_reserved_loopback();
        let (reservation, local) = d4_reserved_loopback();
        drop(reservation);
        let mut host = HeadlessNetworkHost::bind_on(now, local, peer_address).unwrap();
        let (foreign, _) = d4_reserved_loopback();
        let before = host.snapshot();
        foreign
            .send_to(&encode(&Message::Hello { nonce: 999 }).unwrap(), local)
            .unwrap();
        foreign.send_to(b"not a protocol frame", local).unwrap();
        host.poll(now).unwrap();
        assert!(!host.connected());
        assert_eq!(host.stats.foreign, 2);
        assert_eq!(host.stats.malformed, 0);
        assert_eq!(host.stats.welcomes, 0);
        assert!(host.last_sent_snapshot_frame().is_none());
        assert!(Rc::ptr_eq(&before, &host.snapshot()));

        peer.send_to(&encode(&Message::Hello { nonce: 100 }).unwrap(), local)
            .unwrap();
        host.poll(now).unwrap();
        assert!(host.connected());
        let expired = now + PEER_TIMEOUT + Duration::from_millis(1);
        host.poll(expired).unwrap();
        assert!(!host.connected());
        foreign
            .send_to(&encode(&Message::Hello { nonce: 1000 }).unwrap(), local)
            .unwrap();
        host.poll(expired + Duration::from_millis(1)).unwrap();
        assert!(!host.connected());
        assert_eq!(host.nonce, Some(100));
        assert_eq!(host.session(), INITIAL_SESSION);
        assert_eq!(host.stats.foreign, 3);
        assert!(Rc::ptr_eq(&before, &host.snapshot()));

        peer.send_to(&encode(&Message::Hello { nonce: 101 }).unwrap(), local)
            .unwrap();
        host.poll(expired + Duration::from_millis(20)).unwrap();
        assert!(host.connected());
        assert_eq!(host.session(), INITIAL_SESSION + 1);
        assert_eq!(host.epoch(), INITIAL_EPOCH + 1);
        assert!(Rc::ptr_eq(&before, &host.snapshot()));
        host.stop();
        assert!(host.socket_closed());
        drop(UdpSocket::bind(local).unwrap());
    }

    #[test]
    fn d4_selected_publication_matches_replica_and_same_peer_reconnect() {
        fn assert_publication(
            host: &HeadlessNetworkHost,
            client: &HeadlessNetworkClient,
        ) -> Vec<u8> {
            let bytes = host.last_sent_snapshot_frame().unwrap();
            assert!(bytes.len() <= MAX_WIRE_BYTES);
            let Message::Snapshot(sent) = decode(bytes).unwrap() else {
                panic!("Snapshot frame");
            };
            assert_eq!(&sent, client.snapshot().unwrap().as_ref());
            assert_eq!(encode(&Message::Snapshot(sent)).unwrap(), bytes);
            bytes.to_vec()
        }

        let now = Instant::now();
        let (peer_reservation, peer_address) = d4_reserved_loopback();
        let (host_reservation, local) = d4_reserved_loopback();
        drop(host_reservation);
        let mut host = HeadlessNetworkHost::bind_on(now, local, peer_address).unwrap();
        drop(peer_reservation);
        let mut client = HeadlessNetworkClient::bind_on(
            peer_address,
            local,
            INITIAL_SESSION,
            INITIAL_EPOCH,
            100,
        )
        .unwrap();
        assert!(client.select_initial_proxy(local).is_err());
        assert_eq!(client.bound_host(), local);
        assert!(host.last_sent_snapshot_frame().is_none());
        client.poll(now).unwrap();
        host.poll(now).unwrap();
        client.poll(now).unwrap();
        let initial_frame = assert_publication(&host, &client);
        host.poll(now).unwrap();
        let before_input = host.snapshot();
        host.step(now + Duration::from_millis(10)).unwrap();
        assert_ne!(host.snapshot().positions()[1], before_input.positions()[1]);
        assert_eq!(host.last_sent_snapshot_frame().unwrap(), initial_frame);
        host.step(now + Duration::from_millis(60)).unwrap();
        host.step(now + Duration::from_millis(80)).unwrap();
        client.poll(now + Duration::from_millis(80)).unwrap();
        let old_frame = assert_publication(&host, &client);
        assert_ne!(old_frame, initial_frame);
        let old_replica = client.snapshot().unwrap();
        let old_generation = host.snapshot().generation();

        let expired = now + PEER_TIMEOUT + Duration::from_millis(100);
        host.poll(expired).unwrap();
        assert!(!host.connected());
        assert_eq!(host.stats.timed_out, 1);
        let before_rotation = host.snapshot();
        let rejoined = expired + Duration::from_millis(100);
        client
            .reconnect(INITIAL_SESSION + 1, INITIAL_EPOCH + 1, 101)
            .unwrap();
        assert!(client.snapshot().is_none());
        client.set_direction(Direction::Still);
        client.poll(rejoined).unwrap();
        host.poll(rejoined).unwrap();
        assert!(Rc::ptr_eq(&before_rotation, &host.snapshot()));
        client.poll(rejoined).unwrap();
        assert_publication(&host, &client);
        host.poll(rejoined).unwrap();
        host.step(rejoined + Duration::from_millis(10)).unwrap();
        host.step(rejoined + Duration::from_millis(30)).unwrap();
        host.step(rejoined + Duration::from_millis(60)).unwrap();
        client.poll(rejoined + Duration::from_millis(60)).unwrap();
        let final_frame = assert_publication(&host, &client);
        let fresh = client.snapshot().unwrap();
        assert_eq!(fresh.session, INITIAL_SESSION + 1);
        assert_eq!(fresh.epoch, INITIAL_EPOCH + 1);
        assert_eq!(fresh.generation, old_generation);
        assert!(fresh.tick > old_replica.tick);
        assert_eq!(old_replica.session, INITIAL_SESSION);
        assert_eq!(old_replica.epoch, INITIAL_EPOCH);
        assert_eq!(host.stats.reconnects, 1);
        assert!(host.stats.accepted_inputs >= 2);
        client.stop();
        host.stop();
        assert!(client.socket_closed());
        assert!(host.socket_closed());
        assert_eq!(host.last_sent_snapshot_frame().unwrap(), final_frame);
        drop(UdpSocket::bind(peer_address).unwrap());
        drop(UdpSocket::bind(local).unwrap());
    }

    #[test]
    fn d3_client_expected_new_session_rejects_old_welcome_and_snapshot() {
        let now = Instant::now();
        let host = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let SocketAddr::V4(host_addr) = host.local_addr().unwrap() else {
            panic!()
        };
        let mut client =
            HeadlessNetworkClient::bind(host_addr, INITIAL_SESSION, INITIAL_EPOCH, 100).unwrap();
        assert!(client.snapshot().is_none());
        assert!(
            client
                .reconnect(INITIAL_SESSION, INITIAL_EPOCH, 100)
                .is_err()
        );
        client
            .reconnect(INITIAL_SESSION + 1, INITIAL_EPOCH + 1, 101)
            .unwrap();
        let old = encode(&Message::Welcome {
            peer: REMOTE_PEER,
            session: INITIAL_SESSION,
            epoch: INITIAL_EPOCH,
            generation: HOST_GENERATION,
            tick: 0,
            player: Player::Two,
        })
        .unwrap();
        host.send_to(&old, client.endpoint()).unwrap();
        client.poll(now).unwrap();
        assert!(!client.connected());
        assert!(client.snapshot().is_none());
        assert!(client.stats.stale >= 1);
        client.stop();
    }
}
