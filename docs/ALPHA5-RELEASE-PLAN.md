# Alpha.5 — Two-package release scope and remaining gates

Date: 2026-10-30 (Asia/Taipei). Status: YT-212 Done at A-E qualification scope after
final independent PASS0; unchanged R is selected as candidate C. The owner separately
authorizes strict/builtin delivery through PR30, conditional squash Merge, MSVC
verification and selecting unchanged R as C after all necessary evidence passes.
This batch does not create a tag, execute release-hosted jobs, change Environments
or Secrets, sign or publish. The initial planning checkpoint and failures below
remain historical evidence.

## Later source and execution selection — 2026-10-31

The owner subsequently authorizes advancing to the final publication gate.
[The alpha.5 hosted record](ALPHA5-HOSTED-RECORD.md) records the new
C `0a9fa8a594e3dd2ce753422dd848df10cd8c98df`, after PR31 and actual master CI,
with unchanged package inputs and the now-qualified Windows xtask correction.
Old `0f0c84a`, the original qualification limits and all failed outputs below
remain historical. Tag/build/readback/S/frozen payloads and publication have
separate actual records; this does not promote a future output to PASS.

## Committed input and proposed release scope

| Input | Actual value / disposition |
|---|---|
| Accepted migration | [YT-211](YT-211-TWO-PACKAGE-MIGRATION.md), Done at migration and affected local-proof scope after independent followup PASS0; original contract/final review failures retained |
| Local source checkpoint | `0f867f11a7e73675158597f0ec3caff71f1d8d0f` |
| Tree | `4c612e6b3418641114f91bff2e66fee58930f3c6` |
| Parent / branch | `678adfec81f1878b0f3d0a886d8a12ef2e83eccc`; `codex/alpha4-registry-continuation` |
| Commit authority/result | Owner-authorized strict/builtin staged scan and native hook; zero blocking findings, twelve non-blocking UNC/drive-path warnings. Exact scanned tree equals commit tree; worktree clean immediately after Commit; no Push |
| Proposed product version/cohort | `0.9.0-alpha.5`; schema2 / `consolidated`, exactly `yetlet` and `yetlet-browser` |
| Qualification R / current acceptance | `0f0c84abf9b6c269987cef31c2d0651fa2d1f3ff`, tree `a0940f6806fc92ac8b16b96fc4827645ac82b316`; PR30 and master CI PASS. Final independent acceptance PASS0; unchanged R selected as C |
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

## Original qualification and remaining release gates

| Gate | Smallest remaining work | Exit evidence / reuse limit |
|---|---|---|
| A. Active workflow/source vocabulary | Completed at R: the previous `-p yetlet-core` and eight retired library selections are replaced by the two SDKs. Migrate its active native selection to the two packages and explicitly equivalent selected features, preserving OS/support limits. Evidence/sign/result workflow input descriptions still say schema1 despite accepting schema2; align that user-facing input text | Focused workflow/argument/feature check and required PR CI; no hosted release execution and no broad new platform claim |
| B. Current distribution/consumer instructions | Completed at R: generated Docker guides describe the actual single `yetlet` overlay, complete SDK closure and conditional source-path mapping. Reconcile current headers in Compatibility, Documentation Scope and release-policy navigation with the two-package source, while preserving dated history and all accepted old outputs | Exact guide/overlay correspondence and documentation checks. Existing generated-source proof is retained unless an executable template/input changes |
| C. Delivery and final source | Completed under separate owner authority: reviewed corrections and consolidation delivered through PR30; exact merged R, tree and master CI recorded below. Preserve the original branch/history and paused PR29; do not auto-Merge its old continuation-only scope | Exact `Rust 1.99 MSRV` and `Stable native, samples, and Chrome` checks on actual delivered source. R is the input to D/E; select unchanged R as C only after their required evidence is accepted. Do not prefill a future squash SHA |
| D. Final two-package assembly | Normally package immutable checkpoint R in base-to-Browser order, staging the verified base archive/checksum. Verify normalized manifests, packaged locks, licenses/notices, included source and external native/WASM consumers. Produce current inventory/security/license correspondence for the changed package/feature/target map | No bootstrap, `--no-verify`, ignore or policy downgrade. Reuse unchanged upstream/advisory/license decisions; check only changed or still-missing closure cells with the existing tools |
| E. Remaining platform cells | At the same R, supplement the new source/package MSVC default and supported integrated consumer; exclude Signing/Redis Windows claims. Use required CI's new Chrome assets/results and add only the remaining selected Firefox DOM/CSP observation. Keep actual GPU observations historical unless fresh hardware qualification is explicitly selected | Record exact source/archive/consumer/tool/feature/target and cleanup. Do not replay old long workloads, Valkey/IdP/cloud tests or complete device matrices |
| F. Original hosted build/custody | After C and explicit execution authority, create its immutable unsigned tag, run the two-package build and independent original readback, compare the final local/hosted bytes and retain complete private evidence outside transient artifacts | New C/tag/run/attempt/receipt/digest bindings; historical alpha.4 hashes cannot substitute. Build45min; original readback preflight5min/protected15min are existing workflow bounds, not a dispatch grant |
| G. Signing and independent result | Select reviewed S distinct from C; use fresh complete schema2 record, encrypted qualification handoff, prepare and freeze both raw payload hashes, sign, then independently verify both actual bundles, two package subjects and complete private manifest. Preserve S/V/P separation and actual master CI | Existing automatic-worker policy and custody/trust pins remain; prepare/sign/result each retain their committed timeout and single-attempt/no-automatic-retry rules. Source or payload changes require disposition, not reuse of an old signature |
| H. Distribution/rights and final owner gate | Refresh read-only name/owner/verified-email/index checks for both names. Confirm the existing90-day `CRATES_IO_TOKEN` can create/update these names; old sixteen-name scope or Secret presence alone is insufficient. Prepare exact six-file public plan/notes and owner-gated GitHub-then-registry promotion | Final owner approves the actual `release-publish` run after preflight; verify every GitHub asset and registry archive/checksum, then a fresh registry-only two-package consumer. Each write once; preserve failure/429, no automatic retries |

