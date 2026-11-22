use super::{WindowPhase, WindowSlot};
use crate::composition::{
    AdmissionOutcome, BlockingDriver, Driven, ExclusiveClaim, Host, HostActivity, HostRunMode,
    Managed, ManagedResource, ReleaseCompletion, ResourceGroup, ShutdownHandle, WorkerCompletion,
};
use std::{
    any::Any,
    cell::RefCell,
    collections::VecDeque,
    error::Error,
    fmt, io,
    marker::PhantomData,
    panic::{AssertUnwindSafe, catch_unwind},
    rc::Rc,
    sync::{Arc, Mutex},
    task::{Context, Poll, Wake, Waker},
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop, EventLoopProxy},
    window::{Window, WindowAttributes, WindowId},
};

/// An ordinary native application event, delivered only after group admission.
#[derive(Debug)]
pub enum NativeEvent {
    Window {
        index: usize,
        event: WindowEvent,
    },
    /// A bounded caller-thread opportunity for application/network progress.
    /// This is not a guaranteed frame rate or a background executor.
    Tick,
}

struct Shared {
    windows: Vec<WindowSlot>,
    acquire_requested: bool,
    windows_created: bool,
    creation_error: Option<io::Error>,
    admitting: bool,
    stop_requested: bool,
    released: bool,
    shutdown: Option<ShutdownHandle>,
    events: VecDeque<NativeEvent>,
}

impl Shared {
    fn ready(&self) -> bool {
        !self.stop_requested
            && !self.released
            && !self.windows.is_empty()
            && self.windows.iter().all(WindowSlot::is_ready)
    }

    fn fence(&mut self) {
        self.admitting = false;
        self.events.clear();
    }
}

/// Local observations and explicit requests for one managed native window group.
///
/// This handle does not transfer cleanup ownership. A shared_window clone is an
/// actual Window owner: retain it in a later managed renderer/resource and
/// release it before native window finalization. Merely dropping the HostGroup
/// is not managed shutdown.
#[derive(Clone)]
pub struct NativeHandle(Rc<RefCell<Shared>>);

impl NativeHandle {
    pub fn window_count(&self) -> usize {
        self.0.borrow().windows.len()
    }

    pub fn window_phase(&self, index: usize) -> Option<WindowPhase> {
        self.0.borrow().windows.get(index).map(WindowSlot::phase)
    }

    pub fn redraw_event_count(&self, index: usize) -> Option<u64> {
        self.0
            .borrow()
            .windows
            .get(index)
            .map(WindowSlot::redraw_event_count)
    }

    /// Current usability of every window after a real visible usable redraw.
    /// Resize or occlusion can temporarily make this false after Core has
    /// retained the startup readiness observation. It does not close group
    /// admission and says nothing about renderer readiness.
    pub fn is_ready(&self) -> bool {
        self.0.borrow().ready()
    }

    pub fn is_admitting(&self) -> bool {
        let state = self.0.borrow();
        state.admitting && !state.stop_requested && !state.released
    }

    pub fn shared_window(&self, index: usize) -> Option<Arc<Window>> {
        self.0.borrow().windows.get(index)?.shared_window()
    }

    pub fn request_redraw(&self, index: usize) -> io::Result<()> {
        self.0
            .borrow()
            .windows
            .get(index)
            .ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "unknown native window index")
            })?
            .request_redraw()
    }

    /// Request coordinated shutdown of the entire HostGroup.
    pub fn request_shutdown(&self) {
        let shutdown = {
            let mut state = self.0.borrow_mut();
            state.stop_requested = true;
            state.fence();
            state.shutdown.clone()
        };
        if let Some(shutdown) = shutdown {
            shutdown.request_shutdown();
        }
    }
}

fn ignore_event<E>(_: &NativeHandle, _: NativeEvent) -> Result<(), E> {
    Ok(())
}

/// An inert native Host with one root-owned caller-thread Winit loop.
///
/// Add windows with with_window instead of adding a second NativeHost. run must
/// be called on the platform entry thread; run_async is rejected before any Host
/// acquires resources. Winit's process-wide loop limit still applies to loops
/// created outside Yetlet, and a completed loop is never recreated.
///
/// The first managed dependency is the window group. A custom Host can append
/// renderer resources through into_resources().with_managed_resource(...);
/// reverse finalization then retires the activity, later renderers, and windows.
/// No renderer, Web server, runtime, thread or process is implicitly installed.
///
/// E is inferred from the surrounding HostGroup, including WebError in a mixed
/// Web/native group. For native-only use, HostGroup::<std::io::Error>::new()
/// selects the ordinary I/O error type. Application callbacks may borrow local
/// values and use non-Send errors; they run synchronously on this same thread.
///
/// Closing/destroying any managed window, or suspension, requests whole-group
/// shutdown. Ordinary callbacks stop immediately; cleanup continues inside the
/// existing Winit callbacks. This initial adapter does not restart after suspend.
pub struct NativeHost<E = io::Error, F = fn(&NativeHandle, NativeEvent) -> Result<(), E>> {
    name: &'static str,
    handle: NativeHandle,
    callback: F,
    poll_interval: Duration,
    _error: PhantomData<fn() -> E>,
}

