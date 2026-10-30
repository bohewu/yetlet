use super::{
    Bootstrap, Draft, FieldErrors, Item, Msg, Save, SaveFeedback, Workspace, status_counts,
    validation_fields,
};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::{JsFuture, spawn_local};
use web_sys::{
    Document, Element, HtmlButtonElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement,
};
use yetlet::actions::{ActionError, ApiErrorEnvelope};
use yetlet::reactive::{GraphHandle, LocalGraph};
use yetlet::view::binding::BindingKey;
use yetlet_browser::actions::HttpExecutor;
use yetlet_browser::dom_keyed::{DomEventListener, KeyedChildren, KeyedEntry};

type Handle = GraphHandle<Workspace, Msg, &'static str>;

#[wasm_bindgen]
extern "C" {
    type StatusChart;
    #[wasm_bindgen(method, catch, js_name = update)]
    fn update_chart(this: &StatusChart, counts: &[u32]) -> Result<(), JsValue>;
    #[wasm_bindgen(method, catch, js_name = destroy)]
    fn destroy_chart(this: &StatusChart) -> Result<(), JsValue>;
    type DescriptionEditor;
    #[wasm_bindgen(method, catch, js_name = connect)]
    fn connect_editor(this: &DescriptionEditor, callback: &JsValue) -> Result<(), JsValue>;
    #[wasm_bindgen(method, catch, js_name = update)]
    fn update_editor(
        this: &DescriptionEditor,
        id: f64,
        text: &str,
        read_only: bool,
        reset: u32,
        invalid: bool,
    ) -> Result<(), JsValue>;
    #[wasm_bindgen(method, catch, js_name = destroy)]
    fn destroy_editor(this: &DescriptionEditor) -> Result<(), JsValue>;
}

type EditCallback = wasm_bindgen::closure::Closure<dyn FnMut(f64, String)>;
struct DescriptionOwner {
    handle: DescriptionEditor,
    callback: RefCell<Option<EditCallback>>,
}
impl Drop for DescriptionOwner {
    fn drop(&mut self) {
        // JS invalidates its entry point before the callback field is released.
        let _ = self.handle.destroy_editor();
    }
}

