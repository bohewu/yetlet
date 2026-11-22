//! One finite, real native event loop with UDP and optional API-only Web.

use std::{
    cell::{Cell, RefCell},
    io::{self, Read, Write},
    net::{Ipv4Addr, SocketAddr, TcpStream, UdpSocket},
    rc::Rc,
    sync::mpsc::{self, Receiver, SyncSender, TrySendError},
    task::{Context, Poll},
    thread::{self, ThreadId},
    time::{Duration, Instant},
};

use yetlet::{
    Host, HostGroup, ResourceGroup,
    composition::{
        AdmissionOutcome, HostActivity, JoinStatus, Managed, ManagedResource, Phase,
        ReleaseCompletion, ReleaseStatus, ShutdownHandle, WorkerCompletion,
    },
    native_winit::{
        NativeEvent, NativeHandle, NativeHost, WindowPhase,
        winit::{
            dpi::{LogicalPosition, LogicalSize},
            event::WindowEvent,
            window::Window,
        },
    },
    udp::{SendOutcome, UdpHandle, UdpHost},
};

#[cfg(feature = "web")]
use yetlet::web::{App, WebError, WebHandle};

#[cfg(feature = "web")]
type AppError = WebError;
#[cfg(not(feature = "web"))]
type AppError = io::Error;

fn app_error(error: io::Error) -> AppError {
    #[cfg(feature = "web")]
    {
        error.into()
    }
    #[cfg(not(feature = "web"))]
    {
        error
    }
}

const CLIENT_LIMIT: Duration = Duration::from_secs(8);
const PAYLOAD: &[u8] = b"native-hosts-echo";
const HTTP_BODY: &str = "native-hosts-ready";

#[derive(Default)]
struct Observations {
    redraws: [usize; 2],
    udp_callbacks: usize,
}

#[derive(Clone, Copy)]
struct Endpoints {
    udp: SocketAddr,
    http: Option<SocketAddr>,
}

enum ClientMessage {
    Ready(Endpoints),
    Abort,
}

struct StopOnDrop(ShutdownHandle);
impl Drop for StopOnDrop {
    fn drop(&mut self) {
        self.0.request_shutdown();
    }
}

fn ensure(condition: bool, message: &str) -> io::Result<()> {
    if condition {
        Ok(())
    } else {
        Err(io::Error::other(message))
    }
}

fn on_caller(caller: ThreadId) -> io::Result<()> {
    ensure(
        thread::current().id() == caller,
        "callback left the calling thread",
    )
}

fn remaining(deadline: Instant) -> io::Result<Duration> {
    let remaining = deadline.saturating_duration_since(Instant::now());
    if remaining.is_zero() {
        Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "client absolute deadline",
        ))
    } else {
        Ok(remaining)
    }
}

fn http_probe(address: SocketAddr, deadline: Instant) -> io::Result<()> {
    let mut stream = TcpStream::connect_timeout(&address, remaining(deadline)?)?;
    let mut request =
        b"GET /health HTTP/1.1\r\nHost: localhost\r\nConnection: close\r\n\r\n".as_slice();
    while !request.is_empty() {
        stream.set_write_timeout(Some(remaining(deadline)?))?;
        let written = match stream.write(request) {
            Ok(written) => written,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        ensure(written != 0, "HTTP request write made no progress")?;
        request = &request[written..];
    }
    let mut response = Vec::new();
    let mut buffer = [0; 512];
    loop {
        stream.set_read_timeout(Some(remaining(deadline)?))?;
        let length = match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(length) => length,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error),
        };
        ensure(
            response.len() + length <= 4096,
            "HTTP response exceeded client bound",
        )?;
        response.extend_from_slice(&buffer[..length]);
    }
    let response = String::from_utf8(response).map_err(io::Error::other)?;
    let (headers, body) = response
        .split_once("\r\n\r\n")
        .ok_or_else(|| io::Error::other("missing HTTP response boundary"))?;
    ensure(
        headers.starts_with("HTTP/1.1 200 "),
        "HTTP status was not 200",
    )?;
    ensure(body == HTTP_BODY, "HTTP response body did not match")?;
    println!("client http_endpoint={address} status=200");
    Ok(())
}

