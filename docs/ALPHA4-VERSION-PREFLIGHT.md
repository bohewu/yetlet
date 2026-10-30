# Alpha.4 — Coordinated sixteen-package version preparation

Date: 2026-10-30 (Asia/Taipei). **Required affected verification and independent
Reviewer PASS, zero issues/required followups.** The owner selected version
`0.9.0-alpha.4`, not an immutable
candidate commit, tag, hosted execution, signature or publication.

## Authority and baseline

The owner authorized synchronizing the sixteen libraries, internal requirements
and lockfiles, preserving third-party dependencies and supplementing only
affected verification. No Commit/Push, tag, hosted job, signing or publication
is authorized for this slice. Work stays in the existing master checkout at
`1740ba2dc6423f998153b4152ee5e1e524192483`, tree
`ff522a37d2414c50a4bf77dce4484a3b788c1218`; it was clean before editing.

PR25's squash merge and exact-tree PR/master CI PASS are the source baseline,
not alpha.4 qualification. Its [single-publication policy](RELEASE-PUBLICATION.md)
and Config/Secrets cycle correction remain unchanged. Three automatic worker
Environments retain their ref restrictions, disabled admin bypass and existing
age identity. The owner-gated master-only publisher has `CRATES_IO_TOKEN`
presence confirmed, without reading its value. Its actual registry authority
has not been exercised. This version preparation changes no Environment or
Secret; the evidence Environment still has the existing alpha.3 tag policy.

The [accepted alpha.3 release](ALPHA3-HOSTED-RECORD.md) and all earlier tags,
archives, signatures, failed observations and original acceptance labels remain
immutable. None is rewritten or reassigned to alpha.4. The earlier
[alpha.3 preflight](ALPHA3-VERSION-PREFLIGHT.md) retains its historical bootstrap
and optional failed-harness boundaries; the current recipe uses normally
verified final archives without bootstrap or verification bypasses.

## Version and dependency scope

All sixteen manifest versions resolve to `0.9.0-alpha.4`. Eleven inherit the
workspace version; five retain isolated manifests. Seventeen normal, optional
and development internal requirements move with them. The workspace examples
that already inherit its package version follow that inheritance; they do not
join the sixteen-library release cohort. CLI/tool/vendor and independent
example versions remain unchanged.

Nineteen affected lockfiles change only 111 local package-version fields.
All 2,361 sourced package blocks across twenty-one tracked lockfiles remain
byte-identical to baseline. No third-party version, checksum, dependency
declaration, feature, target, MSRV or patch changes. Thirteen package READMEs
update only their current source-version text. These forty-six input paths are
exact old-version-to-new-version substitutions. Library/runtime and public API
source is unchanged.

The all-kind Config/Secrets graph remains acyclic: sixteen names, seventeen
declarations. A shared Settings example stays in Config behind its existing
`secret-provider` feature; Secrets gains no reverse Config dependency.

## Affected verification

Retain commands, results, source/lock correspondence and archive identities under
`target/yt096-alpha4-version-20261030/` (local, ignored). Use Rust/Cargo1.99.0,
offline locked inputs and the already implemented sixteen-package recipe.

Required checks for this slice:

- Offline/locked metadata for all nineteen affected roots, without lock drift.
- Sixteen normal Cargo package verifications in all-kind dependency order,
  staging each actual predecessor archive digest; no bootstrap or null package
  checksums. Inspect actual normalized manifests, packaged locks and source
  payload correspondence.
- The existing external packaged consumer with Auth/session,
  Config/secret-provider and Redis/tls, resolving all sixteen exact new versions
  from those verified archives without checkout patches.
- Existing current-version preparation/custody identity checks, rebuilt
  sixteen-package fixture and Python sixteen-manifest version check. These
  validate changed version bindings rather than replaying runtime qualification.
- Repository docs, links, Markdown/DAG and changed-file whitespace checks, then
  independent review of the actual diff and retained evidence.

Existing unchanged native/browser/platform, security, workload and performance
results keep their original sources and limits. No private key or registry token
is read, and no synthetic crypto roundtrip is needed for a version-only edit.
Local packaging is not a hosted or registry-upload result.

## Actual local results

| Check | Result and scope |
|---|---|
| Metadata and source locks | All nineteen affected roots PASS offline/locked; all twenty-one tracked locks retain the expected exact version-only contents afterward |
| Normal final packages | All sixteen PASS with actual predecessor checksums; seventeen commands including vendor, no bootstrap or verification bypass |
| Archive/source correspondence | All sixteen manifests/locks have alpha.4 identities; twenty-six internal registry checksums match actual new archives; sixty-seven runtime/test/example payloads match the pre-migration baseline exactly |
| Version-only archive delta | Relative to retained normal alpha.3 archives for the same library source, only manifest/README/lock versions and derived internal package checksums differ |
| Exact-byte registry planning | Sixteen offline envelopes and the acyclic sixteen-name/seventeen-edge graph PASS with an explicitly synthetic transport record; no candidate or upload claim |
| External packaged consumer | All sixteen exact versions resolve from verified directory sources; Auth/session, Config/secret-provider and Redis/tls compile/run PASS; no checkout patch and no lock drift |
| Version-sensitive tooling | Existing preparation identity, custody legacy-ref refusal and real sixteen-package fixture each PASS under compiled alpha.4; Python actual-sixteen-manifest version/mismatch case PASS |

The first consumer command retains exit101: the historical consumer lock pinned
`equivalent1.0.3`, while the current source-locked vendor supplies `1.0.2`.
This is a stale fixture input, not a changed source dependency. One correction
regenerates **only the temporary consumer lock**, offline, against the already
vendored current-source dependency set. Its 183 third-party package identities
and checksums all occur in existing source locks. The corrected run and locked
metadata PASS. No repository lock, dependency requirement or third-party source
was changed to repair that harness; the first lock/output remains retained.

Archive inventories, normal package logs, raw metadata, the reused consumer
inputs and initial/corrected outcomes remain in the evidence directory. Prior
application/platform/security qualification is not rerun. The earlier alpha.3
optional crypto-harness failure remains historical and is not relabeled PASS.

Independent Reviewer `/root/alpha4_version_independent_review_20261030` accepts
the actual fifty-path diff and retained evidence with **PASS**, zero issues and
required followups. All fifty source hashes match before/after review; native
role/model/actual high effort are verified. Only completion metadata follows
that accepted boundary. Owned temporary source, vendor, Cargo home, build targets
and consumer are removed; sixteen archives, source correspondence, consumer
inputs/locks, logs and the cleanup receipt remain retained.

## Delivery and execution boundary

YT-020 remains **Blocked** until actual registry activation, exact index/download
readback and a registry-only consumer succeed under separate publication
authority. This preparation does not select C/R/S/V/P or claim release readiness.
After independent acceptance, Git delivery still needs authority and
strict/builtin Guard. Confirm exact-source PR/master CI before selecting the
merged immutable candidate. A later authorized alpha.4 tag/build/readback must
use new source/run/artifact/hash bindings and the exact tag-policy rollover;
older signatures and expired artifacts cannot qualify new bytes.
