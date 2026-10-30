# Alpha.5 — Two-package release scope and remaining gates

Date: 2026-10-30 (Asia/Taipei). Status: proposed release plan, not an immutable
candidate selection or execution approval. The owner authorizes committing the
accepted consolidation, then documenting scope, candidate inputs and missing
verification. This planning step does not Push, Merge, create a tag, execute
hosted jobs, change Environments/Secrets, sign or publish anything.

## Committed input and proposed release scope

| Input | Actual value / disposition |
|---|---|
| Accepted migration | [YT-211](YT-211-TWO-PACKAGE-MIGRATION.md), Done at migration and affected local-proof scope after independent followup PASS0; original contract/final review failures retained |
| Local source checkpoint | `0f867f11a7e73675158597f0ec3caff71f1d8d0f` |
| Tree | `4c612e6b3418641114f91bff2e66fee58930f3c6` |
| Parent / branch | `678adfec81f1878b0f3d0a886d8a12ef2e83eccc`; `codex/alpha4-registry-continuation` |
| Commit authority/result | Owner-authorized strict/builtin staged scan and native hook; zero blocking findings, twelve non-blocking UNC/drive-path warnings. Exact scanned tree equals commit tree; worktree clean immediately after Commit; no Push |
| Proposed product version/cohort | `0.9.0-alpha.5`; schema2 / `consolidated`, exactly `yetlet` and `yetlet-browser` |
| Final C / remote master CI | Pending: checkpoint above is an implementation input, not a selected release C. The concrete gaps below must close before final source reconciliation |
| Tag / S / V / P / hosted outputs | Not selected or created for alpha.5; no frozen payload, run, artifact or signature identity is invented |

The two public implementation packages share one View type. `yetlet` owns the
portable model and opt-in native modules, with default HTML; `yetlet-browser`
owns DOM/action transport/optional GPU integrations, with default DOM. Publish
order is **yetlet, then yetlet-browser**. There is no reverse development edge,
bootstrap package or old-crate facade fleet. Library tests/examples, Lua/WGSL,
package guides and dual licenses remain archive inputs.

CLI, applications, Lab scenes, benchmark fixtures and repository helpers stay
unpublished. Applications select additive Cargo features and explicit startup
options; Settings loading is optional. No automatic service installation, new
runtime API, renderer feature, DI/AOP or third-party upgrade is proposed.
The migration's compatible serde_json pin alignments use identities already
present in the previous lock union; this is not a claim that every lock block
or dependency declaration stayed byte-identical.

