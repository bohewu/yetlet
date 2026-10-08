//! Browser-only Rust/WASM entrypoint for the Yetlet Todo demo.
//!
//! DOM events are handled locally. This crate never imports Axum, HTTP client
//! libraries, or server-only capabilities.

#[cfg(target_arch = "wasm32")]
mod runtime {
    use std::cell::RefCell;
    use std::rc::Rc;

    use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};
    use web_sys::{Element, Event};
    use yetlet_dom::mount as mount_view;
    use yetlet_todo_ui::{Todo, browser_page, initial_todos, todo_list, toggle_first};

    struct ClickBinding {
        element: Element,
        callback: Closure<dyn FnMut(Event)>,
    }

    impl Drop for ClickBinding {
        fn drop(&mut self) {
            let _ = self.element.remove_event_listener_with_callback(
                "click",
                self.callback.as_ref().unchecked_ref(),
            );
        }
    }

    thread_local! {
        static CLICK_BINDING: RefCell<Option<ClickBinding>> = const { RefCell::new(None) };
    }

    fn document() -> Result<web_sys::Document, JsValue> {
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("window is not available"))?
            .document()
            .ok_or_else(|| JsValue::from_str("document is not available"))
    }

    /// Mount the browser page and register exactly one local button listener.
    #[wasm_bindgen]
    pub fn mount() -> Result<(), JsValue> {
        // Retiring a previous mount detaches its listener; no leaked closures.
        CLICK_BINDING.with(|binding| binding.borrow_mut().take());

        let document = document()?;
        let host = document
            .get_element_by_id("app")
            .ok_or_else(|| JsValue::from_str("missing #app mount point"))?;
        let todos: Rc<RefCell<Vec<Todo>>> = Rc::new(RefCell::new(initial_todos()));

        mount_view(&host, &browser_page(&todos.borrow()))?;

        let list_host = document
            .get_element_by_id("todo-list-host")
            .ok_or_else(|| JsValue::from_str("missing TodoList host"))?;
        let status = document
            .get_element_by_id("todo-status")
            .ok_or_else(|| JsValue::from_str("missing status element"))?;
        let button = document
            .get_element_by_id("toggle-first")
            .ok_or_else(|| JsValue::from_str("missing toggle button"))?;

        let local_state = todos.clone();
        let callback = Closure::<dyn FnMut(Event)>::new(move |_event| {
            let next_view = {
                let mut items = local_state.borrow_mut();
                if !toggle_first(&mut items) {
                    return;
                }
                todo_list(&items)
            };
            if let Err(error) = mount_view(&list_host, &next_view) {
                web_sys::console::error_1(&error);
                return;
            }
            let complete = local_state.borrow()[0].completed;
            status.set_text_content(Some(if complete {
                "First task: completed"
            } else {
                "First task: active"
            }));
        });
        button.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;
        CLICK_BINDING.with(|binding| {
            *binding.borrow_mut() = Some(ClickBinding {
                element: button,
                callback,
            });
        });
        document
            .document_element()
            .ok_or_else(|| JsValue::from_str("no document element"))?
            .set_attribute("data-yetlet-ready", "true")?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use runtime::mount;
