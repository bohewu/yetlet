#![cfg(not(target_arch = "wasm32"))]
//! Real caller-driven UDP, public Host integration and retained lifecycle proofs.

use std::{
    cell::{Cell, RefCell},
    io,
    marker::PhantomData,
    net::{Ipv4Addr, SocketAddr, UdpSocket},
    rc::Rc,
    task::{Context, Poll, Waker},
    thread,
    time::{Duration, Instant},
};

use yetlet::{
    Host, HostGroup, ResourceGroup,
    composition::{
        AdmissionOutcome, ExclusiveClaim, HostActivity, HostBuildError, HostOperation, HostRunMode,
        Managed, ManagedResource, Phase, ReleaseCompletion, WorkerCompletion,
    },
    udp::{MAX_DATAGRAM_BYTES, MAX_PACKETS_PER_POLL, SendOutcome, UdpEndpoint, UdpHost},
};

const LIMIT: Duration = Duration::from_secs(2);

struct Probe<E, F, J> {
    work: F,
    joining: J,
    admitted: bool,
    stopping: bool,
    terminal: Option<Result<(), E>>,
    joins: usize,
    deadline: Instant,
}

fn probe<E, F>(work: F) -> Probe<E, F, fn(usize) -> bool>
where
    F: FnMut() -> Result<bool, E>,
{
    Probe {
        work,
        joining: |_| true,
        admitted: false,
        stopping: false,
        terminal: None,
        joins: 0,
        deadline: Instant::now() + LIMIT,
    }
}

impl<E, F, J> Probe<E, F, J> {
    fn with_join<K>(self, joining: K) -> Probe<E, F, K> {
        Probe {
            work: self.work,
            joining,
            admitted: self.admitted,
            stopping: self.stopping,
            terminal: self.terminal,
            joins: self.joins,
            deadline: self.deadline,
        }
    }
}

impl<E, F, J> Host for Probe<E, F, J>
where
    F: FnMut() -> Result<bool, E>,
    J: FnMut(usize) -> bool,
{
    type Error = E;
    type Activity = Self;
    type Resources = ();

    fn name(&self) -> &'static str {
        "probe"
    }

    fn into_resources(self) -> ResourceGroup<Self> {
        ResourceGroup::new(self)
    }
}

impl<E, F, J> HostActivity for Probe<E, F, J>
where
    F: FnMut() -> Result<bool, E>,
    J: FnMut(usize) -> bool,
{
    type Error = E;

    fn poll_start(&mut self, _: &mut Context<'_>) -> Poll<Result<(), E>> {
        Poll::Ready(Ok(()))
    }

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), E>> {
        Poll::Ready(Ok(()))
    }

    fn open_admission(&mut self) -> Result<AdmissionOutcome, E> {
        self.admitted = true;
        Ok(AdmissionOutcome::Opened)
    }

    fn poll_finished(&mut self, _: &mut Context<'_>) -> Poll<()> {
        if self.terminal.is_some() {
            return Poll::Ready(());
        }
        if !self.admitted {
            return Poll::Pending;
        }
        assert!(Instant::now() < self.deadline, "UDP integration deadline");
        match (self.work)() {
            Ok(false) => Poll::Pending,
            Ok(true) => {
                self.terminal = Some(Ok(()));
                Poll::Ready(())
            }
            Err(error) => {
                self.terminal = Some(Err(error));
                Poll::Ready(())
            }
        }
    }

    fn close_admission(&mut self, _: &mut Context<'_>) -> Result<(), E> {
        self.admitted = false;
        Ok(())
    }

    fn request_shutdown(&mut self, _: &mut Context<'_>) -> Result<(), E> {
        self.stopping = true;
        Ok(())
    }

    fn poll_join(&mut self, _: &mut Context<'_>, _: Duration) -> Result<WorkerCompletion<E>, E> {
        assert!(self.stopping);
        self.joins += 1;
        if !(self.joining)(self.joins) {
            return Ok(WorkerCompletion::Pending);
        }
        Ok(WorkerCompletion::Joined(
            self.terminal.take().unwrap_or(Ok(())),
        ))
    }
}

