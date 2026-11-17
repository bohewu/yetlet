# Yetlet Roadmap — Evidence-Gated Milestones

## M28 — Composable Application and Multi-Host Research

**Architecture Proposed; existing Web baseline recorded.** The
[M28 study](M28-COMPOSABLE-APPLICATION-PLAN.md) explores Composable Application
Framework without replacing pure Web/API/SSR.
Host, renderer and execution topology remain independent capability choices:
View supports HTML/DOM; game scenes use separate GPU backends; hosts may select
zero/one/multiple renderers and need not own dedicated threads/processes.
Single-writer commands/snapshots apply to the shared simulation POC, not all
HTTP requests. [ADR 0013](adr/0013-composable-application-and-hosts.md) is **Proposed**.

- [x] **M28-A / YT-265 clarification:** initial review at `4e5bdc1` retained; independent delta Reviewer `wc_job__Fg38dxSv31AqFr5` actual PASS, terminal exit 0, no issues/followups; document checks PASS. Renderer/lifecycle/Web-preservation planning only, no runtime result implied.
- [x] **M28-B0 / YT-276:** [Existing Web baseline](M28-COMPOSABLE-APPLICATION-PLAN.md#10-b0-existing-web-baseline--yt-276) at `30b0625`: 55 selected tests plus 8 doctests and five normal dependency/feature observations PASS. Independent Reviewer `wc_job_gxUREPbXwflV_qFr` actual PASS, terminal exit 0, no issues/followups. No product/template changes, performance certification or new Host claim; Git delivery remains separate.
- [x] **M28-B1 / YT-266:** [Private same-thread POC](M28-COMPOSABLE-APPLICATION-PLAN.md#11-b1-private-same-thread-multi-host-proof--yt-266): two concrete independent-lifecycle hosts, one owner, bounded commands and immutable shared Snapshot with `Rc` local data. Focused 7/7, Fmt/Clippy, demo and zero-normal-dependency graph PASS. Independent Reviewer `wc_job_q8KjqtfssJDRTlml` actual PASS, terminal exit 0 with zero issues/followups; private research only, no public Host API or B2 evidence.
- [x] **M28-B2 / YT-267:** [Private owned-worker proof](M28-COMPOSABLE-APPLICATION-PLAN.md#12-b2-owned-worker-and-required-host-lifecycle-proof--yt-267): one explicitly owned std worker, two caller-thread Host attachments, bounded command/snapshot ports, startup/runtime fault rollback, saturation-independent stop, timeout-then-join and borrowed scheduler preservation. Original R1 `wc_job_Nh4ynWCpWGPFMEFG` P2 FAIL retained and fixed; focused 1/1 and complete 15/15 tests, Fmt/Clippy, two demos and zero external deps PASS. Independent R2 `wc_job_UqJr-AVptjqv0L0D` terminal/exit0 actual PASS, issues=[]/followups=[]. Private POC only; no public API or C/D claim.
- [x] **M28-C1 / YT-268:** [Twin Squares headless proof](M28-COMPOSABLE-APPLICATION-PLAN.md#13-c1-deterministic-headless-twin-squares--yt-268): one integer reducer, canonical bounded inputs, Pause/Resume, whole-square bounds, checked counters and immutable snapshots. Focused 13/13 PASS (15 prior B1/B2 tests not rerun), Fmt/Clippy, display-free demo and zero external dependencies; independent Reviewer `wc_job_9AM-fykfMMzURQb4` actual PASS, terminal exit 0, zero issues/followups. Private research only; Git delivery separate, C2 not started.
- [x] **M28-C2 / YT-269:** [Native opt-in integration](M28-COMPOSABLE-APPLICATION-PLAN.md#14-c2-opt-in-native-gamehost--yt-269): Ubuntu X11/winit with RTX 4090 Vulkan wgpu Surface, actual two-player keyboard movement, Pause/Resume, bounded fixed ticks and separate rendering. Original R1 `wc_job_BLgscEDX_x91p33c` and R2 `wc_job_L7Eq2gsTBng989Xo` P2 FAIL retained; private pending Pause/Resume projection fix passed focused 1/1 and full Native 11/11 tests plus Fmt/Clippy. Independent R3 `wc_job_Pi934pMwws_L1XwD` terminal exit0 actual PASS with zero issues/followups. Git delivery separate; C3 visual/lifecycle, UDP and public Host API not claimed.
- [ ] **M28-C3 / YT-270:** actual native GPU/window visual, resize/minimize/error/recovery and scheduler/performance qualification remains separate and unstarted.
- [ ] **M28-D / YT-271–273:** bounded protocol and UDP between two application processes, each composing its required hosts; controlled faults, reconnect/stop and headless network mode.
- [ ] **Conditional LAN / YT-274:** two physical devices when available; loopback never substitutes for that evidence.
- [ ] **M28-E / YT-275:** promotion decision requires B0 pure-Web preservation as well as selected C/D evidence; no compulsory actor/queue/tick/HostGroup on Web requests. Packaging, template and positioning remain separate choices.

M28-A/B0/B1/B2/C1 are complete at their separately accepted scopes.
C2 is the selected private implementation slice; C3/D/E remain Todo and require separate authority.
No Axum refactor, public API, dependency/default-feature change, SDK/CLI release
or public product-description rewrite is included. See the
[complete task dependency table](M28-COMPOSABLE-APPLICATION-PLAN.md#7-work-breakdown-and-dependencies).

## M27 — Real-world Developer Experience

- [x] **Correct CLI first-use lock guidance (YT-260).** Align the first-use sequence in `docs/CLI-GUIDE.md` with CLI 0.4.0: `yetlet new` already creates the exact app lock and SDK-resolution receipt; fetch the existing lock with `cargo fetch --locked` before build/run. Preserve explicit offline-cache and no-implicit-install boundaries. Independent final Reviewer `wc_job_ab4Zl3f_I3Q3_Fu4` PASS.
- [x] **Document the generated Service operational contract (YT-261).** Add conditional `--service` guidance to the generated README: liveness/readiness routes, readiness transition on graceful shutdown, and the distinct bind configuration for plain Service versus Service with Settings/Vite. Documentation only; no runtime behavior change. Independent final Reviewer `wc_job_ab4Zl3f_I3Q3_Fu4` PASS.
- [x] **Windows x64 MSVC first-use and Native Watch (YT-262).** The [PowerShell CLI Guide](CLI-GUIDE.md#windows-x64-msvc-first-app-and-native-watch) and First App link cover CLI 0.4.0 / SDK alpha.8, environment-only Doctor, read-only Inspect and locked Native build/run/watch. Actual MSVC Native Watch edit/restart/Ctrl+Break (`0xC000013A`) and owned cleanup passed; the separate M26 first-app result supplies ordinary Run/Ctrl+C/CSP evidence. Windows docs and independent Reviewer `wc_job_6WNK0nuaj1AQDaRL` PASS. Documentation-only, no Browser/Vite or release qualification.


- [x] **Windows Browser first use and bounded CLI Ctrl+C race fix (YT-263).** Add the [PowerShell Browser guide](BROWSER-APP.md#windows-x64-msvc-powershell-first-use) with exact CLI/SDK pins, trusted physical Node selection and locked WASM/Vite preparation; reuse scoped Ubuntu Chrome evidence for shared Browser tasks and ordinary CSP without requiring Windows Chrome. The source-only Windows CLI cleanup fix has focused MSVC 1/1, same-invocation Ctrl+C native exit and process/5173 cleanup evidence plus independent CLI-only Reviewer PASS. Deliberate CSP negative-enforcement and Browser Watch/HMR remain out of scope. Integrated Reviewer PASS, Strict Guards PASS, [PR #97](https://github.com/bohewu/yetlet/pull/97) merged at `597c760` with exact PR/master Rust/Docs/CodeQL CI and live Pages readback PASS. No CLI package release; the source-only fix is reserved for the next candidate.

- [x] **CLI 0.4.1 patch release (YT-264).** [Source preparation](CLI-041-RELEASE-PREPARATION.md) and [published Hosted Record](CLI-041-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-17): CLI source PR #98, signer dossier PR #99, independent review, Strict Guard and CI PASS; immutable C/tag, original Build, encrypted readback, signer S and independent verifier PASS. Owner approved [publication Run 44025334874](https://github.com/bohewu/yetlet/actions/runs/44025334874), which published an exact five-asset GitHub Experimental pre-release and byte-identical crates.io CLI 0.4.1; installed registry-only consumer six cases with Native Style and Browser Style Chrome/WASM runtime PASS. SDK alpha.8 unchanged. The Reader Edition version selection is updated in this documentation-only follow-up, pending its own PR/CI/Pages delivery.

The M27-A/B audit and prioritization are recorded in [DX Gap Report](M27-DX-GAP-REPORT.md) and [Prioritized Plan](M27-PRIORITIZED-PLAN.md). Both work items received independent plan Reviewer R4 PASS and independent implementation Reviewer `wc_job_ab4Zl3f_I3Q3_Fu4` PASS; YT-260 and YT-261 are complete at their bounded documentation scope.

## M26 — Application Startup & Sample Experience

- [x] **SDK `0.9.0-alpha.8` application startup and Axum prelude.**
  `App::serve()` binds development HTTP to `127.0.0.1:7878`.
  `App::serve_on(address)` accepts an explicit TCP address, and the optional
  `server_axum::prelude` supplies common imports. Applications that own the
  listener or coordinate graceful shutdown can use `into_router()` with native
  Tokio, Axum and Tower. The two published SDK packages passed a fresh locked
  registry consumer. Separately, PR #86 passed Rust, Docs and CodeQL on merged
  commit `9d1075ab6442a9d8cb58606c170327aa32799bf7`. Native Windows x64 MSVC
  qualification at that source revision passed 148 tests across the SDK Axum
  adapter and all 13 official samples.
  [Startup API](API-REFERENCE.md#published-application-startup--sdk-alpha8) · [Sample startup paths](SAMPLES.md#startup-paths-in-this-checkout) ·
  [SDK publication](ALPHA8-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-12) · [M26 source qualification](https://github.com/bohewu/yetlet/blob/1508b6910add4e74c9e2311a3f3eb4666539afce/docs/YT-251-258-APPLICATION-STARTUP-SAMPLE-PLAN.md#10-next-handoff)

- [x] **CLI `0.4.0` SDK-aware scaffolding.**
  An exact SDK archive capability match selects the simple startup for eligible
  Native and Native Style templates. The published CLI passed six hosted Linux
  generation cases. Separate locked registry first-app runs with exact SDK
  `0.9.0-alpha.8` passed on Linux and Windows x64 MSVC. On both platforms,
  `/` and `/hello/Ada` returned HTTP 200 with the default CSP, and the app lock
  and creation receipt were preserved. The Windows run also verified Ctrl+C
  shutdown and reuse of the selected test port. At that M26 checkpoint,
  the Reader Edition selected SDK `0.9.0-alpha.8` and CLI `0.4.0`; the
  current CLI0.4.1 edition is a separate publication.
  [CLI startup profiles](CLI-GUIDE.md#generated-startup-profiles) ·
  [CLI publication](CLI-040-HOSTED-RECORD.md#owner-approved-publication-and-installed-consumer--2026-11-12) ·
  [First App](FIRST-APP.md) · [Reader Edition selection](../site/pages.toml)

**M26 authoring and tooling outcomes.** Typed View builders (YT-234/235) first
shipped in SDK alpha.7 with 101 constructors, selected setters and compile-time
void-child guards over the owned View tree. CLI 0.3.0 first shipped private
application Style generation (YT-237–241) with explicit module registration,
deterministic external CSS and default-CSP delivery. Ordinary CSS, optional
Tailwind/Vite and the native-first checkout Style Showcase retain their own
scope. The SDK resolver (YT-247) separates CLI build SDK/snapshot/template
identity from each generated app's exact registry pins, locks and receipt.
CLI 0.3.1 adopted `sdk-latest-v2`, maximum non-yanked SemVer including
prereleases, followed by selected-version checks without fallback. Cargo owns
toolchain/dependency feasibility. CLI 0.4.0 adds the startup scaffolds above.
[View guide](VIEW-COMPONENTS.md#typed-tag-builders) ·
[Style workflow](CLI-GUIDE.md#optional-application-owned-rust-css-yt-239-working-tree) ·
[Style registration](CLI-CONTRACT.md#application-owned-rust-style-registration-and-css-publication) ·
[Resolver contract](CLI-GUIDE.md#sdk-resolution-yt-247) ·
[ADR 0010](adr/0010-dynamic-sdk-version-resolution.md).

**Dated qualification and navigation outcomes (2026-11-07–12).** YT-242's
516-file role inventory inspected 54 entries semantically. Its initial P2 and
YT-243's Windows OpenSSL prerequisite finding were corrected before independent
acceptance. Linux Chrome/Firefox, local MSVC/Wasm, exact package consumers and
generated native Style HTTP/CSS retain separate source bounds. The corrected
CI setup passed seven Style tests locally, with hosted image/CI execution still
separate. YT-244–246 distinguish source-only API/package audit, alpha.7
preparation, accepted distribution and Reader authorities. YT-248's original
publisher preflight FAIL precedes its separately accepted registry/Style
consumer PASS. YT-250 preserves both CLI 0.3.1 candidates, Windows Browser Style
shutdown correction and the replacement's fresh hosted/signing/consumer proof.
YT-249's edition rollover requires reviewed README/Pages follow-through after
each verified publication. Development versions do not advance the Reader.
[Backlog acceptance](BACKLOG.md#yt-237yt-250--style-delivery-sdk-resolution-and-release-preparation) ·
[Style source qualification](QUALIFICATION-MATRIX.md#historical-two-package-migration-and-style-source-qualification) ·
[CLI 0.3.0 publication](CLI-030-HOSTED-RECORD.md) ·
[CLI 0.3.1 preparation](CLI-031-RELEASE-PREPARATION.md) ·
[CLI 0.3.1 publication](CLI-031-HOSTED-RECORD.md) ·
[Reader rollover](DOCUMENTATION-SCOPE.md#historical-reader-source-and-rollover-evidence).

**Repository and reader milestones (2026-10-31–11-07).** YT-236 is Done at
local source/static CodeQL concurrency scope. Four language scans and
protected-master/weekly/manual analysis remain, same-PR supersession is scoped,
and Rust overlay caching remains unsupported. Hosted cancellation behavior
was unverified at acceptance. YT-233's separate cache/evidence layout, nine
GNU/Windows focused cases and scoped inventories retain their Windows path
failure, unavailable role trace and lost/not_started listing. Later explicitly
authorized compiler-internals cleanup observed +53.20 GiB and preserved
retained sources/evidence. The remote full-result readback timeout remains.
YT-231 corrected the SQLite race test to admit the existing finite Busy503,
prove one committed item/history and strict post-commit stale409, preserving
250ms/worker/runtime/security bounds. YT-230 added manifest-derived release-doc
gates, preserved its first boundary/history failure and rejected the historical
CLI020 README at its actual scope. These Done results retain independent review.
[CodeQL evidence](CODEQL-CI-PERFORMANCE.md#historical-codeql-pr-concurrency-and-rust-analysis-evidence) ·
[Development artifacts](DEV-ARTIFACTS.md) ·
[SQLite correction](../examples/lab-operations-board/README.md#historical-board-desktop-and-sqlite-concurrency-evidence) ·
[Release-document gate](RELEASING.md#release-documentation-gate).

YT-217's local eight-chapter preview retained two sidebar assertion failures,
then passed the owner-authorized eight-URL/four-fragment correction and local
navigation/search/touch/screenshots. YT-218 separately delivered PR43/master
`64d1074` and first Pages deployment `41198529190/1`, actual public bytes and
hosted reader acceptance. Settings/harness404 observations remain historical.
YT-232's later alpha.6/CLI0.2 edition passed eleven affected checks and local
native/Wasm/static sources before PR54/master `993ce56` and first-attempt
Pages delivery. Its fifteen HTTPS files/219 links and final four-language
CodeQL PASS retain the original browser-unavailable boundary. YT-226's later
Yetlet-built cutover and current edition have separate records.
[Reader milestones](DOCUMENTATION-SCOPE.md#historical-reader-source-and-rollover-evidence) ·
[First Pages delivery](GITHUB-PAGES-PUBLICATION.md#completed-delivery-and-first-publication--2026-10-31) ·
[Public reader boundaries](RELEASE-READINESS.md#public-reader-acceptance-boundaries).

**Historical release milestones (2026-10-29–11-02).** The dated gates in this
table belong to their original source and checkpoint. Current distribution is
SDK alpha.8 / CLI 0.4.1, established by the accepted records above. Formal item
states and substantive acceptance remain in [Backlog](BACKLOG.md).

| Dated milestone | State, boundary and canonical proof |
|---|---|
| 2026-11-01–02, YT-227–229 | Planning selected SDK alpha.6 before independent CLI0.2, retained exact SDK binding and Reader hold, and deferred OIDC Trusted Publishing. SDK Cc7e6372/SVP6531001 verified two subjects/all 44 private entries before owner-approved41308935654/1 published six public files/both registry packages and passed a fresh locked consumer. CLI C7fb963a/SVP86e5da2 verified one subject/all 54 entries before 41319625709/1 published five files/exact archive and passed install/version/help/new. Original failures and preparation-only Pending gates remain in the [SDK record](ALPHA6-RELEASE-PREPARATION.md) and [CLI record](CLI-020-RELEASE-PREPARATION.md) |
| 2026-10-31, YT-214–216 | Source-installed CLI implementation/package/first-app proof preceded actual publication. CLI0.1.0 uploaded and installed but failed its stale version label before help/new. Wrong SDK-ref custody receipts remain rejected. Separate CLI0.1.1 Cae5dd4c/SVP1207ba5 passed actual install/version/help/scaffold, preserving the first master SQLite FAIL and authorized failed-Stable-only retry. No release-stage retry or SDK change. [First CLI releases](COMPATIBILITY.md#historical-first-cli-releases-and-consumer-failures) |
| 2026-10-30–31, YT-210–213 and alpha.5 | Four-module one-package feasibility passed normal archive/native/Wasm/target-isolation proof before the explicit two-package selection. Initial contract/final test-guide P2s remain corrected historical findings. PR30 R0f0c84a qualified native/Wasm/MSVC/DOM/inventory, with matching GNU/MSVC archives and consumer lock. Windows xtask POSIX E0277 was resolved separately by YT-213 GNU24/MSVC13 tests, explicit pre-effect platform rejection and retained warning. Hosted alpha.5 selected distinct C0a9fa8a/SV864f2ab/Pbb414d1, owner-approved41104391525/1 and actual registry consumer. Obsolete actions crate deletion/PR29 closure and branch-recovery bundle retain their own scope. [Migration](COMPATIBILITY.md#two-package-feature-and-target-contract), [qualification/failures](ALPHA5-RELEASE-PLAN.md#actual-qualification-results--yt-212), [publication](ALPHA5-HOSTED-RECORD.md#completed-publication-and-subsequent-cleanup) |
| 2026-10-30, alpha.4 / YT-020 | Sixteen-version preflight retained third-party bytes. Prepare/sign/V passed at S/V6ed70c3, while the first publisher failed GitHub staging. Later GitHub twenty-file publication and only the first registry write succeeded before an incorrect success-response assumption stopped the job. Remaining fifteen/consumer were unexecuted. An owner-selected eleven-minute-before-each-PUT/210-minute continuation with live authority checks stayed separately gated and was later paused for consolidation. Account quota was unproven. [Preflight](ALPHA4-VERSION-PREFLIGHT.md), [immutable outcome](ALPHA4-HOSTED-RECORD.md#actual-public-release-and-one-package-registry-outcome), [continuation contract](RELEASE-PUBLICATION.md#registry-only-continuation-contract--2026-10-30) |
| 2026-10-30, first-name registry bootstrap | Config/Secrets normal publish dry-runs retained offline/rustup failures and changed packaged-lock checksums, requiring new artifact identities. The adopted contract moved the Settings example to Config, removed the reverse development edge and required all-kind ordering/frozen exact-byte promotion. Sixteen offline envelopes and prototype checks established local feasibility only. Absent-name GETs/existing login did not prove claimability or rights. Original plan/receipt P2 and Guard refusal remain historical. [Dependency evidence](RELEASE-PUBLICATION.md#dated-local-evidence-and-remaining-provenance), [artifact contract](RELEASE-PUBLICATION.md#first-registry-publication-and-immutable-artifact-contract) |
| 2026-10-29–30, alpha.3 | Used-binding retirement, sixteen normal packages/external consumer/nineteen roots/four version cases passed. Optional synthetic readback remained unverified after a recipient-pin harness mismatch. C557ab76/SV30baf25 later completed build/custody,16 subjects/all 112 entries and exact20 public files. Original Python upload failure remained, with approved gh-api recovery and no registry claim. [Recovery](ALPHA3-VERSION-PREFLIGHT.md), [hosted/signing/publication](ALPHA3-HOSTED-RECORD.md) |
| 2026-10-29, alpha.2 | Local sixteen-package/toolchain/lock preflight passed. PR22 Stable then failed the stale-alpha.1 synthetic rebuild fixture while MSRV passed; focused source-derived fixture correction and independent repair passed, with required delivery/CI still gating Merge. Distinct Caf7bbe5/tag/build40895514392/1 passed packaging but failed custody for alpha.1 receipt fields, with no decrypt/verifier/sign/publication. Local three-check correction could not qualify that output. [Migrated preflight/gates](RELEASING.md#historical-alpha2-version-package-and-release-gates--2026-10-29), [original failure/correction](QUALIFICATION-MATRIX.md#subsequent-pr22-integration-failure-and-local-correction), [immutable hosted failure](ALPHA2-HOSTED-RECORD.md) |
| 2026-10-29, YT-208/209 | Maintainer SOP preceded versioned data-only records, protected no-OIDC V and closed public staging. Thirty focused checks and independent review accepted local implementation. PR20 importlib-consumer CI FAIL preceded reviewed compatibility repair, same-tree squash7de77c8 and CI40873628263/40875245510 PASS. Owner setup of result Environment25684187945 and AGE_IDENTITY passed, with positive hosted qualification Pending then and later alpha.3 V accepted separately. [Record contract](RELEASING.md#versioned-record-and-receipt-contract), [integration failure](CI-SECURITY-REVIEW.md#versioned-release-automation-integration-history) |
| 2026-10-28–29, alpha.1 D6 | Distinct Cbf33638/Sd65e318, unsigned build40669449040/1/verifier40688772221/1, successful prepare40806988578/1 and sign40811275645/1 bind both frozen payloads,16 subjects/all 117 entries. One spent owner-local custody exception, exact-output acceptance and independent review closed D6/YT-096. Separate approval published20 Experimental GitHub files with no registry upload. Earlier prepare40717870186 preflight,40789535074 HTTP403 and40795794183 Go-cache-cleanup FAILs remain ineligible. [Exact frozen/signed identities and acceptance](RELEASE-READINESS.md#alpha1-frozen-inputs-signatures-and-acceptance--2026-10-29) |
| 2026-10-27–28, original unsigned/custody work | Earlier C3eacfe3/v0.1.0 build40628943583/verifier40631789238 and owner-local51-file retention stayed distinct from alpha.1's57-file retention. No offsite/indefinite availability followed. The LOCAL synthetic-key rotation drill from4793620 passed nine cases/two builds/25 commands after an extraction failure. CI-lane planning and the separate signer D1/D3 amendment retained Pending implementation/S/manifest/D6 gates at their dates. [Hosted history](RELEASE-READINESS.md#historical-release-and-hosted-source-checkpoints), [rotation](QUALIFICATION-MATRIX.md#custody-and-local-rotation-evidence--2026-10-27-and-28), [ADR 0009](adr/0009-release-build-and-signer-identities.md) |

The complete dated M26 chronology, exact intermediate Git/CI/source-freeze
identities, reviewer scopes and original failed attempts remain at local Git
`8d6fad7e95ee25849872b974f922d10c71b70a95`. Raw logs/signatures/custody stay at
their original ignored/external locations. Local object recovery establishes
neither remote GitHub reachability nor continued raw-evidence availability.
[Original M26 chronology](https://github.com/bohewu/yetlet/blob/8d6fad7e95ee25849872b974f922d10c71b70a95/docs/ROADMAP.md#m26--application-startup--sample-experience).

Milestones are sequenced by capability and dependencies rather than invented delivery dates. The [MVP specification](MVP.md) provides acceptance criteria. Each gate must retain real commands, results, and relevant architectural decisions.

| Milestone | Outcome | Backlog | Exit evidence |
|---|---|---|---|
| **M0 — Design baseline** | Specifications, architecture, ADRs, backlog, repository agent guidance | YT-001 | Linked docs, unique task IDs and dependencies, proposals distinguished from verified facts |
| **M1 — Native SSR foundation** | Cargo workspace, minimal shared components, safe HTML renderer, HTTP adapter | YT-002–YT-007 | `GET /` produces escaped HTML without JS; native tests pass |
| **M2 — Browser DOM/WASM** | Browser build, DOM mount/local state, shared components | YT-008–YT-010 | One component source compiles for both targets; local click updates DOM |
| **M3 — Selective islands** | External loader, asset manifest, island activation, bounded initial state | YT-011–YT-013, YT-017 | `/search` renders complete SSR, loads only the island on interaction, never forwards raw events |
| **M4 — API/action integration** | Typed actions, HTTP executor, JSON API, strict CSP/security evidence | YT-014–YT-016, YT-018 | Clear local/remote boundary, correct API errors, working browser/CSP E2E |
| **M5 — MVP validation/docs** | Cross-target regression, browser smoke, baseline measurements, examples | YT-019–YT-020 | Evidence for [A01–A10](MVP.md#3-acceptance-matrix) and public limitations |
| **M6 — Handler-first API and optional extensions (locally complete)** | App/Page APIs, explicit module functions, independent Request ID proof and migrated Todo | YT-030–YT-034 | Native/Chrome/Firefox contracts PASS; release assets unchanged; measured latency comparison, no mandatory MVC or DI |
| **M7 — Bounded keyed DOM and local reactivity (locally complete)** | Measured strategies, promoted low-level contracts, production Todo keyed graph and final decision freeze | YT-025, YT-035–YT-043 | Equivalent output, ownership/disposal, cross-browser lifecycle, measured bundle cost, Search isolation, complete regression and repeat reviewer PASS |
| **M8 — Sample-driven application experience** | Standardized sample contract, API-only REST app, opt-in Session cart, DX audit and evidence freeze | YT-044–YT-048 | Copyable applications, pay-for-what-you-use dependencies, security/lifecycle evidence and per-stage reviewer PASS |
| **M9 — Sample expansion and API contract qualification (locally complete)** | Nine canonical sample names plus three supplements, SQL-first persistence, contract classification | YT-049–YT-059 | Twelve samples, 170-export inventory, complete regression and repeat final reviewer PASS |
| **M10 — Release engineering and production-security qualification** | Licensing, governance, compatibility, threat model, supply-chain and operational qualification | YT-060–YT-067 | Maintainer decisions, hosted controls, reproducible release evidence and internal Codex-assisted security assessment |
| **M11 — Done at selected product and Experimental renderer scopes** | Independent complete-product and browser-first opt-in renderer lines | YT-069–YT-081 | Reviewed plan; later separately authorized product/renderer evidence, isolation and independent implementation review; no release activation |
| **M12 — Done at bounded migration scope** | Rust-native active developer build/check/smoke/qualification/release tooling, isolated from framework dependencies | YT-082–YT-086 | Exact 52-file/three-workflow parity, preserved suite gates, actual Linux/Windows owned-child evidence, Python-free transitive command graph and separate independent review per item; no public CLI or release activation |
| **M13 — Done at bounded Linux CLI preview scope** | Unpublished Rust `yetlet` binary, minimal handler-first scaffold and local doctor/build/dev | YT-087 | [CLI contract](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-087-CLI-PLAN.md), fail-closed templates and processes, actual local HTTP/PTY evidence and final independent Reviewer PASS; no registry publication, Windows qualification or release activation |
| **M14 — Done at bounded Linux/Windows CLI preview scope** | Two internal OS adapters for filesystem-safe scaffolding and foreground process ownership | YT-088 | [CLI platform qualification limits](CLI-GUIDE.md#native-cli-platform-and-generated-deployment-boundaries), real GNU/MSVC positive new/build/dev HTTP/console/cleanup and per-slice independent reviews; no public release |
| **M15 — Done at bounded Linux OCI/Kubernetes configuration-generator preview scope** | Opt-in CLI emission of Dockerfile, Compose and portable Kubernetes manifests without platform SDK coupling | YT-089 | [Generator plan](CLI-GUIDE.md#native-cli-platform-and-generated-deployment-boundaries), source-path-aware image build, app-owned bind/health/SIGTERM, offline generation and per-slice independent verification; no image publication or cluster deployment |
| **M16 — Done at bounded YT-090 / YT-097 scope** | Opt-in typed native configuration and optional CLI scaffolding | YT-090, YT-097 | [Guide](CONFIGURATION.md), [focused Linux evidence and independent Reviewer PASS](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-090-VERIFICATION.md); [CLI integration Linux/Windows GNU/MSVC and independent Reviewer PASS](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-097-VERIFICATION.md); no default/core/browser or release change |
| **M17 — Done at bounded YT-091 Tailwind scope** | Opt-in build-time Tailwind CSS 4 application scaffolding | YT-091 | [Guide](CLI-GUIDE.md#opt-in-tailwind-css-4-yt-091-independently-reviewed), [evidence](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-091-VERIFICATION.md); actual Linux HTTP/CSS/CSP/signals and native Windows GNU/MSVC Tailwind source-bound builds PASS; independent Reviewer `wc_job_T7Blu0CYQUOmaQTL` PASS, no core/browser change or release activation |
| **M18 — Done at bounded YT-098–YT-101 scopes** | Optional asset contract, Vite production/dev profiles and cloud delivery | YT-098–YT-101 | [Bounded Linux Vite spike](QUALIFICATION-MATRIX.md#browser-and-vite-feasibility-evidence) passed build/native/browser checks; [YT-098 application-owned producer proof](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-098-VERIFICATION.md) passed 11 native tests, four local Chrome checks, cleanup and independent Reviewer `/root/yt098_browser_closeout_review` (zero issues/followups). [YT-099 production CLI integration](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-099-VERIFICATION.md) Done at bounded experimental scope: Linux/Windows GNU/MSVC, local Chrome JS on/off, cleanup and independent Reviewer `/root/yt099_local_browser_acceptance_review` PASS; [YT-100 opt-in dev bridge](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-100-VERIFICATION.md) Done: pinned Vite8 env P1 fixed; native Windows GNU/MSVC source-bound tests, Linux Plain/Tailwind Chrome 11/11 HMR/CSP/WS, actual dual-terminal Ctrl-C/restart/503/cleanup PASS; independent Reviewer `wc_job_MprRYhxeijOu6xG5` PASS (zero issues/followups). Windows live HMR remains unverified; [YT-101 cloud delivery](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-101-VERIFICATION.md) Done: real Linux Docker/Compose/K3s Pod/Service/asset PASS, Windows MSVC native/source PASS, subsequent Windows GNU 726/726 source and native tests/generator PASS after official MinGW64 GCC/Binutils installation, independent Reviewer `wc_job_fYtD9F8KXkXyzvYQ` PASS; no stable shared asset API, default/core/browser or release change |
| **M19 — Production-oriented application and developer experience (partial)** | Production Starter, signing, Admin Dashboard, DI/DX research, browser-only starter and optional HA/rolling deployment asset compatibility | YT-092–YT-095, YT-102–YT-112 | Real native/browser/reference-app proof, independently reviewed DX research, a separately qualified browser-only static workflow, and independently validated multi-version asset-retention/rolling-availability evidence; no automatic production support or release claim |
| **M20 — Release-candidate qualification in progress** | Candidate API/security/supply-chain validation before explicit public-release decision | YT-096, YT-020 | [Current selected source/tag, hosted build/verifier and owner-host retention](RELEASE-READINESS.md#historical-release-and-hosted-source-checkpoints) PASS at bounded unsigned sixteen-package scope; D6 identity/signing and remaining trust/registry/final-release gates stay open. Historical original-nine qualification and failure dispositions follow: [Selected `ce56e08` candidate](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence): original nine experimental 0.1.0 libraries and Rust 1.99; local source-package/clean-consumer preflight passes on Ubuntu 26.04.1. [Selected dependency/API preflight](RELEASE-SUPPLY-CHAIN.md#historical-candidate-inventory-and-technical-consolidation) adds 23 scoped SBOM/license checks, nine passing RustSec scans and explicit coverage limits. [Partial platform execution](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) adds Ubuntu 24.04.5 container userspace native/doctest PASS, five wasm builds and fresh Linux Firefox PASS; resumed Windows MSVC standalone-package execution passes 83 native tests/four doctests, with historical cache failures and shared-kernel/Chrome scope retained. [Two-host archive rebuild](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-096-TWO-HOST-REBUILD.md) matches all nine Windows archives byte-for-byte to Linux with normal Cargo verification. [Azure Run Command continuation](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) passes actual Ubuntu 24.04.5 guest execution, empty-cache nine-archive rebuild, 83 native tests/four doctests and five wasm libraries; earlier SSH failures preserved and resources deleted. [GitHub-hosted candidate rebuild](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) passes at the exact candidate with nine byte-identical archives and matched source/lock/tools, adding separately operated execution evidence. [Final documentary reconciliation](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence) joins nine archives to 23 SBOMs and retained compiler/platform/browser evidence without runtime reruns. The [owner accepted the bounded support scope and known inventory differences](QUALIFICATION-MATRIX.md#historical-nine-package-candidate-platform-and-builder-evidence), keeping exact-candidate Chrome and complete compiled/per-library/system inventory Not verified; [Final independent candidate review](RELEASE-READINESS.md#historical-release-and-hosted-source-checkpoints) completed with technical/security evidence accepted but overall release gate FAIL, release-ready NO. Only support/inventory disposition is resolved; [minimum formal trust research](https://github.com/bohewu/yetlet/blob/66de5b5794ee9587dfbfe5e5c629c572e1af7b8a/docs/YT-096-FORMAL-TRUST-PROPOSAL.md#historical-nine-package-proposal--2026-10-20) proposes conditional GitHub OIDC/public Sigstore and exact source/evidence bindings without implementation or policy selection. The [metadata decision brief](https://github.com/bohewu/yetlet/blob/66de5b5794ee9587dfbfe5e5c629c572e1af7b8a/docs/YT-096-TRUST-DISCLOSURE-DECISIONS.md#historical-decision-brief--2026-10-23) records public exposure/impacts and pending owner choices while retaining the original candidate and evidence without reruns or signing. Formal integrity/trust, registry proof, public activation controls and concrete final maintainer approval remain open; YT-020 stays Blocked, no auto-merge/release |
| **M21 — Opt-in server capability roadmap (registered)** | Separate optional Authentication/Authorization, Session integration, rate limiting and conditional provider studies without bloating core | YT-113–YT-123 | [Staged capability plan](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection): YT-113 research and YT-114 native foundation Done after separate independent Reviewer PASS; YT-115 Done at Memory bridge scope and YT-116/119/120 Done at independently reviewed study scopes; [YT-117 rate limiting](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection) Done at per-process scope after independent Reviewer PASS; YT-118/121/122 research Done after independent Reviewer PASS and [YT-123 research](https://github.com/bohewu/yetlet/blob/4ad0978f8a70b36c307f5e538aa1a3aa496a7403/docs/YT-123-TRACING-CORRELATION-STUDY.md) Done after independent Reviewer PASS; design gates and independent review before implementation; no M20 release-gate dependency or production claim |
| **M22 — Production Readiness Phase 2 (bounded local scope complete)** | Sequential app-owned admission, opt-in observability, bounded capacity/recovery and upgrade compatibility | YT-127–YT-131 | [Contracts, baseline gaps and final matrix](PRODUCTION-READINESS-PHASE2.md): YT-127 Done at bounded A1 scope after independent Reviewer PASS; YT-128 Done after [local verification and independent Reviewer PASS](https://github.com/bohewu/yetlet/blob/be337ff02826978a9baa1b88ddf85696cb4406b4/docs/YT-128-OBSERVABILITY.md); YT-129 [local load/recovery evidence](QUALIFICATION-MATRIX.md#historical-operations-capacity-and-recovery-cohorts) Done after independent Reviewer PASS; YT-130 [local restore/runbook evidence](https://github.com/bohewu/yetlet/blob/be337ff02826978a9baa1b88ddf85696cb4406b4/docs/YT-130-BACKUP-RESTORE.md) Done after independent Reviewer PASS; YT-131 [v1-to-v2 implementation and local evidence](https://github.com/bohewu/yetlet/blob/be337ff02826978a9baa1b88ddf85696cb4406b4/docs/YT-131-MIXED-VERSION-COMPATIBILITY.md) Done after independent Reviewer PASS and controller acceptance; A1 → A2 → B1 → B2 → C1 with source-bound evidence and cleanup. Keep Implemented, Verified and Deployment-qualified separate; no generic infrastructure, M20 candidate expansion or production/release claim |

**M19 form/DOM authoring (YT-107–109):** Done at their originally
bounded application feedback, narrow `hidden` attribute, and
DOM-capability research scopes. Current [View/form usage](VIEW-COMPONENTS.md#configuration-and-errors)
and [bounded DOM contract](BROWSER-APP.md#historical-native-dom-controls-and-navigation-boundaries) retain actual
rendering/browser ownership and strict attribute/escaping policies.
These do not create general form binding, a remote event circuit or
a newly qualified package release. Original source, browser failures,
reviewer decisions and source identity remain in local Git
`b610acc7e098020b6b59dfa0d1ab2b75b059e656` without public-link reachability claims.

YT-110 is Done at the separately owner-selected [bounded DOM capability scope](BROWSER-APP.md#historical-native-dom-controls-and-navigation-boundaries).
Common View checks, finite semantic/accessibility vocabulary, native control
initialization and constrained navigation have native/WASM and actual browser
proof with cleanup. Independent Reviewer role-directed Job wc_job_v5BMe9GdWQmvY03d
PASS has zero issues or required followups. The earlier executor dispatch rejection
and current-session implementation remain explicit. No forms runtime, keyed/reactive
ownership rewrite or release-candidate requalification is implied.

**YT-111–112 developer workflow completion:** Done at separately reviewed
read-only CLI `inspect` project/Loader scope and selected Linux browser-only
Rust edit/build/reload scope. `doctor` remains environment-only; Vite owns
TS/CSS HMR, while Rust/Wasm changes use serialized source-hashed builds
and an application-owned full reload, **not** arbitrary persistent Rust state.
The original YT-112 Reviewer FAIL on module routing/runtime evidence was
corrected and accepted on source-matched Linux; new Windows browser-watch
execution and current release qualification were not implied. Current
[CLI guide](CLI-GUIDE.md#historical-cli-and-developer-loop-qualification)
and [Browser app](BROWSER-APP.md) own usage; full original tests and
reviewers remain at fixed local Git `ed7a082b1f9f72430f06866f7d550b0ed9ecbae9`.
## M19–M20 — Product readiness and release qualification (planned)

The owner restored five previously discussed but never registered work items.
The original registration marked these items **Todo**; subsequent bounded
closeouts below record implementation and review evidence. Their scopes are defined in [M19 and M20 Backlog](BACKLOG.md#m19--production-oriented-application-experience-planned).

**M19, application experience:** YT-092 is an opt-in Production Starter
reference after YT-098, with typed configuration, developer secrets, health,
tracing, graceful shutdown, truthful failure modes and operational evidence;
it does not wait for Vite HMR. YT-093 first defines an opt-in
[signing/provider contract](SECRET-PROVIDERS.md#local-signing-and-replaceable-provider-contract--yt-093), then exercises local
PFX/PEM signing with established cryptographic libraries, multiple named keys
and confirmed CLI rotation with automatic selected-profile settings updates.
Azure Key Vault/OpenBao
remain future separately opt-in providers, with no present SDK/service dependency
or mandatory framework key management. YT-094 exercises a full responsive,
accessible Tailwind/Vite Admin Dashboard after YT-092/YT-099, using actual
persisted application/auth boundaries rather than synthetic-only production
claims. YT-095 is a smaller research comparison of native State/FromRef/traits
against optional Rust DI containers, after real YT-092/YT-093 consumers exist.
No mandatory IoC container follows from this research.

**YT-094 bounded closeout (2026-10-19):** Done at the separately
accepted Admin reference-application scope with native SSR, local Rust/WASM
triage, source-matched Windows input, OIDC, Valkey Sessions and shared SQLite
evidence. The [current app guide](../examples/admin-dashboard/README.md)
owns current behavior; the exact closeout matrix, prior Reviewer failures
and original source qualification are recoverable at fixed local Git
`4ad0978f8a70b36c307f5e538aa1a3aa496a7403`. Production TLS, distributed durability, HA, unqualified
accessibility and public-release claims remain excluded. Earlier staged
"In progress" statements refer to their dated snapshots, not present status.
**YT-094 first slice (2026-10-19, historical):** the
[SSR administration reference](../examples/admin-dashboard/README.md) starts from
the existing CLI's Vite/Tailwind/settings scaffold and reuses the persisted Lab
Operations Board through explicit application composition. The selected CSS-only
Vite build covers responsive list/filter/pagination, native forms, detail/history
and consistent failure states. Local Linux Chrome and CLI evidence is recorded in
the application guide; independent Reviewer `/root/yt094_slice_review` PASS has
zero material issues or required followups. The later
[Windows native slice](QUALIFICATION-MATRIX.md#historical-windows-native-browser-and-cli-evidence) passes source-matched MSVC
tests, CLI lifecycle and HTTP, with independent Reviewer PASS and zero issues or
required followups; browser/OS input remain open. YT-094 remains In progress:
OS-level IME/accessibility qualification, wider native/browser coverage and
production authentication/deployment gates are not claimed by this first slice.

**YT-094 Rust/WASM slice (2026-10-19):** the
[triage workspace](https://github.com/bohewu/yetlet/blob/4ad0978f8a70b36c307f5e538aa1a3aa496a7403/docs/YT-094-WASM-TRIAGE.md) exercises per-item drafts, local
filtering/keyed reorder and explicit authenticated HTTP saves with the existing
Rust graph/DOM/action APIs. It retains a no-JS native workflow and confines WASM
CSP to the selected page. Linux checks are separate from the earlier Windows
native source. [Observed authoring gaps](https://github.com/bohewu/yetlet/blob/4ad0978f8a70b36c307f5e538aa1a3aa496a7403/docs/YT-094-API-OBSERVATIONS.md) seed the
bounded YT-105 comparison; packaging evidence feeds existing YT-104. No framework
redesign or additional platform qualification is implied. Independent Reviewer
`/root/yt094_wasm_review` follow-up PASS follows one corrected queued-save
lifetime finding; zero remaining material issues or required followups.

**YT-094 returned Windows evidence (2026-10-19, historical):** the
[Windows follow-up](QUALIFICATION-MATRIX.md#historical-windows-native-browser-and-cli-evidence) retains the passed
native/browser matrix and now reconciles operator-passed Bopomofo, draft/focus,
Narrator, save/reload and physical-stop journeys. Images corroborate drain and
exit 0; returned observations confirm no server/listener or running input/AT
tools. All 14 returned evidence files match their supplied hashes, and all 310
selected source inputs match `a402b6f`. The Windows review's historical fail for
the then-missing exit code and its configured/observed-model mismatch remain
explicit. No workload or product fix was repeated. At that stage YT-094 remained
In progress; separate source-side reconciliation Reviewer PASS has zero
issues/followups. Production authentication/deployment and other unqualified scopes require an
explicitly selected next slice rather than reopening passed native journeys.

**YT-094 opt-in OIDC slice (2026-10-19):** the
[application-owned integration](../examples/admin-dashboard/README.md#historical-auth-session-oidc-and-local-policy-evidence) uses `openidconnect`, existing
settings/SecretProvider composition, native forms, PKCE and explicit verified
subject-to-permission mapping. Focused Linux tests and strict Clippy pass;
independent code review PASS has zero issues or required code followups.
After the operator WAF adjustment, real token exchange, unassigned/replay denial,
explicit Project 1 access, cross-project denial, native persisted edits/history
and local logout passed with JavaScript disabled. CLI build/run/Ctrl-C and
cleanup passed at the documented Linux cancellation exit 1; independent
returned-evidence Reviewer PASS has zero issues or required followups. Private client,
account and browser evidence stays outside the working tree. Earlier Windows
acceptance is preserved for its original source. Separate Windows OIDC testing
was explicitly waived by the owner; the dedicated client/account are deleted.
Routine IME/assistive-technology rechecks are not selected; focused Ubuntu checks
may accompany relevant input/accessibility changes.

**YT-094 opt-in Valkey shared sessions (2026-10-19):** the
[application slice](PROVIDER-OPERATIONS.md#redis-session-cas-operation-outcome-and-cookie-ordering) keeps the existing Memory default
and reuses the optional async provider through settings. Two actual Compose
replicas on pinned Valkey 9.1.2 passed shared login/logout, app restart,
bounded fail-closed pause/outage and recovery. Native tests and strict Clippy
pass; independent Reviewer PASS has zero issues or required followups. Owned
containers/volumes/listeners were cleaned up. Valkey has no persistence in this lab;
SQLite business data remains replica-local in that initial fixture. The later
[shared-SQLite slice](PROVIDER-OPERATIONS.md#local-auth-permissions-and-multi-instance-authority) separately passes two-app business
reads/updates, conflicts, restart and cleanup on one local volume, with independent
Reviewer PASS. No core/browser dependency, IdP
resource, registry, multi-host or production HA claim is added.

**YT-104 browser-only starter (2026-10-19):** Done at finite CLI-created
Rust/Wasm task-list and static-host scope: separately selected Linux
Chrome/desktop/mobile and native Windows GNU create/build/run/cleanup
cases were accepted after independent review. No native router/backend,
universal HMR, production hosting or release was selected. The
[current browser app guide](BROWSER-APP.md) owns supported commands;
the original 2026-10-19 evidence remains at fixed local Git `ed7a082b1f9f72430f06866f7d550b0ed9ecbae9`.
**YT-105 form-binding comparison (2026-10-19, research only):** the first
[Admin/Todo study](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-105-FORM-BINDING-RESEARCH.md) measures control/value/event
glue and recommends documenting native action-time drafts versus graph-owned
drafts over existing `web-sys` and listener ownership. No public form-binding helper or
runtime change is selected. HTTP feedback is compared in the third study below.
Independent Reviewer
`/root/yt105_form_binding_research_review_oct19` PASS has zero issues or required
followups for this research slice.

**YT-105 state-read/initial-render comparison (2026-10-19, research only):**
the [second Admin/Todo study](ARCHITECTURE.md#weak-handle-published-state-snapshot-and-explicit-first-render)
identifies duplicated committed-state mirrors and proposes a weak
`GraphHandle::snapshot()` matching existing owner clone/error semantics. Initial
rendering remains explicit application setup rather than constructor side
effects. That research did not authorize implementation; the subsequent
separately selected snapshot slice is recorded below. Independent Reviewer
`/root/yt105_state_read_review_oct19` PASS has zero issues or required followups
for this research slice.

**YT-105 HTTP-feedback comparison (2026-10-19, research only):** the
[third Admin/Todo study](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-105-HTTP-FEEDBACK-RESEARCH.md) finds the existing
body-token CSRF sufficient for these callers and traces field-detail loss in
both the Admin handler and common executor. It recommends a private Admin HTTP
adapter/payload trial and Todo code mapping before changing public error types;
custom ActionExecutor implementations still have the same fixed ActionError.
All research topics are covered. Independent Reviewer
`/root/yt105_http_feedback_review_oct19` PASS has zero issues or required
followups and explicitly approves YT-105 Done at research-only scope.
No HTTP-feedback application/API implementation is selected by that study.

**YT-105 weak-handle snapshot (2026-10-19, separately authorized):** the
[implementation slice](ARCHITECTURE.md#weak-handle-published-state-snapshot-and-explicit-first-render) adds `GraphHandle::snapshot()`
using the owner's existing read/error/borrow semantics. Focused native tests
and strict Clippy plus Wasm/rustdoc/docs checks pass. Independent Reviewer
`/root/yt105_graph_snapshot_impl_review_oct19` PASS has zero issues or required
followups; this separately selected minimal implementation is complete.
That original slice left consumer mirrors and initial-render/HTTP/form behavior
unchanged.

**YT-105 Admin/Todo adoption (2026-10-20, separately selected):** the same
[snapshot record](https://github.com/bohewu/yetlet/blob/c3478b7f4e0c929a94ccbf21e05217535553a73d/docs/YT-105-GRAPH-SNAPSHOT.md#admin--todo-adoption--2026-10-20)
now documents both consumers reading through weak handles without separate
state mirrors. Wasm/native/strict Clippy and actual Chrome checks pass, including
Admin CLI build/run/SIGINT drain and Todo's existing lifecycle smoke. Consumer
Reviewer `/root/yt105_consumer_snapshot_review_oct20` PASS has zero issues or
required followups; this bounded adoption is complete. No profiling/speedup
claim or initial-render/HTTP/form change is selected.

**YT-106 — Third-party component interoperability:** The independent
research selected app-owned Rust/TS/Wasm component instances rather than
a mandatory browser framework. The subsequent independently reviewed
Admin Chart.js loaded-item bars and CodeMirror plain-text editor use
per-page CSP style-nonce scoping, explicit owners/disposal, non-script
fallback and existing CLI/Vite assets. Browser evidence is bounded to
the source-matched Chrome/TypeScript trials; unapproved inline styles
remain blocked. Current [Admin sample](../examples/admin-dashboard/README.md)
owns the usage and lifecycle; original tests, CSP failures and reviewers
remain at local Git `983653adadcedb4ec196016b75281d39e2ed0acf`.
No editor/interop Core API, unsafe-inline policy, browser-wide assurance
or automatic framework integration is selected.
**YT-095 dependency composition (2026-10-20, research only):** the
[current-consumer comparison](ARCHITECTURE.md#research-only-handler-resumption-and-native-composition) retains native
constructors and Axum State/FromRef after inspecting Starter, signing, Admin
and existing settings/secret examples. It compares shaku/dill without installing
them, records static authoring costs and unmeasured build costs, and preserves
generic signing, finite session selection and explicit resource lifetimes.
No framework/container API, ADR change or application refactor is selected.
Independent Reviewer `/root/yt095_composition_research_review_oct20` PASS has
zero issues or required followups; YT-095 is Done at research-only scope.

**YT-093 bounded closeout (2026-10-18):** the
[signing verification](QUALIFICATION-MATRIX.md#local-signing-key-cli-and-azure-evidence--2026-10-18) records the opt-in signing
crate, local multi-key CLI, confirmed automatic settings updates and independent
Reviewer PASS. Linux and source-matched Windows native signing/key operations
pass; actual Linux CLI sample build/run and terminal confirmation pass. The
separate Azure CLI experiment verifies three remote signatures locally and
removes active lab resources, with disclosed soft-delete retention. The original
Windows exit-125 evidence remains retained; the separately reviewed
[CLI follow-up](QUALIFICATION-MATRIX.md#windows-exit-125-diagnosis-and-native-results) identifies the Cargo ACL refusal
and passes build/run/Ctrl-C cleanup with protected task-local tools. No cloud adapter, HTTPS feature or production
security/release qualification follows from this bounded Done status.

**YT-092 bounded closeout (2026-10-18):** the independent
[Issue Tracker starter](../examples/production-starter/README.md) exercises
SSR forms, a command JSON API and explicit SQLite persistence using existing
settings/CLI and optional modules. [Qualification](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-092-STARTER-VERIFICATION.md)
records Linux 15/15 and native Windows GNU 16/16 tests, actual HTTP, persistence
across process restart, native console Ctrl-C, frozen CLI build/run, strict
Linux lint/format/docs checks and independent Reviewer `/root/yt092_review` PASS
with zero issues/followups. YT-092 is Done only at this reference-app scope;
public hosting/auth/HA, broader browser qualification and generator source
contract changes remain outside this closeout.

**YT-102, research-only Developer Experience:** following YT-101, study the
real application authoring and feedback loop: opt-in unified Rust/Vite dev
orchestration, Rust watch/rebuild/restart, failure reporting, diagnostics,
scaffold/onboarding, typed configuration and development secrets, testing,
preview/deployment ergonomics, and Linux/Windows parity. Compare reuse of
existing tools with minimal Yetlet-specific integration; keep a separate
frontend-independent and two-terminal fallback. Produce an evidence-based,
prioritized DX research report and candidate future backlog slices with
independent Reviewer PASS. Research does **not** implement a unified dev command,
make Node/Vite mandatory, authorize process automation, or alter production
runtime/CSP, release or deployment.

**YT-102 CLI implementation, developer workflow and diagnostics:**
Accepted original 2026-10-18 contract and authorizations introduced
short settings-first CLI grammar, environment-scoped doctor, source-constrained
scaffolds/generators, Vite ownership and selected native process supervision.
The separately reviewed DX audit retained Cargo, Watchexec and Vite and
documented first-use setup rather than adding a general new runtime.
The selected dotenv symlink escape was subsequently corrected; the
historical [CLI source audit](QUALIFICATION-MATRIX.md#historical-cli-contract-process-and-sdk-resolver-evidence)
retains its original F-01 finding and cannot certify later bytes.

A further independently reviewed [Linux completion-latency improvement](QUALIFICATION-MATRIX.md#historical-cli-contract-process-and-sdk-resolver-evidence)
skips the fixed 500ms grace only after safe WNOWAIT, single-thread and
retained-child checks; failure/cancellation preserves the full cleanup.
Selected native Windows GNU/MSVC build, signal/process/console and
[Vite pair evidence](QUALIFICATION-MATRIX.md#historical-windows-native-browser-and-cli-evidence) remains
host/source-specific, not a general browser/production test.
Source-selected project inspection and Rust Watch/Vite HMR now appear
in the [current CLI guide](CLI-GUIDE.md#historical-cli-and-developer-loop-qualification).
Original implementation logs, failed attempts and verification are
recoverable from fixed local Git `ed7a082b1f9f72430f06866f7d550b0ed9ecbae9`; no fresh product tests
were performed for documentation consolidation.
**YT-103 opt-in rolling asset retention (2026-10-18):** Accepted at one
bounded local deployment laboratory, not as a new Yetlet Core/CLI generator
or production HA guarantee. Independently built v1/v2 sources and a static
asset union retained old-tab lazy JS/CSS through update and rollback.
All 2,364 measured SSR requests and one held eight-second request
completed through the selected K3s rollout/SIGTERM sequence; source
checks, fail-closed missing/private assets and owned cleanup were recorded.
Node/host/zone failure availability, CDN longevity and general rolling
deployment remained unverified. See [asset policy](ASSETS.md#cache-and-retention)
and retained [original qualification](ASSETS.md#historical-single-host-rolling-asset-qualification).
**YT-104, opt-in browser-only application workflow:** complete the delivery path
around the existing Rust/WASM DOM and local-state capabilities. First define a
bounded sample against the current supported DOM subset, then integrate CLI
creation, run/stop and build through the existing settings-first conventions.
Prefer existing WASM/Vite packaging tools where appropriate. The result must
run from HTML/JS/WASM/static assets on a local static host, without a Yetlet
native server or backend-generated shell; ordinary local interactions require
no API. Remote work remains a separately selected explicit API boundary. Browser
configuration is public and separate from native settings/secrets. Document
JavaScript/WASM requirements, build prerequisites, asset paths and navigation
fallback choices. Require source-matched Linux/Windows CLI lifecycle, actual
browser mount/state, MIME/CSP and cleanup evidence plus independent review.
This item is Todo and separate from YT-094's SSR-first Admin reference. Its
registration does not select a general SPA/router, CLI flag, cloud deployment,
publication or production support, and does not change SSR/island defaults or
automatically gate YT-096. Implementation requires a separate owner selection.

**M20, release:** YT-096 selects an immutable release candidate only with
maintainer approval and gathers API/MSRV, security, cross-platform, supply-chain,
signed provenance and realistic application qualification after YT-092/YT-094/
YT-101. It must not broaden a release candidate to YT-093/095 experiments
without an explicit support decision. The separately blocked YT-020 remains
the publication and registry/trust activation gate. A reviewed example or green
CI does not authorize publication, visibility changes, auto-merge or deployment.

**Dependency-aware scheduling:** proceed with YT-098 → YT-099 for the asset
infrastructure; YT-092 may begin when YT-098 passes, without waiting for
YT-100 (HMR). YT-093 can proceed independently from typed configuration.
YT-094 requires the starter and production Vite contract, and YT-096 is the
last explicit candidate-qualification step. YT-101 is complete at its bounded
M18 scope. YT-102 DX research and the independent YT-103 deployment/asset
compatibility item can be scheduled separately once their dependencies are met;
neither automatically becomes a YT-096 release prerequisite unless the owner
selects the capability for a supported candidate. M18 and M19 are not strictly
sequential global barriers.

## M21 — Opt-in HTTP/server capabilities (registered)

The [YT-113–YT-123 batch plan](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection) is an
owner-approved **roadmap only**, separate from the exact-candidate M20 release
qualification. [ADR 0005](adr/0005-opt-in-capability-modules.md) already defines
optional module composition: no default Session/Auth/Cookie/Cache/Trace provider,
no fixed request-context identity field and no provider registry or DI container.

**Primary sequence:** YT-113 reviews the Authentication/Authorization contract
using existing Auth Notes, Admin OIDC and the Session provider; YT-114 introduces
a narrow, server-only `yetlet-auth` principal/extractor/policy seam after contract
selection; YT-115 separately integrates protected Session/Cookie identity;
YT-116 evaluates conditional OIDC reuse; YT-117 selects then validates an
opt-in scoped single-process rate limiter. Each needs its own owner-selected
implementation or research gate and independent review.

**M21 conditional studies and accepted status:** All eleven original
YT-113–123 items closed at individually reviewed scopes. YT-113/114/115
produced the documented Auth contract, native foundation and Memory bridge;
YT-117 produced bounded default-off **per-process** rate limiting.
YT-116 OIDC, YT-118 Audit, YT-119 opaque Bearer, YT-120 Cookie primitives,
YT-121 data Cache, YT-122 private Output Cache and YT-123 Tracing were
**research-only Done**. They selected reusing native Axum/Tower and
application-owned policy, not seven provider crates or a global plugin
registry. The [capability map](CAPABILITIES.md#research-decisions-and-unselected-candidates)
and [extensions guide](EXTENSIONS.md) own the current decisions;
original independent reviews remain in fixed local Git `4ad0978f8a70b36c307f5e538aa1a3aa496a7403`.
No WAF control plane, PostgreSQL schema, proxy, automatic cache of private
data or production security assurance is introduced by these studies.
The separately selected [YT-117 contract](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection) fixes endpoint/group
composition, reusable settings versus shared in-memory accounting and default-off
startup installation before minimal implementation. The selected implementation
passed 17 focused tests, one doctest, strict Clippy/rustdoc/fmt/docs and independent
Reviewer `/root/yt117_rate_limit_independent_review_20261022` with zero issues or
required followups. The initial five baseline normal dependency graphs were byte-identical. The
separately selected [Settings-example follow-up](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection)
uses the existing opt-in loader; its shared-lock libc pin and fresh focused
evidence are recorded separately from that original acceptance.
No benchmark, hot reload or distributed quota is selected.
The [YT-114 foundation](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection) passed 15 integration tests, one compiled
example, focused native/graph/WASM-target checks and independent Reviewer
`/root/yt114_foundation_review_20261022` PASS with zero issues/required followups;
the separately authorized [YT-115 Memory bridge](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection) is now
Done after independent Reviewer PASS. YT-116/119/120 research passed independent
Reviewer `/root/auth_studies_review_20261022` with zero issues and required
followups for each item. The [Auth group Standard security scan](EXTENSIONS.md#historical-opt-in-auth-session-and-rate-limiter-selection)
completed with all six Auth-crate files reviewed and zero reportable findings;
this is scoped offline review, not whole-repository or production qualification.
No OIDC/Bearer adapter is selected.
The [authentication contract study](PROVIDER-OPERATIONS.md#local-auth-permissions-and-multi-instance-authority) passed independent
Reviewer `/root/yt113_contract_review_20261022` with zero issues/required followups;
YT-114 and YT-115 were separately authorized. Accepted
YT-066, YT-094, YT-111/112 and YT-096/020 statuses remain unchanged.

The owner separately selected [YT-124 multi-instance Auth/Valkey Session research](PROVIDER-OPERATIONS.md#local-auth-permissions-and-multi-instance-authority).
It is Done at research scope after independent Reviewer PASS and does not change the
original eleven-item registration counts above. Valkey is already the Admin
reference's shared-session server; YT-066b3 retains its historical Redis scope.
The study defines async Auth integration, cross-instance lifecycle, current
permissions and failure/commit semantics while reusing YT-094 and YT-103 evidence.
Implementation remains subject to a later owner decision; no runtime or
deployment execution is selected.

After YT-124 PASS, the owner selected [YT-125 minimal Valkey/Auth integration](../examples/admin-dashboard/README.md#historical-auth-session-oidc-and-local-policy-evidence).
It is Done after independent Reviewer PASS at the standalone application-owned
resolver scope, with fresh shared local policy and actual two-process Valkey
qualification. The focused HTTP test, two unit tests and strict lint/format
checks pass, four baseline normal graphs remain unchanged and owned cleanup
is verified. Business-data mutation,
Admin migration, generic helper promotion and production/HA claims are outside
this slice. The record preserves actual scope, commands and independent review.

The owner then selected [YT-126 opt-in Admin adoption](../examples/admin-dashboard/README.md#historical-auth-session-oidc-and-local-policy-evidence).
This application-owned resolver/extractor and fresh shared-policy integration is
Done after focused new two-Admin-process evidence and independent Reviewer
PASS with zero issues/followups. Defaults, browser graphs, OIDC verification and existing
Session-provider APIs remain separate. The existing pre-SQL race is disclosed;
strict cross-store business-commit ordering remains an unselected later gate.

## M18 — Asset contract and Vite integration (Done at bounded scopes)

The [Vite feasibility spike](QUALIFICATION-MATRIX.md#browser-and-vite-feasibility-evidence) and
[isolated experiment](../experiments/vite-integration/README.md) demonstrate
Rust-owned SSR with actual plain Vite and Vite/Tailwind production assets under
strict CSP. This is not a formal CLI or framework integration. YT-098 defines
an optional asset contract first; YT-099 then adds production Vite scaffolding.
YT-100 separately addresses HMR security/process ownership, and YT-101 addresses
cloud asset/config delivery. YT-098 is **Done** at its application-owned
experimental scope with two actual producers using a
[neutral contract](ASSETS.md). All 11 native tests and four local Chrome checks
passed, including real computed CSS, JS-disabled SSR, lazy stylesheet order,
strict CSP and owned-process cleanup. Independent native Reviewer
`/root/yt098_browser_closeout_review` returned PASS with zero issues/followups;
the earlier evidence-only failure remains in the verification history.
YT-099 is also **Done** at bounded production-scaffold scope: native Linux and
Windows GNU/MSVC source-bound checks, real Plain Vite and Cloud+Config+Tailwind
Plugin Chrome JS on/off, strict CSP/MIME, lazy CSS order and process cleanup
passed. Independent native Reviewer `/root/yt099_local_browser_acceptance_review`
returned PASS with zero issues/followups, closing its prior missing-browser
evidence gate. YT-100 is now **Done** at its isolated opt-in dev bridge scope:
real Plain and Tailwind Chrome HMR/CSP/WS plus two actual foreground
Ctrl-C/restart/cleanup paths PASS, and independent Reviewer
`wc_job_MprRYhxeijOu6xG5` passed with zero issues/required followups.
YT-101 is **Done at bounded opt-in cloud-delivery scope**:
[implementation and real runtime evidence](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-101-VERIFICATION.md) cover
plain/plugin Vite and optional typed config/config-only generators, absolute
runtime roots, sanitized three-context inventories, runtime-only token wiring
and 13/13 unchanged legacy outputs. Linux Docker Buildx/non-root runtime,
Compose and isolated three-node K3s Service/Pod/asset bytes PASS with full
owned cleanup. Windows MSVC 726/726 source-bound native tests and protected
generator outputs PASS; supplemental Windows GNU 1.99 native CLI tests,
Strict Clippy, Build, config-only app and Docker/Compose/K8s generators also
PASS with 726/726 source hashes after official toolchain installation. The
original GNU environment failure remains in the historical evidence.
Independent read-only Reviewer
`wc_job_fYtD9F8KXkXyzvYQ` returned PASS (0 issues/followups).
The accepted VITE-SPIKE
source and YT-091 standalone Tailwind templates remain unchanged. YT-020 remains Blocked; no CI/CD or
release activation follows.
The earlier experiment/planning Reviewer `wc_job_SuuEr3YetGlVbs_8` PASS
remains separate historical evidence for the Vite experiment and plan.

## M17 — Explicit build-time Tailwind CSS

YT-091 adds `yetlet new NAME --with-tailwind`, optionally combined with cloud
and typed configuration. Developers explicitly build root CSS with their
selected verified standalone Tailwind 4.3.3 executable before Cargo/run;
scaffolding, build/dev and requests never run or fetch a compiler. Literal Rust
classes feed a bounded startup CSS snapshot served at its SHA-256 URL with
strict CSP, CSS MIME, nosniff and immutable asset caching. Existing templates,
core/browser APIs and the twelve-sample catalog remain unchanged. All generated
source variants stay outside the deployment generators' accepted contract.
[Implementation evidence](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-091-VERIFICATION.md) records actual checks and
operational limits. Parent native Linux HTTP/CSP/assets and Windows GNU/MSVC generated-app
qualification passed on exact source. A separate independent read-only
Reviewer `wc_job_T7Blu0CYQUOmaQTL` returned PASS with zero issues/followups,
closing YT-091 at its bounded unpublished CLI scope. YT-097 remains Done;
YT-020 release remains Blocked.

## M16 — Typed application configuration

YT-090 implements an isolated unpublished native Serde configuration consumer
with finite profiles, explicit development secrets, known-key typed overrides,
safe diagnostics and app-owned validation before binding. The 19 focused tests,
format/strict Clippy, check-only examples and documentation/catalog checks pass.
Independent Reviewer `wc_job_6icPnMo4PotogYtw` returned PASS with zero issues or
required followups, completing this bounded YT-090 scope; Windows native runtime
and production qualification remain unverified.
YT-097 is implemented in the working tree for explicit
`yetlet new --with-config`, including `--cloud`, with passing local Linux
CLI/offline generated-app checks and unchanged no-config output. The parent
later verified actual Linux HTTP/CSP/health/SIGTERM/SIGINT and native Windows
GNU/MSVC generated-app/Cargo tests. A new independent read-only Reviewer
`wc_job_efQ_nYLzdZltPEjt` passed (zero issues and required followups), closing
YT-097 at this bounded unpublished opt-in CLI scope.
[Evidence](https://github.com/bohewu/yetlet/blob/b610acc7e098020b6b59dfa0d1ab2b75b059e656/docs/YT-097-VERIFICATION.md) preserves these limits. Existing
Docker/Compose/Kubernetes generators reject the separate configuration cloud app.
Tailwind YT-091 is now separately authorized under M17 above. Certificates,
external vaults, hot reload, formal release and CI/CD remain deferred;
YT-020 remains Blocked.

## M15 — Opt-in Docker, Compose and Kubernetes generators

**Done at bounded configuration-generator scope.** M15 delivered
opt-in Dockerfile/Compose/Kubernetes *configuration outputs* with
source confinement; Linux Docker/Compose and isolated K3s tests
plus native Windows GNU/MSVC generated-output parity passed separately.
These were not hosted production deployments or runtime-coupled
containers in Yetlet Core. See the [CLI generator limits](CLI-GUIDE.md#native-cli-platform-and-generated-deployment-boundaries),
[current CLI guide](CLI-GUIDE.md#native-cli-platform-and-generated-deployment-boundaries),
and [deployment profiles](PRODUCTION-DEPLOYMENT-PROFILES.md).

## M13–M14 — Developer CLI preview and platform parity

YT-087 initially delivered a **local Linux-only, unpublished CLI**.
Windows GNU/MSVC initially proved compile/fail-closed behavior, then
YT-088 separately qualified positive Windows `new/build/dev`, native
HTTP/CSP and Ctrl-C/Break/child cleanup. Each scoped review and original
failure remains recoverable in local Git `b610acc7e098020b6b59dfa0d1ab2b75b059e656`; neither
preview status nor qualified historical platform set should be projected
onto current release packages. The [CLI contract](CLI-CONTRACT.md),
[CLI platform limits](CLI-GUIDE.md#native-cli-platform-and-generated-deployment-boundaries) and
[current CLI guide](CLI-GUIDE.md) own current development and release distinctions.

## M12 — Repository-toolchain migration

YT-082–086 ported seven builder/check/catalog paths and related scripts
toward Rust `xtask`; exact original migration and source-closure evidence
are in the [M12 plan](DEVELOPMENT.md#rust-tooling-migration-contracts-and-historical-inventory) and
[migration ledger](DEVELOPMENT.md#rust-tooling-migration-contracts-and-historical-inventory). Historical acceptance
did not run every unselected runtime/environment case. All individual
YT-082–086 Backlog items remain Done at their accepted bounds; no
separate CLI/public release is inferred from this migration.

## M11 — Product readiness and optional scene rendering (selected delivery)

[M11 plan](ARCHITECTURE.md#m11-product-and-renderer-decisions) files YT-069–YT-081 after the completed bounded research. M11-P targets one small persisted, authorized, usable SSR-first product with existing CSS; M11-G independently targets browser proof and possible first-party opt-in Experimental `yetlet-wgpu`. Neither line depends on public release, and product readiness does not depend on GPU. [Product research](ARCHITECTURE.md#m11-product-and-renderer-decisions) preserves handoff workstreams A–E and exactly three priority gaps; [GPU plan](ARCHITECTURE.md#m11-product-and-renderer-decisions) preserves native/browser, lifecycle, performance and fallback boundaries. [ADR 0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md) accepts only the bounded Experimental opt-in separation after owner approval and scoped implementation evidence, not stable/public qualification.

**Historical planning checkpoint:** YT-069 covers documentation and owner-permitted planning review only. At that checkpoint, YT-070–YT-081 were Todo and required separate activation. Documentation/static checks and [self-review](ARCHITECTURE.md#m11-product-and-renderer-decisions) cannot qualify a runtime or satisfy later independent implementation/security gates. The original nine candidates and YT-020 remain unchanged. The YT-068/68-item status below is a retained pre-M11 checkpoint, not the current expanded backlog count.

**Current non-GPU continuation (2026-10-14):** YT-070–YT-073 remain Done at authorized local scopes. [YT-074](../examples/lab-operations-board/README.md#historical-board-desktop-and-sqlite-concurrency-evidence) is Done at the owner-amended local scope, with retained real Chewing IME PASS, repaired [Firefox/Orca spoken AT PASS](../examples/lab-operations-board/README.md#historical-board-desktop-and-sqlite-concurrency-evidence) and final Chrome/Firefox PASS. The owner accepted five staggered cohorts for a new full-duration mixed workload; the original synchronized-burst failure remains a limitation on that Ubuntu host. The full 62-minute mixed run passed with 45,000 measured requests and zero errors; fresh independent review passed with no material findings. [Cross-environment synchronized evidence](https://github.com/bohewu/yetlet/blob/1d0eabcb99eeeb681795c1db3e27b2a5b47e0bcb/docs/YT-074-CROSS-ENVIRONMENT-VERIFICATION.md) adds native Windows/Azure Ubuntu full PASS at the original gates, with 44,850/45,000 actual formal requests, while retaining original Ubuntu FAIL and all instrumentation limits. The evidence supports an original-host system/storage limitation; no original-host repair or broader acceptance is claimed. [Handoff](PRODUCTION-READINESS-PHASE2.md#m11-board-acceptance-and-original-blocked-journeys) records completed documentation/static review and the subsequent conditional one-commit local delivery authority, with no new load or push. No GPU, release or deployment activation follows.

**M11 Experimental GPU acceptance (YT-075–081):** The owner authorized
separate local browser-first qualification, CPU/GPU lifecycle and measured
comparisons. Under the opt-in Firefox 158/X11/Intel Vulkan profile, the
Rust/wgpu Wasm consumer passed selected real pixel checks, picking,
loss/recovery, owner retirement and the independent reviews. Firefox's
default WebDriver profile without explicit WebGPU preference instead
showed the intended HTML/no-GPU fallback. Prior Linux Chrome/NVIDIA or
other targets had failed/unqualified attempts; Windows JS shader baseline
did not establish the same Rust/Wasm browser path.

The [retained YT-077 comparison](QUALIFICATION-MATRIX.md#experimental-gpu-browser-and-physics-lab-evidence) covers finite
Canvas2D versus WebGPU lanes **without a demonstrated useful speedup**;
the [cross-crate API audit](API-CONTRACTS.md#cross-crate-api-differences-and-scoped-gpu-refinement) and
[support guide](../crates/yetlet-browser/guides/wgpu-support.md) preserve
explicit SceneError/JsValue and first retirement versus outstanding-borrow
settlement, app-owned fallback and failure boundaries. This research was
accepted at Experimental source/browser scope, not general hardware,
performance, production or first-party 3D-engine qualification.
Original receipts/failed attempts remain at fixed local Git
`983653adadcedb4ec196016b75281d39e2ed0acf`; no remote SHA
reachability or fresh browser execution is implied by doc consolidation.
## Developer documentation refresh — YT-068

The [developer documentation index](README.md) and root README now separate native SSR setup, optional browser behavior and the nine unpublished candidates from retained qualification history. The nine candidate guides, current/history navigation and [final Private source/package boundary](RELEASE-SUPPLY-CHAIN.md#current-preparation-source-boundary-2026-10-12) are reconciled. YT-068 is Done at documentation scope after bounded validation; the independent delivery gate/result is recorded separately in the run; documentation-stage checks do not attest that gate; YT-020 stays Blocked and established YT-030/YT-065/YT-066/YT-067 bounds are retained. YT-029 is Done at bounded deadline/overhead research scope; YT-021 is Done at bounded incremental SSR comparison scope, retaining the app-owned prototype and rejecting core streaming/suspense promotion. YT-024 is Done at bounded typed network adapter proof scope, adopting the app-owned adapter pattern and rejecting core transport framework promotion. YT-022 is Done at bounded data-only handler/capture research scope, retaining explicit application-owned descriptors and rejecting generic/core resumability promotion. YT-023 is Done at bounded native wgpu/Vulkan scene research scope, retaining an opt-in app-owned prototype with actual draw/readback and frame/allocation-byte observations while rejecting core/browser-portability promotion. The next unblockable YT-020 preparation action is the explicit immutable-candidate/source/package and formal release-policy decision record; any new preflight/rebuild/provenance/SBOM/license/signing execution needs separate authority. Historical `8e557e7` signed bytes do not attest policy HEAD `abb2cce` plus current dirty research, root manifest/lock and candidate READMEs. A later local commit alone cannot activate release. No release gate or historical qualification result changes through this entry refresh.

## Critical path

```text
YT-001
  └─ YT-002 (workspace/toolchains)
       ├─ YT-003 (shared view)
       │    ├─ YT-004 (HTML renderer) → YT-005 (server adapter) → YT-006 (SSR demo)
       │    └─ YT-008 (browser build) → YT-009 (DOM renderer) → YT-010 (shared component)
       └─ YT-007 (native build/test CI)
YT-010 → YT-011 (loader) → YT-012 (islands) → YT-013 (search demo)
YT-003 → YT-014 (actions) → YT-015 (HTTP executor) → YT-016 (API integration)
YT-012 → YT-017 (state manifest)
YT-013 + YT-016 + YT-017 → YT-018 (CSP test)
YT-006 + YT-010 + YT-013 + YT-016 + YT-018 → YT-019 (E2E/performance)
YT-019 → YT-020 (release documentation)
YT-027 + YT-016 → YT-030 (thin App facade) → YT-031 (handler/results DX)
YT-030 + YT-026 → YT-032 (opt-in contract) → YT-033 (optional proof)
YT-030 + YT-031 + YT-033 → YT-034 (Todo handler-first demo)
# M6 is not blocked by separate public-release ticket YT-020.
YT-009 → YT-025 (M7a strategy benchmark) → YT-035 (authoring prototype)
YT-035 → YT-036 (local reactive graph) → YT-037 (keyed lifecycle)
YT-036 + YT-037 → YT-038 (SSR/WASM regression)
YT-038 → YT-039 (binding contracts)
YT-039 → YT-040 (local graph) + YT-041 (keyed DOM)
YT-040 + YT-041 → YT-042 (production Todo adoption) → YT-043 (final freeze)
YT-043 → YT-044 (sample contract) → YT-045 (REST) → YT-046 (Session cart)
YT-046 → YT-047 (DX audit) → YT-048 (M8 freeze)
YT-048 → YT-049 (review M9 plan) → YT-050 (Hello) → YT-051 (Middleware)
YT-051 → YT-052 (Forms) → YT-053 (Files) → YT-054 (SQLite)
YT-054 → YT-055 (SSE) → YT-056 (Multi-Island) → YT-057 (Auth)
YT-057 → YT-058 (Modular Shop) → YT-059 (API contract review/freeze)
YT-059 → YT-060 (license/policy) → YT-064 (release supply chain)
YT-059 → YT-061 (repository governance)
YT-059 → YT-062 (SemVer/document metadata) → YT-064
YT-057 + YT-059 → YT-063 (production threat model)
YT-059 → YT-065 (scheduled compatibility matrix)
YT-046 + YT-054 + YT-057 + YT-059 → YT-066 (provider operations)
YT-063 + YT-064 → YT-067 (Codex-assisted internal security scan)
```

Prove native SSR before committing to an elaborate browser view IR. Avoid simultaneously designing DOM, SSR, islands, and GPU abstraction layers before the smallest dual-target implementation works.

## Gate checklist

### M0 — Documentation-only design baseline

- [x] `README.md`, `AGENTS.md`, and `docs/*` exist with valid relative links.
- [x] All backlog IDs are unique; every work item has acceptance evidence requirements and dependencies.
- [x] ADRs distinguish proposed choices from tested behavior.
- [x] `python3 scripts/check_docs.py` passes across tracked and untracked documentation, including links and whitespace.

**M0 evidence (2026-10-08):** `python3 scripts/check_docs.py` PASS (initially 9 documents, 25 backlog items, dependency DAG). `git diff --check` PASS for tracked changes; untracked documents were checked separately by the script.

### M1 — Native SSR foundation

- [x] `cargo test --workspace --all-targets --locked` passed at the M1 baseline (11 tests).
- [x] `cargo build -p yetlet-todo-server --locked` produced a binary started during an HTTP smoke run.
- [x] `GET /` returns escaped content with no mandatory browser runtime; strict CSP is active.
- [x] `yetlet-server-axum` uses the established Axum/Tower stack and `/api/ping` is a separate JSON route.

**M1 evidence (2026-10-08):** Rust 1.99, 11/11 tests, format/Clippy and native plus WASM shared UI compile PASS. A bounded localhost smoke returned 200 HTML for `GET /` and 200 for `/healthz`, without script/WASM references or unsafe-inline CSP. HTML was 423 bytes at that snapshot; **not** a performance baseline. CI YAML was created but had not run on GitHub at that time.

### M2 — Browser renderer

- [x] Browser WASM mounted and updated actual DOM in headless Chrome, not merely compiled.
- [x] The same `todo_list` Rust source rendered identical initial HTML in native SSR and browser DOM.
- [x] Two local clicks updated state without network requests or WebSockets.

**M2 evidence (2026-10-08):** 16/16 Rust tests and Clippy PASS at the M2 gate. `scripts/build_browser.py` generated a hashed manifest and browser modules. The headless Chrome `scripts/smoke_browser.py` confirmed byte-identical SSR/DOM TodoList, state changes, and no remote event circuit. See [M2 verification](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements). Remote GitHub Actions execution was not verified.

### M3 — SSR islands and bounded state v1

- [x] `GET /search` renders a complete usable SSR input and result list without eagerly importing Search WASM.
- [x] Small external `search-loader.js` and versioned assets activate Search WASM on local interaction.
- [x] Input events remain in the browser; initial SSR results/input/heading are preserved.
- [x] `YT-017`: Explicit allowlisted, size-bounded Search Island v1 JSON state, escaped as an HTML attribute. Invalid versions fail closed and preserve readable SSR markup.

**M3 evidence (2026-10-08):** Native/WASM builds, `scripts/build_browser.py`, and headless Chrome `scripts/smoke_browser.py` verified lazy loading, DOM preservation, local search/empty/reset, versioned state acceptance, and incompatible-state fallback. See [M3 verification](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements). YT-011–YT-013 and YT-017 are complete at **prototype** scope; generalized resumability is not claimed.

### M4 — Explicit actions, API, and security

- [x] `YT-014`: Typed `Command`/`ActionExecutor` and injected `LocalExecutor` with unit and browser event-boundary evidence.
- [x] `YT-015`: Explicit same-origin-path `HttpExecutor` sends only semantic JSON commands; no raw DOM-event forwarding or automatic mutation retries.
- [x] `YT-016`: `GET/POST /api/todos` in-memory snapshot API validates body, command, and error status; failed mutations preserve state.
- [x] `YT-018`: Prototype strict CSP and browser checks, typed HTTP 4xx responses, nonmatching Origin rejection, and bounded JSON body handling.

**M4 evidence (2026-10-08):** Native Cargo/Clippy, browser WASM build and headless Chrome E2E demonstrate the local/HTTP action separation, one typed Create POST, server snapshot rendering, visible typed 422 error, and regression of SSR/Islands. See [M4 verification](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements). This is an **anonymous single-origin HTTP demo**, not an authenticated production security review.

### M4.1 — User-approved HTTP pipeline integration

This bounded extension comes from the Middleware Pipeline & RequestContext handoff and preserves the existing implementation sequence. See [ADR 0004](adr/0004-request-context-and-middleware.md) and [HTTP pipeline](HTTP-PIPELINE.md).

- [x] `YT-026`: Request-only context, request-local typed extensions and distinct application state.
- [x] `YT-027`: Native Tower pre-routing global layer, nested groups, route layers and explicit 404/405/error semantics.
- [x] `YT-028`: Actual API/SSR integration, streaming/request-isolation tests and browser boundary regression.

**M4.1 evidence (2026-10-08):** 46/46 tests, strict Clippy/format, English docs, browser compile and one headless regression PASS. Two browser WASM artifacts remain byte-identical. Native overhead/production assurance remain unmeasured; see [HTTP pipeline](HTTP-PIPELINE.md).
- [x] `YT-029`: [Opt-in deadline and measured native overhead research](HTTP-PIPELINE.md#application-owned-deadlines-streaming-and-admission-timing), completed 2026-10-12 at application-owned example scope. Adopt native Tower/Axum composition for explicitly selected use; general providers, supported core API promotion and production qualification remain deferred. Original P0 evidence above is unchanged.

### M5 — MVP release-evidence freeze

- [x] Reproducible **local** results for A01–A09, mapped to actual Rust/Chrome/Firefox tests.
- [x] Release-mode baseline measured for local SSR first-byte, HTML, gzip artifact estimates, Browser/WASM and Search Island readiness; environment and limitations recorded.
- [x] Documentation distinguishes locally tested prototype capabilities from production/public release gates.
- [ ] **Public-release authorization:** The license and bounded qualification work are complete. [Private experimental 0.1.0 preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12) selects the original nine libraries and prepared API/MSRV/maintenance scope; immutable-source/registry, trusted release integrity, public reporting/scanner activation and explicit approval remain open. Application deployment qualification is separate.

**M5 local evidence (2026-10-08):** [M5 acceptance and measurements](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements) documents A01–A10, release Rust/Chrome 155, Firefox 157 physical-keyboard E2E, 30 SSR timing samples, 5 Chrome navigation/activation samples, and asset sizes. `YT-019` is locally Done; `YT-020` is Blocked for publication, with explicit owners in [Release readiness](RELEASE-READINESS.md). GitHub CI has not executed the new M5 workflow changes.

**Later delivery/preparation update (2026-10-11):** The earlier hosted-CI statement above records the M5 checkpoint. Maintainer PR #4 and post-merge `a61f409` required CI passed. YT-065/066/067 retain their bounded completion and historical source identities. The maintainer authorized private `0.1.0` preparation only; local package verification is not a hosted run on these uncommitted changes, signed release evidence or publication approval. YT-020 remains Blocked for actual activation.

**Private integrity update (2026-10-12):** PR #5 delivered the original package preparation as `633db585`; required PR/master runs `38144070219` / `38145022739` passed with the original check names. The maintainer then selected Rust 1.99 before first publication. [Integrity preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12) records byte-identical nine-crate source archives from separate Ubuntu/Windows hosts, honest shared-control limits, target inventories, unsigned provenance drafts and public verification fixtures. These working overlays have no new hosted CI or release authority. Protected-check migration and the existing concrete activation gates remain open; YT-020 stays Blocked.

**Later authorized Private follow-up (2026-10-12):** Reviewed integrity preparation was committed as `9ec9b7b`. PR #6 first-head run `38193149046` passed both new jobs on tree-equal synthetic checkout `cdc67bb6`; protected checks were then atomically migrated to Rust 1.99 MSRV plus the unchanged Stable job, with all other controls retained. [Hosted rebuilding](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-020-HOSTED-REBUILD.md) prepares a single manual GitHub-operated builder for the same immutable source as the primary. Later candidate/merge/rebuild evidence is retained separately and never attributed to the first-head run. Signing and actual release activation remain open; YT-020 stays Blocked.

### M6 — Handler-first developer API and opt-in modules (locally complete)

**Goal:** Let developers build a small SSR site or typed JSON API with ordinary Rust handlers and components, without mandatory MVC Controllers, source layouts, model binders, DI containers or result hierarchies. Expose a **thin Axum/Tower facade**, not a second router. Keep all optional capabilities explicitly installed and browser-safe. See [ADR 0006](adr/0006-handler-first-mvc-optional.md) and [ADR 0005](adr/0005-opt-in-capability-modules.md).

- [x] `YT-030 / M6a`: Thin `App<S>` facade supports ordinary GET/POST handlers, native `MethodRouter`, nested groups, scoped middleware, explicit state and raw Router escape hatches.
- [x] `YT-031 / M6b`: Page and async View route convenience; native typed inputs/results/rejections, redirects and errors; RequestContext remains optional.
- [x] `YT-032 / M6c`: configure/try_configure execute typed setup functions once; no plugin registry, service locator or mandatory provider dependency.
- [x] `YT-033 / M6d`: Independent yetlet-request-id crate proves opt-out, scoping, error/short-circuit/concurrent behavior and core/browser dependency isolation.
- [x] `YT-034 / M6e`: Todo migration preserves SSR, API and middleware contracts; Chrome/Firefox regression and release before/after measurement completed. No zero-overhead performance claim.

**M6a local evidence (2026-10-09):** Five App Builder integration tests and a compiling rustdoc example passed; the existing RequestContext/Tower integration remains intact. [M6a verification](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements) documents API surface, native semantics and remaining gaps. This is neither the M6b result API nor a performance/production release claim.

**Complete M6 local evidence (2026-10-09):** 66 workspace tests and 4 doctests, strict Clippy, native/minimal/wasm32 builds, Chrome 155 and Firefox 157 regressions passed. Release asset hashes and HTML sizes are unchanged. The same-session native first-byte median changed from 0.019 to 0.033 ms; historical M5 was 0.034 ms. See [M6 verification](QUALIFICATION-MATRIX.md#early-ssr-and-handler-measurements) for unrounded methodology/limits and [Extensions](EXTENSIONS.md) for the actual composition contract. Production/release gates remain separate.

**Deferred:** MVC-specific helpers are optional future choices, **not required** for M6; so are Controller inheritance, convention discovery, `#[controller]` macros, full template DSL, Session/Authentication/Redis/Cache providers, a dynamic Plugin Manager and a mandatory DI container.

### M7 — Component authoring, keyed DOM and local reactivity (locally complete)

- [x] `YT-025 / M7a`: Isolated release benchmark compares current subtree replacement, positional View patch, keyed direct DOM and static template slots with identical seeded workloads.
- [x] `YT-035 / M7b`: Isolated keyed/static-slot authoring API covers native SSR, identity preflight, typed text/attributes, one conditional/local event, keyed structure and explicit disposal.
- [x] `YT-036 / M7c`: Explicit owner plus weak handles/subscriptions, FIFO batching, bounded re-entrant passes, fail-closed errors and local keyed-render commits without remote events.
- [x] `YT-037 / M7d`: Controlled row input, keyed insert/remove/reorder, focus/selection and listener cleanup verified in Chrome 155 and Firefox 157.
- [x] `YT-038 / M7e`: Complete native/SSR/WASM/CSP/bundle/performance regression and freeze a no-decision API classification.
- [x] `YT-039 / M7f`: Promote renderer-neutral component/instance/key/slot and inert-attribute contracts into `yetlet-core`; retain fixed authoring as experimental.
- [x] `YT-040 / M7g`: Extract the proven synchronous strong-owner/weak-handle local graph into an optional production crate.
- [x] `YT-041 / M7h`: Implement a general keyed DOM reconciliation and explicit binding/listener ownership primitive.
- [x] `YT-042 / M7i`: Migrate the production Todo Browser local list path and prove Chrome/Firefox identity/lifecycle behavior.
- [x] `YT-043 / M7j`: Run the final full regression, bundle/performance comparison, architecture decision and evidence freeze.

**M7a local evidence (2026-10-09, retained report `2026-10-09T02:02:22+00:00`):** 200 rows, 500 updates, 10 measured rounds in Chrome 155. Median total round: subtree replacement 213.500 ms; positional View patch 41.700 ms; keyed direct DOM 2.000 ms; static template slots 1.900 ms. All strategies produced equivalent final states. Only keyed/slot prototypes preserved the untouched keyed row after reorder; whole replacement also lost owned input identity/focus. Production Browser/Search digest remained `3f3c5f074478c525`. See [M7a verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence). This is architecture evidence, not a selected production renderer or framework comparison.

**M7b local evidence (2026-10-09):** Four native authoring/SSR tests, one compiling rustdoc example, wasm32/release build and an isolated Chrome 155 E2E passed. Mismatched instance/key or actual child-node/slot shape fails before binding; successful bind preserves SSR root/row/label/text identity, handles local typed events and conditional insertion/removal, preserves keyed rows across reverse/append/remove, and disposes listeners explicitly without stale handles affecting newer owners. Production digest remains `3f3c5f074478c525`. See [M7b verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7c local evidence (2026-10-09):** Thirteen reactive-core tests plus the existing four M7b tests and one doctest pass. The isolated Chrome fixture verifies a three-message/one-commit batch, deferred re-entrant dispatch, explicit observer/owner disposal, recoverable reducer failure, faulted renderer failure with keyed DOM identity preserved, and no HTTP/WebSocket circuit. Reviewer regressions prove nested subscription cleanup, caller iterator/state release and reducer/commit failure cleanup occur outside graph borrows without retaining replayable queue work; reducer-triggered owner disposal skips remaining messages/commit. See [M7c verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7d local evidence (2026-10-09):** The fixed keyed row adds one allowlisted controlled text input. Chrome 155 and Firefox 157 preserve its node identity, live value, focus and selection through keyed reorder; focused row removal and owner disposal remove input/click listeners so detached events cannot update the graph. Native SSR escaping/authoring checks and strict CSP remain intact. See [M7d verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7e local evidence (2026-10-09):** 86 workspace tests, 5 doctests, strict Clippy/format, four-package wasm32 checks, production Chrome 155 and Firefox 157 regressions, the complete M7a workload, and isolated M7b/M7c/M7d Chrome/Firefox release regressions pass. Production Browser/Search assets remain byte-identical at `3f3c5f074478c525`, and dependency graphs exclude both M7 experimental crates. The outcome is a no-decision evidence freeze: several small primitives are candidates for later extraction, but no current M7 API or renderer is promoted. See [M7e verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7f local evidence (2026-10-09):** `yetlet-core` now owns validated `ComponentIdentity`, `BindingKey`, `SlotName`, `InertAttributeName` and `SlotDescriptor` contracts. The isolated authoring prototype consumes those types while retaining its experimental fixed shape. Five core tests, seventeen prototype tests, one doctest, strict targeted Clippy and wasm32 compile pass. See [M7f verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7g local evidence (2026-10-09):** The exact M7c graph implementation and its thirteen ownership/batching/re-entrancy/failure/teardown tests now live in dependency-free `yetlet-reactive`. The keyed-binding fixture uses a compatibility re-export and retains four authoring tests plus one doctest. Strict targeted Clippy and native/wasm32 checks pass. See [M7g verification](M7G-VERIFICATION.md).

**M7h local evidence (2026-10-09):** Optional `yetlet-dom-keyed` now owns unique-order validation, keyed entry state, reconciliation and disposable listener lifetimes. The existing fixed-shape fixture consumes it and passes Chrome 155/Firefox 157 reorder, form-state and teardown regressions. Direct/feature-gated placement in `yetlet-dom` changed production bytes; the independent crate restored the canonical production digest `3f3c5f074478c525`. See [M7h verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7i local evidence (2026-10-09):** The production Todo Browser local list uses `yetlet-reactive` plus `yetlet-dom-keyed`, while the remote typed HTTP path and Search Island remain separate. Chrome 155 and Firefox 157 preserve local list/row identity, verify queued click→dispose retirement, ignore a delayed HTTP completion after disposal/remount, and retain SSR/CSP/API/Search regressions. The repaired production digest is `faebb0709dccd863`; the Browser runtime increases by 82,726 raw / 20,042 local-gzip bytes relative to M7e, while Search JS/WASM remain byte-identical after separate package builds. See [M7i verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

**M7j final evidence (2026-10-09):** Replacement validation passes 91 workspace tests, 5 doctests, strict Clippy/format, six-package wasm32, debug/release Chrome 155 and Firefox 157 regressions including queued/in-flight lifetime races, the full M7a workload, isolated keyed lifecycle suites, dependency checks and a repeated release baseline. ADR 0007 selects only bounded identity/graph/keyed-DOM primitives and the Todo local-list adoption. Production digest is `faebb0709dccd863`; Search remains byte-identical. The first final reviewer found the lifetime P2 that this source repairs; repeat reviewer `wc_job_XZzdXEOIx6owj1o5` returned PASS with no issues or follow-ups. See [M7j verification](QUALIFICATION-MATRIX.md#keyed-renderer-and-lifecycle-evidence).

### M8 — Sample-driven application experience

**Goal:** Validate that Yetlet's existing handler-first, component-first and opt-in capability model produces understandable applications with low accidental boilerplate. Samples are evidence, not excuses to add Controllers, DI, a macro DSL or a general client framework.

- [x] `YT-044 / M8a`: Standardize a versioned runnable-sample catalog, local READMEs and validation gate.
- [x] `YT-045 / M8b`: Add an API-only REST sample with no browser/WASM dependencies.
- [x] `YT-046 / M8c`: Add an SSR shopping cart with an explicit opt-in Session provider and bounded security contract.
- [x] `YT-047 / M8d`: Audit repeated boilerplate and dependency cost across the three applications.
- [x] `YT-048 / M8e`: Run the complete regression and freeze the M8 evidence/architecture decision.

**M8a local evidence (2026-10-09):** `examples/samples.toml`, sample-local READMEs and `scripts/check_samples.py` now validate repository-contained paths, exact launch commands, complete documented routes, companion/browser mappings and direct optional dependencies. The focused negative suite covers false declarations and malformed types; 10 Todo tests pass. Final reviewer `wc_job_vQ6pGDE0slh9gV8c` returned PASS with no issues. M8a changes no runtime behavior. See [M8a verification](SAMPLES.md#catalog-and-planning-acceptance).

**M8b local evidence (2026-10-09):** The API-only Book CRUD sample passes 7 tests, strict Clippy, a real localhost smoke, catalog/docs gates and native dependency isolation. Independent reviews found routing-contract, authority-parser and no-Origin Host-validation defects; all material findings were repaired with focused regressions and complete M8b validation. Final reviewer `wc_job_d911MsTpbwT6fZY3` returned PASS with no issues or required follow-ups. See [M8b verification](SAMPLES.md#rest-api-acceptance).

**M8c local evidence (2026-10-09):** The independent `yetlet-session` provider and native-only SSR cart implement random opaque identity, absolute expiry/capacity, fixation resistance, cookie lifecycle, session-bound CSRF, refresh persistence, bounded cart/form state and explicit absence from unrelated dependency graphs. Stage reviewers repaired atomic-rotation, stale-cookie and non-loopback-listener defects; M8e later found and repaired the stale-destroy removal-cookie variant. Final replacement validation passes 114 tests and 5 doctests. See [M8c verification](M8C-VERIFICATION.md).

**M8d local evidence (2026-10-09):** A reproducible audit compares application-owned Rust source, tests, routes, direct dependencies and optional capabilities across Todo, REST and Cart. Repeated patterns are either already handled, small native bootstrap or application/security-specific; no runtime API promotion is justified. Reviewer `wc_job_lZSDSBgMKWu1X5X7` returned PASS with no issues or required follow-ups. See [M8d verification](SAMPLES.md#dx-audit-and-final-freeze).

**M8e final evidence (2026-10-09):** The complete workspace/native/browser/sample matrix, release artifact/dependency isolation, retained render/keyed fixtures, local baseline and M8d metrics pass. Final reviewer `wc_job_dospP8AM0wUmvHFd` found one stale-destroy cookie race; it was repaired with a deterministic two-request regression and 114-test replacement validation. Repeat reviewer `wc_job_J_MVNzFaVgTH1Gr1` returned PASS with no issues or required follow-ups. M8 is locally complete. See [M8e verification](QUALIFICATION-MATRIX.md#sample-and-api-freeze-evidence).

### M9 — Sample expansion and API contract qualification (implementation underway)

**Goal:** Challenge M8 application ergonomics using the original nine-sample catalog sequence plus three targeted supplements, then classify real API compatibility maturity. The [M9 implementation plan](SAMPLES.md#catalog-and-planning-acceptance) is the authoritative scope and test matrix; M9p planning is complete, and implementation begins with bounded independently reviewed samples.

- [x] **YT-049 / M9p:** Documentation, unique task/dependency graph, SQL-first/no-ORM policy and independent read-only planning reviewer PASS before any M9 code.
- [x] **YT-050 / M9a:** Minimal native Hello World sample is cataloged and independently reviewed.
- [x] **YT-051 / M9b:** Middleware Lab with native global/group/route scopes, streaming/error validation and independent reviewer PASS.
- [x] **YT-052 / M9c:** Stateless SSR Forms Validation with field errors, native PRG, English language override and independent repeat reviewer PASS.
- [x] **YT-053 / M9d:** Bounded File Server, static/download headers, protected multipart, private storage, interrupted-write cleanup, dependency/asset isolation and independent reviewer PASS.
- [x] **YT-054 / M9e:** SQL-first SQLite CRUD using application-local `rusqlite`, complete regression and repeat reviewer PASS.
- [x] **YT-055 / M9f:** Bounded SSE Notifications with replay/cancellation/backpressure, complete isolation and independent repeat reviewer PASS.
- [x] **YT-056 / M9g:** Multi-Island Dashboard with separate bundles/state/lifetimes, missing/invalid fallback, Chrome/Firefox and final reviewer PASS.
- [x] **YT-057 / M9h:** Security-bounded Auth Notes with login/logout rotation, CSRF, per-user authorization/isolation and security-focused reviewer PASS.
- [x] **YT-058 / M9i:** Modular Shop explicit-module integration, transactional/idempotent orders, disabled-module behavior and repeat independent reviewer PASS.
- [x] **YT-059 / M9j:** [API contract inventory](API-CONTRACTS.md), Stable Candidate/Experimental/Internal classification, full native/browser/provider regression and repeat independent final reviewer PASS.

**Explicit data-access boundary:** No built-in Yetlet ORM, ORM runtime, mandatory repository/DbContext, query DSL, migration framework or hidden database provider. YT-054 evaluates existing SQL-first `rusqlite` or `sqlx`-style direct/typed query libraries as **app-selected optional dependencies**. Any third-party ORM is external and opt-in only, never a default dependency. See [M9 plan](SAMPLES.md#sql-first-data-access-and-sqlite-acceptance).

**Planning evidence (2026-10-09):** [M9 planning verification](SAMPLES.md#catalog-and-planning-acceptance) records 40 documents, 59 backlog items, the unchanged three-entry runnable catalog, SQL-first opt-in policy, complete task DAG and reviewer `wc_job_fr93PI1YLOtR3KjE` PASS with zero findings. At the planning freeze YT-049 was Done and YT-050–YT-059 were Todo; current per-stage progress follows below.
**M9a / YT-050 evidence (2026-10-09):** A distinct, cataloged native-only `yetlet-hello-world` proves deterministic SSR, an escaped dynamic route, strict CSP, native 404/405 and absence of optional providers or browser dependencies. Workspace tests 117/117, doctests 5/5, strict Clippy, real HTTP smoke, four-sample catalog and native release build pass; independent reviewer `wc_job_Htk-Quo_aFExMJs3` returned PASS. See [M9a verification](SAMPLES.md#hello-world-acceptance).
**M9b / YT-051 evidence (2026-10-09):** The native-only Middleware Lab demonstrates global/group/method routing, short-circuit, typed RequestContext state, real third-party Tower interoperability and streamed/failing bodies. Workspace tests 124/124, doctests 5/5, strict Clippy, localhost smoke, five-sample catalog and optional-dependency isolation PASS; independent reviewer `wc_job_UnKflDaA8ZQHXCce` returned PASS with no issues. See [M9b verification](SAMPLES.md#middleware-lab-acceptance).
**M9c / YT-052 evidence (2026-10-09):** Native SSR form validation and semantic error associations, escaped untrusted attributes, bounded 2 KiB bodies, strict CSP and no-side-effect 303 PRG pass 130 tests, five doctests and a real localhost smoke. First reviewer `wc_job_7D2rtHdGEfwW1Aet` found an inherited-language accessibility P2; a sample-owned `lang="en"` override on both main views and GET/error/success regressions repaired it. Repeat reviewer `wc_job_lGtk_ZIoAvK4F6PW` returned PASS with no required followups. No DB/ORM/Session/CSRF claim. See [M9c verification](SAMPLES.md#forms-validation-acceptance).

**M9d / YT-053 evidence (2026-10-09):** The bounded File Server passes 14 focused tests, 144 workspace tests, five doctests, full Clippy/format, six WASM packages and one localhost smoke including interrupted-upload cleanup. Browser/Search release digest remains `faebb0709dccd863`, all 136 retained assets byte-identical; file dependencies stay application-local. Independent read-only reviewer `wc_job_g8JSu80-Jseg6lmG` returned PASS with no findings, and runtime telemetry matched the saved `gpt-6-astra`/high configuration. The exact reviewed seventh catalog entry was promoted. See [M9d verification](SAMPLES.md#file-server-acceptance).
**M9e / YT-054 evidence (2026-10-09):** A bounded `rusqlite` versus compatible `sqlx` spike selected direct application-local `rusqlite`; explicit SQL/migration/transactions run behind a bounded `spawn_blocking` gate. Nine focused tests, 153 workspace tests, five doctests, three-process persistence/concurrency smoke, 22 dependency trees, six WASM packages and 136 byte-identical production assets pass. First reviewer `wc_job_Q3pBfUqPYGYNSIrD` found incomplete required-column validation; the fail-closed schema check and regressions were repaired, the full matrix rerun, and repeat reviewer `wc_job_aVBprnEsYc8o3ubi` returned PASS. The exact eighth catalog entry was then promoted. See [comparison](SAMPLES.md#sql-first-data-access-and-sqlite-acceptance) and [M9e verification](SAMPLES.md#sql-first-data-access-and-sqlite-acceptance).
**M9f / YT-055 evidence (2026-10-10):** Native HTTP SSE uses an application-owned bounded Hub with finite IDs, a 16-event replay buffer, eight subscribers, four-event per-client queues, heartbeat comments and disconnect cleanup. Ten focused tests, 163 workspace tests, five doctests, real heartbeat/live/reconnect smoke, all 23 package trees, six wasm32 packages and 136 byte-identical production assets pass. First reviewer `wc_job_LF-i2IsQxFXO-obd` found control characters could be trimmed before validation; raw-input rejection and regressions repaired it, the full matrix reran, and repeat reviewer `wc_job_fa8KJiTxGRuWIEDF` returned PASS. See [M9f verification](SAMPLES.md#sse-notifications-acceptance).
**M9g / YT-056 evidence (2026-10-10):** Complete SSR hosts separately built Counter/Filter islands with independent manifests, versions, listeners and disposal. Six server tests, 169 workspace tests, five doctests, release Chrome/Firefox identity/focus/late-callback/failure-recovery and missing/malformed-manifest fallback pass. Eight wasm32 packages, all 26 dependency trees and 136 byte-identical production assets pass. Reviewers `wc_job_63A__nJFK3BgNWPk` and `wc_job_6pk1_UnsnH8WR4Ns` exposed missing browser fallback and retained-log evidence; both were repaired without runtime expansion, and final reviewer `wc_job_OMstvGrtDiKZ5p6C` returned PASS. See [M9g verification](SAMPLES.md#multi-island-dashboard-acceptance).
**M9h / YT-057 evidence (2026-10-10):** Loopback-only synthetic terminal codes authenticate Alice/Bob; `yetlet-session` separately owns opaque identity/CSRF; every note route enforces server-owned ownership. Login/logout rotate Session ID and CSRF. Twelve focused tests, 181 workspace tests, five doctests, a real two-user smoke, all 27 dependency trees, eight wasm32 packages and 136 byte-identical production assets pass. Security-focused reviewer `wc_job_EgSLng_VafIvs3ju` returned PASS with no findings; the exact eleventh catalog entry was promoted. See [M9h verification](SAMPLES.md#auth-notes-acceptance).
**M9i / YT-058 evidence (2026-10-10):** Identity, catalog, Session cart and direct-SQL orders compose through ordinary Rust modules. Nine focused tests, 190 workspace tests, five doctests, a three-process restart/authorization/disabled-module smoke, 28 dependency trees, eight wasm32 packages and 136 byte-identical production assets pass. First reviewer `wc_job_a_WnhFSftzNoYYgf` found a readiness-failure leak in the Python smoke harness; terminate/kill/reap/stream-close regressions repaired it, and repeat reviewer `wc_job_zbWS7OSbUtaKzYcn` returned PASS. The exact twelfth catalog entry was promoted. See [M9i verification](SAMPLES.md#modular-shop-acceptance).
**M9j / YT-059 evidence (2026-10-10):** The [API inventory](API-CONTRACTS.md) classifies 170 public declarations/constants. Complete regression passes 190 tests, five doctests, 11 native smokes, eight wasm32 packages, three Chrome and three Firefox suites, 28 dependency trees and 136 byte-identical production files. First reviewer `wc_job_BzkzTPcKr4FKATTr` found omitted constants/errors and over-broad upstream reexport classification; both were repaired without runtime changes, and repeat reviewer `wc_job_QkQkovlBlvrhPxCE` returned PASS. Stable Candidates remain pre-SemVer evidence; YT-020 remains Blocked. See [M9j verification](API-CONTRACTS.md#original-inventory-acceptance-and-review-limits).

**Historical trust/registry policy checkpoint (2026-10-12):** [Private public-key preparation and future crates.io distribution](RELEASE-SUPPLY-CHAIN.md#private-maintainer-key-preparation-policy-and-historical-signatures) are selected for the original nine experimental `0.1.0` libraries. Only documentation and independent review are authorized now. Actual key/signing, real registry rights/resolution and formal release integrity/approval remain unverified; OIDC, certificate/chain, transparency/inclusion and builder/source guarantees are preserved. YT-020 stays Blocked and no repository visibility transition is implied.

**Later authorized Private signing (2026-10-12):** The reviewed policy was locally committed as `abb2cce`. [Dedicated-key signing and separate verification](RELEASE-SUPPLY-CHAIN.md#private-maintainer-key-preparation-policy-and-historical-signatures) now pass for three preparation signatures, 37 artifacts and six negative cases. Formal log-required verification rejects the no-log bundle; all release guarantees remain mandatory. Source/build/inventory identities are retained without broad reruns. YT-020 stays Blocked for formal activation; no push, publication or visibility change occurred.

### M10 — Release engineering and production-security qualification

**Goal:** turn prototype evidence into explicit support, governance and production-security decisions without expanding Yetlet core or implying release approval.

**Current-source planning / YT-199 (2026-10-26):** [Delivery scope and evidence
gaps](RELEASE-READINESS.md#historical-source-only-package-inventory-versus-current-publication), Done at documentation-planning scope
after independent Reviewer PASS with zero issues/required followups, compares
`64a6d1c` with the unchanged `ce56e08` candidate. Four source/four evidence bindings
and HEAD/status/staged-zero state matched through review; docs checks passed.
Separate the original nine candidates, seven later unpublished opt-in packages,
application-owned references and experimental tools. Map changed production/test
source, retained proof and missing packaging/feature/browser/security/publication
claims without replaying accepted qualification. Recommend owner selection of a
later delivery scope before any missing checks; this selects no replacement
candidate, stable API, new product work, deployment or Commit/Push. YT-096 remains
In progress with D1–D6 Pending and YT-020 Blocked; adopter hardware/SLO/HA and
operational ownership remain deployment choices, not framework completion gates.

**Sixteen-package co-release review / YT-200 (2026-10-26):**
[Feasibility and missing verification plan](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-200-SIXTEEN-PACKAGE-RELEASE-PLAN.md),
Done at planning-review scope after independent Reviewer PASS, zero issues and
required followups, recommends coordinated experimental preparation of all
sixteen implemented libraries with per-package platform and
feature limits. Register YT-201 metadata/packaged consumers, YT-202 native/MSRV/
opt-in gaps, YT-203 Wasm/DOM/optional GPU gaps, YT-204 inventory/security gaps and
YT-205 technical/reader evidence consolidation as Todo. Reconcile inputs and
reuse accepted proof before any focused new check. Preserve YT-199 as its earlier
reviewed recommendation; no manifest change, product test/build/scan, candidate
selection, resource allocation, Commit/Push, signing or publication occurs in
this planning slice. Four target/five evidence/sixteen manifest bindings matched
through review; YT-199 is preserved and documentation checks passed. The old
ce56e08 candidate, D1–D6 Pending and YT-020 Blocked
remain unchanged; eventual sixteen-subject trust/registry activation is separate.

**YT-201 package preparation (2026-10-26):** [Metadata and local packaged-consumer
preflight](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof), Done at metadata/local-preflight scope
after independent Reviewer PASS, zero issues/required followups, follows
authorized continuation after guarded planning commit f955251. Prepare
sixteen experimental 0.1.0 packages, normal/dev registry requirements, seven
license pairs and bounded package guides; retain workspaces/defaults and all
33 library source files. Ten locks update local versions only. Sixteen normal
Cargo package checks and two isolated native default/integrated-feature consumers
PASS; source-cache and VCS-field inspection failures remain retained. This is
dirty-source/hash-bound local preflight, not final source C, public registry,
Windows/browser/GPU or release qualification. YT-202–205 remain Todo, D1–D6
Pending and YT-020 Blocked; new changes have no Commit/Push authorization.
All 45 source/55 evidence bindings matched through review; documentation checks
passed and staged files remain zero. Package/readme/archive inputs are unchanged
by completion metadata.

**YT-202 native evidence (2026-10-26):** [GNU results and Windows prerequisite
gap](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof), Blocked on configured MSVC Rust/VC Tools/
Windows SDK. Reviewed YT-201 source8fd8751 and delivery integration74f53fe share
the complete tree. Four default normal/build graphs and two new Ubuntu24 userland
native consumers with Rust/Cargo1.99/OpenSSL3.5.9 PASS; no browser compilation,
provider service or old test replay. Owned test containers/images removed. The
current native Windows Runner exposes GNU tools only; no installation or target
substitution is made. Independent partial-evidence Reviewer PASS, zero issues/
required followups, verifies four source/32 evidence bindings and native cleanup;
documentation checks PASS. No whole-item Done is claimed. Both strict Push
preflights stopped before any Push; alternate
delivery history and Windows setup/defer decisions await the owner. Source/API,
YT-096/D1–D6 and YT-020 gates remain unchanged; new documents are uncommitted.

- [x] **YT-060 / M10a:** Dual `MIT OR Apache-2.0` license, package metadata, security policy and maintainer-directed/no-external-PR contribution policy; independent reviewer PASS.
- [x] **YT-061 / M10b:** Collaborator-only PRs, protected `master`, required checks, SHA-pinned GitHub-owned Actions and Dependabot configured; unsupported private-state security services recorded; repeat reviewer PASS.
- [x] **YT-062 / M10c:** Minimal title/language document metadata, provisional compatibility policy, legacy generic compile fixtures, complete stable/MSRV validation and repeat reviewer PASS.
- [x] **YT-063 / M10d:** Product-neutral production edge/authentication threat model, private-cache/CSRF repair, documentation validation and repeat security-focused reviewer PASS.
- [x] **YT-064 / M10e:** Release design/preflight, actual license/source checker, 41 native CycloneDX SBOMs, 16 checksum tests and independent reviewer PASS; not signed/reproducible release activation.
- [x] **YT-065 / M10f — Done at automated qualification scope:** Hosted run `38043565238` passed Ubuntu, Windows MSVC, macOS ARM64 and Linux Firefox jobs; local Windows GNU and offline Docker passed. Real IME, assistive-technology speech, WebKit/mobile and zoom remain explicitly Not verified optional coverage, not supported-platform or conformance claims.
- [x] **YT-066 / M10g — Done at single-host Docker multi-instance and selected Redis-provider scope:** Actual local Session isolation/restart and SQLite operations passed with reviewer. YT-066a is complete at single-host Docker/Memory proof scope; b2 implementation and [b3](QUALIFICATION-MATRIX.md#historical-session-auth-oidc-and-cross-host-evidence) single-host/single-primary/no-persistence Redis qualification passed independent review and were locally committed. The [responsibility clarification](https://github.com/bohewu/yetlet/blob/4ad0978f8a70b36c307f5e538aa1a3aa496a7403/docs/YT-066-PERSISTENCE-RECOVERY-PLAN.md) separates framework semantics/configuration from application operations. The status and limitations are reconciled in [M10g closeout](PROVIDER-OPERATIONS.md#m10-local-operations-and-bounded-framework-closeout), with a separate final documentation review gate. Production TLS/ACL policy, backups, RPO/RTO and operational ownership do not block YT-066. Broader durability/HA/load/deployment guarantees remain unverified, with no new execution.
- [x] **YT-067 / M10h — Done (internal source/fix scope):** Official Codex Security Standard scan `fc4d4af8-5864-4ace-b098-23ebfe249b31` completed on the pre-patch baseline with 227/227 regular files reviewed and one Low finding. Tiny endpoint-validation correction passed Rust/MSRV/WASM/Chrome regressions and read-only reviewer `wc_job_sTA8TdP3N3Veu9uC` PASS. The historical earlier scan remains incomplete; no post-patch complete rescan or external certification is claimed.

**Post-M9 hardening evidence (2026-10-10):** [CI/security review](CI-SECURITY-REVIEW.md) records the first hosted CI failure, bounded browser-harness repair, full-SHA Action pins, read-only permissions, patched `RUSTSEC-2026-0009`, Rust 1.88 verification and the remaining GitHub/security/release gaps. GitHub Actions run `38024582602` completed successfully for commit `86c30a42114ca74f4197f0389707861308a844a7`, including both required jobs and all Chrome release suites.

**M10 sequencing decision (2026-10-10):** The maintainer selected `MIT OR Apache-2.0` and a maintainer-directed model that does not accept public pull requests. YT-060/YT-061 now implement those policies without changing repository visibility. YT-062 retains the tiny/handler-first contract: title plus validated root language only, no arbitrary head manager, CSP DSL, DI, MVC or plugin subsystem. See [M10a](CI-SECURITY-REVIEW.md#m10-policy-governance-and-scan-evidence), [M10b](CI-SECURITY-REVIEW.md#m10-policy-governance-and-scan-evidence), and [M10c planning verification](COMPATIBILITY.md#m10-metadata-compatibility-evidence).

**M10a/M10b evidence (2026-10-10):** All 28 workspace packages carry `MIT OR Apache-2.0` and the canonical repository URL; all 19 examples are nonpublishable, and all nine library package archives contain both license texts. Reviewer `wc_job_MPF4l2t8JaJXBguv` returned PASS. GitHub remains Private with collaborator-only PR creation, squash-only merging, admin-enforced protected `master`, both required CI checks, GitHub-owned SHA-pinned Actions, read-only workflow tokens and Dependabot alerts/security fixes. First governance reviewer found missing check-run evidence and stale monitoring wording; both were repaired, and repeat reviewer `wc_job_lu2WynqvKYemLlmd` returned PASS. Public-only PVR/secret-scanning/CodeQL activation remains a visibility-transition checklist item.

**M10c / YT-062 evidence (2026-10-10):** Legacy page/render signatures remain; `LanguageTag`, `DocumentMetadata`, metadata-aware render functions and one compile compatibility fixture add explicit root-language ownership. Forms Validation adopts root `lang="en"` and removes its nested workaround. First reviewer `wc_job_VWOJoR-lV3PAyS_-` found a narrowed `Page::new` generic bound; the exact `Into<Cow<'static, str>>` contract and borrowed/owned generic regression were restored, complete stable/MSRV evidence reran, and repeat reviewer `wc_job_rz9Ci3gWOLfLKdqN` returned PASS. The bounded API remains Experimental; see [M10c verification](COMPATIBILITY.md#m10-metadata-compatibility-evidence).

**M10d / YT-063 evidence (2026-10-10):** Planning reviewer `wc_job__wg1bHF6DG3ztjjg` passed. The [Threat Model](THREAT-MODEL.md) defines a product-neutral TLS edge, forwarding-header, OIDC, Session/CSRF, authorization, rate-limit, CORS, secrets, telemetry and availability contract while keeping those capabilities external/opt-in. First completed reviewer `wc_job_UPiINEJpZ28tLFyl` found missing private-cache isolation and an overbroad CSRF-token prohibition; both were repaired, documentation validation reran, and repeat reviewer `wc_job_AlEmAzdabqC_tuHv` returned PASS. The document remains requirements evidence, not production certification; see [M10d verification](THREAT-MODEL.md#m10-threat-model-acceptance-and-corrected-boundaries).

**Remaining M10 execution evidence (2026-10-10):** [Approved plan](QUALIFICATION-MATRIX.md#m10-native-browser-and-delivery-reconciliation), [release preflight](RELEASE-SUPPLY-CHAIN.md#m10-unsigned-supply-chain-preflight-evidence), [compatibility matrix](QUALIFICATION-MATRIX.md#m10-native-browser-and-delivery-reconciliation), [local operations](PROVIDER-OPERATIONS.md#m10-local-operations-and-bounded-framework-closeout) and [internal security scan verification](CI-SECURITY-REVIEW.md#m10-policy-governance-and-scan-evidence) keep executed evidence separate from optional unperformed manual coverage, the distributed-provider work required at that historical stage and release gates. The later YT-066 closeout above qualifies only its selected experimental provider; broader deployments remain unverified. No default runtime dependencies, core subsystems or ORM were added.

## Post-MVP research

| Track | Research question |
|---|---|
| Streaming SSR | [YT-021 comparison](HTTP-PIPELINE.md#application-owned-deadlines-streaming-and-admission-timing) retains app-owned incremental fragments; large/delayed first-body gains trade off lower throughput, so supported-core streaming/suspense promotion is rejected |
| Compile-time template IR | M7b–M7e prove one fixed shape; M7f–M7i promote only low-level contracts and one production keyed-list path. General trees remain required before any macro decision |
| Resumability | [resumability decision](ARCHITECTURE.md#research-only-handler-resumption-and-native-composition) proves two finite versioned local handlers with bounded numeric captures and native size/cost evidence; retain explicit app-owned descriptors, reject generic/core resumability promotion |
| Custom transport | [YT-024 proof](EXTENSIONS.md#application-owned-action-transport) uses existing ActionExecutor for one opt-in typed TCP adapter with bounded loopback/error/cancellation evidence; adopt the app-owned pattern, reject core framework promotion |
| WebGPU renderer | [YT-023 study](ARCHITECTURE.md#experimental-scene-browser-gpu-and-physics-lab-boundaries) proves opt-in native wgpu/Vulkan draws at 10k–100k with raw completion-wall-time and explicit allocation-byte observations; retain the app-owned prototype, reject core/browser-portability promotion; no matched DOM/CPU speedup is established |
| Worker compute | Which computations can leave the main thread without compromising DOM correctness? |
| Tooling | When do HMR, bundler integration, and devtools justify their complexity? |

## Decision rule

For every proposed feature, identify its owner: kernel, renderer, transport, server adapter, or optional tooling. If ownership is unclear or responsibilities overlap, document the decision or run a bounded spike before adding core abstractions.

## M22 — Production Readiness Phase 2

[Registration and operational boundaries](PRODUCTION-READINESS-PHASE2.md) track
YT-127 (A1 admission), YT-128 (A2 observability), YT-129 (B1 load/soak/recovery),
YT-130 (B2 backup/restore) and YT-131 (C1 mixed-version compatibility).
[YT-127 implementation/local verification](https://github.com/bohewu/yetlet/blob/be337ff02826978a9baa1b88ddf85696cb4406b4/docs/YT-127-ADMISSION-OVERLOAD.md) is
Done at bounded A1 scope after independent Reviewer PASS, zero issues and
required followups; YT-128 is Done after [local verification and independent Reviewer PASS](https://github.com/bohewu/yetlet/blob/be337ff02826978a9baa1b88ddf85696cb4406b4/docs/YT-128-OBSERVABILITY.md); YT-129 is [Done after independent Reviewer PASS at local load/recovery scope](QUALIFICATION-MATRIX.md#historical-operations-capacity-and-recovery-cohorts); YT-130 is [Done after independent Reviewer PASS at local restore scope](https://github.com/bohewu/yetlet/blob/be337ff02826978a9baa1b88ddf85696cb4406b4/docs/YT-130-BACKUP-RESTORE.md); YT-131 is [Done after independent Reviewer PASS and controller acceptance at local C1 scope](https://github.com/bohewu/yetlet/blob/be337ff02826978a9baa1b88ddf85696cb4406b4/docs/YT-131-MIXED-VERSION-COMPATIBILITY.md).
Implementation, verification and deployment qualification remain separate.
Fully validate and independently review each item before its successor;
unresolved material P0–P2 findings prevent acceptance. The [final five-capability matrix](PRODUCTION-READINESS-PHASE2.md#final-production-readiness-matrix) consolidates accepted scopes; the controller owns aggregate documentation checks and final consolidation review. The separate release-owner decision remains YT-096 D1–D6; [M23 profile planning](#m23--production-deployment-profile-contracts) records accepted deployment-contract planning without expanding accepted M22 proof. Neither authorizes signing or release.

Reuse accepted YT-103 asset evidence, YT-123 research and bounded YT-066/YT-094
recovery records only within their original scope. The generator source closure
is already repaired separately. This stage stays application-owned, opt-in and
local/isolated; no new generic framework or validation infrastructure, external
deployment or public activation is selected. M20 candidate `ce56e08`, YT-096
In progress with D1–D6 Pending, and YT-020 Blocked remain unchanged.

## M23 — Production deployment profile contracts

M23 delivers optional framework capabilities, application/reference integrations,
developer guidance and bounded source/fixture proof. YT-132's independently
reviewed deployment-profile contract remains Done at documentation scope.
YT-133 is Done after separate base-contract/operator-selection and corrected
framework/responsibility reviews, minimal implementation, isolated integration
and cleanup. Its Linux/systemd/native-release, SSH/loopback Nginx/SQLite stack
is an optional private single-instance development reference. Numeric fixture
parameters are reproducibility choices. A production Profile is a consumer
evidence record. Consumer host/accounts/keys/service users/secrets, on-call,
SLO/RPO/RTO/retention/alerts, IdP/infra rollout and production qualification
remain consumer decisions. Security, CSRF/Auth/secret isolation, honest source
limits, unknown-write handling and stop/drain-before-migration remain required.
[Deployment profiles](PRODUCTION-DEPLOYMENT-PROFILES.md) ·
[Starter reference](../examples/production-starter/README.md#single-instance-reference-and-schema-recovery-boundaries) ·
[Formal M23 acceptance/dependencies](BACKLOG.md#m23--production-deployment-profile-contracts-planning).

| Accepted dated scope, 2026-10-23 | Outcome and surviving gate/limit |
|---|---|
| YT-134 local release/schema-2/ext4 |23,549 requests; paced1/2/4-worker points passed, Step8/readiness503 failed; exact acknowledged writes reconciled. Separate ephemeral Azure workload/reboot/readback/cleanup and independent review preserved failed cloud points and probe/clock limits. Fixture capacity establishes no universal req/s or consumer SLO. [Distinct cohorts](QUALIFICATION-MATRIX.md#historical-operations-capacity-and-recovery-cohorts) |
| YT-135 recovery | Three-row snapshot restore, explicit two-acknowledgment loss, missing-index refusal and four-row forward-restart readback passed; rows/index/sequence/unknown-outcome/forward-fix rules remain. [Current-schema recovery](PRODUCTION-READINESS-PHASE2.md#current-schema-recovery-and-starter-ownership) |
| YT-136 telemetry | Enabled/stopped/disabled collection, last-good preservation, safe correlated log projection and oversize refusal passed with cleanup/independent acceptance. Collector, sink, alert and retention operation remain optional. [Original collection proof](https://github.com/bohewu/yetlet/blob/be337ff02826978a9baa1b88ddf85696cb4406b4/docs/YT-136-TELEMETRY-COLLECTION.md) |
| YT-137/138 research | Starter/Board retain manual uncertain-write reconciliation and no automatic retry; Shop retains its existing order deduplication scope. Future replay identity/authorization/atomic effects/retention/restore remain app-owned. Accepted pre-SQL authorization race persists for actual Admin/Board saves until separately selected commit-time authority. Research Done selects no implementation. [Write outcomes](https://github.com/bohewu/yetlet/blob/be337ff02826978a9baa1b88ddf85696cb4406b4/docs/YT-137-WRITE-OUTCOME-IDEMPOTENCY.md), [commit ordering](https://github.com/bohewu/yetlet/blob/be337ff02826978a9baa1b88ddf85696cb4406b4/docs/YT-138-AUTHORIZATION-COMMIT-ORDERING.md) |
| YT-139/140 local authority and OIDC | Stale-Session restore/re-login, business/policy reconciliation and four Admin drains passed. Minimum origin integration and three new focused OIDC tests passed. New live proof used single-process Board listeners without successful token exchange or cross-process Admin OIDC. Accepted IdP/Valkey proof retains its original scope. [Authority/recovery](PROVIDER-OPERATIONS.md#local-auth-permissions-and-multi-instance-authority), [proxy/replica contract](PROVIDER-OPERATIONS.md#oidc-cross-instance-origin-and-callback) |
| YT-141 controlled arrival | Tool comparison and minimal k6 read-only reference passed 14,006 measured valid200 plus501 warmup, p50/p95 1/2ms, p99 2–4ms, six phase bounds/no drops/errors;20 rows/schema/index/sequence reconciled with native stop/cleanup. No saturation maximum. [Cohort](QUALIFICATION-MATRIX.md#historical-operations-capacity-and-recovery-cohorts) |
| YT-142 saturation/recovery | First1,000/s point FAILED with 5 SQL-admission503 among 10,001 requests, app CPU49.49%, no generator drops/throttling.250/s recovery passed 7,501 valid200,30 healthy bins, first0–1s healthy and ten-bin confirmation10.000s. Deliberate stop before 2,000/s leaves no sustained/CPU maximum or new matched passing bracket.20-row/schema/index/sequence equality and cleanup passed. [Original saturation/recovery](https://github.com/bohewu/yetlet/blob/1d0eabcb99eeeb681795c1db3e27b2a5b47e0bcb/docs/YT-142-SATURATION-RECOVERY.md) |

Each accepted item retained independent contract/final review as applicable,
owned cleanup and source/app/binary/schema/data bindings. Conditional authority
prerequisites preserve acceptance order. Starter YT-129–131 proof does not
qualify Admin, another consumer or production. Exact test/reviewer/hash scopes
remain in the formal rows and the locally verified
[original M23 history](https://github.com/bohewu/yetlet/blob/8d6fad7e95ee25849872b974f922d10c71b70a95/docs/ROADMAP.md#m23--production-deployment-profile-contracts).
Remote reachability and ignored/external evidence availability are unverified.

**Historical native/SSR/capacity qualification (YT-143–174):** Done only at
each dated and independently reviewed local comparison/attribution scope.
The [qualification matrix](QUALIFICATION-MATRIX.md#historical-performance-and-capacity-evidence-bounded)
now summarizes accepted and failed populations without treating source
studies as current product capacity or universal latency guarantees. The
[SSR measurement sample](../examples/ssr-performance-baselines/README.md)
and [Axum workload sample](../examples/axum-workload-comparison/README.md)
retain current reproduction boundaries and representative original results.

- YT-143/144 compared identical synthetic native Axum/Yetlet and optional
  PostgreSQL paths; the native SSR 4k/s pair was noncomparable after a drop,
  and initial PostgreSQL observer failure was corrected only for unstarted
  cases. No stable framework-overhead or SQL-capacity claim.
- YT-145–160 separated SSR construction, escaping, URL checks, allocation
  events, body/document serialization and full HTTP timing; the private
  first-child capacity candidate was rejected. CPU/frequency/VM confounders
  and source-derived controls preclude a general speedup guarantee.
- YT-161–169 isolated benchmark profiles, SSH-versus-direct transport and
  short/sustained capacity; 6k/s sustained **failed** while independent 5k/s
  duration-specific cohorts passed. All drops and failed populations remain
  counted; no production SLO, universal capacity or cross-host attribution.
- YT-170–174 used synthetic preverified Auth identities, read-only
  Session/PostgreSQL fixtures, Busy and recovery populations and limited
  passive observability. None establishes real login/OIDC, globally shared
  admission or an exact HTTP tail cause. Security/cleanup/source bounds remain.

The **256 formal Backlog items** and their dependencies continue to
distinguish the individual accepted tasks. Complete original histories,
negative tests and section anchors are recoverable from local Git commit
`1d0eabcb99eeeb681795c1db3e27b2a5b47e0bcb`; their external GitHub
reachability has **not** been verified or authorized for publication.

YT-175 documents the [Session burst/overload policy contract](PROVIDER-OPERATIONS.md#provider-overload-and-quota-distinctions), Done at document-only contract scope, dependent on YT-174/YT-117/YT-127. Retain four active/eight total and existing queue/deadline/worker/security bounds; compare fail-fast admission, application HTTP admission/waiting, separate opt-in rate quotas and bounded safe-read retry. Keep 503 capacity failures distinct from 429 quotas and identity/authorization failures; uncertain writes require reconciliation. Recommend existing provider protection plus separately selected early application admission, without activating implementation, tuning, Settings changes or prior-proof replay. Final independent Reviewer /root/yt175_session_burst_overload_contract_review_20261025 PASS, zero issues/required followups; new documents remain uncommitted/unpushed.

YT-176 selects [opt-in fail-fast outer HTTP admission](HTTP-PIPELINE.md#opt-in-application-http-admission-contract), Done at bounded local integration scope and dependent on YT-175. Contract first, then a private example feature/startup cap and shared native Axum gate outside Session/Auth, held through response construction/finalization. Zero added queue/retry, private 503, unchanged four-active/eight-total Session and deadline/security/worker ownership. New in-process focused cases plus feature-off compile/source checks; no old proof replay, live services or performance qualification. Contract Reviewer PASS; six new composition tests (one old oracle filtered), feature-off compile, format and docs checks PASS. Final independent Reviewer /root/yt176_http_admission_final_review_20261025 PASS, zero issues/required followups; new changes remain uncommitted/unpushed.

YT-177 records the [isolated HTTP admission=4 workload](QUALIFICATION-MATRIX.md#opt-in-http-admission-4816-bounded-cohorts), Done at bounded policy-behavior scope, dependent on YT-176/YT-174; final independent Reviewer /root/yt177_admission_workload_final_review_20261025 PASS, zero issues/required followups. Contract/mapping PASS precede one new12550request cohort:12452valid/98checked HTTP-only503,37454complete records, zero Session busy/queue timeout/SQL errors/drops; source/cap/oracle/counters support conditional pre-Session attribution without gate timestamps. Both eight-read recoveries and exact-owned cleanup18.752s PASS. More rejections than historicalnine remain failed requests, with no causal capacity/SLO uplift. No source/Session/queue/retry change, old proof replay or new-change Commit/Push.

YT-178 records the [isolated HTTP admission=8 comparison](QUALIFICATION-MATRIX.md#opt-in-http-admission-4816-bounded-cohorts), Done at bounded comparison scope and dependent on YT-177; final independent Reviewer /root/yt178_admission_eight_final_review_20261025 PASS, zero issues/required followups. Contract/mapping PASS, same binary/caller/sampler reused without build/replay.12549selected/12475valid/74HTTP-only503,37499complete records,0Session busy/queue timeout/SQL errors/drops. Numerically24fewer failures/23more successes than retained cap4 withone fewer boundary attempt, while burst p95/failed client tails worsen; no randomized/default-cap/capacity/SLO claim. Both recoveries and exact-owned cleanup18.919s PASS; source/Session4active8total/zero added queue/new retry unchanged, failures intact, no new-change Commit/Push.

YT-179 records the [isolated HTTP admission=16 comparison](QUALIFICATION-MATRIX.md#opt-in-http-admission-4816-bounded-cohorts), Done at bounded comparison scope and dependent on YT-178. Contract/mapping PASS, same binary/caller reused without build/replay;12552selected/12540valid/12checked503 (7HTTP-only,5providerBusy),37637complete records,0QueueTimeout/SQLerrors/drops. Numerically62fewer failures/65more successes than cap8 withthree extra boundary attempts and observed downstream pressure; latency/arrival/population uncertainty prevents default-cap/capacity/SLO claims. Both recoveries and exact-owned cleanup18.853s PASS; Session4active8total/zero added queue/new retry unchanged, all failures intact. Final independent Reviewer /root/yt179_admission_sixteen_final_review_20261025 PASS, zero issues/required followups;4docs/352evidence/105mapping/94retained hashes stable. No new-change Commit/Push.

YT-180 records the [example HTTP admission capacity selection guide](../examples/axum-workload-comparison/README.md#http-admission-capacity-selection), Done at documentation-only scope and dependent on YT-177/YT-178/YT-179. Document application-owned opt-in startup choices4/8/16, shared per-process scope, fixed Session4/8 and separate rate quotas; reuse accepted counts, all/valid/failed timing and boundary/recovery/cleanup evidence without tests/builds/load/analysis replay. Suitable priorities preserve total-failure versus provider-pressure and latency tradeoffs; no universal default, runtime/API/Settings/limit change or further experiment. Independent Reviewer /root/yt180_http_admission_capacity_guide_review_20261025 PASS, zero issues/required followups;4document/12evidence/19retained hashes stable, new docs/whitespace PASS. No Commit/Push.


### Post-benchmark application follow-ups — YT-181–YT-184

The maintainer selected YT-181–184 after the benchmark phase through YT-180.
Registration was Todo, followed by separately reviewed contracts,
implementation/new focused proof and final independent acceptance. All four are
now Done at their bounded application-reference scopes. Audit remains
application-owned and public output cache remains consumer-gated/deferred.
[Original evaluation and schedule](https://github.com/bohewu/yetlet/blob/be337ff02826978a9baa1b88ddf85696cb4406b4/docs/YT-181-184-APPLICATION-FOLLOWUPS.md).

- **YT-181 commit-time authorization.** Default-off Board/Admin opt-in selects
  a fresh v2 SQL reference, private Store-bound old-generation tickets and
  same-IMMEDIATE-transaction item/history/revoke order. Trusted revoke/re-enable
  advances generations. Existing-v2 open denies until trusted re-enable;
  restored allow and auto-migration are refused. Six new Board cases plus one
  Admin setting case in two feature conditions passed. Contract and final
  implementation reviews passed with 19 source/41 evidence/12 retained bindings.
  Native middleware/domain composition, Session/file-policy race, bounded
  workers and unknown-outcome/no-blind-retry semantics persist.
  [Authority contract](PROVIDER-OPERATIONS.md#commit-time-authority-and-revocation).
- **YT-182 tracing/worker composition.** Default-off Board/Admin safe
  server-generated RequestId correlation uses scoped Dispatch during HTTP
  future destruction and blocking work, separating response return, drop and
  worker completion. Five new Board cases plus an Admin case in two feature
  conditions yielded seven successful executions after retained compile/drop
  failures. Final independent review verified 21 implementation/81 evidence/
  15 retained bindings. Allowlisted namespace-filtered JSON remains app-owned;
  durable audit, body/remote propagation, exporter/framework extraction and
  performance qualification were excluded.
  [Tracing contract](PROVIDER-OPERATIONS.md#request-tracing-and-owned-worker-spans).
- **YT-183 display cache.** Default-off Shop Moka projection reuse preserves
  two-second source-age, bounded retention/fill wait, late-fill-safe local
  generation invalidation and cancellation/oversize/failure handling. Cart,
  Session/CSRF/order identity and checkout price/stock SQL remain authoritative.
  Nine new executions, strict Clippy/fmt/docs/default isolation and independent
  review passed 20 file/41 evidence/16 retained bindings after the retained offline
  dependency failure. Distributed coherence, latency/throughput and framework
  cache promotion remain unqualified.
  [Shop reference](../examples/modular-shop/README.md#optional-catalog-display-cache--yt-183).
- **YT-184 cross-host/persistent restart.** The optional two-VM SQL/Valkey TLS
  reference used canonical private HTTPS/Secure Session cookies, one authority
  disk and fresh-epoch re-login. Each authorized generation had90 minutes with
  20 reserved for cleanup and a cumulative US$5 ceiling. The original SKU CLI
  timeout preceded same-SKU REST admission. G1 bootstrap failed on two Path
  expressions. G2 reached READY/eight SQL backends and wrong-name TLS refusals,
  then failed the inet netmask observer before raw-row persistence. G3 passed
  eight encrypted SQL connections/four per peer and HTTPS wrong-name refusal,
  then failed the first Auth form without raw HTTP status/body. Source/ACL
  simulation found missing TIME permission. Failed local app-identity probes
  remained invalid diagnostics and stopped that harness path. A separately
  reviewed local named-app/password-AUTH test proved +time-only create/read/
  CSRF/rotate/revoke with namespace/admin denials, preserving the compile
  failure and TCP/oneshot-only scope. G4 passed cross-host Auth/fullJSONSSR/
  tenant/current-policy/logout/app-restart, then failed the first expected200
  after fixed2s Valkey restart while both offline503 remained. One local AOF
  case retained 503 at 2s then200 after 107.495/110.289ms additional observation,
  leading to bounded5s/100ms read-only readiness capture. G5 reused G4 Auth and
  completed authority outage/recovery and same-VM/disk restart with nativeexit0,
  retained transient503, full unchanged SQL/current restrictive policy and
  persisted unexpired old-Session refusal under a fresh epoch, followed by
  policy-respecting login/JSONSSR/logout. G1–5 cleanup passed 440.35/493.61/
  469.97/467.05/512.50s respectively, all groups/credentials gone. Local
  named-app and AOF cleanups took1.034/3.882s. Independent final review
  `/root/yt184_cross_host_final_results_review_20261025` PASS0 verified 17 source/
  83 evidence/eight runtime-config-lock bindings and accepted composed G4/G5
  proof. No sixth allocation was authorized. Arbitrary restore, production/HA
  and release qualification remain outside this accepted reference.
  [Topology, failures and accepted results](PRODUCTION-DEPLOYMENT-PROFILES.md#historical-shared-authority-and-azure-persistence-qualification).

The original contract-stage Pending states, named reviews, failed attempts and
exact source/cleanup receipts remain locally recoverable under the unchanged
[post-benchmark heading](https://github.com/bohewu/yetlet/blob/8d6fad7e95ee25849872b974f922d10c71b70a95/docs/ROADMAP.md#post-benchmark-application-follow-ups--yt-181yt-184).
Formal Backlog acceptance and dependency order remain authoritative. These
results selected no broader roadmap, benchmark replay or release activation.

**M23 onboarding and documentation sequence (YT-185–191): Done at their
separately reviewed, bounded scopes.** YT-185 selected application-owned
typed options for Board/Admin and Shop with separate Cargo feature flags
and explicit default-off runtime startup; four new cases / ten selected
executions passed, preserving all Auth/Session/CSRF semantics. No general
configuration framework or automatic capability registry was selected.

YT-186 classified canonical source/Reader versus maintainer evidence and
prepared a minimum API map, not publication. YT-187 replaced the root
release-history-heavy README with a product/first-run entry and later
corrected its initial Reviewer P2 browser prerequisite gap. YT-188 aligned
ten integration guides to the five-part [Guide Template](GUIDE-TEMPLATE.md)
while preserving old snippets and anchors; an early strict Guard/handoff
failure was subsequently corrected rather than silently reclassified.
YT-189 added a source-backed [First App](FIRST-APP.md) walkthrough,
YT-190 created 27 [Troubleshooting](TROUBLESHOOTING.md) mappings in six
groups without reproducing errors, and YT-191 closed dated CLI/View/opt-in
navigation gaps using the [View guide](VIEW-COMPONENTS.md),
[Opt-in reference](OPT-IN-QUICK-REFERENCE.md) and [CLI guide](CLI-GUIDE.md).
Full original Done/reviewer/failed-check evidence remains at fixed **local**
Git `4792565e6cff7a94c8c2c860d1858f840f122cbe`, not yet proven publicly reachable on GitHub. None of these
steps shipped a new runtime, performed a broader test matrix or activated
Reader/Pages publication; the separately accepted YT-217 local Reader and
YT-218 actual hosted publication retain different authorities.
## M24 — Application-owned 3D physics lab: contract and feasibility

YT-192 selects [documentation-only minimum research](ARCHITECTURE.md#experimental-scene-browser-gpu-and-physics-lab-boundaries)
for a separate opt-in 3D physics example: private wgpu pipeline, Rapier CPU
physics, first desktop-browser candidate, bounded cube/sphere scene, fixed-step
clock and explicit GPU failure/lifetime recovery. Existing flat renderer APIs,
default graphs, source-bound historical proof and release scope stay unchanged.
Numeric limits are selected starting bounds, not measured capacity/performance.
Done at documentation/source-level feasibility scope after independent Reviewer
/root/yt192_physics_lab_contract_independent_review_20261026 PASS0, zero followups;
four Markdown paths, docs/links/structure/whitespace/DAG PASS and1,109 non-target
tracked hashes unchanged. Exact joint compile, new physics/browser/GPU behavior
and resource/performance proof remain Not run. No implementation/sample,
dependency/lock change, runtime/old qualification replay or Commit/Push/publication.
A later isolated prototype needs separate authority; no new implementation item
is automatically activated.


**YT-193–198 — Isolated Physics Lab implementation:** After separately
reviewed YT-192 feasibility, an opt-in native-SSR-first application gained
application-owned Rapier CPU simulation and a private, explicitly activated
browser WebGPU 3D pipeline. The selected native/browser tests and actual
Firefox/Wayland/NVIDIA flows were bounded; failed GPU/adaptation attempts
and original Reviewer corrections remain recorded, not transformed into
general browser support.

The [Physics Lab guide](../examples/physics-lab/README.md) now owns first
click-to-Run, Pause/Step/Reset, explicit retry/cleanup, four scene presets
including bounded slope/friction and an **estimated draw-submission FPS**
which is neither GPU presentation rate nor a benchmark. The
[pre-Wasm startup guide](WASM-STARTUP.md) owns static HTML help, customizable
loader and failure handling before asynchronous Wasm initialization.
Native no-JS remains readable; ordinary navigation does not initialize
a GPU. Nothing here changes Core, the default dependency graph,
engineering-grade physics guarantees or production/browser support.
Complete original YT-193–198 implementation/negative tests are
recoverable at local Git `983653adadcedb4ec196016b75281d39e2ed0acf`
(not yet verified reachable on GitHub).
**YT-202 remaining-cell inventory (2026-10-27):** [Current continuation](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof) retains all accepted GNU/package evidence and the original source binding; no library path changed at delivered4e5ef64. The reconnected Windows Runner now has Rust1.99MSVC, VC Tools and SDK ready. Only fourteen-package native default and Auth/session plus Config/secret-provider consumers remain. Both are Not run: prior official artifact transfer was policy-rejected, and detailed diagnostics require an unavailable admin scope. No exact configuration remedy has been established and no bypass or old test replay occurs. YT-202 remains Blocked pending an admitted route, those executions and final independent review; no new Commit/Push/Merge. Earlier missing-toolchain and Git delivery failures stay historical.

**YT-202 authorized source-route update (2026-10-27):** The owner explicitly selects Git clone for the remaining Windows checks. [The new task-owned clone](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof) is admitted and fixed at74f53fe/tree2faa, matching accepted YT-201 source; no rejected artifact is relocated or transferred. Windows prerequisites are ready; preparation uses the existing Cargo bootstrap/directory-source convention and a newly bound equivalent fourteen-package consumer. The earlier artifact/admin diagnostic blockers are historical. YT-202 is In progress, with both MSVC executions and final independent review pending; accepted GNU/package/provider/browser evidence is retained without replay. No new Commit/Push/Merge or final release selection.

**YT-202 MSVC execution (2026-10-27):** [Two new native consumers](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof) PASS on prepared Rust1.99MSVC/VC14.44/SDK26100: fourteen default package dependencies, then Auth/session and Config/secret-provider with missing-Memory-layer refusal and CPU-only wgpu. Inputs/lock remain unchanged; all40 package-file hashes reconcile to committed CRLF checkout and100 external lock identities are retained, without a Linux archive-byte-identity claim. The fresh fixture dependency-preparation failure and environment-observation correction remain recorded; no passing native run or old suite was replayed. Six task-owned Windows directories are removed and evidence retained. YT-202 awaits final independent review before Done; no Commit/Push/Merge or release activation.

**YT-202 final native acceptance (2026-10-27):** [Final independent Reviewer PASS](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof), zero issues/required followups, closes YT-202 at selected native/MSRV/feature scope. Both fourteen-package MSVC consumers pass with exact source/feature/lock/tool/result/cleanup bindings;4documentation/30new/32retained evidence hashes and24ZIP members match. Original request records Rust1.99MSVC selection; generic non-Git source freshness and directory-source/CRLF limits remain explicit. Prior failures/reviews and accepted GNU evidence stay intact without replay. Four task documents remain uncommitted; no Commit/Push/Merge or release activation. YT-203 is next; YT-204/205, YT-096/D1–D6 and YT-020 gates remain separate.

**YT-203 source-bound browser continuation (2026-10-27):** [WASM/DOM/optional GPU inventory](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof) reuses accepted CI4e5ef64 original-five/MSRV/Chrome evidence, unchanged DOM contracts and YT-081 actual crate GPU inputs; Lab and Tasks remain separate application evidence. Only missing normalized six-package WASM default/browser compile and current-package Firefox154 DOM cells execute: both Cargo checks and23new controls/navigation/CSP/atomic-refusal observations PASS. Source/lock/assets,34-node default isolation and owned cleanup are retained; directory-source/old-GPU-lock/compile-only and unavailable historical tmp limits remain explicit. Final independent Reviewer is pending before YT-203 Done. No new runtime/sample, old suite replay, new-change Commit/Push/Merge or release activation.

**YT-203 final browser acceptance (2026-10-27):** [Independent Reviewer PASS](QUALIFICATION-MATRIX.md#historical-sixteen-package-qualification-and-wasm-proof), zero issues/required followups, closes the selected packaged WASM/DOM/optional GPU evidence gaps. All4 candidate docs,37 evidence/fixture files,17 protected inputs and6 packaged-source comparisons verify; original GPU result/review and28 retained inputs reconcile to the six disclosed changed wrappers. New default Wasm build, browser-feature compile and23 Firefox observations pass with owned cleanup; accepted Chrome/GPU/BFCache proof is reused without replay or scope expansion. Completion metadata alone follows acceptance. Four YT-203 documents remain uncommitted; no Push/Merge, runtime feature or release activation. YT-204 remains Todo and is next; YT-205, YT-096/D1–D6 and YT-020 gates remain separate.

**YT-204 inventory/security continuation (2026-10-27):** After strict/builtin guarded YT-203 commitf31eed7 without Push, [current selected coverage](RELEASE-SUPPLY-CHAIN.md#historical-license-and-advisory-qualification) records42 cells:6 exact-input inventories retained without replay and36 new library SBOM closures matching metadata.31 new license/source cells pass;5 fail unchanged policy for Zlib/BSL-1.0/ISC.16 current-db package-lock audits report0 vulnerabilities/exit0 while10 retain Tokio informational warning; no ignore/upgrade. Scoped Standard source scan63212d73 completes120 regular files/18 aliases with0 reportable findings, preserving old Low/remediation/Auth identities and application/system limits. Delivery review is pending; YT-204 stays In progress for an authorized narrow license decision and successful affected checks. No runtime/lock/policy change, old qualification replay or new-change Commit/Push/Merge; YT-205 is unopened and formal release gates remain separate.

**YT-204 bounded delivery acceptance (2026-10-27):** [Independent Reviewer PASS](RELEASE-SUPPLY-CHAIN.md#historical-license-and-advisory-qualification), zero issues/required followups, verifies4docs/5canonical/6supplemental/138source bindings, sixteen archives, six retained cells and all36 new closures. It accepts truthful inventory/gap reporting and explicitly keeps YT-204 In progress for the five failed license-policy checks; no policy amendment or release approval is implied. Four owned extraction/cache/database directories are removed after review, raw inert evidence and sealed bindings retained. Four task documents remain uncommitted, staged0; no Push/Merge or old qualification replay. YT-205 and formal release gates remain unchanged.

**YT-204 authorized policy followup (2026-10-27):** The owner approves only Zlib/BSL-1.0/ISC additions to the license allowlist, notice preservation and the five failed checks. [All five reruns PASS](RELEASE-SUPPLY-CHAIN.md#historical-license-and-advisory-qualification), zero license/source errors,39 retained input and10 notice hashes unchanged; six unused-allowance warnings remain. All other policy fields, manifests/locks/runtime and prior failures/scan/advisory evidence stay intact.42/42 selected coverage combines prior passes and these five outcomes; no other successful check is replayed. Policy-wide versus per-component distribution scope is documented without relicensing Yetlet. YT-204 remains In progress pending final independent review; no ignore, upgrade or Commit/Push/Merge.

**YT-204 final policy acceptance (2026-10-27):** [Independent Reviewer PASS](RELEASE-SUPPLY-CHAIN.md#historical-license-and-advisory-qualification), zero issues/required followups, closes the authorized license blocker and marks YT-204 Done at selected inventory/security scope. Six candidates/five followup evidence bindings,39 inputs/10 notices/138 source-and-alias bindings and prior canonical/supplemental records verify; all five affected checks pass under the sole three-ID allowlist delta. Other policy, dependency/runtime and historical warning/failure identities remain intact. Restored package trees and owned Cargo home are removed, raw evidence preserved. Six paths stay uncommitted/staged0; no other passed-check replay, Commit/Push/Merge or release activation. YT-205 is next but unopened; formal release gates remain separate.

**YT-205 selected-source consolidation (2026-10-27):** After strict/builtin guarded28-path commita2d6474 without Push, the owner selects that immutable C for all16 experimental0.1.0 libraries. [The local packet](RELEASE-READINESS.md#historical-release-and-hosted-source-checkpoints) binds16 clean archives,10 changed-lock normal package PASS and6 exact compilation reuses, a new integration consumer,8first rustdoc commands and14 affected inventory cells PASS.28 inventories, scoped source scans and earlier native/browser/GPU/audit proof retain their identities without replay; all preparation failures stay recorded. Reader drafts correct cohort/provider guidance but do not change C archives. Required current-source CI/rebuild, affected native dependency evidence and reader-source disposition remain pending; YT-205 stays In progress pending independent delivery review and the remaining acceptance conditions. No new-change Commit/Push/Merge, signing, publication or D1-D6 decision.

**YT-205 independent checkpoint acceptance (2026-10-27):** [Read-only Reviewer PASS](https://github.com/bohewu/yetlet/blob/c440c4542c888671750738a5908a667d2124db15/docs/YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md#independent-checkpoint-acceptance--2026-10-27), zero issues/packet repair requirements, verifies10document/185evidence/1179protected bindings,16clean C archives and their retained/new proof distinctions. The reviewer explicitly keeps item_can_be_done:false for current-source CI/rebuild, affected native dependency evidence and reader-source disposition; YT-205 remains In progress. Owned caches/extracted packages are removed; archives, failure logs, SBOMs and rustdoc remain. No product replay or new-change Commit/Push/Merge, and D1-D6 remain Pending.

**YT-205 current-lock GNU delta (2026-10-27):** [One additional Ubuntu24/OpenSSL3.5.9 integration run](https://github.com/bohewu/yetlet/blob/c440c4542c888671750738a5908a667d2124db15/docs/YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md#once-only-current-lock-ubuntu24-continuation) PASS at C/Tokio1.53.3 with unchanged source/lock and observed Rust1.99/glibc2.39. It closes only this selected native dependency gap; no old suite is replayed. Owned image/container/vendor/build inputs are removed and all raw evidence retained. The earlier checkpoint PASS remains intact; separate independent GNU delta review PASS0 verifies10docs/12additional records, original185records unchanged, and explicitly keeps item_can_be_done:false. Current-source hosted CI/sixteen-package rebuild, MSVC dependency evidence and reader-source disposition keep YT-205 In progress; no Commit/Push/Merge or formal decision.

**YT-205 rebuild preparation (2026-10-27):** After strict/builtin guarded ten-document commit012ecb3 without Push, [the bounded builder extension](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-205-SIXTEEN-PACKAGE-REBUILD.md) preserves original-nine mode and adds explicit sixteen selection, six manifest/lock roots, separate bootstrap/normal verification and conditional pinned OpenSSL3.5.9 preparation. Five focused tests, native compile/lint/format and static manual-workflow checks PASS; expected fixture compile rejection retained and old product proofs not replayed. Independent review is pending. No hosted workflow/OpenSSL setup execution, successor candidate, new-change Commit/Push/Merge or D1-D6 decision; whole YT-205 remains In progress.

**YT-205 rebuild extension acceptance (2026-10-27):** [Independent Reviewer PASS](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-205-SIXTEEN-PACKAGE-REBUILD.md#independent-acceptance-and-remaining-gate), zero issues/required followups, verifies10candidate/28evidence/1181protected hashes and18 unchanged symlink targets. Five focused controls and compile/lint/format/workflow/docs checks pass; fixtures are removed and intentional rejection logs retained. Acceptance is local preparation only: actual hosted CI/rebuild, MSVC and final reader/source conditions remain open, so wholeYT205_can_be_done:false. New code/docs remain uncommitted/staged0; no Push/Merge/dispatch, candidate change or D1-D6 disposition.

**YT-205 new candidate and remaining order (2026-10-27):** The owner authorizes
the ten-path rebuild-extension commit and selects resulting
`118f3d0f496b487ef20fe243f4e73179f9eadc43`, tree
`07faa3e90c0fca5d27a4e55262c75acf851d884a`, as new technical C. Explicit
strict/builtin Guard/native hook PASS0 and the committed tree matches `tree_oid`.
The [remaining verification order](https://github.com/bohewu/yetlet/blob/c440c4542c888671750738a5908a667d2124db15/docs/YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md#new-candidate-and-remaining-verification-order--2026-10-27)
is separately authorized delivery, required PR CI, explicit sixteen-cohort
hosted/local archive comparison, affected MSVC cells and final independent
acceptance. Historical a2d6474 artifacts/results retain their source identities;
new-C archives and remote/platform checks are not run. Selection/order notes
remain uncommitted; no Push/Merge/dispatch. YT-205 stays In progress, YT-020
Blocked and D1–D6 Pending.

**YT-205 current-C technical execution (2026-10-27):** Authorized strict/builtin
Push updates PR12 to118f3d0 after5commits/56file operations/0findings and native
hook PASS. [New execution evidence](RELEASE-READINESS.md#historical-release-and-hosted-source-checkpoints)
records both required CI jobs PASS with a C-equivalent synthetic merge tree,
explicit sixteen hosted/local rebuilds with all33 phases PASS each and all16
archives byte-identical, and two affected current-lock MSVC consumer PASS.
Forty-six reader sources now match the accepted draft at C; prior input-bound
proof is retained without manual replay. Owned scratch is removed. Windows
original text evidence remains readable at its source; a denied local ZIP transfer
is preserved without alternate transfer or a local-receipt claim. Final independent
Reviewer PASS0 accepts the actual local/remote evidence and closes YT-205 at
selected technical/reader scope. Seven document paths stay
uncommitted/staged0; no Merge/signing/publication and D1–D6 Pending.

**YT-206 public-source preview preparation (2026-10-27):** After authorized
strict/builtin closeout commit778423e without Push, the
[disclosure inventory](CI-SECURITY-REVIEW.md#public-source-preview-and-private-reporting) distinguishes technical
C qualification from whole-repository visibility. Current1,191files,114remote
reachable commits,38existing runs,16artifacts,12PR bodies and15synthetic screenshots
are inventoried without replaying accepted product qualification. Existing history
and evidence are recommended for an Experimental preview only after owner
acceptance of the concrete disclosure categories. GitHub Private Vulnerability
Reporting is selected; a separate mailbox/domain is optional. Form availability
and maintainer notifications must be verified during authorized public activation
before announcement. YT-206 remains In progress for disclosure/activation choices.
Actual collaborator-only
PR creation and both required checks are observed/preserved. No Commit/Push/Merge,
visibility/history/log deletion, signing or publication; YT-096/D1–D6 and YT-020
remain separate.

**YT-206 post-activation reconciliation (2026-10-27):** The later owner approval
accepts the inventoried source/history/Actions disclosure and authorizes Merge and
Public visibility. The [activation record](CI-SECURITY-REVIEW.md#public-source-preview-and-private-reporting)
records PR12 squash commit d7065bb, full-tree equality with reviewed3d63f0b,
both required CI jobs PASS, and strict/builtin exit0 with0blocking findings and
9retained non-blocking warnings. The repository is Public as an Experimental
source preview; the delivery branch, failed records, required checks and
collaborator-only PR policy are preserved. GitHub private reporting is enabled
and its public entry is visible; maintainer watch is subscribed/not ignored.
The [signed-in form and actual notification delivery](CI-SECURITY-REVIEW.md#public-source-preview-and-private-reporting)
remain Not verified, with no test report submitted; YT-206 stays In progress for
those checks. Separately authorized About description/topics now reflect the
experimental Rust/HTML-first/SSR/optional WASM scope. The earlier inventory and
failures remain historical rather than being rewritten as current results.
This documentation reconciliation reuses accepted product evidence without
replay; no Commit/Push or further external operation. Technical C118f3d0 remains
selected, YT-096 In progress, YT-020 Blocked and D1–D6 Pending; no release-artifact
signing, version tag, package publication or reader-site export.

**YT-206 executed reporting completion (2026-10-27):** The owner requests actual
completion rather than a scope waiver and explicitly authorizes one private
non-vulnerability test. The [non-admin reporting form and browser submission](CI-SECURITY-REVIEW.md#public-source-preview-and-private-reporting)
PASS with privacy/policy fields; an actual private-report email and collaborator
notice arrive at the configured owner Gmail route after21s/22s. Recipient envelope,
advisory reference, marker and GitHub DKIM/SPF reconcile; the single record is
closed and retained, unpublished with no CVE or private fork. The earlier owner403,
non-persisting optional Web inbox selection and corrected mail-To assertion remain
failures/limits; Email is the verified route. No old qualification is replayed and
no missing check is moved to another item. Independent completion Reviewer PASS0
marks YT-206 Done at this selected scope; existing activation/inventory history remains intact. Technical
C118f3d0, YT-096 In progress, YT-020 Blocked and D1–D6 Pending remain unchanged;
no Commit/Push, signing, version tag, package publication or reader-site export.

**YT-096 supply-chain purpose and D1–D5 recommendation (2026-10-27):** The
[owner packet](RELEASE-SUPPLY-CHAIN.md#adopted-release-trust-and-disclosure-policy--yt-096) makes the remaining purpose
explicit: reduce substituted release bytes, official-signer impersonation and
source/build mismatch risks. It proposes exact public identity/payload fields,
sixteen0.1.0 source packages, technical C118f3d0 as reuse baseline, a future
co-located release-workflow source, Cosign3.1.4 with public upstream binary/root/
configuration reference digests, and bounded evidence/attachment mapping.
No future source SHA or fresh output/certificate is invented. Existing ce56e08
failures, preparation-only provenance and technical evidence keep their identities;
the recipe remains source-package scope without SLSA-level, defect-free-code or
complete-system-inventory claims. Documentation and independent review only;
no old validation replay, implementation, Commit/Push, signing or publication.
All D1–D6 remain Pending, YT-096 In progress and YT-020 Blocked; YT-206 stays Done.


**YT-096 D1–D5 owner adoption (2026-10-27):** The owner adopts the
[recommended policy](RELEASE-SUPPLY-CHAIN.md#adopted-release-trust-and-disclosure-policy--yt-096): GitHub workload/public
Sigstore disclosure, closed sixteen-subject public provenance/DSSE and complete
private evidence, sixteen experimental `0.1.0` packages with technical baseline
`118f3d0` and future co-located `C_release`/`release.yml`/proposed `v0.1.0`,
Cosign3.1.4/reference roots subject to actual bootstrap/validity/compatibility
checks, and source-package-only claims with acyclic bindings. D1–D5 are Adopted
(policy); D6 remains Pending. The owner authorizes build/verify-only Release
preparation workflow implementation, with new changes uncommitted/unpushed.
No real release SHA, tag, final output/payload, authenticated Cosign installation
or new build result is established by adoption; hosted execution, OIDC/certificate
requests, signing and publication remain unapproved. Historical Pending notes,
review PASS and failed release verdicts above retain their dated scope; they do
not approve new implementation. YT-096 stays In progress, YT-020 Blocked and
YT-206 Done. The authorized build/verify-only
[`release.yml` and closed preparation helper](RELEASING.md#existing-workflow-map-and-limits)
are implemented. All 27 focused checks (four unit, three integration, three
selected legacy and seventeen checksum checks), fmt, Clippy, YAML and five Bash
syntax checks PASS; the original private identity guard and provenance tail are
byte-identical. Independent implementation review PASS, zero issues and required followups. No actual
hosted run, real sixteen-product train, release candidate or tag is selected or
executed; no new commit, push, signing or publication occurred. The next bounded
step is owner selection/approval of actual source/tag and hosted preparation, without replaying accepted product
qualification. Earlier failure history remains in the linked implementation
record; focused checks do not establish a signing-ready or completed release.


**YT-096 encrypted custody adoption (2026-10-27):** The owner adopts the
[age custody and D2 safe-metadata extension](RELEASING.md#custody-handoff-and-manual-key-lifecycle),
including a source-pinned public recipient for build/capture and transient private
identity use only in an isolated manually approved protected Environment verifier.
A dedicated local identity and authenticated task-scoped Linux amd64 age bootstrap
exist; local integration/readback PASS with13 focused accepted checks, fmt/strict
Clippy and syntax/API fixtures PASS. Authenticated age preserves original/restored
LOCAL-manifest bytes; native17 partial and tamper rejection retain their scope.
`complete_product_custody=false`; no actual OpenSSL/sixteen-package build is
qualified. Independent implementation review PASS, zero issues and required followups. Actual
Environment protection/Secret installation, hosted execution and durable retention
are unperformed and unapproved. Source freeze follows these changes/review and an
actual reviewed post-squash master SHA; no `C_release` or tag is selected. D6 stays
Pending, YT-096 In progress and YT-020 Blocked; no new-change Commit/Push,
OIDC/signing or publication. Historical checks/reviews retain their original scope.

## Unified startup composition guidance — YT-207

The [Settings, options and logging guide](SETTINGS-OPTIONS-LOGGING.md) connects
the existing Config typed example, Admin/Board options mapping, Shop direct
options and Starter diagnostics. The sequence is explicit source loading,
application validation, narrow typed options and native state construction.
Package/Cargo selection remains distinct from per-instance startup settings;
the Settings loader stays optional and the application owns its subscriber,
filtering and secret redaction. No DI, runtime, API, dependency, new sample or
live reload is added. This documentation-only follow-up to YT-185/YT-188 is Done
after independent Reviewer /root/yt207_settings_options_logging_review_20261028
PASS, zero issues/required followups. All eight reviewed source hashes matched;
guide links, snippets and settings were manually verified, and current
docs/whitespace checks PASS (75 required documents/207 items). Only completion
metadata follows acceptance. Accepted product verification is reused without
replay and shown commands were not executed; no Commit/Push or release
operation occurs, and D6 remains Pending.

## Historical YT-096 — Existing-remote tag baseline design

After separately approved PR13 integration and actual master CI, the owner
selected `C_release` `3eacfe3` and approved the reviewed Environment/Secret setup.
The planned new-tag strict/builtin pre-push scan then stopped on 13 historical
personal/current-machine path findings. No tag or hosted build was created.
The [bounded baseline design](RELEASING.md#release-identities-and-freeze-order) proposes
explicit acceptance tied to the actual single remote push destination, exact
branch/commit and one new lightweight tag, with fresh remote checks and native
hook enforcement. The original full-history default and findings remain intact.
Only the design and independent review are authorized. Independent design
Reviewer /root/yt096_remote_tag_baseline_design_review_20261028 PASS, zero
issues/required followups, follows a P2 evidence correction distinguishing the
direct planned-input Guard assessment from unexecuted native hook enforcement.
Initial FAIL and focused followup PASS are retained; current docs/whitespace
checks PASS (75 required documents/207 items), with no old proof replay.
No Guard implementation/installation, hook or policy change, history rewrite,
old product-proof replay, Commit/Push, tag, hosted execution, signing or
publication occurs. YT-096 stays In progress, YT-020 Blocked and D6 Pending.

## YT-096 — Actual unsigned hosted build and retained custody

The [dated execution record](RELEASE-READINESS.md#historical-release-and-hosted-source-checkpoints) supersedes the earlier
design-only/no-run checkpoint for stages A–F. Managed hooks and actual native
strict/builtin enforcement passed under the reviewed explicit existing-remote
boundary; no historical finding was fixed or suppressed. The unsigned tag and
remote master matched selected `C_release` `3eacfe3`. Build `40628943583`/attempt1
and protected verifier `40631789238`/attempt1 passed once, with owner `bohewu`
Environment approval and successful private-file cleanup. Product evidence is
PREPARATION_SUCCESS, original exit0 and complete=true. Actual ciphertext,
transport/safe readback receipts, public eighteen files, immutable source/tool
bindings and run/approval metadata are retained outside Git through owner
disposition; local SHA/exact-set/provenance correspondence checks passed.
Original Guard/design-review/extraction failures and the stream-observer
correction remain retained; no repush, rearming or hosted rerun followed.

This documentation-only reconciliation grants no operational authority and
replays no accepted product verification. Owner-host retention is not an offsite
backup or indefinite-availability guarantee. D6 remains Pending, YT-096 In progress
and YT-020 Blocked; signing identity/bootstrap/roots, exact signing payloads,
registry/trust and final maintainer release approval remain open. No signing,
publication, Commit or Push occurs in this document update.

## M25 — Broader HTML DOM support and static-site delivery

- [x] **Broader HTML DOM coverage with optional link and image policies**
  (YT-219–221), first published in **SDK `0.9.0-alpha.6`**. The renderer admits
  89 ordinary elements with scoped attributes and canonical structures; explicit
  image options add `img` for a 90-element selected scope. Existing entry points
  retain same-origin navigation and no images. Read the [DOM guide](../crates/yetlet-browser/guides/dom.md)
  for per-mount policies, parser parity and ownership; the [dated inventory](API-CONTRACTS.md#historical-html-element-and-attribute-inventory)
  remains the coverage and Typed Tag Builder selection reference. Element
  admission does not imply every attribute or browser API. The [DOM guide](../crates/yetlet-browser/guides/dom.md)
  and [original policy evidence](https://github.com/bohewu/yetlet/blob/b9873f15bcfd4c36a2274508ef7517e5a83b807b/docs/YT-221-DOM-URL-IMAGE-CONTRACT.md) retain focused
  native/Wasm/Chrome/Firefox evidence and original failures, not exhaustive
  element, performance or platform qualification. The [publication record](ALPHA6-RELEASE-PREPARATION.md#owner-approved-publication-and-registry-consumer--2026-11-01)
  separately proves distribution and registry resolve/compile.

- [x] **Opt-in static HTML export through `yetlet build --static`**
  (YT-222–223), first published in the independent **CLI `0.2.0`**. A trusted
  application-owned native exporter supplies finite public pages/assets; the CLI
  writes complete HTML with explicit base paths, CSP and optional prepared Wasm
  islands. HTML-only export needs no backend, browser or Node. Output is
  create-only and private receipts stay outside uploaded content. Start with the
  [CLI guide](CLI-GUIDE.md#static-html-build) and [standalone example](../examples/static-site/README.md);
  retain the [static export contract](CLI-CONTRACT.md#historical-static-export-protocol-and-guardrails) and
  [original implementation evidence](https://github.com/bohewu/yetlet/blob/b9873f15bcfd4c36a2274508ef7517e5a83b807b/docs/YT-223-STATIC-HTML-IMPLEMENTATION.md). Bounded GNU/MSVC
  CLI execution and Chrome/Firefox checks remain tied to their original inputs.
  The [CLI publication and installed-consumer record](CLI-020-RELEASE-PREPARATION.md#owner-approved-publication-and-installed-consumer--2026-11-02)
  is separate from those static/browser checks. This tooling adds no static-export
  SDK module, hosting service or deployment guarantee.

- [x] **Yetlet builds and deploys the public Reader Edition** (YT-224–226).
  Yetlet renders the canonical guide sources as static HTML, and GitHub Pages
  deploys the resulting artifact. On November 1, 2026, the authorized YT-226
  cutover replaced the mdBook site using run `41289520993/1` from master
  `0ae6b93`. All 15 artifact outputs matched the accepted build, and HTTPS
  readback of eight chapters and six assets matched the artifact bytes and MIME
  types. The independent final review accepted the cutover with the recorded
  Firefox checker failure retained. The [formal Pages publication record](GITHUB-PAGES-PUBLICATION.md#authorized-yetlet-publication--2026-11-01)
  describes the accepted deployment, source, hashes and limits. The original
  [YT-224 local pilot](https://github.com/bohewu/yetlet/blob/f03e8886bc1dbe28f135e05e527b69e01eb3deba/docs/YT-224-DOCS-PILOT.md),
  [YT-225 hosted artifact](https://github.com/bohewu/yetlet/blob/f03e8886bc1dbe28f135e05e527b69e01eb3deba/docs/YT-225-DOCS-PUBLIC-BUILD.md#authorized-delivery-and-hosted-artifact--2026-11-01)
  and [YT-226 independent acceptance](https://github.com/bohewu/yetlet/blob/f03e8886bc1dbe28f135e05e527b69e01eb3deba/docs/YT-226-PAGES-CUTOVER-PLAN.md#independent-final-acceptance--2026-11-01)
  remain recoverable from fixed Git history. The [Reader Edition manifest](../site/pages.toml)
  independently selects the current SDK and CLI pair.

Formal Done items and dependencies remain in the [M25 Backlog](BACKLOG.md#m25--broader-html-dom-support-and-static-site-delivery).
Static serialization does not require DOM mounting; the documentation pilot joins
the two capabilities as an application-owned consumer.

The superseded [shared planning and execution summary](https://github.com/bohewu/yetlet/blob/1508b6910add4e74c9e2311a3f3eb4666539afce/docs/YT-219-224-DOM-STATIC-SITE-PLAN.md)
is retained at fixed Git revision `1508b6910add4e74c9e2311a3f3eb4666539afce`.
Its original headings, including the [static-build boundary](https://github.com/bohewu/yetlet/blob/1508b6910add4e74c9e2311a3f3eb4666539afce/docs/YT-219-224-DOM-STATIC-SITE-PLAN.md#yt-222--static-html-build-contract)
and [pilot scope](https://github.com/bohewu/yetlet/blob/1508b6910add4e74c9e2311a3f3eb4666539afce/docs/YT-219-224-DOM-STATIC-SITE-PLAN.md#yt-224--application-owned-documentation-pilot),
remain available there. Current usage belongs to the guides above; the linked
contracts, original verification records and formal release records remain in place.
