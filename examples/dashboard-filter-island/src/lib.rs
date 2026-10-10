//! Independently mounted filter island that preserves its SSR input node and focus.

#[cfg(target_arch = "wasm32")]
mod browser {
    use std::{cell::RefCell, rc::Rc};

    use serde::Deserialize;
    use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};
    use web_sys::{Document, Element, Event, EventTarget, HtmlInputElement};

    #[derive(Deserialize)]
    struct FilterState {
        schema_version: u32,
        component: String,
        items: Vec<String>,
    }

    struct Listener {
        target: EventTarget,
        callback: Closure<dyn FnMut(Event)>,
    }

    impl Listener {
        fn new(
            target: &EventTarget,
            callback: impl FnMut(Event) + 'static,
        ) -> Result<Self, JsValue> {
            let callback = Closure::new(callback);
            target.add_event_listener_with_callback("input", callback.as_ref().unchecked_ref())?;
            Ok(Self {
                target: target.clone(),
                callback,
            })
        }
    }

    impl Drop for Listener {
        fn drop(&mut self) {
            let _ = self.target.remove_event_listener_with_callback(
                "input",
                self.callback.as_ref().unchecked_ref(),
            );
        }
    }

    struct Runtime {
        root: Element,
        _listener: Listener,
    }

    impl Drop for Runtime {
        fn drop(&mut self) {
            let _ = self.root.remove_attribute("data-yetlet-activated");
            let _ = self.root.set_attribute("data-yetlet-disposed", "true");
        }
    }

    thread_local! {
        static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) };
    }

    fn document() -> Result<Document, JsValue> {
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("window missing"))?
            .document()
            .ok_or_else(|| JsValue::from_str("document missing"))
    }

    fn element(document: &Document, identifier: &str) -> Result<Element, JsValue> {
        document
            .get_element_by_id(identifier)
            .ok_or_else(|| JsValue::from_str(&format!("missing #{identifier}")))
    }

    fn render(
        document: &Document,
        input: &HtmlInputElement,
        list: &Element,
        count: &Element,
        items: &[String],
    ) -> Result<(), JsValue> {
        let query = input.value().to_lowercase();
        let matching: Vec<&String> = items
            .iter()
            .filter(|item| item.to_lowercase().contains(&query))
            .collect();
        list.set_text_content(None);
        for item in &matching {
            let row = document.create_element("li")?;
            row.set_text_content(Some(item));
            list.append_child(&row)?;
        }
        count.set_text_content(Some(&format!("{} matching items", matching.len())));
        Ok(())
    }

    #[wasm_bindgen]
    pub fn mount_filter() -> Result<(), JsValue> {
        dispose_filter();
        let document = document()?;
        let root = element(&document, "filter-island")?;
        if root.get_attribute("data-yetlet-island").as_deref() != Some("filter")
            || root.get_attribute("data-island-version").as_deref() != Some("1")
        {
            return Err(JsValue::from_str("unsupported filter boundary/version"));
        }
        let encoded = root
            .get_attribute("data-island-state")
            .ok_or_else(|| JsValue::from_str("missing filter state"))?;
        let state: FilterState = serde_json::from_str(&encoded)
            .map_err(|_| JsValue::from_str("invalid filter state"))?;
        if state.schema_version != 1 || state.component != "yetlet-dashboard-filter" {
            return Err(JsValue::from_str("unsupported filter state"));
        }

        let input: HtmlInputElement = element(&document, "filter-query")?.dyn_into()?;
        let list = element(&document, "filter-results")?;
        let count = element(&document, "filter-count")?;
        render(&document, &input, &list, &count, &state.items)?;

        let listener = {
            let document = document.clone();
            let input_for_event = input.clone();
            let list = list.clone();
            let count = count.clone();
            let items = Rc::new(state.items);
            Listener::new(input.as_ref(), move |_event| {
                if let Err(error) = render(&document, &input_for_event, &list, &count, &items) {
                    web_sys::console::error_1(&error);
                }
            })?
        };
        root.remove_attribute("data-yetlet-disposed")?;
        root.set_attribute("data-yetlet-activated", "true")?;
        RUNTIME.with(|runtime| {
            *runtime.borrow_mut() = Some(Runtime {
                root,
                _listener: listener,
            });
        });
        Ok(())
    }

    #[wasm_bindgen]
    pub fn dispose_filter() {
        RUNTIME.with(|runtime| {
            runtime.borrow_mut().take();
        });
    }
}

#[cfg(target_arch = "wasm32")]
pub use browser::{dispose_filter, mount_filter};
