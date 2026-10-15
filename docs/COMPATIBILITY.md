# Yetlet Experimental Compatibility Policy

**Date:** 2026-10-10
**Scope:** YT-062 / M10c, with YT-020 private preparation selected on 2026-10-11.
**Status:** Prepared policy for experimental `0.1.0` and the original nine library candidates. Public activation remains blocked; no released supported version exists.

The maintainer authorized [Private release preparation](YT-020-PRIVATE-RELEASE-PREP.md), not publication or a visibility change. Existing API classifications remain unchanged.

## Release train

Yetlet-owned crates use one workspace version and one coordinated release train. Runnable examples, benchmark fixtures, generated WASM glue and Python harnesses are validation assets, not independently versioned public packages.

The current `0.y` version remains experimental under SemVer. The classifications in [API Contracts](API-CONTRACTS.md) impose repository discipline but do not yet promise downstream compatibility.

For the selected `0.1.x` train, patches preserve documented API behavior and MSRV. An incompatible change requires the next coordinated minor release with rationale, migration notes and the gates below. Publication alone does not promote an Experimental API to Stable Candidate or promise 1.0 stability.

### Separately owned unpublished experimental workspaces

The optional Redis Session provider and the owner-approved
`yetlet-wgpu` candidate use **isolated nested Cargo workspaces/locks**,
`publish = false`, and pre-public Experimental contracts. They are not
among the original nine coordinated YT-020 release candidates; their
`0.1.0` (GPU) or separate provider version is not evidence that they
have entered the root release train. The default HTML/DOM/server
dependency paths and root lock remain independent. See the
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

## Prepared maintenance and qualification scope

After separate public activation, the latest approved `0.1.x` patch is the maintained experimental line on a best-effort basis; older patches are upgrade targets rather than separate maintenance branches. No production support, security response deadline or SLA is promised. Until publication, reports target private `master` and there is no released supported version.

The [Private preparation](YT-020-PRIVATE-RELEASE-PREP.md) selects the bounded native and Linux desktop Chrome/Firefox qualification/reporting scope. The [qualification matrix](QUALIFICATION-MATRIX.md) retains actual versions and source identities. That selection does not make historical lanes current-source execution or promise arbitrary versions, mobile support, device certification or accessibility conformance. Optional real IME/AT/WebKit/mobile/zoom cells remain Not verified.

## Compile fixtures

`crates/yetlet-server-axum/tests/public_api_compat.rs` compiles and exercises both the legacy and explicit-metadata server entry points. It is a focused source-compatibility fixture, not an ABI promise or a substitute for a future API-diff tool.

## Still pending

Licensing/security policy (YT-060) and private-state maintainer-only governance (YT-061) are complete and independently reviewed. Release design/preflight (YT-064) is complete at that scope, not signed publication. YT-065 is complete at its bounded automated desktop/native/browser scope; real-device/IME/assistive-technology/WebKit/mobile/zoom evidence remains optional and Not verified. YT-066 is Done at single-host Docker multi-instance and selected Redis-provider scope; broader deployment durability/HA/TLS/recovery remains Not verified. The selected experimental policy is prepared privately, with [remaining activation gates](YT-020-PRIVATE-RELEASE-PREP.md#remaining-yt-020-blockers) for immutable source, registry resolution, release integrity, public reporting/scanner controls and explicit approval. The completed YT-067 scan/fix retains its source boundary; no external certification or post-patch whole-repository scan is claimed. Public release YT-020 remains Blocked.
