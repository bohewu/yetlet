# YT-102 — State-preserving JavaScript and TypeScript HMR

Date: 2026-10-18 (Asia/Taipei).

## Current CLI boundary and source-root follow-up

The owner clarified that CLI acceptance is configured **run, stop/cleanup and
build**. Vite owns compilation, dependency tracking and HMR; Yetlet does not
qualify Vite's complete feature set. Frontend JS/TS files now use normal imports,
custom names and subdirectories without registration. Physical root checks keep
private/tooling files and outside links unavailable. This replaces the initial
fixed-name restriction described in the historical evidence below.

On this Linux host (Node24.21.0, Chrome155.0.8059.84, Rust/Cargo1.99.0), direct
`yetlet build` exited0, `yetlet run` served Rust3000 and Vite5173 with HTTP200,
and SIGINT stopped both owned trees with the native CLI exit-2. Both ports were
closed and no owned processes remained. The already-started browser integration
check also passed, including live-added nested TS/JS files; it is supplementary
integration evidence, not a new Vite acceptance gate. No further Windows browser
matrix was run after the owner narrowed the scope.

Changed code: `templates/vite/vite.dev.bridge.mjs` under `tools/yetlet-cli` and
its two existing Node/browser tests. The browser check now resolves selected
`frontend/` layouts as well as legacy flat apps. Focused bridge tests passed9/9.
Evidence: `artifacts/js-ts-hmr-implementation-20261018/source-root-followup/`,
especially `cli-smoke-result.json`, build/run logs and `linux-browser/result.json`.
The focused independent reviewer follow-up returned PASS with zero issues and
no required followups; its result is retained in `source-root-followup/review.json`.

## Initial implementation and evidence scope

The following sections preserve the initial implementation and its evidence;
the current source-path policy and CLI acceptance boundary are stated above.

The owner explicitly requested JS/TS HMR with interaction state retained,
verification in a Windows browser through WebCodex, and independent review.
This implements the previously missing behavior; the
[earlier negative verification](YT-102-WINDOWS-JS-TS-VERIFICATION.md) remains
historical evidence, including its unresolved same-session recovery observation.

The generated `app.ts` owns the counter, self-accepts updates and stores state in
Vite `hot.data` before disposal. It accepts `shared.ts` updates and removes the old
listener before remounting. `shared.ts` accepts lazy updates and uses the latest
handler on the next click. Existing interaction DOM is retained. Disposed mounts
ignore pending async clicks and stale callbacks, preventing late writes and
duplicate listeners. This is the generated application's small state boundary,
not a general state migration or reactive component system.

The bridge adds only `/app.js`, `/shared.js`, `/lazy.js` and `/companion.js` beside
the existing TypeScript paths. Host/origin/query/path/read-only checks remain in
place. TypeScript remains the default and the dev document still loads `app.ts`.
The tsconfig admits JavaScript dependencies. No CLI option or settings schema
was added; existing settings still control frontend startup/build independently.

Product changes: `tools/yetlet-cli/templates/vite/{app.ts,shared.ts,tsconfig.json,vite.dev.bridge.mjs}`.
Focused regressions: `tools/yetlet-cli/tests/{vite-dev.mjs,vite-hmr-browser.mjs}`.
The existing browser check gains optional `--module-hmr` for two edit/restore
rounds and thirteen clicks, preserving its existing flat-layout Linux setup.
It was syntax checked here; the actual Windows checks below use native WebCodex
browser operations rather than that Linux browser launcher.

## Native Windows evidence

Local branch `feat/yt090-configuration`, HEAD
`a883469862c5be3d88b7fe22262e10be8b53b92a`, plus the preserved dirty checkout.
WebCodex project `agent:ncut-bohe-c420c5de1cac4bd4:yetlet-360e2631`, session
`wc_sess_ol95jcfM1xSqJN8i`. Windows checkout remains at
`3128d23591feee15ce976d7b83a56dcd587048d7` with artifacts-only dirtiness.