struct WithResource<H, R> {
    host: H,
    resource: R,
}

impl<H, R> Host for WithResource<H, R>
where
    H: Host,
    R: ManagedResource<Error = H::Error>,
{
    type Error = H::Error;
    type Activity = H::Activity;
    type Resources = Managed<R, H::Resources>;

    fn name(&self) -> &'static str {
        self.host.name()
    }

    fn validate(&self) -> Result<(), Self::Error> {
        self.host.validate()
    }

    fn exclusive_claims(&self) -> Vec<ExclusiveClaim> {
        self.host.exclusive_claims()
    }

    fn into_resources(self) -> ResourceGroup<Self::Activity, Self::Resources> {
        self.host
            .into_resources()
            .with_managed_resource("journal", self.resource)
    }
}

struct Journal<'a, E, F> {
    acquired: &'a Cell<usize>,
    released: &'a Cell<usize>,
    before_release: F,
    error: PhantomData<fn() -> E>,
}

impl<E, F: FnMut()> ManagedResource for Journal<'_, E, F> {
    type Error = E;

    fn poll_acquire(&mut self, _: &mut Context<'_>) -> Poll<Result<(), E>> {
        self.acquired.set(self.acquired.get() + 1);
        Poll::Ready(Ok(()))
    }

    fn poll_release(
        &mut self,
        _: &mut Context<'_>,
        _: Duration,
    ) -> Result<ReleaseCompletion<E>, E> {
        (self.before_release)();
        self.released.set(self.released.get() + 1);
        Ok(ReleaseCompletion::Released(Ok(())))
    }
}

#[test]
fn plans_are_inert_and_validate_exact_claims_and_bounded_configuration() {
    let reserved = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let address = reserved.local_addr().unwrap();
    let first = UdpHost::<io::Error>::new()
        .with_name("first")
        .with_address(address);
    let second = UdpHost::<io::Error>::new()
        .with_name("second")
        .with_address(address);
    let one = first.handle();
    let two = second.handle();
    let result = HostGroup::new().with_host(first).with_host(second).build();
    match result {
        Err(HostBuildError::ConflictingClaim {
            first,
            second,
            claim,
        }) => {
            assert_eq!((first, second), ("first", "second"));
            assert_eq!(claim.kind, "udp-endpoint");
            assert_eq!(claim.key, address.to_string());
        }
        _ => panic!("same explicit UDP endpoint must conflict before acquisition"),
    }
    for handle in [one, two] {
        assert!(handle.local_addr().is_none());
        assert!(!handle.is_ready() && !handle.is_admitting());
    }
    for capacity in [0, MAX_DATAGRAM_BYTES + 1] {
        assert!(
            HostGroup::new()
                .with_host(UdpHost::<io::Error>::new().with_max_datagram_bytes(capacity))
                .build()
                .is_err()
        );
    }
    for budget in [0, MAX_PACKETS_PER_POLL + 1] {
        assert!(
            HostGroup::new()
                .with_host(UdpHost::<io::Error>::new().with_packet_budget(budget))
                .build()
                .is_err()
        );
    }
    fn assert_send<T: Send>() {}
    assert_send::<UdpEndpoint>();
}

