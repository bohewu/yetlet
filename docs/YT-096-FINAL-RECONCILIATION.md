# YT-096 — inventory and platform evidence reconciliation

Date: 2026-10-20. **Documentary reconciliation complete; release qualification
remains partial.** Candidate remains
`ce56e0814e5423782440acd2c959ca777d572cd4`, original nine experimental,
unpublished 0.1.0 libraries, Rust MSRV 1.99 / qualification toolchain 1.99.0.
Independent ordinary slice review passed with zero issues or required followups. YT-096 remains In progress;
YT-020 remains Blocked.
The owner subsequently accepted the bounded support scope and known inventory
differences on 2026-10-20; missing evidence remains Not verified as recorded
in the [acceptance decision](#owner-support-and-inventory-acceptance-2026-10-20).

## Authority, method and unchanged source

The owner requested committing the reviewed GitHub rebuild, then reconciling
existing build inventories and selected platform/browser evidence without
rerunning passing work, followed by independent reviewer review. Signing and
publication remain prohibited. The first step committed six reviewed documents
as `5fb01dc54e512a6d89b26a718d06b2bf9c4763d1`; strict/builtin staged Commit Guard
passed with zero findings and the exact scanned tree was verified. That local
commit was not pushed. Neither documentation commit changes the selected
candidate or any source-package byte.

This slice reads retained records, joins archive/lock/SBOM identities and checks
their hashes. It adds no product change, dependency, validator framework,
workflow, runtime test, browser run, build, audit or new SBOM. The
[reconciliation record](../artifacts/yt096-final-reconciliation-20261020/RECONCILIATION.json)
contains complete package/target/digest links. All 721 earlier evidence files
and 63 selected source inputs remain unchanged. Historical failures, PASS
results, original reviewer identities and owner waivers remain intact.

## Intended artifacts and inventory binding

The intended distributables remain exactly nine source `.crate` archives.
Compiled executables, generated wasm/JS/browser assets, CLI/config/secrets,
Redis/Valkey/GPU providers, samples and frontend packages are outside that
distributed scope. Qualification fixtures do not become release artifacts.

| Package | GNU inventory | MSVC inventory | wasm inventory |
| --- | --- | --- | --- |
| yetlet-core | Retained SBOM | Retained SBOM | Retained SBOM |
| yetlet-actions | Retained SBOM | Retained SBOM | Retained SBOM |
| yetlet-reactive | Retained SBOM | Retained SBOM | Retained SBOM |
| yetlet-html | Retained SBOM | Retained SBOM | Outside selection |
| yetlet-dom | Retained SBOM | Retained SBOM | Retained SBOM |
| yetlet-dom-keyed | Retained SBOM | Retained SBOM | Retained SBOM |
| yetlet-server-axum | Retained SBOM | Retained SBOM | Outside selection |
| yetlet-request-id | Retained SBOM | Retained SBOM | Outside selection |
| yetlet-session | Retained SBOM | Retained SBOM | Outside selection |

The direct join confirms all **23** package/target records bind the original
archive SHA-256 and packaged lock SHA-256, default features, no explicit
feature additions, and unchanged CycloneDX 1.5 output bytes. The existing
36-entry unsigned bundle binds those nine archives, 23 SBOMs and four supporting
records. Its bytes and identities remain unchanged; it was not regenerated or
signed. Full archive hashes remain in the
[accepted archive table](YT-096-TWO-HOST-REBUILD.md#archive-comparison).

The [source-inventory comparison](YT-096-SUPPLY-CHAIN-DELTA.md#target-inventory-and-sbom-coverage)
has 23/23 normal/build metadata matches, 21 exact normal/build Cargo-tree matches
and two conservative server-axum differences. GNU and MSVC server-axum SBOMs
retain `serde_derive 1.0.229`, enabled through development-feature unification.
No missing normal/build tree component is recorded. This is conservative
source resolution, not an exact feature graph for each compiled library.

The actual [Azure compiler observations](YT-096-AZURE-RUN-COMMAND.md#actual-cargo-observations-and-inventory-limits)
contain 140 Cargo events: 97 native-test events / 70 package-version pairs and
43 wasm-build events / 32 pairs. Native 68/70 occur in the GNU SBOM union;
`tower-http 0.6.11` and transitive `bitflags 2.13.2` are the two known test/dev-only
pairs in retained development metadata. All 32 wasm pairs occur in the wasm SBOM
union. Every observed version occurs in the unchanged root lock. No unexplained
package/version difference is recorded. Aggregate coverage does not establish
per-library feature parity; no new Windows compiler-artifact observation exists.

The existing 23 license/source checks and nine package-lock RustSec results are
reused at their original source/policy/database/time. RustSec used database
commit `999ec89d5d3f10f20be14750ec8c2d8b44e0ac9b` and finished at
2026-10-20 07:53:16 UTC. This reconciliation performs no fresh scan or claim
about future advisories. The original 331-directory staging cache and root
research workspace resolution are not substituted for release inventories.

**Inventory limits remain:** Cargo output-file hashes, exact per-library
compiled features, Rust sysroot, implicit/native linker and complete system
inputs are not established. Tool hashes and the Azure dpkg list are separate
observations, not a complete closure. JS/loader/glue assets are outside Cargo
SBOM coverage and outside the nine source distributables. The source SBOM bundle
and hosted unsigned provenance are separate retained objects; this crosswalk
does not create an authenticated combined release statement.

## Final evidence table at the selected scope

| Selected cell | Reused evidence and result | Exact limit |
| --- | --- | --- |
| Ubuntu 24.04 family x64 GNU / Rust 1.99.0 | Actual Ubuntu 24.04.5 Azure guest: nine selected workspace packages, 83 native tests and four doctests PASS | Kernel 7.0.0-1017-azure; native baseline only, not Ubuntu 24 desktop/browser qualification. Earlier shared-kernel container PASS remains historical |
| Windows x64 MSVC / Rust 1.99.0 | Nine unchanged standalone package roots: 83 native tests and four doctests PASS, matched tool/archive/lock inputs | Windows build 26300 / Visual Studio 2022 Professional; not GNU or Windows-browser execution. Core PASS retained before the eight-package resume; original cache failures remain preserved |
| Five wasm libraries | core/actions/reactive/dom/dom-keyed packaged builds PASS; actual Azure workspace build also PASS | Build qualification only, no wasm test-suite execution or universal browser support |
| Linux Firefox | Source-reconciled Todo/Search debug workflow PASS on Ubuntu 26.04.1, Firefox 158.0 / geckodriver 0.37.1 | Retained receipt and 133 source inputs; browser-internal diagnostics retained. Not an Ubuntu 24 desktop run |
| Linux Chrome | Accepted YT-105 consumer workflow PASS on Chrome 155.0.8059.84; native Chrome exit 0 and cleanup PASS retained | Historical consumer receipt has no original independent candidate-wide source manifest. The owner accepts historical reuse only; exact-candidate Chrome qualification remains Not verified |
| Source-archive rebuilding | Original Linux, Windows, empty-cache Azure and separately operated GitHub-hosted outputs match all nine complete archive bytes | Source archives only; no compiled executable/wasm reproducibility or authenticated builder statement |

Native counts and browser coverage refer to the existing
[platform record](YT-096-PLATFORM-QUALIFICATION.md),
[Azure guest](YT-096-AZURE-RUN-COMMAND.md),
[hosted rebuild](YT-096-GITHUB-HOSTED-REBUILD.md) and
[Chrome consumer verification](YT-105-GRAPH-SNAPSHOT.md#consumer-verification).
No historical signature, failure or earlier source identity was reassigned.

The retained browser-source record matches the candidate's Git blobs and
current file identities for all **133** entries, including 18 license aliases;
all **eight** loader/glue/wasm/CSS/SVG asset hashes match the recorded debug
digest `2b8ea18c1298900c`. Candidate Cargo.lock selects wasm-bindgen 0.2.129;
the source-bound xtask checks CLI/crate version equality before building. No
separate CLI binary digest is invented. Existing asset hashes and a present-day
source check cannot retroactively create a missing Chrome execution manifest.

One local inspection initially compared a dereferenced license alias to a Git
record hashing its link text. A focused mode-aware read correction compared
Git mode120000 entries using link bytes and regular files using file bytes;
all 133 entries then matched. Materialized archive license bytes remain covered
by the original package inventory. No source, evidence or runtime test changed.
The [read correction](../artifacts/yt096-final-reconciliation-20261020/BROWSER-SOURCE-MATCH.json)
records that bounded inspection failure separately from product behavior.

## Owner support and inventory acceptance (2026-10-20)

After the final candidate security/release review was committed as `2118495`,
the owner explicitly accepted the recommended experimental source-package
scope and known inventory differences, instructed that missing evidence remain
Not verified, and requested documentation plus independent reviewer review.
This is the maintainer's support/inventory disposition for immutable candidate
`ce56e0814e5423782440acd2c959ca777d572cd4`, not final release approval.
It requires no runtime rerun and authorizes no signing or publication.

| Subject | Accepted scope or known difference | Impact and retained qualification limit |
| --- | --- | --- |
| Distributables | Exactly the nine default-feature experimental, unpublished 0.1.0 source packages listed above; Rust MSRV 1.99 / pin 1.99.0 | CLI/config/secrets/signing, Redis/Valkey/GPU, samples, frontend glue and generated executables/browser assets do not join this release train. Existing API classifications and best-effort experimental policy stay unchanged |
| Native baselines | Existing Ubuntu 24.04 family x64 GNU and Windows x64 MSVC evidence, each with 83 native tests/four doctests PASS | Accept only the actual environments and source identities in the evidence table; no arbitrary kernel, architecture, Windows GNU or Windows-browser support expansion |
| Five wasm libraries | core/actions/reactive/dom/dom-keyed build qualification | No wasm test-suite execution or universal browser-runtime qualification is inferred |
| Linux Firefox | Source-reconciled Todo/Search debug workflow on Ubuntu 26.04.1 / Firefox 158.0 | No Ubuntu 24 desktop, optimized-browser-build or arbitrary Firefox-version qualification is inferred |
| Linux Chrome | Reuse the historical YT-105 consumer workflow PASS at its original scope | The original candidate-wide source manifest is absent. Exact-candidate Chrome qualification remains Not verified; current hashes cannot retroactively supply execution identity |
| Conservative source inventory | Retain serde_derive 1.0.229 in the GNU and MSVC server-axum SBOMs as documented dev-feature over-inclusion | Accept the conservative source-resolution difference without deleting or rewriting SBOMs; exact compiled feature equivalence is not established |
| Known test/dev-only packages | Retain tower-http 0.6.11 and bitflags 2.13.2 as the two native test/dev-only observations outside the GNU SBOM union | These are explained development differences, not unidentified missing runtime packages. The recorded comparison has no unexplained package/version gap |
| Inventory omissions | Use the 23 archive/lock/target-bound source-resolution SBOMs and recorded aggregate Cargo observations at their actual scopes | Exact per-library compiled features/output hashes, Rust sysroot, complete native/linker/system closure and new Windows compiler-event coverage remain Not verified. Separate tool/dpkg observations are not a complete binary/system SBOM; JS/loader/glue assets remain outside Cargo coverage and the distributables |
| Other capabilities | Preserve historical PASS results and existing owner waivers without expanding this candidate's scope | No new WebKit/mobile/macOS/Windows GNU/device/performance, application HA/durability/IdP, certificate-custody or GPU qualification; no production SLA or 1.0 API stability claim |

The support/inventory acceptance finding now has an explicit maintainer
disposition. Acceptance records limits; it does not manufacture missing
evidence, promote a Not verified cell to PASS, authenticate a builder, merge the
unsigned source/SBOM bundle with hosted provenance, or qualify compiled-output
reproducibility. Original archives, SBOMs, logs, failures, reviewer verdicts
and scan/database timestamps retain their original bytes and meaning.

## Residual decisions and release boundary

Four existing release conditions remain open after this limited acceptance:
formal integrity/trust (including approved source/tag, authenticated builder
and artifact/SBOM/provenance binding, certificate/chain and transparency proof),
actual registry rights/upload/download/consumer qualification, applicable
public activation/reporting controls, and concrete final maintainer release
approval. The [original final review](YT-096-SECURITY-RELEASE-REVIEW.md) remains
FAIL / release-ready NO; this decision does not replace its verdict or the
[release supply-chain policy](RELEASE-SUPPLY-CHAIN.md). YT-096 stays In progress
and YT-020 stays Blocked. Signing/publication remain outside this authorization.

Windows OIDC and repeated Windows IME/assistive-technology work retain the
owner's waivers. Optional WebKit/mobile/macOS/Windows GNU/device/performance,
application HA/durability/OIDC/certificate-custody and GPU capabilities receive
no new qualification. Required/scheduled CI and compatibility/API classifications
remain unchanged. No runtime or external resource was started by this slice.
The reconciliation was subsequently committed as `c0726d2` before the final
candidate review; that documentation commit preserves the selected source and
all earlier evidence identities.

The subsequent [final candidate security/release review](YT-096-SECURITY-RELEASE-REVIEW.md)
is now complete: technical/security evidence accepted with limits, overall
release gate FAIL and release-ready NO because existing evidence/approval
conditions remain unmet. The subsequent owner decision above resolves only
support/inventory-limit acceptance. The next preparation that needs no
signing/publication is an explicit proposal for the remaining trust/identity
and source/tag policy; actual signing, registry/public activation and final
release approval still require their separate authority and evidence.

## Independent ordinary slice review

Reviewer `/root/yt096_final_reconciliation_review_oct20` returned PASS with
zero issues, required followups or optional notes. Native trace verification
matched the registered reviewer role, model gpt-6-astra and effective high
effort; the shared resolver observation is enforced. The
[exact verdict](../artifacts/yt096-final-reconciliation-20261020/REVIEW-RESULT.json)
and [bounded acceptance](../artifacts/yt096-final-reconciliation-20261020/REVIEW-ACCEPTANCE.json)
accept this documentary reconciliation and its preserved limits. Final edits
record that verdict and documentation/preservation checks only. This ordinary
slice review is separate from the remaining final security/release gate and
does not approve support expansion, signing, publication or release activation.

## Owner-acceptance documentation review

Reviewer `/root/yt096_support_inventory_acceptance_review_oct20` returned
PASS with zero issues, required followups or optional notes for the subsequent
seven-document owner-decision update. Native trace verification matched the
registered reviewer role, gpt-6-astra model and effective high effort; the
shared resolver observation is enforced. The
[exact result](../artifacts/yt096-support-inventory-acceptance-20261020/REVIEW-RESULT.json)
and [verified receipt](../artifacts/yt096-support-inventory-acceptance-20261020/REVIEW-ACCEPTANCE.json)
cover this documentation scope only. These closing lines record that returned
result; they do not reissue the original final release verdict.

The documentation checks and preservation receipts are retained in
[the focused checks](../artifacts/yt096-support-inventory-acceptance-20261020/CHECKS.json).
All 760 prior YT-096 evidence files, 63 selected source inputs and six unchanged
security/supply-chain/API policy files retain their hashes. No runtime workload
was repeated or external resource started. This update remains uncommitted
and unpushed; no signing, publication or release activation occurred.