impl<E> NativeHost<E> {
    pub fn new(attributes: WindowAttributes) -> Self {
        Self {
            name: "native",
            handle: NativeHandle(Rc::new(RefCell::new(Shared {
                windows: vec![WindowSlot::new(attributes)],
                acquire_requested: false,
                windows_created: false,
                creation_error: None,
                admitting: false,
                stop_requested: false,
                released: false,
                shutdown: None,
                events: VecDeque::new(),
            }))),
            callback: ignore_event::<E>,
            poll_interval: Duration::from_millis(16),
            _error: PhantomData,
        }
    }
}

impl<E, F> NativeHost<E, F> {
    pub fn with_name(mut self, name: &'static str) -> Self {
        self.name = name;
        self
    }

    /// Prepare another window under this Host's existing root-loop description.
    pub fn with_window(self, attributes: WindowAttributes) -> Self {
        self.handle
            .0
            .borrow_mut()
            .windows
            .push(WindowSlot::new(attributes));
        self
    }

    /// Bound idle polling latency for caller-driven work such as UDP.
    /// The interval must be nonzero; event/waker notifications can run sooner.
    pub fn with_poll_interval(mut self, interval: Duration) -> Self {
        self.poll_interval = interval;
        self
    }

    pub fn on_event<G>(self, callback: G) -> NativeHost<E, G>
    where
        G: FnMut(&NativeHandle, NativeEvent) -> Result<(), E>,
    {
        NativeHost {
            name: self.name,
            handle: self.handle,
            callback,
            poll_interval: self.poll_interval,
            _error: PhantomData,
        }
    }

    pub fn handle(&self) -> NativeHandle {
        self.handle.clone()
    }
}

impl<E, F> Host for NativeHost<E, F>
where
    E: From<io::Error>,
    F: FnMut(&NativeHandle, NativeEvent) -> Result<(), E>,
{
    type Error = E;
    type Activity = NativeActivity<E, F>;
    type Resources = Driven<NativeDriver<E>, Managed<NativeWindows<E>, ()>>;

    fn name(&self) -> &'static str {
        self.name
    }

    fn validate(&self) -> Result<(), E> {
        if self.poll_interval.is_zero() || Instant::now().checked_add(self.poll_interval).is_none()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "native polling interval must be nonzero and fit the caller clock",
            )
            .into());
        }
        Ok(())
    }

    fn exclusive_claims(&self) -> Vec<ExclusiveClaim> {
        vec![ExclusiveClaim::new("winit-event-loop", "process")]
    }

    fn into_resources(self) -> ResourceGroup<Self::Activity, Self::Resources> {
        let windows = NativeWindows {
            handle: self.handle.clone(),
            _error: PhantomData,
        };
        let driver = NativeDriver {
            handle: self.handle.clone(),
            poll_interval: self.poll_interval,
            attempted: false,
            _error: PhantomData,
        };
        ResourceGroup::new(NativeActivity {
            handle: self.handle,
            callback: self.callback,
            started: false,
            finished: false,
            completion: None,
        })
        .with_managed_resource("windows", windows)
        .with_blocking_driver(driver)
    }
}

fn prepare_native(mode: HostRunMode) -> io::Result<()> {
    match mode {
        HostRunMode::Blocking => Ok(()),
        HostRunMode::Async => Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "NativeHost requires synchronous HostGroup::run on the platform entry thread",
        )),
    }
}

/// Inferred default Window dependency. Use NativeHost::into_resources to extend it.
#[doc(hidden)]
pub struct NativeWindows<E> {
    handle: NativeHandle,
    _error: PhantomData<fn() -> E>,
}

