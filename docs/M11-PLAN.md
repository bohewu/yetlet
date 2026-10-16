# M11 — Product Readiness and Optional Scene Rendering

**Date:** 2026-10-12. **Planning task:** YT-069.
**Status:** YT-069 is complete at documentation and owner-permitted self-review scope; implementation is not authorized by this plan. [Planning review](M11-PLANNING-REVIEW.md) records the documentation checks, reconciliation and assurance limits.
**Source baseline:** `c7ecc49eeb667e145cfb7e764c10b72ba104bbee`, existing `master` checkout, clean at inspection. No branch/worktree change.

## 1. Charter and authority

Establish M11 with two independent lines:

- **M11-P:** a small complete SSR-first product, safe useful HTML/form behavior, reuse of an existing CSS system, precise XSS boundaries and qualification against a declared operating envelope.
- **M11-G:** browser-first proof and possible promotion of `yetlet-wgpu` as a first-party optional Experimental scene renderer.

The owner's product-readiness handoff is the basis of [the research document](M11-PRODUCT-READINESS-RESEARCH.md). The latest conversation separately authorizes writing repository documentation and either self-review or reviewer dispatch. It does not authorize production-code edits, dependency acquisition/upgrades, new Cargo/browser/GPU tests, credentials, deployment, commits, push or publication. Existing documentation/static consistency checks are the only validation planned for this document delivery.

This is a project filing and reviewed proposal, not permission to implement all backlog rows. Later task activation must identify the exact slice, allowed writes/commands, dependency changes, owned resources and acceptance scope. Keep the current checkout and preserve concurrent work. No application launches or browser interaction are needed for YT-069.

## 2. Decisions and non-decisions

| Topic | Planning disposition |
|---|---|
| First complete product | Recommend Lab Operations Board for a noncritical lab/project team; one persisted entity workflow, permissions, history, conflict handling and meaningful status stream |
| Visual system | Recommend exact locally hosted Bootstrap CSS-only proof, subject to asset/license approval and CSP compatibility; Tailwind build-time CSS is the documented alternative |
| Framework gap priorities | Exactly G1 safe DOM/form semantics, G2 document/CSS asset integration, G3 complete product trust/failure journeys; see research section 6 |
| Default runtime | Complete no-JS/no-WASM SSR remains useful; HTML/DOM/default server do not depend on GPU or Session |
| GPU endpoint | Proposed `yetlet-wgpu` first-party opt-in Experimental adapter; neither builtin activation nor stable/public release |
| Existing research | YT-021/022/024/029 remain their documented app-owned research/recipe decisions; no automatic streaming/resumability/custom-TCP/deadline framework promotion |
| Shared scene model | No `yetlet-scene` extraction before a second materially different consumer; no game renderer/engine expansion |
| Release | YT-020 remains Blocked; original nine candidates unchanged. M11 does not require registry publication to investigate private product use |
| Review | The owner permits current-session self-review for this planning delivery. It cannot satisfy independent security, exact-review, native-trace or formal-assurance gates for later work |

[Proposed ADR 0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md) records these boundaries without declaring an implemented architecture. Existing ADRs, M7/M10 evidence and research decisions remain intact.

## 3. Task allocation

These are newly filed repository IDs. They replace the preceding conversation's illustrative, unregistered GPU-only YT-069–YT-074 numbering. `Todo` means planned and awaiting activation, not authorized execution. An accepted no-promotion decision may close a decision/proof item only at that explicit scope; it cannot close a crate-delivery requirement as implemented.

