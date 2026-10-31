# Write Views and reusable components

Continue [your first native application](FIRST-APP.md) by expanding its initial
heading into a small page. Use ordinary Rust functions and the existing
`View`/`Element` API to pass data, compose components and render a list.
**Experimental SDK `0.9.0-alpha.5`, published on crates.io.** This guide
continues the registry-first application from the linked walkthrough.

## Prerequisites

Complete the plain native [first-app walkthrough](FIRST-APP.md), including the
published `yetlet` SDK dependency, app lock/cache and existing routes.
Use its same Rust/Cargo toolchain and explicit app/CLI paths. This guide needs
no additional package, Cargo feature, Settings loader, browser build or database.

The example below is code for the generated app's existing `src/main.rs`, not a
second project or a standalone program. Stop its running server before editing.
Its existing imports, Tokio main, listener and route registration remain needed.

## Minimal integration

Keep exactly one `use yetlet::view::{Element, View};` import as shown below;
the plain scaffold already has it. Replace the original `home` function with
the following type, helper functions and `home` handler. Keep the other imports,
`hello` route/handler and main function. The existing `.page("/", "Yetlet", home)` registration can call
this same no-argument async `View` handler without a Controller or adapter.

```rust
use yetlet::view::{Element, View};

struct Task {
    title: String,
    completed: bool,
}

fn task_item(task: &Task) -> View {
    Element::new("li")
        .attribute("class", if task.completed { "is-complete" } else { "is-open" })
        .child(View::text(task.title.as_str()))
        .child(View::text(if task.completed { " — done" } else { " — open" }))
        .into()
}

fn task_list(tasks: &[Task]) -> View {
    if tasks.is_empty() {
        return Element::new("p").child(View::text("No tasks yet.")).into();
    }
    Element::new("ul")
        .children(tasks.iter().map(task_item))
        .into()
}

fn task_panel(heading: &str, tasks: &[Task]) -> View {
    Element::new("main")
        .child(Element::new("h1").child(View::text(heading)))
        .child(task_list(tasks))
        .into()
}

async fn home() -> View {
    let tasks = [
        Task { title: "Read the guide".into(), completed: true },
        Task { title: "Write a component".into(), completed: false },
    ];
    task_panel("Tasks for <Yetlet>", &tasks)
}
```

The public [core constructors](../crates/yetlet/src/view/mod.rs) own the tree.
`child` takes one value convertible into View; `children` takes an iterator of
Views. `tasks.iter().map(task_item)` therefore supplies the latter directly.
Tag/attribute names are static strings; dynamic text and attribute values become
owned strings. The returned View does not borrow `tasks` or the heading, and the
local task data can go out of scope after construction. Borrowing the input
avoids cloning the whole application record; copying display strings is still
part of constructing these owned nodes.

A reusable component here is simply a function returning a View. Its arguments
are ordinary Rust data. `task_panel` composes `task_list`, which composes
`task_item`; the `if` chooses the empty-state subtree before rendering. There is
no Component trait, registration, macro, implicit state subscription or DI step.
The existing [Todo list](../examples/todo-ui/src/lib.rs) uses the same iterator/
child composition with real app-owned records.

Use the first-app [build/run instructions](FIRST-APP.md#3-prepare-the-app-lockcache-then-build-and-run)
with the same explicit app manifest. These edits do not change dependencies;
there is no reason to regenerate the app lock just to change a View. Commands
remain reader instructions, not a claim that this documentation example was run.

## Expected result

Open the first app's existing printed loopback address, normally
`http://127.0.0.1:3000/`. The page displays:

- `Tasks for <Yetlet>` as literal heading text;
- `Read the guide — done`;
- `Write a component — open`.

The server renderer escapes the angle brackets; they do not create a Yetlet
HTML tag. The `is-complete`/`is-open` class names only label elements; no CSS or
automatic completed-task styling is installed. Changing `tasks` to an empty
array with the same inferred Task type selects `No tasks yet.` instead of a list.
Stop and build/run again to see source changes; ordinary native run is not watch.

This is a static response constructed from the current data on each request.
It adds no task mutation, persistent state, browser event listener or automatic
DOM update. The existing `/hello/Ada` route remains as described in the first-app
walkthrough. A no-JS page needs none of those optional browser capabilities.

## Configuration and errors

`Element` and `View::text` store values; construction does not sanitize them.
The selected renderer applies its own output policy. Pass untrusted display
strings to `View::text` or validated inert attribute values, never pre-escape
text or treat it as trusted markup. There is no raw HTML insertion API. Do not
put credentials, Session handles, CSRF tokens or trusted identities into a
reusable display model for generic browser serialization.

For the server, the existing `App::page` adapter returns a complete escaped
HTTP document with its strict CSP. Use ordinary `.get` handlers returning
`Page` for parameterized/fallible cases, following the
[server guide](../crates/yetlet/guides/server-axum.md). A rendering policy error
maps to a generic 500; check supported tags/attributes, duplicates, void children
and URL policy rather than weakening validation. The
[HTML guide](../crates/yetlet/guides/html.md) owns fragment/error details.

Native form semantics, business validation, authentication, authorization and
CSRF remain application responsibilities; composition alone supplies none of
them. Follow [forms](../examples/forms-validation/README.md) and
[Session cart](../examples/cart-server/README.md) for existing complete paths.

The renderer-neutral shape does not make all Views browser-compatible.
[DOM mounting](../crates/yetlet-browser/guides/dom.md) has a finite vocabulary and browser
navigation/lifetime checks. List children and CSS classes do not create keyed
identity or events. Use the explicit [keyed DOM](../crates/yetlet-browser/guides/dom-keyed.md)
and [reactive](../crates/yetlet/guides/reactive.md) guides only when selecting
those capabilities, with their ownership and disposal contracts.

## API reference

- [Core guide](../crates/yetlet/guides/core.md) and
  [core source/rustdoc](../crates/yetlet/src/view/mod.rs): constructors and ownership.
- [Hello World handlers](../examples/hello-world/src/lib.rs) and
  [Todo shared Views](../examples/todo-ui/src/lib.rs): existing application patterns.
- [Current API reference](API-REFERENCE.md) and
  [opt-in quick reference](OPT-IN-QUICK-REFERENCE.md): later capability selection.
- [Troubleshooting](TROUBLESHOOTING.md): existing setup, renderer and HTTP failures.
