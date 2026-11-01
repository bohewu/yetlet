#[cfg(target_arch = "wasm32")]
mod browser {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    use wasm_bindgen::{JsCast, prelude::*};
    use yetlet::view::{Element, View};
    struct Runtime {
        button: web_sys::Element,
        click: Closure<dyn FnMut(web_sys::Event)>,
    }
    impl Drop for Runtime {
        fn drop(&mut self) {
            let _ = self
                .button
                .remove_event_listener_with_callback("click", self.click.as_ref().unchecked_ref());
        }
    }
    thread_local! {static OWNER:RefCell<Option<Runtime>>=const {RefCell::new(None)};}
    #[wasm_bindgen]
    pub fn activate() -> Result<(), JsValue> {
        OWNER.with(|owner| {
            if owner.borrow().is_some() {
                return Err(JsValue::from_str("counter is already active"));
            }
            let document = web_sys::window()
                .and_then(|w| w.document())
                .ok_or_else(|| JsValue::from_str("document unavailable"))?;
            let host = document
                .get_element_by_id("island-host")
                .ok_or_else(|| JsValue::from_str("counter host unavailable"))?;
            let view: View = Element::new("div")
                .child(
                    Element::new("button")
                        .attribute("id", "increment")
                        .attribute("type", "button")
                        .child(View::text("Increment")),
                )
                .child(
                    Element::new("span")
                        .attribute("id", "count")
                        .child(View::text("0")),
                )
                .into();
            yetlet_browser::dom::mount(&host, &view)?;
            let button = document
                .get_element_by_id("increment")
                .ok_or_else(|| JsValue::from_str("counter button unavailable"))?;
            let text = document
                .get_element_by_id("count")
                .ok_or_else(|| JsValue::from_str("counter text unavailable"))?;
            let value = Rc::new(Cell::new(0u32));
            let click = Closure::wrap(Box::new(move |_: web_sys::Event| {
                let n = value.get().saturating_add(1);
                value.set(n);
                text.set_text_content(Some(&n.to_string()));
            }) as Box<dyn FnMut(web_sys::Event)>);
            button.add_event_listener_with_callback("click", click.as_ref().unchecked_ref())?;
            *owner.borrow_mut() = Some(Runtime { button, click });
            Ok(())
        })
    }
    #[wasm_bindgen]
    pub fn dispose() {
        OWNER.with(|owner| {
            owner.borrow_mut().take();
        });
    }
}
