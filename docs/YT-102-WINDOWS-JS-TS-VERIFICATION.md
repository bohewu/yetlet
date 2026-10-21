# YT-102 — Windows JavaScript and TypeScript update boundaries

Historical pre-implementation evidence: the later owner-authorized
[state-preserving HMR implementation](YT-102-STATEFUL-HMR-IMPLEMENTATION.md)
adds module acceptance/disposal and fixed JavaScript routes, with fresh Windows
browser evidence. The failures and recovery limitation below remain unchanged
as observations of the earlier source.

Date: 2026-10-18 (Asia/Taipei).

## Result and authority

The owner requested Windows JS/TS HMR verification through WebCodex, followed
by independent Reviewer review. The current generated application's observed
TypeScript update behavior is **automatic full-document reload with state reset**.
State-preserving JS/TS module replacement is not qualified. A disposable plain
JavaScript module path is refused by the existing bounded development bridge.
The earlier [CSS HMR result](YT-102-WINDOWS-HMR-VERIFICATION.md) remains separate.

Local checkout: `$REPO`, branch `feat/yt090-configuration`,
HEAD `a883469862c5be3d88b7fe22262e10be8b53b92a` plus the preserved dirty tree.
Evidence root: `artifacts/windows-js-ts-hmr-20261018/`.

## Native target and unchanged inputs

WebCodex project: `agent:ncut-bohe-c420c5de1cac4bd4:yetlet-360e2631`.
Session: `wc_sess_ol95jcfM1xSqJN8i`. Windows checkout remains at
`3128d23591feee15ce976d7b83a56dcd587048d7`, artifacts-only dirty state.
The check reuses the exact generated Plain Vite application and prebuilt actual
Rust SSR executable from the CSS task. All 36 prepared application file hashes
and the backend executable hash were verified before interaction. Vite8.3.4 and
Node24.18.0 remain the already prepared native inputs; no installation or rebuild
is needed for these source-only edits.

Native WebCodex browser tools capture snapshots, screenshots, console and network
events at 1424x805. The gateway does not expose the active engine/version. No
browser script evaluation or new hot-accept implementation is used.

The current app.ts/shared.ts/lazy.ts modules have no `import.meta.hot.accept`
or disposal boundary. shared.ts owns a local counter and event listener; lazy.ts
is dynamically imported on click. The documented finite bridge admits named
TypeScript paths and rejects `/lazy.js`; adding files/imports requires review of
that bounded profile. No contract or allowlist is expanded by this verification.

## Observed browser cases

| Case | Actual native result | Classification |
|---|---|---|
| Loaded dynamic lazy.ts edit | State1 resets to the SSR initial message; next click executes changed output `TypeScript dynamic update: 1`; a new Document request returns200 and Vite reconnects | TS automatic reload fallback observed |
| Independent static shared.ts edit | With lazy.ts already restored and unchanged, state1 resets, changed button text appears and the next click starts at1; a new Document returns200 | TS automatic reload fallback observed |
| Equivalent disposable lazy.js import | `/lazy.js` returns403; browser records the refused resource and unhandled import promise; no module result appears | Plain JS path unsupported; no JS HMR pass |
| Restore sources in the initial live session | Exact original source bytes are restored, but the initial browser still requests `/lazy.js` and receives403 | Automatic same-session recovery not established |
| Fresh servers and browser after restoration | Original `/lazy.ts` returns200; original button/result behavior recovers with `Lazy module loaded: 1` | Baseline recovery observed after restart |

Dynamic TS evidence uses diagnostic cursor39 to74. The independent static-only
edit uses109 to144; its source receipt verifies lazy.ts was unchanged. The
intermediate74 to109 delta is combined setup/restoration and is not attributed
to one file. The JS probe is144 to179; the retained unsuccessful same-session
restoration is179 to214. Recovery uses a new browser and its own cursor scope.
Full bounded diagnostic receipts and fresh snapshots are in browser-evidence.json.
Mutation receipts that hash `read_text()` input describe newline-normalized source
text; after-write hashes and all final preservation manifests hash actual bytes.

The Vite server log records `page reload lazy.ts` and `page reload shared.ts`.
The TS edit deltas have no new console errors or failed requests; expected
JavaScript403/import errors and the baseline favicon404 are retained separately.
The stale import after restoration was not isolated to a specific cache/watcher
cause. Restart resolving this observed case does not prove restart is always
required. No implementation repair follows from this unsupported-path probe.

Six screenshots were captured and actually inspected: baseline.png, dynamic-ts.png,
static-ts.png, js-denied.png, restored.png and recovered.png. restored.png records
the unsuccessful initial-tab recovery; recovered.png records the successful
fresh-process check. The two observations are not merged into one passing result.

## Ownership, cleanup and preservation

Initial ownership Job `wc_job_TdYEKJIBzFGiyVBy` and the bounded fresh-process check
`wc_job_PGwYFuP2-q9pWqzA` both completed exit0. These are ownership/check exits,
not a declaration that state-preserving JS/TS hot replacement passed. Each owned
browser was explicitly closed and its native inventory returned zero. Each exact
Rust/Vite process was terminated and waited; Windows termination exit1 is kept
separate from the successful wrapper exits. Ports3000/5173 stopped responding.

All original TS sources are restored and the temporary JS file is removed.
All 36 prepared application inputs, Cargo.lock, frontend dist, backend executable
and 393 implementation files retain their hashes. Windows checkout HEAD/status
is unchanged. The prior transferred CLI README differs from current local
documentation only; runtime/test/template correspondence is retained.

A diagnostic log-print wrapper hit Windows CP950 encoding after its source
preservation assertions passed. This operational output failure is recorded in
browser-evidence.json; later UTF-8 log inspection succeeds. It does not replace
the native browser observations or justify a runtime change.

Returned native receipts are in windows-results/. Transfer ZIP SHA-256:
`706e89cde91d4c61c3679490427c23a39843ee297ebe31a5ace7f9a03a894276`.
Independent Reviewer: **PASS**, zero issues and zero required follow-ups.
This verdict covers the soundness of the bounded verification record, including
the unsupported features and unsuccessful same-session recovery. It does not
promote state-preserving JS/TS HMR to a passing feature. The reviewer independently
inspected all six screenshots, native browser diagnostics, source/template hashes,
ZIP hash, cleanup receipts, terminal Job envelopes and documentation check result.
Role, configured model and effective effort match the child trace. Evidence is in
review-result.json, review-trace.json, initial-job-terminal.json,
recovery-job-terminal.json, docs-check.json and local-preservation.json.
The docs check passes75 documents/103 backlog items; whitespace checks pass.
Local HEAD/index and all 100 CLI file hashes remain unchanged.

## Remaining scope

State-preserving JS/TS module acceptance/disposal, broader JS source admission,
other browser engines, Tailwind-specific behavior and Rust automatic restart are
not established. This task changes verification documentation only; no runtime
or template behavior, release or deployment is activated. YT-102's broader DX
research remains the next unblocked backlog work; YT-020 remains Blocked.
Only this record, the documentation index and the CSS record's follow-up link
change repository files. No commit, push, publication or deployment was performed.
