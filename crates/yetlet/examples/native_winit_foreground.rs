//! YT-282-D1 real foreground Winit/Composition consumer, without GPU.
//! One caller-thread EventLoop creates a real OS Window only at resumed;
//! matching RedrawRequested confirms Window readiness, not GPU presentation.
//! Original application C1 is a single caller-local Rc domain; no second owner.

use std::{
    cell::Cell,
    error::Error,
    io,
    rc::Rc,
    time::{Duration, Instant},
};
use yetlet::composition::{Attached, Builder, Completion, Composition, Empty, Local, Phase};
use yetlet::native_winit::winit::{
    application::ApplicationHandler,
    dpi::LogicalSize,
    event::WindowEvent,
    event_loop::{ActiveEventLoop, ControlFlow, EventLoop},
    window::{Window, WindowId},
};
use yetlet::native_winit::{WindowPhase, WindowSignal, WindowSlot};

#[derive(Default)]
struct C1Domain {
    revision: Cell<u64>,
    stopped: Cell<bool>,
}

fn stop_domain(domain: &mut Rc<C1Domain>) -> io::Result<()> {
    domain.stopped.set(true);
    Ok(())
}

fn close_window(window: &mut WindowSlot) -> io::Result<()> {
    window.close()
}

type DomainClose = fn(&mut Rc<C1Domain>) -> io::Result<()>;
type WindowClose = fn(&mut WindowSlot) -> io::Result<()>;
type Parts =
    Attached<Attached<Empty, Local<Rc<C1Domain>, DomainClose>>, Local<WindowSlot, WindowClose>>;
type Root = Composition<io::Error, Parts>;

struct Probe {
    root: Root,
    started: Instant,
    redraw_ready: bool,
    created: bool,
    rejected_early_close: bool,
    redraw_count: u64,
    created_id: Option<WindowId>,
    negative_mode: bool,
    hidden_rejected: bool,
    hidden_after_ready_rejected: bool,
    destroyed_rejected: bool,
    late_redraw_ignored: bool,
}

impl Probe {
    fn fail(&mut self, active: &ActiveEventLoop, error: io::Error) {
        eprintln!("YT282_D1_FAILURE cause={error}");
        self.root.fail_required(error);
        active.exit();
    }
}

impl ApplicationHandler for Probe {
    fn resumed(&mut self, active: &ActiveEventLoop) {
        let Some(slot) = self.root.parts_mut().last().value_mut() else {
            return; // late Winit resume after a recorded required failure
        };
        match slot.resumed(active) {
            Ok(id) => {
                self.created = true;
                self.created_id = Some(id);
                assert_eq!(slot.phase(), WindowPhase::Created);
                assert!(!slot.is_ready());
                let handle = slot.shared_window().expect("actual OS Window");
                let early_close = slot.close().unwrap_err();
                self.rejected_early_close = early_close.kind() == io::ErrorKind::WouldBlock;
                drop(handle); // actual Window stays in its sole foreground slot
                eprintln!(
                    "YT282_D1_CREATED pid={} window_id={id:?} size={:?} ready=false close_while_shared_rejected={}",
                    std::process::id(),
                    slot.last_size(),
                    self.rejected_early_close,
                );
                if self.negative_mode {
                    // Deterministic negative stimulus on the actual hidden
                    // X11 window: a queued redraw must never imply visible.
                    assert_eq!(slot.window().unwrap().is_visible(), Some(false));
                    assert_eq!(
                        slot.observe_window_event(id, &WindowEvent::RedrawRequested),
                        WindowSignal::Ignored
                    );
                    assert!(!slot.is_ready());
                    self.hidden_rejected = true;
                    eprintln!(
                        "YT282_D1_HIDDEN_REJECTED pid={} actual_window={id:?} synthetic_redraw_rejected=true",
                        std::process::id()
                    );
                    slot.window().unwrap().set_visible(true);
                    slot.request_redraw().expect("request real visible redraw");
                }
            }
            Err(error) => self.fail(active, error),
        }
    }

