# YT-108 — Narrow SSR/DOM hidden attribute consistency

Date: 2026-10-21. Status: Done at the selected hidden-attribute scope.
Source baseline: 096f8db1e2c5b0f0fd1e664e776503cc21a74727, after PR #10.

## Selected scope and authority

The owner selected YT-108 after the [YT-107 closeout](YT-107-FORM-FEEDBACK.md),
then authorized implementation on the new feature branch. This is a separate
bounded authoring improvement, not a reopening of YT-107 or release qualification.
The original observation is recorded in the [API follow-up](YT-094-API-OBSERVATIONS.md#yt-107-formsave-follow-up-2026-10-21).

The existing HTML renderer accepts safe hidden attributes. The DOM renderer's
finite allowlist did not, requiring Todo to hide its initially empty field error
after mounting. Add only hidden to that allowlist; retain direct DOM creation,
complete View preflight and all existing tag/attribute restrictions.

## Contract

The [HTML Standard](https://html.spec.whatwg.org/multipage/interaction.html#the-hidden-attribute)
defines hidden as an enumerated attribute, not a string-to-boolean conversion.
For the selected ordinary visibility workflow:

| Shared View / DOM operation | Meaning |
| --- | --- |
| Omit hidden / remove_attribute("hidden") | Do not request hiding through this attribute |
| attribute("hidden", "") / set_attribute("hidden", "") | Request the ordinary Hidden state |
| attribute("hidden", "hidden") | The same ordinary Hidden state |
| attribute("hidden", "false") | Still Hidden; the string is not a visibility toggle |

Both renderers preserve attribute values: SSR escapes them and DOM uses
set_attribute rather than innerHTML. The until-found keyword remains a native
browser value, not a new framework reveal lifecycle. Find-in-page/beforematch,
CSS overrides and broader platform support are not qualified by this slice.
Hidden is presentation, never authorization or secret isolation.

Todo now declares its empty remote-title-error node hidden in browser_page and
removes only the redundant initialization-only title_feedback call. The actual
validation/edit/success bindings still set/remove hidden and update safe text and
ARIA. Recovery-button visibility, pending/lifetime behavior and Admin are unchanged.
No attribute bag, general forms runtime, VDOM, new dependency, public helper,
transport, release activation or global SSR/DOM policy unification is selected.

## Focused verification plan

Existing repository contracts require safe rendering, strict CSP, real evidence
before Done and independent review. Use existing focused commands and smoke seams:

- DOM accepted hidden values and rejection of executable/style/URL or arbitrary
  attributes even on hidden elements; nested unsafe tags still fail preflight.
- SSR absent/present/raw-value/escaping cases, complete Todo View validation and
  the initial empty error node's serialized hidden state.
- Native package tests, changed-package strict Clippy, relevant WASM compilation,
  formatting, docs and whitespace checks.
- Existing release-profile Chrome Todo/Search smoke with initial hidden state,
  actual HTTP 422 showing the error, input correction hiding it again, preserved
  HTTP revision/lifetime checks and owned-resource cleanup.

Reuse accepted Windows, Firefox, IME/accessibility and YT-107 evidence; do not
repeat unrelated suites. Keep YT-096's ce56e08 candidate and YT-020 gates unchanged.

## Verification record

Executed on 2026-10-21 with Rust 1.99.0 in the existing Ubuntu checkout. The
source remains based on the PR #10 squash commit above, with this bounded
uncommitted change. All 4,278 pre-existing untracked paths are preserved; the
path-list digest matches the before-work observation.

| Check | Actual result |
| --- | --- |
| cargo test --offline --locked -p yetlet-dom -p yetlet-html -p yetlet-todo-ui -p yetlet-todo-browser -p yetlet-todo-server --all-targets | PASS: 34 tests; 4 DOM, 5 HTML, 11 shared Todo, 4 Todo browser, 8 server unit and 2 server integration |
| Strict Clippy for those native packages and targets | PASS with -D warnings |
| cargo check --offline --locked --target wasm32-unknown-unknown -p yetlet-dom -p yetlet-todo-browser | PASS |
| Strict Clippy for the same WASM targets | PASS with -D warnings |
| cargo xtask build browser --release | PASS: asset digest 0a5ebb65f2918634; Todo and Search entries built separately |
| cargo build --offline --locked --release -p yetlet-todo-server | PASS |
| cargo xtask smoke browser-chrome --profile release | PASS: actual hidden/show/hide assertions plus the existing Todo/Search, exact HTTP/revision, keyed identity and queued/in-flight lifetime checks |
| Root and xtask formatting; xtask strict Clippy | PASS |
| cargo xtask check docs; git diff --check | PASS: documentation/link/structure/whitespace and 108-item DAG checks |

The actual browser ran once. Its receipt is
`target/m12/yt085/browser-chrome-Gmjpil/result.json`. Adjacent Chrome/server exit
receipts confirm closed listeners and streams, reaped owned trees and removal of
the browser profile, with no cleanup error. The server stopped via its existing
SIGTERM smoke teardown; Chrome exited 0. No application preview remains running.

The first build attempt stopped before any browser/server launch because the
Runner user lacked wasm-bindgen. Its failed log is retained. The exact locked
wasm-bindgen-cli 0.2.129 was then built offline from the existing Cargo cache into
`/tmp/yetlet-yt108-20261021-saGPav/tools`, selected only through that command's PATH.
No global installation, repository dependency/lock update or guard weakening was
needed. The temporary tool build reported existing upstream future-compatibility
warnings for buf_redux and multipart; it completed successfully and these are not
new application dependencies. Focused logs remain in that temporary directory.

## Independent review

Independent Reviewer Job `wc_job_0a2iPQ326nNUvRDi` completed with process exit 0
and the exact role output: overall_status pass, no issues, no required_followups
and no optional_notes. It inspected all eight changed/new targets, adjacent Todo
bindings, native/WASM/build logs and the browser/cleanup receipts without rerunning
passed workloads or changing the workspace.

The healthy/current/eligible/trusted workspace profile selected gpt-6-astra.
The shared resolver selected adaptive high for ordinary ad-hoc review; runtime
telemetry observed that same model and effort, and re-resolution found no conflict
or degradation. This is an independent role-directed Codex Job result, not native
agent_type identity, formal assurance or a new release qualification.

YT-108 is Done at the selected bounded scope. Dynamic visibility, native browser
value behavior and the broader unsupported/accepted platform boundaries remain
explicit. No commit, push, release, deployment or unrelated retest was performed.
