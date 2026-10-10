# Yetlet Compatibility Policy — Pre-Public Draft

**Date:** 2026-10-10
**Scope:** YT-062 / M10c
**Status:** Provisional policy for the private `0.1.0` workspace. It becomes a public compatibility commitment only after explicit release decisions.

## Release train

Yetlet-owned crates use one workspace version and one coordinated release train. Runnable examples, benchmark fixtures, generated WASM glue and Python harnesses are validation assets, not independently versioned public packages.

The current `0.y` version remains experimental under SemVer. The classifications in [API Contracts](API-CONTRACTS.md) impose repository discipline but do not yet promise downstream compatibility.

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

Rust 1.88 is the current declared and CI-verified MSRV. Before public release, an MSRV increase requires a documented technical/security reason and CI evidence. After public release, the release policy must state which version class may raise MSRV; no silent increase is permitted.

## Compile fixtures

`crates/yetlet-server-axum/tests/public_api_compat.rs` compiles and exercises both the legacy and explicit-metadata server entry points. It is a focused source-compatibility fixture, not an ABI promise or a substitute for a future API-diff tool.

## Still pending

Licensing/security policy (YT-060) and private-state maintainer-only governance (YT-061) are complete and independently reviewed. Release design/preflight (YT-064) is complete at that scope, not signed publication. Hosted/manual browser/OS coverage (YT-065), distributed-provider operations (YT-066), complete internal Codex-assisted security assessment (revised YT-067; no external certification), trusted release provenance and explicit public activation remain open. Public release YT-020 remains Blocked.
