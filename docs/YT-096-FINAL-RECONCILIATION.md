# YT-096 — inventory and platform evidence reconciliation

Date: 2026-10-20. **Documentary reconciliation complete; release qualification
remains partial.** Candidate remains
`ce56e0814e5423782440acd2c959ca777d572cd4`, original nine experimental,
unpublished 0.1.0 libraries, Rust MSRV 1.99 / qualification toolchain 1.99.0.
Independent ordinary slice review passed with zero issues or required followups. YT-096 remains In progress;
YT-020 remains Blocked.

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
| Linux Chrome | Accepted YT-105 consumer workflow PASS on Chrome 155.0.8059.84; native Chrome exit 0 and cleanup PASS retained | Historical consumer receipt has no original independent candidate-wide source manifest. No fresh candidate Chrome execution or retroactive source identity is claimed; final support acceptance remains an explicit decision |
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

## Residual decisions and release boundary

The evidence comparison is finished at its documented scope. It does not
silently expand support or mark every release gate PASS:

1. Final support acceptance must explicitly address historical Chrome consumer
   evidence and the absence of a full original candidate receipt. Reusing PASS
   results as requested does not invent the missing binding or require a rerun
   during this slice.
2. Final inventory acceptance must retain the two conservative SBOM entries,
   two native test/dev-only packages and non-Cargo/per-library/output-digest
   omissions. Complete compiled-binary/system inventory remains Not verified.
3. Formal builder/source/tag/trust policy, authenticated artifact/SBOM/provenance
   binding, signatures, actual registry rights/download/consumer verification,
   independent final security/release review and maintainer release approval
   remain open. Signing/publication are outside this authorization.

Windows OIDC and repeated Windows IME/assistive-technology work retain the
owner's waivers. Optional WebKit/mobile/macOS/Windows GNU/device/performance,
application HA/durability/OIDC/certificate-custody and GPU capabilities receive
no new qualification. Required/scheduled CI and compatibility/API classifications
remain unchanged. No runtime or external resource was started by this slice.
New reconciliation documents remain uncommitted pending the owner's next step.

The next work that needs no signing or publication is an independent final
candidate security/release review and a concrete maintainer decision on the
recorded support/inventory limits. That review cannot substitute for the
remaining formal trust/registry execution or final release authorization.

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
