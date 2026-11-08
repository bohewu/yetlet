# YT-237–YT-240 — Optional CSS authoring and project stylesheet roadmap

**Date:** 2026-11-07 (Asia/Taipei)
**Status:** **Planning accepted (independent Reviewer PASS)**; YT-237 is **Done at contract-research scope after independent design Reviewer PASS** under the 2026-11-08 owner instruction. YT-238 is **Done at prototype scope after independent implementation Reviewer PASS**. YT-239 is **Done at source and bounded Linux integration scope after independent implementation Reviewer PASS** under SIMPLIFY. YT-240 is **Done at bounded Todo/Linux sample scope after independent Reviewer PASS**.
**Baseline:** `master` at `3ecbfc36a91a2a06ef2dee69a7a0d848a2ecd16e`, `feature/style-css-roadmap`.
**Direction:** Define goals, hard dependencies, acceptance criteria and go/no-go gates before implementing a Rust CSS system.

## Goal and scope

Make it possible for a Yetlet application to **opt in** to Rust-authored
common layout styles (Flexbox, Grid, spacing, sizing, colors, typography and
responsive rules), and produce a **project-owned external CSS file** before
the application runs. Keep authored ordinary CSS as the default and Tailwind
or Vite as optional alternatives. Both native SSR without JS and browser/WASM
DOM must consume the same normal stylesheet without renderer changes.

The hardest question is **style declaration discovery/registration**, not the
name of a `.display()` method. A build tool cannot automatically know that
a `StyleSheet::new()` exists anywhere in Rust code or safely infer every
conditional View that might appear at runtime.

## Existing substrate (not new deliverables)

- [View builder YT-234/235](YT-235-TAG-COVERAGE-PLAN.md) provides a typed
  authoring facade over the original shared `View`/`Element` and class
  attributes. It has no CSS serializer.
- [CLI contract](CLI-CONTRACT.md) uses `yetlet build`, `yetlet watch` and
  optional `new --tailwind` / `--vite`; persistent choices belong in
  settings, not proliferating `--with-*` flags.
- [YT-098 asset experiment](ASSETS.md) validates **application-owned**
  standalone CSS/Vite manifest loading, stylesheet ordering and safe paths.
  It did **not** publish a stable `yetlet-assets` framework crate.
- [Renderer and load ADR](adr/0003-renderers-and-loading.md) separates
  native HTML, browser DOM and scene renderers and selects build-time assets.
  [Opt-in ADR](adr/0005-opt-in-capability-modules.md) avoids mandatory
  dependencies and provider registries.
- [DEFAULT_CSP](../crates/yetlet/src/server_axum/mod.rs) allows external
  same-origin CSS via `style-src 'self'`. Existing rules forbid inline
  `style` attributes and runtime `<style>` injection for ordinary Views.

## Ordered dependencies and decision gates

| Item | Status | Objective | Required predecessor / evidence |
|---|---|---|---|
| **YT-237** | **Done (contract research)** | Decide style registration, host-side execution, deterministic CSS contract | [Reviewed Contract](YT-237-STYLE-REGISTRATION-CONTRACT.md); independent design Reviewer PASS, no implementation |
| **YT-238** | **Done (prototype)** | Bounded Rust CSS authoring/serialization proof | [Prototype evidence](YT-238-STYLE-BUILDER-PROOF.md); independent implementation Reviewer PASS; SIMPLIFY recommendation |
| **YT-239** | **Done (source/Linux integration)** | Opt-in `build`/`watch` and stylesheet asset integration | [Integration evidence](YT-239-STYLE-INTEGRATION.md); owner-approved SIMPLIFY, focused/real Linux evidence and independent Reviewer PASS |
| **YT-240** | **Done (Todo/Linux sample)** | Native SSR + browser/WASM sample and qualification | [Todo source and evidence](YT-240-TODO-STYLE-QUALIFICATION.md); Chrome/Firefox bounded real-browser and CSS/asset/CSP proof, independent Reviewer PASS |

**Order:** YT-237 -> YT-238 -> go/no-go -> YT-239 -> YT-240.
**Stop rule:** If YT-238 shows that native CSS or existing Tailwind/Vite is
simpler than the Rust DSL, stop or simplify the DSL; YT-239/240 remain Todo
or deferred. Planning review never authorizes implementation, Git delivery
or release.

## YT-237 — Style discovery/registration and output contract (Done; contract research)

