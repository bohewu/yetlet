//! Optional caller-thread Winit foreground Window adapter.
//!
//! The caller owns its EventLoop, runs it on the supported foreground thread
//! and forwards actual ApplicationHandler callbacks. No automatic scheduler,
//! domain, GPU, renderer or hidden worker is created here.
//! A real Window is created in `resumed`; only a matching nonzero-size,
//! non-occluded RedrawRequested establishes Window Ready. This is NOT GPU
//! presentation or engine-ready evidence.
//!
//! The slot is deliberately !Send/!Sync. Renderer owners may clone
//! Arc<Window>, but must release their clone before this slot can claim close.

use std::{io, marker::PhantomData, rc::Rc, sync::Arc};
use winit::{
    dpi::PhysicalSize,
    event::WindowEvent,
    event_loop::ActiveEventLoop,
    window::{Window, WindowAttributes, WindowId},
};

/// Lifecycle of the native Window only, not optional GPU readiness.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowPhase {
    Prepared,
    Created,
    Ready,
    Suspended,
    /// The OS reported destruction; stale queued redraw events are rejected.
    Destroyed,
    Closed,
}

/// One filtered caller-delivered WindowEvent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowSignal {
    Ignored,
    RedrawReady,
    CloseRequested,
    /// The OS destroyed this Window; it is no longer ready or usable.
    Destroyed,
    Resized(PhysicalSize<u32>),
    Occluded(bool),
}

/// Caller-owned Window slot; no global Send/Sync bound or EventLoop ownership.
pub struct WindowSlot {
    attributes: WindowAttributes,
    window: Option<Arc<Window>>,
    phase: WindowPhase,
    last_size: Option<PhysicalSize<u32>>,
    occluded: bool,
    close_requested: bool,
    redraw_event_count: u64,
    _foreground_only: PhantomData<Rc<()>>,
}

impl WindowSlot {
    /// Pure preparation: no OS window or EventLoop is created here.
    pub fn new(attributes: WindowAttributes) -> Self {
        Self {
            attributes,
            window: None,
            phase: WindowPhase::Prepared,
            last_size: None,
            occluded: false,
            close_requested: false,
            redraw_event_count: 0,
            _foreground_only: PhantomData,
        }
    }

    pub fn phase(&self) -> WindowPhase {
        self.phase
    }
    pub fn is_ready(&self) -> bool {
        // Query actual current visibility too: a caller can hide a Window
        // without receiving an Occluded/Resized event first.
        self.phase == WindowPhase::Ready
            && !self.occluded
            && !self.close_requested
            && self.last_size.is_some_and(nonzero_size)
            && self
                .window
                .as_ref()
                .is_some_and(|w| w.is_visible() == Some(true))
    }
    pub fn window(&self) -> Option<&Window> {
        (self.phase != WindowPhase::Destroyed).then_some(())?;
        self.window.as_deref()
    }
    pub fn window_id(&self) -> Option<WindowId> {
        self.window().map(Window::id)
    }
    pub fn last_size(&self) -> Option<PhysicalSize<u32>> {
        self.last_size
    }
    /// Count of accepted visible redraw events, not GPU frames presented.
    pub fn redraw_event_count(&self) -> u64 {
        self.redraw_event_count
    }
    pub fn close_requested(&self) -> bool {
        self.close_requested
    }

    /// Clone the shared owned Window for an independently selected renderer.
    /// An outstanding `Arc<Window>` prevents honest close/suspend completion.
    pub fn shared_window(&self) -> Option<Arc<Window>> {
        (self.phase != WindowPhase::Destroyed)
            .then(|| self.window.clone())
            .flatten()
    }

