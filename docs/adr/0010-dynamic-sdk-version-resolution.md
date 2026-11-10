# ADR 0010 — Separate SDK Resolution from CLI Build and Source Delivery

- **Status:** Accepted; Owner correction on 2026-11-10 supersedes v1 selection for CLI 0.3.1 source. Original Phase B implementation/review remains historical.
- **Date:** 2026-11-10 (Asia/Taipei).
- **Task:** YT-247, Dynamic SDK Version Resolution; YT-248 released the initial policy; YT-250 corrects latest selection.
- **Related:** [CLI contract](../YT-247-SDK-VERSION-RESOLUTION-CONTRACT.md),
  [Phase A handoff](../YT-247-PHASE-A-HANDOFF.md),
  [Phase B implementation](../YT-247-SDK-VERSION-RESOLUTION-IMPLEMENTATION.md),
  [version authorities](../YT-246-VERSION-SOURCE-AUDIT.md),
  [CLI 0.3.0 preparation](../CLI-030-RELEASE-PREPARATION.md),
  [ADR 0001](0001-separation-of-concerns.md),
  [ADR 0009](0009-release-build-and-signer-identities.md).

## Context at design time

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

## Decision

Adopt the Owner-accepted normative [YT-247 CLI contract](../YT-247-SDK-VERSION-RESOLUTION-CONTRACT.md).
Keep resolution inside the independent Rust CLI:

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
4. Admit canonical exact SemVer for explicit selection. Default policy
   `sdk-latest-v2` / `latest-semver` chooses the greatest available non-yanked
   SemVer with the required same-version package set, including future releases
   and prereleases. No finite approval list, stable-first priority, build-SDK
   admission bound or frozen numeric MSRV ceiling applies.
5. After selection, validate required features and the Browser dependency on the
   selected SDK. Fail selected latest without older-version fallback. Cargo owns
   actual toolchain and dependency feasibility. Metadata and lock generation do
   not promise that future SDK compiler, build or template APIs are compatible.
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
with the CLI; current selection does not certify future template API compatibility.

## Alternatives and tradeoffs

| Alternative | Assessment |
|---|---|
| Update CLI's dependency to alpha.7 | Appropriate later build-input selection, but supplies none of the dynamic resolver behavior. |
| Query crates.io directly inside new | Mixes command orchestration, transport and policy; cannot isolate deterministic tests/cache behavior. |
| Greatest available non-yanked SemVer with the required package pair | Owner-selected policy for CLI 0.3.1. Allows future SDKs without another CLI approval-list release; selected-version feature/dependency or Cargo failures remain visible without fallback. |
| Finite reviewed prereleases and stable-first ordering | Original sdk-compat-v1 policy shipped in CLI 0.3.0; superseded by the Owner correction. |
| Silently use stale cache after a network error | Hides freshness/yank differences. Require a deliberate offline retry and visible cache provenance. |
| Copy a fixed source snapshot for every chosen SDK | Breaks package identity and provenance. Registry manifests/locks resolve the selected package instead. |
| Resolve transitive dependencies ourselves | Duplicates Cargo. Delegate graph/MSRV feasibility to Cargo and validate selected SDK lock entries. |
| Preserve source-only new and defer app lock | Keeps the old no-process behavior, but does not meet the requested successful-creation lock contract. Accept the bounded new process/network cost explicitly. |

The historical v1 policy admitted alpha.6/alpha.7 for ordinary native/browser
and alpha.7 for Style, required declared MSRV at most Rust 1.99 and chose stable
first, then reviewed prereleases. Those historical rules and their original
review evidence are preserved in the contract and Phase B record. They do not
constrain CLI 0.3.1 or require another CLI release to admit a future SDK.
Offline creation has two prerequisites: resolver metadata
and Cargo's dependency cache. New now has failure/cleanup paths for metadata and
Cargo setup, so filesystem/process protections must cover staging and cancellation.
Exact SDK pins give stable SDK identity; retaining the actual app lock gives
transitive graph identity. Neither is a reproducible-binary certification.

## Acceptance boundary

Phase A was delivered in PR #68, merge `1c98a50432e3d90e0bf6e481c0682737b9cc94e3`.
The Owner then accepted the Contract and ADR and authorized Phase B implementation,
independent review and guarded Git delivery. The implementation record reports
the source boundary, verification and platform limitations. Independent review
passes at that bounded scope; YT-248 requires a separate instruction afterward.
No SDK/tag/archive/Reader Edition, Runner permissions, historical release record,
signing or publication changes are authorized by this decision.

## Owner correction and CLI 0.3.1 boundary — YT-250

The Owner-corrected decision changes selection while retaining the resolver,
provider/cache, exact pins, app lock/receipt and immutable release boundaries.
Build SDK and source snapshot remain alpha.7, with the snapshot bound to source
`5dc8a9a990ac083e07081ea3be008748239c8a58`. CLI 0.3.1 is source preparation;
[its preparation record](../CLI-031-RELEASE-PREPARATION.md) tracks outstanding
independent review and candidate/hosted gates. Published distribution and Reader
Edition remain SDK alpha.7 / CLI 0.3.0. Formal CLI 0.3.1 publication is excluded
from this preparation; the original acceptance boundary above is historical.
