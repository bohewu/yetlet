# YT-203 — Packaged WASM, DOM and optional GPU evidence

Date: 2026-10-27 (Asia/Taipei). Status: **Done at selected evidence-reconciliation
and missing-check scope after independent Reviewer PASS.** No new source/runtime feature, maintained
sample, Commit, Push or Merge is authorized by this delivery.

## Authority and fixed inputs

The owner authorized committing the four accepted YT-202 documents, without
Push, then inventorying YT-203 and executing only missing selected checks under
the [sixteen-package plan](YT-200-SIXTEEN-PACKAGE-RELEASE-PLAN.md#proposed-per-package-platform-and-feature-scope).
The YT-202 commit is `acbc9cdd58888883783e5421b4ed6c8e2517aa28`, tree
`f3b802f81378476fe8eb965f77ebd8b743b41e7c`: strict/builtin staged Guard PASS,
four files, zero findings, with matching pre-commit tree and unchanged hooks.
The working tree was clean afterward. No Push occurred.

The qualification inputs are the five original WASM libraries (Core, Actions,
Reactive, DOM and keyed DOM) plus opt-in `yetlet-wgpu`. Library sources still
match the [YT-201 package preparation](YT-201-PACKAGE-PREPARATION.md) and its
accepted sixteen-package source; the later OIDC patch and YT-202 commit do not
change these inputs. This task does not choose a new final release candidate.
YT-096/D1–D6, YT-020 and subsequent YT-204/205 gates remain separate.

## Retained evidence versus real gaps

| Selected cell | Reuse and newly required evidence |
|---|---|
| Original five WASM source builds | Retain successful Rust 1.99 MSRV and Stable CI at head4e5ef64; current relevant sources are identical |
| Current Chrome DOM/keyed/island behavior | Retain three passing Chrome suites from run40431263041; no rerun |
| YT-110 controls/navigation/atomic rejection | Retain reviewed native/Chrome behavior and exact unchanged DOM implementation; add only the missing current-package Firefox observation |
| Normalized package WASM/default/browser feature path | YT-201 verified archives natively; source-workspace WASM checks do not alone show this new packaged consumer. Add one default WASM build and one explicit browser-feature compile |
| Actual optional GPU renderer | Retain YT-081's real crate consumer, unchanged runtime/locks/assets and selected Firefox158/X11/Intel-Vulkan configuration; no new GPU execution |
| Common startup/Tasks BFCache | Retain YT-196 at its source-bound application scope; selected helper/entry bytes remain unchanged. It is not a substitute for DOM/wgpu package execution |
| Physics Lab and other platforms | Lab uses its own private renderer; no package GPU qualification is inferred. New GPU/OS/mobile/IME/AT matrices are not selected |

Retained CI: [run40431263041](https://github.com/bohewu/yetlet/actions/runs/40431263041),
head `4e5ef64ad0b754856200b63455a347cf29b15c01`. The recorded MSRV/WASM and
Chrome steps pass. Native/browser consumer/library source comparison to the
current checkout is empty; this task neither triggers hosted CI nor reassigns
old results to changed source.

Compared with YT-110 commit71b5f08, DOM, keyed DOM, Reactive and Actions runtime
sources/manifests are identical. Core's later private child-capacity change has
its accepted YT-152 evidence; the new WASM/Firefox consumer uses the current
packaged Core. The old YT-110 raw `/tmp` directory is no longer present.
Its committed report and independent acceptance are retained; raw historical
logs are not claimed to have been reverified or recreated.

## Optional GPU reconciliation

[YT-081's refined API qualification](YT-081-WGPU-API-REFINEMENT.md) and
[independent source review](YT-081-WGPU-API-INDEPENDENT-REVIEW.md) remain accepted.
The original result SHA-256 is
`aef12ca8f882140064e3d3f1632791921507c29ec8457cfd7c7afe699a3fc96d`;
the independent result remains
`112bf24ea0e5ecbefe27611560180b4de5249a2b9acf5f9dfd47aa910e083756`.

Twenty-eight of the original thirty-four inputs still match exactly, including
all crate runtime Rust/WGSL, consumer runtime/loader sources, both isolated
locks, and actual JS/WASM assets. The six changed wrappers are the crate
manifest/README, root manifest/lock, distribution manifest and native server
binary. Crate manifest changes are only publish eligibility, README and
repository metadata; dependencies/features remain identical. Scene asset
selection still points to the exact retained f5584f407ee4f7b2 bytes. Root and
native-binary changes are not represented as byte-identical historical inputs.
The distribution manifest's historical candidate-source fields are not rewritten.

The retained runtime proof covers real GPU drawing, picking, controlled
loss/recovery, stale completions, first/repeated disposal and meaningful HTML
fallback at that original exact configuration. It does not qualify a newly
compiled GPU binary, the current native server binary, the new consumer lock,
Firefox154 GPU, Chrome GPU or a native GPU backend.
The [package support boundary](../crates/yetlet-browser/guides/wgpu-support.md) stays unchanged.
Metadata-only changes do not create authority or necessity to replay that GPU run.

## New packaged-source WASM evidence

A task-local external workspace selects the six libraries by registry version
`=0.1.0`, using the accepted YT-201 directory sources. No repository path
dependency or patch supplies a library. All six original manifests and shipped
runtime files match the current library inputs. The directory-source null
registry-checksum limit remains; this is not actual crates.io consumption.

The new fixture has its own lock and source/config/assets. Its lock is distinct
from the retained GPU runtime lock; the fresh compile configuration and old
GPU execution are kept separate. No repository manifest/lock or library
implementation is changed. The fixture exports only the scoped DOM test seam;
it is ignored verification data, not a public Core API or new maintained sample.

| New command | Result / limit |
|---|---|
| `cargo +1.99.0 build --offline --locked --target wasm32-unknown-unknown --lib --release` | PASS with explicit fixture manifest, vendor config, owned target directory and four jobs; produces the actual default Wasm used by Firefox |
| `cargo +1.99.0 check --offline --locked --target wasm32-unknown-unknown --lib --features browser` | PASS with the same explicit inputs; reaches packaged `SceneOwner` API. Compile only, no device/browser GPU execution |
| Existing wasm-bindgen CLI0.2.129 | Generates the fixture's web JS and 85,936-byte Wasm; library/CLI version match |
| Default normal/build graph | 34 unique packages; no upstream wgpu/native GPU backend, HTTP server, Auth/Session/Config/Secrets/Signing/Rate-limit/Redis provider dependency |
| Source/input binding | All17 existing source/manifest/lock bindings unchanged; six normalized package roots and fixture source/config/lock/generated asset hashes retained |

Existing Cargo sources/tools suffice. No package, toolchain, browser or system
installation or external provider workload is performed.

## New Firefox QA

Flow: task-owned loopback fixture -> actual packaged Rust DOM mount -> native
edit/reset/GET and safe navigation/refusal -> expected DOM and preserved subtree.

Environment: Firefox154.0/Linux, geckodriver0.37.1, disposable headless profile,
requested window1280x900; captured content1280x814. Browser plugin `browser`
skill is absent; the repository's existing Firefox/W3C WebDriver method is used
for the explicitly selected target. No new Playwright/browser installation or
existing suite invocation is needed. This is desktop-engine behavioral proof,
not a mobile/physical-input/IME/assistive-technology or visual-design certificate.

| QA observation | Result |
|---|---|
| Page identity / meaningful content / ready state | PASS, fixture title and actual Wasm readiness; visually inspected screenshot contains the native form |
| Framework overlay / app exceptions | No overlay or relevant uncaught application error observed in the captured checks; no claim of a silent browser-internal log |
| Native controls | Defaults, leading textarea newline/CRLF normalization, boolean presence, selected option, readonly/disabled/fieldset state, actual native edits/reset/required/GET pass |
| Navigation | Browser-resolved same-origin root/fragment destinations are frozen before later base changes; actual fragment keeps the page owner and root link reaches the local destination |
| Invalid View / URL refusal | Duplicate attributes, void/nested/control child shapes, absolute/protocol-relative/backslash spellings and invalid document bases reject while retaining the original subtree |
| CSP | Normal fixture uses self-only scripts/connect/form/base policy plus wasm-unsafe-eval, no unsafe-eval/inline. A deliberate inline script is refused |
| Screenshot | Native default/reset state inspected; no layout polish or broad accessibility claim |

**23 new checks PASS in one execution.** The default Wasm from the new package
build is loaded; JavaScript does not emulate the Rust renderer.
Controlled cross-origin/credentials/non-HTTP/unresolvable base tests use a
negative-only route without base-uri restriction so the real DOM guard is
reached. The normal route retains base-uri self; no application policy is weakened.
The expected inline-CSP violation is recorded separately from application errors.

Owned browser session/profile, geckodriver and HTTP server are cleaned, the
browser exits and both loopback ports close. The task-local Cargo home/build
directory are also removed after retaining the fixture/assets/results. No shared
cache, service, personal profile, previous evidence or device preference is changed.

## Evidence, limits and acceptance

Evidence root: `target/yetlet-yt203-wasm-dom-20261027-0jxcwh4x/evidence/`.
Key records: contract.json, retained-ci-jobs.json, retained-msrv-wasm.log,
retained-chrome-excerpt.log, retained-gpu-reconciliation.json,
retained-dom-ci-bfcache-reconciliation.json, package-source-binding.json,
new-compile-results.json, new-verification-binding.json, firefox-result.json,
firefox-driver.log, screenshot-inspection.json and build-cleanup.json.
The one-shot fixture source and served JS/Wasm remain beside the evidence;
they are not a new validation framework.

No browser/GPU failure or product repair occurred. Old historical failures and
reviews are retained, with no passing qualification replay. Existing wrappers,
unavailable raw historical logs, compile-only versus runtime, directory-source
versus registry, and old versus fresh lock identities remain explicit.
Current document/whitespace checks pass. Only this report and
Backlog/Roadmap/Release Readiness navigation summaries are changed;
no Commit, Push or Merge of new work.

Screenshot evidence is retained as `firefox-controls.png` at the evidence root.

## Final independent acceptance — 2026-10-27

Independent Reviewer `yt203_packaged_wasm_dom_gpu_final_review_20261027`:
**PASS, zero issues and zero required followups.** The read-only review verifies
all four candidate-document and37 evidence/fixture hashes, all17 protected
source bindings, the six packaged manifest/source matches, new Cargo/Firefox
assertions and cleanup. It independently reconciles the original GPU result,
review and28 unchanged inputs against the six disclosed changed wrappers,
and confirms the limited CI/DOM/BFCache reuse without replay.

The native dispatch trace verifies the reviewer role, model `gpt-6-astra` and
effective effort `high`; the shared resolver reports enforced, no conflict
or degradation. The accepted result and filtered trace are retained in
`reviewer-result.json`, `review-runtime-trace.json` and
`review-observed-decision.json`. Reviewed inputs remain unchanged through
acceptance; subsequent edits only record completion metadata and navigation.

YT-203 is Done at this bounded scope. Selected platform/configuration and
historical-evidence limits above remain in force. YT-204 is the next unopened
item; YT-205 and formal candidate/trust/publication gates remain separate.
