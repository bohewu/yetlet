# YT-094 — Windows Rust/WASM follow-up

Date: 2026-10-19. Runtime candidate: `a402b6f`.
**Windows native and browser: PASS at the scope below. Returned native IME,
keyboard and spoken-AT journeys: PASS by operator attestation, with the evidence
limits below. Physical stop: drain and numeric exit 0 corroborated by screenshots.
YT-094 remains In progress.**
No application source changed, and no commit, push or deployment occurred in this
verification continuation. The earlier [Windows SSR result](YT-094-WINDOWS-VERIFICATION.md)
and [Linux WASM result](YT-094-WASM-TRIAGE.md) remain separate evidence.

## Environment and source boundary

The native checks used an isolated WebCodex project and resumed session;
the task directory is recorded in the local execution receipts.
Windows 10.0.26300 x86-64, Rust/Cargo 1.99.0 MSVC, Node 24.18.0.
The existing protected CLI retains SHA-256
`0b48c547e8549ea9de540b0b4f37caa3b14fd9ec050679c85c031d85876baad2`.
Its source did not change between the previously qualified baseline and this
candidate. Native server startup used that CLI's `run` command throughout.

Source preparation copied the previously reviewed `4bb8f08` Windows source,
then applied the 20 changed application-slice files from `a402b6f` through
revision-fenced file edits. All 20 SHA-256 values (`../artifacts/yt094-windows-wasm-20261019/runtime-source-hashes.json`, local evidence) matched the committed bytes;
the existing embedded Board assets were copied unchanged. This is a bounded
runtime-source comparison, not a claim that the complete Git tree was recreated.
An optional wider inventory encountered Git license symlink blobs represented as
materialized Windows files; that comparison is not acceptance evidence.

Dependencies were prepared separately with locked Cargo fetch and offline npm
installation. The existing CLI trust rule initially rejected this task's newly
created directory because of inherited writable ACLs. Only this new directory's
DACL was tightened to its owner, SYSTEM and Administrators. Existing global Cargo,
other projects, unrelated windows and executable trust rules were preserved.
The restored baseline assets corrected a fixture copy omission, not a product
source failure. Earlier failed preparation observations remain in the Session.

