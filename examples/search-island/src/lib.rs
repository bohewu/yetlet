//! Search island: progressively enhance pre-rendered native SSR markup.
//!
//! The island does not replace its SSR parent/labels/input or initial results.
//! After an input event it owns only #search-results and the aria-live counter.
//! No event transport, API client, persistent circuit or remote RPC is used.

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::cell::RefCell;

    use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};
    use web_sys::{Event, HtmlInputElement};
    use yetlet_browser::dom::mount as mount_view;
    use yetlet_todo_ui::{decode_search_state, filtered_todos, search_results};

    struct InputBinding {
        input: HtmlInputElement,
        callback: Closure<dyn FnMut(Event)>,
    }

    impl Drop for InputBinding {
        fn drop(&mut self) {
            let _ = self.input.remove_event_listener_with_callback(
                "input",
                self.callback.as_ref().unchecked_ref(),
            );
        }
    }

    thread_local! {
        static INPUT_BINDING: RefCell<Option<InputBinding>> = const { RefCell::new(None) };
    }

    #[wasm_bindgen]
    pub fn mount_search() -> Result<(), JsValue> {
        INPUT_BINDING.with(|binding| binding.borrow_mut().take());

        let document = web_sys::window()
            .ok_or_else(|| JsValue::from_str("window missing"))?
            .document()
            .ok_or_else(|| JsValue::from_str("document missing"))?;
        let root = document
            .get_element_by_id("search-island")
            .ok_or_else(|| JsValue::from_str("search island boundary missing"))?;
        if root.get_attribute("data-yetlet-island").as_deref() != Some("search")
            || root.get_attribute("data-island-version").as_deref() != Some("1")
        {
            return Err(JsValue::from_str("unsupported island boundary/version"));
        }
        let input: HtmlInputElement = document
            .get_element_by_id("search-query")
            .ok_or_else(|| JsValue::from_str("search input missing"))?
            .dyn_into()?;
        let results = document
            .get_element_by_id("search-results")
            .ok_or_else(|| JsValue::from_str("results host missing"))?;
        let counter = document
            .get_element_by_id("search-count")
            .ok_or_else(|| JsValue::from_str("aria-live counter missing"))?;

        let encoded = root
            .get_attribute("data-island-state")
            .ok_or_else(|| JsValue::from_str("missing island state"))?;
        let source = decode_search_state(&encoded)
            .map_err(|_| JsValue::from_str("invalid or incompatible island state"))?;
        let input_for_event = input.clone();
        let callback = Closure::<dyn FnMut(Event)>::new(move |_event| {
            let matches = filtered_todos(&source, &input_for_event.value());
            let view = search_results(&matches);
            if let Err(error) = mount_view(&results, &view) {
                web_sys::console::error_1(&error);
                return;
            }
            counter.set_text_content(Some(&format!("{} matching tasks", matches.len())));
        });

        input.add_event_listener_with_callback("input", callback.as_ref().unchecked_ref())?;
        INPUT_BINDING.with(|binding| {
            *binding.borrow_mut() = Some(InputBinding {
                input: input.clone(),
                callback,
            });
        });

        // Preserve the initial server-rendered nodes for the empty query.
        // If the user typed during lazy loading, replay their current value
        // once the Rust input listener has attached.
        if !input.value().is_empty() {
            input.dispatch_event(&Event::new("input")?)?;
        }
        root.set_attribute("data-yetlet-activated", "true")?;
        Ok(())
    }
}
#[cfg(target_arch = "wasm32")]
pub use browser::mount_search;
