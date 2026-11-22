//! YT-272 / D2 — Private loopback-only std UDP application adapter.
//!
//! Each application process owns ONE caller-thread GameHost plus either a
//! NetworkHost (authoritative C1 Simulation via D1 HostFixture) or a
//! NetworkClient (read-only D1 ReplicaFixture). No per-Host process/thread,
//! public Transport trait, listener exposure, firewall change or D3 fault
//! injector is added.

use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    rc::Rc,
    time::{Duration, Instant},
};
use yetlet_composable_host_research::{
    protocol::{
        Binding, EndpointId, HostFixture, MAX_WIRE_BYTES, Message, ReplicaFixture, WireSnapshot,
        decode, encode,
    },
    simulation::{Command, Direction, Player, Snapshot},
};

pub const BIND_IP: Ipv4Addr = Ipv4Addr::LOCALHOST;
pub const DATAGRAM_BUFFER: usize = MAX_WIRE_BYTES + 1;
pub const MAX_RECEIVE_PER_WAKE: usize = 8;
pub const REMOTE_PEER: u64 = 22;
pub const SHARED_SESSION: u64 = 7101;
pub const EPOCH: u64 = 1;
pub const GENERATION: u64 = 1;
pub const HELLO_NONCE: u64 = 918_627;
pub const SNAPSHOT_TICK_INTERVAL: u64 = 3; // 20/s at 60 Hz
pub const MAX_CLIENT_HELLO_ATTEMPTS: u32 = 3;

pub fn loopback_v4(port: u16) -> SocketAddrV4 {
    SocketAddrV4::new(BIND_IP, port)
}

#[derive(Default, Debug, Clone)]
pub struct Counters {
    pub rx: u64,
    pub tx: u64,
    pub rx_accepted: u64,
    pub rx_rejected: u64,
    pub tx_snapshots: u64,
    pub tx_inputs: u64,
    pub tx_welcomes: u64,
    pub tx_hellos: u64,
    pub rx_oversize: u64,
}

fn bind_loopback() -> io::Result<(UdpSocket, SocketAddrV4)> {
    let socket = UdpSocket::bind(loopback_v4(0))?;
    socket.set_nonblocking(true)?;
    let SocketAddr::V4(address) = socket.local_addr()? else {
        return Err(io::Error::new(io::ErrorKind::InvalidData, "not IPv4"));
    };
    if address.ip() != &BIND_IP || address.port() == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "bind was not IPv4 loopback",
        ));
    }
    Ok((socket, address))
}

fn send(sock: &UdpSocket, destination: SocketAddrV4, frame: &[u8]) -> io::Result<bool> {
    if destination.ip() != &BIND_IP || destination.port() == 0 || frame.len() > MAX_WIRE_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "invalid destination or size",
        ));
    }
    match sock.send_to(frame, destination) {
        Ok(size) if size == frame.len() => Ok(true),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::WriteZero,
            "partial UDP datagram",
        )),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(false),
        Err(error) => Err(error),
    }
}
fn from_loopback(source: SocketAddr) -> Option<SocketAddrV4> {
    match source {
        SocketAddr::V4(addr) if addr.ip() == &BIND_IP && addr.port() != 0 => Some(addr),
        _ => None,
    }
}

/// Foreground, single-thread, single owner. No hidden socket worker.
pub struct NetworkHost {
    socket: Option<UdpSocket>,
    local: SocketAddrV4,
    peer: Option<SocketAddrV4>,
    nonce: Option<u64>,
    welcome_count: u32,
    owner: HostFixture,
    stats: Counters,
}
impl NetworkHost {
    pub fn bind() -> io::Result<Self> {
        let (socket, local) = bind_loopback()?;
        let owner = HostFixture::new(
            GENERATION,
            [
                (
                    EndpointId(1001),
                    Binding {
                        peer: 11,
                        session: SHARED_SESSION,
                        epoch: EPOCH,
                        player: Player::One,
                    },
                ),
                (
                    EndpointId(1002),
                    Binding {
                        peer: REMOTE_PEER,
                        session: SHARED_SESSION,
                        epoch: EPOCH,
                        player: Player::Two,
                    },
                ),
            ],
        )
        .map_err(|error| io::Error::other(format!("host binding: {error:?}")))?;
        Ok(Self {
            socket: Some(socket),
            local,
            peer: None,
            nonce: None,
            welcome_count: 0,
            owner,
            stats: Counters::default(),
        })
    }
    pub fn endpoint(&self) -> SocketAddrV4 {
        self.local
    }
    pub fn peer(&self) -> Option<SocketAddrV4> {
        self.peer
    }
    pub fn counters(&self) -> &Counters {
        &self.stats
    }
    pub fn snapshot(&self) -> Rc<Snapshot> {
        self.owner.snapshot()
    }
    pub fn connected(&self) -> bool {
        self.peer.is_some()
    }
    pub fn socket_closed(&self) -> bool {
        self.socket.is_none()
    }