fn drive_client(
    messages: Receiver<ClientMessage>,
    shutdown: ShutdownHandle,
) -> io::Result<Endpoints> {
    let _stop = StopOnDrop(shutdown);
    let deadline = Instant::now() + CLIENT_LIMIT;
    let endpoints = match messages.recv_timeout(remaining(deadline)?) {
        Ok(ClientMessage::Ready(endpoints)) => endpoints,
        Ok(ClientMessage::Abort) => return Err(io::Error::other("Host startup aborted")),
        Err(error) => {
            return Err(io::Error::new(io::ErrorKind::TimedOut, error));
        }
    };
    let socket = UdpSocket::bind((Ipv4Addr::LOCALHOST, 0))?;
    socket.set_write_timeout(Some(remaining(deadline)?))?;
    ensure(
        socket.send_to(PAYLOAD, endpoints.udp)? == PAYLOAD.len(),
        "partial client UDP send",
    )?;
    socket.set_read_timeout(Some(remaining(deadline)?))?;
    let mut bytes = [0; 64];
    let (length, peer) = socket.recv_from(&mut bytes)?;
    ensure(
        peer == endpoints.udp && &bytes[..length] == PAYLOAD,
        "UDP echo did not match",
    )?;
    println!("client udp_endpoint={peer} bytes={length}");
    if let Some(http) = endpoints.http {
        http_probe(http, deadline)?;
    }
    remaining(deadline)?;
    Ok(endpoints)
}

struct LocalActivity<'a> {
    caller: ThreadId,
    borrowed: &'a str,
    observed: Rc<RefCell<Observations>>,
    native: NativeHandle,
    udp: UdpHandle,
    #[cfg(feature = "web")]
    web: WebHandle,
    publish: SyncSender<ClientMessage>,
    published: bool,
    admitting: bool,
    stopping: bool,
    terminal: Option<AppError>,
}

impl LocalActivity<'_> {
    fn publish_when_usable(&mut self) -> Result<(), AppError> {
        on_caller(self.caller)?;
        ensure(
            self.borrowed == "caller-owned application data",
            "borrowed data changed",
        )?;
        if self.published
            || !self.native.is_admitting()
            || !self.udp.is_admitting()
            || self.observed.borrow().redraws.contains(&0)
        {
            return Ok(());
        }
        #[cfg(feature = "web")]
        if !self.web.is_admitting() {
            return Ok(());
        }
        let udp = self
            .udp
            .local_addr()
            .ok_or_else(|| io::Error::other("admitted UDP endpoint missing"))?;
        #[cfg(feature = "web")]
        let http = Some(
            self.web
                .local_addr()
                .ok_or_else(|| io::Error::other("admitted Web endpoint missing"))?,
        );
        #[cfg(not(feature = "web"))]
        let http = None;
        match self
            .publish
            .try_send(ClientMessage::Ready(Endpoints { udp, http }))
        {
            Ok(()) => {
                self.published = true;
                println!(
                    "admitted caller={:?} windows=2 udp={udp} http={http:?}",
                    self.caller
                );
                Ok(())
            }
            Err(TrySendError::Full(_)) => Ok(()),
            Err(TrySendError::Disconnected(_)) => Err(app_error(io::Error::other(
                "client stopped before readiness publication",
            ))),
        }
    }
}

impl HostActivity for LocalActivity<'_> {
    type Error = AppError;

    fn poll_start(&mut self, _: &mut Context<'_>) -> Poll<Result<(), AppError>> {
        Poll::Ready(on_caller(self.caller).map_err(app_error))
    }

    fn poll_ready(&mut self, _: &mut Context<'_>) -> Poll<Result<(), AppError>> {
        Poll::Ready(Ok(()))
    }

    fn open_admission(&mut self) -> Result<AdmissionOutcome, AppError> {
        self.admitting = true;
        Ok(AdmissionOutcome::Opened)
    }

    fn poll_finished(&mut self, _: &mut Context<'_>) -> Poll<()> {
        if self.terminal.is_some() {
            return Poll::Ready(());
        }
        if self.admitting
            && let Err(error) = self.publish_when_usable()
        {
            self.terminal = Some(error);
            return Poll::Ready(());
        }
        Poll::Pending
    }

    fn close_admission(&mut self, _: &mut Context<'_>) -> Result<(), AppError> {
        on_caller(self.caller)?;
        self.admitting = false;
        Ok(())
    }

    fn request_shutdown(&mut self, _: &mut Context<'_>) -> Result<(), AppError> {
        on_caller(self.caller)?;
        self.stopping = true;
        Ok(())
    }

    fn poll_join(
        &mut self,
        _: &mut Context<'_>,
        _: Duration,
    ) -> Result<WorkerCompletion<AppError>, AppError> {
        on_caller(self.caller)?;
        ensure(self.stopping, "custom Host joined before shutdown")?;
        Ok(WorkerCompletion::Joined(
            self.terminal.take().map_or(Ok(()), Err),
        ))
    }
}

