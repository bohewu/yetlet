//! Application-owned interactive slice over the existing authorization/storage boundary.
use super::*;
use axum::{Json, extract::rejection::JsonRejection};
use domain::WorkItem;
use yetlet_actions::{ApiErrorBody, ApiErrorEnvelope};
use yetlet_board_triage::{Bootstrap, Item, MAX_ITEMS, Save};
use yetlet_core::Element;

#[derive(Clone)]
pub(super) struct WasmPage;
// WASM compilation is allowed only on this explicitly enhanced page. No inline
// scripts, unsafe-eval, remote origin, or persistent server circuit is enabled.
pub(super) const CSP: &str = "default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; connect-src 'self'; font-src 'self'; object-src 'none'; base-uri 'none'; form-action 'self'; frame-ancestors 'none'";

impl From<WorkItem> for Item {
    fn from(item: WorkItem) -> Self {
        Self {
            id: item.id,
            version: item.version,
            title: item.fields.title,
            description: item.fields.description,
            status: item.fields.status,
            priority: item.fields.priority,
            assignee: item.fields.assignee,
        }
    }
}
fn button(id: &'static str, label: &str) -> Element {
    let button = views::text("button", label)
        .attribute("type", "button")
        .attribute("class", "btn btn-outline-secondary");
    if id.is_empty() {
        button
    } else {
        button.attribute("id", id)
    }
}
fn control(label: &str, name: &'static str, kind: &'static str) -> Element {
    let id = format!("triage-{name}");
    let mut input = Element::new(kind)
        .attribute("id", &id)
        .attribute("class", "form-control");
    if kind == "input" {
        input = input
            .attribute("type", "text")
            .attribute("maxlength", if name == "title" { "160" } else { "100" });
    }
    if kind == "textarea" {
        input = input.attribute("maxlength", "4000").attribute("rows", "5");
    }
    Element::new("div")
        .attribute("class", "triage-field")
        .child(views::text("label", label).attribute("for", id))
        .child(input)
}
fn select(label: &str, name: &'static str, options: &[(&str, &str)]) -> Element {
    let id = format!("triage-{name}");
    let mut input = Element::new("select")
        .attribute("id", &id)
        .attribute("class", "form-select");
    for (value, label) in options {
        input = input.child(views::text("option", *label).attribute("value", *value));
    }
    Element::new("div")
        .attribute("class", "triage-field")
        .child(views::text("label", label).attribute("for", id))
        .child(input)
}
pub(super) async fn workspace(
    State(state): State<AppState>,
    Extension(session): Extension<Session<LoginSession>>,
    RoutePath(project): RoutePath<i64>,
) -> Result<Response, Failure> {
    let principal = user(&session, Some(project), true)?;
    let items = state
        .database
        .list(project, ListQuery::default().normalize()?)
        .await?
        .items
        .into_iter()
        .take(MAX_ITEMS)
        .map(Item::from)
        .collect::<Vec<_>>();
    let snapshot = Bootstrap {
        schema: 1,
        project,
        items,
    };
    let json = serde_json::to_string(&snapshot).map_err(|_| Failure::Unavailable)?;
    let csrf = csrf(&session)?;
    let mut rows = Element::new("ul")
        .attribute("id", "triage-list")
        .attribute("class", "triage-list");
    for item in &snapshot.items {
        rows = rows.child(
            Element::new("li")
                .attribute("data-id", item.id.to_string())
                .child(
                    button("", "Select")
                        .attribute("data-select", item.id.to_string())
                        .attribute("hidden", ""),
                )
                .child(
                    views::text("a", &item.title)
                        .attribute(
                            "href",
                            format!("/board/projects/{project}/items/{}", item.id),
                        )
                        .attribute("data-title", ""),
                )
                .child(
                    views::text("span", format!("{} · {}", item.status, item.priority))
                        .attribute("data-meta", ""),
                ),
        );
    }
    let operator = if project == 1 { "alice" } else { "bob" };
    let body = Element::new("section").attribute("id", "triage-workspace")
        .child(Element::new("div").attribute("class", "triage-intro")
            .child(views::text("p", "A focused workspace for the 12 most recently updated items. Filter, reorder and keep several drafts; save one item at a time."))
            .child(views::text("a", "View all work items →").attribute("href", format!("/board/projects/{project}/items"))))
        .child(views::hidden("triage-bootstrap", json).attribute("id", "triage-bootstrap"))
        .child(views::hidden("triage-csrf", &csrf).attribute("id", "triage-csrf"))
        .child(views::text("p", "Interactive workspace is loading. You can always open an item above to view or edit it using native pages.")
            .attribute("id", "triage-message").attribute("role", "status").attribute("aria-live", "polite").attribute("class", "triage-notice"))
        .child(Element::new("div").attribute("id", "triage-toolbar").attribute("hidden", "").attribute("class", "triage-toolbar")
            .child(control("Filter loaded items", "filter", "input"))
            .child(button("triage-reverse", "Reverse order"))
            .child(views::text("span", "").attribute("id", "triage-count")))
        .child(Element::new("div").attribute("class", "triage-columns")
            .child(Element::new("section").attribute("aria-label", "Loaded work items").attribute("class", "triage-queue")
                .child(views::text("h2", "Work queue"))
                .child(rows).child(views::text("p", "No items match this filter.").attribute("id", "triage-empty").attribute("hidden", "")))
            .child(Element::new("section").attribute("id", "triage-editor").attribute("hidden", "").attribute("class", "triage-editor").attribute("aria-label", "Selected item draft")
                .child(views::text("p", "LOCAL DRAFT").attribute("class", "triage-eyebrow"))
                .child(views::text("h2", "Selected item").attribute("id", "triage-heading"))
                .child(control("Title", "title", "input"))
                .child(control("Description", "description", "textarea"))
                .child(Element::new("div").attribute("class", "triage-field-pair")
                    .child(select("Status", "status", &[("open", "Open"), ("active", "Active"), ("blocked", "Blocked"), ("closed", "Closed")]))
                    .child(select("Priority", "priority", &[("low", "Low"), ("normal", "Normal"), ("high", "High")])))
                .child(select("Assignee", "assignee", &[("", "Unassigned"), (operator, operator)]))
                .child(Element::new("div").attribute("class", "triage-editor-actions")
                    .child(button("triage-save", "Save selected").attribute("class", "btn btn-primary"))
                    .child(button("triage-reset", "Reset draft")))
                .child(views::text("a", "Open latest record / native editor →").attribute("id", "triage-latest").attribute("href", format!("/board/projects/{project}/items")))
                .child(views::text("p", "Drafts stay in this tab. Reloading or leaving discards unsaved changes.").attribute("class", "triage-hint"))));
    let view = views::shell(Some(principal), Some(&csrf), "Triage workspace", body);
    let mut response = page(&state, "Triage workspace | Lab Operations", view, false)?;
    if let Some(script) = &state.triage_script {
        // Re-render from the same escaped view contract; the module path was
        // validated before opening storage. The bootstrap contains no markup.
        let bytes = axum::body::to_bytes(response.into_body(), 512 * 1024)
            .await
            .map_err(|_| Failure::Unavailable)?;
        let html = String::from_utf8(bytes.to_vec()).map_err(|_| Failure::Unavailable)?;
        response = axum::response::Html(html.replacen(
            "</head>",
            &format!("<script type=\"module\" src=\"{script}\"></script></head>"),
            1,
        ))
        .into_response();
        response.extensions_mut().insert(WasmPage);
    }
    Ok(response)
}
fn error(failure: Failure) -> Response {
    let code = match failure {
        Failure::Conflict => "conflict",
        Failure::Forbidden => "forbidden",
        Failure::Unauthorized => "unauthorized",
        Failure::Invalid | Failure::Transition => "validation",
        Failure::NotFound => "not_found",
        _ => "unavailable",
    };
    (
        failure.status(),
        Json(ApiErrorEnvelope {
            error: ApiErrorBody {
                code: code.into(),
                message: if failure == Failure::Transition {
                    "Check draft fields and allowed status transitions. Your draft was not saved."
                } else {
                    failure.message()
                }
                .into(),
            },
        }),
    )
        .into_response()
}
pub(super) async fn save(
    State(state): State<AppState>,
    Extension(session): Extension<Session<LoginSession>>,
    RoutePath(project): RoutePath<i64>,
    payload: Result<Json<Save>, JsonRejection>,
) -> Response {
    let result = async {
        let principal = user(&session, Some(project), true)?;
        let Json(payload) = payload.map_err(|_| Failure::Invalid)?;
        require_csrf(&session, &payload.csrf)?;
        let item = payload.item;
        if item.id <= 0 || item.version <= 0 {
            return Err(Failure::Invalid);
        }
        let fields = Fields {
            title: item.title,
            description: item.description,
            status: item.status,
            priority: item.priority,
            assignee: item.assignee,
        };
        if !fields.errors(project).is_empty() {
            return Err(Failure::Transition);
        }
        user(&session, Some(project), true)?;
        state
            .database
            .save(principal, Some(item.id), item.version, fields)
            .await
            .map(Item::from)
    }
    .await;
    match result {
        Ok(item) => Json(item).into_response(),
        Err(failure) => error(failure),
    }
}
