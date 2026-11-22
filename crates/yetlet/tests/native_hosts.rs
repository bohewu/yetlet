use std::{
    cell::Cell,
    io,
    rc::Rc,
    task::{Context, Poll, Waker},
    time::{Duration, Instant},
};
use yetlet::{
    Host, HostGroup, ResourceGroup,
    composition::{
        AdmissionOutcome, HostActivity, HostBuildError, HostOperation, Managed, ManagedResource,
        ReleaseCompletion, StartStatus, WorkerCompletion,
    },
    native_winit::{NativeEvent, NativeHost, WindowPhase, winit::window::Window},
};

#[test]
fn multiple_windows_are_inert_and_duplicate_root_loops_reject_before_acquisition() {
    let native = NativeHost::<io::Error>::new(Window::default_attributes().with_title("first"))
        .with_window(Window::default_attributes().with_title("second"));
    let first = native.handle();
    assert_eq!(first.window_count(), 2);
    assert!(!first.is_ready());
    assert!(!first.is_admitting());
    for index in 0..2 {
        assert_eq!(first.window_phase(index), Some(WindowPhase::Prepared));
        assert!(first.shared_window(index).is_none());
        assert_eq!(first.redraw_event_count(index), Some(0));
    }

    let second = NativeHost::new(Window::default_attributes()).with_name("second-root");
    let second_handle = second.handle();
    let result = HostGroup::new().with_host(native).with_host(second).build();
    match result {
        Err(HostBuildError::ConflictingClaim { claim, .. }) => {
            assert_eq!(claim.kind, "winit-event-loop");
            assert_eq!(claim.key, "process");
        }
        _ => panic!("two independent native loops must reject at build"),
    }
    assert!(first.shared_window(0).is_none());
    assert!(second_handle.shared_window(0).is_none());
}

struct Idle;

impl HostActivity for Idle {
    type Error = io::Error;

    fn poll_start(&mut self, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        Poll::Ready(Ok(()))
    }

    fn open_admission(&mut self) -> io::Result<AdmissionOutcome> {
        Ok(AdmissionOutcome::Opened)
    }

    fn close_admission(&mut self, _: &mut Context<'_>) -> io::Result<()> {
        Ok(())
    }

    fn request_shutdown(&mut self, _: &mut Context<'_>) -> io::Result<()> {
        Ok(())
    }

    fn poll_join(
        &mut self,
        _: &mut Context<'_>,
        _: Duration,
    ) -> io::Result<WorkerCompletion<io::Error>> {
        Ok(WorkerCompletion::Joined(Ok(())))
    }
}

struct ProbeResource {
    acquisitions: Rc<Cell<usize>>,
    releases: Rc<Cell<usize>>,
}

impl ManagedResource for ProbeResource {
    type Error = io::Error;

    fn poll_acquire(&mut self, _: &mut Context<'_>) -> Poll<io::Result<()>> {
        self.acquisitions.set(self.acquisitions.get() + 1);
        Poll::Ready(Ok(()))
    }

    fn poll_release(
        &mut self,
        _: &mut Context<'_>,
        _: Duration,
    ) -> io::Result<ReleaseCompletion<io::Error>> {
        self.releases.set(self.releases.get() + 1);
        Ok(ReleaseCompletion::Released(Ok(())))
    }
}

struct EarlierHost(ProbeResource);

impl Host for EarlierHost {
    type Error = io::Error;
    type Activity = Idle;
    type Resources = Managed<ProbeResource, ()>;

    fn name(&self) -> &'static str {
        "earlier"
    }

    fn into_resources(self) -> ResourceGroup<Self::Activity, Self::Resources> {
        ResourceGroup::new(Idle).with_managed_resource("probe", self.0)
    }
}

#[test]
fn async_mode_rejects_native_before_even_earlier_managed_acquisition() {
    let acquisitions = Rc::new(Cell::new(0));
    let releases = Rc::new(Cell::new(0));
    let native = NativeHost::new(Window::default_attributes());
    let handle = native.handle();
    let mut group = HostGroup::new()
        .with_host(EarlierHost(ProbeResource {
            acquisitions: acquisitions.clone(),
            releases: releases.clone(),
        }))
        .with_host(native)
        .build()
        .unwrap();
    let mut cx = Context::from_waker(Waker::noop());
    assert_eq!(
        group.poll_start(&mut cx),
        Poll::Ready(StartStatus::Stopping)
    );
    let failure = group.failures().primary().unwrap();
    assert_eq!(failure.operation, HostOperation::PrepareDriver);
    assert_eq!(failure.host, "native");
    assert_eq!(failure.error().kind(), io::ErrorKind::Unsupported);
    assert_eq!(acquisitions.get(), 0);
    assert_eq!(releases.get(), 0);
    assert!(handle.shared_window(0).is_none());
    assert!(
        group
            .poll_shutdown(&mut cx, Instant::now() + Duration::from_secs(1))
            .is_ready()
    );
    assert_eq!(releases.get(), 0);
}

#[test]
fn native_callback_and_error_can_borrow_non_send_application_state() {
    #[derive(Debug)]
    enum LocalError<'a> {
        Io(io::Error),
        Application(&'a str, Rc<()>),
    }

    impl From<io::Error> for LocalError<'_> {
        fn from(error: io::Error) -> Self {
            Self::Io(error)
        }
    }

    let message = String::from("borrowed application error");
    let identity = Rc::new(());
    let calls = Cell::new(0);
    let native = NativeHost::new(Window::default_attributes()).on_event(|_, event| {
        if matches!(event, NativeEvent::Tick) {
            calls.set(calls.get() + 1);
        }
        Err(LocalError::Application(message.as_str(), identity.clone()))
    });
    let mut group = HostGroup::new().with_host(native).build().unwrap();
    let mut cx = Context::from_waker(Waker::noop());
    assert_eq!(
        group.poll_start(&mut cx),
        Poll::Ready(StartStatus::Stopping)
    );
    match group.failures().primary().unwrap().error() {
        LocalError::Io(error) => assert_eq!(error.kind(), io::ErrorKind::Unsupported),
        LocalError::Application(message, identity) => {
            panic!("callback ran before native admission: {message} {identity:?}");
        }
    }
    assert!(
        group
            .poll_shutdown(&mut cx, Instant::now() + Duration::from_secs(1))
            .is_ready()
    );
    drop(group);
    assert_eq!(calls.get(), 0);
}

#[test]
fn invalid_poll_interval_is_a_pure_build_error() {
    for interval in [Duration::ZERO, Duration::MAX] {
        let native =
            NativeHost::<io::Error>::new(Window::default_attributes()).with_poll_interval(interval);
        let handle = native.handle();
        match HostGroup::new().with_host(native).build() {
            Err(HostBuildError::InvalidHost(failure)) => {
                assert_eq!(failure.operation, HostOperation::Validate);
                assert_eq!(failure.error().kind(), io::ErrorKind::InvalidInput);
            }
            _ => panic!("invalid native polling interval must reject at build"),
        }
        assert!(handle.shared_window(0).is_none());
    }
}
