use super::{Datagram, MAX_DATAGRAM_BYTES, MAX_PACKETS_PER_POLL, UdpEndpoint, validate_capacity};
use crate::composition::{
    AdmissionOutcome, ExclusiveClaim, Host, HostActivity, HostRunMode, ReleaseCompletion,
    ResourceGroup, WorkerCompletion,
};
use std::{
    io,
    marker::PhantomData,
    net::{Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex, Weak},
    task::{Context, Poll},
    time::Duration,
};

#[derive(Default)]
struct Observation {
    address: Option<SocketAddr>,
    ready: bool,
    admitting: bool,
}

#[derive(Default)]
struct Shared(Mutex<Observation>);

impl Shared {
    fn update(&self, f: impl FnOnce(&mut Observation)) {
        f(&mut self.0.lock().unwrap_or_else(|error| error.into_inner()));
    }

    fn clear(&self) {
        self.update(|observation| *observation = Observation::default());
    }
}

/// Weak, read-only metadata; clones never extend the UDP socket's lifetime.
///
/// This handle has no send operation. A managed send uses the borrowed
/// [`Datagram`] context during admission; independent socket users can use
/// [`UdpEndpoint`] and their own lifecycle instead.
#[derive(Clone)]
pub struct UdpHandle(Weak<Shared>);

impl UdpHandle {
    /// The actual bound endpoint while the activity retains its socket.
    /// Returns None before binding and after close or owner destruction.
    pub fn local_addr(&self) -> Option<SocketAddr> {
        let shared = self.0.upgrade()?;
        shared
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .address
    }

    pub fn is_ready(&self) -> bool {
        self.0.upgrade().is_some_and(|shared| {
            shared
                .0
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .ready
        })
    }

    pub fn is_admitting(&self) -> bool {
        self.0.upgrade().is_some_and(|shared| {
            shared
                .0
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .admitting
        })
    }
}

/// Inert description of one bounded caller-driven UDP activity.
///
/// The default listener is IPv4 loopback with an independently assigned port.
/// Build validates capacity and exact nonzero endpoint conflicts. Binding and
/// nonblocking setup happen only during managed start. Normal packet delivery
/// begins after every Host is ready.
///
/// E is the application's common error type. An enclosing HostGroup or sibling
/// Host may infer it; standalone callers can write UdpHost::<io::Error>::new().
/// A callback may borrow local state and return its original non-Clone error.
pub struct UdpHost<E = io::Error, H = for<'a> fn(Datagram<'a>) -> Result<(), E>> {
    name: &'static str,
    address: SocketAddr,
    max_datagram_bytes: usize,
    packet_budget: usize,
    handler: H,
    shared: Arc<Shared>,
    error: PhantomData<fn() -> E>,
}

fn ignore_datagram<E>(_: Datagram<'_>) -> Result<(), E> {
    Ok(())
}

impl<E> UdpHost<E> {
    pub fn new() -> Self {
        Self {
            name: "udp",
            address: SocketAddr::from((Ipv4Addr::LOCALHOST, 0)),
            max_datagram_bytes: MAX_DATAGRAM_BYTES,
            packet_budget: MAX_PACKETS_PER_POLL,
            handler: ignore_datagram::<E>,
            shared: Arc::default(),
            error: PhantomData,
        }
    }
}

impl<E> Default for UdpHost<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E, H> UdpHost<E, H> {
    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    pub fn with_address(mut self, address: SocketAddr) -> Self {
        self.address = address;
        self
    }

    pub fn with_max_datagram_bytes(mut self, max_datagram_bytes: usize) -> Self {
        self.max_datagram_bytes = max_datagram_bytes;
        self
    }

    /// Set the receive budget per caller poll, in 1..=MAX_PACKETS_PER_POLL.
    ///
    /// Callback work itself must also be bounded; no scheduler preempts it.
    pub fn with_packet_budget(mut self, packet_budget: usize) -> Self {
        self.packet_budget = packet_budget;
        self
    }

