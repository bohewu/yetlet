# M12 — Rust Repository Toolchain Plan

**Task:** YT-082. **State:** Done after independent document review PASS. No implementation item is accepted by this planning result.
**Baseline:** clean `master`, `7cfac613f908c5c19fa581438288386809870b56`, current checkout (`$HOME/repos/yetlet` on the recorded Ubuntu runner).

The owner requests removal of Python from the **active repository developer build,
verification and release-check toolchain**, using Rust tooling first and a public
CLI only later. The directional backlog review passed with no issues or required
followups. That initial PASS approves the proposal only; a subsequent independent document review accepted this plan. Neither review
implement a command, or close YT-083–YT-086. See the exact review and baseline in
[YT-082 verification](YT-082-VERIFICATION.md) and the complete
[52-file migration ledger](M12-PYTHON-MIGRATION.md).

## Authority and boundaries

| Source | Requirement and concrete reference |
|---|---|
| `explicit_user` | Parent request: review the suggested backlog, then proceed through YT-086 if sound; replace the repository Python toolchain with Rust before a public CLI. The full approved proposal is retained at `target/m12/yt082/backlog-review/prompt.txt`. This assignment executes **YT-082 documents only**. |
| `explicit_user` | `validation_infrastructure.authorized=true`: migration of the toolchain itself, including its existing tests, builds and CI. This permits later focused Rust product tests for the migrated tools, not a new generic validation framework. Source: the current YT-082 assignment and the approved proposal's original-goal paragraph. |
| `existing_contract` | [AGENTS.md](../AGENTS.md), [ADR 0003](adr/0003-renderers-and-loading.md), [ADR 0006](adr/0006-handler-first-mvc-optional.md), [ADR 0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md): HTML-first runtime, separate browser/assets/transport, optional dependencies, strict CSP, escaping and evidence-gated scope. |
| `existing_contract` | The ledger's tracked scripts, direct tests and three workflows at baseline define actual scenarios, bounds, manifests, negative cases and reporting behavior. [Sample catalog](../examples/samples.toml), [sample rules](SAMPLES.md), [compatibility](COMPATIBILITY.md) and [release supply chain](RELEASE-SUPPLY-CHAIN.md) remain contracts. |
| `necessary_compatibility` | `qualification/yt066b/tests/cookie_fencing.rs` directly launches `tests/cookie_browser.py`; browser and qualification scripts import each other. A Rust command that leaves these callers or helpers on Python does not remove the dependency. Live root/sample/AGENTS commands must change only with working replacements. |

No Rust source, Cargo configuration/lock, CI, sample, browser/server or profile is
changed by YT-082. No installation, download, build, software suite, qualification,
public release, commit, push, hosted trigger, cloud operation or worktree follows
from this planning delivery. The current root native `cargo build`, `cargo run`,
tests and existing live Python commands remain usable until their replacements
prove parity. Historical evidence is not rewritten as a fresh Rust test result.
YT-020 stays Blocked. YT-087 (`yetlet new/dev/build/doctor`) is Deferred, unfiled
and not activated; no public CLI, framework API, GPU support or 3D promise is added.

## Ordered slices and admission gates

`YT-081 → YT-082 → YT-083 → YT-084 → YT-085 → YT-086`; future YT-087 is outside
this chain's execution authority. Each item needs its **own independent Reviewer
PASS** before Done or dependent implementation. Parent review owns YT-082
acceptance and bookkeeping. Filing these tasks is not their execution.

