//! Browser-only Rust/WASM entrypoint for the Yetlet Todo example.
//!
//! DOM events are local. Explicit typed executors choose the owned local graph
//! or semantic JSON HTTP commands. No raw DOM event, persistent circuit, or
//! server repository is included in this crate.

#[cfg(any(target_arch = "wasm32", test))]
mod remote;

#[cfg(test)]
mod tests {
    #[test]
    fn complete_browser_page_obeys_dom_policy() {
        let page = yetlet_todo_ui::browser_page(&yetlet_todo_ui::TodoSnapshot::initial().items);
        assert_eq!(yetlet_browser::dom::validate(&page), Ok(()));
    }
}

#[cfg(target_arch = "wasm32")]
mod runtime {
    use super::remote::{Phase, RemoteForm, decode_snapshot};
    use std::{
        cell::{Cell, RefCell},
        rc::Rc,
    };

    use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
    use wasm_bindgen_futures::{JsFuture, spawn_local};
    use web_sys::{Document, Element, HtmlInputElement, Node};
    use yetlet::actions::{ActionError, ActionExecutor};
    use yetlet::reactive::{GraphHandle, LocalGraph, ReactiveError};
    use yetlet::view::binding::BindingKey;
    use yetlet_browser::actions::HttpExecutor;
    use yetlet_browser::dom::mount as mount_view;
    use yetlet_browser::dom_keyed::{DomEventListener, KeyedChildren, KeyedEntry};
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
    }

    impl ActionExecutor<TodoCommand> for LocalGraphExecutor {
        async fn execute(&self, command: TodoCommand) -> Result<TodoSnapshot, ActionError> {
            self.handle.dispatch(command).map_err(graph_action_error)?;
            self.handle.snapshot().map_err(graph_action_error)
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

    #[derive(Clone)]
    struct RemoteControls {
        input: HtmlInputElement,
        create: Element,
        review: Element,
        resume: Element,
        status: Element,
        list: Element,
        error: Element,
        section: Element,
    }
    impl RemoteControls {
        fn render(&self, form: &RemoteForm) -> Result<(), JsValue> {
            if form.can_create(&self.input.value()) {
                self.create.remove_attribute("disabled")?;
            } else {
                self.create.set_attribute("disabled", "")?;
            }
            for (button, visible) in [
                (
                    &self.review,
                    matches!(
                        form.phase,
                        Phase::Unconfirmed | Phase::Reviewing | Phase::Reviewed
                    ),
                ),
                (&self.resume, form.phase == Phase::Reviewed),
            ] {
                if visible {
                    button.remove_attribute("hidden")?;
                } else {
                    button.set_attribute("hidden", "")?;
                }
            }
            if form.phase == Phase::Reviewing {
                self.review.set_attribute("disabled", "")?;
            } else {
                self.review.remove_attribute("disabled")?;
            }
            self.section.set_attribute(
                "aria-busy",
                if matches!(form.phase, Phase::Saving | Phase::Reviewing) {
                    "true"
                } else {
                    "false"
                },
            )?;
            Ok(())
        }
    }

    async fn read_server() -> Result<TodoSnapshot, ActionError> {
        let response = JsFuture::from(
            web_sys::window()
                .ok_or(ActionError::Network)?
                .fetch_with_str("/api/todos"),
        )
        .await
        .map_err(|_| ActionError::Network)?
        .dyn_into::<web_sys::Response>()
        .map_err(|_| ActionError::InvalidResponse)?;
        if !response.ok() {
            return Err(ActionError::InvalidResponse);
        }
        let text = JsFuture::from(response.text().map_err(|_| ActionError::Network)?)
            .await
            .map_err(|_| ActionError::Network)?
            .as_string()
            .ok_or(ActionError::InvalidResponse)?;
        decode_snapshot(&text)
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
        let graph = LocalGraph::new(
            initial,
            |state, command| apply_todo_command(state, command).map(|_| ()),
            {
                let list = Rc::clone(&list);
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
                    Ok(())
                }
            },
        );
        let local_executor = Rc::new(LocalGraphExecutor {
            handle: graph.handle(),
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

        // Claim one request synchronously, before its future is queued.
        let remote = Rc::new(RefCell::new(RemoteForm::default()));
        let ui = RemoteControls {
            input: get_element(&document, "remote-title")?.dyn_into()?,
            create: get_element(&document, "create-remote")?,
            review: get_element(&document, "remote-review")?,
            resume: get_element(&document, "remote-resume")?,
            list: get_element(&document, "remote-list-host")?,
            status: get_element(&document, "remote-status")?,
            error: get_element(&document, "remote-title-error")?,
            section: get_element(&document, "remote-actions")?,
        };
        ui.render(&remote.borrow())?;
        let clear_title_binding = DomEventListener::new(ui.input.as_ref(), "input", {
            let ui = ui.clone();
            let remote = remote.clone();
            move |_| {
                if ui.input.has_attribute("aria-invalid") {
                    let _ = title_feedback(&ui.input, &ui.error, None);
                }
                if remote.borrow().phase == Phase::Ready {
                    ui.status.set_text_content(Some(
                        if remote.borrow().can_create(&ui.input.value()) {
                            "Title edited. Create on server when ready."
                        } else {
                            "This title was just created. Edit it to create another task."
                        },
                    ));
                }
                let _ = ui.render(&remote.borrow());
            }
        })?;
        let http = Rc::new(
            HttpExecutor::new("/api/todos").map_err(|error| JsValue::from_str(error.code()))?,
        );
        let remote_binding = DomEventListener::new(ui.create.as_ref(), "click", {
            let ui = ui.clone();
            let remote = remote.clone();
            let lifetime = lifetime.clone();
            move |_| {
                if !lifetime.is_active() {
                    return;
                }
                let title = ui.input.value();
                if !remote.borrow_mut().begin(&title) {
                    return;
                }
                let _ = title_feedback(&ui.input, &ui.error, None);
                ui.status.set_text_content(Some("Saving to server..."));
                let _ = ui.render(&remote.borrow());
                let executor = http.clone();
                let remote = remote.clone();
                let ui = ui.clone();
                let lifetime = lifetime.clone();
                spawn_local(async move {
                    if !lifetime.is_active() {
                        return;
                    }
                    let result = executor
                        .execute(TodoCommand::Create {
                            title: title.clone(),
                        })
                        .await;
                    if !lifetime.is_active() {
                        return;
                    }
                    match result {
                        Ok(snapshot) => {
                            remote.borrow_mut().saved(title.clone());
                            if mount_view(&ui.list, &todo_list(&snapshot.items)).is_err() {
                                ui.status.set_text_content(Some("The task was saved, but the list could not be displayed. Keep your title and review the server list."));
                                remote.borrow_mut().unconfirmed();
                            } else {
                                ui.status.set_text_content(Some(&format!(
                                    "Saved on server (revision {}). {}",
                                    snapshot.revision,
                                    if ui.input.value() == title {
                                        "Edit the title to create another task."
                                    } else {
                                        "Your edited title is ready for another create."
                                    }
                                )));
                            }
                        }
                        Err(ActionError::Remote { status: 422, code })
                            if code == "invalid_title" =>
                        {
                            remote.borrow_mut().rejected();
                            if ui.input.value() == title {
                                let _ = title_feedback(
                                    &ui.input,
                                    &ui.error,
                                    Some("Enter 1 to 100 characters without control characters."),
                                );
                                ui.status.set_text_content(Some(
                                    "Check the task title. Your text is still here.",
                                ));
                            } else {
                                ui.status.set_text_content(Some("The submitted title was rejected. Review your edited title before trying again."));
                            }
                        }
                        Err(ActionError::Remote {
                            status: 400..=499, ..
                        }) => {
                            remote.borrow_mut().rejected();
                            ui.status.set_text_content(Some("Request rejected. Your text is still here. Review the title or server limits before trying again."));
                        }
                        Err(_) => {
                            remote.borrow_mut().unconfirmed();
                            ui.status.set_text_content(Some("Create could not be confirmed; it may have reached the server. Your text is still here. Review the server list before starting another create."));
                        }
                    }
                    let _ = ui.render(&remote.borrow());
                });
            }
        })?;
        let review_binding = DomEventListener::new(ui.review.as_ref(), "click", {
            let ui = ui.clone();
            let remote = remote.clone();
            let lifetime = lifetime.clone();
            move |_| {
                if !lifetime.is_active() || !remote.borrow_mut().begin_review() {
                    return;
                }
                ui.status
                    .set_text_content(Some("Reading the server list... Your title stays here."));
                let _ = ui.render(&remote.borrow());
                let ui = ui.clone();
                let remote = remote.clone();
                let lifetime = lifetime.clone();
                spawn_local(async move {
                    if !lifetime.is_active() {
                        return;
                    }
                    let result = read_server().await;
                    if !lifetime.is_active() {
                        return;
                    }
                    let displayed = result.ok().filter(|snapshot| {
                        mount_view(&ui.list, &todo_list(&snapshot.items)).is_ok()
                    });
                    remote.borrow_mut().reviewed(displayed.is_some());
                    ui.status.set_text_content(Some(if displayed.is_some() {
                        "Server list loaded. Compare it with your title. Choose Start another create only when you intend an additional task."
                    } else { "Could not load the server list. Your title is still here. Review the server list again when available." }));
                    let _ = ui.render(&remote.borrow());
                });
            }
        })?;
        let resume_binding = DomEventListener::new(ui.resume.as_ref(), "click", {
            let ui = ui.clone();
            let remote = remote.clone();
            let lifetime = lifetime.clone();
            move |_| {
                if lifetime.is_active() && remote.borrow_mut().resume() {
                    ui.status.set_text_content(Some(
                        "Ready for an explicit new create. Review the title before submitting.",
                    ));
                    let _ = ui.render(&remote.borrow());
                }
            }
        })?;

        RUNTIME.with(|runtime| {
            *runtime.borrow_mut() = Some(BrowserRuntime {
                lifetime,
                _graph: graph,
                _bindings: vec![
                    local_binding,
                    remote_binding,
                    clear_title_binding,
                    review_binding,
                    resume_binding,
                ],
            });
        });
        document_element.set_attribute("data-yetlet-ready", "true")?;
        Ok(())
    }

    fn title_feedback(
        input: &HtmlInputElement,
        error: &Element,
        message: Option<&str>,
    ) -> Result<(), JsValue> {
        error.set_text_content(message);
        if message.is_some() {
            input.set_attribute("aria-invalid", "true")?;
            input.set_attribute("aria-describedby", "remote-title-error")?;
            error.remove_attribute("hidden")?;
        } else {
            input.remove_attribute("aria-invalid")?;
            input.remove_attribute("aria-describedby")?;
            error.set_attribute("hidden", "")?;
        }
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