impl<E: From<io::Error>> ManagedResource for NativeWindows<E> {
    type Error = E;

    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), E> {
        prepare_native(mode).map_err(Into::into)
    }

    fn poll_acquire(&mut self, _: &mut Context<'_>) -> Poll<Result<(), E>> {
        let result = {
            let mut state = self.handle.0.borrow_mut();
            if let Some(error) = state.creation_error.take() {
                Poll::Ready(Err(error))
            } else if state.windows_created {
                Poll::Ready(Ok(()))
            } else {
                state.acquire_requested = true;
                Poll::Pending
            }
        };
        result.map(|result| result.map_err(E::from))
    }

    fn poll_release(
        &mut self,
        _: &mut Context<'_>,
        _: Duration,
    ) -> Result<ReleaseCompletion<E>, E> {
        self.release_windows()
            .map(|completion| match completion {
                ReleaseCompletion::Pending => ReleaseCompletion::Pending,
                ReleaseCompletion::Released(result) => {
                    ReleaseCompletion::Released(result.map_err(E::from))
                }
            })
            .map_err(E::from)
    }
}

impl<E> NativeWindows<E> {
    fn release_windows(&mut self) -> io::Result<ReleaseCompletion<io::Error>> {
        let mut state = self.handle.0.borrow_mut();
        state.fence();
        for window in state.windows.iter_mut().rev() {
            match window.close() {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                    // The root loop keeps ticking while a later external owner
                    // completes. Keep the exact original WindowSlot meanwhile.
                    return Ok(ReleaseCompletion::Pending);
                }
                Err(error) => return Err(error),
            }
        }
        state.released = true;
        state.acquire_requested = false;
        state.windows_created = false;
        state.shutdown = None;
        // A shutdown request may have bypassed the next acquire poll. Preserve
        // that still-unreported original creation error at terminal release.
        let outcome = state.creation_error.take().map_or(Ok(()), Err);
        Ok(ReleaseCompletion::Released(outcome))
    }
}

/// Inferred caller-local activity. Its original completion is callback quiescence;
/// the root event-loop driver separately survives every dependency finalizer.
#[doc(hidden)]
pub struct NativeActivity<E, F> {
    handle: NativeHandle,
    callback: F,
    started: bool,
    finished: bool,
    completion: Option<Result<(), E>>,
}

impl<E, F> HostActivity for NativeActivity<E, F>
where
    E: From<io::Error>,
    F: FnMut(&NativeHandle, NativeEvent) -> Result<(), E>,
{
    type Error = E;

    fn prepare_driver(&mut self, mode: HostRunMode) -> Result<(), E> {
        prepare_native(mode).map_err(Into::into)
    }

    fn poll_start(&mut self, _: &mut Context<'_>) -> Poll<Result<(), E>> {
        self.started = true;
        Poll::Ready(Ok(()))
    }

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), E>> {
        let state = self.handle.0.borrow();
        if state.ready() || state.stop_requested {
            Poll::Ready(Ok(()))
        } else {
            Poll::Pending
        }
    }

    fn poll_finished(&mut self, _: &mut Context<'_>) -> Poll<()> {
        if !self.started {
            return Poll::Pending;
        }
        if self.finished {
            return Poll::Ready(());
        }
        loop {
            let event = {
                let mut state = self.handle.0.borrow_mut();
                if state.stop_requested {
                    state.fence();
                    self.finished = true;
                    self.completion = Some(Ok(()));
                    return Poll::Ready(());
                }
                if !state.admitting {
                    return Poll::Pending;
                }
                state.events.pop_front()
            };
            let Some(event) = event else {
                return Poll::Pending;
            };
            // No native-state borrow is held across application code.
            if let Err(error) = (self.callback)(&self.handle, event) {
                self.completion = Some(Err(error));
                self.finished = true;
                self.handle.request_shutdown();
                return Poll::Ready(());
            }
        }
    }

    fn open_admission(&mut self) -> Result<AdmissionOutcome, E> {
        let mut state = self.handle.0.borrow_mut();
        // Core already retained this Host's startup readiness observation.
        // Resize/occlusion can temporarily invalidate Window usability while a
        // sibling starts; only a real stop or final release closes admission.
        if state.stop_requested || state.released {
            return Ok(AdmissionOutcome::Closed);
        }
        state.admitting = true;
        // Startup redraws establish Window readiness. Request new redraws for
        // ordinary rendering after the whole group has opened admission.
        for window in &state.windows {
            window.request_redraw().map_err(E::from)?;
        }
        Ok(AdmissionOutcome::Opened)
    }

    fn close_admission(&mut self, _: &mut Context<'_>) -> Result<(), E> {
        self.handle.0.borrow_mut().fence();
        Ok(())
    }

    fn request_shutdown(&mut self, _: &mut Context<'_>) -> Result<(), E> {
        self.handle.request_shutdown();
        self.finished = true;
        if self.completion.is_none() {
            self.completion = Some(Ok(()));
        }
        Ok(())
    }

    fn poll_join(&mut self, _: &mut Context<'_>, _: Duration) -> Result<WorkerCompletion<E>, E> {
        if !self.finished {
            return Ok(WorkerCompletion::Pending);
        }
        // Application callbacks have returned on this exact caller thread.
        // Their stored values remain owned until final activity release.
        Ok(WorkerCompletion::Joined(
            self.completion.take().unwrap_or(Ok(())),
        ))
    }
}