| Item | Bounded deliverable | Required evidence before acceptance |
|---|---|---|
| YT-082 | This repository plan, exact inventory/mapping, rollout and documented checks; backlog, roadmap and index updates. | Existing docs checker plus all new docs in memory; exact 52-path mapping; tracked and untracked whitespace; independent document review. No software tests claimed. |
| YT-083 | Unpublished isolated Rust `xtask`, cargo alias, help/read-only doctor and owned subprocess foundation. | Focused Rust tests of literal argv/cwd, spaces/non-ASCII, stdout/stderr, spawn/child failure, timeout/cancel and descendant cleanup. Actual Linux **and Windows owned-child** validation; unavailable execution is Blocked/Not run, never PASS. Ordinary root Cargo commands still work and tool dependencies stay isolated. |
| YT-084 | All seven builders, docs/catalog checks and corresponding Rust tests; live build/catalog command adoption after parity. | Compare selected identical inputs with old tools during transition, including debug/release where supported, separate entry builds, lock/CLI compatibility, bytes/hashes/schema/escaping/security and existing negative catalog cases. No canonical Rust path may invoke Python or Node. |
| YT-085 | All active native/browser smoke and integration tooling, imported protocol/PNG helpers, and Rust-test browser fixture counterparts. | Preserve each ledger suite's assertions and outputs, port the cleanup tests, run focused native tests and relevant actual existing Chrome/Firefox journeys on available capabilities. Verify cleanup on success/failure/timeout. Missing browser/GPU capabilities are nonzero and Not run/Blocked, never a successful skip. Do not replace suites with generic smoke. |
| YT-086 | Benchmarks, qualification, release checksums/private rebuild, their tests, remaining transitive callers and CI invocation switch. | Frozen-input arithmetic/security/negative tests plus one bounded relevant runtime case, exact invocation-graph audit, controlled-PATH/blocked-Python sentinel exercising canonical commands, no lost assertions or unsupported placeholders. Preserve three workflows' gates and limits. No 62-minute load, physical IME/AT or cloud requalification merely for a language port. Independently reviewed migration closure; operational-envelope claims remain historical and bounded. |

The existing qualification matrix is not enlarged for this port. In particular,
Windows foundation validation must use installed, task-local capabilities rather
than change global defaults: owner-reported Ubuntu Rust 1.99 and Windows Rust
1.88/1.99 installations are context, **not new execution evidence**. Select the
installed 1.99 toolchain explicitly for this root; do not attempt to install or
switch the machine's global default. Historical Rust 1.88 evidence stays bound to
its original isolated consumer. An unavailable Windows execution path blocks the
required foundation gate instead of being papered over by Unix tests.

## Repository command layer and dependency isolation

Proposed implementation: `xtask/` is one `publish = false` crate with its own
`[workspace]`, `Cargo.lock` and target directory. A root `.cargo/config.toml` alias
will be equivalent to `xtask = "run --locked --manifest-path xtask/Cargo.toml --"`.
It is **root-only**: Cargo's relative manifest argument is relative to the invoking
cwd; inheritance of the alias into a subdirectory does not make that argument
root-relative. From a subdirectory use
`cargo run --locked --manifest-path /absolute/checkout/xtask/Cargo.toml -- <command>`
(quote the absolute path when needed), or change cwd to the repository root.
The Rust program locates its own repository from its manifest location, not from
an assumed caller cwd. Relative user output/directory arguments resolve against
the invocation cwd; internal builds run with the explicit correct root/nested cwd.

`cargo xtask` displays help without launching work. `cargo xtask doctor` checks
the installed Cargo/rustc identity and root toolchain compatibility read-only;
`doctor --capability <name>` additionally checks the selected command's tools,
targets, browser/driver, OS primitives and manifest/lock requirements. No option
silently installs anything. Cargo may compile the tool before doctor starts;
read-only refers to doctor's behavior, not a claim that `cargo run` writes no
build cache. Locked/offline invocation with pre-resolved dependencies is needed
where network-free Cargo bootstrap is required. Missing cached dependencies must
fail rather than trigger an automatic recovery install.

The explicit subcommands are `build`, `check`, `test`, `smoke`, `measure`,
`qualify`, `release`, and narrowly internal `fixture`; their exact names and
old-path counterparts are in the ledger. Names here are proposed commands, not
interfaces already installed. Keep parsing, process ownership and each command's
logic in the one tool; share a small helper only for actual repeated protocol or
lifecycle behavior. No plugin framework, DI layer, generic test orchestrator,
shared public tooling crate or speculative public CLI foundation is needed.

