# YT-186 — Current API reference and documentation publication scope

Date: 2026-10-26 (Asia/Taipei). Status: **Done at documentation-preparation scope
after independent Reviewer PASS, zero issues/required followups.**

## Authority and bounded delivery

The maintainer requests current API/reference and reader-documentation cleanup:
select publication scope and classifications first, then minimum document changes,
repo documents plus rustdoc as the source, preserved historical evidence, stale
usage corrections, Backlog/Roadmap registration and independent review before Done.
No new sample/runtime, old qualification replay, publication, Commit or Push.

The clean checkout starts at `332462b31f6d549b46977c885d71036cb7acc715` on
`docs/yt113-123-opt-in-capability-plan`. YT-184/185 are committed and independently
accepted at their bounded scopes. This slice changes documentation only, without
changing public Rust signatures, Cargo features, external keys, support targets,
source/release candidate or YT-096/D1–D6 and YT-020 decisions.

## Contract selected before adjustment

1. Add [publication/source classification](DOCUMENTATION-SCOPE.md): distinguish
   current API reference, reusable developer guides, application/tool guides,
   dated research/evidence and maintainer/private operational material. A small
   explicit reader-source selection replaces any assumption that all `docs/**`
   is suitable for a website. A page linked as evidence is not implicitly selected
   for publication. Repository visibility/history is a separate decision.
2. Add [current API reference](API-REFERENCE.md): one current entry map covering
   the original nine libraries and the seven later opt-in packages, with selection,
   ownership, error/trust boundaries and links to actual guides/source. The map
   is not an exhaustive signature dump or a new compatibility classification.
   Repository source/rustdoc owns signatures and cfg availability; current guides
   own usage/behavior; dated evidence owns only its recorded source and scope.
   No separately maintained Wiki/API contract copy is selected.
3. Preserve `API-CONTRACTS.md` as explicitly historical M9 classification and
   add a current-reference pointer. Add reader-first navigation to root/index
   and the capability map without deleting old index text, moving evidence or
   breaking existing anchors. Treat the repository index as mixed maintainer/
   reader navigation, not a website export.
4. Correct concrete stale guidance: Board's old uncommitted and Windows-unknown
   statements, and the root overview's overly narrow Redis-provider summary.
   Also distinguish the retained real IME/spoken-AT desktop observations from
   general/current-source platform qualification.
   Distinguish historical provider qualification from the later bounded Valkey/TLS
   reference; do not broaden it to general HA/restore or current-device guarantees.
   Existing application-owned write authorization, tracing and display-cache
   options are reference app APIs, not new framework packages.
5. Register YT-186 In progress. Only after independent Reviewer PASS may status
   become Done at documentation-preparation scope. Keep historical PASS/FAIL,
   dates, source IDs, limits and original immutable candidate unchanged.

## Minimum verification

Use read-only source/manifest/guide inspection plus the existing
`cargo xtask check docs`, tracked/new-document whitespace and exact bounded-diff/
hash checks. These validate this new documentation delivery, not old product
qualification. Do not run cargo tests/doc/doctests/build/clippy, browsers, CSP,
providers, benchmarks, Azure, release/signing or site tooling. Rustdoc commands
are instructions only and must be labeled unexecuted. No generated documentation
site, publication workflow, scanner or validation infrastructure is authorized.

Local command results and pre-change hashes are retained under
`target/yt186-documentation-20261026/evidence/`. Independent review inspects actual
changed documents, minimal referenced source and the preserved evidence boundary;
no optional P3 cleanup may expand this slice. Existing documents outside the
listed delivery files remain byte-identical.


## Delivered documents and observed checks

The selected publication/source classification was written before the minimum
existing-document changes. The current reference maps sixteen framework package
carriers, their main entry points and boundaries, plus Board/Admin/Shop options
and the CLI. It is deliberately a navigation/contract summary, not a complete
export audit or a second signature inventory. Source-linked rustdoc generation
instructions are provided but were not executed.

Ten delivery paths:

- New `docs/DOCUMENTATION-SCOPE.md`, `docs/API-REFERENCE.md` and this record.
- Updated `README.md`, `docs/README.md`, `docs/API-CONTRACTS.md`,
  `docs/CAPABILITIES.md`, `docs/BOARD-START.md`, `docs/BACKLOG.md` and `docs/ROADMAP.md`.

The original index body remains present byte-for-byte behind an explicitly
labeled details section, with its existing anchors. No evidence document was
moved, removed or relabeled as current execution. The M9 classification only
receives a current-reference pointer. The bounded old summary corrections use
[Windows native evidence](YT-094-WINDOWS-VERIFICATION.md),
[desktop acceptance](YT-074-DESKTOP-ACCEPTANCE.md) and
[the later Valkey/TLS reference](YT-184-AZURE-AUTH-SESSION-RESTART.md);
source/device/topology limits remain explicit.

The new-document delivery check `cargo xtask check docs` exits 0: the tool reports
75 required documents and 186 backlog items, with links, Markdown, whitespace and
DAG PASS. `git diff --check` passes. For each of the three new documents,
`git diff --no-index --check -- /dev/null <document>` exits 1 with no diagnostics:
that is the expected content-difference status, not a whitespace failure.
492 pre-change source/guide/history inputs outside this delivery match their
original SHA-256 hashes; exactly ten Markdown paths differ, staged zero.

No runtime test, rustdoc build/doctest, general build/lint, sample gate, cloud,
provider, browser, performance, security scan or release validation ran. Existing
proof is retained without replay. The document gate is not a full content/privacy
review of every repository file or a publication-readiness certificate. Destination,
final page/assets, content/link review and repository visibility stay separate
future owner decisions, without reopening the original release candidate.
Independent Reviewer `/root/yt186_api_docs_scope_independent_review_20261026`
returns PASS, zero issues/required followups. Native role/model/effective effort
match the saved profile and resolver. Before/after checks match all ten delivery
sources, 14 evidence files and 492 protected inputs, with HEAD/branch/status
unchanged and staged zero. Only this record and Backlog/Roadmap/index completion
metadata follows review. YT-186 is Done at documentation-preparation scope.
No publication-ready-site, full export/privacy audit or stability promotion is
claimed. No further implementation item is selected; YT-096 remains In progress with
D1–D6 Pending, and YT-020 remains Blocked.
