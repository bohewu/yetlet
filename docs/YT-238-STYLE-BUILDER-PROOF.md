# YT-238 — Application-local CSS builder proof

**Date:** 2026-11-08. **Status:** Done at prototype scope after independent
implementation Reviewer PASS. No public API is promoted. The owner separately
decides whether to promote work to YT-239.

This implements the bounded [YT-237 contract](YT-237-STYLE-REGISTRATION-CONTRACT.md)
and [YT-238 roadmap scope](YT-237-240-STYLE-CSS-ROADMAP.md#yt-238--minimal-rust-css-builder-proof)
in `feature/style-css-roadmap`, baseline
`3ecbfc36a91a2a06ef2dee69a7a0d848a2ecd16e`. Only the new
`experiments/style-authoring/` tree and this document are implementation sources.
Core, renderers, HTTP/CSP, root manifests, CLI and templates have no changes.
Ordinary CSS remains the default; this reference fixture is not a shipped
`yetlet new` capability. YT-239 integration and YT-240 qualification remain gated.

## Sources and default registration

| Source | Responsibility |
|---|---|
| [Generator manifest](../experiments/style-authoring/stylegen/Cargo.toml) and [lock](../experiments/style-authoring/stylegen/Cargo.lock) | Private standalone generator, empty workspace, no dependencies |
| [Builder](../experiments/style-authoring/stylegen/src/builder.rs) | Finite typed declarations, whole-output validation and ordered serialization |
| [Registry](../experiments/style-authoring/stylegen/src/styles/mod.rs) | Already calls `layout::styles()` then `dialog::styles()` |
| [Layout starter](../experiments/style-authoring/stylegen/src/styles/layout.rs) | Shell tokens, Flex toolbar, Grid content and minimum-width media |
| [Dialog module](../experiments/style-authoring/stylegen/src/styles/dialog.rs) | Conditional-only class, registered independently of View execution |
| [Shared names](../experiments/style-authoring/src/style_names.rs) | Target-neutral constants included by both dependency graphs |
| [View fixture](../experiments/style-authoring/src/lib.rs) and [manifest](../experiments/style-authoring/Cargo.toml) | Real Yetlet tag builders and conditional View; no server or executable application |
| [Plain CSS](../experiments/style-authoring/assets/plain.css) | Same complete declarations/order as generated CSS |
| [Publisher](../experiments/style-authoring/publish.py) | Developer-only Python stdlib tooling modeling a future coordinator |
| [Rust tests](../experiments/style-authoring/stylegen/tests/behavior.rs) and [publication tests](../experiments/style-authoring/tests/test_publish.py) | Focused changed-behavior coverage |

The default stylesheet entrypoint works immediately: both named modules are
already registered. The generator calls that root without importing Yetlet or
executing `page(false)` or `page(true)`. The View test constructs both branches:
only the open branch includes `app-dialog-open`. The generator test independently
checks all registered bytes against the plain-CSS fixture, including that class.
This proves the selected sample's superset; it cannot discover forgotten modules.

To add another module, create `stylegen/src/styles/example.rs` returning a
`Module` with a unique owner name, declare `mod example;` in the registry, then
append `example::styles()` at the intended cascade position. Put any View class
constants in `src/style_names.rs`. Creating a file alone does not register it.
Add its declarations to the expected fixture/test when changing this sample.
Repeated registration is an error even when data is equal.

For **future suitable Rust-style `yetlet new` templates**, the owner requires the
same working default entrypoint, registered starter definitions, shared names and
selected generator/configuration wiring to be emitted automatically. Actual
scaffolding and settings support belong to YT-239 after separate promotion
approval. Existing ordinary-CSS templates retain their default behavior.

## Implemented finite vocabulary

`Declaration` accepts block/flex/grid/none, common alignment/justification,
gap/padding/width/height, px/rem/percent, size-only auto, positive repeat/fr,
font size, normal/bold weight and RGB/hex color. Length/color custom-property
values and references are distinct enum variants; names require `--app-` and a
validated ASCII suffix. References validate their names and declared value kind;
they do not resolve the eventual CSS cascade or prove an externally overridden
custom property has the right value. Minimum-width media accepts literal px/rem;
percent, auto and variable media widths fail. No raw selectors, declarations,
URLs or arbitrary value strings are supported.

Generated classes use the exact contract grammar, including the 128-byte bound.
Finalization rejects nonfinite/negative numbers, invalid channels/hex, zero or
negative Grid repetition/fraction, invalid names and property-invalid auto.
The diagnostic includes module, class, base/media context and property when
applicable. A failing stylesheet returns no partial successful CSS; the binary
serializes before writing stdout and reports errors on stderr with exit 1.

A module uniquely owns a class across base and media contexts. Equal declarations
do not excuse ownership collisions. Same-owner base plus media is allowed;
repeated class/context fails. Module, rule, declaration and media positions are
explicit; repeated declarations preserve insertion order. No environment, clock,
path or random input enters CSS. Hash maps perform membership checks only.
Output is UTF-8, LF, with a final newline. This is trusted developer authoring,
not a CSS sanitizer or a sandbox for arbitrary Rust code.

## Publication boundary

The generator has no path argument and writes only CSS to stdout. The Python
publisher is prototype tooling, not an application/generator runtime dependency,
Yetlet command, stable record schema or public API. Its single-run command uses
literal argv, selected Rust 1.99.0 for both host discovery and Cargo execution,
explicit generator manifest/bin, frozen mode, disabled default features, host
target and an explicit reusable cache. It never invokes Cargo from a build
script, request, View or WASM.

The local publisher deliberately supports only
`assets/generated/<ASCII-name>.generated.css`. This small allowlist excludes
absolute paths, traversal, source/config/lock/cache destinations and frontend
roots. It checks symlink components and regular-file destinations. Private
`.stylegen/publication.json` ownership and pending intent stay outside assets;
[local ignore rules](../experiments/style-authoring/.gitignore) exclude them and
the generated destination. An existing unowned file is refused. Later writes
require the previous accepted SHA-256 to match disk bytes; manual changes or
missing accepted files are conflicts. Identical bytes return `unchanged` without
rewriting CSS or metadata.

Successful complete stdout is checked for nonempty UTF-8 and at most 2 MiB before
publication. Stderr remains diagnostic output; nonzero producers never publish.
Private disk staging avoids unbounded in-memory capture. The trusted producer
can still consume disk before the post-exit size check; arbitrary-process output,
time limits, cancellation and process-tree supervision are deferred coordinator
work. The publisher does not parse arbitrary CSS syntax.

A sibling temporary file is flushed/fsynced and replaced with `os.replace`.
Observed Linux semantics provide replacement visibility; this is not a claim of
power-loss durability or Windows qualification. Windows execution is explicitly
refused. The filesystem assumes one trusted owner and stable ancestors, without
hostile rename races, concurrent publishers or hard-link isolation.

Before CSS replacement, a private pending intent records previous acceptance and
the candidate hash. Prepublication generator/write failure preserves accepted
bytes and metadata. After CSS replacement, metadata failure reports **unaccepted
partial publication**, retains the intent/previous record and blocks another
publish. CSS and metadata are not a two-file atomic transaction. An interruption
may also leave an intent before replacement; it remains conservatively blocked.
`--reconcile` is an explicit operator action that accepts only a matching pending
candidate and compatible previous ownership. A mismatched destination requires
manual investigation. It never silently blesses new bytes or claims old bytes
survived a post-rename failure. Controlled tests exercise this interruption
boundary; no actual machine crash or power loss was induced.

## Commands and focused results

Run from the repository root with the installed toolchain; no download/install
fallback is used. Initial lock creation was setup only:

```sh
cargo +1.99.0 generate-lockfile --offline --manifest-path experiments/style-authoring/stylegen/Cargo.toml
cargo +1.99.0 generate-lockfile --offline --manifest-path experiments/style-authoring/Cargo.toml
```

Both exited 0. Proof commands preserve the locks. Continue using the allocated
`.cache/yetlet/builds/yt238-style-builder-20261108` cache, which also holds the
required fresh-cache measurement. Its paired layout record binds the already
created evidence directory without overwriting baseline/role/policy/prompt files.
The first focused tests/publisher used `target/yt238-style-authoring-20261108`;
that initial allocation did not follow the new-isolated-cache location policy.
It and its original evidence are preserved. The parent corrected the documented
commands and verified the focused Cargo tests/publisher against the approved
cache path; see `parent-cache-checks.json`. No cache was removed or cleaned.

```sh
rustc +1.99.0 -vV
cargo +1.99.0 fmt --manifest-path experiments/style-authoring/stylegen/Cargo.toml --check
cargo +1.99.0 fmt --manifest-path experiments/style-authoring/Cargo.toml --check
cargo +1.99.0 test --frozen --quiet --manifest-path experiments/style-authoring/stylegen/Cargo.toml --no-default-features --target x86_64-unknown-linux-gnu --target-dir "$PWD/.cache/yetlet/builds/yt238-style-builder-20261108"
cargo +1.99.0 test --frozen --quiet --manifest-path experiments/style-authoring/Cargo.toml --no-default-features --target x86_64-unknown-linux-gnu --target-dir "$PWD/.cache/yetlet/builds/yt238-style-builder-20261108"
python3 -m unittest discover -s experiments/style-authoring/tests -v
cargo +1.99.0 tree --frozen --manifest-path experiments/style-authoring/stylegen/Cargo.toml --no-default-features --target x86_64-unknown-linux-gnu --edges normal
cargo +1.99.0 tree --frozen --manifest-path experiments/style-authoring/Cargo.toml --no-default-features --target x86_64-unknown-linux-gnu --edges normal
python3 experiments/style-authoring/publish.py --target-dir "$PWD/.cache/yetlet/builds/yt238-style-builder-20261108"
git diff --check
```

All final commands exited 0. Generator tests: **8 passed**; View tests: **1
passed**; publication tests: **10 passed**. Cases cover conditional membership,
byte equality, duplicates/collisions, ordering, invalid values, path/ownership
rejection, size/UTF-8/producer failures, no-op, pre-write preservation and
post-rename reconciliation. Real publisher runs reported `published`, then
`unchanged`. Compile and native file behavior are verified; browser/rendered
layout and HTTP/CSP execution were not run. CSP source remains unchanged.

The first Rust build failed with E0277 in RGB channel validation. One product
repair changed iteration over references; no runnable generator behavior was
accepted from that build. The next test build exposed E0659 from ambiguous test
glob imports, a harness failure corrected once with explicit imports. The
focused rerun passed. Product repair budget used: **1 of 2**; operational retries:
**1 of 2** (an initial ADR read used an incorrect filename; the actual ADR was
then read at its repository path). This was an operational read failure, with no
product behavior disproved. Original build/test failures and their
classifications are retained.

Evidence lives under `artifacts/verification/yt238-style-builder-20261108/`:
`rust-initial-test.*`, `rust-repair-1.*`, `harness-correction.json`,
`rust-focused-test.*`, `view-test.*`, `publisher-initial-test.*`, dependency-tree
logs, `measurements.json`, named CSS/stderr captures, `published-record.json`,
`source-lines.json`, and final check/source/cleanup records. The parent also ran
mandatory `cargo xtask check docs` after consolidating the status documents:
PASS, exit 0 (`docs-check-before-review.*`). This repository checker covers its
fixed 75-document list and 233 tabulated backlog IDs; direct new-file/link checks
supplement it for this addition. The independent PASS is recorded below.

## CSS comparison and observed DX

The checked-in plain CSS is exactly **674 bytes / 37 physical lines**, matching
generated CSS SHA-256
`3d4ccc096f29d2119332292512f6f03c5304e255d1b12249afd0c83239a02512`.
It includes identical toolbar/Grid/dialog/tokens/media declarations in identical
order. This establishes source/output equivalence, not computed browser layout.

Physical line counts include blank lines/comments and rustfmt expansion:

| Source category | Lines |
|---|---:|
| Rust app style modules + registration | 72 |
| Shared names (used by both styles and View) | 5 |
| Shared local builder implementation | 297 |
| Generator library/binary wiring | 21 |
| Prototype publisher | 183 |
| Both isolated manifests + locks | 42 |
| Local ignore rules | 2 |
| View implementation, excluding its test module | 22 |
| Rust route total implementation/setup, excluding tests/plain comparison | 644 |
| Focused tests, including 36 View test lines | 384 |
| Plain CSS alternative, style only | 37 |
| Plain CSS + same View/names/app manifest/lock, excluding tests | 88 |
| Whole prototype source including tests and plain comparison | 1065 |

Thus style-only Rust authoring is 72 lines, or 77 including shared names, before
paying for the builder. The plain alternative still benefits from the same View
class constants; its 88-line total excludes the Rust generator/publisher and
needs no generated-file ownership mechanism. Counts exclude documentation,
cache/evidence, private publication records and generated CSS. Neither route's
setup count should be confused with just the property declarations.

The generator normal dependency tree has only `app-stylegen`; the View graph has
`style-authoring-view-fixture -> yetlet`, default features disabled. Neither graph
contains the other. No HTTP/browser/Node dependencies are needed for CSS output.

For each measurement, the exact argv is recorded in `measurements.json`. The
command uses an absolute manifest/cache, the explicit `app-stylegen` bin and
`--frozen --quiet --no-default-features --target x86_64-unknown-linux-gnu`.
Selected compiler: Rust 1.99.0, host `x86_64-unknown-linux-gnu`, Linux. Elapsed
wall time includes Cargo checking/compilation and generator execution:

| Single observation | Seconds | Output bytes |
|---|---:|---:|
| Fresh isolated cache | 0.284137 | 674 |
| Same-source warm cache | 0.034797 | 674 |
| One token declaration edited | 0.198789 | 671 |
| Token edit restored | 0.141769 | 674 |

Changing `LengthToken("--app-space", Rem(0.75))` to `Rem(1.0)` emitted
`--app-space: 1rem` while existing variable references stayed unchanged. Native
CSS makes the corresponding edit directly to `--app-space`; it has no
Rust-generator compile stage and no fabricated comparable compile timing.
The Rust form offers enum/property completion and reusable ordinary functions,
but adds imports, explicit registration, a second manifest/lock and compile cost.
This small sample does not establish a productivity or performance advantage.

A controlled `Padding(Px(-1.0))` edit compiled but finalized with exit 1,
**zero stdout bytes** and this stderr diagnostic:

```text
stylegen: module layout class app-toolbar context base property padding: expected finite nonnegative number
```

The edit was restored. Final generation again produced the original 674 bytes
and hash. Negative native CSS can be typed directly; browser parsing and property
rules determine its effect. This Rust diagnostic is earlier and contextual, but
only covers the finite supported vocabulary. No arbitrary CSS validation is
claimed. These are single local samples without statistical or cross-platform
performance conclusions.

## Recommendation and remaining decisions

**Recommendation: SIMPLIFY — keep this application/tooling-local.** Explicit
composition and token reuse work with no Core/browser generator dependency, and
bounded publication is demonstrable. For this tiny stylesheet, 644 setup and
implementation lines versus 88 for the plain-CSS alternative do not justify a
reusable framework API. Ordinary CSS remains the preferred small-app default.
A future owner-approved YT-239 slice can evaluate opt-in coordination and default
starter wiring while keeping the builder private; this document does not approve
that promotion or prescribe a public fluent API.

Independent YT-238 implementation review is complete. The next action is the
owner's promotion decision; no integration is authorized yet. Public API packaging, actual
CLI/build/watch/settings/scaffolds, Vite/asset delivery, concurrent edits,
cancellation, production fingerprinting and retained asset lifetimes are deferred.
Windows, browser/WASM, native HTTP/no-JS and computed responsive/CSP qualification
remain unverified here. No application/browser/server/watcher was launched; all
owned one-shot Cargo/Python subprocesses completed and temporary product-test
resources were cleaned. Build caches and accepted evidence remain intentionally
retained under the development-artifact policy.

## Independent review and completion

**YT-238 is Done at prototype scope (2026-11-08).** The independent Reviewer
returned PASS in Job `wc_job_hfecy9ZrNUTO61BZ`, terminal completed, exit 0:
zero issues, required follow-ups and optional notes. This was a separate
role-directed Codex Job using the exact saved Reviewer contract, with observed
`gpt-6-astra` / `high` matching the workspace role and reasoning resolver.
It is independent ordinary review, not current-session self-review or native
managed-role assurance. The implementation Job `wc_job_u0VTVNBJmilOwSDI` also
completed with exit 0, observed `gpt-6.1-sol` / `high`.

All 22 review inputs (17 prototype files and five documents) matched their
saved SHA-256 values throughout review. `review-observation.json`,
`reviewed-source-hashes.json`, exact `reviewed-inputs/` copies, role/policy records,
process telemetry and `reviewer-result.json` preserve this evidence. Completion
annotations follow the PASS; prototype source remains byte-identical to the
reviewed input. Final documentation/source checks and Git/job observations are
retained in `FINAL-HANDOFF.md` and the final evidence records in the same
verification directory.

The default-registration follow-up is accepted for the reference fixture and
the future YT-239 template contract. It does not ship CLI scaffolding. The
measured recommendation remains **SIMPLIFY**; YT-239/240 remain Todo pending
separate owner promotion. No Commit, Push, PR, Merge or release occurred.