#[test]
fn two_managed_loopback_hosts_reply_on_the_callers_thread_with_borrowed_state() {
    let caller = thread::current().id();
    let packets = Rc::new(RefCell::new(Vec::new()));
    let borrowed = String::from("local state");
    let peer = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    peer.set_nonblocking(true).unwrap();
    let peer_address = peer.local_addr().unwrap();
    let first_packets = packets.clone();
    let first = UdpHost::<io::Error>::new()
        .with_name("first")
        .with_max_datagram_bytes(16)
        .with_packet_budget(1)
        .on_datagram(|packet| {
            assert_eq!(thread::current().id(), caller);
            assert_eq!(borrowed, "local state");
            first_packets
                .borrow_mut()
                .push((packet.bytes().to_vec(), packet.sender()));
            assert_eq!(packet.reply(b"one")?, SendOutcome::Sent(3));
            Ok(())
        });
    let second_packets = packets.clone();
    let second = UdpHost::<io::Error>::new()
        .with_name("second")
        .with_max_datagram_bytes(16)
        .on_datagram(|packet| {
            assert_eq!(thread::current().id(), caller);
            second_packets
                .borrow_mut()
                .push((packet.bytes().to_vec(), packet.sender()));
            assert_eq!(
                packet.send_to(packet.sender(), b"two")?,
                SendOutcome::Sent(3)
            );
            Ok(())
        });
    let one = first.handle();
    let two = second.handle();
    let addresses = Cell::new(None);
    let responses = RefCell::new(Vec::new());
    let mut sent = false;
    let inspect = probe(|| -> io::Result<bool> {
        assert!(one.is_ready() && two.is_ready());
        assert!(one.is_admitting() && two.is_admitting());
        if !sent {
            let a = one.local_addr().unwrap();
            let b = two.local_addr().unwrap();
            assert_ne!(a, b);
            addresses.set(Some((a, b)));
            peer.send_to(&[0, 0xff], a)?;
            peer.send_to(b"second", b)?;
            sent = true;
        }
        let mut bytes = [0; 16];
        for _ in 0..2 {
            match peer.recv_from(&mut bytes) {
                Ok((length, sender)) => {
                    responses
                        .borrow_mut()
                        .push((bytes[..length].to_vec(), sender));
                }
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => break,
                Err(error) => return Err(error),
            }
        }
        Ok(responses.borrow().len() == 2)
    });
    let mut hosts = HostGroup::new()
        .with_host(first)
        .with_host(second)
        .with_host(inspect)
        .build()
        .unwrap();
    assert!(one.local_addr().is_none() && two.local_addr().is_none());
    hosts.run().unwrap();
    assert_eq!(hosts.phase(), Phase::Stopped);
    assert_eq!(packets.borrow().len(), 2);
    assert!(packets.borrow().contains(&(vec![0, 0xff], peer_address)));
    assert!(
        packets
            .borrow()
            .contains(&(b"second".to_vec(), peer_address))
    );
    let (a, b) = addresses.get().unwrap();
    assert!(responses.borrow().contains(&(b"one".to_vec(), a)));
    assert!(responses.borrow().contains(&(b"two".to_vec(), b)));
    for handle in [one.clone(), two.clone()] {
        assert!(handle.local_addr().is_none());
        assert!(!handle.is_ready() && !handle.is_admitting());
    }
    let rebound = (UdpSocket::bind(a).unwrap(), UdpSocket::bind(b).unwrap());
    drop(rebound);
}