    pub fn on_datagram<F>(self, handler: F) -> UdpHost<E, F>
    where
        F: for<'a> FnMut(Datagram<'a>) -> Result<(), E>,
    {
        UdpHost {
            name: self.name,
            address: self.address,
            max_datagram_bytes: self.max_datagram_bytes,
            packet_budget: self.packet_budget,
            handler,
            shared: self.shared,
            error: PhantomData,
        }
    }

    pub fn address(&self) -> SocketAddr {
        self.address
    }

    pub fn handle(&self) -> UdpHandle {
        UdpHandle(Arc::downgrade(&self.shared))
    }
}

impl<E, H> Host for UdpHost<E, H>
where
    E: From<io::Error>,
    H: for<'a> FnMut(Datagram<'a>) -> Result<(), E>,
{
    type Error = E;
    type Activity = UdpActivity<E, H>;
    type Resources = ();

    fn name(&self) -> &'static str {
        self.name
    }

    fn validate(&self) -> Result<(), E> {
        validate_capacity(self.max_datagram_bytes)?;
        validate_packet_budget(self.packet_budget)?;
        Ok(())
    }

    fn exclusive_claims(&self) -> Vec<ExclusiveClaim> {
        if self.address.port() == 0 {
            Vec::new()
        } else {
            vec![ExclusiveClaim::new(
                "udp-endpoint",
                self.address.to_string(),
            )]
        }
    }

    fn into_resources(self) -> ResourceGroup<Self::Activity> {
        ResourceGroup::new(UdpActivity {
            address: self.address,
            max_datagram_bytes: self.max_datagram_bytes,
            packet_budget: self.packet_budget,
            handler: Some(self.handler),
            shared: self.shared,
            endpoint: None,
            terminal_error: None,
            prepared: false,
            start_attempted: false,
            admission_closed: false,
            stopping: false,
            joined: false,
        })
    }
}

/// Original socket, callback and completion retained by the Core resource group.
///
/// The callback remains owned until final release, after all original group
/// completions. Dropping this owner closes its socket through ordinary Rust
/// ownership; it does not represent completed managed shutdown.
pub struct UdpActivity<E = io::Error, H = for<'a> fn(Datagram<'a>) -> Result<(), E>> {
    address: SocketAddr,
    max_datagram_bytes: usize,
    packet_budget: usize,
    handler: Option<H>,
    shared: Arc<Shared>,
    endpoint: Option<UdpEndpoint>,
    terminal_error: Option<E>,
    prepared: bool,
    start_attempted: bool,
    admission_closed: bool,
    stopping: bool,
    joined: bool,
}

impl<E, H> UdpActivity<E, H> {
    fn start(&mut self) -> io::Result<()> {
        if self.stopping || self.joined {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "UDP activity is stopping",
            ));
        }
        if self.start_attempted {
            return if self
                .shared
                .0
                .lock()
                .unwrap_or_else(|error| error.into_inner())
                .ready
            {
                Ok(())
            } else {
                Err(io::Error::new(
                    io::ErrorKind::NotConnected,
                    "UDP acquisition did not complete",
                ))
            };
        }
        validate_packet_budget(self.packet_budget)?;
        self.start_attempted = true;
        self.endpoint = Some(UdpEndpoint::unbound(self.max_datagram_bytes)?);
        let endpoint = self.endpoint.as_mut().expect("retained UDP endpoint");
        endpoint.bind(self.address)?;
        let address = endpoint.local_addr()?;
        self.shared.update(|observation| {
            observation.address = Some(address);
            observation.ready = true;
        });
        Ok(())
    }
}