    /// Create the real Window only in the caller's Winit resumed callback.
    /// Repeated resumed is idempotent; after suspended it may recreate.
    ///
    /// # Errors
    /// Real OS Window creation error, or NotConnected after terminal close.
    pub fn resumed(&mut self, active: &ActiveEventLoop) -> io::Result<WindowId> {
        if self.phase == WindowPhase::Closed {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Window slot closed",
            ));
        }
        if self.phase == WindowPhase::Destroyed {
            // The OS destroyed the old Window; never reuse its stale handle.
            // Another owning renderer must release its Arc before recreation.
            self.release_window()?;
            self.phase = WindowPhase::Suspended;
        }
        if let Some(window) = self.window.as_ref() {
            return Ok(window.id());
        }
        let window = active
            .create_window(self.attributes.clone())
            .map_err(io::Error::other)?;
        let id = window.id();
        self.last_size = Some(window.inner_size());
        self.window = Some(Arc::new(window));
        self.occluded = false;
        self.close_requested = false;
        self.phase = WindowPhase::Created;
        self.request_redraw()?;
        Ok(id)
    }

    /// Forward the caller's actual Winit WindowEvent; ignores unrelated IDs.
    /// Does not execute any domain step, GPU draw or event-loop exit itself.
    pub fn observe_window_event(&mut self, id: WindowId, event: &WindowEvent) -> WindowSignal {
        let Some(window) = self.window.as_ref() else {
            return WindowSignal::Ignored;
        };
        if window.id() != id || self.phase == WindowPhase::Destroyed {
            return WindowSignal::Ignored;
        }
        match event {
            WindowEvent::Destroyed => {
                self.phase = WindowPhase::Destroyed;
                self.close_requested = true;
                self.last_size = None;
                WindowSignal::Destroyed
            }
            WindowEvent::RedrawRequested
                if !self.occluded
                    && !self.close_requested
                    && window.is_visible() == Some(true)
                    && self.last_size.is_some_and(nonzero_size) =>
            {
                self.phase = WindowPhase::Ready;
                self.redraw_event_count += 1;
                WindowSignal::RedrawReady
            }
            WindowEvent::RedrawRequested => {
                // Invalidate an earlier Ready after a hidden/unknown-visible
                // redraw; never keep a stale Running Window receipt.
                self.phase = WindowPhase::Created;
                WindowSignal::Ignored
            }
            WindowEvent::CloseRequested => {
                self.close_requested = true;
                self.phase = WindowPhase::Created;
                WindowSignal::CloseRequested
            }
            WindowEvent::Resized(size) => {
                self.last_size = Some(*size);
                self.phase = WindowPhase::Created;
                if nonzero_size(*size) && !self.occluded && window.is_visible() == Some(true) {
                    window.request_redraw();
                }
                WindowSignal::Resized(*size)
            }
            WindowEvent::Occluded(occluded) => {
                self.occluded = *occluded;
                self.phase = WindowPhase::Created;
                if !occluded && window.is_visible() == Some(true) {
                    window.request_redraw();
                }
                WindowSignal::Occluded(*occluded)
            }
            _ => WindowSignal::Ignored,
        }
    }

    /// Request OS redraw without claiming Window or GPU readiness.
    ///
    /// # Errors
    /// NotConnected when no live Window belongs to this slot.
    pub fn request_redraw(&self) -> io::Result<()> {
        if self.phase == WindowPhase::Destroyed {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "OS Window destroyed",
            ));
        }
        let window = self
            .window
            .as_ref()
            .ok_or_else(|| io::Error::new(io::ErrorKind::NotConnected, "no native Window"))?;
        window.request_redraw();
        Ok(())
    }

    fn release_window(&mut self) -> io::Result<()> {
        if let Some(original) = self.window.take() {
            // Atomic ownership resolution: a concurrent Weak upgrade cannot
            // win between a strong_count check and a later Drop.
            match Arc::try_unwrap(original) {
                Ok(window) => drop(window),
                Err(original) => {
                    self.window = Some(original); // retain exact owner on failure
                    return Err(io::Error::new(
                        io::ErrorKind::WouldBlock,
                        "other Window handles remain: release the renderer first",
                    ));
                }
            }
        }
        self.last_size = None;
        self.occluded = false;
        self.close_requested = false;
        Ok(())
    }

    /// Release the actual native Window for an optional Winit suspend callback.
    /// The owning root stays alive; a later resumed can recreate it.
    ///
    /// # Errors
    /// WouldBlock while a renderer or other owner still holds Window Arc.
    pub fn suspended(&mut self) -> io::Result<()> {
        if self.phase == WindowPhase::Closed {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "Window slot closed",
            ));
        }
        self.release_window()?;
        self.phase = WindowPhase::Suspended;
        Ok(())
    }

    /// Idempotent explicit Window close; does not stop domain/EventLoop/worker.
    ///
    /// # Errors
    /// WouldBlock if another Window handle still exists, allowing the
    /// caller-owned Composition to retain Pending and retry after its release.
    pub fn close(&mut self) -> io::Result<()> {
        if self.phase == WindowPhase::Closed {
            return Ok(());
        }
        self.release_window()?;
        self.phase = WindowPhase::Closed;
        Ok(())
    }
}
fn nonzero_size(size: PhysicalSize<u32>) -> bool {
    size.width > 0 && size.height > 0
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prepared_creates_no_os_window_and_never_claims_ready() {
        let mut slot = WindowSlot::new(Window::default_attributes());
        assert_eq!(slot.phase(), WindowPhase::Prepared);
        assert!(!slot.is_ready());
        assert_eq!(slot.redraw_event_count(), 0);
        assert_eq!(slot.window_id(), None);
        assert_eq!(
            slot.request_redraw().unwrap_err().kind(),
            io::ErrorKind::NotConnected
        );
        slot.close().unwrap();
        slot.close().unwrap();
        assert_eq!(slot.phase(), WindowPhase::Closed);
        assert_eq!(
            slot.request_redraw().unwrap_err().kind(),
            io::ErrorKind::NotConnected
        );
    }
    #[test]
    fn suspended_without_a_window_remains_unready_until_resumed() {
        let mut slot = WindowSlot::new(Window::default_attributes());
        slot.suspended().unwrap();
        assert_eq!(slot.phase(), WindowPhase::Suspended);
        assert!(!slot.is_ready());
        assert_eq!(slot.window_id(), None);
        slot.close().unwrap();
        assert_eq!(
            slot.suspended().unwrap_err().kind(),
            io::ErrorKind::NotConnected
        );
    }
}