/// The original panic payload from Winit EventLoop builder initialization.
///
/// Winit documents an off-entry-thread construction panic. Only that builder
/// boundary is caught; user callbacks and lifecycle code are not caught, and the
/// process panic hook is never changed. The source remains inspectable through
/// the returned io::Error, including custom payload ownership. Formatting uses
/// an owned message captured before inspection, without locking the payload.
pub struct EventLoopPanic {
    message: String,
    payload: Mutex<Box<dyn Any + Send>>,
}

impl EventLoopPanic {
    /// Inspect the original payload without waiting for another inspection.
    ///
    /// Returns None while the payload is already held, including nested calls.
    /// A previous inspector panic does not discard the original payload: its
    /// poisoned guard is recovered. Formatting this error never locks the payload.
    pub fn try_with_payload<R>(&self, inspect: impl FnOnce(&(dyn Any + Send)) -> R) -> Option<R> {
        let payload = match self.payload.try_lock() {
            Ok(payload) => payload,
            Err(std::sync::TryLockError::Poisoned(error)) => error.into_inner(),
            Err(std::sync::TryLockError::WouldBlock) => return None,
        };
        Some(inspect(payload.as_ref()))
    }
}

impl fmt::Debug for EventLoopPanic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("EventLoopPanic")
            .field("message", &self.message)
            .finish_non_exhaustive()
    }
}

impl fmt::Display for EventLoopPanic {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Winit event-loop construction panicked: {}",
            self.message
        )
    }
}

impl Error for EventLoopPanic {}

fn capture_builder<T>(build: impl FnOnce() -> io::Result<T>) -> io::Result<T> {
    catch_unwind(AssertUnwindSafe(build)).map_err(|payload| {
        let message = if let Some(message) = payload.downcast_ref::<String>() {
            message.clone()
        } else if let Some(message) = payload.downcast_ref::<&'static str>() {
            (*message).to_owned()
        } else {
            "non-string panic payload".to_owned()
        };
        io::Error::other(EventLoopPanic {
            message,
            payload: Mutex::new(payload),
        })
    })?
}

/// Only used before the first lifecycle progress callback, while all Host
/// descriptors remain inert. It never drives cleanup after an active loop exits.
fn begin_loop<T>(
    shutdown: &ShutdownHandle,
    progress: &mut dyn FnMut(&mut Context<'_>) -> Poll<()>,
    build: impl FnOnce() -> io::Result<T>,
) -> io::Result<Option<T>> {
    if shutdown.is_requested() {
        let _ = progress(&mut Context::from_waker(Waker::noop()));
        return Ok(None);
    }
    match capture_builder(build) {
        Ok(event_loop) => Ok(Some(event_loop)),
        Err(error) => {
            shutdown.request_shutdown();
            // Unattempted descriptors can normally retire in this single pass.
            // If it remains Pending, returning the original error lets Core
            // retain DriverReturnedEarly; there is no parking/retry loop here.
            let _ = progress(&mut Context::from_waker(Waker::noop()));
            Err(error)
        }
    }
}

/// Inferred one-shot root driver. No replacement loop is created after exit.
///
/// Normal shutdown finishes Core cleanup inside native callbacks before exit.
/// If the backend exits early, Core retains this driver, original failures and
/// unfinished resources. This portable run_app driver cannot resume a consumed
/// Winit loop; a later run returns an error without recreating it. Core permits
/// further lifecycle progress only through that same retained driver. When this
/// one-shot driver cannot resume, its obligations remain retained and incomplete;
/// no cleanup thread or successful native-dependent finalization is inferred.
#[doc(hidden)]
pub struct NativeDriver<E> {
    handle: NativeHandle,
    poll_interval: Duration,
    attempted: bool,
    _error: PhantomData<fn() -> E>,
}

#[derive(Debug)]
struct WakeEvent;

struct NativeWake(EventLoopProxy<WakeEvent>);

impl Wake for NativeWake {
    fn wake(self: Arc<Self>) {
        let _ = self.0.send_event(WakeEvent);
    }

    fn wake_by_ref(self: &Arc<Self>) {
        let _ = self.0.send_event(WakeEvent);
    }
}

impl<E: From<io::Error>> BlockingDriver<E> for NativeDriver<E> {
    fn run(
        &mut self,
        shutdown: &ShutdownHandle,
        progress: &mut dyn FnMut(&mut Context<'_>) -> Poll<()>,
    ) -> Result<(), E> {
        if self.attempted {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "native event loop already attempted/exited; no replacement loop was created",
            )
            .into());
        }
        self.attempted = true;
        {
            let mut state = self.handle.0.borrow_mut();
            state.shutdown = Some(shutdown.clone());
            if state.stop_requested {
                shutdown.request_shutdown();
            }
        }
        // Core preflight has already completed for every Host. No application
        // acquisition runs until the first callback below.
        let Some(event_loop) = begin_loop(shutdown, progress, || {
            EventLoop::<WakeEvent>::with_user_event()
                .build()
                .map_err(io::Error::other)
        })
        .map_err(E::from)?
        else {
            return Ok(());
        };
        let waker = Waker::from(Arc::new(NativeWake(event_loop.create_proxy())));
        let mut app = NativeLoop {
            handle: self.handle.clone(),
            shutdown,
            progress,
            waker,
            poll_interval: self.poll_interval,
            resumed: false,
            finished: false,
        };
        // The original EventLoopError is preserved even if its last callback
        // already completed Core resource finalization.
        event_loop
            .run_app(&mut app)
            .map_err(io::Error::other)
            .map_err(E::from)
    }
}

