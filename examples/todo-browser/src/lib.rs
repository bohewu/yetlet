//! Browser-only Rust/WASM entrypoint for the Yetlet Todo example.
//!
//! DOM events are local. Explicit typed executors choose the owned local graph
//! or semantic JSON HTTP commands. No raw DOM event, persistent circuit, or
//! server repository is included in this crate.

#[cfg(target_arch = "wasm32")]
mod runtime {
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
    use wasm_bindgen_futures::spawn_local;
    use web_sys::{Document, Element, HtmlInputElement, Node};
    use yetlet_actions::{ActionError, ActionExecutor, HttpExecutor};
    use yetlet_core::binding::BindingKey;
    use yetlet_dom::mount as mount_view;
    use yetlet_dom_keyed::{DomEventListener, KeyedChildren, KeyedEntry};
    use yetlet_reactive::{GraphHandle, LocalGraph, ReactiveError};
    use yetlet_todo_ui::{
        Todo, TodoCommand, TodoSnapshot, apply_todo_command, browser_page, todo_list,
    };

    type TodoGraph = LocalGraph<TodoSnapshot, TodoCommand, ActionError>;
    type TodoGraphHandle = GraphHandle<TodoSnapshot, TodoCommand, ActionError>;

    struct TodoRow {
        text: Node,
    }

    #[derive(Clone)]
    struct LocalGraphExecutor {
        handle: TodoGraphHandle,
        committed: Rc<RefCell<TodoSnapshot>>,
    }

    impl ActionExecutor<TodoCommand> for LocalGraphExecutor {
        async fn execute(&self, command: TodoCommand) -> Result<TodoSnapshot, ActionError> {
            self.handle.dispatch(command).map_err(graph_action_error)?;
            Ok(self.committed.borrow().clone())
        }
    }

    #[derive(Clone)]
    struct RuntimeLifetime(Rc<Cell<bool>>);

    impl RuntimeLifetime {
        fn new() -> Self {
            Self(Rc::new(Cell::new(true)))
        }

        fn is_active(&self) -> bool {
            self.0.get()
        }

        fn invalidate(&self) {
            self.0.set(false);
        }
    }

    struct BrowserRuntime {
        lifetime: RuntimeLifetime,
        _graph: TodoGraph,
        _bindings: Vec<DomEventListener>,
    }

    impl Drop for BrowserRuntime {
        fn drop(&mut self) {
            self.lifetime.invalidate();
        }
    }

    thread_local! {
        static RUNTIME: RefCell<Option<BrowserRuntime>> = const { RefCell::new(None) };
    }

    fn document() -> Result<Document, JsValue> {
        web_sys::window()
            .ok_or_else(|| JsValue::from_str("window is not available"))?
            .document()
            .ok_or_else(|| JsValue::from_str("document is not available"))
    }

    fn get_element(document: &Document, identifier: &str) -> Result<Element, JsValue> {
        document
            .get_element_by_id(identifier)
            .ok_or_else(|| JsValue::from_str(&format!("missing #{identifier}")))
    }

    fn graph_action_error(error: ReactiveError<ActionError>) -> ActionError {
        match error {
            ReactiveError::Reducer(error) | ReactiveError::Commit(error) => error,
            ReactiveError::Disposed
            | ReactiveError::Faulted
            | ReactiveError::ObserverIdExhausted
            | ReactiveError::ReentrantLimit { .. } => ActionError::Internal,
        }
    }

    fn todo_key(item: &Todo) -> Result<BindingKey, JsValue> {
        BindingKey::new(item.id.to_string()).map_err(|error| JsValue::from_str(&error.to_string()))
    }

