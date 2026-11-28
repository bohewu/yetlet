# Write Views and reusable components

Continue [your first native application](FIRST-APP.md) by expanding its initial
heading into a small page. Use ordinary Rust functions and the existing
`View`/`Element` API to pass data, compose components and render a list.
**Experimental SDK `1.0.0-rc.1`, published on crates.io.** This guide
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

Keep the generated `use yetlet::web::prelude::*;` import. It already
brings `Element` and `View` into scope. Replace the original `home` function
with the following type, helpers and handler. Keep the `hello` route and
handler and the main function. The existing `.page("/", "Yetlet", home)` registration can call
this same no-argument async `View` handler without a Controller or adapter.

```rust
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

## Typed tag builders

`yetlet::view::tags` is an **optional authoring API, first published in SDK
`0.9.0-alpha.7`** and retained by later SDK versions. It offers typed
constructors as a simpler alternative to the existing `Element::new` API,
without a new rendering runtime, an extra crate or an automatic browser
feature. They are also available from published SDK `1.0.0-rc.1`.
The first-app example above intentionally retains the reviewed
[Reader Edition](../site/pages.toml), which is independently pinned to its
published SDK and CLI. See the [alpha.7 publication and registry consumer](ALPHA7-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-09)
for distribution, rather than treating an earlier source-only acceptance as
publication or browser qualification.

```rust
use yetlet::view::{View, tags::{div, h1, input, textarea}};

let panel: View = div()
    .id("app")
    .class("container")
    .add_class("active")
    .child(h1().text("Hello <Yetlet>"))
    .child(input().placeholder("Search..."))
    .child(textarea().placeholder("Notes..."))
    .into();
```

These constructors produce the exact same owned `Element` / `View` tree as
`Element::new`. They do not invoke a template compiler or alter the HTML/DOM
renderers. `.id()` replaces an existing id, `.class()` replaces the class
tokens, and `.add_class()` appends distinct HTML ASCII-whitespace-separated
tokens in first-seen order, leaving just one class attribute. An empty addition
on a tag without classes does nothing.

`.placeholder()` is exposed on typed `input()` and `textarea()`, not on
`div()`. Child/text methods are not exposed on typed void elements such as
`input()`. Generic `.attribute(name, value)` remains append-only, just like
`Element::attribute`, so a duplicate appended *after* a convenience setter
can still be rejected by the renderer. A following dedicated setter
consolidates its own attribute name. There is no `.style()` helper: inline
style remains forbidden by the existing HTML and DOM output policies.

The original `Element::new` constructor and mixed child composition remain
source compatible. Typed authoring is only an ergonomic, partial compile-time
aid; renderer policies still validate the complete tree and can differ between
native HTML and DOM. The [implementation](../crates/yetlet/src/view/tags.rs)
is the current callable source. The [original YT-234 design and independent
local acceptance](https://github.com/bohewu/yetlet/blob/9a6f954dc38f3b9e854fce5a08c9b843182657f8/docs/YT-234-VIEW-AUTHORING-API-PLAN.md#source-only-acceptance-record--2026-11-07)
remain recoverable at that fixed pre-deletion commit, separate from the later
SDK publication.


### Standard HTML authoring coverage

The published SDK alpha.7 module has **101 standard HTML constructors**
selected from the dated [HTML inventory](API-CONTRACTS.md#historical-html-element-and-attribute-inventory):
89 currently admitted Browser DOM elements, `img()` (admitted in Browser only
when the caller selects an `ImagePolicy`) and 11 native SSR-only/deferred
elements. The same owned `View` tree reaches the existing HTML and DOM
renderers; these constructors do not add browser DOM support.

```rust
use yetlet::view::{View, tags::{a, div, img, table, tbody, td, tr}};

let page: View = div()
    .child(a().href("/about").text("About"))
    .child(img().src("/logo.png").alt("Logo"))
    .child(table().child(tbody().child(tr().child(td().text("Hello")))))
    .into();
