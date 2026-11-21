//! Standalone external consumer of public Yetlet package APIs.
//! These are separate feature paths, not a new host/input/runtime abstraction.

#[cfg(feature = "pure-web")]
pub fn pure_web_router() -> axum::Router {
    use yetlet::server_axum::App;

    async fn status() -> &'static str {
        "ready"
    }

    // Keep native Axum and original Yetlet App; do not construct Composition.
    App::new().get("/status", status).into_router()
}

#[cfg(feature = "local-headless")]
pub fn local_rc_custom_host() {
    use std::{cell::Cell, rc::Rc, time::Instant};
    use yetlet::composition::{Builder, Completion, Phase};

    let source = Rc::new(Cell::new(0_u64));
    let close_seen = Rc::new(Cell::new(false));
    let close_receipt = Rc::clone(&close_seen);
    let mut audit: Vec<u64> = Vec::new();

    // A real external application callback/borrowed Host, not a no-op marker.
    let mut root = Builder::<&'static str>::new()
        .required_local("caller-local Rc", Rc::clone(&source), move |_| {
            close_receipt.set(true);
            Ok(())
        })
        .borrowed("external audit", &mut audit)
        .build();

    assert!(root.all_required_ready());
    for next in 1..=3 {
        root.parts_mut()
            .previous()
            .last()
            .value_mut()
            .expect("local value active")
            .set(next);
        root.parts_mut()
            .last()
            .value_mut()
            .expect("audit active")
            .push(source.get());
    }
    assert_eq!(root.complete_until(Instant::now()), Completion::Complete);
    assert_eq!(root.phase(), Phase::Stopped);
    assert!(root.failures().primary().is_none());
    drop(root);
    assert_eq!(audit, [1, 2, 3]);
    assert!(close_seen.get());
}

#[cfg(feature = "native-udp")]
pub fn non_web_native_udp() -> std::io::Result<()> {
    use yetlet_hosts::{native_winit::WindowSlot, udp::UdpEndpoint};

    // Prepare the native slot without an OS window/event loop or renderer.
    let slot = WindowSlot::new(winit::window::Window::default_attributes());
    assert!(!slot.is_ready());
    assert!(slot.window().is_none());

    // Real owned local UDP socket, no Axum/Tokio or application codec.
    let mut endpoint = UdpEndpoint::bind_loopback(64)?;
    assert!(endpoint.local_addr()?.ip().is_loopback());
    assert!(endpoint.poll_bounded(0, |_, _| Ok(()))?.budget_reached);
    endpoint.close()?;
    assert!(endpoint.is_closed());
    Ok(())
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "pure-web")]
    #[test]
    fn direct_pure_web_app_is_external_and_does_not_require_composition() {
        let _router: axum::Router = super::pure_web_router();
    }

    #[cfg(feature = "local-headless")]
    #[test]
    fn caller_local_rc_and_custom_borrowed_audit_close_in_order() {
        super::local_rc_custom_host();
    }

    #[cfg(feature = "native-udp")]
    #[test]
    fn non_web_native_slot_and_udp_without_runtime() {
        super::non_web_native_udp().unwrap();
    }
}
