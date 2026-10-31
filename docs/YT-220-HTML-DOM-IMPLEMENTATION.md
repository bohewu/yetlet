# YT-220 — Ordinary HTML DOM vocabulary implementation

Date: 2026-10-31. Status: Done at selected unreleased-source scope; independent Reviewer PASS0.
Baseline: `c52ba85208629c0fa34368e11c3c7cc527fb1e2e`, the strict/builtin
commit of the accepted [YT-219 contract](YT-219-HTML-DOM-CONTRACT.md).
That seven-document commit passed the direct Guard and native hook with zero
findings; no Push. The owner then selected this implementation. New changes
remain uncommitted/unpushed; no package version, release or live-site change.

## Delivered source scope

The checkout DOM renderer now admits **89 named HTML elements**: the38 original
tags plus all51 additions selected by YT-219. This is **unreleased source**;
the published SDK0.9.0-alpha.5 still has its recorded38-element DOM subset.
YT-221's img/navigation options are not implemented here. Current image,
external-link, executable/embedded, document-root and namespace exclusions stay.

| Changed surface | Result |
|---|---|
| `crates/yetlet-browser/src/dom/mod.rs` | Existing validate/mount entry points, duplicate/URL checks and prevalidation/owned replacement remain; parent-aware selected structure checks and pre text-stream construction |
| Private `dom/vocabulary.rs` | Finite89-name set, preserved old-name admission, selected scoped attributes, keyword/numeric validation and meter/progress bounds |
| Private `dom/structure.rs` | Canonical table sections, colgroup/span, details/summary, figure/caption and classic select/optgroup/datalist shapes |
| `dom/tests_yt220.rs` | Five focused acceptance/boundary tests; no new test framework or public API |
| `crates/yetlet/src/html/mod.rs` | Private initial-pre-newline compensation and one new escaped-output/text-boundary test |
| Existing Todo native-controls consumer and private content module | Shared document example with code/figure/table/disclosure, datalist, grouped multiple select, meter/progress, canvas/dialog hosts; one new native consumer test |

No dependency, manifest, lockfile, Cargo feature, View variant, public error
variant, Settings provider, Core event/lifetime protocol, renderer options or
native OS adapter changes. Existing context/keyed/event owners remain explicit.
The private modules do not create a generic HTML conformance engine or a second
application runtime.

## Behavior and compatibility details

All51 names match the accepted inventory. Structural child nodes must include
their selected owner in the supplied View: construct a whole table with sections
and rows, details with its first summary, and figure with its optional first/last
caption. This slice does not expose a separate fragment-context validation API
for mounting an orphan table part/summary/caption into an already-owned parent.
Only HTML formatting whitespace is admitted in the new table structures; NBSP
and other non-formatting text are rejected before mutation. Existing select's
previous whitespace admission is retained for compatibility.

All existing globally admitted names on the old38 tags remain accepted, including
historically broad type/for/placeholder/maxlength/autocomplete. New-element uses
and new names are scoped. Existing literal boolean presence remains unchanged:
for example multiple/open/disabled with a string false are still present/true.
New numeric checks bound table spans, dimensions, list/control sizes and the
selected meter/progress ranges; floats reject non-finite/Rust-only spellings.
Meter permits an equal min/max with its value at that bound, as the HTML authoring
inequalities allow. Input min/max/step values remain native-type-dependent (for
example dates and step=any), not a new all-input numeric schema. Labels, ID refs,
form associations and named disclosure groups are application-owned document
semantics; they do not become an authorization or binding service.

Pre serialization emits one extra LF only when its actual initial text would
otherwise lose the first normalized LF in the HTML parser. An initial code/span
element does not get that compensation. Direct DOM construction joins adjacent
text inside pre and normalizes CR/CRLF across each such run, including nested
code/span text. It does not merge across intervening element markup. Existing
textarea handling is retained. Escaping and public escape_text remain unchanged.
This intentionally changes affected pre output bytes to preserve parsed text;
it does not alter unrelated SSR admission or globally validate SSR tables.

Default root/fragment navigation grammar, browser HTTP(S)/same-origin/credential
checks and destination freezing are unchanged. Resources stay rejected. Full
validation/construction still precedes clearing the target. Existing limits on
final-operation rollback, focus/selection and listener ownership remain.

## New verification and retained observations

Receipts are retained in `target/yt220-20261031/`; fixture source, temporary build
lock, exact JS/Wasm and browser receipts/screenshots are archived outside the
repository in the same-named evidence set. The isolated fixture imports the
actual modified crates and Todo consumer; it is not a JS replacement renderer.
No generator, browser runner, test orchestration framework or dependency is
added to the repository.