struct LocalResource<'a> {
    caller: ThreadId,
    borrowed: &'a str,
    native: NativeHandle,
    release_passes: &'a Cell<usize>,
    acquired: &'a Cell<bool>,
    local: Rc<Cell<usize>>,
}

impl LocalResource<'_> {
    fn validate_release(&self) -> io::Result<()> {
        on_caller(self.caller)?;
        ensure(
            self.borrowed == "caller-owned application data",
            "borrowed resource changed",
        )?;
        ensure(
            (0..2).all(|index| self.native.shared_window(index).is_some()),
            "native windows closed before the custom resource",
        )?;
        ensure(
            !self.native.is_admitting(),
            "native admission remained open during release",
        )?;
        Ok(())
    }
}

impl ManagedResource for LocalResource<'_> {
    type Error = AppError;

    fn poll_acquire(&mut self, _: &mut Context<'_>) -> Poll<Result<(), AppError>> {
        self.acquired.set(true);
        Poll::Ready(on_caller(self.caller).map_err(app_error))
    }

    fn poll_release(
        &mut self,
        cx: &mut Context<'_>,
        _: Duration,
    ) -> Result<ReleaseCompletion<AppError>, AppError> {
        if let Err(error) = self.validate_release() {
            // A failed assertion is terminal, not a retryable cleanup operation.
            return Ok(ReleaseCompletion::Released(Err(app_error(error))));
        }
        let pass = self.release_passes.get() + 1;
        self.release_passes.set(pass);
        self.local.set(pass);
        println!(
            "resource release_pass={pass} native_windows_live=2 caller={:?}",
            self.caller
        );
        if pass == 1 {
            // One finite continuation: the same native loop must service this wake.
            cx.waker().wake_by_ref();
            Ok(ReleaseCompletion::Pending)
        } else {
            Ok(ReleaseCompletion::Released(Ok(())))
        }
    }
}

struct LocalHost<'a> {
    activity: LocalActivity<'a>,
    resource: LocalResource<'a>,
}

impl<'a> Host for LocalHost<'a> {
    type Error = AppError;
    type Activity = LocalActivity<'a>;
    type Resources = Managed<LocalResource<'a>, ()>;

    fn name(&self) -> &'static str {
        "local"
    }

    fn into_resources(self) -> ResourceGroup<Self::Activity, Self::Resources> {
        ResourceGroup::new(self.activity)
            .with_managed_resource("borrowed-native-resource", self.resource)
    }
}