| ID | Line | Bounded outcome | Depends on |
|---|---|---|---|
| YT-069 | M11p | Source-grounded research, integrated task plan, proposed ADR, documentation checks and permitted planning review | YT-068, YT-023 |
| YT-070 | M11-P | Isolated realistic form/CSS proof using app-owned wrappers; exact CSS/source/license/CSP decision | YT-069 |
| YT-071 | M11-P | Selected safe HTML/form/property/URL contracts and targeted XSS/lifecycle regression; promote only justified primitives | YT-070 |
| YT-072 | M11-P | Two-consumer stylesheet/document contract proof; additive tiny helper or explicit retain-wrapper decision | YT-070 |
| YT-073 | M11-P | Complete persisted Lab Operations Board with roles/object authorization, history, conflicts and status/reconnect states | YT-071, YT-072 |
| YT-074 | M11-P | Approved application-envelope load/failure/identity/browser/accessibility qualification and risk statement | YT-073 |
| YT-075 | M11-G | Exact dependency/feature choice and real browser WebGPU island, independent assets and SSR fallback | YT-069, YT-023 |
| YT-076 | M11-G | Resize/DPR, async ownership, loss/recovery, suspend, dispose/remount and resource bounds | YT-075 |
| YT-077 | M11-G | Equivalent dynamic workloads, primary Canvas2D comparison, startup/bundle/frame/update evidence | YT-076 |
| YT-078 | M11-G | Picking/selection semantics, keyboard/touch alternatives, accessible data path and safe overlays | YT-077 |
| YT-079 | M11-G | Explicit promotion decision and, if separately approved, bounded first-party `yetlet-wgpu` Experimental crate | YT-078 |
| YT-080 | M11-G | Selected browser/GPU target qualification and candidate API/dependency/support boundary | YT-079 |
| YT-081 | M11 closeout | Reconcile selected outcomes, unmet/deferred gates, actual evidence and reviewer results without release activation | YT-074, YT-080 |

```text
YT-068 + YT-023 -> YT-069 (documentation only)

M11-P: YT-069 -> YT-070 -> (YT-071 + YT-072) -> YT-073 -> YT-074
M11-G: YT-069 + YT-023 -> YT-075 -> YT-076 -> YT-077 -> YT-078
       YT-078 -> YT-079 -> YT-080

YT-074 + YT-080 -> YT-081 (selected-scope closeout)
```

The table is the exact dependency definition. No product implementation item depends on YT-075–YT-080. GPU proof can reuse current app-owned SSR/asset patterns without waiting for a new stylesheet API. Conceptual parallel lines do not authorize simultaneous agents writing the same checkout.

## 4. Acceptance contracts

### M11-P

YT-070 consumes the research matrix and an approved CSS artifact. Produce one useful no-JS styled form/list plus an optional bounded interactive form proof. Include real select/textarea/checkbox semantics, missing/injected asset cases, CSP enforcement and lifecycle; no arbitrary broad allowlist expansion. It must retain local ownership/cleanup and compare app-wrapper versus proposed helper cost.

YT-071 accepts only selected reusable behavior justified by the proof: boolean omission, validated ordinary tags/attributes/URLs, live/default property handling, validation/error/reset and ownership. Keep native SSR as the fallback. Add focused regression and real browser/IME evidence where required; an allowlist-only patch cannot meet the gate. Independent security-focused review is mandatory before accepting a widened security boundary. A justified app-owned solution must state what remains unsupported rather than pretending generic coverage exists.

YT-072 must show two consumers needing the same ordered same-origin stylesheet contract. Either retain the existing application wrapper with documented tradeoffs, or add an explicitly reviewed minimal server-adapter helper while preserving legacy signatures, no-asset output and CSP. No raw head HTML, script manager, CSS engine or mandatory UI library.

YT-073 delivers the complete Board contract in research section 5: direct SQL, validation, PRG, version conflicts, transactional history, Viewer/Operator and cross-project checks, selected Session semantics, persisted metric/status state, bounded notifications and authoritative refresh after a replay gap. Test real stored text across edit/save/read/preview. The first fixture is noncritical and loopback-only; synthetic authentication cannot be relabeled as real identity. No remote machine control or school production data.