    fn todo_class(item: &Todo) -> &'static str {
        if item.completed {
            "is-complete"
        } else {
            "is-open"
        }
    }

    fn preflight_todo_row(element: Element, item: &Todo) -> Result<KeyedEntry<TodoRow>, JsValue> {
        if !element.tag_name().eq_ignore_ascii_case("li")
            || element.get_attribute("data-id").as_deref() != Some(&item.id.to_string())
            || element.get_attribute("data-completed").as_deref()
                != Some(if item.completed { "true" } else { "false" })
            || element.get_attribute("class").as_deref() != Some(todo_class(item))
        {
            return Err(JsValue::from_str("Todo keyed row preflight mismatch"));
        }
        let text = element
            .first_child()
            .ok_or_else(|| JsValue::from_str("Todo keyed row text missing"))?;
        if text.node_type() != Node::TEXT_NODE
            || text.next_sibling().is_some()
            || text.node_value().as_deref() != Some(item.title.as_str())
        {
            return Err(JsValue::from_str("Todo keyed row text mismatch"));
        }
        Ok(KeyedEntry::new(element, TodoRow { text }))
    }

    fn bind_todo_list(host: &Element, items: &[Todo]) -> Result<KeyedChildren<TodoRow>, JsValue> {
        if host.children().length() != 1 {
            return Err(JsValue::from_str("Todo list host shape mismatch"));
        }
        let root = host
            .first_element_child()
            .ok_or_else(|| JsValue::from_str("Todo list root missing"))?;
        if !root.tag_name().eq_ignore_ascii_case("ul")
            || root.get_attribute("class").as_deref() != Some("todo-list")
            || root.children().length() as usize != items.len()
        {
            return Err(JsValue::from_str("Todo list root preflight mismatch"));
        }
        let mut entries = Vec::with_capacity(items.len());
        for (index, item) in items.iter().enumerate() {
            let element = root
                .children()
                .item(index as u32)
                .ok_or_else(|| JsValue::from_str("Todo keyed row missing"))?;
            entries.push((todo_key(item)?, preflight_todo_row(element, item)?));
        }
        KeyedChildren::from_existing(root, entries)
    }

    fn update_todo_row(
        key: &BindingKey,
        element: &Element,
        row: &mut TodoRow,
        item: &Todo,
    ) -> Result<(), JsValue> {
        let expected_key = todo_key(item)?;
        if &expected_key != key || element.get_attribute("data-id").as_deref() != Some(key.as_str())
        {
            return Err(JsValue::from_str("Todo keyed row identity changed"));
        }
        let completed = if item.completed { "true" } else { "false" };
        if element.get_attribute("data-completed").as_deref() != Some(completed) {
            element.set_attribute("data-completed", completed)?;
        }
        let class_name = todo_class(item);
        if element.get_attribute("class").as_deref() != Some(class_name) {
            element.set_attribute("class", class_name)?;
        }
        if row.text.node_value().as_deref() != Some(item.title.as_str()) {
            row.text.set_node_value(Some(&item.title));
        }
        Ok(())
    }

    fn create_todo_row(
        document: &Document,
        key: &BindingKey,
        item: &Todo,
    ) -> Result<KeyedEntry<TodoRow>, JsValue> {
        if todo_key(item)? != *key {
            return Err(JsValue::from_str("Todo keyed create identity mismatch"));
        }
        let element = document.create_element("li")?;
        element.set_attribute("data-id", key.as_str())?;
        element.set_attribute(
            "data-completed",
            if item.completed { "true" } else { "false" },
        )?;
        element.set_attribute("class", todo_class(item))?;
        let text: Node = document.create_text_node(&item.title).into();
        element.append_child(&text)?;
        Ok(KeyedEntry::new(element, TodoRow { text }))
    }

    fn clear_runtime() {
        RUNTIME.with(|runtime| {
            runtime.borrow_mut().take();
        });
    }

    /// Mount the full client route, with a local graph executor and HTTP executor.
    #[wasm_bindgen]
    pub fn mount() -> Result<(), JsValue> {
        clear_runtime();
        let lifetime = RuntimeLifetime::new();

        let document = document()?;
        let document_element = document
            .document_element()
            .ok_or_else(|| JsValue::from_str("document element missing"))?;
        document_element.remove_attribute("data-yetlet-disposed")?;
        document_element.remove_attribute("data-yetlet-ready")?;

        let host = get_element(&document, "app")?;
        let initial = TodoSnapshot::initial();
        mount_view(&host, &browser_page(&initial.items))?;

        let list_host = get_element(&document, "todo-list-host")?;
        let list = Rc::new(RefCell::new(bind_todo_list(&list_host, &initial.items)?));
        list_host.set_attribute("data-yetlet-local-runtime", "keyed-graph")?;
        let committed = Rc::new(RefCell::new(initial.clone()));
        let graph = LocalGraph::new(
            initial,
            |state, command| apply_todo_command(state, command).map(|_| ()),
            {
                let list = Rc::clone(&list);
                let committed = Rc::clone(&committed);
                move |snapshot: &TodoSnapshot| {
                    let result = list.borrow_mut().reconcile(
                        &snapshot.items,
                        todo_key,
                        update_todo_row,
                        create_todo_row,
                    );
                    if let Err(error) = result {
                        web_sys::console::error_1(&error);
                        return Err(ActionError::Internal);
                    }
                    *committed.borrow_mut() = snapshot.clone();
                    Ok(())
                }
            },
        );
        let local_executor = Rc::new(LocalGraphExecutor {
            handle: graph.handle(),
            committed,
        });

        let status = get_element(&document, "todo-status")?;
        let local_button = get_element(&document, "toggle-first")?;
        let local_binding = DomEventListener::new(local_button.as_ref(), "click", {
            let executor = Rc::clone(&local_executor);
            let lifetime = lifetime.clone();
            move |_event| {
                let executor = Rc::clone(&executor);
                let lifetime = lifetime.clone();
                let status = status.clone();
                spawn_local(async move {
                    if !lifetime.is_active() {
                        return;
                    }
                    let result = executor.execute(TodoCommand::Toggle { id: 1 }).await;
                    if !lifetime.is_active() {
                        return;
                    }
                    match result {
                        Ok(snapshot) => {
                            status.set_text_content(Some(if snapshot.items[0].completed {
                                "First task: completed"
                            } else {
                                "First task: active"
                            }));
                        }
                        Err(error) => {
                            status
                                .set_text_content(Some(&format!("Local error: {}", error.code())));
                        }
                    }
                });
            }
        })?;

        // Remote events never move across the network. A local click creates a
        // TodoCommand and opts into an HTTP executor using ordinary fetch/JSON.
        let remote_input: HtmlInputElement = get_element(&document, "remote-title")?.dyn_into()?;
        let remote_button = get_element(&document, "create-remote")?;
        let remote_host = get_element(&document, "remote-list-host")?;
        let remote_status = get_element(&document, "remote-status")?;
        let http = Rc::new(
            HttpExecutor::new("/api/todos").map_err(|error| JsValue::from_str(error.code()))?,
        );
        let remote_binding = DomEventListener::new(remote_button.as_ref(), "click", {
            let lifetime = lifetime.clone();
            move |_event| {
                let title = remote_input.value();
                let executor = Rc::clone(&http);
                let host = remote_host.clone();
                let lifetime = lifetime.clone();
                let status = remote_status.clone();
                spawn_local(async move {
                    if !lifetime.is_active() {
                        return;
                    }
                    status.set_text_content(Some("Saving to server..."));
                    let result = executor.execute(TodoCommand::Create { title }).await;
                    if !lifetime.is_active() {
                        return;
                    }
                    match result {
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
                            status
                                .set_text_content(Some(&format!("Remote error: {}", error.code())));
                        }
                    }
                });
            }
        })?;

        RUNTIME.with(|runtime| {
            *runtime.borrow_mut() = Some(BrowserRuntime {
                lifetime,
                _graph: graph,
                _bindings: vec![local_binding, remote_binding],
            });
        });
        document_element.set_attribute("data-yetlet-ready", "true")?;
        Ok(())
    }

    /// Explicitly release graph, keyed entry state and all browser listeners.
    /// The current DOM remains readable until a later mount or navigation.
    #[wasm_bindgen]
    pub fn dispose() -> Result<(), JsValue> {
        clear_runtime();
        let document = document()?;
        let root = document
            .document_element()
            .ok_or_else(|| JsValue::from_str("document element missing"))?;
        root.remove_attribute("data-yetlet-ready")?;
        root.set_attribute("data-yetlet-disposed", "true")?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
pub use runtime::{dispose, mount};
