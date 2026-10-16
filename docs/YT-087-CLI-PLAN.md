# YT-087 — Unpublished Yetlet Developer CLI Preview Plan

**State (2026-10-16):** Design independently reviewed PASS after two corrected blockers; YT-087a is Done at Linux local preview scope after the independent read-only Reviewer `wc_job_eWTKCMqqc5Ox5Bzn` PASS with zero issues/followups (see [YT-087a verification](YT-087A-VERIFICATION.md)). [YT-087b verification](YT-087B-VERIFICATION.md) records independently accepted Linux frozen build/dev after a host PTY/HTTP continuation and focused resistant-SIGINT correction; [087c documentation and validation](YT-087C-VERIFICATION.md) also received final independent Reviewer `wc_job_avwDfY3wH233pvN7` PASS (zero issues/followups). YT-087 is Done at the bounded Linux-only unpublished CLI preview scope, not public-release or Windows-support scope. The maintainer authorized starting YT-087 after M12 commit `94cf8f555b554a26c2416842eab06110b229904d`. This document is not a claim that a public binary exists or that YT-020 release gates have passed.

## Goal and non-goals

Offer a small Rust-native `yetlet` developer CLI for the **handler-first, HTML-first** application path. Its first usable vertical slice scaffolds one native SSR application and diagnoses its local prerequisites; a later slice handles foreground development and native builds. Browser WASM, Session/Auth, MVC/Controllers, DI, WebGPU, watchers/HMR and deployment remain explicit opt-ins or out of scope.

YT-087 does **not** authorize crates.io publication, global installation, tag, Git push, new CI/hosted execution, credential acquisition, automatic dependency downloads, or claiming stable runtime/production support. YT-020 remains Blocked; this CLI is an unpublished local preview. The M12 `cargo xtask` command remains internal and unchanged.

## Proposed user commands (not yet a frozen API)

```text
yetlet doctor [--framework-path PATH] [--json]
yetlet new NAME --framework-path PATH [--dir DIRECTORY]
yetlet build --manifest-path Cargo.toml [--release]
yetlet dev --manifest-path Cargo.toml
```

The first-run path is deliberately explicit: `yetlet new APP --framework-path FRAMEWORK`, followed by `cargo generate-lockfile --offline --manifest-path APP/Cargo.toml` using already cached dependencies, then `yetlet build --manifest-path APP/Cargo.toml`. No hidden initial lock creation, Cargo build or registry resolution occurs inside `new`. The application's lockfile is its own, never the root Yetlet or CLI lockfile.

- `doctor` is read-only and checks the installed Rust/Cargo identity, required Rust 1.99 toolchain and an explicitly selected local framework checkout; it reports missing capabilities as failures, never installs or downloads. JSON diagnostics must not contain secrets or unredacted private paths.
- `new` creates a single minimal, handler-first native SSR project: ordinary Axum/Tower handlers, Yetlet `View`/`App`/`Page` where supported, safe escaped HTML, strict CSP, default loopback listener and **no JS/WASM**. No Controller/DI/provider scaffolding. Source templates are embedded in the CLI; the command does not call Python/Node, shell fragments, Cargo build or network operations implicitly.
- Because the Yetlet crates are not published, `new` requires `--framework-path` and writes explicit local path dependencies. It must not silently invent registry or Git dependencies. The generated project explains this preview limitation and how to relocate the checkout; a registry-backed mode is a later release decision.
- `build` launches `cargo build --frozen` with literal argv and explicit manifest/cwd for an application that already has its own `Cargo.lock`, without depending on `xtask`. It rejects an absent/stale lockfile or uncached dependency with actionable nonzero diagnostics, never silently calls lock generation or downloads. `--release` selects Cargo's release profile.
- `dev` is one owned foreground `cargo run --frozen` invocation using the same existing application lockfile; it is **not** a file watcher or hot-reload server. Correctly propagate child exits and Ctrl-C, reap Cargo and its owned application descendants and preserve diagnostics. No process-name-based kills, detached services, automatic browser launch or background agent.
- Unknown commands/options are usage errors rather than ignored flags. Public help and `doctor` never start a server or build generated applications.

## Candidate crate layout

`tools/yetlet-cli/` is a separate unpublished Rust Cargo workspace with a binary named `yetlet`, its own lockfile and tests. Keep it separate from the root framework dependency graph and the existing unpublished `xtask`; avoid adding a mandatory CLI dependency to application crates. Reuse Rust stdlib and mature parsers where justified, not a generic plugin framework or application generator engine.

Generated `Cargo.toml`, `src/main.rs`, minimal README and ignore rules must use the real current `yetlet-core` and `yetlet-server-axum` interfaces, plus explicit Axum/Tokio dependencies. Do not copy root workspace-only dependency inheritance into a standalone application. Derived paths must work when checkout or project directory contains spaces or non-ASCII characters.

