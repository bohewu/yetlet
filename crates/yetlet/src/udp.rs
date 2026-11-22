//! Optional, bounded UDP transport on the caller's thread.
//!
//! [`UdpHost`] supplies ordinary [`crate::HostGroup`] lifecycle management;
//! [`UdpEndpoint`] is the independently owned socket for manual integrations.
//! Both use the same nonblocking receive/send implementation, with no runtime,
//! background thread, packet codec, session policy or guaranteed delivery.
//!
//! The managed Host requires the group's synchronous
//! [`run`](crate::composition::BuiltHostGroup::run) driver. Its bounded polling
//! runs on that caller, including native event-loop callbacks when selected.
//! Async preparation is rejected before acquisition: a std UDP socket alone
//! cannot register packet readiness with an arbitrary executor.
//!
//! # Ordinary UDP Host
//!
//! An annotation selects the default IO error when there is no sibling Host to
//! infer the application's error type:
//!
//! ```no_run
//! use yetlet::{HostGroup, udp::UdpHost};
//!
//! let udp: UdpHost = UdpHost::new();
//! let mut hosts = HostGroup::new().with_host(udp).build().unwrap();
//! hosts.run().unwrap();
//! ```
//!
//! Supply a bounded synchronous callback to inspect bytes or reply. Callbacks
//! may borrow local application state; neither they nor their errors require
//! Clone, Send or 'static:
//!
//! ```no_run
//! use std::{cell::Cell, io};
//! use yetlet::{HostGroup, udp::UdpHost};
//!
//! let received = Cell::new(0);
//! let plan = HostGroup::new();
//! let shutdown = plan.shutdown_handle();
//! let udp = UdpHost::<io::Error>::new().on_datagram(|datagram| {
//!     received.set(received.get() + 1);
//!     datagram.reply(datagram.bytes())?;
//!     shutdown.request_shutdown();
//!     Ok(())
//! });
//! let mut hosts = plan.with_host(udp).build().unwrap();
//! hosts.run().unwrap();
//! ```
//!
//! A sibling Host can infer a common error type through From<io::Error>. This
//! module has no Web dependency; the example requires both optional features:
//!
//! ```no_run
//! # #[cfg(feature = "web")]
//! # {
//! use yetlet::{HostGroup, udp::UdpHost, web::App};
//!
//! let udp = UdpHost::new();
//! let mut hosts = HostGroup::new()
//!     .with_host(App::new().into_host())
//!     .with_host(udp)
//!     .build()
//!     .unwrap();
//! hosts.run().unwrap();
//! # }
//! ```
//!
//! Loopback and an ephemeral port are the default. An explicit address opts
//! into other interfaces; it establishes neither peer trust nor physical-LAN
//! readiness. Applications select trusted senders and interpret packet bytes.

use std::{
    io,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
};

mod host;
pub use host::{UdpActivity, UdpHandle, UdpHost};

/// Conservative maximum IPv4 UDP payload size; not a protocol frame format.
pub const MAX_DATAGRAM_BYTES: usize = 65_507;

/// Maximum datagrams per caller poll; a large requested budget cannot create
/// an effectively unbounded receive loop.
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

/// Borrowed bytes, sender and bounded sending capability for one callback.
///
/// The context cannot outlive the callback, keep the socket alive or send after
/// the Host's admission fence. Sending is a single nonblocking OS attempt and
/// uses the same capacity/destination checks as [`UdpEndpoint::send_to`].
pub struct Datagram<'a> {
    bytes: &'a [u8],
    sender: SocketAddr,
    socket: &'a UdpSocket,
    max_datagram_bytes: usize,
}

impl Datagram<'_> {
    pub fn bytes(&self) -> &[u8] {
        self.bytes
    }

    /// The address returned by the socket receive operation; not authenticated.
    pub fn sender(&self) -> SocketAddr {
        self.sender
    }

    pub fn send_to(&self, destination: SocketAddr, bytes: &[u8]) -> io::Result<SendOutcome> {
        send_datagram(self.socket, self.max_datagram_bytes, destination, bytes)
    }

    pub fn reply(&self, bytes: &[u8]) -> io::Result<SendOutcome> {
        self.send_to(self.sender, bytes)
    }
}

