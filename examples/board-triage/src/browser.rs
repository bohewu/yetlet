use super::{Bootstrap, Draft, Item, Msg, Save, Workspace, status_counts};
use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};
use wasm_bindgen::{JsCast, JsValue, prelude::wasm_bindgen};
use wasm_bindgen_futures::spawn_local;
use web_sys::{
    Document, Element, HtmlButtonElement, HtmlInputElement, HtmlSelectElement, HtmlTextAreaElement,
};
use yetlet_actions::{ActionError, ActionExecutor, HttpExecutor};
use yetlet_core::binding::BindingKey;
use yetlet_dom_keyed::{DomEventListener, KeyedChildren, KeyedEntry};
use yetlet_reactive::{GraphHandle, LocalGraph};

type Handle = GraphHandle<Workspace, Msg, &'static str>;

#[wasm_bindgen]
extern "C" {
    type StatusChart;
    #[wasm_bindgen(method, catch, js_name = update)]
    fn update_chart(this: &StatusChart, counts: &[u32]) -> Result<(), JsValue>;
    #[wasm_bindgen(method, catch, js_name = destroy)]
    fn destroy_chart(this: &StatusChart) -> Result<(), JsValue>;
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
            set_control(
                &element(&self.document, &format!("triage-{name}"))?,
                text,
                pending,
            )?;
        }
        for (id, disabled) in [
            ("triage-save", state.pending.is_some() || !draft.dirty()),
            ("triage-reset", pending || !draft.dirty()),
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
fn failure(error: ActionError) -> String {
    match error {
        ActionError::Remote { status: 409, .. } => "Conflict: your draft is still here. Open the latest record before retrying; this workspace has not overwritten it.".into(),
        ActionError::Remote { status: 401 | 403, .. } => "Permission or session changed. Your draft was not saved; sign in again using another tab.".into(),
        ActionError::Remote { status: 422, .. } => "Draft rejected. Check title, field lengths and status transition. Your draft is still here.".into(),
        _ => "Save could not be confirmed. Keep your draft and check the latest record before retrying.".into(),
    }
}
#[wasm_bindgen]
pub fn mount_triage(chart: JsValue) -> Result<(), JsValue> {
    dispose_triage();
    let chart =
        (!chart.is_null() && !chart.is_undefined()).then(|| ChartOwner(chart.unchecked_into()));
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
    let executor = HttpExecutor::new(&format!("/board/projects/{project}/triage"))
        .map_err(|_| js("Invalid action endpoint"))?;
    let save_handle = handle.clone();
    let lifetime = active.clone();
    bindings.push(DomEventListener::new(element(&document, "triage-save")?.as_ref(), "click", move |_| {
        let Ok(state) = save_handle.snapshot() else { return; };
        if state.pending.is_some() { return; }
        let Some(draft) = state.selected().filter(|draft| draft.dirty()) else { return; };
        let command = Save { csrf: csrf.clone(), item: draft.value.clone() };
        if save_handle.dispatch(Msg::Begin).is_err() { return; }
        let executor = executor.clone(); let handle = save_handle.clone(); let active = lifetime.clone();
        spawn_local(async move {
            // The owner may be disposed before this queued future first runs.
            if !active.get() {
                return;
            }
            let result: Result<Item, _> = executor.execute(command).await;
            if !active.get() { return; }
            match result {
                Ok(item) => { if handle.dispatch(Msg::Saved(item)).is_err() { dispatch(&handle, Msg::Failed("Unexpected save response. Keep your draft and check the latest record.".into())); } }
                Err(error) => dispatch(&handle, Msg::Failed(failure(error))),
            }
        });
    })?);
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