## Security and failure contracts

1. Validate Cargo package name and destination before writing. Reject traversal, symlink escapes, existing/nonempty destinations, unsafe device names where applicable, and absolute/relative ambiguity. Do not overwrite user data or follow unsafe symlinks.
2. Stage only under a verified, symlink/reparse-safe parent into a uniquely owned temporary sibling. Publish with a **non-replacing, atomic directory move**: on Linux use a parent-directory-handle-relative no-replace operation such as `renameat2(RENAME_NOREPLACE)`; on Windows use an OS-verified non-replacing move with reparse-safe parent handling, or fail closed if that guarantee cannot be established. An existence check followed by ordinary replacing `std::fs::rename` is forbidden. Hold/revalidate trusted parent identity through publication; a parent symlink/junction swap must fail closed. On failure clean up only the exact owned temporary tree after confirming its identity, never remove an unrelated or ambiguously replaced destination. Unknown publication outcome is nonzero and demands inspection, not blind retry/rollback.
3. Quote and escape generated TOML/Rust values correctly. Never interpolate shell commands, project names or local paths into scripts. Keep generated application defaults loopback-only and preserve CSP and HTML/state escaping.
4. Use literal argv, explicit cwd and bounded subprocess cleanup. Missing executable, bad Rust version, failed build, nonzero child exit, timeout and cancellation remain distinct errors; no success-shaped stubs or retries after uncertain mutation.
5. No network/install by `new` or `doctor`. A fresh scaffold deliberately has **no** `Cargo.lock`: its generated README must direct one separate, explicit `cargo generate-lockfile --offline --manifest-path APP/Cargo.toml` step, which succeeds only with a sufficient local Cargo cache. Subsequent `build`/`dev` always use `--frozen`, require that application lock and fail closed on missing/stale lock or cache without network fallback. This bootstrap must not change root `Cargo.lock`, `xtask/Cargo.lock` or CLI's own lockfile. Online dependency resolution is deferred.

## Ordered YT-087 slices and evidence gates

| Slice | Deliverable | Required evidence |
|---|---|---|
| 087a | CLI crate, literal parser/help, `doctor`, safe `new` minimal SSR scaffold | Rust unit tests: names, paths/traversal/symlink/reparse, overwrite and parent-identity races, **destination appears immediately before publication** (no overwrite), ambiguous rollback, spaces/Unicode and manifest escaping; actual fresh external project explicitly generates its own offline lockfile and compiles with `--frozen`, native SSR GET returns escaped HTML with CSP and no JS/WASM; root/CLI locks unchanged; independent review |
| 087b | `build` and foreground `dev` under explicit manifest | Actual local generated app `cargo build/run --frozen`, missing/stale-lock and cache-unavailable negatives, child native error propagation, Ctrl-C/owned teardown including Cargo descendants, no orphan listener and no hidden watch/install; Windows native process and no-replace publication behavior tested if capable or reported Not verified; independent review |
| 087c | CLI docs, consistency, isolated tests and final acceptance review | Offline locked fmt/check/strict Clippy/test, docs/sample checks, no Python/Node invocation on executed commands, no root graph regression; independent Reviewer PASS with zero blockers before YT-087 Done |

Use existing cached toolchains only; if dependencies or Windows execution are unavailable, disclose Not run/Blocked rather than changing host/global configuration or claiming cross-platform PASS. Preserve Linux and Windows evidence separately. Release/publication still belongs to YT-020 and an explicit future approval.

## Scope and decision gate

### First independent design review and required correction

Read-only independent Reviewer Job `wc_job_YVAgSOYdrlfPO5qz` returned **FAIL** (R1: unspecified first application lockfile, R2: overwrite/symlink race during directory publication). The explicit offline `cargo generate-lockfile` bootstrap and non-replacing, parent-anchored publication contract above are corrective design requirements, **not implementation or new validation evidence**. These corrections must pass a distinct focused design re-review before 087a can be accepted.

The separate focused design re-review Job `wc_job_-ZsSRbXwI3fDdoBQ` returned **PASS**, zero issues/followups, closing R1/R2 at **design scope only**. The YT-087a code was subsequently implemented and locally checked as documented in [YT-087a verification](YT-087A-VERIFICATION.md); a new independent implementation review is required before considering that slice accepted.

First review this design against ADR 0006, the real hello-world and App/Page APIs, the root build contract and existing M12 tooling. A separate independent Reviewer must approve the planned command semantics and honest unpublished scope before treating implementation as accepted. Resolve reviewer findings without broadening architecture. At each implementation slice keep original receipts and source identity; no commit or push without a fresh explicit request.