Tool dependencies belong to this isolated development crate; its own test-only
dependencies belong in its dev-dependencies. No runtime crate depends on xtask,
and xtask need not depend on framework crates. Do not join it to the root graph
or perturb default production feature resolution to simplify tooling. Compare
root framework dependency/feature selection before/after; separately built
Browser/Search and nested Experimental scene assets retain independent builds.
No new ADR is necessary: this implementation plan preserves existing framework
architecture and selects only isolated repository automation. Revisit an ADR only
if later work actually changes a fundamental boundary.

## Command, error and process contracts

- Keep each old script's options/defaults/bounds and debug/release choice on its
  mapped command. Unsupported flags are usage errors, not ignored arguments.
  Native sample run commands remain ordinary Cargo commands. Nothing launches a
  browser/server implicitly through help, doctor or a documentation check.
- Success is exit 0 only after the command's full checks and cleanup pass. Tool
  validation/capability failures exit 1, usage exits 2, tool-owned timeout exits
  124, and spawn/cleanup infrastructure failure exits 125. Child failures retain
  their exact native exit code where representable and also report the child
  status; reserved-code collisions retain the origin in diagnostics. Unix child
  signals retain signal identity and nonzero `128 + signal` status; cancellation
  is never swallowed. Windows exit/termination identity is retained, including
  values Cargo cannot faithfully relay. Cargo's wrapper status alone is not the
  child-status record. Existing failed/not-verified report semantics survive.
- Use literal argv and explicit cwd/environment, never shell concatenation,
  Python wrappers/embedding/translation or Node/JS interpreter workarounds.
  Browser-side JS evaluation/loaders are existing browser product/test logic and
  remain browser-side. Preserve executable discovery, configured environment and
  documented existing fallback paths; verify identities rather than trust names.
- Keep stdout reports/summaries and stderr diagnostics distinct; concurrently
  drain child output to avoid pipe deadlocks. Retain required logs and failure
  codes without dumping secrets, synthetic login codes, raw Cookies, private
  paths or unredacted exceptions where the old suite redacts them. Existing
  sanitized forms/qualification report schemas remain supported.
- Bound readiness, HTTP/protocol waits, subprocess waits and cleanup using
  monotonic deadlines and existing per-suite budgets. Timeout never retries an
  uncertain mutation. Retain each suite's stage reporting and diagnostic files.
  New generic deadlines must not truncate an explicitly selected old profile.
- Register ownership immediately on spawn, including partial startup. Use owned
  Unix process groups and Windows Job Objects/equivalent scoped descendant
  ownership; terminate, escalate, wait/reap, close streams, delete owned profiles
  and confirm owned listeners/processes gone on success, error, timeout and
  cancellation. Never kill by executable name or attach to a user browser. A
  cleanup failure prevents PASS even if assertions passed. Do not require host
  policy changes for Windows; report unavailable primitives honestly.
- Handle spaces/non-ASCII and native path separators without lossy string shell
  conversion. Keep catalog/build source paths repo-contained, reject traversal
  and unsafe symlink escape. Release artifact inputs have their stricter explicit
  directory/no-follow rules below. Do not weaken those for cross-platform ease.

## Output and compatibility contracts by family