The owner authorized this research on 2026-11-08. The
[Reviewed Contract](YT-237-STYLE-REGISTRATION-CONTRACT.md) compares alternatives
before selecting ordinary Rust composition, shared class strings and an isolated
host generator. Independent design Reviewer PASS (Job
`wc_job_hrrQG-bXFiMCgGQ3`, zero issues/followups) completed this research only.
Subsequent owner decisions separately authorized YT-238 and YT-239.

**Goal:** Select a reliable way for a build process to discover the complete
set of project style definitions without executing arbitrary application routes.

**Research / deliverables:**

1. **Explicit registry first.** Evaluate a single app-owned
   `styles::stylesheet()` entrypoint composing modules such as
   `layout::styles()` and `dashboard::styles()`. Compare compile-time
   registration and source scanning, but do not select hidden global
   registration or an AST scanner without clear evidence. Prove on paper
   that a class used only in a conditionally constructed View is still emitted
   when the View did not execute at generation time.
2. **Execution boundary.** Evaluate a separate **host-side generator target**
   (e.g. an explicitly selected Cargo binary), how the CLI locates/invokes
   it, and how it avoids recursive Cargo invocation from `build.rs`, web
   server startup, default Node downloads and WASM dependencies.
3. **API shape.** Choose whether an app can pass a lightweight `CssClass`
   handle to existing `.class()` or should use a stable string class name.
   Avoid expanding 101 typed tag builders with CSS properties, implicit
   View traversal, ambient mutable registries, new renderer variants or
   mandatory proc macros.
4. **Determinism.** Select class/selector naming, rule order, duplicate
   registration and collision behavior, CSS escaping/typed-value limits,
   stable development `app.generated.css` output, and optional production
   content hashing. Distinguish trusted developer CSS from untrusted dynamic
   strings; forbid directly interpolating runtime/user values into CSS.
5. **Integration contract.** Specify external same-origin stylesheet linking,
   strict CSP, file ownership, CSS cascade/ordering relative to ordinary CSS,
   Tailwind and Vite, style registration for routes/components, `build` and
   `watch` responsibilities, opt-in project settings, cancellation/failure
   and stale-asset/rollback expectations.
6. **Bounded DSL candidate.** Select minimum Flexbox, Grid, sizing, spacing,
   typography, colors, CSS custom properties and one `@media` breakpoint.
   Leave all other CSS available as authored plain CSS. No complete CSS parser,
   utility compiler, dynamic theming runtime or automatic CSS tree-shaker.

**Acceptance:** English design contract with a registry/host-process/output
diagram, explicit conditional View case, typed/invalid/duplicate examples,
alternatives and tradeoffs, native/SSR/WASM/CLI/Asset ownership matrix,
safety limitations, test plan and **independent design Reviewer PASS**.
Document the decision and get owner approval before implementing YT-238.
If the design changes a fundamental ADR decision, update that ADR.

## YT-238 — Minimal Rust CSS builder proof

**Status:** Done at prototype scope. The [proof and comparison](YT-238-STYLE-BUILDER-PROOF.md)
record 674 identical generated/plain CSS bytes, 19 focused tests and
formatting/docs/source checks PASS. Independent Reviewer PASS, Job
`wc_job_hfecy9ZrNUTO61BZ`, zero issues/followups. Recommendation: **SIMPLIFY**,
keeping the builder application/tooling-local. The owner subsequently approved
YT-239 under that recommendation.

**Goal:** Prove (or falsify) that ergonomic Rust CSS declarations offer
sufficient value without another heavyweight style framework.

**Accepted prototype scope / evidence:**

- One explicit style entrypoint incorporating **at least two Rust style
  modules**; a deliberate host-side generator target; no runtime registry.
  The reference fixture starts with a useful starter module and root registration
  already wired. Show the explicit step to add a later module; source creation
  alone is not discovery. This models the owner's default-template requirement.
- Demonstrate a Flex toolbar, Grid container, gap/padding/width, type/color,
  a CSS custom property and a responsive `@media` rule.
- Include one class used only behind a conditional View branch and show
  generation emits its CSS without running that branch.
- Native tests for byte-for-byte deterministic output, repeatability,
  repeatable generation and duplicate-registration policy specified by YT-237, a duplicate/conflict
  negative case, unsafe or invalid typed CSS values, safe output selection,
  and clear errors. Do not claim general CSS injection sanitization.
- Compare a source fixture in Rust style DSL against a small plain CSS
  stylesheet for readability, lines of code, target dependencies and build
  time. Keep all new support opt-in and application/test local.
- **No CLI integration, public crate/release, JS/WASM runtime changes or
  production deployment** in this research-only slice.

