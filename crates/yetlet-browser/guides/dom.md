# yetlet-browser::dom

Mount a shared Rust View into an explicitly owned browser surface. This optional
renderer replaces a subtree; native SSR works without selecting it.

**Experimental SDK `0.9.0-alpha.5`, published on crates.io.** `yetlet-browser`
selects `dom` by default; actual mounting requires the WASM target and browser
host. The commands below run checkout examples at their documented scope.

**Unreleased checkout change:** [YT-220](../../../docs/YT-220-HTML-DOM-IMPLEMENTATION.md)
extends the source DOM vocabulary to89 elements with scoped attributes and
canonical table/disclosure/control validation. These additions are not in the
published SDK0.9.0-alpha.5's38-element baseline. No new package is published;
use the selected source checkout to try this section's new capabilities.

**Unreleased opt-in:** [YT-221](../../../docs/YT-221-DOM-URL-IMAGE-CONTRACT.md)
adds per-mount navigation/image options. Existing entry points retain89 tags,
same-origin navigation and no images; explicitly selected images add img only.
These APIs also require the source checkout and are not in SDK0.9.0-alpha.5.

## Prerequisites

Use this checkout with the pinned Rust/Cargo toolchain. Native `validate` is
available without a browser. Actual mounting requires WASM and a browser host.
Before the commands below, prepare `wasm32-unknown-unknown` and the Cargo.lock-
matching wasm-bindgen CLI (0.2.129): follow the [root browser setup](../../../README.md#choose-your-next-step),
including tool installation location/PATH. Run commands from the repository root;
stop another server using port 3000 first.

## Minimal integration

### Mount an owned surface

`validate(&view)` is available on native and WASM targets and returns `Result<(), DomPolicyError>`. On WASM, `mount(&web_sys::Element, &view)` validates and constructs the replacement before clearing the old subtree; browser API failures return `JsValue`. There is no native `mount` implementation.

The [Search Island caller](../../../examples/search-island/src/lib.rs) keeps complete SSR readable and replaces only its results host after activation. Follow [Todo setup](../../../examples/todo-server/README.md), or run from the repository root after installing the documented WASM build prerequisites:

```bash
cargo xtask build browser
cargo run --locked -p yetlet-todo-server
```

Open `http://127.0.0.1:3000/search`. The external loader builds/loads assets explicitly; this crate does not load modules or issue HTTP actions.

## Expected result

At `http://127.0.0.1:3000/search`, complete SSR is readable before browser
activation; the selected island updates only its owned results surface. Static
validation alone does not mount anything or prove browser navigation acceptance.

## Configuration and errors

### Supported subset and lifetime

The published [YT-110 baseline](../../../docs/YT-110-DOM-CAPABILITIES.md#dom-vocabulary)
supports38 tags and42 named attributes. The unreleased YT-220 checkout adds all51
ordinary elements selected by the [coverage contract](../../../docs/YT-219-HTML-DOM-CONTRACT.md):
text semantics, explicit table sections, figure/caption, pre, classic controls,
details/summary and canvas/dialog hosts. New scoped attributes and numeric/keyword
checks coexist with old admission rules. Unsupported names, duplicates, void
children and selected invalid shapes fail before mutation. No wildcard attributes,
inline style/handler, namespace or raw HTML escape is admitted. Resource URLs
remain excluded through the default entry points.

Build new structural families together in the supplied View: tables own explicit
row sections, details owns its first summary, and figure owns its optional caption.
This subset has no separate orphan-table-part/summary fragment context API. Native
disclosure/control behavior remains browser-owned; imperative dialog methods and
focus are explicitly application-owned. Pre's parsed initial newline and adjacent
CR/LF stream now match direct DOM construction; affected serialized pre bytes may
change. Published artifacts retain their old behavior.
See the [implementation record and limits](../../../docs/YT-220-HTML-DOM-IMPLEMENTATION.md).

Initial input/textarea/select/checkbox defaults are declarative; live values, chosen events, reset and submission remain browser/application-owned. Textarea text has LF normalization and preserves an initial newline across SSR and DOM. See the shared [native controls example](../../../examples/todo-ui/src/controls.rs), available at `/app` and the no-JS `/controls` route in Todo.

`href` accepts a root-relative path or fragment; form `action` accepts a root-relative path. Static `validate` checks the narrow spelling. WASM mount additionally uses the browser URL parser and effective document base, rejects non-HTTP(S), cross-origin or credential-bearing destinations, and stores absolute URLs before touching the target. Therefore mount can reject a View that passed static validation. Later base changes do not retarget accepted attributes. The broader SSR URL policy remains separate; see [navigation limits](../../../docs/YT-110-DOM-CAPABILITIES.md#navigation-and-submission-boundary).

The caller owns the target and event listener lifetime. Keep listeners outside the replaced subtree or explicitly release/rebind them. Subtree replacement does not preserve child identity, focus or selection as a general contract, and unexpected DOM API failures are not transactional rollback. Choose the opt-in `dom-keyed` feature for proven identity-sensitive collections. There is no general VDOM, hydration, resumability or GPU acceleration.

### Explicit links and images (unreleased source)

Pass typed options to one owned surface; neither policy enables the other.
This example allows a named HTTPS link and local images, without permitting
external form submission:

```rust
use yetlet::view::{Element, View};
use yetlet_browser::dom::{DomOptions, ImagePolicy, NavigationPolicy, validate_with_options};

let options = DomOptions::default()
    .with_navigation_policy(NavigationPolicy::HttpsOrigins(vec![
        "https://docs.example.invalid".into(),
    ]))
    .with_image_policy(ImagePolicy::SameOrigin);
let view: View = Element::new("section")
    .child(Element::new("a")
        .attribute("href", "https://docs.example.invalid/guide")
        .attribute("target", "_blank")
        .attribute("rel", "noopener noreferrer")
        .child(View::text("Read the guide")))
    .child(Element::new("img")
        .attribute("src", "/images/example.png")
        .attribute("alt", "Example diagram")
        .attribute("width", "320")
        .attribute("height", "180"))
    .into();
validate_with_options(&view, &options)?;
// On Wasm: mount_with_options(&owned_target, &view, &options)?;
# Ok::<(), yetlet_browser::dom::DomPolicyError>(())
```

The shared [Todo consumer](../../../examples/todo-ui/src/controls.rs) exposes
`controlled_links_and_image` separately; the existing Todo routes do not install
this policy automatically. Applications own activation and the host CSP.

Options store public origin strings; native validation checks structure/spelling
only. It does not parse configuration or prove destination membership. Mounting
parses every configured origin as a canonical HTTPS origin (optional trailing
slash; no credentials/path/query/fragment/wildcards), validates the complete View
and freezes all href/action/src destinations before creating any nodes. Rejection
retains the old surface without resource assignment. Root-relative URLs cannot
escape through a foreign document base, even if that origin is separately listed.
Use `ImagePolicy::HttpsOrigins` to permit named HTTPS image origins independently.

Images require src and alt (empty alt is valid decoration), remain void, and
accept finite dimensions/loading/decoding/fetchpriority/referrerpolicy values.
Responsive srcset/sizes/picture/source remain deferred. Selected anchors accept
only `_self`/`_blank`, finite rel tokens and scoped metadata; `_blank` requires
explicit noopener. Ping/download/named windows remain excluded. Forms retain
same-origin root-relative actions and get/post under every option.

After admission, native loading, HTTP redirects, image errors and CSP are
browser-owned; the policy checks initial assigned origins, not redirect chains.
Loads may start before insertion and are not rolled back. Failed images keep
native error/alt behavior; there is no framework retry or display guarantee.
Accepted resources can send browser-managed credentials/referrers. This is not
authentication or privacy isolation. See the [full policy and evidence](../../../docs/YT-221-DOM-URL-IMAGE-CONTRACT.md).

## API reference

See [DOM source/rustdoc](../src/dom/mod.rs), the [current API reference](../../../docs/API-REFERENCE.md)
and the linked supported-vocabulary/navigation contract below.

### License and compatibility

Dual-licensed under [MIT](../../../LICENSE-MIT) or [Apache-2.0](../../../LICENSE-APACHE).
SDK `0.9.0-alpha.5` is published; APIs remain Experimental. Read
[compatibility](../../../docs/COMPATIBILITY.md) and the
[current API reference](../../../docs/API-REFERENCE.md) at the matching revision.
The historical evidence below retains its original candidate and scope; it does
not qualify every later checkout or promise production support.

<details>
<summary>Historical delivery context and evidence</summary>

**Experimental `0.9.0-alpha.5`, unpublished.** Requires Rust 1.99 or newer. This is one of Yetlet's sixteen library candidates; use the local checkout rather than assuming a registry release. APIs are pre-public, and bounded prototype verification is not production qualification.

`yetlet_browser::dom` mounts a shared View by constructing real browser nodes and replacing one owned subtree. It depends on `yetlet::view`; `wasm-bindgen` and `web-sys` are added only for `wasm32`. Native SSR does not require this crate.

</details>
