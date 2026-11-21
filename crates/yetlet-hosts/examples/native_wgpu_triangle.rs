//! YT-282-D2 caller-owned Winit + independently selected Surface renderer.
//! One local C1 domain is observed, never advanced by drawing. Auto-exits.

use std::{
    cell::Cell,
    error::Error,
    future::Future,
    io,
    rc::Rc,
    sync::Arc,
    task::{Context, Poll, Wake, Waker},
    thread,
    time::{Duration, Instant},
};
use winit::{
    application::ApplicationHandler,
    dpi::{LogicalSize, PhysicalSize},
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
use yetlet::composition::{Attached, Builder, Completion, Composition, Empty, Local, Phase};
use yetlet_hosts::{
    native_winit::{WindowPhase, WindowSignal, WindowSlot},
    renderer_wgpu::{
        DrawReceipt, FatalError, RenderCounters, RenderError, SurfacePhase, SurfaceRenderer, wgpu,
    },
};

#[derive(Default)]
struct C1Domain {
    revision: Cell<u64>,
    stopped: Cell<bool>,
}
fn stop_domain(domain: &mut Rc<C1Domain>) -> io::Result<()> {
    domain.stopped.set(true);
    Ok(())
}

fn initial_phase(size: PhysicalSize<u32>) -> SurfacePhase {
    if size.width == 0 || size.height == 0 {
        SurfacePhase::DeferredZeroSize
    } else {
        SurfacePhase::Unconfigured
    }
}

fn renderer_error(renderer: &SurfaceRenderer<'_>, error: RenderError) -> io::Error {
    match renderer.first_device_error() {
        Some(cause) => io::Error::other(Arc::clone(cause)),
        None => match renderer.first_fatal_error() {
            Some(cause) => io::Error::other(Arc::clone(cause)),
            None => match renderer.first_device_loss() {
                Some(cause) => io::Error::other(Arc::clone(cause)),
                None => io::Error::other(error),
            },
        },
    }
}

struct Foreground {
    // Renderer drops before WindowSlot even on an early error.
    renderer: Option<SurfaceRenderer<'static>>,
    cleanup_error: Option<Arc<FatalError>>,
    window: WindowSlot,
    counters: RenderCounters,
    window_ready: bool,
    diagnostic: bool,
    zero_deferred: bool,
    close_blocked: bool,
    resize_requested: bool,
    actual_resize: bool,
}
impl Foreground {
    fn prepare(&mut self, active: &ActiveEventLoop) -> io::Result<()> {
        if self.renderer.is_some() {
            return Ok(());
        }
        let id = self.window.resumed(active)?;
        let size = self.window.last_size().expect("created Window size");
        eprintln!(
            "YT282_D2_WINDOW_CREATED pid={} window_id={id:?} size={size:?} window_ready={} gpu_present=0",
            std::process::id(),
            self.window.is_ready()
        );
        let started = Instant::now();
        // Diagnostic exercises initial zero at the real renderer API, while
        // retaining the actual OS size for restoration. Never claim an OS zero.
        let initial_size = if self.diagnostic {
            PhysicalSize::new(0, 0)
        } else {
            size
        };
        let mut renderer = initialize(SurfaceRenderer::new(
            self.window.window_handle().expect("owned safe target"),
            initial_size.width,
            initial_size.height,
        ))??;
        eprintln!(
            "YT282_D2_GPU_PREPARED adapter={:?} backend={:?} driver={:?} phase={:?} counters={:?} init_ms={}",
            renderer.adapter_info().name,
            renderer.adapter_info().backend,
            renderer.adapter_info().driver_info,
            renderer.phase(),
            renderer.counters(),
            started.elapsed().as_millis()
        );
        // This proof specifically requires an actual hardware Vulkan adapter.
        if renderer.adapter_info().backend != wgpu::Backend::Vulkan
            || renderer.adapter_info().device_type != wgpu::DeviceType::DiscreteGpu
        {
            return Err(io::Error::other(
                "hardware Vulkan adapter required for this example proof",
            ));
        }
        assert_eq!(
            renderer
                .draw()
                .map_err(|error| renderer_error(&renderer, error))?,
            DrawReceipt::Deferred(initial_phase(initial_size))
        );
        assert_eq!(renderer.counters(), RenderCounters::default());
        eprintln!(
            "YT282_D2_INITIAL_DEFERRED supplied_size={initial_size:?} synthetic={} actual_os_size={size:?} phase={:?} counters={:?}",
            self.diagnostic,
            renderer.phase(),
            renderer.counters()
        );
        self.close_blocked = self.window.close().unwrap_err().kind() == io::ErrorKind::WouldBlock;
        eprintln!(
            "YT282_D2_TARGET_RETAINED close_while_renderer_alive_rejected={}",
            self.close_blocked
        );
        if self.diagnostic {
            let before = renderer.counters();
            assert!(
                !renderer
                    .resize(0, 0)
                    .map_err(|error| renderer_error(&renderer, error))?
            );
            assert_eq!(
                renderer
                    .draw()
                    .map_err(|error| renderer_error(&renderer, error))?,
                DrawReceipt::Deferred(SurfacePhase::DeferredZeroSize)
            );
            let after = renderer.counters();
            self.zero_deferred = after.configure_calls == before.configure_calls
                && after.acquire_calls == before.acquire_calls
                && after.present_calls == before.present_calls;
            eprintln!(
                "YT282_D2_ZERO_SIZE stimulus=synthetic_dimensions actual_os_size={size:?} no_configure_acquire_present={}",
                self.zero_deferred
            );
        }
        if size.width > 0 && size.height > 0 {
            renderer
                .resize(size.width, size.height)
                .map_err(|error| renderer_error(&renderer, error))?;
        }
        eprintln!(
            "YT282_D2_GPU_SIZE_APPLIED phase={:?} counters={:?} window_ready={}",
            renderer.phase(),
            renderer.counters(),
            self.window.is_ready()
        );
        self.renderer = Some(renderer);
        Ok(())
    }

    fn event(&mut self, id: WindowId, event: &WindowEvent) -> io::Result<bool> {
        let signal = self.window.observe_window_event(id, event);
        match signal {
            WindowSignal::CloseRequested | WindowSignal::Destroyed => {
                return Err(io::Error::other(
                    "Window closed before bounded proof completed",
                ));
            }
            WindowSignal::Resized(size) => {
                eprintln!(
                    "YT282_D2_RESIZED stimulus=actual_os_event window_id={id:?} size={size:?}"
                );
                let renderer = self.renderer.as_mut().expect("prepared renderer");
                renderer
                    .resize(size.width, size.height)
                    .map_err(|error| renderer_error(renderer, error))?;
                if self.resize_requested && size == PhysicalSize::new(720, 480) {
                    self.actual_resize = true;
                }
            }
            WindowSignal::RedrawReady => {
                let renderer = self.renderer.as_mut().expect("prepared renderer");
                if !self.window_ready {
                    self.window_ready = true;
                    eprintln!(
                        "YT282_D2_WINDOW_READY stimulus=actual_os_redraw window_id={id:?} accepted_redraws={} counters_before_draw={:?}",
                        self.window.redraw_events(),
                        renderer.counters()
                    );
                }
                // Recovery is explicitly caller-driven with current physical size.
                if renderer.phase() == SurfacePhase::Unconfigured {
                    let size = self.window.window().expect("live Window").inner_size();
                    renderer
                        .resize(size.width, size.height)
                        .map_err(|error| renderer_error(renderer, error))?;
                }
                match renderer
                    .draw()
                    .map_err(|error| renderer_error(renderer, error))?
                {
                    DrawReceipt::Presented {
                        present_calls,
                        suboptimal,
                    } => {
                        if present_calls == 1 {
                            eprintln!(
                                "YT282_D2_FIRST_PRESENT window_id={id:?} counters={:?} suboptimal={suboptimal} c1_revision=0 scanout_verified=false",
                                renderer.counters()
                            );
                            if self.diagnostic {
                                self.resize_requested = true;
                                let _ = self
                                    .window
                                    .window()
                                    .unwrap()
                                    .request_inner_size(PhysicalSize::new(720, 480));
                            }
                        }
                        return Ok(true);
                    }
                    receipt => eprintln!(
                        "YT282_D2_DEFERRED stimulus=actual_draw_result receipt={receipt:?} phase={:?}",
                        renderer.phase()
                    ),
                }
            }
            _ => {}
        }
        Ok(false)
    }
}

fn close_foreground(part: &mut Foreground) -> io::Result<()> {
    if let Some(mut renderer) = part.renderer.take() {
        let closed = renderer.close(); // release actual Surface/owned Window Arc first
        part.cleanup_error = renderer.frame_cleanup_error().cloned();
        let before = renderer.counters();
        assert_eq!(renderer.draw(), Err(RenderError::Closed));
        assert_eq!(renderer.resize(640, 420), Err(RenderError::Closed));
        assert_eq!(renderer.close(), closed); // idempotent, including cleanup failure
        assert_eq!(renderer.counters(), before);
        part.counters = before;
        eprintln!(
            "YT282_D2_GPU_CLOSED phase={:?} counters={before:?} late_calls_rejected=true frame_cleanup_failed={}",
            renderer.phase(),
            part.cleanup_error.is_some()
        );
    }
    part.window.close()?;
    eprintln!(
        "YT282_D2_WINDOW_CLOSED phase={:?} surface_released=true",
        part.window.phase()
    );
    // Release Window after Surface even when discard failed; retain the cleanup
    // error across retries so Composition cannot falsely report Complete.
    if let Some(error) = &part.cleanup_error {
        return Err(io::Error::other(Arc::clone(error)));
    }
    Ok(())
}

// Only initialization may wait for a pending future. No new runtime or thread.
// The deadline bounds cooperative polling; native driver FFI is not preemptible.
fn initialize<F: Future>(future: F) -> io::Result<F::Output> {
    struct WakeCaller(thread::Thread);
    impl Wake for WakeCaller {
        fn wake(self: Arc<Self>) {
            self.0.unpark();
        }
        fn wake_by_ref(self: &Arc<Self>) {
            self.0.unpark();
        }
    }
    let deadline = Instant::now() + Duration::from_secs(5);
    let waker = Waker::from(Arc::new(WakeCaller(thread::current())));
    let mut context = Context::from_waker(&waker);
    let mut future = std::pin::pin!(future);
    loop {
        if Instant::now() >= deadline {
            return Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "GPU initialization deadline",
            ));
        }
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => {
                if Instant::now() >= deadline {
                    return Err(io::Error::new(
                        io::ErrorKind::TimedOut,
                        "native initialization exceeded deadline",
                    ));
                }
                return Ok(value);
            }
            Poll::Pending => thread::park_timeout(
                deadline
                    .saturating_duration_since(Instant::now())
                    .min(Duration::from_millis(10)),
            ),
        }
    }
}