#[test]
fn managed_receive_budget_and_permanent_fence_control_callback_sending() {
    let called = Cell::new(0);
    let peer = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let host = UdpHost::<io::Error>::new()
        .with_max_datagram_bytes(4)
        .with_packet_budget(2)
        .on_datagram(|packet| {
            called.set(called.get() + 1);
            assert_eq!(
                packet.reply(b"oversize").unwrap_err().kind(),
                io::ErrorKind::InvalidInput
            );
            assert_eq!(
                packet
                    .send_to(SocketAddr::from((Ipv4Addr::LOCALHOST, 0)), b"ok")
                    .unwrap_err()
                    .kind(),
                io::ErrorKind::InvalidInput
            );
            Ok(())
        });
    let handle = host.handle();
    let address = Cell::new(None);
    let resources = host.into_resources().map_activity(|mut activity| {
        let mut cx = Context::from_waker(Waker::noop());
        activity.prepare_driver(HostRunMode::Blocking).unwrap();
        assert!(matches!(activity.poll_start(&mut cx), Poll::Ready(Ok(()))));
        assert!(matches!(activity.poll_ready(&mut cx), Poll::Ready(Ok(()))));
        let bound = handle.local_addr().unwrap();
        address.set(Some(bound));
        assert!(handle.is_ready() && !handle.is_admitting());
        for packet in [b"one", b"two", b"tri"] {
            peer.send_to(packet, bound).unwrap();
        }
        assert!(activity.poll_finished(&mut cx).is_pending());
        assert_eq!(called.get(), 0, "no callbacks before group admission");
        assert_eq!(activity.open_admission().unwrap(), AdmissionOutcome::Opened);
        assert!(activity.poll_finished(&mut cx).is_pending());
        assert_eq!(
            called.get(),
            2,
            "one poll obeys the configured packet budget"
        );
        activity.close_admission(&mut cx).unwrap();
        assert!(!handle.is_admitting());
        assert_eq!(activity.open_admission().unwrap(), AdmissionOutcome::Closed);
        assert!(activity.poll_finished(&mut cx).is_pending());
        assert_eq!(called.get(), 2, "fenced queued data cannot enter callbacks");
        activity.request_shutdown(&mut cx).unwrap();
        assert!(activity.poll_finished(&mut cx).is_ready());
        assert_eq!(handle.local_addr(), Some(bound), "retain socket until join");
        assert!(matches!(
            activity.poll_join(&mut cx, Duration::ZERO).unwrap(),
            WorkerCompletion::Joined(Ok(()))
        ));
        assert!(handle.local_addr().is_none());
        activity
    });
    drop(resources);
    let rebound = UdpSocket::bind(address.get().unwrap()).unwrap();
    drop(rebound);
}

#[test]
fn weak_metadata_clones_do_not_keep_a_dropped_original_socket_alive() {
    let host: UdpHost = UdpHost::new();
    let handle = host.handle();
    let cloned = handle.clone();
    let resources = host.into_resources().map_activity(|mut activity| {
        activity.prepare_driver(HostRunMode::Blocking).unwrap();
        assert!(matches!(
            activity.poll_start(&mut Context::from_waker(Waker::noop())),
            Poll::Ready(Ok(()))
        ));
        activity
    });
    let address = handle.local_addr().unwrap();
    drop(resources); // ordinary RAII close; not a managed completion claim
    for handle in [handle, cloned] {
        assert!(handle.local_addr().is_none());
        assert!(!handle.is_ready() && !handle.is_admitting());
    }
    let rebound = UdpSocket::bind(address).unwrap();
    drop(rebound);
}

#[test]
fn async_mode_is_rejected_before_any_managed_resource_or_socket_acquisition() {
    let acquired = Cell::new(0);
    let released = Cell::new(0);
    let udp: UdpHost = UdpHost::new();
    let handle = udp.handle();
    let host = WithResource {
        host: udp,
        resource: Journal {
            acquired: &acquired,
            released: &released,
            before_release: || {},
            error: PhantomData,
        },
    };
    let mut hosts = HostGroup::new().with_host(host).build().unwrap();
    let deadline = Instant::now() + LIMIT;
    while hosts
        .poll_run(&mut Context::from_waker(Waker::noop()))
        .is_pending()
    {
        assert!(Instant::now() < deadline, "inert rollback deadline");
    }
    let failure = hosts.failures().primary().unwrap();
    assert_eq!(failure.operation, HostOperation::PrepareDriver);
    assert_eq!(failure.error().kind(), io::ErrorKind::Unsupported);
    assert_eq!((acquired.get(), released.get()), (0, 0));
    assert!(handle.local_addr().is_none());
    assert!(!handle.is_ready() && !handle.is_admitting());
}