**Build/assets (YT-084).** Snapshot exact selected source bytes and applicable
Cargo manifest/lock, tool versions, mode, argv and cwd for comparison. Independently
build every entry: Todo versus Search, Dashboard Counter versus Filter, and nested
scene versus root counter. Preserve `--locked`, scene `--offline`, target/profile,
build-time assets, URL prefixes, cache/content type, output roots and all manifest
schema/version fields. The applicable root/nested lock must agree with the
installed wasm-bindgen CLI, currently **0.2.129**; absence, ambiguity or mismatch
is nonzero. Preserve the sorted filename/NUL/bytes digest algorithms and domain
prefixes (`yetlet-v1`, `yt075-v1`, and each isolated builder's own prefix), full or
truncated hashes, JSON formatting where bytes are consumed, generated HTML/Rust,
source hash inventories and no unrelated asset changes. Differences need explicit
parity explanation/approval, not blanket regeneration of expected hashes.

Form-proof and Board CSS builders preserve pinned Bootstrap 5.3.8 SHA-384,
license/content limits, allowlisted SVG tags/attributes, externalization, no
remaining data URLs/imports, registered same-origin URLs, output MIME inventory,
content-addressed names and generated `asset_data.rs`. `--check` remains read-only
and rejects missing/extra/stale bytes. No auto downloads/installs: ordinary builds
use present pinned inputs. The old explicit `--fetch` operation is a separate
opt-in pinned-input acquisition, mutually exclusive with `--check`, and requires
execution authority; this plan does not run it. It must never become a fallback.

**Docs/catalog (YT-084).** Preserve English-only coverage and excluded directories,
required file/newline/fence/whitespace/link checks, unique backlog IDs, dependency
existence and cycle checks. Preserve catalog schema/type/duplicate/companion,
route/source-marker and README exact-command matching, actual optional dependency
closure and repo-contained paths. `browser_build` currently names a `.py` file;
its planned Rust command representation and corresponding README/catalog/checker
change must land together with parity, preserving command existence and declared
build identity rather than deleting this validation. No sample is dropped to
avoid migrating its build. Negative cases remain represented by Rust fixtures.

**Native/browser suites (YT-085).** The ledger names separate suites. Preserve
SSR/DOM equivalence, local-only event boundaries, typed HTTP failures, CSP/XSS,
lazy loading and invalid-state fallback; Todo ownership/races, keyed identity,
focus/selection/disposal; independent Dashboard lifetime/malformed manifests;
forms keyboard/Unicode/semantic labels and sanitized reports. Keep Chrome CDP
WebSocket framing/event/error handling and Firefox WebDriver/BiDi operations
actually used by the current suites, their JS and PNG decoding/pixel assertions.
Do not add a Playwright/Node dependency or replace real pixels with compilation.
Native cases preserve auth/CSRF/IDOR, transactions/idempotency/rollback, file
bounds, SQLite restart/conflicts and SSE heartbeat/replay/disconnect behavior.
GPU lifecycle, picking, candidate and default-profile fallback remain distinct;
explicit experimental preference profiles never stand in for default support.
Missing GPU is an honest unavailable result, not fabricated GPU acceptance.

**Benchmarks/qualification (YT-086).** Preserve raw samples/counts, warmup versus
measured phases, percentile algorithms, seeds/order, output/source/binary hashes,
budgets, unknown-outcome semantics and non-overwrite rules. M5 measurements stay
informational; M7 captures production-before state and all equivalence/focus/sample
assertions. Sample DX stays a bounded source/dependency metric, not performance.
GPU dynamic comparisons retain all matched lanes, actual samples, deadlines and
PNG checks; no missing lane may be described as an inferior renderer or speedup.

Board profiles remain `bounded`, `full`, `full-sync`, `sync-12m`,
`staggered-12m`, with existing durations, pacing/cohorts and tracing distinctions.
Full retains 120-second warmup, three 600-second measured rounds and 1800-second
soak; 25 clients/10 SSE clients, 10,000 items/90,000 history envelope and original
strict gates: read p95 `<250 ms`, write p95 `<500 ms`, combined p99 `<1000 ms`,
unexpected fraction `<0.001`. SQLite WAL/FULL, no automatic write retry, raw tails,
SSE reconnect/relogin, cleanup and tracing limitations remain explicit. Keep the
original Ubuntu synchronized FAIL and separate Windows/Azure PASS in
[cross-environment evidence](YT-074-CROSS-ENVIRONMENT-VERIFICATION.md). No old
receipt is relabeled as a port run. A small fixture/runtime case proves migration
behavior only, never this 62-minute operating envelope, physical IME or spoken AT.

Local operations retain restart/fault classification, read-only/locked/corrupt
SQLite, online backup deadline and no overwrite. Multi-instance/Redis retain
loopback-only pinned disposable resources, Memory limits, CAS/retirement/TTL and
unknown outcomes, pure SSR no-JS Cookie ordering and exact-owner cleanup/rechecks.
Existing signal/redaction/report-write failure tests are product requirements for
the port; no generic smoke can replace them. No cloud deployment or new persistence
guarantee follows.

**Release/private rebuild (YT-086).** Preserve flat explicit-directory checksum
schema/notice, deterministic sorted entries, duplicate-key rejection, no overwrite,
no manifest self-reference, unsafe-name/traversal and nonregular/symlink rejection,
ancestor no-follow traversal, concurrent-file fingerprint checks and secret-safe
errors. Bounds remain 128 files, 64 MiB each, 256 MiB total and 64 KiB manifest.
Current POSIX no-follow prerequisites fail closed elsewhere; Windows subprocess
support does not imply an accepted Windows checksum implementation.

Port private-rebuild inline Python as Rust logic, including exact nine-package
allowlist, 18 license-link aliases and extraction/type/path checks, immutable
clean source/archive, selected-input canonical digest, no overlays, fresh Cargo
home/target, pinned 1.99 GNU tools/hashes, `SOURCE_DATE_EPOCH`, four jobs, locked
fetch then offline packaging, input recheck, exact archives, build logs/records,
unsigned provenance and checksum create/verify. Tool bootstrap must not pollute
selected package inputs, clean-source assertion or fresh package caches. Preserve
manual-only activation, immutable workflow/source/ref identities and approved ref
policy, `contents: read`, no persisted token, full-SHA actions, concurrency, timeout
and artifact retention; never broaden dispatch refs or permissions. This is
preparation, not signing/publication/trusted provenance. Hosted workflow install
steps are existing explicit CI provisioning, not license for xtask to auto-install
or for this task to trigger them. Preserve [release gates](RELEASE-READINESS.md).

## Adoption, rollout and final exit

1. Freeze each migrated family's selected inputs/tool identities at its actual
   transition baseline; this plan's baseline identifies the old contracts, not a
   future modified-source software acceptance. Keep old/new outputs separate.
2. Prove positive and existing negative parity with direct Rust tests/fixtures and
   bounded applicable runtime checks. Python comparison is transitional only.
   Do not mark an old path retired just because a Rust wrapper exists.
3. Switch that family's live README/AGENTS/sample/catalog references only after
   parity; keep historical commands with their original dates/source/results.
   Switch CI invocation in YT-086 after dependencies pass. Include transitive
   helpers and Rust test callers, not only visible top-level workflow commands.
4. In YT-086 exercise canonical commands with blocked `python`/`python3` (including
   Windows variants/absolute interpreter escapes in the audited process graph)
   and controlled installed-tool PATH. Exercise missing-capability negative cases:
   nonzero with honest Not run/Blocked, no silently skipped successful suite.
   This proves language independence, not operating-envelope qualification.
5. Close only when every ledger mapping and three workflows are accounted for,
   current commands/transitive processes need no Python or Node interpreter,
   required tests/platform evidence and each independent reviewer pass exist,
   root dependency/build compatibility is retained, and no assertions were lost.
   Historical immutable receipts may remain; no active implicit Python invocation
   may hide behind a historical label. Retain unmigrated statuses until proven.

During transition a failing new path blocks adoption; retain the functioning old
command and report the gap. Do not erase old tests or relax gates to reach a
Python-free count. No deletion is necessary for YT-082; later removal/retirement
requires proven parity and explicit ledger evidence. No open framework API
decision blocks this plan. Capability-specific implementation details remain
owned by their listed item and cannot increase scope or waive an existing gate.
