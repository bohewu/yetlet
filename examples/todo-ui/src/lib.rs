//! Shared UI source independent of any HTTP server or browser API.
//!
//! Both native HTML SSR and browser DOM rendering consume these same Views.

use yetlet_core::{Element, View};

#[derive(Debug, Clone, PartialEq, Eq)]
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
            Element::new("p")
                .attribute("id", "todo-status")
                .attribute("aria-live", "polite")
                .child(View::text("First task: active")),
        )
        .into()
}

#[cfg(test)]
mod tests {
    use super::{Todo, browser_page, initial_todos, todo_list, toggle_first};
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
    fn browser_page_embeds_the_same_todo_component() {
        let todos = initial_todos();
        let page = render_to_string(&browser_page(&todos)).unwrap();
        let list = render_to_string(&todo_list(&todos)).unwrap();
        assert!(page.contains(&list));
        assert!(page.contains("id=\"toggle-first\""));
    }
}
