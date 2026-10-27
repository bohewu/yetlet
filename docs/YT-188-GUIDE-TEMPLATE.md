# YT-188 — Consistent developer integration guides

Date: 2026-10-26 (Asia/Taipei). Status: **Done at documentation-only integration
guide scope after independent Reviewer PASS, zero issues/required followups.**

## Authority and bounded scope

The maintainer authorizes committing YT-186/187 first, then a documentation-only
minimum adjustment using five guide sections: prerequisites, minimal integration,
expected result, configuration/errors and API reference. Preserve existing
semantics/evidence, do not replay accepted qualification, obtain independent
Reviewer before Done, and leave new work uncommitted/unpushed/unpublished.

YT-186/187's eleven paths are committed as `268a32e`, strict/builtin direct Guard
and hook PASS, zero findings; tree `4f70101d4a8d575143c65b9dcb9b44c4ff71d43f`.
The checkout was clean after commit; no Push. Prior review/correction records
remain retained. This slice starts on the same branch and changes no runtime,
manifest/lock, API signature, feature, external key, sample or release decision.

## Selected guide set and template

Apply the [guide template](GUIDE-TEMPLATE.md) to ten existing integration guides:
server-axum, DOM, Auth, Session and rate-limit crate READMEs; HTTP pipeline,
Extensions, Configuration, Secret Providers and Settings from SecretProvider.
These cover the root entry's core framework integration paths. Package inventories,
architecture/naming/compatibility policies, application-specific walkthroughs,
CLI reference and retained qualification are different document classes; do not
force all repository documents into a tutorial template or rewrite their content.

Each guide gets the same five top-level sections, with a short current purpose:

1. Prerequisites: checkout/toolchain/target/dependency/feature and required source
   inputs, including explicit browser setup and example-only offline assumptions.
2. Minimal integration: reuse existing code/commands; label setup fragments and
   assumed application types, and link the complete existing source where needed.
3. Expected result: what the existing snippet/command is supposed to do, without
   claiming new execution, qualification, performance or automatic capabilities.
4. Configuration and errors: retain exact ownership, security, failure, bounds,
   lifecycle, feature/settings and authority semantics. Configuration remains
   optional and startup-only where selected; identity never grants authorization.
5. API reference: current source/rustdoc links, license/compatibility context;
   historical delivery evidence stays explicitly labeled at the end.

Preserve old section anchors by retaining their heading text as subsections;
preserve existing code blocks and historical results. Move old delivery-status
introductions and verification into labeled details, rather than presenting them
as current instructions. A source-bound old CLI option remains historical;
current CLI guidance uses the already-canonical `--settings` spelling. No second
signature inventory, site generator, generic validator or runtime abstraction.
Root README stays at its reviewed product entry; template/index/publication-source
navigation receives only necessary links. Register YT-188 In progress, Done only
after independent acceptance.

## Bounded verification

Inspect actual guides and immediately relevant existing source/manifests. Run only
`cargo xtask check docs`, tracked/new-file whitespace and direct bounded source/
snippet/anchor checks. These validate the new document changes. Do not run
runtime tests, browser/CSP, samples, rustdoc builds/doctests, general builds/lints,
cloud/providers/benchmarks/security or release qualification. All example/build
commands in guides are instructions, not commands executed by this task.

Baseline/original guide bytes and commands are retained in
`target/yt188-guide-template-20261026/evidence/`. Preserve unrelated source and the
YT-186/187 artifacts. No Commit/Push/publication of this new delivery. YT-096
remains In progress with D1–D6 Pending; YT-020 remains Blocked.


## Delivered scope and retained evidence

Ten existing guides now expose exactly the five selected top-level sections.
The current purpose/preview note precedes them; original useful headings remain
as subsections, including the former settings-guide title in historical context.
Existing error/security/ownership text is preserved. All 29 original fenced
code blocks are retained byte-for-byte; new listener-free example commands are
source-backed instructions and were not executed. Browser prerequisites link
the root's pinned target/wasm-bindgen/PATH setup; setup fragments explicitly
state their required host/types and do not claim to run a listener.

The shared template is linked from the reader index and the existing publication
source classification. Root README, current API map, original evidence reports,
source, manifests and locks are unchanged. The old Config CLI spelling/status
paragraph remains under a historical label with a current `--settings` pointer.

Actual sixteen delivery paths: the ten guides listed above; new
`docs/GUIDE-TEMPLATE.md` and this record; `docs/BACKLOG.md`, `docs/ROADMAP.md`,
`docs/README.md` and `docs/DOCUMENTATION-SCOPE.md`. Direct source checks confirm
all old heading texts and snippets, exactly sixteen Markdown paths, staged zero
and 489 protected source/guide/history hashes unchanged. This is not a browser
rendering, novice-user study or full publication/privacy certification.

The new-delivery `cargo xtask check docs` exits 0, reporting 75 required documents
and 188 backlog items; links/Markdown/whitespace/DAG PASS. `git diff --check` passes;
new template/record no-index whitespace checks exit 1 with empty diagnostics,
expected content-difference semantics. Subsequent final document bookkeeping
will receive the same narrow gate. No product test, rustdoc generation, sample
or old qualification gate, cloud/service exposure, publication, Commit or Push
runs for this new delivery. Independent Reviewer
`/root/yt188_integration_guides_independent_review_20261026` returns PASS, zero
issues/required followups. Native role/model/effective effort match the saved
profile and central resolver. Before/after checks match all sixteen source,
12 evidence and 489 protected-input hashes, with HEAD/branch/status unchanged.
Only record and Backlog/Roadmap/index completion metadata follows this acceptance.

YT-188 is Done at the selected ten-guide/template scope. This is not a new
runtime/platform qualification or permission to publish. No next implementation
is selected; YT-096 remains In progress with D1–D6 Pending, YT-020 Blocked.
The new sixteen-path delivery remains unstaged/uncommitted/unpushed.
