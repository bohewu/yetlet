# M9g Verification — Multi-Island Dashboard

**Date:** 2026-10-10
**Task:** YT-056 / M9g
**Status:** Done. Browser fallback evidence retained; final reviewer `wc_job_OMstvGrtDiKZ5p6C` PASS and exact catalog entry promoted.

## Scope

`yetlet-multi-island-dashboard` serves complete native SSR for two panels. Optional companion crates `yetlet-dashboard-counter-island` and `yetlet-dashboard-filter-island` are separately compiled, bound and lazy imported. Their generated assets live under ignored `target/multi-island-dashboard-dist/`; the production Todo/Search `dist/` is not a build target for this sample.

Each island has a distinct loader, module, WASM file, boundary name, state component/version, listener ownership and explicit `mount`/`dispose` lifecycle. Counter guards delayed callbacks with a runtime lifetime token. Filter preserves its SSR input identity and focus. No common state, automatic hydration, browser RPC, global event transport or new Yetlet framework API was added.

## Manifest and SSR fallback

The server validates a content-addressed manifest and the exact existence of every referenced same-origin file independently for Counter and Filter. Missing/invalid whole manifests yield script-free SSR and the default strict CSP. A missing Counter WASM or invalid Filter loader URL omits only that loader while preserving both SSR panels. Six native server tests cover full fallback, separate loader injection, per-island degradation, static asset containment, loopback admission and native 404/405 behavior.

## Focused browser evidence

Debug Chrome and Firefox E2E use isolated local processes/profiles. They establish:

- neither WASM bundle loads when the SSR dashboard first completes;
- deterministic activation of Counter fetches only Counter module/WASM, not Filter;
- Filter activation fetches its own bundle later;
- Counter and Filter root/input/value node identities remain stable through local updates;
- Filter input focus is retained while its own rows update;
- disposing Counter before a delayed callback prevents that callback from changing retired DOM;
- detached controls do nothing, remount creates one effective listener, and current Counter state remains;
- an intentionally invalid Filter version fails only that island, preserves its SSR-owned DOM and does not stop Counter;
- restoring the version remounts Filter from the live input value;
- both owned server/browser/driver processes are reaped.

The initial browser harness tried to trigger lazy activation with programmatic focus. Headless Chrome did not reliably emit that synthetic focus path, so deterministic tests use the loader's documented local `yetlet-activate` event and then perform real Filter focus/input assertions after mount. One early query used `wasm`, which is not a contiguous substring of `WebAssembly`; it was corrected to `web`, while the dispose test sets `rust` and verifies detached listeners do not update before remount. A default favicon 404 was removed with a fixed native 204 route so unexpected HTTP failures remain a strict E2E assertion. Firefox's synchronous WebDriver wrapper required one remount statement to be wrapped as an expression. None of these harness repairs weakened runtime acceptance conditions.

## Acceptance commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --doc --workspace --locked
cargo check --locked --target wasm32-unknown-unknown \
  -p yetlet-dashboard-counter-island \
  -p yetlet-dashboard-filter-island
python3 scripts/build_multi_island_dashboard.py --release
cargo build --release --locked -p yetlet-multi-island-dashboard
python3 scripts/smoke_multi_island_dashboard.py --profile release
python3 scripts/smoke_multi_island_dashboard_firefox.py --profile release
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

Required full evidence includes exact release bundle sizes/digest, Chrome 155 and Firefox 157 lifecycle results, missing/invalid manifest tests, companion/production dependency isolation and byte-identical production `dist/`. Keep this sample outside the verified catalog until an independent read-only reviewer PASS. Do not begin YT-057 before YT-056 is Done.

## Complete validation freeze

The full release matrix passed after the focused debug checks. Retained logs, source/dist hashes, package trees, exact candidate catalog stanza, asset sizes and summary are under Git-ignored `target/m9g-evidence/`.