**Acceptance:** Real CSS text and source fixtures, focused tests and tool
commands, limitations, a recorded **go / simplify / no-go** decision and
independent implementation Reviewer PASS. YT-239 is conditional on a
positive result and explicit owner approval.

## YT-239 — Optional build/watch and asset integration (Done; source/Linux integration)

**Entry gate met (2026-11-08):** YT-238 PASS and owner approval to integrate under
SIMPLIFY. Implement in the existing Rust CLI; the owner prefers minimal Python
dependence while permitting necessary/convenient auxiliary use. The generated
application's normal lifecycle must not require Python.

**Completion:** [Implemented contract and evidence](YT-239-STYLE-INTEGRATION.md)
record default-wired templates, strict settings, protected Rust publication,
coordinated watch, native/Tailwind and Vite/browser compatibility and cleanup.
Independent Reviewer PASS, Job `wc_job_3s_tnZYc7hUssjK5`, exit 0, zero
issues/followups. Windows compile/runtime and two recorded cloud-lock failures
remain unresolved; YT-240 and public release remain separately gated.

**Goal:** A project that selects Rust styles can generate, serve and update
its stylesheet through existing CLI orchestration, without touching HTTP
request handlers or DOM rendering.

**Accepted work / evidence:**

- For selected Rust-style application templates, make actual `yetlet new` emit
  the complete default stylesheet root, starter definitions, shared class names,
  generator manifest/lock and accepted configuration/asset connection. Reuse
  existing selection mechanisms; no new public flag is selected here. `new`
  creates files without running the generator. Ordinary-CSS templates remain
  unchanged. Test default output and document registration of later modules.
- Make a configured host-side generator part of `yetlet build` and
  `yetlet watch`, with safe project-root selection, no recursive Cargo
  build from `build.rs`, no Node default, explicit errors and recovery.
  Preserve the exact behavior of apps without the style opt-in.
- Reuse the finite application-owned Asset contract or a narrowly reviewed
  equivalent for external CSS paths, same-origin links, CSS MIME, stable
  ordering/dedup and static output. Do not promote YT-098 experimental
  assets to a stable public package automatically.
- Preserve Tailwind standalone and Vite modes. Vite retains ownership of
  CSS HMR if selected; native-only watch can require a normal page refresh
  rather than claiming cross-mode hot replacement.
- Production outputs must be reproducible; any content-addressed path must
  verify its digest and document retention of older CSS during rollout and
  rollback. Prevent serving private manifests, source and project files.
- Document normal CSS + generated CSS ordering, collision precedence,
  errors on invalid config, failure/recovery, native/Windows portability
  and CSP unchanged.

**Acceptance:** Focused CLI/asset native regressions, CSS edit/watch/recovery
run, opt-out unchanged behavior, optional Vite/Tailwind compatibility proof,
asset path/content/CSP regression and independent Reviewer PASS.
Public CLI/SDK release remains separately gated.

## YT-240 — Todo sample and end-to-end qualification (Done; bounded Linux)

**Entry gate met:** YT-239 accepted, and the owner selected the existing Todo
sample before any dedicated Style Showcase. The [source and validation record](YT-240-TODO-STYLE-QUALIFICATION.md)
contains the bounded Native SSR/Chrome/Firefox evidence. Independent Reviewer
PASS, Job `wc_job_7bfgn4kA_CNHLN22`, exit 0, zero issues/followups. Firefox
390px, Windows and production deployment remain unverified.

**Goal:** Demonstrate an actual project using generated CSS on native SSR and
browser/WASM without coupling the CSS generator to either renderer.

**Acceptance criteria and verified proof:**

- Native SSR page with Flex/Grid/responsive CSS, external same-origin
  stylesheet and **no JS/WASM runtime**.
- Browser/WASM page using the same stylesheet and toggling a class whose
  CSS was registered in advance but absent from initial rendered Views.
  Verify actual computed style, not merely the presence of a class.
- Two viewport breakpoints, stable CSS bytes and links, correct MIME,
  no duplicated asset requests, no inline styles or CSP violations;
  JS-disabled SSR content must still be useful.
- Real build/watch style edit and recovery, bounded browser smoke
  (including cleanup of owned processes), tests for Native/Browser and
  platform-specific compile/run evidence as appropriate. Windows claims
  need real Windows evidence, not a cross-target build alone.
- Prefer adapting an existing sample; add a new catalog entry only if
  [sample catalog](SAMPLES.md) and owner-approved need justify it. README
  must give copyable commands and explicit experimental limits.