A/B were static carryovers discovered during planning. Their corrections are now
accepted at delivery scope and present in R; this does not make the original
consolidation checkpoint or old artifacts release-ready. Old-name strings
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

## Actual qualification results — YT-212

| Evidence | Actual result / source limit |
|---|---|
| Delivery | Local corrections `82cc685`, then integration `17693f5`; strict/builtin staged and exact outgoing range scans plus native hooks PASS, twelve previously inspected non-blocking path warnings. [PR30](https://github.com/bohewu/yetlet/pull/30) squash-merged; original branch/history and paused PR29 preserved |
| Immutable R | `0f0c84abf9b6c269987cef31c2d0651fa2d1f3ff`; tree `a0940f6806fc92ac8b16b96fc4827645ac82b316`, equal to reviewed PR tree |
| Required CI | [PR run41075755771/1](https://github.com/bohewu/yetlet/actions/runs/41075755771) and [master run41076622827/1](https://github.com/bohewu/yetlet/actions/runs/41076622827): both MSRV and Stable/native/samples/Chrome jobs PASS; no retry. Chrome156.0.8078.47, three actual DOM suites |
| Local assembly / consumers | Normal base then Browser verification and final-predecessor checksums PASS; fresh native/WASM archive consumers PASS, native Auth API refusal on WASM accepted. All101 SDK/root manifest/lock/license/toolchain inputs match R, so unchanged GNU proof is reused rather than replayed for workflow/root-guide edits |
| MSVC | Online WebCodex runner `ncut-bohe-244143825ae3488e`: exact R, Rust1.99.0, VC14.44.35207 / SDK10.0.26100.0; both normal packages and general / integrated Auth-Session plus SecretProvider consumers PASS. The general consumer explicitly selects features; it does not mean all capabilities are SDK defaults. Signing, Redis provider and actual GPU are excluded |
| Cross-platform archive binding | Both GNU/MSVC archives are byte-identical: `yetlet` SHA256 `662d67180cec2fd6e50c3757bdc910d057cd37428785ab695b0d275a9e4e5091`; `yetlet-browser` SHA256 `2fbeec5d4324367cf682d722fc25a35f851fc5293fc6e353aedc35ca6771c9a6`. MSVC consumer lock also matches the locally inventoried consumer, SHA256 `0ffb747be8105a215c3fd02eaf286208a0b2e15a74d27da4e700141edb7723a4` |
| Firefox | Firefox159 Todo, keyed bindings, multi-island dashboard and forms/accessibility suites PASS with current assets and cleanup receipts. Existing native/WASM/GPU/provider/benchmark evidence retains its original scope; no new physical GPU, macOS or cloud qualification is claimed |
| Inventory / security | Unchanged policy: GNU/native160, selected MSVC89, broad WASM73 normal/build dependencies; licenses/sources PASS, CycloneDX1.5 SBOM components exactly match each indexed closure. Current packaged lock RustSec PASS0 vulnerabilities/warnings; exact-source CI also passes its root audit. No ignore, allowlist change or dependency upgrade |
| Focused workflow / guides | Original nine-capability selection, equivalent native feature compile, generated service/settings/Vite export/compile, template golden, workflow shell syntax and repaired xtask checks PASS; docs75 / backlog212 and whitespace PASS at the correction checkpoint |
| Independent review | A/B and local CI repairs PASS0 before Commit; final evidence/closeout review PASS0 (Reviewer/gpt-6-astra/high verified). YT-212 Done at the selected qualification scope |
| Custody / cleanup | Task-owned Windows checkout, dependency cache, vendor and builds removed;71 inert receipts/archives retained separately (1,557,858 bytes), manifest SHA256 `d956dfafaa9c221cb059f4b5ad2196f45b024d096b735e14432db2a63de54a31`. Linux source maps/logs/receipts/failures are retained outside the repo at `~/.local/share/yetlet-release-evidence/preparation/yt212-alpha5-qualification-20261030/`; compiler/dependency caches and rejected ZIP payloads are excluded |

The Windows ZIP metadata policy rejection remains recorded; no transfer was
attempted and the payload was not relocated or encoded to bypass it. Separately
authorized public Git clone provides the reviewed source. One PowerShell/cmd
environment quoting failure is retained. Repository-only `xtask` failed to
compile on MSVC in its pre-existing POSIX Docker branch (`qualify/docker.rs:194`,
E0277); SDK qualification then uses the same normal Cargo commands directly.
At R this is an unresolved repository tooling limit, not a passing Windows xtask
claim. The later YT-213 working-tree correction below retains that original
failure; it does not silently alter the selected immutable C.

Windows Git's inherited `core.autocrlf=true` initially converted `git archive`
inputs to CRLF. Those successful CRLF builds/consumers and their checksums are
retained at that scope; they do not bind the final LF archives. A single canonical
`git -c core.autocrlf=false archive` correction, without global configuration
changes, supplements only the affected package/consumer inputs in the same task.
The final LF archives and consumer lock exactly match GNU. Clone/fetch/vendor,
CI, browsers and unrelated accepted tests were not repeated. Earlier local
workspace/fixture-license, stale scene-path and Clippy failures are also retained
with their focused corrections; none is relabeled as an original PASS.

Final independent acceptance is PASS0. Under the owner's conditional authority,
candidate C is `0f0c84abf9b6c269987cef31c2d0651fa2d1f3ff`, tree
`a0940f6806fc92ac8b16b96fc4827645ac82b316`. The qualification sources and accepted
archives are unchanged; these closeout documents do not change SDK/package inputs.
The next bounded step is an owner-authorized alpha.5 tag/two-package hosted build
and original readback at that exact C (F), then newly bound prepare/sign/result
and final owner-gated publication (G/H). These operations are not authorized or
executed by this qualification batch. Historical D6 and all old releases/tags,
signatures, registry packages and paused continuation remain unchanged.

## Windows xtask followup — YT-213

The owner separately authorizes fixing the repository-only MSVC compilation
limit above before the next release step. This does not reopen YT-212's SDK
qualification or relabel its original E0277/CRLF/operational failures. The selected
immutable C `0f0c84abf9b6c269987cef31c2d0651fa2d1f3ff` remains recorded; this
uncommitted tooling correction is not silently included in it.

A private `Result<String>` POSIX identity helper replaces the diverging, inferred
`str` local. Non-Unix Lab construction rejects before files/sockets/Docker are
created, and a directly constructed Lab's build rejects before filesystem writes.
Unix keeps its uid/gid, Docker arguments, limits, scenario and cleanup. No new
Windows Docker support, package/API dependency or framework runtime is added.

| Affected proof | Actual result / limit |
|---|---|
| GNU | `cargo test --offline --locked --manifest-path xtask/Cargo.toml --test qualify_multi_instance`:24 PASS, including new Unix setup preservation. Scoped library/binary/test Clippy with `-D warnings` PASS; ordinary `--help` PASS |
| MSVC | Rust1.99 / VC14.44.35207 / SDK10.0.26100.0 on the existing Windows runner: same two source hashes as GNU; normal `cargo test --offline --locked ... --test qualify_multi_instance`:13 PASS, including both suite entrypoints/actual CLI refusal and direct-build refusal before writes. `--help` and `check docs` PASS; the latter reads the immutable R fixture's75 documents/212 items |
| Platform limit | Docker qualification still requires POSIX; no Docker command/container is started by these checks. Unix's existing identity, arguments, limits and cleanup are unchanged. No new Windows Docker capability claim |
| Existing warning | Windows `release/checksums.rs::sort_json` is unused. An extra task-local `RUSTFLAGS=-D warnings` experiment failed on it after the original E0277 was fixed; that failure is retained. Normal compilation leaves the warning visible; no allow/ignore or unrelated checksum-source edit. Zero-warning Windows lint is not claimed |
| Source access / custody | Parent-project canonical editing refused the `target` artifact path. Formal registration of the exact existing checkout reauthorized its source root under Runner authority; the registered Project reports `allow_patch=true` and canonical source edits PASS. No relocation or alternate script write. Source status/hash and bounded diff were preserved before unregistering the temporary Project.19 inert receipt/source files retained (61,549 bytes); clone/cache/build removed, manifest SHA256 `e4c959d310cf657d8058a2db13741a85f383a5311e75aacb89e488200ebd9175` |
| Independent acceptance | PASS0, registered Reviewer/gpt-6-astra/high verified; YT-213 Done at tooling scope. No Git/tag/release operation |

Only affected tooling checks are supplemented. All101 selected SDK/package inputs
and xtask Cargo manifest/lock remain byte-identical to the previous accepted
source. Existing SDK/archive/WASM/browser evidence is reused, with no rerun.

For a future alpha.5 tag to include this fix, separately authorized guarded
Commit/PR delivery, actual CI and a new candidate selection are needed; preserve
old C and compare package inputs before reusing its unchanged SDK proof. This
batch does not Commit, Push, create a tag, execute release-hosted, sign or publish.