    fn window_event(&mut self, active: &ActiveEventLoop, id: WindowId, event: WindowEvent) {
        let Some(slot) = self.root.parts_mut().last().value_mut() else {
            // An already-failed/stopping root denies mutation; a late queued
            // redraw must not panic or bypass explicit cleanup.
            return;
        };
        let signal = slot.observe_window_event(id, &event);
        match signal {
            WindowSignal::RedrawReady => {
                self.redraw_count += 1;
                assert!(self.root.parts_mut().last().mark_ready());
                if self.root.all_required_ready() {
                    let slot = self.root.parts().last().value();
                    let domain = self.root.parts().previous().last().value();
                    assert_eq!(domain.revision.get(), 0, "Winit cannot advance domain");
                    self.redraw_ready = true;
                    eprintln!(
                        "YT282_D1_WINDOW_READY pid={} window_id={id:?} size={:?} redraw_events={} gpu_present=false domain_revision={} root_phase={:?}",
                        std::process::id(),
                        slot.last_size(),
                        slot.redraw_event_count(),
                        domain.revision.get(),
                        self.root.phase(),
                    );
                    if self.negative_mode {
                        // Negative method events are explicitly synthetic;
                        // the preceding Ready callback itself was real X11.                        // The OS Window was actually visible and RedrawReady.
                        // Hiding it later must revoke live Window readiness
                        // even before any platform Occluded callback arrives.
                        let slot = self.root.parts_mut().last().value_mut().unwrap();
                        slot.window().unwrap().set_visible(false);
                        assert_eq!(slot.window().unwrap().is_visible(), Some(false));
                        assert!(!slot.is_ready());
                        let before = slot.redraw_event_count();
                        assert_eq!(
                            slot.observe_window_event(id, &WindowEvent::RedrawRequested),
                            WindowSignal::Ignored
                        );
                        assert_eq!(slot.phase(), WindowPhase::Created);
                        assert_eq!(slot.redraw_event_count(), before);
                        self.hidden_after_ready_rejected = true;
                        eprintln!(
                            "YT282_D1_HIDDEN_AFTER_READY_REJECTED pid={} actual_window={id:?} synthetic_redraw_rejected=true event_count_unchanged=true",
                            std::process::id()
                        );
                        assert_eq!(
                            slot.observe_window_event(id, &WindowEvent::Destroyed),
                            WindowSignal::Destroyed
                        );
                        assert!(!slot.is_ready());
                        assert_eq!(slot.window_id(), None);
                        assert_eq!(
                            slot.observe_window_event(id, &WindowEvent::RedrawRequested),
                            WindowSignal::Ignored
                        );
                        self.destroyed_rejected = true;
                        eprintln!(
                            "YT282_D1_DESTROYED_REJECTED pid={} window_id={id:?} synthetic_destroyed_followup_redraw_ignored=true",
                            std::process::id()
                        );
                        self.fail(
                            active,
                            io::Error::other("injected required failure for late event regression"),
                        );
                        self.window_event(active, id, WindowEvent::RedrawRequested);
                        self.late_redraw_ignored = true;
                    } else {
                        active.exit();
                    }
                }
            }
            WindowSignal::CloseRequested => {
                self.fail(
                    active,
                    io::Error::other("Window close requested before readiness"),
                );
            }
            WindowSignal::Destroyed => {
                self.fail(
                    active,
                    io::Error::other("OS Window destroyed before normal stop"),
                );
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, active: &ActiveEventLoop) {
        if self.started.elapsed() > Duration::from_secs(6) {
            self.fail(
                active,
                io::Error::new(io::ErrorKind::TimedOut, "real Winit redraw not observed"),
            );
        } else {
            active.set_control_flow(ControlFlow::WaitUntil(
                self.started + Duration::from_secs(6),
            ));
        }
    }

    fn exiting(&mut self, _active: &ActiveEventLoop) {
        // Preserve the sole domain owner in the root until shutdown, and
        // close the actual Window before the domain's own closure.
        self.root.request_stop();
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let negative_mode = matches!(args.as_slice(), [mode] if mode == "--negative");
    if !args.is_empty() && !negative_mode {
        return Err("usage: native_winit_foreground [--negative]".into());
    }
    let event_loop = EventLoop::new()?; // created on the actual foreground thread
    let domain = Rc::new(C1Domain::default());
    let window = WindowSlot::new(
        Window::default_attributes()
            .with_title("Yetlet M28 D1 Winit Foreground")
            .with_inner_size(LogicalSize::new(640.0, 420.0))
            .with_visible(!negative_mode),
    );
    let mut root = Builder::<io::Error>::new()
        .required_local(
            "C1 sole owner",
            Rc::clone(&domain),
            stop_domain as DomainClose,
        )
        .required_foreground("native window", window, close_window as WindowClose)
        .build();
    assert!(!root.all_required_ready());
    let mut app = Probe {
        root,
        started: Instant::now(),
        redraw_ready: false,
        created: false,
        rejected_early_close: false,
        redraw_count: 0,
        created_id: None,
        negative_mode,
        hidden_rejected: false,
        hidden_after_ready_rejected: false,
        destroyed_rejected: false,
        late_redraw_ignored: false,
    };
    let loop_result = event_loop.run_app(&mut app);
    if let Err(error) = loop_result {
        app.root.fail_required(io::Error::other(error));
    }
    let completed = app.root.complete_until(Instant::now());
    let phase = app.root.phase();
    let stopped = domain.stopped.get();
    let slot = app.root.parts().last().value();
    eprintln!(
        "YT282_D1_FINAL pid={} created={} redraw_ready={} redraw_count={} window_closed={} domain_stopped={} close_while_shared_rejected={} completion={completed:?} phase={phase:?}",
        std::process::id(),
        app.created,
        app.redraw_ready,
        app.redraw_count,
        slot.phase() == WindowPhase::Closed,
        stopped,
        app.rejected_early_close,
    );
    if negative_mode {
        let injected_failure = app
            .root
            .failures()
            .primary()
            .is_some_and(|error| error.to_string().contains("injected required failure"));
        if !app.created
            || !app.redraw_ready
            || !app.hidden_rejected
            || !app.hidden_after_ready_rejected
            || !app.destroyed_rejected
            || !app.late_redraw_ignored
            || !app.rejected_early_close
            || app.redraw_count == 0
            || slot.phase() != WindowPhase::Closed
            || !stopped
            || completed != Completion::Complete
            || phase != Phase::Failed
            || domain.revision.get() != 0
            || !injected_failure
        {
            return Err("negative visible/Destroyed/late-event lifecycle failed".into());
        }
        eprintln!(
            "YT282_D1_NEGATIVE_PASS pid={} actual_visible_redraw=true hidden_synthetic_rejected=true hidden_after_ready_rejected=true destroyed_synthetic_rejected=true late_event_after_failure_ignored=true cleanup=Complete primary_preserved=true",
            std::process::id()
        );
        return Ok(());
    }
    if !app.created
        || !app.redraw_ready
        || !app.rejected_early_close
        || app.redraw_count == 0
        || slot.phase() != WindowPhase::Closed
        || !stopped
        || domain.revision.get() != 0
        || completed != Completion::Complete
        || phase != Phase::Stopped
        || app.root.failures().primary().is_some()
    {
        return Err("real caller-thread Window/Composition acceptance failed".into());
    }
    Ok(())
}