fn main() -> io::Result<()> {
    let caller = thread::current().id();
    let borrowed = String::from("caller-owned application data");
    let observations = Rc::new(RefCell::new(Observations::default()));
    let native_observations = observations.clone();
    let native = NativeHost::new(
        Window::default_attributes()
            .with_title("Yetlet Native Hosts - First")
            .with_inner_size(LogicalSize::new(360.0, 220.0))
            .with_position(LogicalPosition::new(80.0, 80.0)),
    )
    .with_window(
        Window::default_attributes()
            .with_title("Yetlet Native Hosts - Second")
            .with_inner_size(LogicalSize::new(360.0, 220.0))
            .with_position(LogicalPosition::new(480.0, 80.0)),
    )
    .on_event(|handle, event| -> Result<(), AppError> {
        on_caller(caller)?;
        ensure(
            borrowed == "caller-owned application data",
            "native borrow changed",
        )?;
        if let NativeEvent::Window {
            index,
            event: WindowEvent::RedrawRequested,
        } = event
        {
            let id = handle
                .shared_window(index)
                .map(|window| window.id())
                .ok_or_else(|| io::Error::other("redrawn native window missing"))?;
            let mut observed = native_observations.borrow_mut();
            observed.redraws[index] += 1;
            if observed.redraws[index] == 1 {
                println!(
                    "native window_index={index} id={id:?} real_redraw=true caller={caller:?}"
                );
            }
        }
        Ok(())
    });
    let native_handle = native.handle();
    let udp_observations = observations.clone();
    let udp = UdpHost::new()
        .with_max_datagram_bytes(64)
        .with_packet_budget(4)
        .on_datagram(|packet| -> Result<(), AppError> {
            on_caller(caller)?;
            ensure(packet.bytes() == PAYLOAD, "unexpected UDP packet")?;
            ensure(
                packet.reply(packet.bytes())? == SendOutcome::Sent(PAYLOAD.len()),
                "UDP reply was not submitted",
            )?;
            udp_observations.borrow_mut().udp_callbacks += 1;
            Ok(())
        });
    let udp_handle = udp.handle();
    #[cfg(feature = "web")]
    let web = App::new()
        .get("/health", || async { HTTP_BODY })
        .into_host()
        .with_address("127.0.0.1:0");
    #[cfg(feature = "web")]
    let web_handle = web.handle();

    // Sibling Web infers the common WebError; the smaller variant selects IO.
    #[cfg(feature = "web")]
    let plan = HostGroup::new().with_host(web);
    #[cfg(not(feature = "web"))]
    let plan = HostGroup::<io::Error>::new();
    let shutdown = plan.shutdown_handle();
    let (publish, messages) = mpsc::sync_channel(1);
    let release_passes = Cell::new(0);
    let acquired = Cell::new(false);
    let local_counter = Rc::new(Cell::new(0));
    let local = LocalHost {
        activity: LocalActivity {
            caller,
            borrowed: &borrowed,
            observed: observations.clone(),
            native: native_handle.clone(),
            udp: udp_handle.clone(),
            #[cfg(feature = "web")]
            web: web_handle.clone(),
            publish: publish.clone(),
            published: false,
            admitting: false,
            stopping: false,
            terminal: None,
        },
        resource: LocalResource {
            caller,
            borrowed: &borrowed,
            native: native_handle.clone(),
            release_passes: &release_passes,
            acquired: &acquired,
            local: local_counter.clone(),
        },
    };
    let mut hosts = plan
        .with_host(native)
        .with_host(udp)
        .with_host(local)
        .build()
        .map_err(io::Error::other)?;
    ensure(
        native_handle.window_count() == 2,
        "inert native window plan changed",
    )?;
    ensure(
        !native_handle.is_ready() && udp_handle.local_addr().is_none(),
        "build acquired resources",
    )?;

    let endpoints = thread::scope(|scope| -> io::Result<Endpoints> {
        let client = scope.spawn(move || drive_client(messages, shutdown));
        let run_error = hosts.run().err().map(|error| error.to_string());
        if run_error.is_some() {
            let _ = publish.try_send(ClientMessage::Abort);
        }
        let client_result = client
            .join()
            .map_err(|_| io::Error::other("client thread panicked"))?;
        if let Some(error) = run_error {
            return Err(io::Error::other(error));
        }
        client_result
    })?;

    ensure(
        hosts.phase() == Phase::Stopped,
        "HostGroup did not stop successfully",
    )?;
    for outcome in hosts.outcomes() {
        println!(
            "host name={} join={:?} release={:?}",
            outcome.host, outcome.join, outcome.release
        );
        ensure(
            outcome.join == JoinStatus::Succeeded && outcome.release == ReleaseStatus::Succeeded,
            "a Host retained an unfinished or failed outcome",
        )?;
    }
    ensure(
        (0..2).all(|index| {
            native_handle.window_phase(index) == Some(WindowPhase::Closed)
                && native_handle.shared_window(index).is_none()
        }),
        "native windows retained owners after final release",
    )?;
    ensure(
        udp_handle.local_addr().is_none() && !udp_handle.is_admitting(),
        "UDP remained live",
    )?;
    #[cfg(feature = "web")]
    ensure(
        !web_handle.is_ready() && !web_handle.is_admitting(),
        "Web remained live",
    )?;
    #[cfg(feature = "web")]
    ensure(
        web_handle.local_addr() == endpoints.http,
        "Web bound-address metadata changed",
    )?;
    ensure(
        acquired.get() && release_passes.get() == 2 && local_counter.get() == 2,
        "resource cleanup incomplete",
    )?;
    ensure(
        observations.borrow().udp_callbacks == 1,
        "UDP callback count changed",
    )?;
    let rebound = UdpSocket::bind(endpoints.udp)?;
    drop(rebound);
    println!(
        "complete windows_closed=2 udp_echo=true http={} pending_release=true caller={caller:?} gpu_present=false",
        endpoints.http.is_some()
    );
    Ok(())
}