struct NativeLoop<'a> {
    handle: NativeHandle,
    shutdown: &'a ShutdownHandle,
    progress: &'a mut dyn FnMut(&mut Context<'_>) -> Poll<()>,
    waker: Waker,
    poll_interval: Duration,
    resumed: bool,
    finished: bool,
}

impl NativeLoop<'_> {
    fn progress_once(&mut self, active: &ActiveEventLoop) {
        if self.finished {
            return;
        }
        if self.handle.0.borrow().stop_requested {
            self.shutdown.request_shutdown();
        }
        let mut cx = Context::from_waker(&self.waker);
        if (self.progress)(&mut cx).is_ready() {
            self.finished = true;
            active.exit();
        }
    }

    fn step(&mut self, active: &ActiveEventLoop) {
        self.progress_once(active);
        if self.finished || !self.resumed || self.shutdown.is_requested() {
            return;
        }
        let changed = {
            let mut state = self.handle.0.borrow_mut();
            if !state.acquire_requested {
                false
            } else {
                state.acquire_requested = false;
                let mut creation_error = None;
                for window in &mut state.windows {
                    if let Err(error) = window.resumed(active) {
                        creation_error = Some(error);
                        break;
                    }
                    // Each successful WindowSlot retains its original Window
                    // before the next fallible creation.
                }
                state.windows_created = creation_error.is_none();
                state.creation_error = creation_error;
                true
            }
        };
        if changed {
            self.progress_once(active);
        }
    }

    fn stop(&mut self, active: &ActiveEventLoop) {
        self.handle.request_shutdown();
        self.shutdown.request_shutdown();
        self.progress_once(active);
    }
}

