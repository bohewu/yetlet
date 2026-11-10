# ADR 0010 — Separate SDK Resolution from CLI Build and Source Delivery

- **Status:** Proposed; Phase A independent Reviewer PASS. Owner acceptance and Phase B implementation pending.
- **Date:** 2026-11-10 (Asia/Taipei).
- **Task:** YT-247, Dynamic SDK Version Resolution; YT-248 release depends on Phase B.
- **Related:** [CLI contract](../YT-247-SDK-VERSION-RESOLUTION-CONTRACT.md),
  [Phase A handoff](../YT-247-PHASE-A-HANDOFF.md),
  [version authorities](../YT-246-VERSION-SOURCE-AUDIT.md),
  [CLI 0.3.0 preparation](../CLI-030-RELEASE-PREPARATION.md),
  [ADR 0001](0001-separation-of-concerns.md),
  [ADR 0009](0009-release-build-and-signer-identities.md).

## Context

The independent CLI currently compiles with exact SDK alpha.6. Its embedded
workspace also supplies alpha.6 to registry-facing native/browser scaffolds.
The full embedded SDK source map serves the qualified native source-export
path; new does not copy that SDK source. Resource refresh currently couples
build dependency, snapshot and reviewed source version. A generated root app
lock is currently a separate user action.

SDK alpha.7 is already published, while the public Reader Edition deliberately
remains alpha.6 / CLI 0.2.0. Updating the CLI build dependency cannot establish
registry discovery, compatibility, offline behavior or safe application locking.
Unconditionally coupling runtime app selection to the snapshot would also risk
mislabeling alpha.6 source as an alpha.7 app.

## Proposed decision

Adopt the normative [YT-247 CLI contract](../YT-247-SDK-VERSION-RESOLUTION-CONTRACT.md)
only after Owner acceptance. Keep resolution inside the independent Rust CLI:

1. A small resolver composes a registry/index provider, local public-metadata
   cache and pure template compatibility predicate. Only the provider performs
   HTTP. No new framework crate, plugin registry or mandatory trait hierarchy.
2. New passes exact/default selection, template/options and online/offline mode
   into the resolver, then passes one typed result into scaffold. Scaffold pins
   registry SDK dependencies exactly; browser requires an equal-version pair.
3. Maintain three independent authorities: CLI build SDK in its own manifest/lock;
   immutable embedded snapshot version/source/digests for bounded source delivery;
   runtime app SDK selected against registry/cache and shipped template policy.
   Template revision is identified by CLI version, selector and policy revision.
   Keeping build SDK and snapshot equal for a release remains valid and useful.
4. Admit only canonical exact SemVer for explicit selection. Default chooses
   compatible stable first, then only exact reviewed prereleases. Initial policy
   admits alpha.6/alpha.7 for ordinary native/browser and alpha.7 for Style.
   Its stable set is empty. Default therefore chooses alpha.7 when available,
   with an Experimental notice. Unknown future alphas, beta/rc and stable versions
   are not trusted automatically. The finite experimental set reflects the lack
   of a general prerelease compatibility promise. Later reviewed stable intervals
   can extend policy without changing command/provider/scaffold responsibilities.
5. Exclude a candidate if a required package is absent/yanked, has missing required
   features or unknown/too-high MSRV, or fails template policy. Cargo separately
   resolves the transitive graph; metadata cannot prove API/runtime compatibility.
6. Online always refreshes/revalidates required metadata and fails on network
   failure. Offline is explicit (`--offline` or `CARGO_NET_OFFLINE=true`), uses
   complete local metadata without network, reports age and cache-limited latest,
   and fails on missing/corrupt data. No silent stale-cache fallback. Keep the
   CLI-owned cache independent of Cargo's private sparse cache representation.
7. Change successful new to create the root app Cargo.lock in private protected
   staging before create-only publication. Preserve all registry checksums and
   the separate Style generator lock. Never seed from the CLI/framework lock.
   Generate no SDK source copy. Persist a creation receipt with source/policy,
   selected SDK checksums and app lock digest; later frozen builds never resolve
   again. This does not promise identical transitive locks across creation dates.
8. Preserve historical CLI0.1.0/0.1.1 -> alpha.5 and CLI0.2.0 -> alpha.6 records.
   Future release bindings describe build SDK separately from allowed app choices.
   Keep schema3/cli, immutable C/tag, separate S/V/P and one CLI package subject.
   Ship policy and snapshot identity inside the checksum-bound CLI archive instead
   of mutating old release-record schemas or substituting runtime selection for C.
9. Prove the resolver with fake index/cache/clock/transport and local Cargo fixture
   resolution. Keep real crates.io integration opt-in and report compile, browser,
   CSP and cleanup proof separately. Do not replay accepted Style qualification
   in Phase A or describe unimplemented behavior as shipped.

The existing Docker/Compose exporter remains restricted to its qualified
snapshot source/lock representation. Dynamic SDK selection does not broaden it
or allow arbitrary source downloads. Browser startup/template source is shipped
with the CLI and admitted through the per-template compatibility policy.

## Alternatives and tradeoffs

| Alternative | Assessment |
|---|---|
| Update CLI's dependency to alpha.7 | Appropriate later build-input selection, but supplies none of the dynamic resolver behavior. |
| Query crates.io directly inside new | Mixes command orchestration, transport and policy; cannot isolate deterministic tests/cache behavior. |
| Choose the greatest registry version | Can choose yanked, unsupported prerelease, missing companion or wrong-MSRV packages. |
| Automatically accept the next alpha in the same numeric train | Small maintenance cost, but no demonstrated API/asset compatibility; rejected for experimental releases. |
| Stable-only default today | Predictable, but every default new would fail with the current approved all-prerelease SDK set. Exact reviewed fallback preserves an explicit usable experimental default. |
| Silently use stale cache after a network error | Hides freshness/yank differences. Require a deliberate offline retry and visible cache provenance. |
| Copy a fixed source snapshot for every chosen SDK | Breaks package identity and provenance. Registry manifests/locks resolve the selected package instead. |
| Resolve transitive dependencies ourselves | Duplicates Cargo. Delegate graph/MSRV feasibility to Cargo and validate selected SDK lock entries. |
| Preserve source-only new and defer app lock | Keeps the old no-process behavior, but does not meet the requested successful-creation lock contract. Accept the bounded new process/network cost explicitly. |

Finite prerelease policy requires another reviewed CLI policy delivery before
new experimental SDKs become compatible candidates; registry presence alone
never authorizes them. Offline creation has two prerequisites: resolver metadata
and Cargo's dependency cache. New now has failure/cleanup paths for metadata and
Cargo setup, so filesystem/process protections must cover staging and cancellation.
Exact SDK pins give stable SDK identity; retaining the actual app lock gives
transitive graph identity. Neither is a reproducible-binary certification.

## Acceptance boundary

This ADR reserves the next unused number after 0009 and records a proposal.
Independent design PASS plus Git delivery completes Phase A only. Stop for Owner
acceptance before changing resolver, parser, scaffold, resource, manifest,
release script or workflow code. YT-247 stays open until Phase B acceptance;
YT-248 is blocked on that implementation. No SDK/tag/archive/Reader Edition,
Runner permissions, release record, signing or publication is changed here.