impl<E, H> HostActivity for UdpActivity<E, H>
where
    E: From<io::Error>,
    H: for<'a> FnMut(Datagram<'a>) -> Result<(), E>,
{
    type Error = E;

    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), E> {
        if mode != HostRunMode::Blocking {
            return Err(io::Error::new(
                io::ErrorKind::Unsupported,
                "managed UDP requires the synchronous caller-driven HostGroup::run driver",
            )
            .into());
        }
        self.prepared = true;
        Ok(())
    }

    fn poll_start(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), E>> {
        if !self.prepared {
            return Poll::Ready(self.prepare_driver(HostRunMode::Async));
        }
        Poll::Ready(self.start().map_err(E::from))
    }

    fn poll_ready(&mut self, _cx: &mut Context<'_>) -> Poll<Result<(), E>> {
        if self
            .shared
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .ready
        {
            Poll::Ready(Ok(()))
        } else {
            Poll::Ready(Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "UDP endpoint is not ready",
            )
            .into()))
        }
    }

    fn poll_finished(&mut self, _cx: &mut Context<'_>) -> Poll<()> {
        if self.stopping || self.joined || self.terminal_error.is_some() {
            return Poll::Ready(());
        }
        if !self
            .shared
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .admitting
        {
            return Poll::Pending;
        }
        let result = self
            .endpoint
            .as_mut()
            .expect("admitted UDP endpoint")
            .poll_context(
                self.packet_budget,
                self.handler.as_mut().expect("retained UDP callback"),
            );
        match result {
            Ok(_) => Poll::Pending,
            Err(error) => {
                self.terminal_error = Some(error);
                self.admission_closed = true;
                self.shared.update(|observation| {
                    observation.ready = false;
                    observation.admitting = false;
                });
                Poll::Ready(())
            }
        }
    }

    fn open_admission(&mut self) -> Result<AdmissionOutcome, E> {
        if self.admission_closed || self.stopping || self.joined || self.terminal_error.is_some() {
            return Ok(AdmissionOutcome::Closed);
        }
        let mut observation = self
            .shared
            .0
            .lock()
            .unwrap_or_else(|error| error.into_inner());
        if !observation.ready {
            return Err(
                io::Error::new(io::ErrorKind::NotConnected, "UDP endpoint is not ready").into(),
            );
        }
        observation.admitting = true;
        Ok(AdmissionOutcome::Opened)
    }

    fn close_admission(&mut self, _cx: &mut Context<'_>) -> Result<(), E> {
        self.admission_closed = true;
        self.shared
            .update(|observation| observation.admitting = false);
        Ok(())
    }

    fn request_shutdown(&mut self, _cx: &mut Context<'_>) -> Result<(), E> {
        self.admission_closed = true;
        self.shared
            .update(|observation| observation.admitting = false);
        self.stopping = true;
        Ok(())
    }

    fn poll_join(
        &mut self,
        _cx: &mut Context<'_>,
        _budget: Duration,
    ) -> Result<WorkerCompletion<E>, E> {
        if !self.stopping && self.terminal_error.is_none() && !self.joined {
            return Ok(WorkerCompletion::Pending);
        }
        if let Some(endpoint) = self.endpoint.as_mut() {
            endpoint.close()?;
        }
        self.shared.clear();
        self.joined = true;
        Ok(WorkerCompletion::Joined(
            self.terminal_error.take().map_or(Ok(()), Err),
        ))
    }

    fn poll_release(
        &mut self,
        _cx: &mut Context<'_>,
        _budget: Duration,
    ) -> Result<ReleaseCompletion<E>, E> {
        self.handler.take();
        self.endpoint.take();
        Ok(ReleaseCompletion::Released(Ok(())))
    }
}

impl<E, H> Drop for UdpActivity<E, H> {
    fn drop(&mut self) {
        if let Some(endpoint) = self.endpoint.as_mut() {
            let _ = endpoint.close();
        }
        self.shared.clear();
    }
}

fn validate_packet_budget(packet_budget: usize) -> io::Result<()> {
    if packet_budget == 0 || packet_budget > MAX_PACKETS_PER_POLL {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "managed UDP packet budget must be in 1..=64",
        ));
    }
    Ok(())
}
