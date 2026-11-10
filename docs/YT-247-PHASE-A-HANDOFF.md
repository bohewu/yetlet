# YT-247 — Phase A Review and Implementation Handoff

Date: 2026-11-10 (Asia/Taipei). **Historical Phase A freeze: independent design review PASS.**

Subsequent delivery: [PR #68](https://github.com/bohewu/yetlet/pull/68), merge
`1c98a50432e3d90e0bf6e481c0682737b9cc94e3`, exact-head and master CI PASS.
The Owner accepted the Contract and ADR 0010 and explicitly authorized Phase B.
See the [implementation record](YT-247-SDK-VERSION-RESOLUTION-IMPLEMENTATION.md)
for that continuation. The pending/stop language below preserves the original
Phase A checkpoint, not the current authorization state.

## Deliverables and preserved work

- [Normative CLI contract](YT-247-SDK-VERSION-RESOLUTION-CONTRACT.md): command/error,
  compatibility/prerelease, provider/cache, exact pin/lock, testing and provenance
  contracts, with module and resolution flow diagrams.
- [ADR 0010](adr/0010-dynamic-sdk-version-resolution.md): proposed architectural
  decision, alternatives and consequences.
- [Backlog](BACKLOG.md) and [Roadmap](ROADMAP.md): YT-247 Phase A/B split and
  YT-248 dependency. YT-247 is not Done at design-only scope.
- [CLI 0.3.0 preparation](CLI-030-RELEASE-PREPARATION.md): retained Style release
  acceptance and prior independent plan review, now YT-248.
- [SDK alpha.7 handoff](YT-245-ALPHA7-RELEASE-PREPARATION.md): immutable SDK history
  and separate resolver/CLI release references.

Initial branch `feat/cli-030-style-templates`, HEAD
`54c1e4436ad15ece4e1fad74fbc26c98884a98c4`; four pre-existing planning files were
read and copied into this run's ignored `initial/` evidence directory before
editing. No reset, worktree replacement or source/resource regeneration occurs.

## Validation and independent review

Evidence root: `artifacts/verification/yt247-contract-20261110/` (ignored).
The cache/evidence pair is created by `scripts/dev_artifacts.py`; historical
qualification failures and evidence remain in place.

- `cargo xtask check docs`: PASS (75 required documents, 245 backlog items; DAG/language).
- `python3 scripts/release_docs.py check`: PASS; existing publication and Reader Edition authorities unchanged.
- Tracked/untracked whitespace, final newline and direct checks of all 11 changed document local links/fragments/fences: PASS. Two pre-existing stale anchors in touched documents were corrected to existing headings; no validator was added.
- Independent Reviewer `/root/yt247_contract_review_20261110`: PASS, read-only
  ad hoc review against the Owner handoff and adjacent CLI/release source.
  Native trace matches role `reviewer`, configured model `gpt-6-astra` and
  effective effort `high`; shared resolver observation is enforced, not degraded.
  All 11 document hashes were unchanged across review. Result, trace and resolver
  observations remain in this evidence root.
- Compile, runtime, browser/CSP and Windows validation: not run in Phase A;
  no implementation changes. Prior accepted evidence keeps its original scope.

The reviewer covered version/template decoupling, registry/offline semantics,
prerelease policy, pin/lock responsibilities, embedded snapshot purpose,
historical CLI bindings and extension boundaries. `reviewer-result.json` retains
this result; `reviewer-trace-final.json` retains the bounded runtime observation:

```json
{"overall_status":"pass","issues":[],"required_followups":[],"optional_notes":[]}
```

This is independent design acceptance, not Owner design approval or Phase B
implementation acceptance. Only factual result/delivery annotations follow the
reviewed document snapshot; no resolver design change is made after PASS.

## Git delivery

At this document freeze, independent review is PASS. Delivery order: staged
Strict/Builtin guard, commit,
exact outgoing range guard, PR, exact-head CI, merge and master CI. Publication,
release workflows and Pages deployment are excluded. Documentation CI is
path-filtered; record selected checks and any workflow not triggered, without
claiming it executed. Do not change workflows just to manufacture a check.
Actual commit/PR/CI receipts are retained in the same ignored evidence root and
reported in the final Owner handoff; this precommit record cannot contain its own
future commit/merge identity.

## Owner decision and next implementation

**Stop after Phase A Git delivery.** The proposed finite prerelease fallback,
initial per-template compatibility sets, new Cargo-lock creation behavior and
explicit offline/no-stale semantics require Owner acceptance before Phase B.
No remaining architectural alternative is silently delegated to implementation.

Once accepted and explicitly instructed, the next unblockable task is YT-247
Phase B, in this order:

1. Add the small internal resolver/provider/policy/cache types and fixture tests.
   Keep registry HTTP outside new. Validate initial candidate/package/feature/MSRV
   policy and report source/freshness; do not widen experimental compatibility.
2. Thread the result through existing native/browser/Style template assembly,
   replacing all snapshot-derived generated pins and README values. Add parsing
   for exact `--sdk-version` and explicit `--offline` with usage/error tests.
3. Extend existing protected filesystem/process adapters only as needed to stage
   new manifests, generate/validate the root app lock, preserve the nested Style
   lock and publish once. Add the creation receipt, cancellation/owned cleanup and
   repeated-destination tests. Keep later build/run/watch frozen and resolver-free.
4. Separate current build/snapshot bindings from generated-app policy in
   release-doc/version-authority checks and tests. Preserve legacy release
   records. Ship policy and immutable resource identity inside the CLI archive;
   keep the source exporter qualified and reject mismatches.
5. Run deterministic fixture/local-registry tests and the separate opt-in real
   registry consumer lane. Report compile/MSRV, native, browser/CSP and cleanup
   results separately. Obtain independent review before marking YT-247 Done.

Do not bump/release CLI merely to implement the resolver. A separate Owner instruction may resume YT-248's
existing reviewed release plan, select CLI build SDK/snapshot alpha.7 separately,
and adapt its installed consumer checks to exact and default resolution.
YT-248 retains the registry-only Style native HTTP/CSS, browser Vite/WASM/runtime,
CSP and cleanup acceptance, with Windows Style revalidation paused/unverified.
A new release still needs its own release-doc gate, source/custody/sign/V and
Owner publication approval. The public Reader Edition remains independently pinned.
