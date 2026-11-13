# Yetlet Experimental Compatibility Policy

## Selected local SDK 0.9.0-alpha.7 typed authoring helpers — YT-234/245

[YT-245](YT-245-ALPHA7-RELEASE-PREPARATION.md) originally selects SDK
`0.9.0-alpha.7`, including
`yetlet::view::tags` with typed fluent constructors and additive convenience
methods. These were absent from alpha.6, first shipped in `0.9.0-alpha.7` and
are now **published in SDK `0.9.0-alpha.8`**.
The historical CLI0.2.0 registry pin remains immutable. The current reader
edition follows the verified publication pair through the required release
closeout; a development manifest change alone does not promote it. The existing owned
`Element`/`View` representation, append-only `Element::attribute`, renderer
policies, native/browser feature boundaries and Rust 1.99 MSRV remain intact.
Convenience `.id`/`.class` set or replace, `.add_class` merges distinct
class tokens, and `.placeholder` is only on typed input/textarea. General raw
`.attribute` still follows append-only duplicate validation. No JSX,
inline-style API, browser runtime or WASM renderer change is introduced.


## Selected local SDK 0.9.0-alpha.7 full ordinary tag authoring — YT-235/245

On top of YT-234, the current checkout includes 101 constructor names
corresponding to the dated YT-219 standard HTML ordinary View inventory
(89 default Browser DOM tags, opt-in `img`, and 11 SSR-only/DOM-deferred tags).
Document/head elements, forbidden executable/embedded tags, foreign namespace
roots and custom elements do not receive convenience constructors. This is
an authoring API change, **not** a change to SSR's broad tag grammar or Browser
DOM's existing admission/URL/image policy. Selected typed attribute setters
(`href`, `src`, `alt`, `type_`, `for_`, `name`, `value`) upsert on
the typed builder; generic `attribute` retains append-only semantics.
The shared `View` representation and renderer behavior are unchanged.
Published SDK alpha.7 includes these builders; CLI 0.2.0 stays independently pinned.
The published SDK alpha.8 and CLI 0.4.0 packages are independently versioned;
only the SDK includes the typed Tag Builder.

