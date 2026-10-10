//! Independently mounted counter island with explicit disposal and late-callback guard.

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use serde::Deserialize;
    use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};
    use web_sys::{Element, Event, EventTarget};

    #[derive(Deserialize)]
    struct CounterState {
        schema_version: u32,
        component: String,
        count: u32,
    }

    #[derive(Clone)]
    struct Lifetime(Rc<Cell<bool>>);

    impl Lifetime {
        fn new() -> Self {
            Self(Rc::new(Cell::new(true)))
        }
        fn active(&self) -> bool {
            self.0.get()
        }
        fn invalidate(&self) {
            self.0.set(false);
        }
    }

    struct Listener {
        target: EventTarget,
        event: &'static str,
        callback: Closure<dyn FnMut(Event)>,
    }

    impl Listener {
        fn new(
            target: &EventTarget,
            event: &'static str,
            callback: impl FnMut(Event) + 'static,
        ) -> Result<Self, JsValue> {
            let callback = Closure::new(callback);
            target.add_event_listener_with_callback(event, callback.as_ref().unchecked_ref())?;
            Ok(Self {
                target: target.clone(),
                event,
                callback,
            })
        }
    }

    impl Drop for Listener {
        fn drop(&mut self) {
            let _ = self.target.remove_event_listener_with_callback(
                self.event,
                self.callback.as_ref().unchecked_ref(),
            );
        }
    }

    struct Runtime {
        root: Element,
        lifetime: Lifetime,
        _listeners: Vec<Listener>,
    }

    impl Drop for Runtime {
        fn drop(&mut self) {
            self.lifetime.invalidate();
            let _ = self.root.remove_attribute("data-yetlet-activated");
            let _ = self.root.set_attribute("data-yetlet-disposed", "true");
        }
    }

    thread_local! {
        static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) };
    }

    fn element(identifier: &str) -> Result<Element, JsValue> {
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("window missing"))?
            .document()
            .ok_or_else(|| JsValue::from_str("document missing"))?
            .get_element_by_id(identifier)
            .ok_or_else(|| JsValue::from_str(&format!("missing #{identifier}")))
    }

    fn set_count(
        root: &Element,
        value: &Element,
        status: &Element,
        count: u32,
    ) -> Result<(), JsValue> {
        value.set_text_content(Some(&count.to_string()));
        value.set_attribute("data-count", &count.to_string())?;
        root.set_attribute("data-current-count", &count.to_string())?;
        status.set_text_content(Some(&format!("Counter value: {count}")));
        Ok(())
    }

    #[wasm_bindgen]
    pub fn mount_counter() -> Result<(), JsValue> {
        dispose_counter();
        let root = element("counter-island")?;
        if root.get_attribute("data-yetlet-island").as_deref() != Some("counter")
            || root.get_attribute("data-island-version").as_deref() != Some("1")
        {
            return Err(JsValue::from_str("unsupported counter boundary/version"));
        }
        let encoded = root
            .get_attribute("data-island-state")
            .ok_or_else(|| JsValue::from_str("missing counter state"))?;
        let state: CounterState = serde_json::from_str(&encoded)
            .map_err(|_| JsValue::from_str("invalid counter state"))?;
        if state.schema_version != 1 || state.component != "yetlet-dashboard-counter" {
            return Err(JsValue::from_str("unsupported counter state"));
        }

        let value = element("counter-value")?;
        let status = element("counter-status")?;
        let increment = element("counter-increment")?;
        let delayed = element("counter-delayed")?;
        let initial = root
            .get_attribute("data-current-count")
            .and_then(|value| value.parse().ok())
            .unwrap_or(state.count);
        let count = Rc::new(Cell::new(initial));
        set_count(&root, &value, &status, initial)?;

        let lifetime = Lifetime::new();
        let increment_listener = {
            let root = root.clone();
            let value = value.clone();
            let status = status.clone();
            let count = Rc::clone(&count);
            let lifetime = lifetime.clone();
            Listener::new(increment.as_ref(), "click", move |_event| {
                if !lifetime.active() {
                    return;
                }
                let next = count.get().saturating_add(1);
                count.set(next);
                if let Err(error) = set_count(&root, &value, &status, next) {
                    web_sys::console::error_1(&error);
                }
            })?
        };
        let delayed_listener = {
            let root = root.clone();
            let value = value.clone();
            let status_for_timeout = status.clone();
            let status_for_click = status.clone();
            let count = Rc::clone(&count);
            let lifetime = lifetime.clone();
            Listener::new(delayed.as_ref(), "click", move |_event| {
                if !lifetime.active() {
                    return;
                }
                status_for_click.set_text_content(Some("Delayed increment queued"));
                let root = root.clone();
                let value = value.clone();
                let status = status_for_timeout.clone();
                let count = Rc::clone(&count);
                let lifetime = lifetime.clone();
                let callback = Closure::once_into_js(move || {
                    if !lifetime.active() {
                        return;
                    }
                    let next = count.get().saturating_add(1);
                    count.set(next);
                    if let Err(error) = set_count(&root, &value, &status, next) {
                        web_sys::console::error_1(&error);
                    }
                });
                if let Some(window) = web_sys::window() {
                    let _ = window.set_timeout_with_callback_and_timeout_and_arguments_0(
                        callback.unchecked_ref(),
                        180,
                    );
                }
            })?
        };

        root.remove_attribute("data-yetlet-disposed")?;
        root.set_attribute("data-yetlet-activated", "true")?;
        RUNTIME.with(|runtime| {
            *runtime.borrow_mut() = Some(Runtime {
                root,
                lifetime,
                _listeners: vec![increment_listener, delayed_listener],
            });
        });
        Ok(())
    }

    #[wasm_bindgen]
    pub fn dispose_counter() {
        RUNTIME.with(|runtime| {
            runtime.borrow_mut().take();
        });
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{dispose_counter, mount_counter};