struct ChartOwner(StatusChart);
impl Drop for ChartOwner {
    fn drop(&mut self) {
        let _ = self.0.destroy_chart();
    }
}
struct Runtime {
    active: Rc<Cell<bool>>,
    _graph: LocalGraph<Workspace, Msg, &'static str>,
    _bindings: Vec<DomEventListener>,
}
impl Drop for Runtime {
    fn drop(&mut self) {
        self.active.set(false);
    }
}
thread_local! { static RUNTIME: RefCell<Option<Runtime>> = const { RefCell::new(None) }; }
fn js(message: &str) -> JsValue {
    JsValue::from_str(message)
}
fn element(document: &Document, id: &str) -> Result<Element, JsValue> {
    document
        .get_element_by_id(id)
        .ok_or_else(|| js("Missing triage element"))
}
fn key(draft: &Draft) -> Result<BindingKey, JsValue> {
    BindingKey::new(draft.value.id.to_string()).map_err(|_| js("Invalid triage key"))
}
fn row_update(element: &Element, draft: &Draft, selected: Option<i64>) -> Result<(), JsValue> {
    element.set_attribute(
        "data-selected",
        if selected == Some(draft.value.id) {
            "true"
        } else {
            "false"
        },
    )?;
    let title = element
        .query_selector("[data-title]")?
        .ok_or_else(|| js("Missing row title"))?;
    if title.text_content().as_deref() != Some(&draft.value.title) {
        title.set_text_content(Some(&draft.value.title));
    }
    element
        .query_selector("[data-meta]")?
        .ok_or_else(|| js("Missing row metadata"))?
        .set_text_content(Some(&format!(
            "{} · {}{}",
            draft.value.status,
            draft.value.priority,
            if draft.dirty() { " · Unsaved" } else { "" }
        )));
    let select = element
        .query_selector("button")?
        .ok_or_else(|| js("Missing row selection"))?;
    select.remove_attribute("hidden")?;
    select.set_attribute(
        "aria-pressed",
        if selected == Some(draft.value.id) {
            "true"
        } else {
            "false"
        },
    )?;
    select.set_attribute(
        "aria-label",
        &format!("Select item {}: {}", draft.value.id, draft.value.title),
    )?;
    Ok(())
}
fn row_create(document: &Document, project: i64, draft: &Draft) -> Result<KeyedEntry<()>, JsValue> {
    let row = document.create_element("li")?;
    row.set_attribute("data-id", &draft.value.id.to_string())?;
    let select = document.create_element("button")?;
    select.set_attribute("type", "button")?;
    select.set_attribute("class", "btn btn-outline-secondary")?;
    select.set_attribute("data-select", &draft.value.id.to_string())?;
    select.set_text_content(Some("Select"));
    row.append_child(&select)?;
    let link = document.create_element("a")?;
    link.set_attribute("data-title", "")?;
    link.set_attribute(
        "href",
        &format!("/board/projects/{project}/items/{}", draft.value.id),
    )?;
    row.append_child(&link)?;
    let meta = document.create_element("span")?;
    meta.set_attribute("data-meta", "")?;
    row.append_child(&meta)?;
    Ok(KeyedEntry::new(row, ()))
}
fn value(element: &Element) -> Result<String, JsValue> {
    if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
        Ok(input.value())
    } else if let Some(input) = element.dyn_ref::<HtmlTextAreaElement>() {
        Ok(input.value())
    } else if let Some(input) = element.dyn_ref::<HtmlSelectElement>() {
        Ok(input.value())
    } else {
        Err(js("Not an input"))
    }
}
fn set_control(element: &Element, next: &str, disabled: bool) -> Result<(), JsValue> {
    // Property updates only when changed: the focused native control retains its
    // selection and IME composition during unrelated graph commits.
    if let Some(input) = element.dyn_ref::<HtmlInputElement>() {
        if input.value() != next {
            input.set_value(next);
        }
        input.set_disabled(disabled);
    } else if let Some(input) = element.dyn_ref::<HtmlTextAreaElement>() {
        if input.value() != next {
            input.set_value(next);
        }
        input.set_disabled(disabled);
    } else if let Some(input) = element.dyn_ref::<HtmlSelectElement>() {
        if input.value() != next {
            input.set_value(next);
        }
        input.set_disabled(disabled);
    } else {
        return Err(js("Not an input"));
    }
    Ok(())
}
struct Dom {
    document: Document,
    rows: KeyedChildren<()>,
    project: i64,
    chart: Option<ChartOwner>,
    description: Option<Rc<DescriptionOwner>>,
}
impl Dom {
    fn render(&mut self, state: &Workspace) -> Result<(), JsValue> {
        let counts = status_counts(state.drafts.iter().map(|draft| &draft.value));
        for (index, count) in counts.iter().enumerate() {
            element(&self.document, &format!("triage-status-{index}"))?
                .set_text_content(Some(&count.to_string()));
        }
        if let Some(chart) = &self.chart
            && chart.0.update_chart(&counts).is_err()
        {
            // A foreign widget failure must not fault the native draft runtime.
            self.chart.take();
        }
        let visible = state.visible();
        let selected = state.selected;
        self.rows.reconcile(
            &visible,
            key,
            |_, element, _, draft| row_update(element, draft, selected),
            |_, _, draft| {
                let entry = row_create(&self.document, self.project, draft)?;
                Ok(entry)
            },
        )?;
        // reconcile updates retained rows; initialize newly created ones too.
        let list = element(&self.document, "triage-list")?;
        for index in 0..list.children().length() {
            let row = list
                .children()
                .item(index)
                .ok_or_else(|| js("Missing row"))?;
            row_update(&row, &visible[index as usize], selected)?;
        }
        let empty = element(&self.document, "triage-empty")?;
        if visible.is_empty() {
            empty.remove_attribute("hidden")?;
        } else {
            empty.set_attribute("hidden", "")?;
        }
        element(&self.document, "triage-count")?.set_text_content(Some(&format!(
            "{} shown · {} unsaved",
            visible.len(),
            state.dirty_count()
        )));
        element(&self.document, "triage-message")?.set_text_content(Some(&state.message));
        let editor = element(&self.document, "triage-editor")?;
        let Some(draft) = state.selected() else {
            editor.set_attribute("hidden", "")?;
            return Ok(());
        };
        editor.remove_attribute("hidden")?;
        let pending = state.pending == state.selected;
        editor.set_attribute("aria-busy", if pending { "true" } else { "false" })?;
        element(&self.document, "triage-save-state")?
            .set_text_content(Some(draft.save_status(pending)));
        let recovery = element(&self.document, "triage-recovery")?;
        if draft.feedback.needs_fresh_page() {
            recovery.remove_attribute("hidden")?;
            element(&self.document, "triage-recovery-message")?
                .set_text_content(Some(draft.feedback.guidance()));
            element(&self.document, "triage-draft-copy")?
                .dyn_into::<HtmlTextAreaElement>()?
                .set_value(
                    &serde_json::to_string_pretty(&draft.value)
                        .map_err(|_| js("Draft unavailable"))?,
                );
        } else {
            recovery.set_attribute("hidden", "")?;
        }
        let sign_in = element(&self.document, "triage-sign-in")?;
        if draft.feedback == SaveFeedback::Access {
            sign_in.remove_attribute("hidden")?;
        } else {
            sign_in.set_attribute("hidden", "")?;
        }
        element(&self.document, "triage-heading")?.set_text_content(Some(&format!(
            "Item #{} · version {}{}",
            draft.value.id,
            draft.saved.version,
            if draft.dirty() { " · Unsaved" } else { "" }
        )));
        for (name, text) in [
            ("title", &draft.value.title),
            ("description", &draft.value.description),
            ("status", &draft.value.status),
            ("priority", &draft.value.priority),
            ("assignee", &draft.value.assignee),
        ] {
            let control = element(&self.document, &format!("triage-{name}"))?;
            set_control(&control, text, pending)?;
            let error = element(&self.document, &format!("triage-{name}-error"))?;
            let message = draft.errors.get(name);
            error.set_text_content(message.map(String::as_str));
            let hint = if name == "description" {
                "triage-description-hint"
            } else {
                ""
            };
            if message.is_some() {
                control.set_attribute("aria-invalid", "true")?;
                control.set_attribute(
                    "aria-describedby",
                    format!("{hint} triage-{name}-error").trim(),
                )?;
                error.remove_attribute("hidden")?;
            } else {
                control.remove_attribute("aria-invalid")?;
                if hint.is_empty() {
                    control.remove_attribute("aria-describedby")?;
                } else {
                    control.set_attribute("aria-describedby", hint)?;
                }
                error.set_attribute("hidden", "")?;
            }
        }
        if let Some(description) = &self.description
            && description
                .handle
                .update_editor(
                    draft.value.id as f64,
                    &draft.value.description,
                    pending,
                    state.description_reset,
                    draft.errors.contains_key("description"),
                )
                .is_err()
        {
            // The native control already has the current draft if enhancement fails.
            self.description.take();
        }
        for (id, disabled) in [
            ("triage-save", state.pending.is_some() || !draft.can_save()),
            (
                "triage-reset",
                pending || !draft.dirty() || draft.feedback.needs_fresh_page(),
            ),
        ] {
            element(&self.document, id)?
                .dyn_into::<HtmlButtonElement>()?
                .set_disabled(disabled);
        }
        element(&self.document, "triage-latest")?.set_attribute(
            "href",
            &format!("/board/projects/{}/items/{}", self.project, draft.value.id),
        )?;
        Ok(())
    }
}
fn dispatch(handle: &Handle, message: Msg) {
    let _ = handle.dispatch(message);
}
enum SaveFailure {
    Global(ActionError),
    Validation(FieldErrors),
}
impl From<ActionError> for SaveFailure {
    fn from(value: ActionError) -> Self {
        Self::Global(value)
    }
}
// One app-owned rich-error path; the common executor's status/code API stays unchanged.
async fn save(executor: &HttpExecutor, command: Save) -> Result<Item, SaveFailure> {
    use web_sys::{Request, RequestInit, Response};
    let body = serde_json::to_string(&command).map_err(|_| ActionError::InvalidResponse)?;
    let init = RequestInit::new();
    init.set_method("POST");
    init.set_body(&JsValue::from_str(&body));
    let request = Request::new_with_str_and_init(executor.endpoint(), &init)
        .map_err(|_| ActionError::InvalidEndpoint)?;
    for (name, value) in [
        ("Content-Type", "application/json"),
        ("Accept", "application/json"),
    ] {
        request
            .headers()
            .set(name, value)
            .map_err(|_| ActionError::Network)?;
    }
    let window = web_sys::window().ok_or(ActionError::Network)?;
    let response: Response = JsFuture::from(window.fetch_with_request(&request))
        .await
        .map_err(|_| ActionError::Network)?
        .dyn_into()
        .map_err(|_| ActionError::InvalidResponse)?;
    let status = response.status();
    let text = JsFuture::from(response.text().map_err(|_| ActionError::Network)?)
        .await
        .map_err(|_| ActionError::Network)?
        .as_string()
        .ok_or(ActionError::InvalidResponse)?;
    if !(200..300).contains(&status) {
        if let Some(fields) = validation_fields(status, &text) {
            return Err(SaveFailure::Validation(fields));
        }
        let code = serde_json::from_str::<ApiErrorEnvelope>(&text)
            .map(|reply| reply.error.code)
            .unwrap_or_else(|_| "remote_error".into());
        return Err(ActionError::Remote { status, code }.into());
    }
    serde_json::from_str(&text).map_err(|_| ActionError::InvalidResponse.into())
}
#[wasm_bindgen]
pub fn mount_triage(chart: JsValue, description: JsValue) -> Result<(), JsValue> {
    dispose_triage();
    let chart =
        (!chart.is_null() && !chart.is_undefined()).then(|| ChartOwner(chart.unchecked_into()));
    let description = (!description.is_null() && !description.is_undefined()).then(|| {
        Rc::new(DescriptionOwner {
            handle: description.unchecked_into(),
            callback: RefCell::new(None),
        })
    });
    let document = web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(|| js("Document unavailable"))?;
    let bootstrap = value(&element(&document, "triage-bootstrap")?)?;
    if bootstrap.len() > 256 * 1024 {
        return Err(js("Triage snapshot too large"));
    }
    let snapshot: Bootstrap =
        serde_json::from_str(&bootstrap).map_err(|_| js("Invalid triage snapshot"))?;
    let project = snapshot.project;
    let state = Workspace::new(snapshot).map_err(js)?;
    let csrf = value(&element(&document, "triage-csrf")?)?;
    let root = element(&document, "triage-list")?;
    if root.children().length() as usize != state.drafts.len() {
        return Err(js("Triage SSR row mismatch"));
    }
    let mut entries = Vec::new();
    for (index, draft) in state.drafts.iter().enumerate() {
        let row = root
            .children()
            .item(index as u32)
            .ok_or_else(|| js("Missing SSR row"))?;
        if row.tag_name() != "LI"
            || row.get_attribute("data-id").as_deref() != Some(&draft.value.id.to_string())
        {
            return Err(js("Triage SSR key mismatch"));
        }
        entries.push((key(draft)?, KeyedEntry::new(row, ())));
    }
    let mut dom = Dom {
        document: document.clone(),
        rows: KeyedChildren::from_existing(root.clone(), entries)?,
        project,
        chart,
        description: description.clone(),
    };
    let graph = LocalGraph::new(
        state,
        |state: &mut Workspace, message| {
            *state = state.reduce(message)?;
            Ok(())
        },
        move |state: &Workspace| {
            dom.render(state).map_err(|_| "DOM update failed")?;
            Ok(())
        },
    );
    let handle = graph.handle();
    let active = Rc::new(Cell::new(true));
    if let Some(description) = &description {
        let handle = handle.clone();
        let active = active.clone();
        let callback = EditCallback::new(move |id: f64, text: String| {
            if active.get()
                && id.is_finite()
                && id.fract() == 0.0
                && id > 0.0
                && id <= 9_007_199_254_740_991.0
            {
                dispatch(
                    &handle,
                    Msg::EditDescription {
                        id: id as i64,
                        value: text,
                    },
                );
            }
        });
        description.handle.connect_editor(callback.as_ref())?;
        *description.callback.borrow_mut() = Some(callback);
    }
    let mut bindings = Vec::new();
    let select_handle = handle.clone();
    bindings.push(DomEventListener::new(
        root.as_ref(),
        "click",
        move |event| {
            let Some(target) = event.target().and_then(|t| t.dyn_into::<Element>().ok()) else {
                return;
            };
            if let Ok(Some(button)) = target.closest("button[data-select]")
                && let Some(id) = button
                    .get_attribute("data-select")
                    .and_then(|s| s.parse().ok())
            {
                dispatch(&select_handle, Msg::Select(id));
            }
        },
    )?);
    for name in [
        "title",
        "description",
        "status",
        "priority",
        "assignee",
        "filter",
    ] {
        let control = element(&document, &format!("triage-{name}"))?;
        let target = control.clone();
        let handle = handle.clone();
        bindings.push(DomEventListener::new(
            control.as_ref(),
            if matches!(name, "status" | "priority" | "assignee") {
                "change"
            } else {
                "input"
            },
            move |_| {
                if let Ok(value) = value(&target) {
                    dispatch(
                        &handle,
                        if name == "filter" {
                            Msg::Filter(value)
                        } else {
                            Msg::Edit { field: name, value }
                        },
                    );
                }
            },
        )?);
    }
    for (id, msg) in [
        ("triage-reverse", Msg::Reverse),
        ("triage-reset", Msg::Reset),
    ] {
        let handle = handle.clone();
        bindings.push(DomEventListener::new(
            element(&document, id)?.as_ref(),
            "click",
            move |_| dispatch(&handle, msg.clone()),
        )?);
    }
    bindings.push(DomEventListener::new(
        element(&document, "triage-select-draft")?.as_ref(),
        "click",
        {
            let document = document.clone();
            move |_| {
                if let Ok(element) = element(&document, "triage-draft-copy")
                    && let Ok(input) = element.dyn_into::<HtmlTextAreaElement>()
                {
                    let _ = input.focus();
                    input.select();
                }
            }
        },
    )?);
    let executor = HttpExecutor::new(&format!("/board/projects/{project}/triage"))
        .map_err(|_| js("Invalid action endpoint"))?;
    let save_handle = handle.clone();
    let lifetime = active.clone();
    bindings.push(DomEventListener::new(
        element(&document, "triage-save")?.as_ref(),
        "click",
        move |_| {
            let Ok(state) = save_handle.snapshot() else {
                return;
            };
            if state.pending.is_some() {
                return;
            }
            let Some(draft) = state.selected().filter(|draft| draft.can_save()) else {
                return;
            };
            let command = Save {
                csrf: csrf.clone(),
                item: draft.value.clone(),
            };
            let submitted_id = command.item.id;
            if save_handle.dispatch(Msg::Begin).is_err() {
                return;
            }
            let executor = executor.clone();
            let handle = save_handle.clone();
            let active = lifetime.clone();
            spawn_local(async move {
                // The owner may be disposed before this queued future first runs.
                if !active.get() {
                    return;
                }
                let result = save(&executor, command).await;
                if !active.get() {
                    return;
                }
                match result {
                    Ok(item) => {
                        if handle.dispatch(Msg::Saved(item)).is_err() {
                            dispatch(
                                &handle,
                                Msg::Failed {
                                    id: submitted_id,
                                    feedback: SaveFeedback::Unconfirmed,
                                },
                            );
                        }
                    }
                    Err(SaveFailure::Global(error)) => dispatch(
                        &handle,
                        Msg::Failed {
                            id: submitted_id,
                            feedback: SaveFeedback::from_error(&error),
                        },
                    ),
                    Err(SaveFailure::Validation(fields)) => {
                        if handle
                            .dispatch(Msg::ValidationFailed {
                                id: submitted_id,
                                fields,
                            })
                            .is_err()
                        {
                            dispatch(
                                &handle,
                                Msg::Failed {
                                    id: submitted_id,
                                    feedback: SaveFeedback::Unconfirmed,
                                },
                            );
                        }
                    }
                }
            });
        },
    )?);
    // Commit once after all bindings exist; LocalGraph::new does not render.
    handle
        .dispatch(Msg::Filter(String::new()))
        .map_err(|_| js("Initial triage render failed"))?;
    element(&document, "triage-toolbar")?.remove_attribute("hidden")?;
    element(&document, "triage-workspace")?.set_attribute("data-ready", "true")?;
    RUNTIME.with(|slot| {
        *slot.borrow_mut() = Some(Runtime {
            active,
            _graph: graph,
            _bindings: bindings,
        });
    });
    Ok(())
}
#[wasm_bindgen]
pub fn dispose_triage() {
    RUNTIME.with(|slot| {
        slot.borrow_mut().take();
    });
}
