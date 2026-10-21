# YT-106 — Admin Description editor

Date: 2026-10-20. Base checkout: `11e510f`, the committed, independently reviewed
[CSP study](YT-106-CODEMIRROR-CSP-RESEARCH.md). The user separately authorized
this minimal plain-text Description trial, existing Vite/Rust/WASM/CLI delivery,
page-local style nonce, native fallback and independent review. Independent
Reviewer `/root/yt106_codemirror_impl_review_oct20` PASS has zero issues or
required followups; this bounded application trial is complete.

## Delivered behavior and ownership

Only the existing interactive Admin triage page enhances Description with
CodeMirror. Rust retains per-item drafts, selection, filtering and explicit typed
HTTP Save. CodeMirror owns its dedicated DOM host, selection and undo/redo.
Equal Rust commits preserve its live document; item switch or programmatic Reset
starts fresh history. An app-local reset token also clears history when Reset
changes only another field and the description text stays equal. Saving makes only the pending item's editor read-only;
another item's draft remains editable. Local typing never posts to the server.

The description remains plain text, bounded to 4,000 Unicode characters. An
over-limit editor transaction is rejected without truncation and displays a limit
hint. The Rust editor message independently checks length, selected item and
pending state. Existing server authorization, CSRF, validation and version checks
remain authoritative. Keyboard bindings are the package's history/default maps;
the Linux browser proof uses Ctrl-Z and Ctrl-Y.

The private TS factory exposes connect/update/destroy. Rust retains its JS handle
and owned Closure with the graph; the callback captures a weak graph handle and
checks runtime lifetime before sending an item-ID/text message. Programmatic
updates suppress echo. Destruction clears the JS callback before releasing the
Rust Closure. Package/startup failure or update failure retains/restores the native
textarea, which the Rust renderer keeps synchronized before calling the editor.
JS-disabled users retain the existing native item/editor links.

Changed application surfaces:

- [description-editor.ts](../examples/admin-dashboard/frontend/description-editor.ts)
  owns the package instance, style check, bounds, history and fallback.
- [triage.ts](../examples/admin-dashboard/frontend/triage.ts) and
  [app.ts](../examples/admin-dashboard/frontend/app.ts) reuse the per-start token
  and acyclic entry-owned lazy imports, including retirement during editor load.
- [browser.rs](../examples/board-triage/src/browser.rs) owns the adapter/Closure;
  [state/reducer tests](../examples/board-triage/src/lib.rs) cover selected-item,
  length and pending-message guards.
- [triage SSR](../examples/lab-operations-board/src/triage.rs), Board
  [security middleware](../examples/lab-operations-board/src/lib.rs) and
  [HTTP tests](../examples/lab-operations-board/src/tests.rs) supply and verify the
  response-local policy and fallback markup.
- [app.css](../examples/admin-dashboard/frontend/app.css), package.json and
  package-lock.json provide the existing theme and locked frontend delivery.

No core widget crate, registry, renderer, transport, settings schema, CLI option
or Rust dependency was added.

## CSP and native fallback

Each enhanced triage HTML response generates 16 random bytes with the existing
`getrandom` dependency, encoded as 32 hexadecimal characters. A private response
extension carries that same value to the final security middleware. The HTML
includes a nonce-bearing meta carrier; JS reads `.nonce`, outside application
state/settings. Only the selected page changes its style directives to:

```text
style-src 'self' 'nonce-<fresh-response-value>'; style-src-attr 'none'
```

The script policy remains `script-src 'self' 'wasm-unsafe-eval'`; other directives
are preserved. Ordinary SSR pages remain script-free with the default CSP and
no nonce carrier. Existing no-store HTML behavior is retained. Random-source
failure emits no carrier and retains the prior policy, leaving native behavior.

The adapter requires the expected nonce shape, passes it to `EditorView.cspNonce`
and confirms an admitted stylesheet plus the editor's applied base positioning
before hiding the native control. Missing or incorrect nonce cannot silently
replace the textarea with an unstyled editor. The shared document stylesheet
may remain after one editor is destroyed; view cleanup must not remove another
instance's styles.

The app's CSS overrides live outside Tailwind's component layer because
CodeMirror injects unlayered base CSS. This preserves the existing typography,
field height, bounded scrolling and focus treatment without changing CSP.

## Focused verification

Ubuntu Linux x86_64, Chrome 155.0.8059.84, existing Playwright/system Chrome;
Browser plugin unavailable. Temporary evidence:
`/tmp/yetlet-yt106-editor-20261020-6rqc0j3q/`. Scripts, screenshots and synthetic
SQLite fixtures remain outside Git. Startup login codes stayed in process memory;
no private secret or IdP configuration was used or recorded.