**Acceptance:** Runnable source and docs, native + browser/CSP/asset evidence,
a platform/support matrix, independent Reviewer PASS and a documented
post-prototype packaging choice: optional `yetlet-style`, tooling-local
helper, or stop/no promotion. No automatic publication/deployment.

## Cross-item non-goals and safety

- Do not add `div().display(...).gap(...)` to typed View or
  `.style()` / runtime `<style>` injection; keep Style and View separate.
- No default Node/Tailwind/Vite dependency, runtime HTTP/WASM CSS compilation,
  global mutable class registration, general CSS AST sanitizer, giant utility
  generator, forced public crate or release, or new CSP exception.
- Keep generated files out of tracked secrets/source, validate output paths,
  and explicitly distinguish source trust from untrusted runtime values.
- Reviewer PASS is limited to the item and observed tests. Future live
  browser/Windows/CI evidence must be obtained in its respective phase.
- A plan review does not grant implementation, Commit, Push, PR, Merge,
  package publication, cache deletion or remote deployment authority.

## Planning acceptance / handoff

The original planning scope, ordering and gates were recorded with four **Todo**
items in [BACKLOG](BACKLOG.md) and [ROADMAP](ROADMAP.md). The owner subsequently
authorized YT-237 contract research, YT-238 implementation and YT-239 integration
under SIMPLIFY. YT-237 and YT-238 are Done at their respective research/prototype
scopes. YT-239 is Done at source and bounded Linux integration scope after
independent implementation review. YT-240 is now Done for the existing Todo at bounded Linux qualification scope;
its dedicated Showcase follow-up is not started. This roadmap is distinct from
the linked implementation evidence.

**Independent planning Reviewer:** PASS — role-directed read-only Job `wc_job_zVhJeQacmnaDxXdW`, process exit 0, `overall_status=pass`, zero issues and required follow-ups. This is ad hoc review, not formal assurance or native managed dispatch.
**Original planning verification:** `cargo xtask check docs` PASS; four Todo IDs, dependency order and whitespace checks PASS; `git diff --check` PASS. No CSS implementation had started at that planning review.
**YT-237 completion (2026-11-08):** Independent design Reviewer PASS, Job
`wc_job_hrrQG-bXFiMCgGQ3`, exit 0, zero issues/followups. Docs/source checks and
explicit checks for the two new documents and item order PASS. The
[Contract acceptance record](YT-237-STYLE-REGISTRATION-CONTRACT.md#13-validation-and-independent-review)
retains scope, input identity and evidence.
**YT-238 authorization follow-up (2026-11-08):** The owner explicitly authorized
starting the prototype and asked suitable templates to include a default registry.
The [Contract addendum](YT-237-STYLE-REGISTRATION-CONTRACT.md#14-owner-follow-up-default-wiring-and-yt-238-authorization)
assigns reference-fixture proof to YT-238 and real CLI scaffolding to YT-239.
**YT-238 completion:** Independent implementation Reviewer PASS, Job
`wc_job_hfecy9ZrNUTO61BZ`, exit 0, zero issues/followups. All 22 input hashes
were unchanged during review. The [completion record](YT-238-STYLE-BUILDER-PROOF.md#independent-review-and-completion)
retains tests, measured costs, SIMPLIFY recommendation and evidence locations.
**YT-239 authorization follow-up (2026-11-08):** The owner approved integration
under SIMPLIFY and prefers avoiding Python in the official generated-app path.
The [latest Contract addendum](YT-237-STYLE-REGISTRATION-CONTRACT.md#15-owner-approval-of-yt-239-and-language-preference)
records the scope and implementation choices.
**YT-239 completion:** Independent implementation Reviewer PASS, Job
`wc_job_3s_tnZYc7hUssjK5`, exit 0, zero issues/followups. All 61 reviewed worktree
source hashes were unchanged during review. The [completion record](YT-239-STYLE-INTEGRATION.md#independent-review-and-completion)
retains the source/Linux scope, evidence and unresolved qualification limits.
**YT-240 completion:** Independent Reviewer PASS, Job
`wc_job_7bfgn4kA_CNHLN22`, exit 0, zero issues/followups. All 81 frozen source
hashes matched throughout review (61 previously dirty, 20 YT-240 targets).
The [Todo qualification record](YT-240-TODO-STYLE-QUALIFICATION.md#independent-review-and-completion)
retains the exact scoped evidence, former README catalog failure/recovery,
platform limits, cleanup and packaging recommendation. A dedicated Style
Showcase, Git delivery and public release remain separate and unstarted.
