//! Browser-only Rust/WASM entrypoint for the Yetlet Todo example.
//!
//! DOM events are local. An explicitly injected executor chooses between
//! the local state handler and semantic JSON HTTP commands. No raw DOM event,
//! persistent circuit, or server repository is included in this crate.

#[cfg(target_arch = "wasm32")]
mod runtime {
    use std::{cell::RefCell, rc::Rc};

    use wasm_bindgen::{JsCast, JsValue, closure::Closure, prelude::wasm_bindgen};
    use wasm_bindgen_futures::spawn_local;
    use web_sys::{Element, Event, HtmlInputElement};
    use yetlet_actions::{ActionExecutor, HttpExecutor, LocalExecutor};
    use yetlet_dom::mount as mount_view;
    use yetlet_todo_ui::{TodoCommand, TodoSnapshot, apply_todo_command, browser_page, todo_list};

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
        static CLICK_BINDINGS: RefCell<Vec<ClickBinding>> = const { RefCell::new(Vec::new()) };
    }

    fn document() -> Result<web_sys::Document, JsValue> {
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("window is not available"))?
            .document()
            .ok_or_else(|| JsValue::from_str("document is not available"))
    }

    fn get_element(document: &web_sys::Document, identifier: &str) -> Result<Element, JsValue> {
        document
            .get_element_by_id(identifier)
            .ok_or_else(|| JsValue::from_str(&format!("missing #{identifier}")))
    }

    fn register_click(
        element: Element,
        callback: Closure<dyn FnMut(Event)>,
    ) -> Result<(), JsValue> {
        element.add_event_listener_with_callback("click", callback.as_ref().unchecked_ref())?;
        CLICK_BINDINGS.with(|bindings| {
            bindings
                .borrow_mut()
                .push(ClickBinding { element, callback });
        });
        Ok(())
    }

    /// Mount the full client route, with separate local and HTTP executors.
    #[wasm_bindgen]
    pub fn mount() -> Result<(), JsValue> {
        CLICK_BINDINGS.with(|bindings| bindings.borrow_mut().clear());

        let document = document()?;
        let host = get_element(&document, "app")?;
        let local_state = Rc::new(RefCell::new(TodoSnapshot::initial()));

        mount_view(&host, &browser_page(&local_state.borrow().items))?;

        // The button remains in DOM while only its list subtree is replaced.
        let list_host = get_element(&document, "todo-list-host")?;
        let status = get_element(&document, "todo-status")?;
        let local_button = get_element(&document, "toggle-first")?;
        let local_executor = Rc::new(LocalExecutor::new({
            let local_state = Rc::clone(&local_state);
            move |command: TodoCommand| apply_todo_command(&mut local_state.borrow_mut(), command)
        }));
        let local_click = Closure::<dyn FnMut(Event)>::new(move |_event| {
            let executor = Rc::clone(&local_executor);
            let host = list_host.clone();
            let status = status.clone();
            spawn_local(async move {
                match executor.execute(TodoCommand::Toggle { id: 1 }).await {
                    Ok(snapshot) => {
                        if let Err(error) = mount_view(&host, &todo_list(&snapshot.items)) {
                            web_sys::console::error_1(&error);
                            return;
                        }
                        status.set_text_content(Some(if snapshot.items[0].completed {
                            "First task: completed"
                        } else {
                            "First task: active"
                        }));
                    }
                    Err(error) => {
                        status.set_text_content(Some(&format!("Local error: {}", error.code())));
                    }
                }
            });
        });
        register_click(local_button, local_click)?;

        // Remote events never move across the network. A local click creates a
        // TodoCommand and opts into an HTTP executor using ordinary fetch/JSON.
        let remote_input: HtmlInputElement = get_element(&document, "remote-title")?.dyn_into()?;
        let remote_button = get_element(&document, "create-remote")?;
        let remote_host = get_element(&document, "remote-list-host")?;
        let remote_status = get_element(&document, "remote-status")?;
        let http = Rc::new(
            HttpExecutor::new("/api/todos").map_err(|error| JsValue::from_str(error.code()))?,
        );
        let remote_click = Closure::<dyn FnMut(Event)>::new(move |_event| {
            let title = remote_input.value();
            let executor = Rc::clone(&http);
            let host = remote_host.clone();
            let status = remote_status.clone();
            status.set_text_content(Some("Saving to server..."));
            spawn_local(async move {
                match executor.execute(TodoCommand::Create { title }).await {
                    Ok(snapshot) => {
                        if let Err(error) = mount_view(&host, &todo_list(&snapshot.items)) {
                            web_sys::console::error_1(&error);
                            status.set_text_content(Some("Render error"));
                            return;
                        }
                        status.set_text_content(Some(&format!(
                            "Saved on server (revision {})",
                            snapshot.revision
                        )));
                    }
                    Err(error) => {
                        // Display only structured error codes, never raw HTML.
                        status.set_text_content(Some(&format!("Remote error: {}", error.code())));
                    }
                }
            });
        });
        register_click(remote_button, remote_click)?;

        document
            .document_element()
            .ok_or_else(|| JsValue::from_str("document element missing"))?
            .set_attribute("data-yetlet-ready", "true")?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use runtime::mount;
