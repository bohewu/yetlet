//! Optional caller-driven UDP byte/socket lifecycle adapter.
//!
//! Nonblocking socket I/O only: no game tick, packet codec, session/peer
//! authorization, retransmission or guaranteed delivery. The application
//! decides which sender to trust and what received bytes mean. There is no
//! Tokio runtime, background worker, Host trait or inherent scheduler.
//!
//! The default listener is IPv4 loopback on an ephemeral port. Binding other
//! interfaces requires the explicit `bind_on` entry point and should not
//! be mistaken for physical-LAN readiness or a secure transport.
//!
//! `UdpEndpoint` is an owned local value. The caller may attach it through
//! `yetlet::composition::Builder::required_local` and pass
//! `UdpEndpoint::close` as its explicit lifecycle callback, without requiring
//! HTTP/Axum or additional dependencies.

use std::{
    io,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
};

/// Conservative maximum IPv4 UDP payload size; not a protocol frame format.
pub const MAX_DATAGRAM_BYTES: usize = 65_507;

/// Maximum caller-selected datagrams per foreground wake; a huge
/// requested budget cannot become an effectively unbounded poll.
pub const MAX_PACKETS_PER_POLL: usize = 64;

/// Outcome of a nonblocking datagram send.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SendOutcome {
    /// Complete datagram submitted to the OS. Not an acknowledgement.
    Sent(usize),
    /// OS socket could not accept the datagram on this nonblocking attempt.
    WouldBlock,
}

/// Bounded polling evidence; no packet is silently treated as an application step.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PollReport {
    /// UDP datagrams removed from this socket's OS receive queue.
    pub received: usize,
    /// Datagrams delivered to the caller callback, not interpreted/applied.
    pub delivered: usize,
    /// Oversize datagrams consumed and rejected without callback delivery.
    pub rejected_oversize: usize,
    /// OS reported WouldBlock: receive queue empty at the instant observed.
    pub would_block: bool,
    /// Exact caller packet budget reached; does not imply data remains queued.
    pub budget_reached: bool,
}

/// One owner of one real, nonblocking UDP socket; no hidden runtime/thread.
pub struct UdpEndpoint {
    socket: Option<UdpSocket>,
    buffer: Vec<u8>,
    max_datagram_bytes: usize,
}

impl UdpEndpoint {
    /// Bind safe IPv4 loopback on an ephemeral port.
    pub fn bind_loopback(max_datagram_bytes: usize) -> io::Result<Self> {
        Self::bind_on(
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            max_datagram_bytes,
        )
    }

    /// Bind an explicitly selected address. The caller owns exposure policy.
    ///
    /// Capacity is bounded to one safely representable UDP payload, and the
    /// receive buffer has one extra byte to **detect** oversized datagrams.
    /// A truncated datagram is never passed to the application callback.
    pub fn bind_on(bind: SocketAddr, max_datagram_bytes: usize) -> io::Result<Self> {
        if max_datagram_bytes == 0 || max_datagram_bytes > MAX_DATAGRAM_BYTES {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "UDP datagram capacity must be in 1..=65507 bytes",
            ));
        }
        let socket = UdpSocket::bind(bind)?;
        socket.set_nonblocking(true)?;
        Ok(Self {
            socket: Some(socket),
            buffer: vec![0; max_datagram_bytes + 1],
            max_datagram_bytes,
        })
    }

    /// Current bound endpoint. After close, returns NotConnected, not stale
    /// readiness or a fictitious active socket.
    pub fn local_addr(&self) -> io::Result<SocketAddr> {
        self.open_socket()?.local_addr()
    }

    /// Maximum delivered or submitted datagram size for this endpoint.
    pub fn max_datagram_bytes(&self) -> usize {
        self.max_datagram_bytes
    }

    /// Whether the underlying owned socket has actually been released.
    pub fn is_closed(&self) -> bool {
        self.socket.is_none()
    }

    fn open_socket(&self) -> io::Result<&UdpSocket> {
        self.socket
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "UDP endpoint is closed"))
    }

    /// Send one complete datagram to an explicitly selected peer. Nonblocking
    /// submission is NOT proof of remote receipt, reliability or authorization.
    pub fn send_to(&self, destination: SocketAddr, bytes: &[u8]) -> io::Result<SendOutcome> {
        let socket = self.open_socket()?;
        if destination.port() == 0 || destination.ip().is_unspecified() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "UDP destination must be a concrete IP address and nonzero port",
            ));
        }
        if bytes.len() > self.max_datagram_bytes {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "UDP outgoing datagram exceeds configured capacity",
            ));
        }
        match socket.send_to(bytes, destination) {
            Ok(length) if length == bytes.len() => Ok(SendOutcome::Sent(length)),
            Ok(_) => Err(io::Error::new(
                io::ErrorKind::WriteZero,
                "partial UDP datagram submission",
            )),
            Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(SendOutcome::WouldBlock),
            Err(error) => Err(error),
        }
    }

    /// Poll no more than `max_packets` datagrams in the foreground caller.
    ///
    /// Each accepted datagram is delivered as borrowed raw bytes together
    /// with its actual sender. Callers supply peer/session policy and codec.
    /// Callback errors propagate without consuming further datagrams; a
    /// previously dequeued datagram cannot be replayed automatically.
    /// Zero budget does not poll. No blocking receive or unbounded loop.
    pub fn poll_bounded<F>(
        &mut self,
        max_packets: usize,
        mut on_datagram: F,
    ) -> io::Result<PollReport>
    where
        F: FnMut(&[u8], SocketAddr) -> io::Result<()>,
    {
        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "UDP endpoint is closed"))?;
        if max_packets > MAX_PACKETS_PER_POLL {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "receive budget exceeds the finite per-wake limit of 64 packets",
            ));
        }
        let mut report = PollReport::default();
        if max_packets == 0 {
            report.budget_reached = true;
            return Ok(report);
        }
        for _ in 0..max_packets {
            let (length, sender) = match socket.recv_from(&mut self.buffer) {
                Ok(packet) => packet,
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    report.would_block = true;
                    return Ok(report);
                }
                Err(error) => return Err(error),
            };
            report.received += 1;
            if length > self.max_datagram_bytes {
                report.rejected_oversize += 1;
                continue;
            }
            on_datagram(&self.buffer[..length], sender)?;
            report.delivered += 1;
        }
        report.budget_reached = true;
        Ok(report)
    }

    /// Idempotently release the **actual** UDP socket. The caller can then
    /// rebind the old endpoint if the operating system permits it.
    pub fn close(&mut self) -> io::Result<()> {
        self.socket.take();
        Ok(())
    }
}