impl ApplicationHandler<WakeEvent> for NativeLoop<'_> {
    fn resumed(&mut self, active: &ActiveEventLoop) {
        self.resumed = true;
        self.step(active);
    }

    fn user_event(&mut self, active: &ActiveEventLoop, _: WakeEvent) {
        self.step(active);
    }

    fn window_event(&mut self, active: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let should_stop = {
            let mut state = self.handle.0.borrow_mut();
            let Some(index) = state
                .windows
                .iter()
                .position(|window| window.window_id() == Some(id))
            else {
                return;
            };
            state.windows[index].observe_window_event(id, &event);
            let should_stop = matches!(event, WindowEvent::CloseRequested | WindowEvent::Destroyed);
            if state.admitting && !state.stop_requested && !should_stop {
                state.events.push_back(NativeEvent::Window { index, event });
            }
            should_stop
        };
        if should_stop {
            self.stop(active);
        } else {
            self.step(active);
        }
    }

    fn about_to_wait(&mut self, active: &ActiveEventLoop) {
        if self.finished {
            return;
        }
        {
            let mut state = self.handle.0.borrow_mut();
            if state.admitting && !state.stop_requested {
                state.events.push_back(NativeEvent::Tick);
            }
        }
        self.step(active);
        if !self.finished {
            active.set_control_flow(
                Instant::now()
                    .checked_add(self.poll_interval)
                    .map(ControlFlow::WaitUntil)
                    .unwrap_or(ControlFlow::Wait),
            );
        }
    }

    fn suspended(&mut self, active: &ActiveEventLoop) {
        self.resumed = false;
        self.stop(active);
    }

    fn exiting(&mut self, active: &ActiveEventLoop) {
        // A final backend callback is still a legitimate caller-thread cleanup
        // opportunity. If progress remains Pending, Core retains that fact.
        if !self.finished {
            self.stop(active);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};

    #[test]
    fn builder_panic_keeps_original_payload_until_error_drop() {
        struct Payload(Arc<AtomicUsize>);
        impl Drop for Payload {
            fn drop(&mut self) {
                self.0.fetch_add(1, Ordering::SeqCst);
            }
        }
        let drops = Arc::new(AtomicUsize::new(0));
        let original = drops.clone();
        let error = capture_builder::<()>(|| std::panic::panic_any(Payload(original))).unwrap_err();
        let source = error
            .get_ref()
            .unwrap()
            .downcast_ref::<EventLoopPanic>()
            .unwrap();
        assert_eq!(
            source.try_with_payload(|payload| payload.is::<Payload>()),
            Some(true)
        );
        assert!(
            catch_unwind(AssertUnwindSafe(|| {
                let _ = source.try_with_payload(|_| panic!("inspection panic"));
            }))
            .is_err()
        );
        assert_eq!(
            source.try_with_payload(|payload| payload.is::<Payload>()),
            Some(true)
        );
        assert!(format!("{error}").contains("non-string panic payload"));
        assert_eq!(drops.load(Ordering::SeqCst), 0);
        drop(error);
        assert_eq!(drops.load(Ordering::SeqCst), 1);
    }

    #[test]
    fn payload_inspection_can_format_the_error_and_nested_inspection_never_waits() {
        let cases: [(Box<dyn Any + Send>, &str); 2] = [
            (
                Box::new(String::from("owned builder panic")),
                "owned builder panic",
            ),
            (Box::new("static builder panic"), "static builder panic"),
        ];
        for (payload, expected) in cases {
            let error = capture_builder::<()>(|| std::panic::resume_unwind(payload)).unwrap_err();
            let source = error
                .get_ref()
                .unwrap()
                .downcast_ref::<EventLoopPanic>()
                .unwrap();
            assert_eq!(
                source.try_with_payload(|_| {
                    assert!(format!("{error}").contains(expected));
                    assert!(format!("{error:?}").contains(expected));
                    assert!(format!("{source:?}").contains(expected));
                    assert_eq!(
                        source.try_with_payload(|_| panic!("busy inspection must not execute")),
                        None::<()>
                    );
                    "inspected"
                }),
                Some("inspected")
            );
        }
    }

    #[derive(Debug)]
    struct OriginalFailure(Arc<()>);

    impl fmt::Display for OriginalFailure {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("original native failure")
        }
    }

    impl Error for OriginalFailure {}

    #[test]
    fn cancellation_does_not_discard_an_unreported_window_creation_error() {
        let host = NativeHost::<io::Error>::new(Window::default_attributes());
        let handle = host.handle();
        let identity = Arc::new(());
        handle.0.borrow_mut().creation_error =
            Some(io::Error::other(OriginalFailure(identity.clone())));
        let mut windows = NativeWindows::<io::Error> {
            handle: handle.clone(),
            _error: PhantomData,
        };
        let mut cx = Context::from_waker(Waker::noop());
        let ReleaseCompletion::Released(Err(error)) =
            windows.poll_release(&mut cx, Duration::ZERO).unwrap()
        else {
            panic!("cancelled acquisition must retain its original failure");
        };
        let source = error
            .get_ref()
            .unwrap()
            .downcast_ref::<OriginalFailure>()
            .unwrap();
        assert!(Arc::ptr_eq(&source.0, &identity));
        assert_eq!(handle.window_phase(0), Some(WindowPhase::Closed));
        assert!(handle.shared_window(0).is_none());
    }

    #[test]
    fn builder_failure_retires_the_still_inert_core_plan_and_retains_original_cause() {
        use crate::composition::{HostGroup, HostOperation, HostRunError, Phase};
        use std::cell::Cell;

        struct InertActivity(Rc<Cell<usize>>);
        impl HostActivity for InertActivity {
            type Error = io::Error;
            fn poll_start(&mut self, _: &mut Context<'_>) -> Poll<io::Result<()>> {
                self.0.set(self.0.get() + 1);
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
        struct FailingBuilder(Arc<()>);
        impl BlockingDriver<io::Error> for FailingBuilder {
            fn run(
                &mut self,
                shutdown: &ShutdownHandle,
                progress: &mut dyn FnMut(&mut Context<'_>) -> Poll<()>,
            ) -> io::Result<()> {
                begin_loop::<()>(shutdown, progress, || {
                    Err(io::Error::other(OriginalFailure(self.0.clone())))
                })?;
                unreachable!("injected builder always fails")
            }
        }
        struct InertHost(Rc<Cell<usize>>, Arc<()>);
        impl Host for InertHost {
            type Error = io::Error;
            type Activity = InertActivity;
            type Resources = Driven<FailingBuilder, ()>;
            fn name(&self) -> &'static str {
                "native-builder-proof"
            }
            fn into_resources(self) -> ResourceGroup<Self::Activity, Self::Resources> {
                ResourceGroup::new(InertActivity(self.0))
                    .with_blocking_driver(FailingBuilder(self.1))
            }
        }
        let starts = Rc::new(Cell::new(0));
        let identity = Arc::new(());
        let mut group = HostGroup::new()
            .with_host(InertHost(starts.clone(), identity.clone()))
            .build()
            .unwrap();
        match group.run() {
            Err(HostRunError::HostFailure(failure)) => {
                assert_eq!(failure.operation, HostOperation::Drive);
                let source = failure
                    .error()
                    .get_ref()
                    .unwrap()
                    .downcast_ref::<OriginalFailure>()
                    .unwrap();
                assert!(Arc::ptr_eq(&source.0, &identity));
            }
            _ => panic!("builder failure must be terminal with its original cause"),
        }
        assert_eq!(starts.get(), 0);
        assert_eq!(group.phase(), Phase::Failed);
    }

    #[test]
    fn inert_retirement_never_retries_pending_or_initializes_after_pre_cancel() {
        let shutdown = ShutdownHandle::default();
        let mut polls = 0;
        let result = begin_loop::<()>(
            &shutdown,
            &mut |_| {
                polls += 1;
                assert!(shutdown.is_requested());
                Poll::Pending
            },
            || Err(io::Error::new(io::ErrorKind::PermissionDenied, "builder")),
        );
        assert_eq!(result.unwrap_err().kind(), io::ErrorKind::PermissionDenied);
        assert_eq!(polls, 1);

        let cancelled = ShutdownHandle::default();
        cancelled.request_shutdown();
        let result = begin_loop::<()>(&cancelled, &mut |_| Poll::Ready(()), || {
            panic!("pre-cancel must never create an EventLoop")
        });
        assert!(result.unwrap().is_none());
    }

    #[test]
    fn observed_native_readiness_survives_usability_loss_while_a_sibling_starts() {
        use crate::composition::{HostGroup, Phase, StartStatus};
        use std::cell::Cell;

        // This fixture supplies a no-OS backend readiness observation. It uses
        // the real NativeActivity admission/callback methods and Core's actual
        // readiness latch, without claiming that an OS Window was observed.
        struct ObservedNative<F> {
            activity: NativeActivity<io::Error, F>,
            usable: Rc<Cell<bool>>,
            ready_polls: Rc<Cell<usize>>,
        }
        impl<F> HostActivity for ObservedNative<F>
        where
            F: FnMut(&NativeHandle, NativeEvent) -> io::Result<()>,
        {
            type Error = io::Error;
            fn poll_start(&mut self, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
                self.activity.poll_start(cx)
            }
            fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<io::Result<()>> {
                self.ready_polls.set(self.ready_polls.get() + 1);
                if self.usable.get() {
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
            }
            fn poll_finished(&mut self, cx: &mut Context<'_>) -> Poll<()> {
                self.activity.poll_finished(cx)
            }
            fn open_admission(&mut self) -> io::Result<AdmissionOutcome> {
                self.activity.open_admission()
            }
            fn close_admission(&mut self, cx: &mut Context<'_>) -> io::Result<()> {
                self.activity.close_admission(cx)
            }
            fn request_shutdown(&mut self, cx: &mut Context<'_>) -> io::Result<()> {
                self.activity.request_shutdown(cx)
            }
            fn poll_join(
                &mut self,
                cx: &mut Context<'_>,
                budget: Duration,
            ) -> io::Result<WorkerCompletion<io::Error>> {
                self.activity.poll_join(cx, budget)
            }
        }
        struct WaitingSibling(Rc<Cell<bool>>);
        impl HostActivity for WaitingSibling {
            type Error = io::Error;
            fn poll_start(&mut self, _: &mut Context<'_>) -> Poll<io::Result<()>> {
                Poll::Ready(Ok(()))
            }
            fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<io::Result<()>> {
                if self.0.get() {
                    Poll::Ready(Ok(()))
                } else {
                    Poll::Pending
                }
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
        struct TestHost<A>(&'static str, A);
        impl<A: HostActivity<Error = io::Error>> Host for TestHost<A> {
            type Error = io::Error;
            type Activity = A;
            type Resources = ();
            fn name(&self) -> &'static str {
                self.0
            }
            fn into_resources(self) -> ResourceGroup<A> {
                ResourceGroup::new(self.1)
            }
        }

        for transition in ["resize", "occlusion"] {
            let callbacks = Cell::new(0);
            let native = NativeHost::<io::Error>::new(Window::default_attributes()).on_event(
                |handle, event| {
                    assert!(handle.is_admitting());
                    assert!(matches!(event, NativeEvent::Tick));
                    callbacks.set(callbacks.get() + 1);
                    Ok(())
                },
            );
            let handle = native.handle();
            // No OS creation or redraw request belongs in this contract test.
            // Current usability remains false; the injected backend below
            // supplies the earlier observation that Core is expected to retain.
            handle.0.borrow_mut().windows.clear();
            let usable = Rc::new(Cell::new(true));
            let ready_polls = Rc::new(Cell::new(0));
            let sibling_ready = Rc::new(Cell::new(false));
            let activity = NativeActivity {
                handle: native.handle,
                callback: native.callback,
                started: false,
                finished: false,
                completion: None,
            };
            let mut group = HostGroup::new()
                .with_host(TestHost(
                    "native",
                    ObservedNative {
                        activity,
                        usable: usable.clone(),
                        ready_polls: ready_polls.clone(),
                    },
                ))
                .with_host(TestHost("sibling", WaitingSibling(sibling_ready.clone())))
                .build()
                .unwrap();
            let mut cx = Context::from_waker(Waker::noop());
            assert!(group.poll_start(&mut cx).is_pending());
            assert_eq!(ready_polls.get(), 1);
            assert!(!handle.is_admitting());

            // Resize and occlusion both temporarily invalidate current Window
            // usability after that startup observation, before another redraw.
            usable.set(false);
            assert!(!handle.is_ready());
            sibling_ready.set(true);
            assert_eq!(
                group.poll_start(&mut cx),
                Poll::Ready(StartStatus::Running),
                "{transition} must not become a permanent admission fence"
            );
            assert_eq!(ready_polls.get(), 1);
            assert!(handle.is_admitting());
            assert!(!group.shutdown_handle().is_requested());
            assert!(group.failures().primary().is_none());

            // Useful non-rendering progress continues while another usable
            // redraw is pending. Temporary unusability does not finish run.
            handle.0.borrow_mut().events.push_back(NativeEvent::Tick);
            assert!(group.poll_run(&mut cx).is_pending());
            assert_eq!(callbacks.get(), 1);
            assert_eq!(group.phase(), Phase::Running);
            group.request_shutdown();
            assert!(
                group
                    .poll_shutdown(&mut cx, Instant::now() + Duration::from_secs(1))
                    .is_ready()
            );
            assert_eq!(group.phase(), Phase::Stopped);
        }
    }

    #[test]
    fn native_stop_and_release_remain_permanent_admission_fences() {
        for released in [false, true] {
            let native = NativeHost::<io::Error>::new(Window::default_attributes());
            let handle = native.handle();
            {
                let mut state = handle.0.borrow_mut();
                state.stop_requested = !released;
                state.released = released;
            }
            let mut activity = NativeActivity {
                handle: native.handle,
                callback: native.callback,
                started: true,
                finished: false,
                completion: None,
            };
            assert_eq!(activity.open_admission().unwrap(), AdmissionOutcome::Closed);
            assert!(!handle.is_admitting());
        }
    }

    #[test]
    fn callbacks_keep_local_original_error_until_join_and_do_not_run_before_admission() {
        struct LocalError<'a>(&'a str, Rc<()>);
        impl From<io::Error> for LocalError<'_> {
            fn from(_: io::Error) -> Self {
                panic!("this callback-only test does not initialize Winit");
            }
        }
        let message = String::from("local callback failure");
        let identity = Rc::new(());
        let expected = identity.clone();
        let host = NativeHost::new(Window::default_attributes())
            .on_event(|_, _| Err(LocalError(message.as_str(), identity.clone())));
        let handle = host.handle();
        let mut activity = NativeActivity {
            handle: host.handle,
            callback: host.callback,
            started: true,
            finished: false,
            completion: None,
        };
        let mut cx = Context::from_waker(Waker::noop());
        handle.0.borrow_mut().events.push_back(NativeEvent::Tick);
        assert!(activity.poll_finished(&mut cx).is_pending());
        // This is a synthetic callback-unit boundary, not Window readiness or
        // native execution evidence. No Window or EventLoop is created.
        handle.0.borrow_mut().admitting = true;
        assert!(activity.poll_finished(&mut cx).is_ready());
        let Ok(WorkerCompletion::Joined(Err(error))) = activity.poll_join(&mut cx, Duration::ZERO)
        else {
            panic!("expected original callback error");
        };
        assert_eq!(error.0, message);
        assert!(Rc::ptr_eq(&error.1, &expected));
        assert!(!handle.is_ready());
        assert!(!handle.is_admitting());
    }
}
