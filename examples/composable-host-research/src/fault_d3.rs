//! D3 private fixed-seed datagram fault link on actual IPv4 loopback.
//! This is a deterministic test adapter, NOT real LAN chaos, a transport API,
//! a network security layer, or a worker. Only the caller polls it.
use crate::protocol::{MAX_WIRE_BYTES, Message, decode};
use std::{
    io,
    net::{Ipv4Addr, SocketAddr, SocketAddrV4, UdpSocket},
    time::{Duration, Instant},
};

pub const MAX_RECV_PER_POLL: usize = 8;
pub const MAX_SEND_PER_DIRECTION: usize = 4;
pub const MAX_QUEUED_PER_DIRECTION: usize = 16;
pub const MAX_QUEUE_TOTAL: usize = MAX_QUEUED_PER_DIRECTION * 2;
pub const MAX_JITTER_MS: u64 = 100;
pub const FAULT_SEED: u64 = 0xD320_2611_1807;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct FaultCounters {
    pub data_seen: u64,
    pub fault_eligible: u64,
    pub dropped: u64,
    pub duplicated: u64,
    pub delayed: u64,
    pub reordered: u64,
    pub forwarded: u64,
    pub control_forwarded: u64,
    pub queue_full: u64,
    pub oversize: u64,
    pub foreign: u64,
    pub max_pending: usize,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FaultChoice {
    pub drop: bool,
    pub duplicate: bool,
    pub delay_ms: u64,
}
/// One guaranteed 10% drop / 5% duplicate for each full block of twenty
/// data packets, with the three selected positions permuted by a fixed seed.
/// All scheduled delays are 0..100ms. No library RNG or floating probabilities.
#[derive(Clone, Debug)]
pub struct FaultPlan {
    order: [u8; 20],
    seed: u64,
}
fn next_random(value: &mut u64) -> u64 {
    *value ^= *value >> 12;
    *value ^= *value << 25;
    *value ^= *value >> 27;
    value.wrapping_mul(0x2545_F491_4F6C_DD1D)
}
impl FaultPlan {
    pub fn new(seed: u64) -> Self {
        let mut s = seed.max(1);
        let mut order = std::array::from_fn(|i| i as u8);
        for index in (1..20).rev() {
            let other = (next_random(&mut s) % (index as u64 + 1)) as usize;
            order.swap(index, other);
        }
        Self { order, seed }
    }
    pub fn select(&self, ordinal: u64) -> FaultChoice {
        let slot = (ordinal % 20) as u8;
        let salt = self
            .seed
            .wrapping_add(ordinal.wrapping_mul(0x9E37_79B9_7F4A_7C15));
        let delay_ms = next_random(&mut salt.max(1)) % (MAX_JITTER_MS + 1);
        FaultChoice {
            drop: slot == self.order[0] || slot == self.order[1],
            duplicate: slot == self.order[2],
            delay_ms,
        }
    }
}

#[derive(Debug)]
struct Packet {
    due: Instant,
    ordinal: u64,
    dest: SocketAddrV4,
    data: Vec<u8>,
}

/// A single private loopback UDP relay controlled by its Client process.
/// One configured Client and one Host endpoint; all others are rejected
/// before admission or response. No broadcasts, wildcard listener or LAN.
pub struct FaultProxy {
    socket: Option<UdpSocket>,
    local: SocketAddrV4,
    client: SocketAddrV4,
    host: SocketAddrV4,
    plan: FaultPlan,
    enabled: bool,
    ordinal: u64,
    outgoing: [Vec<Packet>; 2],
    last_sent_ordinal: [Option<u64>; 2],
    stats: FaultCounters,
}
impl FaultProxy {
    pub fn bind(host: SocketAddrV4, client: SocketAddrV4, seed: u64) -> io::Result<Self> {
        for peer in [host, client] {
            if peer.ip() != &Ipv4Addr::LOCALHOST || peer.port() == 0 {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "explicit loopback peer required",
                ));
            }
        }
        if host == client {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "distinct Host and Client",
            ));
        }
        let socket = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0))?;
        socket.set_nonblocking(true)?;
        let SocketAddr::V4(local) = socket.local_addr()? else {
            return Err(io::Error::other("expected IPv4 loopback"));
        };
        Ok(Self {
            socket: Some(socket),
            local,
            client,
            host,
            plan: FaultPlan::new(seed),
            enabled: false,
            ordinal: 0,
            outgoing: [Vec::new(), Vec::new()],
            last_sent_ordinal: [None, None],
            stats: FaultCounters::default(),
        })
    }
    pub fn endpoint(&self) -> SocketAddrV4 {
        self.local
    }
    pub fn counters(&self) -> FaultCounters {
        self.stats
    }
    pub fn pending(&self) -> usize {
        self.outgoing.iter().map(Vec::len).sum()
    }
    pub fn set_faults(&mut self, active: bool) {
        self.enabled = active;
    }
    pub fn stopped(&self) -> bool {
        self.socket.is_none()
    }
    pub fn stop(&mut self) {
        self.socket.take();
        for queue in &mut self.outgoing {
            queue.clear();
        }
    }

    pub fn poll(&mut self, now: Instant) -> io::Result<()> {
        if self.socket.is_none() {
            return Ok(());
        }
        let mut buf = [0u8; MAX_WIRE_BYTES + 1];
        for _ in 0..MAX_RECV_PER_POLL {
            let (len, from) = match self
                .socket
                .as_ref()
                .expect("live proxy")
                .recv_from(&mut buf)
            {
                Ok(item) => item,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) => return Err(error),
            };
            if len > MAX_WIRE_BYTES {
                self.stats.oversize += 1;
                continue;
            }
            let (direction, dest) = match from {
                SocketAddr::V4(addr) if addr == self.client => (0, self.host),
                SocketAddr::V4(addr) if addr == self.host => (1, self.client),
                _ => {
                    self.stats.foreign += 1;
                    continue;
                }
            };
            let Ok(message) = decode(&buf[..len]) else {
                self.stats.foreign += 1;
                continue;
            };
            let control = !matches!(message, Message::HeldInput { .. } | Message::Snapshot(_));
            if control {
                // The bounded Join/Welcome path is not fault-injected.
                if self.enqueue(direction, dest, &buf[..len], now, self.ordinal) {
                    self.stats.control_forwarded += 1;
                }
                continue;
            }
            let index = self.ordinal;
            self.ordinal = self
                .ordinal
                .checked_add(1)
                .ok_or_else(|| io::Error::other("D3 fault sequence exhausted"))?;
            self.stats.data_seen += 1;
            let choice = if self.enabled {
                self.stats.fault_eligible += 1;
                self.plan.select(index)
            } else {
                FaultChoice {
                    drop: false,
                    duplicate: false,
                    delay_ms: 0,
                }
            };
            if choice.drop {
                self.stats.dropped += 1;
                continue;
            }
            let due = now + Duration::from_millis(choice.delay_ms);
            if choice.delay_ms != 0 {
                self.stats.delayed += 1;
            }
            self.enqueue(direction, dest, &buf[..len], due, index);
            if choice.duplicate && self.enqueue(direction, dest, &buf[..len], due, index) {
                self.stats.duplicated += 1;
            }
        }
        // Directional fairness: at most four due packets each way/wake,
        // regardless of a flood from the opposite endpoint.
        for direction in 0..2 {
            for _ in 0..MAX_SEND_PER_DIRECTION {
                let Some(idx) = self.outgoing[direction]
                    .iter()
                    .enumerate()
                    .filter(|(_, pkt)| pkt.due <= now)
                    .min_by_key(|(_, pkt)| (pkt.due, pkt.ordinal))
                    .map(|(idx, _)| idx)
                else {
                    break;
                };
                let packet = self.outgoing[direction].remove(idx);
                match self
                    .socket
                    .as_ref()
                    .expect("live proxy")
                    .send_to(&packet.data, packet.dest)
                {
                    Ok(written) if written == packet.data.len() => {
                        if self.last_sent_ordinal[direction]
                            .is_some_and(|last| packet.ordinal < last)
                        {
                            self.stats.reordered += 1;
                        }
                        self.last_sent_ordinal[direction] = Some(packet.ordinal);
                        self.stats.forwarded += 1;
                    }
                    Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                        self.outgoing[direction].push(packet);
                        break;
                    }
                    Err(error) => return Err(error),
                    _ => return Err(io::Error::other("partial UDP datagram")),
                }
            }
        }
        Ok(())
    }
    fn enqueue(
        &mut self,
        direction: usize,
        dest: SocketAddrV4,
        data: &[u8],
        due: Instant,
        ordinal: u64,
    ) -> bool {
        if self.outgoing[direction].len() >= MAX_QUEUED_PER_DIRECTION {
            self.stats.queue_full += 1;
            return false;
        }
        self.outgoing[direction].push(Packet {
            due,
            ordinal,
            dest,
            data: data.to_vec(),
        });
        self.stats.max_pending = self.stats.max_pending.max(self.pending());
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::protocol::{Message, encode};

    #[test]
    fn d3_fixed_seed_exact_twenty_packet_ratios_and_delay_bounds() {
        let one = FaultPlan::new(FAULT_SEED);
        let two = FaultPlan::new(FAULT_SEED);
        let choices: Vec<_> = (0..100).map(|i| one.select(i)).collect();
        assert_eq!(choices, (0..100).map(|i| two.select(i)).collect::<Vec<_>>());
        assert_eq!(choices.iter().filter(|c| c.drop).count(), 10);
        assert_eq!(choices.iter().filter(|c| c.duplicate).count(), 5);
        assert!(choices.iter().all(|c| c.delay_ms <= 100));
        for part in choices.as_slice().as_chunks::<20>().0 {
            assert_eq!(part.iter().filter(|c| c.drop).count(), 2);
            assert_eq!(part.iter().filter(|c| c.duplicate).count(), 1);
            assert!(part.iter().all(|c| !(c.drop && c.duplicate)));
        }
        assert_ne!(one.select(0), FaultPlan::new(1).select(0));
    }

    #[test]
    fn d3_flooded_client_direction_does_not_starve_host_to_client() {
        let host = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let client = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        client.set_nonblocking(true).unwrap();
        let SocketAddr::V4(haddr) = host.local_addr().unwrap() else {
            panic!()
        };
        let SocketAddr::V4(caddr) = client.local_addr().unwrap() else {
            panic!()
        };
        let mut proxy = FaultProxy::bind(haddr, caddr, FAULT_SEED).unwrap();
        let start = Instant::now();
        let msg = encode(&Message::Hello { nonce: 123 }).unwrap();
        for i in 0..MAX_QUEUED_PER_DIRECTION {
            assert!(proxy.enqueue(0, haddr, &msg, start + Duration::from_secs(2), i as u64));
        }
        assert_eq!(proxy.pending(), MAX_QUEUED_PER_DIRECTION);
        host.send_to(&msg, proxy.endpoint()).unwrap();
        proxy.poll(start + Duration::from_millis(5)).unwrap();
        assert_eq!(proxy.pending(), MAX_QUEUED_PER_DIRECTION);
        let mut buffer = [0u8; MAX_WIRE_BYTES + 1];
        let (length, source) = client.recv_from(&mut buffer).unwrap();
        assert_eq!(source, SocketAddr::V4(proxy.endpoint()));
        assert_eq!(&buffer[..length], msg);
        assert!(proxy.counters().forwarded >= 1);
        assert!(proxy.counters().max_pending <= MAX_QUEUE_TOTAL);
        proxy.stop();
    }
    #[test]
    fn d3_fault_proxy_real_loopback_finite_buffers_and_directional_fairness() {
        let host = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        let client = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        host.set_nonblocking(true).unwrap();
        let SocketAddr::V4(host_address) = host.local_addr().unwrap() else {
            panic!()
        };
        let SocketAddr::V4(client_address) = client.local_addr().unwrap() else {
            panic!()
        };
        let mut proxy = FaultProxy::bind(host_address, client_address, FAULT_SEED).unwrap();
        let msg = encode(&Message::Hello { nonce: 100 }).unwrap();
        let start = Instant::now();
        // Saturate the *same bounded queue* with future-due datagrams;
        // normal immediate loopback sends may drain too fast to fill it.
        for index in 0..MAX_QUEUED_PER_DIRECTION + 3 {
            proxy.enqueue(
                0,
                host_address,
                &msg,
                start + Duration::from_millis(250),
                index as u64,
            );
        }
        for _ in 0..8 {
            client.send_to(&msg, proxy.endpoint()).unwrap();
        }
        proxy.poll(start).unwrap();
        proxy.poll(start + Duration::from_millis(251)).unwrap();
        assert!(proxy.pending() <= MAX_QUEUE_TOTAL);
        assert!(proxy.counters().max_pending <= MAX_QUEUE_TOTAL);
        assert!(proxy.counters().forwarded > 0);
        assert!(proxy.counters().queue_full > 0);
        assert_eq!(proxy.counters().data_seen, 0);
        let foreign = UdpSocket::bind(SocketAddrV4::new(Ipv4Addr::LOCALHOST, 0)).unwrap();
        foreign.send_to(&msg, proxy.endpoint()).unwrap();
        proxy.poll(start + Duration::from_millis(10)).unwrap();
        assert!(proxy.counters().foreign >= 1);
        proxy.stop();
        assert!(proxy.stopped());
        assert_eq!(proxy.pending(), 0);
    }
}
