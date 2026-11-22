# M28-F2 / YT-282 — ADR0013 promotion boundary

- **Date:** 2026-11-19.
- **Status:** Scoped F2 amendment explicitly accepted on 2026-11-19; bounded public YT-282-B implementation authorized.
- **Source:** [ADR0013](adr/0013-composable-application-and-hosts.md) is Accepted for the scoped F2 amendment; [YT-281 reviewed contract](M28-F2-PUBLIC-COMPOSITION-CONTRACT.md) remains independent R4 **design-only** evidence.
- **Constraint:** The maintainer authorized only additive public std-only Composition Core implementation in YT-282-B, its focused validation and independent review; later adapters and Git/publication actions remain deferred.

## Accepted decision record — 2026-11-19

The maintainer explicitly adopted the narrow acceptance option in the original
WebCodex conversation and authorized the bounded **YT-282-B public std-only
core**, focused tests and independent Reviewer. [ADR0013's decision record](adr/0013-composable-application-and-hosts.md#accepted-scoped-f2-decision--2026-11-19)
records the exact reviewed ADR and packet hashes and R1 actual PASS receipt.

This selects additive default-off `composition` in the existing `yetlet`
package, static typed resources/callbacks and custom Host support without a
general Host Trait. Separate native companion/adapters are directions for
later authorized increments. No current authority extends to those adapters,
F3, Stage/Commit/Push, SDK/CLI version promotion or publication.

The earlier Proposed decision packet and private-A proof below are historical
checkpoints. Their then-open approval gate is now satisfied; their review
receipts and original evidence remain unchanged. YT-282 overall remains Todo
until its later adapter/consumer work is complete. Physical D4 is Not verified;
the latest global Docs result is FAIL19, including the retained historical
FAIL18 diagnostics and one prior ignored review-request diagnostic.

## Historical pre-adoption boundary

The following decision options and private proof constraints describe the state
before the explicit acceptance recorded above.

## Explicit decision required before public API mutation

ADR0013's initial no-generic-Host direction remains sensible for the original B–D POC. The new F1 consumer demonstrates ownership-sensitive repeated operations across real UDP, Web, and Native lifecycles. YT-281 recommends an **opt-in typed composition coordinator**, not a universal Host trait, registry, DI container, or SDK-owned scheduler.

A maintainer decision must either:

1. **Approve a scoped amendment and accept ADR0013** for the specific optional, statically typed, app-owned `yetlet::composition` coordinator with separate `yetlet-hosts` native adapters. Record the exact module/feature boundary, compatibility and Pending/Join semantics before implementation/promotion.
2. **Keep ADR0013 Proposed or reject promotion**. Continue only with private source proofs and re-scope YT-282 public implementation. The public module, Cargo features, and companion package cannot truthfully be called delivered.

No code in this YT-282-A private slice changes the existing `yetlet` public API, Cargo feature/default graph or CLI. This private prototype demonstrates type/lifecycle feasibility and cannot substitute for approval, public package consumption, or YT-283's external custom Host.

## Private YT-282-A boundary

Use existing unpublished, independent std-only `examples/composable-host-research` solely for a source-level static-chain proof. Test an application-owned root holding caller-local `Rc`, owned/borrowed attachments, fallible required startup with explicit preceding rollback, required foreground readiness, first-failure retention, admission-first stop and a fake owned worker that remains retained while completion is Pending. A **private chain trait** for static code recursion is not a universal public Host trait, nor a dynamic registry.

Do not introduce winit/wgpu/UDP/Axum/Tokio into this std-only crate, create a separate process per Host, or implement YT-282-B/C/D/E ahead of a renewed scope/ADR decision.

## Qualification boundary

The new source demands its own focused compile/tests and independent Reviewer. Prior F1 Native+HTTP+UDP and YT-281 design evidence are reused at their original source identity; they do not verify the new Rust type implementation. Global Docs inherited FAIL18 is retained unchanged in ignored F0-B2 evidence. Physical D4 remains Not verified. No Stage/Commit/Push by default.
## Maintainer decision packet — drafted, not approved (2026-11-19)

This packet makes a decision possible without requiring the maintainer to
approve every sample-specific detail or a new runtime. No explicit approval
has been recorded. The exact source baseline remains the reviewed local
F1 commit `363dde2` plus six retained YT-281/YT-282-A dirty paths.

### Narrow acceptance option (recommended; awaiting owner)

> Accept an additive **scoped F2 amendment** to ADR0013: allow an opt-in,
> static typed `yetlet::composition` application-owned resource coordinator,
> plus separately opt-in `yetlet-hosts` native adapters. Keep every existing
> Pure Web and non-Web mode, local `Rc`, caller-owned foreground/runtime
> resources, first failure, admission-before-C1 stop, real Join/Pending,
> and original worker/borrowed resource ownership. Do not accept a general
> `Host` / `Transport` / `Renderer` trait, dynamic registry, scheduler,
> DI container, implicit thread/process per Host, bundled Twin Squares
> policy, or automatic network/physical D4 readiness. Treat acceptance
> as design permission to implement reviewed slices, not product publication.

### Decline/defer option

> Keep ADR0013 Proposed; retain only the existing private YT-281 design
> and YT-282-A test-only proof. Do not modify public `yetlet` modules,
> Cargo features or introduce the `yetlet-hosts` package. A later public
> implementation needs a new explicit decision; no unreviewed adapter
> migration or copied research supervisor substitutes for that approval.

### Future YT-282-B earliest public slice after acceptance

- **Allowed initial public files:** add a new `crates/yetlet/src/composition/`
  module, additive `composition` feature in
  `crates/yetlet/Cargo.toml`, and a cfg-gated module declaration in
  `crates/yetlet/src/lib.rs`. Keep this first slice std-only with the
  default feature/dependency graph unchanged. Separate production
  namespace from the private `composition_f2a` test-only proof; do not
  wholesale copy its test scaffolding into the SDK.
- **Contract:** static typed attachments with fallible startup/rollback,
  required foreground readiness, caller-owned borrowed resources,
  first-failure and secondary cleanup evidence; separate admission-close,
  local stop, HTTP shutdown signal and actual owned completion. Failed
  admission closure must not release the authoritative domain.
  `Pending` retains the same original worker and ownership; a worker
  failed after Join must never be reported as successful.
- **Evidence before calling the slice implemented:** new compiling public
  API unit/consumer tests for local `Rc`, borrowed release, failure
  rollback with owned worker, unready Native part, saturated/failed
  admission, failed cleanup and stop signal, Pending-to-completion on
  identical handle, and source/feature compatibility checks. Distinguish
  real worker tests from test-only synthetic handles. Run Clippy/Fmt,
  exact changed-feature graph comparisons, and independent Reviewer.
- **Separate later adapters:** `web-axum`, `udp`, `native-winit` and
  `renderer-wgpu` only when their own actual consumer and verification
  scope are authorized. External custom Host and package-consumption
  gates remain YT-283/YT-284. No Windows/GPU/HTTP matrix is rerun
  when this first std-only slice changes no native execution behavior.

### Decision record fields

A future owner approval must record date, exact accepted option, ADR
revision, reviewed contract identity, affected SDK feature/package
boundary, expressly deferred items and independent review receipt.
Approval changes neither the historical YT-281/282-A R1–R3 results nor
physical D4. Without that record, the **ADR remains Proposed**, and
`YT-282` remains `Todo` despite the completed private-A proof.