type DomainClose = fn(&mut Rc<C1Domain>) -> io::Result<()>;
type ForegroundClose = fn(&mut Foreground) -> io::Result<()>;
type Parts =
    Attached<Attached<Empty, Local<Rc<C1Domain>, DomainClose>>, Local<Foreground, ForegroundClose>>;
type Root = Composition<io::Error, Parts>;
struct Probe {
    root: Root,
    started: Instant,
    next_redraw: Instant,
    presented: bool,
}
impl Probe {
    fn fail(&mut self, active: &ActiveEventLoop, error: io::Error) {
        eprintln!("YT282_D2_FAILURE cause={error}");
        self.root.fail_required(error);
        active.exit();
    }
}
impl ApplicationHandler for Probe {
    fn resumed(&mut self, active: &ActiveEventLoop) {
        let Some(part) = self.root.parts_mut().last().value_mut() else {
            return;
        };
        if let Err(error) = part.prepare(active) {
            self.fail(active, error);
        }
    }
    fn window_event(&mut self, active: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(part) = self.root.parts_mut().last().value_mut() else {
            return;
        };
        match part.event(id, &event) {
            Ok(true) => {
                self.presented = true;
                self.root.parts_mut().last().mark_ready();
                assert_eq!(
                    self.root.parts().previous().last().value().revision.get(),
                    0
                );
            }
            Ok(false) => {}
            Err(error) => self.fail(active, error),
        }
    }
    fn about_to_wait(&mut self, active: &ActiveEventLoop) {
        let now = Instant::now();
        if now.duration_since(self.started) >= Duration::from_secs(8) {
            self.fail(
                active,
                io::Error::new(io::ErrorKind::TimedOut, "GPU foreground proof deadline"),
            );
            return;
        }
        let Some(part) = self.root.parts().last().value().renderer.as_ref() else {
            return;
        };
        let foreground = self.root.parts().last().value();
        if self.presented
            && part.counters().present_calls >= 2
            && (!foreground.diagnostic || foreground.actual_resize)
            && now.duration_since(self.started) >= Duration::from_secs(2)
        {
            active.exit();
            return;
        }
        if now >= self.next_redraw {
            if foreground
                .window
                .window()
                .is_some_and(|window| window.is_visible() == Some(true))
            {
                let _ = foreground.window.request_redraw();
            }
            self.next_redraw = now + Duration::from_millis(100);
        }
        active.set_control_flow(ControlFlow::WaitUntil(self.next_redraw));
    }
    fn suspended(&mut self, active: &ActiveEventLoop) {
        self.fail(
            active,
            io::Error::other("foreground suspended during bounded GPU proof"),
        );
    }
    fn exiting(&mut self, _active: &ActiveEventLoop) {
        self.root.request_stop();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let diagnostic = matches!(args.as_slice(), [arg] if arg == "--diagnostic");
    if !args.is_empty() && !diagnostic {
        return Err("usage: native_wgpu_triangle [--diagnostic]".into());
    }
    let event_loop = EventLoop::new()?; // actual caller foreground thread
    let foreground = Foreground {
        renderer: None,
        cleanup_error: None,
        window: WindowSlot::new(
            Window::default_attributes()
                .with_title("Yetlet D2 neutral GPU triangle")
                .with_inner_size(LogicalSize::new(640.0, 420.0)),
        ),
        counters: RenderCounters::default(),
        window_ready: false,
        diagnostic,
        zero_deferred: false,
        close_blocked: false,
        resize_requested: false,
        actual_resize: false,
    };
    let root = Builder::<io::Error>::new()
        .required_local(
            "sole C1",
            Rc::new(C1Domain::default()),
            stop_domain as DomainClose,
        )
        .required_foreground(
            "Window + selected GPU",
            foreground,
            close_foreground as ForegroundClose,
        )
        .build();
    let mut app = Probe {
        root,
        started: Instant::now(),
        next_redraw: Instant::now(),
        presented: false,
    };
    if let Err(error) = event_loop.run_app(&mut app) {
        app.root.fail_required(io::Error::other(error));
    }
    app.root.request_stop();
    let completed = app.root.complete_until(Instant::now());
    let foreground = app.root.parts().last().value();
    let domain = app.root.parts().previous().last().value();
    eprintln!(
        "YT282_D2_FINAL pid={} completion={completed:?} phase={:?} window_closed={} renderer_released={} c1_revision={} c1_stopped={} c1_owners={} counters={:?} synthetic_zero_deferred={} actual_positive_resize={} actual_driver_recovery_observed={} elapsed_ms={}",
        std::process::id(),
        app.root.phase(),
        foreground.window.phase() == WindowPhase::Closed,
        foreground.renderer.is_none(),
        domain.revision.get(),
        domain.stopped.get(),
        Rc::strong_count(domain),
        foreground.counters,
        foreground.zero_deferred,
        foreground.actual_resize,
        foreground.counters.recovery_requests > 0,
        app.started.elapsed().as_millis()
    );
    if let Some(error) = app.root.failures().primary() {
        return Err(error.to_string().into());
    }
    if !app.presented
        || !foreground.window_ready
        || !foreground.close_blocked
        || foreground.counters.present_calls < 2
        || foreground.counters.draw_calls != foreground.counters.present_calls
        || foreground.counters.submissions != foreground.counters.present_calls
        || foreground.counters.render_passes != foreground.counters.present_calls
        || foreground.window.phase() != WindowPhase::Closed
        || foreground.renderer.is_some()
        || !domain.stopped.get()
        || domain.revision.get() != 0
        || Rc::strong_count(domain) != 1
        || completed != Completion::Complete
        || app.root.phase() != Phase::Stopped
        || (diagnostic && (!foreground.zero_deferred || !foreground.actual_resize))
    {
        return Err("Window/GPU/sole C1 lifecycle acceptance incomplete".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn synthetic_discard_failure_closes_window_slot_but_never_reports_complete() {
        // No OS Window or GPU is created. Exercise the real consumer close
        // callback with its retained discard diagnostic after renderer release.
        let cleanup = Arc::new(FatalError {
            operation: "Surface::discard_texture",
            diagnostic: "Error in Surface::discard_texture: synthetic invalid Device".into(),
        });
        let foreground = Foreground {
            renderer: None,
            cleanup_error: Some(Arc::clone(&cleanup)),
            window: WindowSlot::new(Window::default_attributes()),
            counters: RenderCounters::default(),
            window_ready: false,
            diagnostic: false,
            zero_deferred: false,
            close_blocked: false,
            resize_requested: false,
            actual_resize: false,
        };
        let original = Arc::new(wgpu::Error::OutOfMemory {
            source: io::Error::other("synthetic primary submission OOM").into(),
        });
        let mut root = Builder::<io::Error>::new()
            .required_foreground(
                "synthetic released renderer",
                foreground,
                close_foreground as ForegroundClose,
            )
            .build();
        root.fail_required(io::Error::other(Arc::clone(&original)));
        for _ in 0..2 {
            assert_eq!(root.complete_until(Instant::now()), Completion::Pending);
            let part = root.parts().last().value();
            assert_eq!(part.window.phase(), WindowPhase::Closed);
            assert!(part.renderer.is_none());
            assert!(Arc::ptr_eq(part.cleanup_error.as_ref().unwrap(), &cleanup));
            let retained = root
                .failures()
                .primary()
                .unwrap()
                .get_ref()
                .unwrap()
                .downcast_ref::<Arc<wgpu::Error>>()
                .unwrap();
            assert!(Arc::ptr_eq(retained, &original));
            assert_eq!(part.counters.present_calls, 0);
        }
    }

    #[test]
    fn initial_deferred_assertion_accepts_each_zero_axis() {
        for size in [
            PhysicalSize::new(0, 0),
            PhysicalSize::new(0, 420),
            PhysicalSize::new(640, 0),
        ] {
            assert_eq!(initial_phase(size), SurfacePhase::DeferredZeroSize);
        }
        assert_eq!(
            initial_phase(PhysicalSize::new(640, 420)),
            SurfacePhase::Unconfigured
        );
    }
}