See the [typed View guide](VIEW-COMPONENTS.md#standard-html-authoring-coverage),
[dated inventory](YT-219-HTML-INVENTORY.md) and
[original YT-235 source-only evidence](https://github.com/bohewu/yetlet/blob/9a6f954dc38f3b9e854fce5a08c9b843182657f8/docs/YT-235-TAG-COVERAGE-PLAN.md#yt-235-final-local-acceptance--2026-11-07)
for inclusion/exclusion and the original verification limits. Actual SDK
publication and registry-only consumer qualification are recorded separately
in [alpha.7](ALPHA7-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-09).

## Current distribution — 2026-11-12

The published SDK is `yetlet` / `yetlet-browser` `0.9.0-alpha.8`; the independently
published CLI is `yetlet-cli` `0.4.0`, with release build SDKalpha.7.
See the [SDK publication record](ALPHA8-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-12)
and [CLI publication record](CLI-040-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-12)
for the actual artifacts and consumers. YT-220/221's DOM vocabulary, pre handling
and opt-in navigation/images are available in SDKalpha.6; YT-223's static export
is available in CLI0.2.0. Existing defaults and Experimental API limits remain.

The public reader/first-app edition remains explicitly pinned to
SDKalpha.8/CLI0.4.0 in [the site manifest](../site/pages.toml).

Some registry-only
reference examples retain their own SDKalpha.5 pins, independently of the
selected builder CLI. Publication does not promote
those consumers or replay their qualification.

Dated preparation and source-only
checkpoints below retain their historical status; they are not current pending
release gates.

## Published CLI Style authoring and checkout Showcase — 2026-11-10

Optional application-local CSS generation and build/watch integration first
shipped in **CLI 0.3.0**. Style templates are distributed in CLI 0.4.0.
The [current CLI Style guide](CLI-GUIDE.md#optional-application-owned-rust-css-yt-239-working-tree)
and [original implementation evidence](https://github.com/bohewu/yetlet/blob/e49657922bcdfe238cd1f79061178a32a4622a01/docs/YT-239-STYLE-INTEGRATION.md)
retain current usage versus dated source-only qualification.
The build SDK and embedded source snapshot are alpha.7; app SDK selection follows
the separate reviewed resolver policy, exact pins and project Cargo.lock.
[Showcase](../examples/style-showcase/README.md) demonstrates the current typed
Views and finite app-owned Style vocabulary with external build-time CSS.
Neither is a public style crate or renderer change. Ordinary CSS stays default.

## M25 published DOM and static-export compatibility

The two-package SDK first delivered the expanded DOM and optional per-mount
navigation/image policies in **0.9.0-alpha.6**. Native DOM validation is
provisional; Wasm mounting enforces canonical allowed origins and frozen
destinations before mutation. Existing entry points preserve default-disabled
images, same-origin navigation/forms and their prior public signatures/error
types. The 89 default-admitted tags become 90 only when the image policy
explicitly admits `img`; neither count implies comprehensive HTML conformance.
See the [current DOM guide](../crates/yetlet-browser/guides/dom.md) and
[dated coverage inventory](YT-219-HTML-INVENTORY.md).

Pre rendering now preserves an initially parsed newline, including CR/CRLF,
where the HTML parser would otherwise consume it. The serialized bytes of
affected `pre` content can change without changing the intended parsed text;
the broad native SSR vocabulary and escaping rules are not globally narrowed.

Independent CLI **0.2.0** first delivered `yetlet build --static`. Its
`Cli::BuildStatic` enum variant may require a new arm in exhaustive downstream
matches, but existing build/run/watch/new and public SDK View/SSR/browser APIs
retain their boundaries. See the [static CLI guide](CLI-GUIDE.md#static-html-build)
and [retained technical protocol](YT-222-STATIC-HTML-CONTRACT.md). Neither
publication is new runtime/browser/production certification. The original
[SDK alpha.6](ALPHA6-RELEASE-PREPARATION.md#owner-approved-publication-and-registry-consumer--2026-11-01)
and [CLI 0.2.0](CLI-020-RELEASE-PREPARATION.md#owner-approved-publication-and-installed-consumer--2026-11-02)
records distinguish source tests from registry consumers. Dated unshipped
checkpoints are available in [immutable Git history](https://github.com/bohewu/yetlet/blob/b9873f15bcfd4c36a2274508ef7517e5a83b807b/docs/YT-221-DOM-URL-IMAGE-CONTRACT.md)
rather than duplicated as current compatibility status.

## Historical two-package pre-publication source — 2026-10-30

The unreleased development train is `0.9.0-alpha.5`, with exactly `yetlet` and
`yetlet-browser`. [YT-211](YT-211-TWO-PACKAGE-MIGRATION.md) records the intentional
crate/module import migration, one shared View and native/browser feature
boundaries. Base defaults select HTML; Browser defaults select DOM. Applications
still opt into capabilities and startup installation explicitly. There are no
compiling old-crate aliases, mandatory providers or new runtime protocols.

The [alpha.5 plan](ALPHA5-RELEASE-PLAN.md) separates accepted local migration
proof from the current qualification at R `0f0c84a` and remaining release gates.
See that record for actual C selection; tag, signer and alpha.5 signatures or
publication remain unselected. Experimental versioning does not promise API
stability or production readiness, and unchanged runtime semantics do not make
old artifacts/signatures attest to a new package layout.

The existing [alpha.4 GitHub pre-release](ALPHA4-HOSTED-RECORD.md) keeps its
sixteen-package source, twenty public assets and signatures. Only
`yetlet-actions0.9.0-alpha.4` reached crates.io; its fifteen-package continuation
is paused. The new names are not registry aliases for those old crates. Existing
platform/GPU/provider observations retain their original inputs; see the
[current feature/platform proposal](ALPHA5-RELEASE-PLAN.md#proposed-feature-and-platform-boundary).

## Historical version and candidate checkpoints

The dated entries below describe their own source/package sets and acceptance
states. References to isolated workspaces or earlier current candidates do not
override the two-package source and qualification boundary above.

**Current checkout (2026-10-29):** The owner approved the coordinated
`0.9.0-alpha.3` [version/package preflight](ALPHA3-VERSION-PREFLIGHT.md).
This changes library identities and internal version requirements, with no API,
runtime, feature, MSRV or third-party dependency upgrade. No new immutable C or
release authorization follows. Consumers must select the prerelease explicitly,
for example `=0.9.0-alpha.3`, once those packages are actually distributed.

Immutable alpha.2 is an unpromoted candidate with a retained
[custody-binding failure](ALPHA2-HOSTED-RECORD.md). Its tag and original evidence
are preserved; the alpha.3 correction does not rewrite them.

The published baseline remains the sixteen-library Experimental
[v0.9.0-alpha.1 GitHub pre-release](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.1)
at `bf3363801132f4481263ceb411d9bba3349df86d`. Its D6 acceptance is Complete;
crates.io/YT-020 remains Blocked. New schema-1 hosted signing-result verification
is Pending. The dated candidate/policy sections below retain their historical
sources, decisions and failures; they do not override this current status.

**Owner-selected version update (2026-10-28):** The next sixteen-package
Experimental preview is `0.9.0-alpha.1`; see the [alpha version preflight](YT-096-ALPHA-VERSION-PREFLIGHT.md).
The owner now selects guarded 59-path commit `5d7b91842f8188d45e5af22367245aebec7ca7d1`
(tree `6adff26fc4fec65e9aa8f40d7172ff266b7e500d`) as the new immutable alpha candidate C.
The [minimum hosted/readback plan](YT-096-ALPHA-HOSTED-PLAN.md) records the
remaining fixed-binding patch, consumed Guard-state disposition and separate
execution-source selection/authorization gates. Existing `v0.1.0`, source
`3eacfe3`, hosted/readback runs and retained bytes stay historical and unchanged;
no alpha output or signing payload inherits their identity. No new execution C,
alpha tag, hosted run or payload SHA is selected by the proposed future patch. The separate fixed-SHA signer design still applies. D6 Pending;
YT-096 In progress; YT-020 Blocked. No tag, hosted execution, signing or publication
is authorized by this migration.

The alpha train retains Rust 1.99 and the existing API/feature/security contracts.
Version `0.9` is not a production or API-stability guarantee. Internal dependency
requirements explicitly include `0.9.0-alpha.1`; consumers selecting this preview
must also opt into a prerelease requirement (for example `=0.9.0-alpha.1`).
The dated `0.1.x` policy and results below describe the previous prepared train;
no prior support or release approval is transferred by changing the number.

**Date:** 2026-10-10
**Scope:** YT-062 / M10c, with YT-020 private preparation selected on 2026-10-11.
**Status:** Prepared policy for experimental `0.1.0` and the original nine library candidates. Public activation remains blocked; no released supported version exists.

The maintainer authorized [Private release preparation](YT-020-PRIVATE-RELEASE-PREP.md), not publication or a visibility change. Existing API classifications remain unchanged.

**Current package preparation (2026-10-26):** [YT-201](YT-201-PACKAGE-PREPARATION.md)
aligns sixteen library manifests to experimental 0.1.0 with compatible registry
dependency requirements and unchanged independent workspaces/default features.
They remain unpublished. This extends preparation, not the immutable ce56e08
candidate, support classifications, final source C or release authorization.

## Historical release train

Yetlet-owned libraries share one coordinated release version and release train.
Their manifests may inherit the root version or declare it in an isolated
workspace; coordinated versioning does not merge their dependency graphs.
Runnable examples, benchmark fixtures, generated WASM glue and Python harnesses
are validation assets, not independently versioned public packages.

**Current candidate selection (2026-10-27):** The owner selects immutable
`118f3d0f496b487ef20fe243f4e73179f9eadc43`, tree
`07faa3e90c0fca5d27a4e55262c75acf851d884a`, after the guarded rebuild-extension
commit. The [verification order](YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md#new-candidate-and-remaining-verification-order--2026-10-27)
preserves the sixteen experimental 0.1.0 libraries and selected platform/feature
limits. Earlier input-bound results remain historical. New-C required CI, sixteen-package
hosted/local byte comparison and affected MSVC consumers now PASS;
[final independent acceptance](YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md#authorized-current-c-execution--2026-10-27)
is PASS with zero issues/required followups, closing selected YT-205 technical
scope. Platform limits remain unchanged and D1–D6 remain Pending.

**Earlier candidate selection (2026-10-27):** The owner selected the sixteen
experimental 0.1.0 libraries at immutable `a2d6474` for
[technical evidence consolidation](YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md).
The original nine-package ce56e08 records retain their identities. Selection
does not complete current-source CI/rebuild gates, expand platform support or
authorize signing, registry publication or activation; D1–D6 remain Pending.

The current `0.y` version remains experimental under SemVer. The classifications in [API Contracts](API-CONTRACTS.md) impose repository discipline but do not yet promise downstream compatibility.

For the selected `0.1.x` train, patches preserve documented API behavior and MSRV. An incompatible change requires the next coordinated minor release with rationale, migration notes and the gates below. Publication alone does not promote an Experimental API to Stable Candidate or promise 1.0 stability.

### Historical isolated optional workspaces

The optional Redis Session provider and `yetlet_browser::wgpu` use **isolated nested
Cargo workspaces/locks** and Experimental contracts. Both belong to the current
sixteen-library GitHub preview cohort, while the original nine-package records
remain historical. Coordinated versioning does not merge dependency graphs,
enable optional features or expand their selected platform/GPU support. The
default HTML/DOM/server dependency paths and root lock remain independent. See the
[YT-081 source/API audit](YT-081-API-CONTRACT-AUDIT.md) before assuming
cross-provider interchangeability or a GPU stability guarantee.

## Change discipline

A change to a **Stable Candidate** requires all of the following before merge:

1. explicit compatibility impact and rationale;
2. a migration note for downstream callers;
3. updated compile-level compatibility fixtures;
4. complete task validation;
5. independent read-only reviewer PASS;
6. an appropriate version-policy decision once public releases exist.

Experimental surfaces may change as evidence expands, but their classification and affected examples/tests must be updated. Internal/support-only and broad upstream reexports carry no Yetlet-owned compatibility promise.

## Server and upstream boundary

Yetlet owns its small handler-first facade, renderer-neutral values and documented provider entry points. It does not own the complete Axum/Tower/http API surface. Broad reexports exist for native interoperability and follow pinned upstream versions; callers needing direct upstream contracts should import those crates directly.

No compatibility policy authorizes a second router, model binder, Controller hierarchy, DI container, plugin registry, ORM, generic forms runtime or arbitrary document-head framework.

## Document contract

The complete-document API owns only:

- an escaped title;
- a validated root language;
- fixed UTF-8 charset and responsive viewport defaults;
- the existing strict CSP response policy.

`Page::new`, `App::page`, `render_document` and `render_page` retain the legacy `zh-Hant` default. Applications needing another root language use `LanguageTag`, `DocumentMetadata`, `Page::with_metadata` or the metadata-aware render functions.

`LanguageTag` implements a conservative ASCII shape suitable for common HTML language tags. It is not a complete BCP 47 registry or canonicalizer. Arbitrary meta/link/script/style/head nodes, raw head HTML and per-page CSP are outside the contract.

## MSRV policy

The maintainer selected Rust 1.99 as the minimum on 2026-10-12, before any `0.1.0` publication, to unify consumer, development, packaging and qualification toolchains. `rust-toolchain.toml` pins Rust/Cargo 1.99.0; root and isolated Redis/qualification manifests agree. This is an explicit pre-release baseline decision, not dependency drift or a published patch increase. Existing Rust 1.88 verification remains historical evidence.

The selected experimental `0.1.x` policy preserves Rust 1.99 in patches. After release, an MSRV increase requires a coordinated minor release, a documented technical/security reason, migration notes and CI evidence; no silent increase is permitted. Downstream preparation using 1.88 must install 1.99 or newer. [Integrity preparation](YT-020-INTEGRITY-PREPARATION.md) retains the original local checks. Maintainer PR #6's first candidate `9ec9b7b` passed both jobs in run `38193149046`, actual tree-equal synthetic checkout `cdc67bb6`. Remote protection was then atomically migrated to `Rust 1.99 MSRV` and `Stable native, samples, and Chrome`, with every other protection unchanged. Later candidate updates still require their own actual CI PASS; this first-head result is not relabeled as their execution.

## Historical prepared maintenance and qualification scope

After separate public activation, the latest approved `0.1.x` patch is the maintained experimental line on a best-effort basis; older patches are upgrade targets rather than separate maintenance branches. No production support, security response deadline or SLA is promised. Until publication, reports target private `master` and there is no released supported version.

The [Private preparation](YT-020-PRIVATE-RELEASE-PREP.md) selects the bounded native and Linux desktop Chrome/Firefox qualification/reporting scope. The [qualification matrix](QUALIFICATION-MATRIX.md) retains actual versions and source identities. That selection does not make historical lanes current-source execution or promise arbitrary versions, mobile support, device certification or accessibility conformance. Optional real IME/AT/WebKit/mobile/zoom cells remain Not verified.

## Compile fixtures

`crates/yetlet/tests/server_axum_public_api_compat.rs` compiles and exercises both the legacy and explicit-metadata server entry points. It is a focused source-compatibility fixture, not an ABI promise or a substitute for a future API-diff tool.

## Historical activation blockers

Licensing/security policy (YT-060) and private-state maintainer-only governance (YT-061) are complete and independently reviewed. Release design/preflight (YT-064) is complete at that scope, not signed publication. YT-065 is complete at its bounded automated desktop/native/browser scope; real-device/IME/assistive-technology/WebKit/mobile/zoom evidence remains optional and Not verified. YT-066 is Done at single-host Docker multi-instance and selected Redis-provider scope; broader deployment durability/HA/TLS/recovery remains Not verified. The selected experimental policy is prepared privately, with [remaining activation gates](YT-020-PRIVATE-RELEASE-PREP.md#remaining-yt-020-blockers) for immutable source, registry resolution, release integrity, public reporting/scanner controls and explicit approval. The completed YT-067 scan/fix retains its source boundary; no external certification or post-patch whole-repository scan is claimed. Public release YT-020 remains Blocked.
