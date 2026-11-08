
## Optional application-owned Rust CSS

This template selects `[cli.styles]` and includes a private, std-only `stylegen/`
package with its own complete lock. Creation does not run it. Prepare the app's
normal dependency lock/cache separately as above, then use `yetlet build`,
`yetlet run` or `yetlet watch`. The official creation/build/run/watch path uses
Rust (and the existing Node coordinator when Vite is selected), without Python.

`stylegen/styles.rs` already composes `layout.rs` and `dialog.rs`. The generator
and Views share `src/style_names.rs`; conditional dialog CSS is registered even
when no dialog View runs. To add a module, create `stylegen/panel.rs`, add
`#[path = "panel.rs"] mod panel;` to `styles.rs`, and add `panel::styles()` to
`StyleSheet::compose`. Creating a source file alone does not register styles.
The builder remains app-owned; use ordinary CSS for unsupported syntax.

Standalone native development serves stable `assets/generated/app.generated.css`.
Successful production builds accept `assets/generated/app.<full-sha256>.css` via
private `.yetlet/styles/production.json`; ship that record with its exact assets.
Requests serve an immutable startup snapshot, with existing CSP and no-store.
Native refresh requires a backend restart and normal page refresh. Do not edit
owned CSS: a changed file conflicts with its ownership record. Generator failure
preserves previous CSS but does not make new inputs fresh. If metadata fails after
CSS replacement, the generation is explicitly unaccepted: retain the record and
output evidence, then restore the prior recorded bytes or perform an explicit
operator reconciliation before retrying. There is no two-file transaction.

With Vite, the configured output is `frontend/style-output/app.generated.css`,
imported after `app.css` through Vite's normal graph. The browser-style variant
moves its app.css connection from the HTML link into that same entry graph. Vite owns bundle hashing,
links and selected HMR; no standalone second link is added. With standalone
Tailwind, document order is Tailwind then generated CSS. Later source order only
breaks ties after layers, importance and specificity: unlayered normal generated
CSS may outrank layered utilities. Keep authored overrides deliberately ordered.
Settings/policy changes require restart. External mutable generator inputs,
generator dependencies/build scripts, automatic installs, automatic asset cleanup,
retention, HA/deployment guarantees and request-time compilation are unsupported.
Windows code is supplied through native adapters; runtime qualification is separate.
Style generation currently refuses Windows headless/raw-console execution; an
attached console with processed Ctrl-C input is required by owned capture.