| Check | Observed result |
|---|---|
| Multi-Island server tests | 6/6 PASS |
| Workspace native tests | 169/169 PASS |
| Workspace doctests | 5/5 PASS |
| Workspace format and strict Clippy | PASS |
| Dashboard server release build | PASS; 2,635,496 raw bytes |
| Counter release bundle | 166,999 raw / 63,710 local-gzip bytes |
| Filter release bundle | 177,694 raw / 68,753 local-gzip bytes |
| Shared dashboard CSS | 416 raw / 279 local-gzip bytes |
| Independent dashboard asset digest | `1eccb326477b59ae` |
| Chrome release lifecycle/isolation E2E | PASS |
| Firefox 157 release lifecycle/isolation E2E | PASS |
| Eight wasm32 packages | PASS |
| Production Browser/Search release | digest `faebb0709dccd863` |
| Production `dist/` | 136/136 files byte-identical |
| Existing catalog/checker | 9 entries and 4/4 tests PASS |
| Candidate catalog | Exact virtual 10-entry catalog PASS |
| Dependency isolation | All 26 workspace package normal trees PASS |
| Documentation/backlog | 48 required documents / 59 task IDs PASS |
| Git/worktree/process hygiene | Original master HEAD, one worktree, empty index, whitespace and cleanup PASS |

Counter and Filter bundle totals each include only that island's loader, generated JS module and WASM. Gzip values are deterministic local estimates, not actual HTTP transfer measurements. Generated dashboard files remain under ignored `target/`; they neither overwrite nor enter production `dist/`.

The server package tree excludes both browser companion packages and all wasm-bindgen/web-sys dependencies. The two companion trees exclude the server adapter, Session, Request ID, SQLite/SQLx and ORMs. No pre-existing package depends on any of the three dashboard packages.

Browser E2E uses isolated profiles and verifies release assets. Production Todo/Search Chrome/Firefox E2E and renderer benchmarks were not repeated because their source and all production files remain unchanged; complete cross-product browser regression remains the M9j freeze.

## First independent reviewer finding and repair

Independent read-only reviewer Job `wc_job_63A__nJFK3BgNWPk` returned one P2 evidence gap. Native tests proved missing and per-island invalid manifests, but the original Chrome/Firefox scripts exercised only the valid generated manifest. The approved M9g acceptance explicitly required browser evidence for missing/invalid whole-manifest fallback.

Both existing browser smoke scripts now launch two additional temporary asset-root servers: one with no manifest and one with malformed JSON. In Chrome and Firefox they independently verify HTTP 200, exact `DEFAULT_CSP`, both readable SSR panels, count/list fallback content, zero script elements, no dashboard global and no loader/module/WASM resource requests. All fallback servers and browser/driver processes are reaped. Release Chrome and Firefox reruns PASS after this evidence repair. Runtime source and generated bundles were unchanged, so the previously passing full workspace, dependency, wasm32, asset-size and production byte-isolation matrix remains applicable.

## Second reviewer evidence follow-up

Repeat reviewer Job `wc_job_6pk1_UnsnH8WR4Ns` confirmed the repaired scripts matched the source freeze but could not accept the claim because the original retained `dashboard-*-release.stdout.log` files predated the fallback additions. It requested only retained execution output or exact accessible execution identifiers, not another runtime change or broader matrix.

The same release Chrome and Firefox scripts were rerun without source changes and their complete output was retained as:

- `target/m9g-evidence/fallback-repair-chrome.stdout.log` — exit 0, including `PASS Chrome: missing and malformed manifests retain strict script-free SSR`;
- `target/m9g-evidence/fallback-repair-firefox.stdout.log` — exit 0, including `PASS Firefox: missing and malformed manifests retain strict script-free SSR`;
- `target/m9g-evidence/fallback-repair-runs.json` — literal commands, exit codes, durations and log paths.

Both runs again executed the valid-bundle ownership/lifecycle scenario before their missing/malformed whole-manifest cases. Their stderr files are retained separately. This closes the requested evidence shape while leaving runtime, assets and catalog unchanged.

## Reviewer gate

The review sequence was intentionally strict:

1. `wc_job_63A__nJFK3BgNWPk` found the missing whole-manifest browser-evidence P2.
2. After Chrome/Firefox script repair, `wc_job_6pk1_UnsnH8WR4Ns` confirmed the source but required retained exit-0 execution logs.
3. After retaining those exact logs and run metadata, final independent reviewer `wc_job_OMstvGrtDiKZ5p6C` completed at terminal exit 0 with explicit `gpt-6-astra`, high reasoning effort and read-only sandbox. Final result:

```json
{"overall_status":"pass","issues":[],"required_followups":[],"optional_notes":[]}
```

All were ordinary local `codex exec <reviewer prompt>` role-directed Jobs, not `codex exec review`, native ACP/`agent_type` dispatch or formal assurance. Reviewers did not edit, stage, commit or push. Only after final PASS was the exact reviewed candidate stanza promoted to `examples/samples.toml`; runtime, loaders, WASM and tests were unchanged. YT-056 is locally Done and YT-057 remains Todo.