```

Frequently used attributes have optional typed setters: `.href()` for
`a`/`area`, `.src()` for `img` and selected media/source tags,
`.alt()` for `img`/`area`, `.type_()` for supported type-bearing
tags, `.for_()` for `label`/`output`, and `.name()`/`.value()` for
selected form-associated elements. All these methods **set/replace** their
own attribute. The general `.attribute()` remains append-only. URL and
other value checks still belong to the selected renderer.

The nine ordinary View void tags (`area`, `br`, `col`, `hr`, `img`,
`input`, `source`, `track`, `wbr`) have no typed `.child()` or
`.text()` method. `video`, `audio`, `source`, `track`, `picture`,
`area`, `map`, `noscript`, `template`, `slot`, and
`selectedcontent` are currently SSR-only/deferred for Browser DOM.
A Browser `img` needs `DomOptions::with_image_policy`. Its constructor
does not opt the application into image loading.

Document/head-owned tags (`html`, `head`, `body`, `base`, `link`,
`meta`, `title`) are intentionally not ordinary View conveniences; use
the separate document writer. The five forbidden executable/embedded tags
remain rejected. Foreign namespaces and custom elements are outside this
API. This is **not** full HTML conformance or an enlargement of either
renderer policy.

See the [current typed constructors](../crates/yetlet/src/view/tags.rs)
and [original YT-235 source tests and independent acceptance](https://github.com/bohewu/yetlet/blob/9a6f954dc38f3b9e854fce5a08c9b843182657f8/docs/YT-235-TAG-COVERAGE-PLAN.md#yt-235-final-local-acceptance--2026-11-07).
Those original native tests, doctests, wasm32 builds and Browser DOM policy
unit checks validate only their dated source; they do not establish later
browser interaction, production support or general HTML conformance.

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

**Form and reactive authoring boundaries:** the original Admin/Todo
application feedback work demonstrated three app-owned slices: convert native
server validation errors to field feedback; model explicit save/recovery state;
and reconcile editor/native/Todo interactions without silently persisting each
keystroke. An HTTP 200, field message or editor reset cannot alone establish a
durable business write, authorization or safe retry. The current
[forms example](../examples/forms-validation/README.md) owns native validation
and redirect/CSRF behavior; [reactivity](../crates/yetlet/guides/reactive.md)
owns scoped committed-state reads and weak-handle disposal. The
[maintained weak-handle snapshot contract](ARCHITECTURE.md#weak-handle-published-state-snapshot-and-explicit-first-render)
describes published-state reads, disposal and explicit first-render policy. Early YT-107–109 checks remain source-bound,
not certification of generalized two-way bindings.

The native and browser renderers accept only their declared `hidden`
attribute combinations. A successful SSR tree does not imply that unsupported
DOM attributes or native control defaults will be mirrored by browser mount;
manual controls, form reset, navigation and focus stay app/browser-owned.
Read the [bounded DOM contract](BROWSER-APP.md#historical-native-dom-controls-and-navigation-boundaries) and
[DOM guide](../crates/yetlet-browser/guides/dom.md).

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

## Binding metadata and explicit lifetimes

Ordinary View functions do not subscribe to state implicitly. Optional browser
binding uses validated component/instance identities, nonempty unique keys and
safe slot descriptors; these metadata values select neither a renderer nor a
macro/compiler. Inert attribute names still require renderer admission and
escaping, and cannot grant raw HTML or reserved framework attributes.

For updates, explicitly select the [keyed DOM module](../crates/yetlet-browser/guides/dom-keyed.md)
and [local graph](../crates/yetlet/guides/reactive.md) when needed. Keyed identity
preserves owned nodes/state through reorder; graph handles are weak and the caller
retains the strong lifetime owner. Listener disposal and queued/awaiting task
retirement are separate obligations. The [browser ownership contract](BROWSER-APP.md)
explains mutation failures and input/focus limits. The historical fixed-list
`KeyedListTemplate`/`BoundKeyedList` authoring fixture is experimental and is not
a current general component, forms, signals or resumability API.

## Single-file components in the source checkout

The checkout adds an experimental, default-off `sfc` build feature. Use the
same local `yetlet` path in both Cargo sections. Select `features = ["html"]`
in `[dependencies]` for SSR and `features = ["sfc"]` in
`[build-dependencies]`. The published rc.1 walkthrough above uses plain Rust.

The runnable [SFC Todo sample](../examples/sfc-todo/README.md) shows an add
form and repeated task items built from separate component files.

For example, save this trusted source as `src/Notice.yetlet`.

```html
<template message="text" visible="bool">
  <aside class="notice"><p when="visible">{message}</p></aside>
</template>
<style>
.notice p { font-weight: bold; }
</style>
<script>
for (const notice of document.querySelectorAll(".notice")) {
  notice.addEventListener("click", () => notice.classList.toggle("selected"));
}
</script>
```

Give all three sections opening and closing tags on separate lines. Each
template has one XML-style root element. Template attributes declare borrowed
text or boolean props. Write one `{text_prop}` per text node or attribute value,
optionally surrounded by literal text. Use `when="bool_prop"` on a child to
conditionally include its subtree. The existing View renderer handles escaping.
The compiler copies the style and script bodies verbatim. Scope selectors and
DOM queries to the component you intend to affect.

In the application’s `build.rs`, assign each component a distinct lowercase
asset ID. IDs start with a letter and may contain letters, digits and hyphens.

```rust
fn main() -> std::io::Result<()> {
    yetlet::sfc::compile(
        &[("notice", "src/Notice.yetlet")],
        std::env::var_os("OUT_DIR").unwrap(),
    )
}
```

Include the resulting Rust file in its own module and pass the generated props.

```rust
mod notice {
    include!(concat!(env!("OUT_DIR"), "/sfc-notice.rs"));
}
let view = notice::render(&notice::Props { message: "Ready", visible: true });
```

The generated `notice::STYLE` and `notice::SCRIPT` each contain
`(filename, contents)`. For example, mount `notice::STYLE.1` as `text/css` at
the URL formed by joining your asset prefix and `notice::STYLE.0`. Link that
URL in the document and serve the script the same way with a JavaScript MIME
type and an external deferred script tag. Asset URLs and cache policy belong
to the application.