YT-074 requires an explicitly approved operating envelope and exact immutable source. Report percentiles/sample counts, achieved request mix, CPU/memory, errors, fault injection, graceful shutdown, restart/rollback, identity/session loss, accessibility and cleanup. Broader browsers/HA/backup operations are not implied; unresolved required cells block the selected claim. A production identity test needs separate IdP/config/credential authority.

### M11-G

[The GPU plan](M11-WGPU-PLAN.md) owns the detailed acceptance matrix. Browser canvas execution is required; native Vulkan completion timing does not substitute. Lifecycle correctness precedes performance, useful performance precedes extraction, and accessible input/fallback remains required. No wgpu dependency or GPU initialization in default SSR/DOM. Compare exact current-source unrelated assets before/after later changes. Native backend expansion is optional, not a universal browser-first gate.

### M11 closeout

Each activated implementation slice requires its own task-fit checks and an independent read-only reviewer before acceptance. Record actual reviewer identity/mode and terminal result, not merely the intention to dispatch. If dispatch is unavailable, follow the applicable repository/role fallback contract and label self-review honestly; it cannot satisfy an independent gate.

YT-081 closes only the selected, actually delivered scope. If the owner defers GPU promotion or another line, record a separate scope amendment and unmet gates; do not change Todo tasks to Done to make M11 look complete. Product-qualified does not imply renderer-qualified, and neither implies YT-020 activation.

## 5. Stage authorization and stop rules

| Stage | Permitted only after stated activation | Stop condition |
|---|---|---|
| Current planning | Read-only source/evidence/web inspection, English repository documents/planning records, static documentation checks, owner-permitted self-review | No new runtime/build/browser/GPU test or credential/external-state change |
| Isolated proof | Exact source/dependency/build scope, one bounded owned browser/CPU/GPU attempt, captured cleanup | Missing capability, unknown process outcome, unexpected scope/security cost or insufficient evidence |
| Framework changes | Explicit patch scope after proof; targeted compiler/browser/security/compatibility tests and independent review | Unsafe raw sink, unreviewed default dependencies, broken source/bundle compatibility or failed review |
| Complete product | Approved domain/data/auth/session scope and bounded local resources | Real data/payment/network exposure without authority; undefined mutation outcome or unowned long-lived work |
| Qualification/release | Exact envelope/targets/source for qualification; separate approvals for packages/signing/registry/deployment | Unmet selected cells, mismatched evidence identity or absence of explicit release/deployment approval |

Do not start new/replacement work while an existing execution is active or uncertain. Before an observable UI test, state the single bounded action; select the current target, refresh after state changes, verify once and stop at terminal success. Close only task-owned resources or a window the owner explicitly asked to close. An interruption stops new actions until active state is understood.

## 6. Finite decisions before later implementation

The recommended next activation is **YT-070**, a real form/CSS proof; **YT-075** can be activated independently for the owner's GPU interest. Neither is started here.

Before activation approve: (a) Board as the first domain and its excluded capabilities; (b) exact CSS/dependency acquisition and CSP-compatible asset method; (c) allowed proof builds/browser activity and owned resources. Before YT-073/074 approve identity/session semantics and the proposed audience/load/latency/browser envelope. Before YT-075/079 approve the exact wgpu target/version scope and later official-crate promotion. These decisions change authority or product scope; implementation details already resolved by source do not require another open-ended research round.

## 7. Documentation delivery evidence

Read [product research](M11-PRODUCT-READINESS-RESEARCH.md) for A–E traceability, the three gaps, XSS matrix, product comparison and visual brief; read [GPU plan](M11-WGPU-PLAN.md) for renderer promotion and lifecycle gates; read [planning review](M11-PLANNING-REVIEW.md) for commands, findings and review limits. Backlog, roadmap, ledger and kanban must agree on task IDs, dependencies and status. Historical 68-item/YT-068 snapshots remain labeled history; M11 does not overwrite their results or signed bytes.