/// One owner of one real, nonblocking UDP socket; no hidden runtime/thread.
///
/// This independent value remains Send. Callers using an external driver can
/// poll it directly and register their own explicit close callback.
pub struct UdpEndpoint {
    socket: Option<UdpSocket>,
    buffer: Vec<u8>,
    max_datagram_bytes: usize,
}

impl UdpEndpoint {
    /// Bind IPv4 loopback on an ephemeral port.
    pub fn bind_loopback(max_datagram_bytes: usize) -> io::Result<Self> {
        Self::bind_on(
            SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            max_datagram_bytes,
        )
    }

    /// Bind an explicitly selected address. The caller owns exposure policy.
    ///
    /// Capacity is bounded to one safely representable UDP payload, and the
    /// receive buffer has one extra byte to detect oversized datagrams. A
    /// truncated datagram is never passed to the application callback.
    pub fn bind_on(bind: SocketAddr, max_datagram_bytes: usize) -> io::Result<Self> {
        let mut endpoint = Self::unbound(max_datagram_bytes)?;
        endpoint.bind(bind)?;
        Ok(endpoint)
    }

    fn unbound(max_datagram_bytes: usize) -> io::Result<Self> {
        validate_capacity(max_datagram_bytes)?;
        Ok(Self {
            socket: None,
            buffer: vec![0; max_datagram_bytes + 1],
            max_datagram_bytes,
        })
    }

    fn bind(&mut self, address: SocketAddr) -> io::Result<()> {
        // Retain the exact acquired owner before a later fallible setup step.
        // The managed activity retains this endpoint through rollback.
        self.socket = Some(UdpSocket::bind(address)?);
        self.open_socket()?.set_nonblocking(true)
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
    /// submission is not proof of remote receipt, reliability or authorization.
    pub fn send_to(&self, destination: SocketAddr, bytes: &[u8]) -> io::Result<SendOutcome> {
        send_datagram(
            self.open_socket()?,
            self.max_datagram_bytes,
            destination,
            bytes,
        )
    }

    /// Poll no more than max_packets datagrams on the caller's thread.
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
        self.poll_context(max_packets, |datagram| {
            on_datagram(datagram.bytes(), datagram.sender())
        })
    }

    fn poll_context<E, F>(
        &mut self,
        max_packets: usize,
        mut on_datagram: F,
    ) -> Result<PollReport, E>
    where
        E: From<io::Error>,
        F: FnMut(Datagram<'_>) -> Result<(), E>,
    {
        let socket = self
            .socket
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "UDP endpoint is closed"))?;
        if max_packets > MAX_PACKETS_PER_POLL {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "receive budget exceeds the finite per-wake limit of 64 packets",
            )
            .into());
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
                #[cfg(windows)]
                Err(error) if error.raw_os_error() == Some(10_040) => {
                    // WSAEMSGSIZE consumes this oversized UDP datagram. Count
                    // the rejection within the same finite receive budget.
                    report.received += 1;
                    report.rejected_oversize += 1;
                    continue;
                }
                Err(error) => return Err(error.into()),
            };
            report.received += 1;
            if length > self.max_datagram_bytes {
                report.rejected_oversize += 1;
                continue;
            }
            on_datagram(Datagram {
                bytes: &self.buffer[..length],
                sender,
                socket,
                max_datagram_bytes: self.max_datagram_bytes,
            })?;
            report.delivered += 1;
        }
        report.budget_reached = true;
        Ok(report)
    }

    /// Idempotently release the actual UDP socket. The caller can then rebind
    /// the old endpoint if the operating system permits it.
    pub fn close(&mut self) -> io::Result<()> {
        self.socket.take();
        Ok(())
    }
}

fn validate_capacity(max_datagram_bytes: usize) -> io::Result<()> {
    if max_datagram_bytes == 0 || max_datagram_bytes > MAX_DATAGRAM_BYTES {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "UDP datagram capacity must be in 1..=65507 bytes",
        ));
    }
    Ok(())
}

fn send_datagram(
    socket: &UdpSocket,
    max_datagram_bytes: usize,
    destination: SocketAddr,
    bytes: &[u8],
) -> io::Result<SendOutcome> {
    if destination.port() == 0 || destination.ip().is_unspecified() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "UDP destination must be a concrete IP address and nonzero port",
        ));
    }
    if bytes.len() > max_datagram_bytes {
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