| Check | Actual result and limit |
|---|---|
| New DOM tests |5 PASS, old8 filtered; tag/scope/exclusion, table/spans, disclosure/controls, keywords/compatibility and numeric boundaries |
| New HTML test |1 PASS,14 filtered; pre initial LF/CR/CRLF, empty/adjacent text, leading child element and escaped markup |
| New Todo content test |1 PASS,12 filtered; shared reference markup and escaping |
| Affected legacy rejection case |1 PASS,7 filtered; its old select/multiple rejection now uses div/multiple because select/multiple is newly admitted |
| Scoped strict Clippy | DOM and HTML libraries PASS; original DOM collapsible-if failure retained, only that failed check repeated after an equivalent condition rewrite |
| Actual shared fixture | Native render/validation and wasm32 build with matching wasm-bindgen0.2.129 PASS; no server-only feature graph selected |
| Chrome156.0.8078.73 | Real SSR versus actual Wasm mount canonical tree/attribute/combined-text parity; pre/table cases; pointer/Enter disclosure, grouped multiple select, numeric properties, explicitly owner-driven dialog;8 rejection cases retain original node |
| Firefox159.0 | Same changed SSR/DOM/parsing cases and pointer/Enter disclosure, native-control properties/dialog,8 rejection cases PASS |
| Mobile | Chrome390x844 touch disclosure and no horizontal page overflow PASS; emulation, not physical-mobile qualification |
| Error/CSP/request observations | Chrome no captured page/console/HTTP/CSP errors; Firefox no captured application error/rejection/CSP violation. Owned server records zero denied-resource requests |
| Screenshots | Chrome desktop/mobile and Firefox desktop captured and visually inspected; fixture styles are test-local, no product restyle |

Commands use offline/locked Cargo and the existing tools. Focused executions:

```sh
cargo test --offline --locked -p yetlet-browser --no-default-features --features dom --lib yt220
cargo test --offline --locked -p yetlet --no-default-features --features html --lib yt220
cargo test --offline --locked -p yetlet-todo-ui --lib yt220
cargo test --offline --locked -p yetlet-browser --no-default-features --features dom --lib dom::tests::rejects_invalid_control_children_and_misplaced_attributes -- --exact
cargo clippy --offline --locked -p yetlet-browser --no-default-features --features dom --lib -- -D warnings
cargo clippy --offline --locked -p yetlet --no-default-features --features html --lib -- -D warnings
```

The Browser plugin was absent; use of the existing Playwright Chromium and
Firefox WebDriver is recorded. The first two-browser cohort stopped at a common
fixture comparison: different SSR/DOM filenames made the same fragment resolve
to different document URLs. The initial failures were preserved. One smallest
canonical fixture correction gives both variants the same explicit same-origin
base; exact absolute-URL/tree comparison then passes in each browser. Product
code and assertions were not weakened, and no already-passed old product suite
was replayed. Native7 new tests preceded the later Clippy-only equivalent
condition rewrite; final Wasm compilation and actual browser checks use that
final source, so the functional native evidence is reused without claiming a
second execution.

Both owned browser contexts/processes closed. Firefox's two temporary profiles
were deleted and actual browser PIDs exited; the loopback fixture server was
stopped with its owned terminal interrupt and its listener is absent. The
expected KeyboardInterrupt at server shutdown is cleanup, not a product failure.
No existing service, hosted workflow, Pages setting or release environment changed.

## Acceptance and next item

Documentation/link/structure/whitespace/DAG checks cover the changed guides and
backlog. Old SDK qualification, all-sample/browser suites, Windows/MSVC, GPU,
provider/Auth/Session/load/security/release campaigns are not rerun or newly
claimed. The actual cross-browser evidence is scoped to the changed cases above,
not complete89-element browser/HTML/ARIA conformance or a performance claim.

Independent Reviewer PASS0 accepts source, consumer, browser/failure/cleanup
receipts and compatibility limits. All16 reviewed hashes matched before these
completion-only metadata updates; zero issues or required followups. Registered
reviewer model gpt-6-astra and effective high effort were verified. YT-220 is Done
at this selected unreleased-source scope. YT-221–224 remain Todo. Next unblockable item:
**YT-221**, controlled navigation/image options, under separate selection.
New work stays uncommitted/unpushed; no version bump or publication follows.

Independent review: **PASS0**, zero issues/required followups.