| Command / surface | Result |
| --- | --- |
| `cargo test --offline --locked -p yetlet-board-triage -p yetlet-lab-operations-board` | PASS: 5 triage + 19 Board tests, including editor-message bounds/identity/pending, header/carrier match, fresh response nonce and no-nonce fallback |
| `cargo check --offline --locked --target wasm32-unknown-unknown -p yetlet-board-triage` | PASS |
| `cargo clippy --offline --locked -p yetlet-board-triage -p yetlet-lab-operations-board --all-targets -- -D warnings` | PASS |
| `cargo clippy --offline --locked --target wasm32-unknown-unknown -p yetlet-board-triage -- -D warnings` | PASS |
| `APP_ENVIRONMENT=Production APP_OIDC__ENABLED=false APP_SESSION__BACKEND=memory tools/yetlet-cli/target/debug/yetlet build --manifest-path examples/admin-dashboard/Cargo.toml` | PASS: existing settings build WASM/bindgen/Vite then native app |
| `npm run typecheck --prefix examples/admin-dashboard/frontend` | PASS after generating bindings |
| `cargo test --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --all-targets` | PASS: 17 tests, including actual asset manifest/static-route checks |
| `cargo clippy --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --all-targets -- -D warnings` | PASS |
| `rustfmt --edition 2024 --check` on the five changed Rust source files; `git diff --check` | PASS |
| `cargo xtask check docs` | PASS: 75 documents, 106 backlog items, links/Markdown/whitespace/DAG |

The existing Admin browser journey was adapted in temporary `admin-browser.mjs`.
`browser-result.json` records nine passing groups:

1. Real editor and nonce stylesheet mount; typing/undo/redo update the native/Rust
   mirror; equal commits retain selection. Filter/reorder retain editor/chart
   identity. Switch/Reset preserve the chosen drafts and clear history. Local
   interactions send zero POSTs.
2. A 4,000-character emoji description is accepted; another character is rejected
   with visible feedback, without truncating the draft. Reset restores text.
3. One explicit Save carries editor text and returns 200 once. Pending read-only
   state does not block another draft; actual 409/422 responses preserve text.
4. Long text scrolls inside the bounded editor. A synthetic copy event with null
   clipboardData exercises the package's offscreen fallback textarea, applied
   CSSOM positioning and delayed cleanup. This is not OS clipboard qualification.
5. Desktop 1440x1000 and mobile 390x844 fit and remain editable; screenshots
   `desktop-editor.png` and `mobile-editor.png` were visually inspected. Pagehide
   removes the view/chart and restores the readable native field. A stale DOM
   input event is inert; reload mounts one editor with a new nonce.
6. Reset also clears history after a Save when only another field was changed
   and Description text itself stays equal.
7. JS-off native links work. Aborted package load, missing carrier or incorrect
   nonce leaves a usable Rust-bound textarea; partial editor DOM is removed.
8. A delayed editor import resolving after pagehide cannot create an editor,
   mount Rust or retain the previously created chart.
9. Ordinary pages retain the default CSP, no module script and no nonce carrier.

Normal editor operation has zero CSP violations/warnings/failed asset requests.
The two expected normal console resource errors are the tested HTTP 409/422.
The isolated package-abort and wrong-nonce cases record their one intentional
resource error/CSP violation respectively; no unexplained error was suppressed.
Earlier development lint/test-assumption failures remain separately recorded;
assertions wait for observed editor state, use the declared redo binding and
compare logical rendered lines rather than multiline DOM innerText formatting.

The actual app ran through **`yetlet run`**, with an isolated loopback port,
fresh synthetic SQLite, Production profile, memory sessions and OIDC disabled.
SIGINT through the CLI stopped the process; the app's drain message and closed
port were verified. Native outcome was `{ code: null, signal: "SIGINT" }`, not
exit 0. No preview remains running.

## Locked package cost

Direct versions: `@codemirror/view` 6.43.15, `@codemirror/state` 6.7.6 and
`@codemirror/commands` 6.11.1. The lock adds 12 packages, all declaring MIT,
including style-mod 4.1.4 and the commands module's language/Lezer dependencies.
No grammar, worker, rich-text layer or external CDN was selected.

Same-host artifact measurements use Python `gzip.compress(..., mtime=0)` at
default level 9. `baseline-assets.json` and `final-assets.json` retain per-file
raw/gzip sizes and SHA-256 identities. These are not HTTP-transfer measurements
or profiling claims.

| Category | Before raw / gzip bytes | After raw / gzip bytes | Delta raw / gzip bytes |
| --- | --- | --- | --- |
| JS, all chunks | 146,603 / 51,475 | 415,408 / 137,658 | +268,805 / +86,183 |
| CSS | 30,433 / 5,787 | 31,515 / 5,964 | +1,082 / +177 |
| WASM | 311,816 / 105,533 | 316,864 / 106,995 | +5,048 / +1,462 |

The lazy editor chunk is 267,887 raw / 85,922 gzip bytes. The existing chart chunk
is byte-identical. The manifest remains acyclic and all files pass the existing
2 MiB per-file / 16 MiB snapshot bounds. Module/WASM/CSS delivery remains through
the existing same-origin asset snapshot and CLI-selected Vite build.

## Limits and next decision

This is a bounded plain-text Admin integration on Ubuntu/Chrome, not a full
CodeMirror feature/browser qualification. Windows, real IME/assistive-technology
and OS clipboard matrices were not reopened. Reset/item switch intentionally
starts fresh undo history; JS-disabled users use native editor links. No new
preview, deployment, publishing or public framework adapter contract is selected.
After review, the next practical step is maintainer trial of this slice or a
separately selected YT-095 composition study; no further implementation is
authorized by this record.