#[test]
fn later_real_bind_failure_rolls_back_the_first_original_socket() {
    let reservation = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let first_address = reservation.local_addr().unwrap();
    let occupied = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let occupied_address = occupied.local_addr().unwrap();
    drop(reservation);
    let first = UdpHost::<io::Error>::new()
        .with_name("first")
        .with_address(first_address);
    let first_handle = first.handle();
    let second = UdpHost::<io::Error>::new()
        .with_name("blocked")
        .with_address(occupied_address);
    let second_handle = second.handle();
    let mut hosts = HostGroup::new()
        .with_host(first)
        .with_host(second)
        .build()
        .unwrap();
    assert!(hosts.run().is_err());
    let failure = hosts.failures().primary().unwrap();
    assert_eq!(failure.host, "blocked");
    assert_eq!(failure.operation, HostOperation::Start);
    assert_eq!(failure.error().kind(), io::ErrorKind::AddrInUse);
    assert_eq!(occupied.local_addr().unwrap(), occupied_address);
    assert!(first_handle.local_addr().is_none());
    assert!(second_handle.local_addr().is_none());
    let rebound = UdpSocket::bind(first_address).unwrap();
    drop(rebound);
}

#[derive(Debug)]
enum LocalError<'a> {
    Io(io::Error),
    Packet { borrowed: &'a str, identity: Rc<()> },
}

impl From<io::Error> for LocalError<'_> {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

struct DropWitness<'a>(&'a Cell<usize>);
impl Drop for DropWitness<'_> {
    fn drop(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

#[test]
fn original_local_borrowed_error_and_callback_survive_other_pending_join() {
    let borrowed = String::from("original borrowed callback failure");
    let identity = Rc::new(());
    let acquired = Cell::new(0);
    let released = Cell::new(0);
    let callback_drops = Cell::new(0);
    let other_joined = Cell::new(false);
    let join_polls = Cell::new(0);
    let witness = DropWitness(&callback_drops);
    let mut original = Some(LocalError::Packet {
        borrowed: &borrowed,
        identity: identity.clone(),
    });
    let udp = UdpHost::<LocalError<'_>>::new().on_datagram(move |_| {
        let _retain_witness = &witness;
        Err(original.take().expect("exact callback error returned once"))
    });
    let handle = udp.handle();
    let address = Cell::new(None);
    let host = WithResource {
        host: udp,
        resource: Journal {
            acquired: &acquired,
            released: &released,
            before_release: || {
                assert!(other_joined.get(), "global original-completion barrier");
                assert_eq!(callback_drops.get(), 1, "retire callback before dependency");
            },
            error: PhantomData,
        },
    };
    let peer = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0)).unwrap();
    let mut sent = false;
    let observer = probe(|| -> Result<bool, LocalError<'_>> {
        if !sent {
            assert!(handle.is_admitting());
            let bound = handle.local_addr().unwrap();
            address.set(Some(bound));
            peer.send_to(b"fail", bound)?;
            sent = true;
        }
        Ok(false)
    })
    .with_join(|attempt| {
        join_polls.set(attempt);
        assert_eq!(
            released.get(),
            0,
            "dependency retained through pending join"
        );
        assert_eq!(
            callback_drops.get(),
            0,
            "callback retained through group joins"
        );
        if attempt == 1 {
            false
        } else {
            other_joined.set(true);
            true
        }
    });
    let mut hosts = HostGroup::new()
        .with_host(host)
        .with_host(observer)
        .build()
        .unwrap();
    assert!(hosts.run().is_err());
    assert_eq!(hosts.phase(), Phase::Failed);
    assert!(join_polls.get() >= 2);
    let failure = hosts.failures().primary().unwrap();
    assert_eq!(failure.operation, HostOperation::Join);
    match failure.error() {
        LocalError::Packet {
            borrowed: actual,
            identity: actual_identity,
        } => {
            assert_eq!(*actual, borrowed);
            assert!(Rc::ptr_eq(actual_identity, &identity));
        }
        LocalError::Io(error) => panic!("unexpected replacement IO error: {error}"),
    }
    assert_eq!(
        (acquired.get(), released.get(), callback_drops.get()),
        (1, 1, 1)
    );
    assert!(handle.local_addr().is_none());
    let rebound = UdpSocket::bind(address.get().unwrap()).unwrap();
    drop(rebound);
}
