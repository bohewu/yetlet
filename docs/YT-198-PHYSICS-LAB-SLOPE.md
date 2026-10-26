# YT-198 — Slope shape and friction comparison

Date:2026-10-26. Status:**Done after independent incremental Reviewer PASS,0issues/required followups.**

## Authority and minimum contract

The owner requests continuation with the slope scene. Preserve the accepted,
uncommitted13-path YT-197 slice and its receipts; no Commit/Push. This is an
application-owned opt-in preset, not a Core/renderer/pose API, dependency, editor,
material-settings or benchmarking feature. Read YT-197 for retained controls,
fixed60Hz/bounded scheduler,resource lifetime and estimated submission-FPS rules.

Add one closed scenario ID slope,label Slope comparison. Keep1–64 dynamic bodies,
five statics and the19-float pose layout. For slope only,the uphill wall slot
becomes a fixed25degree ramp; retain the landing floor,downhill wall and both side
walls. This is not a new hard-containment guarantee; existing finite/bounds,
last-valid-snapshot,invalid-world and explicit Reset/retry rules remain.

Four lanes repeat each row: blue low-friction box/ball,amber high-friction
box/ball. Body coefficients0.1/0.9,ramp0.2,explicit Rapier Average combine rule;
other scenes retain0.6. Objects share the same uphill start per row,unit mass,
zero initial velocity and equal normal clearance; boxes align with ramp rotation.
Use at least4 bodies for a complete comparison; lower counts are prefix cases.
A private Object friction scalar feeds colliders only; it never changes the pose
buffer layout. Use existing box/sphere rendering and Rapier contact/rotation.
No new algorithm or scientific/engineering equivalence guarantee.

Slope participates in native GET/SSR and the same staged selector: first Run
uses controls,Reset applies/pauses; selection alone leaves the active world.
An active-scene legend describes angle,color/materials and the four-body minimum.
It follows the actual world rather than a staged selection. Existing FPS semantics,
Run/Pause/Reset/Step,opt-in,no automatic resume/retry and budgets stay unchanged.
No new GUI action or adjustable angle/material field. Rebuild the same six-file
hashed asset bundle/server together; existing fallback policy remains.

## Bounded new verification

New yt198-filtered tests cover parsing/layout/bounds/normal clearance,actual slope
contact/rolling/sliding/friction contrast,collider coefficients and native SSR/
legend/selection. Do not rerun accepted YT-197 tests or the FPS helper tests.
Compile changed native/Wasm/assets with existing locked tools,then one scoped
actual browser flow: initial slope/legend/noWasm,Run and finite FPS,Pause,staged
selection with retained legend,Reset-away hides legend,Reset-back restores slope.
Keep prior failure receipts,clean owned browser/server/profile/compositor,check
docs/whitespace/source and asset bindings. Independent Reviewer before Done.


## New scoped evidence

The contract above preceded implementation. Scenario::Slope adds the same closed
ID to Rust/native selection and browser label validation. Object.friction is
private CPU material metadata; default0.6 preserves existing scene coefficients.
Slope alone configures explicit Average combining and replaces static slot1002
with the tilted ramp. Neither Object.floats(19),static cardinality,fixed-step
scheduler,private GPU renderer,frame-rate helper nor Wasm call signature changes.
The slope legend is present in SSR and follows activeScenario through the existing
HUD/control update seam,so a staged selector cannot misdescribe the current world.

| New verification | Result |
|---|---|
| `cargo test --offline --locked --manifest-path examples/physics-lab/Cargo.toml --features browser yt198` | 3passed,13filtered: finite/count/pose/clearance bounds through64bodies,actual contact/rotation/sliding/friction contrast,SSR selected option/legend/no-runtime fallback |
| Existing exact bindgen build-assets and default native server build,offline/locked | PASS; six-file bundlebd001dbe7a90692151c5d76d039327e73dab55d90c8706387a19184717be5704 |
| Actual isolated browser flow | PASS: initial slope/active legend/no preRunWasm,Run with finite numeric HUD,material/shape movement,Pause,staged selection,Reset-away/back legend and paused state |

The physics test checks collider coefficients/Average combining and60fixed steps:
lower-friction box translates downhill,higher-friction box holds within tolerance,
both spheres translate and acquire angular velocity. Browser observations use
four bodies,default gravity/speed and actual Wasm/GPU. Downhill displacements at
Pause were approximately1.68/1.75/0.05/1.75m for low box/ball then high box/ball,
measured from rounded SSR initial positions. This is a bounded qualitative demo
observation,not a calibrated material/engineering or timing/performance result.

Firefox154/Linux/private packaged Weston used a disposable opt-in WebGPU profile
with reduced-motion preference,no forced adapter/backend or permanent config/
installation changes. Application adapter text only identifies BrowserWebGpu,
not a device/vendor. The native server used ephemeral127.0.0.1. The actual image
was inspected; active legend,tilted ramp,box/ball colors and numeric estimated FPS
are visible. No page-observed runtime/rejection/CSP error. Selection alone retained
slope poses/ticks/legend; Reset-away hid the legend,Reset-back restored a paused
zero-tick slope and cleared rate samples. Owned server/profile/Firefox/WebDriver/
compositor and socket were cleaned. No product/browser failure occurred.

Local receipts/scripts/image remain outside the checkout:
`/tmp/yetlet-yt198-slope-20261026-ysrx1g9_/`. Focused browser command:
`python3 <evidence-directory>/browser.py`. Prior YT-197 tests,rate-helper tests,
old browser/CSP/GPU/business/benchmark qualifications and all previous failures
were reused/preserved rather than rerun. Full other-preset dynamics,other
browsers/devices,long runs and all-setting physics outcomes are not new claims.
Independent acceptance was pending at that evidence checkpoint; final closure follows.


A local inventory observation was corrected without product/test changes: browser
keys include the digest prefix,and the directory has two unserved bindgen type
declarations beyond the six hashed/runtime files. A temporary exact-six-directory
assertion failed; direct comparison of all eight observed entries matches. The
six-file native allowlist/hash was already correct. A dependent resolver input
was absent after that assertion; no agent/run dispatched. The observation history
is retained in asset-inventory-observation.json; no browser/test was replayed.


## Final acceptance and changed paths

Independent incremental Reviewer PASS,0issues/required followups. All9source/
24evidence hashes,1,131protected current files,the actual incremental patch,
six-file bundle and all eight browser-observed assets matched through review.
Native reviewer/gpt-6-astra/high telemetry matches the shared-resolver decision.
Only completion metadata/docs checks follow acceptance; runtime source unchanged.

YT-198 changes nine paths relative to the accepted uncommitted YT-197 snapshot:

- `examples/physics-lab/src/model.rs`, `src/physics.rs`, `src/server.rs`,
  `loader.js`, `README.md` (remaining paths relative to that example).
- `docs/BACKLOG.md`, `docs/ROADMAP.md`, `docs/README.md` and this record.

The aggregate YT-197/198 working tree contains14changed paths,staged0,HEAD89de4c8.
No Commit/Push. Prior YT-197 evidence remains intact; its frame-rate module,
tests,style,builder and acceptance record were not changed. No further Todo
exists outside the YT-096/020 release decisions. Adjustable slope/materials,
other scenes and wider qualification are unselected followups,not started work.
