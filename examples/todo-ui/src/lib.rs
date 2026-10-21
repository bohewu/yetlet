//! Shared UI source independent of any HTTP server or browser API.
//!
//! Both native HTML SSR and browser DOM rendering consume these same Views.

use serde::{Deserialize, Serialize};
use yetlet_actions::{ActionError, Command};
use yetlet_core::{Element, View};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Todo {
    pub id: u64,
    pub title: String,
    pub completed: bool,
}

pub fn initial_todos() -> Vec<Todo> {
    vec![
        Todo {
            id: 1,
            title: "Share one Rust View across renderers".into(),
            completed: false,
        },
        Todo {
            id: 2,
            title: "Keep DOM events in the browser".into(),
            completed: false,
        },
    ]
}

/// Semantic API command: never a serialized DOM event.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TodoCommand {
    Toggle { id: u64 },
    Create { title: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TodoSnapshot {
    pub revision: u64,
    pub items: Vec<Todo>,
}

impl TodoSnapshot {
    pub fn initial() -> Self {
        Self {
            revision: 0,
            items: initial_todos(),
        }
    }
}

impl Command for TodoCommand {
    type Response = TodoSnapshot;
}

pub const MAX_TODOS: usize = 100;
pub const MAX_TITLE_LENGTH: usize = 100;

/// Shared domain transition, called by a local executor or the server handler.
/// Validation happens before mutation. No automatic retries/idempotency magic.
pub fn apply_todo_command(
    snapshot: &mut TodoSnapshot,
    command: TodoCommand,
) -> Result<TodoSnapshot, ActionError> {
    let revision = snapshot
        .revision
        .checked_add(1)
        .ok_or(ActionError::RevisionOverflow)?;
    match command {
        TodoCommand::Toggle { id } => {
            if id == 0 {
                return Err(ActionError::Validation("invalid_id"));
            }
            let item = snapshot
                .items
                .iter_mut()
                .find(|item| item.id == id)
                .ok_or(ActionError::NotFound)?;
            item.completed = !item.completed;
        }
        TodoCommand::Create { title } => {
            let trimmed = title.trim();
            if trimmed.is_empty()
                || trimmed.chars().count() > MAX_TITLE_LENGTH
                || trimmed.chars().any(char::is_control)
            {
                return Err(ActionError::Validation("invalid_title"));
            }
            if snapshot.items.len() >= MAX_TODOS {
                return Err(ActionError::CapacityReached);
            }
            let next_id = snapshot
                .items
                .iter()
                .map(|item| item.id)
                .max()
                .unwrap_or(0)
                .checked_add(1)
                .ok_or(ActionError::RevisionOverflow)?;
            snapshot.items.push(Todo {
                id: next_id,
                title: trimmed.into(),
                completed: false,
            });
        }
    }
    snapshot.revision = revision;
    Ok(snapshot.clone())
}

/// Pure state transition; changing a Todo does not require a remote request.
pub fn toggle_first(items: &mut [Todo]) -> bool {
    let Some(first) = items.first_mut() else {
        return false;
    };
    first.completed = !first.completed;
    true
}

pub fn todo_list(items: &[Todo]) -> View {
    let children = items.iter().map(|item| {
        View::from(
            Element::new("li")
                .attribute("data-id", item.id.to_string())
                .attribute("data-completed", item.completed.to_string())
                .attribute(
                    "class",
                    if item.completed {
                        "is-complete"
                    } else {
                        "is-open"
                    },
                )
                .child(View::text(&item.title)),
        )
    });
    Element::new("ul")
        .attribute("class", "todo-list")
        .children(children)
        .into()
}

/// Native route: complete HTML, zero browser JS/WASM.
pub fn home_page() -> View {
    Element::new("main")
        .attribute("id", "app")
        .child(Element::new("h1").child(View::text("Yetlet Todo")))
        .child(Element::new("p").child(View::text(
            "Native SSR; no browser runtime or remote UI circuit.",
        )))
        .child(todo_list(&initial_todos()))
        .into()
}

/// Browser route: this *same* TodoList is mounted by the WASM DOM renderer.
pub fn browser_page(items: &[Todo]) -> View {
    Element::new("main")
        .attribute("id", "todo-app")
        .child(Element::new("h1").child(View::text("Yetlet Browser Todo")))
        .child(Element::new("p").child(View::text(
            "Rust/WASM mounts the shared component; clicks stay local.",
        )))
        .child(
            Element::new("section")
                .attribute("id", "todo-list-host")
                .child(todo_list(items)),
        )
        .child(
            Element::new("button")
                .attribute("type", "button")
                .attribute("id", "toggle-first")
                .child(View::text("Toggle first task")),
        )
        .child(
            Element::new("section")
                .attribute("id", "remote-actions")
                .child(Element::new("h2").child(View::text("Explicit HTTP action")))
                .child(
                    Element::new("label")
                        .attribute("for", "remote-title")
                        .child(View::text("Task title")),
                )
                .child(
                    Element::new("input")
                        .attribute("id", "remote-title")
                        .attribute("type", "text")
                        .attribute("maxlength", "100")
                        .attribute("placeholder", "New server task")
                        .attribute("autocomplete", "off"),
                )
                .child(Element::new("p").attribute("id", "remote-title-error"))
                .child(
                    Element::new("button")
                        .attribute("id", "create-remote")
                        .attribute("type", "button")
                        .child(View::text("Create on server")),
                )
                .child(
                    Element::new("p")
                        .attribute("id", "remote-status")
                        .attribute("aria-live", "polite")
                        .child(View::text("No remote action sent")),
                )
                .child(
                    Element::new("button")
                        .attribute("id", "remote-review")
                        .attribute("type", "button")
                        .child(View::text("Review server list")),
                )
                .child(
                    Element::new("button")
                        .attribute("id", "remote-resume")
                        .attribute("type", "button")
                        .child(View::text("Start another create")),
                )
                .child(
                    Element::new("div")
                        .attribute("id", "remote-list-host")
                        .child(
                            Element::new("p")
                                .child(View::text("Server data appears after a remote action")),
                        ),
                ),
        )
        .child(
            Element::new("p")
                .attribute("id", "todo-status")
                .attribute("aria-live", "polite")
                .child(View::text("First task: active")),
        )
        .into()
}

const SEARCH_STATE_SCHEMA_VERSION: u32 = 1;
const SEARCH_COMPONENT: &str = "yetlet-search";
const MAX_SEARCH_STATE_BYTES: usize = 16 * 1024;
const MAX_SEARCH_ITEMS: usize = 128;
const MAX_SEARCH_TITLE_BYTES: usize = 1024;

/// The only fields allowed to cross from native SSR into browser Wasm.
#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchStateV1 {
    schema_version: u32,
    component: String,
    items: Vec<SearchItemV1>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SearchItemV1 {
    id: u64,
    title: String,
    completed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchStateError {
    Oversized,
    InvalidPayload,
    UnsupportedVersion,
    WrongComponent,
    InvalidItems,
}

/// Renderers MUST escape this JSON as a text/attribute value; it is not HTML.
pub fn encode_search_state(items: &[Todo]) -> Result<String, SearchStateError> {
    if items.len() > MAX_SEARCH_ITEMS
        || items
            .iter()
            .any(|item| item.id == 0 || item.title.len() > MAX_SEARCH_TITLE_BYTES)
    {
        return Err(SearchStateError::InvalidItems);
    }
    let state = SearchStateV1 {
        schema_version: SEARCH_STATE_SCHEMA_VERSION,
        component: SEARCH_COMPONENT.to_owned(),
        items: items
            .iter()
            .map(|item| SearchItemV1 {
                id: item.id,
                title: item.title.clone(),
                completed: item.completed,
            })
            .collect(),
    };
    let json = serde_json::to_string(&state).map_err(|_| SearchStateError::InvalidPayload)?;
    if json.len() > MAX_SEARCH_STATE_BYTES {
        return Err(SearchStateError::Oversized);
    }
    Ok(json)
}

/// DOM state is untrusted even when originally generated on the server.
/// Reject unknown schema/component, excess data, unknown fields or bad IDs.
pub fn decode_search_state(raw: &str) -> Result<Vec<Todo>, SearchStateError> {
    if raw.len() > MAX_SEARCH_STATE_BYTES {
        return Err(SearchStateError::Oversized);
    }
    let state: SearchStateV1 =
        serde_json::from_str(raw).map_err(|_| SearchStateError::InvalidPayload)?;
    if state.schema_version != SEARCH_STATE_SCHEMA_VERSION {
        return Err(SearchStateError::UnsupportedVersion);
    }
    if state.component != SEARCH_COMPONENT {
        return Err(SearchStateError::WrongComponent);
    }
    if state.items.len() > MAX_SEARCH_ITEMS {
        return Err(SearchStateError::InvalidItems);
    }
    let mut ids = std::collections::HashSet::new();
    let mut items = Vec::with_capacity(state.items.len());
    for item in state.items {
        if item.id == 0 || item.title.len() > MAX_SEARCH_TITLE_BYTES || !ids.insert(item.id) {
            return Err(SearchStateError::InvalidItems);
        }
        items.push(Todo {
            id: item.id,
            title: item.title,
            completed: item.completed,
        });
    }
    Ok(items)
}

/// Case-insensitive, local-only filtering for the search island.
///
/// M3 uses fixture data compiled into both targets; HTTP data providers are a
/// separate M4 concern. Input is always rendered as text, never HTML.
pub fn filtered_todos(items: &[Todo], query: &str) -> Vec<Todo> {
    let query = query.trim().to_lowercase();
    if query.is_empty() {
        return items.to_vec();
    }
    items
        .iter()
        .filter(|item| item.title.to_lowercase().contains(&query))
        .cloned()
        .collect()
}

pub fn search_results(items: &[Todo]) -> View {
    if items.is_empty() {
        return Element::new("p")
            .attribute("id", "search-empty")
            .child(View::text("No matching tasks"))
            .into();
    }
    todo_list(items)
}

/// SSR markup remains intact until interaction; browser code only owns
/// #search-results and never replaces this page or the input element.
pub fn search_page() -> View {
    search_page_with_todos(&initial_todos()).expect("demo SSR state is bounded")
}

/// Render native SSR state explicitly, instead of depending on an identical
/// fixture separately compiled into browser Wasm.
pub fn search_page_with_todos(todos: &[Todo]) -> Result<View, SearchStateError> {
    let serialized = encode_search_state(todos)?;
    Ok(Element::new("main")
        .attribute("id", "search-page")
        .child(Element::new("h1").child(View::text("Yetlet SSR Search")))
        .child(Element::new("p").child(View::text(
            "Results are server-rendered. Interacting with search lazily loads the island.",
        )))
        .child(
            Element::new("section")
                .attribute("id", "search-island")
                .attribute("data-yetlet-island", "search")
                .attribute("data-island-version", "1")
                .attribute("data-island-state", serialized)
                .child(
                    Element::new("label")
                        .attribute("for", "search-query")
                        .child(View::text("Find a task")),
                )
                .child(
                    Element::new("input")
                        .attribute("id", "search-query")
                        .attribute("type", "search")
                        .attribute("placeholder", "Search task titles")
                        .attribute("autocomplete", "off"),
                )
                .child(
                    Element::new("p")
                        .attribute("id", "search-count")
                        .attribute("aria-live", "polite")
                        .child(View::text(format!("{} matching tasks", todos.len()))),
                )
                .child(
                    Element::new("div")
                        .attribute("id", "search-results")
                        .child(search_results(todos)),
                ),
        )
        .into())
}

#[cfg(test)]
mod tests {
    use super::{
        SearchStateError, Todo, TodoCommand, TodoSnapshot, apply_todo_command, browser_page,
        decode_search_state, encode_search_state, filtered_todos, initial_todos, search_page,
        search_page_with_todos, search_results, todo_list, toggle_first,
    };
    use yetlet_actions::ActionError;
    use yetlet_html::render_to_string;

    #[test]
    fn todo_titles_are_text_not_markup() {
        let view = todo_list(&[Todo {
            id: 1,
            title: "<script>alert(1)</script>".into(),
            completed: false,
        }]);
        let html = render_to_string(&view).unwrap();
        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn local_transition_updates_shared_view() {
        let mut todos = initial_todos();
        let before = render_to_string(&todo_list(&todos)).unwrap();
        assert!(before.contains("data-completed=\"false\""));
        assert!(toggle_first(&mut todos));
        let after = render_to_string(&todo_list(&todos)).unwrap();
        assert!(after.contains("data-completed=\"true\""));
        assert!(after.contains("class=\"is-complete\""));
        assert_ne!(before, after);
        assert!(toggle_first(&mut todos));
        assert_eq!(todos, initial_todos());
        assert!(!toggle_first(&mut []));
    }

    #[test]
    fn todo_command_wire_shape_is_semantic_and_strict() {
        let command = TodoCommand::Create {
            title: "Remote item".into(),
        };
        let encoded = serde_json::to_value(&command).unwrap();
        assert_eq!(
            encoded,
            serde_json::json!({"type":"create","title":"Remote item"})
        );
        assert!(
            serde_json::from_value::<TodoCommand>(
                serde_json::json!({"type":"create","title":"ok","event":"click"})
            )
            .is_err()
        );
        assert!(
            serde_json::from_value::<TodoCommand>(
                serde_json::json!({"type":"submit","target":"#app"})
            )
            .is_err()
        );
    }

    #[test]
    fn browser_page_has_explicit_local_and_remote_action_controls() {
        let page = render_to_string(&browser_page(&initial_todos())).unwrap();
        for id in [
            "toggle-first",
            "remote-title",
            "create-remote",
            "remote-status",
        ] {
            assert!(page.contains(&format!("id=\"{id}\"")));
        }
        assert!(!page.contains("onclick="));
    }

    #[test]
    fn shared_todo_commands_apply_and_reject_invalid_inputs() {
        let mut snapshot = TodoSnapshot::initial();
        let toggled = apply_todo_command(&mut snapshot, TodoCommand::Toggle { id: 1 }).unwrap();
        assert_eq!(toggled.revision, 1);
        assert!(toggled.items[0].completed);

        let created = apply_todo_command(
            &mut snapshot,
            TodoCommand::Create {
                title: "  Saved item  ".to_owned(),
            },
        )
        .unwrap();
        assert_eq!(created.revision, 2);
        assert_eq!(created.items[2].id, 3);
        assert_eq!(created.items[2].title, "Saved item");

        let before = snapshot.clone();
        assert_eq!(
            apply_todo_command(
                &mut snapshot,
                TodoCommand::Create {
                    title: "   ".to_owned(),
                },
            ),
            Err(ActionError::Validation("invalid_title")),
        );
        assert_eq!(
            apply_todo_command(&mut snapshot, TodoCommand::Toggle { id: 99 }),
            Err(ActionError::NotFound),
        );
        assert_eq!(snapshot, before, "rejected actions must not mutate state");
    }

    #[test]
    fn search_filters_locally_and_escapes_results() {
        let todos = initial_todos();
        let filtered = filtered_todos(&todos, "ReNdErErS");
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 1);
        assert!(filtered_todos(&todos, "   ").len() == 2);
        assert!(filtered_todos(&todos, "no-match").is_empty());
        let empty = render_to_string(&search_results(&[])).unwrap();
        assert_eq!(empty, "<p id=\"search-empty\">No matching tasks</p>");
        let page = render_to_string(&search_page()).unwrap();
        assert!(page.contains("data-yetlet-island=\"search\""));
        assert!(page.contains("<ul class=\"todo-list\">"));
        assert!(page.contains("<input id=\"search-query\""));
    }
    #[test]
    fn search_state_round_trip_and_schema_rejection() {
        let todos = initial_todos();
        let encoded = encode_search_state(&todos).unwrap();
        assert_eq!(decode_search_state(&encoded).unwrap(), todos);
        assert_eq!(
            decode_search_state(&encoded.replace("\"schema_version\":1", "\"schema_version\":2")),
            Err(SearchStateError::UnsupportedVersion),
        );
        assert_eq!(
            decode_search_state(&encoded.replace("yetlet-search", "wrong-component")),
            Err(SearchStateError::WrongComponent),
        );
        assert_eq!(
            decode_search_state("{}"),
            Err(SearchStateError::InvalidPayload)
        );
        assert_eq!(
            decode_search_state(&format!("{}{}", " ".repeat(16_385), encoded)),
            Err(SearchStateError::Oversized),
        );
        assert_eq!(
            decode_search_state(&encoded.replace("\"component\"", "\"unknown_field\"")),
            Err(SearchStateError::InvalidPayload),
        );
    }

    #[test]
    fn state_rejects_duplicate_ids_and_extra_fields() {
        let duplicate = vec![
            Todo {
                id: 5,
                title: "first".into(),
                completed: false,
            },
            Todo {
                id: 5,
                title: "second".into(),
                completed: true,
            },
        ];
        let encoded = encode_search_state(&duplicate).unwrap();
        assert_eq!(
            decode_search_state(&encoded),
            Err(SearchStateError::InvalidItems)
        );
        assert_eq!(
            decode_search_state(&encoded.replace(
                "\"completed\":false",
                "\"completed\":false,\"secret\":\"unsupported\"",
            )),
            Err(SearchStateError::InvalidPayload),
        );
    }

    #[test]
    fn state_json_is_escaped_in_ssr_attribute() {
        let todos = vec![Todo {
            id: 9,
            title: "<script>alert('x')</script>\" &".into(),
            completed: false,
        }];
        let page = render_to_string(&search_page_with_todos(&todos).unwrap()).unwrap();
        assert!(page.contains("data-island-state=\"{&quot;schema_version&quot;:1"));
        assert!(page.contains("&lt;script&gt;"));
        assert!(!page.contains("<script>"));
        assert!(matches!(
            encode_search_state(&[Todo {
                id: 7,
                title: "x".repeat(1025),
                completed: false,
            }]),
            Err(SearchStateError::InvalidItems),
        ));
    }

    #[test]
    fn browser_page_embeds_the_same_todo_component() {
        let todos = initial_todos();
        let page = render_to_string(&browser_page(&todos)).unwrap();
        let list = render_to_string(&todo_list(&todos)).unwrap();
        assert!(page.contains(&list));
        assert!(page.contains("id=\"toggle-first\""));
    }
}