Evidence root: `artifacts/js-ts-hmr-implementation-20261018/`.
The 393-file current-source archive SHA256 is
`152a03ba29d59c9fd12e9a72da643b626f3d0f26b2a0da388593e167c8a1527c`.
All extracted source hashes were checked. The native CLI was rebuilt from that
archive, generated `win-stateful --vite`, and built its actual Rust backend plus
production Vite snapshot. Only three declared framework dependencies received
explicit local Cargo paths while retaining their pinned versions. Existing
Node24.18.0, Vite8.3.4 and Rust/Cargo1.99.0 GNU were used without installation.
Documentation was updated after the native archive; runtime and test sources
remain source-matched.

The JavaScript case was prepared before fresh server/browser startup: the pinned
TypeScript compiler emitted equivalent code, `shared.js` and `lazy.js` used
explicit imports, and `app.ts` retained the JS entry body. This proves JavaScript
dependencies under the generated entry, not an alternate JS-only entry selector.
Its production snapshot was rebuilt explicitly before starting Rust.

| Native browser case | Observed result |
| --- | --- |
| Default TS modules | Baseline count1; two rounds of lazy/shared/entry edits and exact source restores; thirteen clicks finish at13 |
| JS shared/lazy dependencies | Both routes return200; equivalent two-round edits/restores finish at13 |
| New code executes | Lazy output changes on next click; shared button and entry heading markers change during HMR |
| State and listeners | Every update keeps the previous result; every click adds exactly one across all replacements |
| Network and console deltas | Twelve acknowledged hot updates per case, zero new Document requests, zero HMR console errors, failed requests or HTTP4xx/5xx |
| Baseline diagnostic | Each fresh page has the existing favicon404/resource error; it is separate from update deltas |
| Cleanup and preservation | Both owned browsers closed; both Rust/Vite pairs reaped; ports3000/5173 closed; all36 prepared app file hashes and393 source hashes restored/unchanged |

The native browser gateway records snapshots, network and console events and
1424x805 screenshots but does not expose its active engine/version. Servers used
the exact prebuilt backend and local Vite binary under an owned lifetime script;
the separate [Windows CLI pair qualification](YT-102-WINDOWS-VITE-VERIFICATION.md)
remains the evidence for CLI console supervision. Both lifetime Jobs exited0;
their explicitly terminated Windows child processes exited1, as recorded.

Raw evidence includes `browser-evidence.json` (TS), `browser-js-evidence.json`,
`browser-summary.json`, three screenshots, `build-terminal.json`,
`ts-terminal.json`, `js-terminal.json`, and `windows-results/` native logs,
manifests, edit hashes, binary hashes and preservation records. The returned ZIP
SHA256 is `1c54277fff93e5f22fa2471787ae508d11c5b478365294fe83981c2bfe237a14`.
An initial preparation script expected the wrong Cargo declaration shape after
successful CLI build/generation; its exit1 receipt is retained. One script-only
correction continued the same prepared application and completed builds at0.

## Local validation and remaining boundaries

- `cargo test --manifest-path tools/yetlet-cli/Cargo.toml --offline --locked --all-targets`: exit0; the existing explicit integration ignores remain.
- `node --test tools/yetlet-cli/tests/vite-dev.mjs`: eight passed, including fixed JS route admission and refusal boundaries.
- Pinned TypeScript emit and equivalent JS-dependency typecheck: exit0. The executor also checked the default TypeScript production build and mount/disposal lifecycle.
- `node --check tools/yetlet-cli/tests/vite-hmr-browser.mjs`, `cargo xtask check docs` and `git diff --check`: exit0; docs check reports75 documents and103 backlog items.

Arbitrary/nested source paths, incompatible state schema migration, replaying
clicks pending during disposal, a standalone JS entry selector, Rust hot restart
and production HMR are outside this change. YT-102's broader DX research remains
Todo; YT-103 rolling asset retention is the next separate backlog item. No commit,
push, global installation, registry publication or deployment was performed.

## Independent review

Independent reviewer `/root/stateful_hmr_final_review` returned **PASS**, with
zero issues and no required followups. It independently checked both archive
hashes, all393 source entries, the scoped file changes, all25 browser records per
case, both count13 screenshots, continuous diagnostic cursors, native exit
receipts, and restoration/cleanup evidence. The result is retained in
`review.json`. Registered reviewer/model/effective effort matched the dispatch
trace; the observed resolver decision is retained with it. After review, only
this result and validation bookkeeping were added; product/test sources stayed
unchanged.