Alpha.5 intentionally changes Rust crate/module imports. Follow the
[migration contract](YT-211-TWO-PACKAGE-MIGRATION.md) and
[naming guide](NAMING.md#two-package-module-paths--yt-211), including
`auth-session`, `session-redis-tls` and `wgpu-browser`. No source compatibility
alias is promised for the former sixteen crate names.

## Proposed feature and platform boundary

Keep the existing experimental support limits, mapped to modules rather than
sixteen independently installable packages. A package count reduction does not
qualify every feature on every target. These are proposed alpha.5 claim limits;
missing current-source cells below are not silently accepted historical passes.

| Capability | Proposed selected boundary | Exclusions / evidence rule |
|---|---|---|
| View/HTML, portable Actions and Reactive | Rust1.99; GNU/MSVC native, portable wasm32 builds | One shared View; browser HTTP execution belongs to Browser. Default/core-only graphs remain light |
| Server, request ID, Memory Session, Auth/Memory bridge, rate-limit, Config/Secrets | GNU/MSVC native; explicit feature combinations and application installation | No native API/secret/storage stack in WASM; no general OIDC, business authorization or distributed-quota promise |
| Signing | GNU native with the pinned static OpenSSL3.5.9 recipe | MSVC/macOS/WASM signing remains unselected; migration tests do not qualify arbitrary system OpenSSL |
| Redis-protocol Session and TLS | GNU native, selected Valkey integration boundary | Existing cross-host/revocation/restart failures and results keep their original inputs; no new HA/failover/restore or Windows-provider claim |
| DOM/keyed DOM/browser Actions | wasm32 plus selected desktop Chrome/Firefox DOM journeys; native static-policy checks remain separate | New archive/asset inputs need applicable proof; linking or compiling is not an actual browser/CSP result |
| WebGPU | CPU policy/model and wasm32 `wgpu-browser` build; retain original device-specific observations separately | No new alpha.5 hardware/device qualification or blanket Chrome WebGPU claim. Physics Lab's private renderer is not SDK GPU evidence |

No new Azure topology, long capacity/soak run, performance optimization campaign,
mobile/WebKit/IME/assistive-technology matrix or production certification is
proposed. A wider claim needs its own actual gap and selection.

## Accepted evidence to retain

[YT-211's result table](YT-211-TWO-PACKAGE-MIGRATION.md#implementation-and-affected-evidence--2026-10-30)
is the detailed local proof index: both SDKs native/WASM, all34 example
manifests,192 workspace example tests, selected isolated combinations,169 base
and18 Browser accepted tests,21 moved example tests,11 doctests, CLI creation/
export/compilation, four actual post-build native HTTP smokes, normal two-archive
predecessor checks, native/WASM archive consumers, shared View identity and
native-API refusal. The generated CLI browser project also compiles for WASM.
Auth-only target selection and bridge-enabled doctests close the two final P2
findings without replaying their unchanged runtime cases.

Default/core-only normal graphs contain only `yetlet`; the broad WASM union
excludes native HTTP/Session/crypto/storage/settings/rate-limit stacks. All
current tracked locks introduce zero external registry version/source/checksum
identities beyond the previous lock union. These statements retain their exact
recorded feature/target/source limits.

Raw results, source maps/snapshots, both review failures and PASS, cleanup and
archive receipts are retained outside the repo at
`~/.local/share/yetlet-release-evidence/preparation/yt211-two-package-migration-20261030/`.
The verified988-file retention manifest excludes build/dependency/compiler
caches. That owner's path is custody context, not a public attachment or a
portable deployment requirement.

**Do not promote the local migration archives into release artifacts.** Some
reviewed guide/rustdoc, test-eligibility and Auth example-guard corrections follow
their recorded packaging inputs. The old archives and hashes remain valid proof
for those inputs. Final C needs fresh normally verified archives and exact
source/manifest/lock/checksum correspondence, not another replay of unaffected
runtime tests. The pre-migration Windows, browser/GPU, security and performance
records retain their original scope; the
[YT-202](YT-202-NATIVE-FEATURE-GAPS.md),
[YT-203](YT-203-WASM-DOM-GPU-EVIDENCE.md) and
[YT-204](YT-204-PACKAGE-SECURITY-COVERAGE.md) matrices are historical evidence,
not alpha.5 package names or new-source acceptance labels.

## Concrete remaining work, in order

| Gate | Smallest remaining work | Exit evidence / reuse limit |
|---|---|---|
| A. Active workflow/source vocabulary | `.github/workflows/qualification.yml` still runs `-p yetlet-core` and eight other retired library names. Migrate its active native selection to the two packages and explicitly equivalent selected features, preserving OS/support limits. Evidence/sign/result workflow input descriptions still say schema1 despite accepting schema2; align that user-facing input text | Focused workflow/argument/feature check and required PR CI; no hosted release execution and no broad new platform claim |
| B. Current distribution/consumer instructions | The generated Docker guide still describes two old dependency path replacements; its actual overlay now has one `yetlet` edge. Update that guide. Reconcile current headers in Compatibility, Documentation Scope and release-policy navigation with the two-package source, while preserving dated history and all accepted old outputs | Exact guide/overlay correspondence and documentation checks. Existing generated-source proof is retained unless an executable template/input changes |
| C. Delivery and final source | Under later Git authority, deliver reviewed remaining corrections/docs and the consolidation through required PR CI, then record the actual merged immutable checkpoint R, its SHA/tree and master CI. Preserve the original branch/history and paused PR29; do not auto-Merge its old continuation-only scope | Exact `Rust 1.99 MSRV` and `Stable native, samples, and Chrome` checks on actual delivered source. R is the input to D/E; select unchanged R as C only after their required evidence is accepted. Do not prefill a future squash SHA |
| D. Final two-package assembly | Normally package immutable checkpoint R in base-to-Browser order, staging the verified base archive/checksum. Verify normalized manifests, packaged locks, licenses/notices, included source and external native/WASM consumers. Produce current inventory/security/license correspondence for the changed package/feature/target map | No bootstrap, `--no-verify`, ignore or policy downgrade. Reuse unchanged upstream/advisory/license decisions; check only changed or still-missing closure cells with the existing tools |
| E. Remaining platform cells | At the same R, supplement the new source/package MSVC default and supported integrated consumer; exclude Signing/Redis Windows claims. Use required CI's new Chrome assets/results and add only the remaining selected Firefox DOM/CSP observation. Keep actual GPU observations historical unless fresh hardware qualification is explicitly selected | Record exact source/archive/consumer/tool/feature/target and cleanup. Do not replay old long workloads, Valkey/IdP/cloud tests or complete device matrices |
| F. Original hosted build/custody | After C and explicit execution authority, create its immutable unsigned tag, run the two-package build and independent original readback, compare the final local/hosted bytes and retain complete private evidence outside transient artifacts | New C/tag/run/attempt/receipt/digest bindings; historical alpha.4 hashes cannot substitute. Build45min; original readback preflight5min/protected15min are existing workflow bounds, not a dispatch grant |
| G. Signing and independent result | Select reviewed S distinct from C; use fresh complete schema2 record, encrypted qualification handoff, prepare and freeze both raw payload hashes, sign, then independently verify both actual bundles, two package subjects and complete private manifest. Preserve S/V/P separation and actual master CI | Existing automatic-worker policy and custody/trust pins remain; prepare/sign/result each retain their committed timeout and single-attempt/no-automatic-retry rules. Source or payload changes require disposition, not reuse of an old signature |
| H. Distribution/rights and final owner gate | Refresh read-only name/owner/verified-email/index checks for both names. Confirm the existing90-day `CRATES_IO_TOKEN` can create/update these names; old sixteen-name scope or Secret presence alone is insufficient. Prepare exact six-file public plan/notes and owner-gated GitHub-then-registry promotion | Final owner approves the actual `release-publish` run after preflight; verify every GitHub asset and registry archive/checksum, then a fresh registry-only two-package consumer. Each write once; preserve failure/429, no automatic retries |

A/B are newly identified static carryovers, not failures from rerun CI or hosted
jobs. They are concrete pre-release followups, and this plan does not implement
them or claim the consolidation checkpoint is release-ready. Old-name strings
in the intentionally preserved legacy-cohort test fixtures remain legitimate;
they must not be mechanically renamed as if all were active package selection.

## Future artifact and automation binding

The new core record is schema2, version `0.9.0-alpha.5`, cohort `consolidated`,
and the eventual exact C. Its only valid library subject set is
`{yetlet, yetlet-browser}`. Build/custody/S/provenance/Release fields are added
from actual outputs in their defined freeze order. No executable request input
selects an arbitrary package list.

Proposed public attachments total **six**: two final `.crate` archives,
`PROVENANCE-UNSIGNED.json`, `RELEASE-CHECKSUMS.json`,
`provenance.sigstore.json`, and `SIGNING-APPROVAL.json`. The existing helpers
already derive these names/counts from the validated cohort. Real bytes/hashes,
release notes and acceptance summary remain pending actual verification. The
private manifest, encrypted private signing bundle, credentials and private
qualification evidence are not public attachments.

Keep the existing automatic `release-sign` and `release-result-verify` workers
restricted to master, no admin bypass, unchanged age recipient/custody. Before a
new original readback, reconcile `release-evidence-verify`'s exact selected tag
policy under later Environment authority. Do not recreate Secrets or rotate the
age key merely for this version. If the publication token is limited to old
names, the owner updates its dedicated scope/Secret without exposing its value.
Signing may disclose the already-adopted certificate/workflow/transparency
fields; no new metadata policy or new human signing gate is introduced here.
The final publication Environment remains the owner's gate.

Historical D6 acceptance is not reset or transferred to new alpha.5 payloads.
New outputs follow the adopted [single-publication policy](RELEASE-PUBLICATION.md).
Existing alpha.4 C/tag/GitHub twenty assets/signatures and the published
`yetlet-actions0.9.0-alpha.4` stay immutable. The remaining fifteen-package
continuation remains paused and is invalid for schema2. Do not re-upload that
published archive, retag old bytes, resume its210minute job, or treat either
new name as a registry alias for the old package.

## Recommended next bounded step

First repair A/B with focused checks and independent review, without changing
runtime or dependencies. Then request the appropriate Git/PR delivery authority,
confirm exact-source CI and complete only missing D/E evidence against that
immutable checkpoint R; choose unchanged R as final C after acceptance. This planning document remains
uncommitted; the current instruction grants none of the later Git, Environment,
tag, hosted, signing or distribution operations above.