The application-required `wasm32-unknown-unknown` target was installed temporarily
and removed after validation; the installed-target list returned to native MSVC.
The matching wasm-bindgen 0.2.129 Windows binary was kept inside the task tools
directory. Its [official release](https://github.com/wasm-bindgen/wasm-bindgen/releases/tag/0.2.129)
archive matched the published GitHub asset digest
`79e348c169d0c10c0b287647f1e753d0985353ca96800bc29be8f4261baf13e6`.
No global bindgen installation was added.

## Completed native checks

| Operation | Observed result |
| --- | --- |
| `yetlet build --manifest-path <source>/examples/admin-dashboard/Cargo.toml` | Exit 0; browser Rust/WASM, matching bindgen, Vite and native application built on Windows |
| Frontend `npm run typecheck` | Exit 0 |
| Admin `cargo test --offline --locked --all-targets` | 12 passed, 0 failed; the Unix-only filesystem test is not applicable |
| Board and triage `cargo test --offline --locked --all-targets` | 17 Board and 3 reducer tests passed, 0 failed |
| Focused Admin and Board/triage native Clippy with `-D warnings` | Both exited 0 |
| Two CLI `run` instances in their own attached native consoles | Actual listener and application startup observed |
| Two `CTRL_C_EVENT` stops | Both native CLI exits 0; admitted work drained, controller Jobs completed, port 3018 closed |
| Restart | Previous session rejected and terminal codes replaced; record version 4 and its 4 history entries preserved |
| `APP_CLI__VITE__AUTO_BUILD=false yetlet build ...` | Exit 0; selected frontend build skipped |

The control event was sent to the exact owned console using the existing Windows
console-event pattern. It verifies native console cancellation, not a physical
keyboard or OS-IME interaction. Controllers remained alive under their original
Runner Jobs while their CLI child ran; the earlier short-lived launcher did not
retain its child and is not passing lifecycle evidence.

## Completed browser checks

The newly callable WebCodex Browser gateway drove its isolated Windows browser
at a 1409×805 viewport. The actual built module and WASM returned 200 and mounted
the workspace. Observed editing, item switching and keyed reverse retained the
first item's title/description draft. Diagnostics showed no HTTP during these
local operations. A title filter reduced the queue to one matching item.

One Save selected action produced one JSON POST with status 200 and advanced the
selected item from version 1 to 2. A separate authorized writer then saved version
3; submitting the stale browser draft returned 409 and retained that exact draft.
Reload deliberately discarded the local draft and showed the authoritative
version. Closing the item saved version 4; Closed-to-Active returned 422 with the
Active draft retained. A read-only SQLite observation after restart confirmed
version 4/Closed, the authoritative title and exactly 4 history entries. Neither
rejected submission mutated the record.

All selected CSS/JS/WASM requests succeeded. WASM was served as
`application/wasm`, with nosniff; WASM compilation permission appeared only on
the enhanced triage response. Ordinary SSR remained script-free with the default
CSP. The only captured browser console errors correspond to intentional 409,
422 and retired-session 401 responses; no unexpected transport or CSP failure
was observed. Windows workspace screenshot (`../artifacts/yt094-windows-wasm-20261019/workspace.png`, local evidence).
This does not add a Windows mobile-device, other-browser or performance claim.

## Remaining desktop gates and cleanup

The Windows desktop showed other interaction and newly opened unrelated windows
while native desktop setup was being inspected. The owner was asked for a short
exclusive desktop slot before focus-taking IME/Narrator input. No answer had
arrived when this bounded evidence was prepared. OS composition and screen-reader
speech were therefore not tested or inferred from DOM/accessibility metadata.
Narrator 10.0.26100.8972 is installed and was not running; it was not started.

The desktop gateway also exposed limits: its key vocabulary rejected an arbitrary
letter key, and taskbar/Edge chrome did not support its InvokePattern operation.
No raw native-input or profile-attachment fallback was used to bypass those
limits. A normal Edge home window opened during desktop setup was not used for
qualification or signed into the application; it was left alone once concurrent
desktop activity appeared. The isolated automation browser was closed explicitly.

Both native controller Jobs reached exit 0. The listener, disposable database and
private startup log were removed; redacted runtime logs, public source/build
artifacts and sanitized results were retained in the task directory. Existing
repository edits, including YT-103 and the uncommitted YT-106 registration, remain
untouched. The local result (`../artifacts/yt094-windows-wasm-20261019/result.json`, local evidence)
contains captured native results, browser diagnostics, scope and cleanup limits.
Independent reviewer review passed for the completed Windows native/browser
evidence, with no issues or required corrections. The reviewer independently
matched all 20 application-slice hashes and preserved both pending desktop cells;
this did not approve YT-094 as complete. Its then-pending desktop journeys are
reconciled in the returned-evidence section below.

## Mint-game desktop follow-up and manual handoff

The owner subsequently authorized the desktop associated with mint-game.
Its prepared application
built and ran through a retained, previously qualified GNU CLI; a native console
control event then produced exit 0, drain and port 3019 closure. This did not
qualify the current CLI binary on that host. The source comparison covered 310
selected inputs, not the complete Git tree.

Native desktop automation did not complete sign-in, real OS composition or
spoken-AT observation. The owner requested a manual continuation. The
[Windows desktop handoff](YT-094-WINDOWS-DESKTOP-HANDOFF.md) records exact paths,
source/tool identities, process-local setup, commands, remaining journeys and
automation limits. Narrator was still running at that pre-handoff observation;
its blue cursor rectangles require the physical Windows+Ctrl+Enter shortcut to
disable it. The on-screen keyboard and test server were stopped. No application
source changed. These are historical preparation observations, superseded for
manual results and cleanup by the returned evidence below.

## Returned native Windows evidence — 2026-10-19

The owner returned FINAL-HANDOFF.md (`../artifacts/yt094-windows-desktop-20261019/FINAL-HANDOFF.md.txt`, local evidence)
from `C:\repos\yetlet\artifacts\yt094-mint-desktop-20261019`.
The chronological RETURN-HANDOFF.md (`../artifacts/yt094-windows-desktop-20261019/RETURN-HANDOFF.md.txt`, local evidence)
and original observations are retained unchanged. All 14 files listed in the
returned SHA256SUMS.txt (`../artifacts/yt094-windows-desktop-20261019/SHA256SUMS.txt`, local evidence)
were copied and matched those hashes locally, including both operator images.
The locally assembled archive (`../artifacts/yt094-windows-desktop-20261019/returned-evidence.zip`, local evidence)
preserves their original names and bytes plus the manifest. Raw handoff text
copies use `.md.txt` to preserve literal observed test data without making that
evidence repository Markdown documentation. The original optional archive was
not imported or verified; no archive-byte equivalence is claimed. Comparing
the 310 selected source hashes (`../artifacts/yt094-windows-desktop-20261019/source-hashes.json`, local evidence)
with live HEAD `a402b6f33525568ee330e18e0b6a04652c3734d5` produced 310 matches
and zero mismatches. This does not establish complete Git-tree or binary
reproducibility.

The returned host is Windows 11 Pro 10.0.26300, Edge 155.0.4283.62. It reused
the retained GNU CLI identified above and the prepared application; no current
CLI rebuild is claimed. The reported application executable SHA-256 is
`351a7aa1836d1954ee07d7aeb1fce5ea7c704691932aa9bddc0b3efbbdb30593`.
No passing workload was repeated, and no product change was needed for this
evidence reconciliation.

| Journey | Accepted result and evidence strength |
| --- | --- |
| Physical Microsoft Bopomofo composition in Title and Description | PASS by operator report, including candidate/composition UI. The agent did not observe the physical composition itself. |
| Reverse, switch items and return; Tab/Shift+Tab | PASS by operator report of retained text and usable focus/controls. |
| Narrator names, roles and save feedback | PASS by explicit operator attestation. No audio, Speech recap or verbatim transcript was supplied; this is not independently recorded speech. No repeat speech test is required solely to change that evidence type. |
| Explicit save/reload | PASS by operator report. The returned agent separately observed item 1 at version 2, both Title and Description equal to U+4F60 U+597D, zero unsaved drafts and disabled Save/Reset; the other loaded item was Switch sample. This corroborates the final page state, not the physical input or prior reload action. |
| Physical Ctrl-C | PASS by operator report, corroborated by the drain-marker image (`../artifacts/yt094-windows-desktop-20261019/manual-evidence/operator-stop.png`, local evidence) and original-console image (`../artifacts/yt094-windows-desktop-20261019/manual-evidence/operator-exit-code.png`, local evidence) showing `$LASTEXITCODE` equal to 0. These images do not independently capture the physical keystroke. |
| Cleanup | The returned final observation (`../artifacts/yt094-windows-desktop-20261019/manual-evidence/final-cleanup-observation.json`, local evidence) records no listener on 3019, no Yetlet, Narrator or on-screen-keyboard processes, and a closed test tab. The manual database and idle PowerShell console remain on Windows. Unrelated tabs were preserved. |

The launcher's `-NoExit` explains why the console remained open. Neither the open
console nor the failed optional SQLite probes establishes a product defect.
The latter omitted Python's URI-mode flag; they are historical probe failures,
not database-corruption evidence. No shutdown fix, database repair or restart
follows from them. The database, startup logs, login codes, cookies, executables,
tools and local role configuration were not copied into the evidence directory.

## Returned review and acceptance reconciliation

The exact Windows reviewer response (`../artifacts/yt094-windows-desktop-20261019/manual-evidence/reviewer-result.json`, local evidence)
remains **fail** for its then-missing numeric exit code. The later exit-code
image resolves that evidence gap; it does not retroactively change the child's
response into a reviewer-issued pass. Its execution summary (`../artifacts/yt094-windows-desktop-20261019/manual-evidence/reviewer-execution-summary.json`, local evidence)
records configured model `gpt-6-astra`, observed model `gpt-6.1-sol`, and matching
reviewer role/effective high effort. The model mismatch remains explicit;
Windows workspace model routing and the configured strong-tier execution were
not verified. The local Windows profile was not installed on this source host.

Separate source-side Independent Reviewer `/root/yt094_returned_evidence_review`
PASS (`../artifacts/yt094-windows-desktop-20261019/reconciliation-reviewer-result.json`, local evidence):
zero issues or required followups. The reviewer independently checked all 14
archived file hashes, the identical manifest, retained copies, all 310 source
inputs and both screenshots. The local child trace matched the registered
reviewer role, configured `gpt-6-astra` and effective high effort. This verifies
this source-side dispatch and reconciliation only; it neither repairs Windows
model routing nor changes its historical reviewer response. The
reconciliation summary (`../artifacts/yt094-windows-desktop-20261019/reconciliation.json`, local evidence)
retains the operation and evidence boundaries.

| Original YT-094 criterion | Reconciled evidence and remaining boundary |
| --- | --- |
| Tailwind/Vite, meaningful SSR, typed forms, validation, pagination and error/empty states | The [first slice](../examples/admin-dashboard/README.md#first-slice-local-evidence--2026-10-19) remains accepted; returned native input evidence does not replace its no-JS proof. |
| Responsive/mobile layout and local Rust/WASM interaction | The [Linux slice](YT-094-WASM-TRIAGE.md#local-verification) and Windows browser checks above remain accepted. Windows mobile devices and other browser versions were not qualified. |
| Role/resource authorization, CSRF/session, concurrency, persistence failure and audit history | Existing native tests and real SSR/browser rejection/restart cases remain accepted at their documented local scope. Synthetic identities do not establish external OIDC or production authentication. |
| Keyboard, CJK OS IME and spoken accessibility | The returned native journeys above supplement browser-protocol composition. Preserve operator-attested acceptance without promoting it to recorded speech or independent observation of physical input. |
| Bounded load and graceful cleanup | The first-slice 40-read/concurrency-8 workload, prior Windows lifecycle checks, returned drain/exit-0 images and cleanup remain accepted; none is a production capacity or multi-replica claim. |
| Independent review | Prior implementation/native/browser reviews remain accepted. The returned Windows review's missing-exit finding is resolved by later evidence, while its model mismatch and original fail remain preserved. Source-side reconciliation review is separate from that historical response. |

YT-094 remains **In progress**. External OIDC/production auth, production
TLS/deployment, durable/shared sessions and storage, and authenticated frontend
HMR remain explicitly unqualified. They require a separately selected scope
before implementation or external-environment changes; no local evidence gap
justifies repeating the passed native journeys. Optional HA/rolling asset
retention is separately tracked in YT-103, and browser delivery/authoring/widget
questions remain YT-104/YT-105/YT-106. YT-104 is an independently unblockable
backlog item; this reconciliation does not start it.

## Current evidence retention and owner-selected follow-up scope

Raw runner receipts, handoffs, archives and screenshots are retained as local
evidence under the recorded `artifacts/` paths; they are not part of the Git
delivery. This document preserves the accepted results, source identities,
historical failures and review boundaries without publishing machine-specific
runner metadata.

The later [OIDC slice](YT-094-OIDC.md) passes its real-provider Linux workflow.
The owner explicitly waived separate Windows OIDC verification after deleting
the dedicated test client and account. Neither is to be recreated for that test.
Previously passed Windows SSR/WASM, input and accessibility results remain
accepted. Future slices do not routinely repeat native IME/assistive-technology
checks; a relevant input/accessibility change may receive a focused Ubuntu
check. No broader platform matrix is a prerequisite for this reference closeout.