    /// Called from winit's foreground event loop, finite packets/wake.
    pub fn poll(&mut self) -> io::Result<()> {
        let mut buf = [0u8; DATAGRAM_BUFFER];
        for _ in 0..MAX_RECEIVE_PER_WAKE {
            let Some(socket) = self.socket.as_ref() else {
                break;
            };
            let (length, from) = match socket.recv_from(&mut buf) {
                Ok(received) => received,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) => return Err(error),
            };
            self.stats.rx += 1;
            if length > MAX_WIRE_BYTES {
                self.stats.rx_oversize += 1;
                self.stats.rx_rejected += 1;
                continue;
            }
            let Some(source) = from_loopback(from) else {
                self.stats.rx_rejected += 1;
                continue;
            };
            let Ok(message) = decode(&buf[..length]) else {
                self.stats.rx_rejected += 1;
                continue;
            };
            match message {
                Message::Hello { nonce } => {
                    if self.peer.is_none() {
                        self.peer = Some(source);
                        self.nonce = Some(nonce);
                    }
                    if self.peer != Some(source)
                        || self.nonce != Some(nonce)
                        || self.welcome_count >= MAX_CLIENT_HELLO_ATTEMPTS
                    {
                        self.stats.rx_rejected += 1;
                        continue;
                    }
                    let response = encode(&Message::Welcome {
                        peer: REMOTE_PEER,
                        session: SHARED_SESSION,
                        epoch: EPOCH,
                        generation: GENERATION,
                        tick: self.owner.snapshot().tick(),
                        player: Player::Two,
                    })
                    .map_err(|error| io::Error::other(format!("welcome codec: {error:?}")))?;
                    if send(socket, source, &response)? {
                        self.stats.tx += 1;
                        self.stats.tx_welcomes += 1;
                        self.welcome_count += 1;
                        let initial =
                            self.owner
                                .snapshot_frame(SHARED_SESSION, EPOCH)
                                .map_err(|error| {
                                    io::Error::other(format!("snapshot codec: {error:?}"))
                                })?;
                        if send(socket, source, &initial)? {
                            self.stats.tx += 1;
                            self.stats.tx_snapshots += 1;
                        }
                        self.stats.rx_accepted += 1;
                    }
                }
                Message::HeldInput { .. } => {
                    if self.peer != Some(source) {
                        self.stats.rx_rejected += 1;
                        continue;
                    }
                    if self.owner.admit(EndpointId(1002), &buf[..length]).is_ok() {
                        self.stats.rx_accepted += 1;
                    } else {
                        self.stats.rx_rejected += 1;
                    }
                }
                _ => self.stats.rx_rejected += 1,
            }
        }
        Ok(())
    }

    pub fn step(&mut self) -> io::Result<()> {
        self.step_with_control(None)
    }

    /// YT-278: select one typed Pause/Resume for the same HostFixture owner.
    /// The ordinary D2 path still delegates to this with no new control.
    pub fn step_with_control(&mut self, control: Option<Command>) -> io::Result<()> {
        let next_tick = self
            .owner
            .next_tick()
            .map_err(|error| io::Error::other(format!("tick overflow: {error:?}")))?;
        // Host physical/local Player One owns the same C1 Simulation as remote input.
        let local = if next_tick < 42 {
            Direction::East
        } else {
            Direction::Still
        };
        self.owner
            .admit_local(Player::One, local)
            .map_err(|error| io::Error::other(format!("local admission: {error:?}")))?;
        let current = self
            .owner
            .advance_with_control(control)
            .map_err(|error| io::Error::other(format!("owner step: {error:?}")))?;
        if current.tick().is_multiple_of(SNAPSHOT_TICK_INTERVAL)
            && let (Some(peer), Some(sock)) = (self.peer, self.socket.as_ref())
        {
            let bytes = self
                .owner
                .snapshot_frame(SHARED_SESSION, EPOCH)
                .map_err(|error| io::Error::other(format!("snapshot encode: {error:?}")))?;
            if send(sock, peer, &bytes)? {
                self.stats.tx += 1;
                self.stats.tx_snapshots += 1;
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        self.owner.stop();
        self.socket.take();
    }
}

pub struct NetworkClient {
    socket: Option<UdpSocket>,
    local: SocketAddrV4,
    expected_host: SocketAddrV4,
    last_hello_at: Option<Instant>,
    hello_count: u32,
    binding: Option<(u64, u64, u64, u64)>, // peer/session/epoch/generation
    replica: Option<ReplicaFixture>,
    last_received: Option<Rc<WireSnapshot>>,
    input_sequence: u64,
    stats: Counters,
}
impl NetworkClient {
    pub fn bind(expected_host: SocketAddrV4) -> io::Result<Self> {
        if expected_host.ip() != &BIND_IP || expected_host.port() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "client needs exact 127.0.0.1 host",
            ));
        }
        let (socket, local) = bind_loopback()?;
        Ok(Self {
            socket: Some(socket),
            local,
            expected_host,
            last_hello_at: None,
            hello_count: 0,
            binding: None,
            replica: None,
            last_received: None,
            input_sequence: 0,
            stats: Counters::default(),
        })
    }
    pub fn endpoint(&self) -> SocketAddrV4 {
        self.local
    }
    pub fn expected_host(&self) -> SocketAddrV4 {
        self.expected_host
    }
    pub fn counters(&self) -> &Counters {
        &self.stats
    }
    pub fn snapshot(&self) -> Option<Rc<WireSnapshot>> {
        self.last_received.as_ref().map(Rc::clone)
    }
    pub fn connected(&self) -> bool {
        self.replica.is_some()
    }
    pub fn socket_closed(&self) -> bool {
        self.socket.is_none()
    }

    pub fn poll(&mut self, now: Instant) -> io::Result<()> {
        let Some(sock) = self.socket.as_ref() else {
            return Ok(());
        };
        if self.replica.is_none()
            && self.hello_count < MAX_CLIENT_HELLO_ATTEMPTS
            && self
                .last_hello_at
                .is_none_or(|previous| now.duration_since(previous) >= Duration::from_millis(250))
        {
            let hello = encode(&Message::Hello { nonce: HELLO_NONCE })
                .map_err(|error| io::Error::other(format!("hello codec: {error:?}")))?;
            if send(sock, self.expected_host, &hello)? {
                self.stats.tx += 1;
                self.stats.tx_hellos += 1;
                self.hello_count += 1;
                self.last_hello_at = Some(now);
            }
        }

        let mut buf = [0u8; DATAGRAM_BUFFER];
        for _ in 0..MAX_RECEIVE_PER_WAKE {
            let (length, from) = match sock.recv_from(&mut buf) {
                Ok(received) => received,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) => return Err(error),
            };
            self.stats.rx += 1;
            if length > MAX_WIRE_BYTES {
                self.stats.rx_oversize += 1;
                self.stats.rx_rejected += 1;
                continue;
            }
            if from_loopback(from) != Some(self.expected_host) {
                self.stats.rx_rejected += 1;
                continue;
            }
            let Ok(message) = decode(&buf[..length]) else {
                self.stats.rx_rejected += 1;
                continue;
            };
            match message {
                Message::Welcome {
                    peer,
                    session,
                    epoch,
                    generation,
                    player,
                    ..
                } => {
                    if player != Player::Two
                        || peer != REMOTE_PEER
                        || session != SHARED_SESSION
                        || epoch != EPOCH
                        || generation != GENERATION
                    {
                        self.stats.rx_rejected += 1;
                        continue;
                    }
                    if self.binding.is_none() {
                        self.replica = Some(
                            ReplicaFixture::new(EndpointId(9001), session, epoch, generation)
                                .map_err(|error| {
                                    io::Error::other(format!("replica binding: {error:?}"))
                                })?,
                        );
                        self.binding = Some((peer, session, epoch, generation));
                        self.stats.rx_accepted += 1;
                    } else {
                        self.stats.rx_rejected += 1;
                    }
                }
                Message::Snapshot(_) => {
                    let Some(replica) = self.replica.as_mut() else {
                        self.stats.rx_rejected += 1;
                        continue;
                    };
                    let Ok(current) = replica.admit_snapshot(EndpointId(9001), &buf[..length])
                    else {
                        self.stats.rx_rejected += 1;
                        continue;
                    };
                    self.last_received = Some(Rc::clone(&current));
                    self.stats.rx_accepted += 1;
                    let next_tick = current.tick.checked_add(1);
                    let next_sequence = self.input_sequence.checked_add(1);
                    if let (Some(tick), Some(sequence), Some((peer, session, epoch, _))) =
                        (next_tick, next_sequence, self.binding)
                    {
                        let dir = if tick < 64 {
                            Direction::West
                        } else {
                            Direction::Still
                        };
                        let input = encode(&Message::HeldInput {
                            peer,
                            session,
                            epoch,
                            sequence,
                            tick,
                            direction: dir,
                        })
                        .map_err(|error| io::Error::other(format!("input encode: {error:?}")))?;
                        if send(sock, self.expected_host, &input)? {
                            self.input_sequence = sequence;
                            self.stats.tx += 1;
                            self.stats.tx_inputs += 1;
                        }
                    }
                }
                _ => self.stats.rx_rejected += 1,
            }
        }
        Ok(())
    }

    pub fn stop(&mut self) {
        self.socket.take();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn d2_bounded_datagrams_have_exact_512_plus_one_oversize_detection() {
        assert_eq!(DATAGRAM_BUFFER, 513);
        assert_eq!(DATAGRAM_BUFFER - MAX_WIRE_BYTES, 1);
        assert_eq!(MAX_RECEIVE_PER_WAKE, 8);
        assert_eq!(SNAPSHOT_TICK_INTERVAL, 3);
    }

    #[test]
    fn d2_real_udp_socket_binds_only_explicit_loopback_with_ephemeral_port() {
        let server = NetworkHost::bind().expect("bind host");
        assert_eq!(server.endpoint().ip(), &Ipv4Addr::LOCALHOST);
        assert_ne!(server.endpoint().port(), 0);
        let mut client = NetworkClient::bind(server.endpoint()).expect("bind client");
        assert_eq!(client.endpoint().ip(), &Ipv4Addr::LOCALHOST);
        assert_ne!(client.endpoint().port(), server.endpoint().port());
        assert!(NetworkClient::bind(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 5001)).is_err());
        assert!(NetworkClient::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).is_err());
        client.stop();
        assert!(client.socket_closed());
    }

    #[test]
    fn d2_host_and_client_session_over_real_loopback_datagrams() {
        let mut host = NetworkHost::bind().unwrap();
        let mut client = NetworkClient::bind(host.endpoint()).unwrap();
        client.poll(Instant::now()).unwrap();
        host.poll().unwrap();
        client.poll(Instant::now()).unwrap();
        assert!(host.connected());
        assert!(client.connected());
        assert_eq!(host.peer(), Some(client.endpoint()));
        assert_eq!(host.counters().rx_accepted, 1);
        assert_eq!(client.counters().tx_hellos, 1);
        assert_eq!(host.counters().tx_welcomes, 1);
        assert!(client.snapshot().is_some());
        assert_eq!(client.snapshot().unwrap().tick, 0);

        // The client's real HeldInput datagram targeted exact next Tick.
        host.poll().unwrap();
        assert!(host.counters().rx_accepted >= 2, "{:?}", host.counters());
        host.step().unwrap();
        assert_eq!(host.snapshot().tick(), 1);
        assert_eq!(host.snapshot().positions()[0].x, 3);
        assert_eq!(host.snapshot().positions()[1].x, 19);
        assert_eq!(client.counters().tx_inputs, 1);
        assert!(host.counters().tx_snapshots >= 1);
        host.stop();
        client.stop();
        assert!(host.socket_closed() && client.socket_closed());
    }

    #[test]
    fn d2_foreign_endpoint_and_oversized_packet_cannot_change_owner() {
        let mut host = NetworkHost::bind().unwrap();
        let attacker = UdpSocket::bind(loopback_v4(0)).unwrap();
        attacker
            .send_to(&vec![0u8; MAX_WIRE_BYTES + 1], host.endpoint())
            .unwrap();
        host.poll().unwrap();
        assert_eq!(host.counters().rx_oversize, 1);
        assert_eq!(host.counters().rx_accepted, 0);
        let before = host.snapshot();
        let crafted = encode(&Message::HeldInput {
            peer: REMOTE_PEER,
            session: SHARED_SESSION,
            epoch: EPOCH,
            sequence: 1,
            tick: 1,
            direction: Direction::West,
        })
        .unwrap();
        attacker.send_to(&crafted, host.endpoint()).unwrap();
        host.poll().unwrap();
        assert_eq!(host.counters().rx_rejected, 2);
        assert_eq!(host.counters().rx_accepted, 0);
        assert!(Rc::ptr_eq(&before, &host.snapshot()));
    }
}
