# YT-204 — Selected package inventory and security coverage

Date: 2026-10-27 (Asia/Taipei). Status: **Done at selected inventory/security
scope after authorized policy followup and final independent Reviewer PASS.**
The sections through the first independent delivery acceptance preserve the
original inventory checkpoint, its failed checks and its bounded PASS.
The [authorized followup](#authorized-license-policy-followup--2026-10-27)
records the later owner decision and current outcomes; release gates remain separate.

## Authority and fixed inputs

The owner authorized committing four accepted YT-203 documents, without Push,
then reconciling YT-204 and filling only missing selected verification cells.
YT-203 is committed as `f31eed7423d00c29d6eedc0d661f7d5a8a58b63c`, tree
`0168f5daf9f7b0eb6f0938510a61a0bb758b2f75`: strict/builtin staged Guard PASS,
four files, zero findings and exact matching pre-commit/committed tree. The
working tree was clean afterward. No Push occurred.

Scope follows the [sixteen-package plan](YT-200-SIXTEEN-PACKAGE-RELEASE-PLAN.md#verification-work-registered-for-later-execution).
Reuse the sixteen accepted [YT-201 archives](YT-201-PACKAGE-PREPARATION.md),
their normalized manifests and included locks; all included runtime/test/example
payloads and original manifests still match current library sources. No package
build, archive generation, native/browser/provider qualification or hosted CI
is replayed. YT-202/203 remain accepted at their recorded inputs. Final source C,
YT-205, YT-096/D1–D6 and YT-020 remain separate decisions and gates.

## Reuse and missing inventory cells

There are **42 selected package/target/feature cells**: sixteen GNU defaults,
fourteen MSVC defaults, six WASM defaults, Auth `session` and Config
`secret-provider` on both native targets, Redis `tls` on GNU, and wgpu
`browser` on WASM. GNU/MSVC here identify metadata targets; analysis executes
on Ubuntu26.04.1 with existing Rust/Cargo1.99.0 tools, not those target OSes.

Core and Reactive have exactly unchanged normalized manifests and full archive
locks relative to accepted YT-096 inventory. Their dependency-free selections
permit reuse of all six metadata/SBOM/license-source results without replay.
Current source/archive identities are rebound separately; old archives are not
relabeled as new archives. All other thirty-six selected inputs need new records.

Counts below exclude the package root and describe normal/build dependency
name/version pairs, not compiled binaries/features. R means retained exact-input
inventory; a dash is outside the selected scope.

| Package | GNU default | MSVC default | WASM default | Explicit feature inventory |
|---|---:|---:|---:|---|
| Core | 0 R | 0 R | 0 R | None |
| Actions | 11 | 11 | 27 | None |
| Reactive | 0 R | 0 R | 0 R | None |
| HTML | 1 | 1 | — | None |
| DOM | 1 | 1 | 20 | None |
| Keyed DOM | 1 | 1 | 20 | None |
| Axum adapter | 50 | 51 | — | None |
| Request-ID | 50 | 51 | — | None |
| Memory Session | 61 | 62 | — | None |
| Auth | 53 | 55 | — | `session`: GNU68 / MSVC70 |
| Rate-limit | 71 | 71 | — | None |
| Config | 19 | 18 | — | `secret-provider`: GNU22 / MSVC21 |
| Secrets | 9 | 9 | — | None |
| Signing | 21 | — | — | Documented GNU native recipe only |
| Redis Session | 107 | — | — | `tls`: GNU119 |
| wgpu | 0 | 0 | 0 | `browser`: WASM65 |

Existing cargo-cyclonedx0.5.9 produces thirty-six new selected library
CycloneDX1.5 SBOMs from isolated, unchanged extracted packages. The normal/build
closure in every new SBOM exactly matches its target-filtered locked Cargo
metadata: **no missing or extra name/version pair**. Additional generator outputs
for package tests/examples are retained without becoming extra selected cells.
Broader metadata and complete locks include development/other-target inputs;
their presence does not imply browser/runtime installation or compiled reachability.
wgpu's three CPU-only defaults have no normal/build dependency.

Representative commands use explicit package, target, feature and owned Cargo
home/config paths; no repository manifest, lock, dependency or policy changes:

```text
cargo +1.99.0 metadata --offline --locked --format-version 1 --manifest-path <package>/Cargo.toml --filter-platform <target> [--features <selection>]
cargo-cyclonedx cyclonedx --manifest-path <package>/Cargo.toml --format json --spec-version 1.5 --describe all-cargo-targets --all --target <target> --target-in-filename [--features <selection>]
cargo-deny --format json --locked --offline --manifest-path <package>/Cargo.toml --metadata-path <metadata> --config <policy> --target <target> check --show-stats licenses sources
cargo +1.99.0 tree --offline --locked --manifest-path <package>/Cargo.toml --target <target> --edges normal,build --prefix none --format <package-features-license> [--features <selection>]
```

The generator has no locked flag; before/after lock hashes remain exact.
Directory-source null registry checksums still do not establish actual registry
downloads, publisher identity or rights. Existing Rust/native recipes are not
replaced by cross-target metadata.

## Actual license-policy failures and minimum decision

Under unchanged cargo-deny0.20.2 and `deny.toml`, **31 new cells pass; five fail**
with eight license errors. Together with six retained passes, that is37/42
accepted license/source cells. All36 new source checks have zero source errors;
25 new warnings only report unused license allowances.

| Failed selected cell | Actual dependency / rejected license |
|---|---|
| Rate-limit GNU default | `governor0.10.4 -> hashbrown0.16.1 -> foldhash0.2.0`: `Zlib` |
| Rate-limit MSVC default | Same `foldhash0.2.0` / `Zlib` |
| Redis Session GNU default | `redis1.7.1 -> xxhash-rust0.8.19`: `BSL-1.0` |
| Redis Session GNU `tls` | The same `BSL-1.0`; `rustls-webpki0.103.16` and `untrusted0.9.0`: `ISC`; `ring0.17.14`: `Apache-2.0 AND ISC` |
| wgpu WASM `browser` | `wgpu27.0.1 -> hashbrown0.16.1 -> foldhash0.2.0`: `Zlib` |

The first failure stopped that command batch. Its raw failure and partial
results remain preserved. Remaining independent cells then execute, and only
the missing tree observation is added for that first failed cell; no passing
metadata/SBOM/check or failed license command is replayed.

These are demonstrated policy mismatches, not an inferred license violation or
a product vulnerability. The previous four-license allowlist has no approved
alternative for these actual expressions. In particular, an AND expression
cannot be satisfied by Apache-2.0 alone.

**Recommendation, not applied:** review and explicitly allow these three SPDX
IDs, retain their source notices and applicable redistribution obligations,
then rerun only the five failed policy checks using unchanged retained metadata.
The concrete prospective allowlist is:

```toml
allow = ["MIT", "Apache-2.0", "BSD-3-Clause", "Unicode-3.0", "Zlib", "BSL-1.0", "ISC"]
```

Primary license texts: [Zlib](https://spdx.org/licenses/Zlib.html),
[Boost Software License1.0](https://spdx.org/licenses/BSL-1.0.html) and
[ISC](https://spdx.org/licenses/ISC.html). This is a proposed narrow policy
decision, not legal sign-off, a wildcard exception or approval to remove notices.
Keep private/build/dev inclusion, confidence threshold and registry/Git source
restrictions unchanged. No dependency upgrade or policy edit is performed here.
Until an authorized decision and successful affected checks, YT-204 stays
In progress and the sixteen-package release license gate is not accepted.

## Current advisory identity and warning disposition

Existing cargo-audit0.22.2 checks all sixteen distinct archive locks, including
252 distinct external name/version pairs. The official database is refreshed
once; later locks use that exact database without another fetch. No target
filter, ignore, stale override or disabled yank check is added.

- Database: `3279c99911354c400e1d41e47c7d60e39258b147`,1378 advisories;
  last updated2026-10-26T22:32:25+01:00, checks completed2026-10-27 around04:09UTC.
- All sixteen commands exit0 and report **zero vulnerabilities**.
- Ten locks retain informational unsound warning **RUSTSEC-2026-0386** for
  Tokio1.53.2. This is not zero-warning evidence and not ten independent flaws.
- The [official advisory](https://rustsec.org/advisories/RUSTSEC-2026-0386.html)
  concerns Unix `AsyncFd` constructors/file-descriptor lifetime. It lists
  `>=1.53.3` as one patched range. Selected crate source has no `AsyncFd`,
  `BorrowedFd`, `AsRawFd` or `from_raw_fd` use; no source-backed attacker path
  into this API is established here. That observation does not audit all
  dependency internals or promise safety for application-supplied native code.
- Complete locks over-include dev/other-target entries; an Actions or CPU-only
  wgpu lock warning is not proof that browser/default runtime links Tokio.
  Keep the warning visible for a separately authorized compatible patch review
  before final release inputs are selected; no new ignore or lock edit occurs.

Root/application OIDC remediation and green CI4e5ef64/run40431263041 are retained
without replay, at their original source/database boundaries. The package-lock
assessment does not claim a fresh whole-root/OIDC audit or reassign its earlier
crypto-bigint failure/fix. Historical YT-096 audit results retain their old time
and database identity; this new advisory observation does not rewrite them.

## Current scoped source-security coverage

Completed Codex Security Standard scan
`63212d73-38a1-4c93-a3c4-23a125100f53` binds current commitf31eed7 and `crates/`:
**120 regular files fully reviewed,18 matching license aliases, zero reportable
source findings, complete scoped coverage and no deferred source task.**
The original narrow diff request was rejected before any scan was created because
that adapter only supports whole-target scope. One scoped Standard scan supplies
the missing current sixteen-library source cell instead; no whole-repository
scan or replacement scan is claimed.

A fresh-context independent baseline reads101 implementation, manifest,
guidance, fixture and settings files. Two focused independent investigations
check Session/Valkey lifecycle/admission and Config/Secrets/Signing trust;
their overlapping paths count once. Parent inspection covers19 remaining
lock/license data files and verifies alias destinations and source bindings.
All worker role/model/effort traces match their resolved configurations.
The exact historical `THREAT-MODEL.md` is retained as provided model context,
not a new application or deployment qualification.

Checked controls include private authentication resolution and request-local
identity, authoritative Session revisions/rotation/revocation/uncertain outcomes,
worker-owned permits after caller cancellation, finite DOM/same-origin action
boundaries, HTML escaping/CSP, typed settings refusal, secret error redaction,
explicit signing identity/pins/revocation and trusted build-time WGSL.
Synthetic examples/generated test keys are distinguished from usable embedded
credentials; no operational credential is identified or used.

The historical whole-repository scanfc4d4af8 at25bff remains pre-patch with its
one Low finding and accepted focused repair. Auth scan6b4f365a retains its
six-file dirty-source scope. Neither old finding count nor review is rewritten.
Current source-scan completion does not approve failed license checks, remove
the advisory warning or establish absence of vulnerabilities.

Canonical scan/model/findings/coverage/generated report and supplemental raw
inventory remain in the private workbench under that scan ID. No raw exploit
details are copied into the repository. The inventory bundle has370 members,
1,253,786 bytes, SHA-256
`aea012fd932cfa173ca57931c5708bae681d4c67c007feebaec7d5715676c4ce`.
Its raw commands, failed results, SBOMs, metadata, locks/hashes and reconciliation
are retained; source bindings, advisory summary and role traces are separate
private supplemental artifacts.

Workbench reports complete usage measurement: total12,963,915 tokens,
input12,907,827 and cached input12,483,328. These are tool-reported token counts,
not a monetary cost or an external audit assurance statement.

## Limits, delivery review and next step

No new application, runtime, public API, sample, dependency, lock, license policy,
ignore, workflow or support promise is introduced. No functional/native/browser/
GPU/IdP/Valkey/Azure workload, old successful qualification or hosted CI is rerun.
Source inventory does not cover exact compiler events/features/output hashes,
Rust sysroot, native OpenSSL/linker/system closure, JS/glue or deployment assets.
Those existing limits remain explicit; no unsupported target is made a gate.

No listener, browser, container or cloud resource is created. Four task-owned
package/Cargo-home/audit-home/database directories are removed after review;
the unused build directory was never created. Raw inert records, the private
bundle and all canonical/supplemental bindings remain unchanged. This delivery
changes this report and Backlog/Roadmap/
Release Readiness summaries only, with no Commit, Push or Merge of new work.

Independent Reviewer accepts the actual failed-check reporting and source/advisory
boundaries while **YT-204 remains In progress**; this does not approve
the proposed policy edit. Next is the narrow license decision and its five failed
checks, followed by any separately authorized advisory patch evaluation. YT-205
remains unopened; YT-096/D1–D6 and YT-020 are unchanged.

## Independent delivery acceptance — 2026-10-27

Reviewer `yt204_inventory_security_delivery_review_20261027`: **PASS, zero
issues and zero required followups for this bounded delivery.** It verifies
four candidate docs, five sealed canonical and six supplemental files,138
protected source/alias bindings, unchanged policy/model, sixteen archive
bindings, six retained cells and all thirty-six new SBOM closures. It confirms
the five actual license failures, eight errors,25 unused-allowance warnings,
zero source errors and sixteen audit results with ten Tokio warnings.

The native trace verifies reviewer role/model `gpt-6-astra` and effective
effort `high`; the resolver is enforced with no conflict or degradation.
Review runs no workload or mutation. Its exact result, source/evidence binding,
observed dispatch, documentation checks, tool usage and subsequent owned cleanup
are retained in the repository-targeted private supplemental collection under
`artifacts/yt204-delivery-20261027/`. Candidate inputs match through acceptance;
only review/cleanup/navigation metadata changes afterward. Existing docs checks
pass with75 required documents/205 backlog items; tracked and new-file whitespace
checks pass, with empty-diagnostic no-index exit1 denoting new content only.

This PASS accepts the reporting, **not YT-204 Done, license-policy amendment or
release readiness**. All four new documents remain uncommitted, staged files
remain zero, and no Push or Merge occurs.

## Authorized license-policy followup — 2026-10-27

The owner explicitly approves adding only `Zlib`, `BSL-1.0` and `ISC` to the
license allowlist, preserving notices and every other policy, rerunning only
the five failed checks, and marking YT-204 Done only after independent Reviewer
PASS. No ignore, dependency upgrade, Commit, Push or Merge is authorized.

`deny.toml` changes exactly that allowlist line. Structural comparison against
the previous policy confirms the sole semantic delta is those three IDs:
confidence0.95, private/dev/build inclusion and registry/Git source restrictions
remain identical. Package manifests, locks, runtime sources, existing notices
and previous scan artifacts remain unchanged. The supply-chain guide is updated
to describe the actual policy and its scope.

The three needed package trees are restored byte-for-byte from the accepted
archives to their original task-owned temporary paths. This preserves the
exact five retained metadata files without rewriting paths or resolving them
again. Existing cargo-deny0.20.2 runs offline against those metadata inputs,
the same target/feature selections and the newly authorized policy.

| Only rerun check | Exit | License errors | Source errors |
|---|---:|---:|---:|
| Rate-limit GNU default | 0 | 0 | 0 |
| Rate-limit MSVC default | 0 | 0 | 0 |
| Redis Session GNU default | 0 | 0 | 0 |
| Redis Session GNU `tls` | 0 | 0 | 0 |
| wgpu WASM `browser` | 0 | 0 | 0 |

All five now PASS. Thirty-nine retained metadata/package/lock/source inputs and
ten relevant third-party license/notice files match their before-check hashes;
notice bytes also match the accepted vendor checksums where listed. Six new
warnings are unused-allowance diagnostics, not license/source errors.

No metadata, SBOM, advisory, source-scan, build or functional-test command is
replayed. Prior31 plus6 retained passes remain valid for their exact inputs:
the only policy change broadens the set of accepted license IDs, while unused
allowances remain warnings. Combined with these five affected checks, selected
license/source coverage is42/42; this is not42 fresh executions. Original failure
logs, source-scan result and Tokio warning remain intact at their prior identities.
New followup evidence is kept separately in the repository-targeted private
collection under `artifacts/yt204-license-20261027/`.

### License scope and distribution obligations

The [cargo-deny allowlist](https://embarkstudios.github.io/cargo-deny/checks/licenses/cfg.html#exceptions)
applies to the entire dependency graph checked with this configuration,
including future dependencies using these IDs. No package/version-specific
exception is created. Yetlet's own `MIT OR Apache-2.0` license is unchanged;
each dependency retains its own license and copyright. Passing the checker
does not itself fulfill redistribution obligations or approve publication.

| SPDX license | Permitted use and actual conditions |
|---|---|
| [Zlib](https://spdx.org/licenses/Zlib.html) | Allows use including commercial applications, modification and redistribution. Do not misrepresent origin; mark altered source versions and retain the notice in source distributions. Product-document acknowledgment is appreciated rather than required by this text. |
| [BSL-1.0](https://www.boost.org/LICENSE_1_0.txt) | Boost Software License1.0 permits use, copying, modification and distribution. Copies/derivatives retain copyright and the full statement, with the stated exception for copies solely in machine-executable object-code form generated by a language processor. It imposes no requirement to publish application source. |
| [ISC](https://www.isc.org/licenses/) | Permits use, copying, modification and distribution for any purpose with or without fee. Copyright and permission notices must accompany all copies. Keep the applicable license text with redistributed copies. |

These three texts permit commercial/closed-source use and do not impose a
network-use source-disclosure requirement. `BSL-1.0` is the **Boost** license.
For compliance planning, distinguish backend-only SaaS operation from supplying
copies to customers: retain the applicable third-party notices with shipped
source/native packages and downloaded JS/WASM bundles. Do not assume the Boost
object-code exception covers every browser-asset form; keeping matching notices
avoids relying on that interpretation. A combined expression such as ring's
`Apache-2.0 AND ISC` retains both sets of obligations.

This is a summary of the cited texts, not a new legal guarantee or blanket
approval of eventual artifacts. Review actual contents and their accompanying
notices before distribution; this followup creates no new browser/binary bundle.
Final publication/content selection remains separate.

## Final policy acceptance — 2026-10-27

Reviewer `yt204_license_allowlist_completion_review_20261027`: **PASS, zero
issues and zero required followups.** The reviewer confirms the exact three-ID
policy delta, all five affected executions and the six candidate/five followup
evidence bindings. It verifies39 protected inputs,10 notices/vendor checksums,
138 existing source/alias bindings, five prior canonical files and six prior
supplements without mismatch. The reported license scope agrees with the
inspected component texts and retains all distribution obligations.

The native trace verifies reviewer role/model `gpt-6-astra` and effective
effort `high`; the resolver is enforced without conflict or degradation.
Review performs no edits, workload, network or Git mutation. Exact result,
dispatch observation and source/evidence binding remain in the private followup
collection. Inputs match through acceptance; only completion/navigation metadata
changes afterward. Restored package trees and the owned Cargo home are removed;
inert raw records and the private evidence bundle remain preserved.

YT-204 is Done at its selected inventory/security scope. Six changed paths
remain uncommitted and staged files remain zero. Tokio's informational warning,
historical source findings and candidate/registry/native/system inventory limits
are unchanged. YT-205 is next but unopened; YT-096/D1–D6 and YT-020 remain
separate. No Commit, Push, Merge or publication occurs.
