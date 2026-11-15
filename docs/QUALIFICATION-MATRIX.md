# Platform, Browser and Accessibility Qualification Matrix

**Task:** YT-065 / M10f. **Current status:** **Done at automated desktop/native/browser qualification scope.** Run `38043565238` tested source `c38837835dac683b09194b475baab2d8eda5a875`. Real IME, assistive-technology speech, WebKit/mobile and zoom remain explicitly Not verified optional coverage. This matrix supplies no supported-device range, accessibility conformance, SLA or production guarantee.

## Coverage and evidence boundaries

| Lane | Defined scope | Current YT-065 evidence | Historical evidence |
|---|---|---|---|
| Ubuntu 24.04 native | Stable Rust check, library tests and doctests for nine reusable crates | Hosted Rust 1.99, x86_64-unknown-linux-gnu: 80 tests/four doctests PASS | [M9j](API-CONTRACTS.md#original-inventory-acceptance-and-review-limits) local native workspace freeze; not this hosted lane |
| Windows 2025 native | Same nine crates, native runner host target | Hosted Rust 1.99, x86_64-pc-windows-msvc: 80 tests/four doctests PASS | Separate authorized local Windows GNU Rust 1.88 run: 80/four PASS; not local MSVC |
| macOS 15 native | Same nine crates, native runner host target | Hosted Rust 1.99, aarch64-apple-darwin: 80 tests/four doctests PASS | No x86_64 macOS qualification claimed |
| Ubuntu 24.04 Firefox | Release Todo/Search, keyed bindings, multi-island suites; SSR forms keyboard/semantic checks | Local and hosted Firefox 157.0: all four suites PASS; hosted forms report cleanup PASS | M9j: three release suites PASS on local Firefox 157/geckodriver 0.37.1; forms keyboard suite did not exist |
| Hosted Chrome baseline | Existing required baseline workflow and existing three release suites | Candidate PR run `38042614403`: all three Chrome release suites PASS; not a new local Chrome claim | M9j: three local release suites PASS on Chrome 155 |
| WebKit/Safari desktop | Manual application journeys below | Not verified; no automated WebKit lane | No qualification claimed |
| iOS Safari / Android Chrome | Real device touch, soft keyboard, viewport and zoom | Not verified; no mobile lane | No qualification claimed |
| Real OS IME | Composition, candidate selection, caret and keyed input lifetime | Not verified; manual only | Existing synthetic keyboard/selection checks do not prove OS composition |
| NVDA / VoiceOver | Spoken names, language, errors, focus and navigation | Not verified; manual only | DOM assertions do not prove assistive technology output |
| Zoom/reflow | Desktop 200%/400%, narrow viewport and mobile text scaling | Not verified; manual only | No qualification claimed |

The native package list is exactly `yetlet-core`, `yetlet-reactive`,
`yetlet-html`, `yetlet-dom`, `yetlet-dom-keyed`, `yetlet-actions`,
`yetlet-server-axum`, `yetlet-request-id` and `yetlet-session`. It uses the
actual runner host target and records `rustc -vV`; runner labels do not imply
both x86_64 and ARM coverage. Sample binaries and Unix-specific sample smokes
are outside these native portability lanes. No failing tests are skipped to
manufacture a platform PASS. Browser packaging is qualified separately on
Linux. Stable Rust is explicitly installed and observed; the unchanged
historical baseline workflow supplied the Rust 1.88 MSRV check. On 2026-10-12,
the maintainer selected Rust 1.99 before first publication. The local baseline
now pins 1.99.0; the new toolchain file does not override rolling stable lanes,
which explicitly set `RUSTUP_TOOLCHAIN=stable`. [Integrity preparation](QUALIFICATION-MATRIX.md#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12)
records the retained source boundaries. [Hosted rebuild preparation](https://github.com/bohewu/yetlet/blob/849c5894f73d9732db9c2ccaa765385dc9d8a819/docs/YT-020-HOSTED-REBUILD.md)
records the later authorized protected-check migration after actual Rust 1.99 CI PASS.

## Historical nine-package candidate platform and builder evidence

Earlier [private preparation and tool fixtures](#private-nine-package-preparation-and-verifier-fixtures--2026-10-11-and-12)
retain their distinct overlay sources and shared-control limits.

On **2026-10-20**, C `ce56e0814e5423782440acd2c959ca777d572cd4`
(tree `0c5fa71179741f7b52dead078ff6d8a9efc30241`) selected nine default-feature,
unpublished experimental0.1.0 packages on Rust1.99.0. Source archive SHA256
`9096cabe9a7b6d8b809c5bd337c2a8ab88f33ff19b6b4c8b59f31267c7335546`
and63 input hashes bound the immutable export; only18 existing license aliases
were materialized. Plain Git archives lack Cargo VCS metadata and use external
source bindings. Source/consumer locks and nine archive digests remain in the
[original candidate record](https://github.com/bohewu/yetlet/blob/c440c4542c888671750738a5908a667d2124db15/docs/YT-096-CANDIDATE-QUALIFICATION.md#immutable-source-and-package-evidence).
Local Git source/heading is verified; remote SHA accessibility is unverified.

| Actual environment / lane | Result | Preserved negative and limit |
|---|---|---|
| Ubuntu26.04.1 GNU local package simulation | Nine normal locked/offline Cargo package verifications; four external directory-source native consumer tests PASS (81 resolved packages, nine Yetlet/71 third-party plus consumer) | Existing cached dependencies, fresh target; no registry download/name-right proof. Optional-key metadata parser failed before tests and was corrected without product changes |
| Ubuntu24.04.5 container | Nine standalone package commands,83 native tests/four doctests PASS; core/actions/reactive/dom/dom-keyed packaged wasm libraries build PASS | Shared Ubuntu26.04 kernel7.0.0-38, not a booted Ubuntu24 guest or browser/wasm runtime suite |
| Windows x64 build26300, VS2022 Professional, Rust1.99.0 MSVC | Nine standalone archive roots eventually PASS83 native/four doctests,76 normalized files/locks preserved | Initial vcvars wrapper failed, then root-workspace offline caches failed101 for wasm-bindgen and unselected openidconnect before tests. Separate standalone core passed5, actions failed101 for inactive-target wasi; one correction expanded75 staged directories to the existing331 and resumed only eight packages. Neither failed lane is a PASS or selected inventory |
| Azure SSH, two disposable guests | Provisioning/host-key reads succeeded; **blocked before source transfer/build**, two SSH/SCP timeouts | Corrected bounded egress allowlist still timed out; no Rust/fetch/package/native/wasm result. Both groups/all12 resources removed. Review PASS accepted truthful blocked report/cleanup only |
| Azure Run Command/private Blob continuation | Actual Ubuntu24.04.5 kernel7.0.0-1017-azure; empty Cargo cache/upstream locked fetch, nine byte-identical archives,83 native/four doctests and five wasm builds PASS | Same maintainer/controller, not independently controlled trust. Managed execution and native exits0; private authenticated HTTPS Blob endpoint was public, not a Private Endpoint. Group/all8 resources and earlier groups removed; no retained VM/storage |
| GitHub-hosted rebuild39418279958/1, job118612562396 | Ubuntu24.04.5 image20261018.339.1, source/workflow=C, exact63 inputs/lock/epoch/tools, nine archives byte-identical to Linux/Windows/Azure, fetch/package exit0 and cleanup PASS | No new native/wasm/browser suite, compiled-binary reproducibility, new SBOM, authenticated attestation or SLSA level. ZIP artifact12262540945 SHA256 `afec916ec8545f9c2f45321bbd4267fdcdade11c3e24c97add4deadce4526126`; local bytes/logs saved before10-27 expiry |
| Firefox158/geckodriver0.37.1 | Source-reconciled Todo/Search debug workflow PASS on Ubuntu26.04.1,133 source entries/eight assets, digest2b8ea18c1298900c | Browser-internal diagnostics retained; not Ubuntu24 desktop, release build, general accessibility/IME or universal Firefox support |
| Historical Chrome155.0.8059.84 consumer reuse | Original native Chrome exit0 and cleanup PASS retained | No original independent candidate-wide source manifest: exact-candidate Chrome **Not verified**, even after present source hashes match |

Owner acceptance after final report commit2118495 accepts these bounded support
and known inventory differences; it does not manufacture missing evidence or
replace the original **FAIL / release-ready NO**. Windows OIDC and repeated
Windows IME/assistive-technology waivers stay scoped. No new macOS/Windows GNU,
device/GPU/HA/durability or production support follows. Full original platform,
Azure, hosted and reconciliation Markdown is locally recoverable at
`c440c4542c888671750738a5908a667d2124db15`; raw evidence stays in
`artifacts/yt096-{candidate-preflight,platform-qualification,azure-clean-builder,github-hosted-rebuild,final-reconciliation}-20261020/`
(brace notation names separate stores, not a new path). Git does not guarantee
those raw bytes or remote history availability.

## Historical two-package migration and Style source qualification

The **2026-10-30 YT-211** migration passed169 accepted base tests (168 initial
plus corrected environment-only OpenSSL gate),18 Browser tests,192 root example
tests,11 SDK doctests and21 moved-example tests. Nine existing external-service
cases remained ignored. Exactly two normal final archives and a native/wasm
directory-source consumer passed; importing native Auth on wasm32 was refused.
Core/default HTML graphs contained only the base SDK; broad wasm feature unions
excluded native security/provider stacks. These are local migration inputs,
not selected/frozen release artifacts or registry/Windows/live-browser proof.

Original contract P2/rate-limit omission, serde-alias rejection, compile/golden/
generator/diagnostic-feature corrections, mistaken .js/.mjs paths, stale smokes
racing a build and OpenSSL3.5.5 versus required3.5.9 failures remain failures.
First final review **FAIL/two P2** was followed by focused Auth target eligibility
and feature-name/doc corrections, then independent PASS accepting bounded Done.
Later alpha.5 publication and owner deletion of old actions alpha.4 were separate
operations in [the hosted record](ALPHA5-HOSTED-RECORD.md); PR29 closed unmerged.

The **2026-11-08 YT-243** source qualification began at
`b1b8fa08c70269cd6083b933c6a200895829ea8d`. Its local packages retained then-published
SDKalpha.6/CLI0.2.0 version labels but were **source-preflight bytes, not replacement
releases**. Linux normal two-SDK/CLI packaging and exact directory-source consumers
PASS; the nested templates/styles/Cargo.toml exclusion first failed packaging,
then storage as manifest.toml fixed the archive while emitted Cargo.toml stayed
unchanged. CLI library107 PASS/one ignored follows initial105 PASS/two failures/
one ignored; focused cloud-lock correction preserved the original production guard.

| YT-243 selected execution | Actual result and exclusion |
|---|---|
| Linux release Chrome157.0.8103.38 / Firefox159 | Both Style/Showcase smokes PASS, no-JS HTML, exact CSP/MIME/nosniff, one stylesheet, computed selection/disclosure/focus and page-realm disposal. Chrome390/768/1280; Firefox500/768/1280 (requested390 clamped). **No Firefox390 or full Firefox console/CSP-log PASS** |
| Local Windows Rust1.99.0 MSVC | View14/typed-tag doctests5/Showcase4, Style6/cloud13/scaffold1/creation3/build6/release-interface2 targeted groups PASS; groups overlap, not a disjoint suite total. Same exact SDK/CLI archives and native/wasm typed consumer, generated native HTTP/CSS PASS with Linux-identical CSS hash; browser-style creation/wasm compile was not a Windows browser runtime/bundle test |
| CI setup | First independent review found missing Windows OpenSSL configuration; corrected final setup with existing static3.5.9 prefix and two Style commands PASS6+1 locally. Hosted dynamic-image/DLL-copy branch **unexecuted**, no hosted CI claim |

Windows setup refusals (missing checksum/nested workspace, untrusted installed
compiler ACL, rejected absolute RUSTC override) remain recorded. A byte-verified
task-owned private compiler copy and normal vetted PATH succeeded without
changing original ACLs or product allowlists. Original consumer renderer-Result
and cache-permission corrections, staged-archive parent-workspace harness limit,
earlier review finding and accepted final PASS all keep their scopes. Three local
archive SHA256 values were `fb9fc6b727279ae18a5b1ae2f7d9f2131c71050a2f8c7c655fa5c8c5fc905e58`
(base), `885669fed852b77b9860e807bab6d7a7e18b8a466789cea73bf3689593eab388`
(Browser), `029419a39c47af4b7141cd6cc384c600224f5acd315b533566146e7cbc6a6491`
(CLI). Owned processes/listeners/profiles were cleaned; no installation or hosted
release ran. Original Markdown at local Git
`c440c4542c888671750738a5908a667d2124db15` preserves full YT-211/YT-243 identities;
raw stores are `target/yt211-single-package-20261030/` and
`artifacts/verification/yt243-release-qualification-20261108/`. Remote original
SHA and raw-byte availability are unverified. Hosted CI, other platforms,
Windows browser runtime and release C/signing/publication remain separate gates.

## Historical sixteen-package qualification and WASM proof

The **2026-10-26/27** YT-201–203 checkpoints prepared a historical experimental
**sixteen-package 0.1.0 source cohort**, not the currently distributed
two-crate SDK or independently versioned CLI. All four complete original reports,
including unsuccessful operations and immutable source/digest records, remain
recoverable at **local Git** `a3f0519eaf0e8d04468b6b8f238121e08dcc754e`;
this unpublished Git identity is **not verified publicly reachable**.
Raw original execution records and browser captures were retained separately
in task-specific ignored evidence directories: Git alone does not reproduce
those bytes. Current publication truth belongs to
[Release Readiness](RELEASE-READINESS.md),
[Release Publication](RELEASE-PUBLICATION.md) and the hosted records.

| Historical source and target | Actual bounded result | Preserved failure / exclusion |
|---|---|---|
| **YT-201** Ubuntu26 GNU package/consumer preparation | **16 ordinary `cargo package --offline --locked --allow-dirty` verifications PASS**, seven newly eligible 0.1.0 manifests, matching README/dual-license and source archives, and **two external directory-source native consumers** (default and selected Auth/Config/Redis integrations) exited zero with `packaged consumer OK` | Initial `core-foundation0.10.1` unavailable for locked offline vendoring: **exit101**, corrected only through a bounded locked source fetch. An archive inspector expected `git.dirty` when omitted/false and failed `KeyError` before consumers; corrected without product changes. Sixteen bootstrap archives made with `--no-verify --exclude-lockfile` were **collection only**, not verified package checks. Directory-source null registry package checksums, dirty `--allow-dirty` inputs and no immutable release source C do **not** prove crates.io upload, registered name ownership, published archives or signed release |
| **YT-202** selected native GNU and Windows MSVC external package consumer | Distinct Ubuntu24/OpenSSL3.5.9 GNU and Windows11/MSVC1.99 execution; Windows external **default and integrated consumer PASS**, with **14 selected packages** and CPU-only wgpu, Auth `session`, Config `secret-provider` and missing Memory layer refusal | Signing/Redis Windows coverage excluded; prior transfer `source_read_failed / policy_rejected` and administrative trace `UNAUTHORIZED` were real blockers, **not** retroactive transfer success. Owner separately authorized pinned Git acquisition. Initial MSVC fixture missing `futures` returned **101** before native tests; corrected with already-locked Tokio. GNU/native, package-directory and later MSVC evidence have different sources/toolchains; no cross-host HA/production/Windows Signing or Redis claim |
| **YT-203** six-package WASM/browser selection | Rust1.99 default Wasm release **build PASS**, explicit `browser` feature **compile PASS**, wasm-bindgen CLI **0.2.129**, **34 distinct** default build-graph packages; actual packaged-source DOM/Wasm mounted in **Firefox154 Linux** and **23 focused checks PASS** for native controls/reset, URL/base refusal, safe navigation, preserved DOM subtree, and CSP (including deliberately blocked inline JS) | Browser-only GPU feature was **compile only**, not WebGPU device/browser rendering. Old selected Firefox158 GPU/Chrome results were retained at different source identities, not repeated. No default native GPU, server/Auth/Redis dependencies in the selected Wasm graph, but no universal dependency, device, mobile, physical IME, assistive technology or production browser qualification. Six source archives were directory-sourced with null registry checksums, **not** actual crates.io downloads |

Selected consumer, OS, feature, source and lock determine what was actually
tested. A PASS at one selected matrix cell does not qualify all targets,
optional providers, new releases or real signed/registry consumers. Historical
license/advisory observations and their initial failures are retained in
[Release Supply Chain](RELEASE-SUPPLY-CHAIN.md#historical-license-and-advisory-qualification).

## Historical CLI contract, process and SDK resolver evidence

The following are **dated source-specific** qualifications, not current release
tests or new package/platform support. Four full originals (including their
failed observations, commands, evidence digests and Reviewer receipts) are
recoverable in **local Git** `47b0a99ffde069236eaa82ef9fac34c5c6348f3c`;
the unpublished GitHub history is **not verified reachable**. Current CLI
behavior and SDK selection instead follow the maintained
[CLI process contract](CLI-CONTRACT.md#historical-cli-source-qualification-and-process-constraints),
[CLI guide](CLI-GUIDE.md#historical-cli-source-evidence-and-operation-limits)
and retained [SDK resolution contract](CLI-GUIDE.md#sdk-resolution-yt-247).

| Historical cohort | Observed accepted scope | Original failures / limits |
|---|---|---|
| **2026-10-18 YT-102 CLI Contract audit** | Original CLI grammar and source audit mapped contracts, source-qualified tests and deferred work | **F-01 linked dotenv input outside frontend root was an observed gap**; prior relevant passing checks did not establish secure refusal. The original audit did **not** deliver an all-criteria PASS, later CLI fixes or a new Windows/browser/release qualification |
| **2026-10-20 YT-102 Linux natural completion** | Under a native Linux child/process-owner observation, a `waitid(WNOWAIT)`-checked natural exit could skip a fixed 500ms grace **only** after leader exit was observed, no cancellation pending, and no additional tracked/adopted direct children, including zombies; uncertain ownership or cancelled work retained the usual bounded grace, reap and escalation | Earlier single CLI warm build **0.565s**, later CLI **0.030/0.034/0.038s** and direct Cargo **0.026/0.025/0.023s** were not simultaneous randomized pairs; no general Vite/watch or cross-platform speedup |
| **2026-10-20 YT-102 Windows MSVC completion follow-up** | Attached-console run preserved application exit **23**, Ctrl-C/Ctrl-Break `0xc000013a`, owned cleanup of three resistant descendants, protected-tool preflight refusal **125** and unchanged lock. On one i7-10700/MSVC fixture, warm build medians: Yetlet **267.108ms**, direct Cargo **225.420ms**, difference **41.688ms** | First native suite had **four PASS/two FAIL** build cases, then two focused cases PASS after environment correction. Those are not one first-pass clean run; command compilation timing is not end-user latency. The Windows medians do not isolate a cause or justify optimization |
| **2026-11-10 YT-247 SDK Resolver Phase B (historical CLI0.2.0)** | SDK `sdk-compat-v1` source/registry policy was implemented for alpha.6/alpha.7, separate from the immutable CLI build SDK/snapshot; offline requires both CLI metadata and Cargo caches, forbids transport, and online complete-set refresh fails closed. Creates exact registry-pinned manifests, app `Cargo.lock` and `.yetlet/sdk-resolution.json` in private staging before no-replace publication. Linux suite **182 PASS/8 ignored**, separate live-registry and six policy-compile lanes PASS | Local `new` binary initially refused a group-writable tool path **exit 125**, so Linux tests were not a successful binary-new journey there. Fixture/alpha.6-snapshot/permissions failures and **two original Windows staging/process Reviewer FAILs** preceded correction and independent final PASS. Windows prerequisite gaps and later distinct MSVC verification cannot be backdated; no browser, runtime or release proof follows. The original alpha.7 default and v1 policy are superseded by current **`sdk-latest-v2`** and published CLI0.4.0—do not treat this older source snapshot as current policy |

Historical Linux test counts, unexecuted Windows-only cases, compile-only
checks, real registry requests, Windows runtime and publication are different
evidence classes. No new product tests, Windows execution, browser run or
production load was performed during documentation consolidation.

## Historical Session, Auth, OIDC and cross-host evidence

These are **dated, separately sourced bounded qualifications** rather
than a production Auth/Session service certification. Complete old
YT-066B/094/113–117/124–126/139/140/175/184 reports and original
negative results are locally recoverable at Git
`0ade9c62b67fe51ecb8e9ee6edc666daed24b5c1`. Original raw
fixtures, screenshot, IdP/Chrome/browser and cloud receipts are
separately task-owned/ignored; local Git history does **not** by
itself restore them or establish public remote reachability.

| Selected evidence cohort | Actually observed / accepted | Retained failure or scope limit |
|---|---|---|
| **YT-066B** Redis Session atomic CAS, operation results, stale responses and Cookie candidate | Selected local Redis provider CAS/rotation/TTL semantics and later separately accepted **three-application, single-host/single-primary** native/Chrome qualification; original operation-result and callback-loss tests included | The **Cookie issuance candidate had independent Reviewer P1 FAIL**: **180 delayed old cookies** and real Chrome's **150-cookie quota** evicted a newer replacement/logout marker. Smaller focused PASS/57-cookie cases were inadequate; no proven protocol repair or production Cookie guarantee follows. An earlier real cleanup P2 failure was independently corrected before later provider acceptance |
| **YT-094** optional Board OIDC, shared local SQLite and Valkey Session | Actual real-IdP loopback callback after WAF adjustment, native policy/CSRF/PKCE tests, separate **two-Board same-file SQLite** (conflict 303/409, held-writer 503), separate **Valkey** shared Session and later app-configurable 1800s default TTL (1s–30day) | Initial WAF/dependency-warning/setup and short 60s fixture were different checkpoints; two local SQLite connections are not distributed SQL; 303/409 success cannot exclude a subsequent Busy503 before version comparison, and token/cookie expiries are not equivalent |
| **YT-113–117** native Auth typed principal, required/optional scope, Memory Session bridge and rate limiting | Source/focused tests and native Settings-example validation PASS with explicit auth-layer provenance, CSRF and bounded quota | Original Clippy, workspace-root and input/setup failures retained; these opt-in features are not a mandatory Core Auth service, general OIDC, automatic Request admission or per-cluster limiter. A rate-denial 429 is not provider-overload 503 |
| **YT-124–126** same-host multi-process Valkey Auth and Admin adoption | Two actual Admin/Board processes exercise login A/resolution B, current shared local policy/account disable and logout/retired-identity refusal, with review PASS | Original harness default overlay and focused method-layer errors corrected later; old Session/policy errors fail 503, not anonymous allow. Local file-policy trust, restart and SQL commit races remain; no multi-host policy availability or distributed transaction proof |
| **YT-139/140** shared authority and OIDC static loopback proxy | Native focused actual trust/epoch/Host/Origin/callback/nonce/one-shot state tests PASS, independently reviewed at source and local fixture identity | Startup snapshots not automatically refreshed, no IdP account-disable feed, untrusted Forwarded ignored; no public HTTPS/multi-host OIDC/callback service. Old Session epoch cannot be restored to revive authority; pre-SQL revocation vs commit race remains |
| **YT-175** Session burst/overload policy | Independent **contract/recommendation** review PASS, no runtime change; 4 active/8 total local Session operations, 80ms wait, 400ms **per API operation** deadline | Not an HTTP deadline or cluster quota; no implemented new HTTP admission, automatic retry, distributed rate budget or guaranteed Retry-After |
| **YT-184** five selected Azure cross-host generations | Independent final review accepted **composed G4 Auth** and **G5 Valkey/PostgreSQL outage, credential epoch and orderly authority-VM restart** evidence; all five owned groups cleaned | G1/G2/G3 failed and G4 first recovery status lacked wire evidence; G5 initially returned **A503/B503** after Valkey start, then **A200/B200** with valid data. No complete single-run PASS, arbitrary restoration, public IdP/HA or customer reconnect-SLO inference |

Qualified browser, native CLI and workstation checks cannot be
silently reused at a different code SHA, IdP snapshot or host.
Different state failures, cookie queue/clock/revocation races and
cross-host persistence are not equivalent. See
[Provider Operations](PROVIDER-OPERATIONS.md#historical-session-auth-and-oidc-contracts-and-failures)
for current operation boundaries and
[Deployment Profiles](PRODUCTION-DEPLOYMENT-PROFILES.md#historical-shared-authority-and-azure-persistence-qualification)
for the selected cloud evidence.

## Historical Windows native, browser and CLI evidence

These are **four different historical, bounded source/host verifications**, not a
single current-version platform matrix or a fresh test run. The complete original
documents, failure receipts and their exact heading identities remain in **local
Git** `b6be7230194a36f87cbefa50ca5c7cc73ae04f3c`; public GitHub reachability
of that unpublished ancestry is **not yet verified**. Current CLI release
qualification and SDK/CLI version identity are separately maintained in
[Release Readiness](RELEASE-READINESS.md) and the
[CLI Guide](CLI-GUIDE.md). The earlier YT-065 lane matrix above does not
silently gain a physical Windows IME or spoken-AT observation.

| Dated cohort | Actual source-specific acceptance | Important original failures and non-claims |
|---|---|---|
| **2026-10-19 YT-094 Admin/Board Windows native MSVC**, `4bb8f08` | 820 source files matched a frozen archive. Protected CLI rebuilt; Admin build/typecheck, **12 Admin + 15 Board tests PASS**; real native HTTP exercised escaped Chinese text, strict CSP/CSS and persisted history across restart with retired sessions rejected. Two attached-console Ctrl-C drains returned **exit 0**, with owned process and port closure verified | Initial fnm reparse PATH spawn **125**, offline Cargo cache/index gaps, PowerShell quoting and too-early handle/exit assertions were setup/harness observations corrected separately; no Windows browser, physical IME, spoken accessibility, production auth/HA or shared-session claim from this native slice |
| **2026-10-19 YT-094 Windows WASM/browser and returned desktop evidence**, `a402b6f` | Source identity is a **20-file application overlay**, not a full Git archive. Admin **12**, Board **17**, reducer **3** tests and focused Clippy passed; actual Windows browser loaded WASM/JS/CSS, retained local drafts, used one POST for save and preserved intentional **409 stale-draft / 422 invalid-state / 401 retired-session** rejections and restart history. Physical Bopomofo, keyboard/navigation and Narrator speech were **operator-attested**, with stop/exit-0 screenshots and retained evidence hashes | Physical composition and spoken audio were **not independently recorded**. The original Windows child Reviewer **FAIL** for missing numeric exit remained FAIL despite later image corroboration; its configured Astra versus observed Sol model mismatch remains uncorrected. A **separate source-side reconciliation Reviewer PASS** confirms returned hashes/screenshots, not the earlier reviewer or model route. The mint-game GNU CLI host is distinct from the MSVC Admin run; no other-browser/mobile or production OIDC claim. The original dated `In progress` status does not override the later [Admin reference closeout](../examples/admin-dashboard/README.md) |
| **2026-10-18 YT-102 Windows GNU real Rust/Vite pair**, `a883469` bounded overlay | **393** source-hash matched inputs and one actual ignored native integration test executed **six** serialized Vite8.3.4/Rust console cases: Ctrl-C/Break, real frontend and backend **exit 23**, occupied port5173 **exit 125**, and disabled auto-start with Rust exit23. Both owned trees and unrelated listener isolation checked before outer safety cleanup; Reviewer PASS | No installed dependency by the CLI; no native MSVC pair runtime, authenticated Windows browser HMR or general publication/HA guarantee. Native cancellation status `0xc000013a` is not interchangeable with a normal successful-exit 0 |
| **2026-11-10 YT-247 Windows MSVC CLI SDK version resolver**, PR#69 merge `9ec1ae6` | Exact protected compiler/OpenSSL/source inputs; first library run **102 PASS / 6 FAIL / 3 ignored**, then **six focused PASS** after fixture/test expectation correction, plus **2 release-interface + 3 Windows-new integration PASS**. Real CLI `new` verified exact registry SDK pins and Cargo/Style locks, online refresh, offline-cache without metadata calls, expected existing-destination/empty-cache errors. Cleanup and retained evidence digests reviewed independently | **Not** a single fresh all-library-green run; original TempDir ACL/console/Style assertion/setup errors and advisory Session `fail/source_unproven` remain evidence. Only an existing test assertion expected `YETLET_STYLE_DEV=1`; production behavior did not change. No Windows server/browser/Vite/GNU equivalence or runtime certification |

These historical successes do **not** turn distinct Windows GNU/MSVC toolchains,
a source-copy overlay, operator testimony or an accepted browser fixture into
cross-platform coverage. Preserve the original rejected responses and failed
Reviewer/test populations rather than converting them into successful cases.
Use the [Admin example](../examples/admin-dashboard/README.md), the
[CLI process contract](CLI-CONTRACT.md) and the
[SDK resolver contract](CLI-GUIDE.md#sdk-resolution-yt-247)
for current application and CLI decision boundaries.

## Historical operations, capacity and recovery cohorts

These measurements and desktop checks are **dated, source-/host-specific**
qualifications, not a current supported-hardware matrix, SLO, readiness
certificate or transferable production capacity. Complete reports and original
FAIL/blocked observations are recoverable in **local Git**
`99906e421c0510231291bf527e2cbd2e7ad0ffda` (the local unpublished
history is not verified reachable through GitHub). Raw results, screenshots,
logs and host-side evidence were separately retained under their original
task-owned/ignored evidence paths and **are not thereby Git-recoverable**.
For current app behavior, use the [Starter example](../examples/production-starter/README.md),
[Lab Board example](../examples/lab-operations-board/README.md), and
[Production Readiness](PRODUCTION-READINESS-PHASE2.md).

| Distinct original source and test fixture | Actual bounded observation | Failed or excluded evidence that must not disappear |
|---|---|---|
| **YT-074 Board desktop, native/AT and load qualification** | Actual selected Firefox/Linux Fcitx CJK input and Orca speech/live-status, separate host/browser/Windows evidence, persisted Board version and history, and bounded desktop/load acceptance. The corrected status message uses a semantic live container and a message child admitted through Orca's real event path | Initial Orca/AT-SPI browser-registration and speech tests produced startup/key echo but **not Board content**; subsequent successful real application journey is distinct. Early caption/table keyboard paths, setup/prototype and original load/host failures remain historical. Screenshots and operator observations do not certify every screen reader/mobile OS or public capacity |
| **YT-129 Starter local load/soak/recovery** | Original application schema-1/debug/tmpfs and co-located, closed-loop client fixture generated **42,043 requests** and separately bounded accepted lower-load/recovery results | The **eight-client point FAILED**, with **1.514% rejected** and **341.73% CPU**. It is excluded from the passing range; source, storage, traffic shape, response status, worker permit and observed CPU are not present-day release limits or a 45s-to-long-soak guarantee |
| **YT-134 local persistent-release Starter profile** | Separate release/schema-2/persistent storage fixture: **23,549** workload requests, **17,829 HTTP200 / 1,945 HTTP201 / 3,775 HTTP503**. Four-worker step was the highest *tested passing* point at approximately **452.48 req/s**; final data contained exactly **1,965 rows = 20 seeds + 1,945 acknowledged creates**, with SQLite integrity check `ok` | Unpaced eight-worker step **FAILED** with **3,775 workload 503** (27.5568%) and readiness rejections, not a hard capacity threshold. Later lower-load recovery passed; no read of a 503 or timeout proves a failed/rolled-back SQL write, no cross-environment equivalence |
| **YT-134 separate optional Azure reference** | A private operator/Run Command/Blob-selected ephemeral Azure disk/VM trial generated **34,329** workload requests: **25,059 HTTP200 / 2,889 HTTP201 / 6,381 HTTP503**. Reconciled **2,909 rows = 20 seeds + 2,889 acknowledged creates**; subsequent lower-load/reboot and explicit resource cleanup were bounded to this fixture | Original public-SSH bootstrap proposal/attempt was stopped and replaced by owner-authorized private access. **Four-worker step FAILED readiness** despite its accepted load-side population; **eight-worker step FAILED** with **6,381 HTTP503**. Reboot/cleanup and nonproduction device/OS provenance do not imply Azure service HA, durable SLO or general deployment guidance |
| **YT-135 current schema-2 recovery** | Separate stop/withdraw, source check, manifest/backup, intentionally rejected invalid restoration and fresh-instance schema/data/API verification; observed **113.14 ms** local withdrawal-to-complete-eligibility **RTO proxy** | Not a live traffic switch, general business RPO/RTO, host loss, power-loss durability or production DR; earlier v1/schema-1 B2 restoration figures cannot be substituted |
| **YT-141 controlled-arrival read-only API fixture** | One explicit source/host client arrival profile produced **14,006** actual exact-body HTTP200 results at selected 100/s and 500/s bands; measured p50/p95 values of the bounded fixture remain in original evidence | These are not a whole-app, SSR/Session, storage durability, current-version or network-wide p95 promise; scheduled arrivals and observed request counts differ, and future rates/hosts require new proof |

The later **owner-amended YT-074 62-minute staggered Board run** selected
five client-cycle offsets (25 clients at 0.5 requests/second each, ten SSE
readers), 120s warmup, three 600s rounds and 1,800s soak with SQLite FULL.
It **PASSED** its selected thresholds with **45,000 measured requests and
zero unexpected HTTP/SSE errors**. Exactly **9,300 committed writes** advanced
the seeded 90,000 history records to **99,300** with SQLite integrity `ok`.
That replaces neither the original **synchronized** full attempt, whose
write p95 **548.299ms exceeded the 500ms threshold and FAILED** before
later rounds/soak, nor the separate earlier host/desktop profiles. The
staggered arrival pattern did not prove maximum throughput or immunity to
synchronized bursts, other load distributions or production capacity.

In every cohort, count HTTP response/error populations, dropped arrivals,
source writes, started/unfinished blocking SQL operations, independent probes
and retry attempts separately. Neither an HTTP200 nor a ready probe proves
durable business writes. A failed step is still a failed step after a passing
recovery. Cross-host cache, CPU, toolchain, client pacing and connection
conditions are part of the result identity; no undocumented default capacity
or production operating threshold is inferred.

## Historical performance and capacity evidence (bounded)

The original research cohorts below are **dated, source- and machine-specific**
observations; they do not establish current-version performance, minimum
hardware, production capacity, a cloud SLA, regression thresholds or
cross-platform equivalence. They remain recoverable from local Git source
`1d0eabcb99eeeb681795c1db3e27b2a5b47e0bcb`, including full profiles,
raw measurements, original failures, comparison rules, independent reviews and
cleanup. That local source SHA is **not yet verified as publicly reachable**.
Current build/fixture configuration belongs to the
[SSR measurement example](../examples/ssr-performance-baselines/README.md)
and [Axum/Yetlet workload example](../examples/axum-workload-comparison/README.md).
The retained [YT-141 controlled-arrival reference](QUALIFICATION-MATRIX.md#historical-operations-capacity-and-recovery-cohorts),
[YT-074 Board qualification](../examples/lab-operations-board/README.md#historical-board-desktop-and-sqlite-concurrency-evidence) and
[YT-175 admission-policy contract](PROVIDER-OPERATIONS.md#provider-overload-and-quota-distinctions)
retain their separate authorities.

| Historical scope | Verified observations and important limits |
|---|---|
| YT-074 desktop and synchronized Board loads | Separate desktop/cross-environment comparison and fsync-stall observations were **not** a portable throughput baseline. Source, storage, platform and synchronized versus staggered conditions matter; failed and noncomparable captures remain at their original identities. The retained [desktop acceptance](../examples/lab-operations-board/README.md#historical-board-desktop-and-sqlite-concurrency-evidence) and [Board qualification](../examples/lab-operations-board/README.md#historical-board-desktop-and-sqlite-concurrency-evidence) remain distinct |
| YT-141–144 API/saturation/router/PostgreSQL | YT-141's read-only controlled-arrival reference had 14,006 valid 200s, no drops and 1/2 ms p50/p95 only for its fixture. YT-142's first **1,000/s point FAILED** with 5 SQL-admission 503s among 10,001 requests; immediate 250/s recovery passed 7,501 valid 200s. YT-143 corrected only an unstarted comparator repetition; its SSR 4,000/s pair was **noncomparable** after a native Axum drop. YT-144's four isolated PostgreSQL pairs had 100,007 measured valid 200s; rejected preload/observer/lookup attempts and scope boundaries remain historical |
| YT-145–160 SSR allocation/latency attribution | Fixed 8/128/1,024-row SSR output baselines and independent escaped-output hashes separate constructing a View, body/document serialization, combined costs and full HTTP p95. Original escaping accounted for 85.70% of measured *document allocation calls* in one YT-146 instrumented profile, **not** of total CPU. Private escape-into and URL-prefix changes reduced selected allocations while held/peak memory did not automatically fall. First-child preallocation (YT-152) was **rejected** on tradeoffs. Changes in timings/frequency/worker CPU cannot uniquely identify the cause of earlier HTTP p95 observations; no general speedup claim |
| YT-161–164 benchmark isolation and transport | The attempted constrained execution profile **was not admitted** in YT-161; controlled-frequency evidence did not result. YT-162 retains 39,977 valid HTTP 200s and **27 dropped arrivals**, with only the first platform pair comparable. YT-163 attributes those 27 drops to a short exposure with full outstanding HTTP brackets but cannot identify a unique origin. YT-164 observed lower p95 under direct private HTTP than SSH forwarding in two bounded pairs; this is a transport/context effect, **not** proof of a faster Yetlet product |
| YT-165–169 finite/sustained capacity | Short isolated 4k and selected 5k/6k points were clean in their specific profiles, while 7k encountered pressure and a prior 8k point was saturated. A **6k/s sustained 300-second run FAILED** with 34,302 dropped arrivals (reported HTTP p95 63.33 ms). Separate **5k/s 300-second** and **5k/s 1,800-second** runs completed with 1,500,001 and 9,000,001 valid 200 responses respectively and zero drops; different allocations and evidence capture prevent a controlled duration-only comparison or production SLO |
| YT-170–174 synthetic Auth/Session/PostgreSQL | The synthetic sign-in is a **preverified fixture identity**, not real credential/OIDC verification; reads and private responses retain app-owned boundaries. YT-170's JSON/SSR fixed exposure had 2,000/2,001 valid 200s at 100/s each. YT-171 retained one Session Busy at JSON 500/s and 19 at SSR 2,000/s, followed by separate passing recoveries. YT-172's 648 read calls split into **552 valid / 96 Busy**. YT-173's bounded HTTP trace correlated 12 Busy outcomes in a 12,550-request cohort; passive host sampling in YT-174 does **not** establish an exact causal timing stage. Cleanup, failure and redaction policies remain source-bound |

**Interpretation:** Counted logical allocation bytes are not whole-process RSS;
nearest-rank HTTP p95 is not necessarily a k6 interpolated percentile; a valid
HTTP status does not establish completed SQL business writes; drops, 503 and
Session Busy must remain failed populations, not disappear into a success-only
denominator. Connection/SSH/frequency, CPU affinity, working sets, tool settings,
VM limits and benchmark fixture are part of the result identity. Unchanged
test/source artifacts are **not rerun** by documentation consolidation.

For application-owned outer HTTP admission and current operational priorities,
use the [workload capacity-selection guide](../examples/axum-workload-comparison/README.md#http-admission-capacity-selection)
and the [bounded cap-four/eight/sixteen results](#opt-in-http-admission-4816-bounded-cohorts),
not a universal default or capacity guarantee from historical throughput. The
[Starter production-readiness guide](PRODUCTION-READINESS-PHASE2.md) keeps the
separately accepted local YT-129 load/recovery and error semantics. Authentication,
authorization, CSRF, strict CSP, durable write ordering and distributed admission
were not changed by these measurements.

## Early provider and HTTP attribution evidence

These dated local research proofs are **not** new hosted qualification,
production capacity, cross-host Session recovery or durable write
guarantees. Complete source/measurement history is in **local Git**
`c3478b7f4e0c929a94ccbf21e05217535553a73d`; remote GitHub availability is unverified.

| Research | Verified bounded observation | Exclusions and historical failures |
|---|---|---|
| Redis async provider | Two real CAS barrier stalls were failed and cleaned; independently corrected test-only barrier followed by five actual Redis integration test passes | Previously accepted cancellation/drain and Cookie/browser tests not rerun; no production persistence or cross-host RPO/RTO |
| Provider-only admission | 648 attempts: 552 validated / 96 Busy; eight recovery reads valid; 72 waves with four active/eight total | No fair queue, exact 1ms scheduling, 80ms expiry or universal capacity ceiling |
| Same-process HTTP Session | 12 SSR Busy/503 rejections, four active/four pending owners; rejected before SQL | Different sixteen-VU arrivals; clock and resource observer interference, no proven unique cause or global SLO |
| Native HTTP/SSR | App-owned streaming can expose first body sooner but finish later; 100ms response-creation timeout can return 504 | No rollback of started work, whole-body deadline, browser paint, TLS/proxy flushing or production latency guarantee |
## Experimental GPU, browser and Physics Lab evidence

These are separate, dated and **bounded** research scopes, not
current universal GPU support or a 60 FPS/SLA criterion.

| Scope | Actual evidence | Explicit exclusions |
|---|---|---|
| Native scene (Oct 12) | Linux RTX4090/Vulkan wgpu27 offscreen 10k/50k/100k static instances with readback proof; 30 frames each, median **0.033760/0.054286/0.083525ms** encode+submit+completion wall | No browser GPU, matched DOM/CPU, GPU timestamp, displayed FPS or actual VRAM consumption |
| Browser Canvas2D/WebGPU (Oct 14) | Firefox158/X11/Intel Vulkan: original 50k screenshot-alignment attempt **FAIL**, separately repaired harness, then **24/24 PASS**, 2160 measured samples, 126 known-pixel screenshots | Different API completion semantics; no useful WebGPU speedup demonstrated, default Firefox/Chrome/Windows/macOS/mobile/NVIDIA not qualified |
| Experimental GPU API | Independently reviewed typed CPU SceneError and logical first-retirement bool, source-specific native/Wasm/Firefox checks | Async Wasm borrows need separate settlement; no stable ABI or platform support promise |
| 3D physics feasibility (Oct 26) | Source/registry comparison of wgpu27/Rapier3D0.36.1 and bounded app-owned design | **No joint Wasm build, browser 3D run or FPS measurement in that study**; later runnable sample has distinct evidence |

Historical source and original failure receipts remain in local Git
dfb27dc816c452e8033ff24e2d853f88a451761a; public SHA reachability is not verified.
## Opt-in HTTP admission 4/8/16 bounded cohorts

These dated **2026-10-25** runs apply to one isolated Auth/Valkey/
PostgreSQL reference, not framework defaults, future CLI versions,
distributed capacity, a physical ceiling or a production SLO.
JSON observe offered 500/s × 5s and SSR 2,000/s × 5s; each
ran a sixteen-request burst and eight serial read recoveries
per JSON and SSR mode. Every checked **503 is a failed request**.

| HTTP cap | Attempted | Valid 200 | Failed checked 503 | Outer HTTP-only / provider Busy / QueueTimeout |
|---:|---:|---:|---:|---|
| 4 | 12,550 | 12,452 | **98** | **98 / 0 / 0** |
| 8 | 12,549 | 12,475 | **74** | **74 / 0 / 0** |
| 16 | 12,552 | 12,540 | **12** | **7 / 5 / 0** |

Cap four rejected 98 before the Session provider, compared
with the separate earlier gate-off cohort's nine Session Busy;
this is a different source/build/oracle, **not an improvement
in successful throughput**. Cap eight had 24 fewer failures
and 23 more successes with **one fewer boundary attempt**;
burst p95 and failed client tails could worsen. Cap sixteen
added three boundary attempts versus cap eight and had seven
outer-only and **five genuine provider Busy** (two JSON burst,
three SSR observe), not twelve identical gate rejections.
None had observed QueueTimeout, SQL errors, denials, generator
drops, invalid 200, incomplete calls or capture overflow.
Both mode-specific eight-read recoveries and cleanup passed
for each run without making its failed requests successes.

Source/feature/startup-cap, selected route/valid GET and exact
private body/header 200/503 oracles plus complete joins are
necessary to attribute outer-only rejections. **No HTTP gate
admit/release/reason event** exists in the trace. Raw
nearest-rank versus native k6 interpolated p95, all/valid/
failed populations, separate Python recovery clocks and
distinct client/server timestamps are noninterchangeable.
These were sequential, not randomized comparisons: arrival,
warming, host/observer changes and caps themselves are confounded.

Original historical negative evidence: cap four needed a new
binary and strict caller oracle; caps eight/sixteen reused the
exact accepted binary/caller. Cap sixteen had one **pre-start
receipt-writing harness FAIL**, referencing `enforcement`
instead of `enforcement_status`; the limited correction
occurred **before startup/measurement**, with no repeated
exposure or runtime test failure. Earlier Session/Auth/CSRF/
browser/live backend proofs were retained, not rerun.
See the [current HTTP gate contract](HTTP-PIPELINE.md#opt-in-application-http-admission-contract)
and [sample's complete p95 and choice table](../examples/axum-workload-comparison/README.md#http-admission-capacity-selection).
Full original studies, failure receipts, raw timing and reviewer
findings survive at **local Git** `619b0d9d1e05c86ea0193efb5fc91a198ec930a8`, but fixed GitHub
historical links are **not yet verified publicly accessible**.
## Workflow source and activation

[Qualification workflow](../.github/workflows/qualification.yml) has only
`schedule` (Wednesday 03:17 UTC) and `workflow_dispatch`. It grants only
`contents: read`, uses full-SHA GitHub-owned checkout/upload
actions, disables persisted checkout credentials, cancels superseded runs,
limits native parallelism to two, and limits native/Firefox jobs to 35/60
minutes. It neither supplies secrets nor requests token write or id-token
permissions. This schedule/manual behavior remains unchanged. Remote protection
now requires `Rust 1.99 MSRV` and `Stable native, samples, and Chrome`, after
both first-candidate jobs passed in run `38193149046`. Historical 1.88 check/run
identities remain accurate. New candidate updates require new actual required CI;
the migration neither weakens protection nor relabels historical qualification.

GitHub requires this workflow on the default branch for activation; schedules
run there and may be delayed or dropped. Manual dispatch also depends on the
default-branch workflow. Local uncommitted YAML cannot establish hosted PASS.
The maintainer authorized that delivery: PR #1 was merged without bypass and run `38043565238` completed successfully. The workflow is now on the default branch; future schedule execution is not preclaimed. No repository visibility or branch-protection setting was changed by this delivery. See
[GitHub trigger documentation](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows).

The [official Ubuntu2404 runner image inventory](https://github.com/actions/runner-images/blob/main/images/ubuntu/Ubuntu2404-Readme.md)
documents installed Firefox/geckodriver and the `GECKOWEBDRIVER` directory.
The Firefox job explicitly requires that directory's executable, puts it on
PATH, and checks Firefox plus both versions before builds. Missing tools fail
the job; no floating driver download or silent browser substitution occurs.
Image software can change: each run records `ImageOS`, `ImageVersion`, source
SHA, Rust host/tool versions, Rust xtask and browser/driver versions. Current
observations, rather than this inventory, establish the tested versions.

Artifacts retain only named qualification build/test/identity logs and the
sanitized forms JSON report for seven days, with unique lane/run/attempt names
and `always()` upload. No entire `target` tree, profiles, raw page state,
cookies or arbitrary environment dump is uploaded. Cargo and xtask failures
propagate through Bash `set -euo pipefail` even when stderr/stdout feed `tee`.
Job timeout/cancellation may prevent normal harness cleanup; the hosted runner
is disposable, and an interrupted job is not a successful qualification.

## One bounded local sequence (main session only)

Use prebuilt release assets from `cargo xtask build browser --release`,
`cargo xtask build keyed-bindings --release` and
`cargo xtask build multi-island-dashboard --release`, plus release native binaries
`yetlet-todo-server`, `yetlet-multi-island-dashboard` and
`yetlet-forms-validation`. Require Firefox and geckodriver on PATH and record
their versions together with Rust and source identity. Run each command once:

```sh
cargo xtask smoke browser-firefox --profile release
cargo xtask smoke keyed-bindings-firefox --profile release
cargo xtask smoke multi-island-dashboard-firefox --profile release
cargo xtask smoke forms-accessibility-firefox --profile release
```

Stop at failure, classify it before repair, and retain exact exit statuses.
The executor does not run this sequence. Main-session logs belong under
`target/m10-remaining/` or `target/qualification/`. Existing build helpers and
suites retain their established output locations and behavior. Main session
must check teardown for those existing suites; this task does not rewrite them.

The new forms command defaults to `release` and also accepts `--profile debug`.
It owns one ephemeral loopback server, one headless WebDriver session and its
POSIX process groups; it tracks startup processes before waiting, closes the
session, stops/reaps both groups with bounded existing helpers, and removes its
temporary tree. Its fixed report is
`target/qualification/forms-accessibility-firefox.json`. It uses real W3C
Tab/Enter/text key actions, without assigning focus or input via JavaScript.
DOM reads assert name/email/submit traversal, both label/control associations,
first-invalid-field autofocus, error text/role/descriptions, root `lang=en`
on initial/error/success documents, Unicode input preservation, and no-state
success/refresh/empty-form behavior. A separate no-redirect HTTP observation
checks 303, Location, no-store and no cookie. Unicode key actions exercise text
entry, not a real operating-system IME.

Parse the JSON only after exit: require command exit 0, `status=pass`,
`cleanup=pass`, `stage=complete`, all five declared check groups, a browser
version, and terminated owned process exit codes. Failure reports retain
bounded fixed stage names, exception types, completed groups and process exit
codes; raw process output and exception messages are discarded. `not_verified`
must retain real IME, screen-reader and other untested scopes even after PASS.
Cleanup failure prevents PASS. No screenshots or visible browser are needed.

## Optional future manual qualification runbook (currently Not verified)

The native [local signing/key CLI evidence](#local-signing-key-cli-and-azure-evidence--2026-10-18)
and [custody test-key drill](#custody-and-local-rotation-evidence--2026-10-27-and-28)
retain separate execution and acceptance scopes.

Record tester/date, source/build identity, OS/architecture, browser version,
assistive technology or IME version/settings, profile and observed result for
each attempt. Use synthetic text and loopback fixtures; retain only redacted
notes. Record PASS, FAIL or Not verified per journey, with reproduction steps
for failures. A blank record or successful automation is not manual evidence.

1. **Keyboard and forms:** Open `/form` in the forms sample. Traverse with Tab
   and Shift+Tab; confirm name, email, submit order and visible focus. Submit
   `x`/`invalid` with Enter; confirm first-field focus and understandable linked
   errors. Correct only name, submit and confirm email focus. Correct email,
   submit, refresh success, and return to a blank form. Check English language,
   labels and no saved data. Repeat relevant Todo/Search and dashboard controls
   without a pointer; inspect focus after keyed move, remove, disposal/remount.
2. **Real IME and caret:** Enable a real Chinese/Japanese/Korean OS IME on
   Linux, Windows and macOS where available. Compose multi-character input,
   choose candidates, cancel composition and insert in the middle of existing
   text. Observe caret/selection and committed text. In the keyed fixture,
   reorder while editing and verify ownership/focus/text/caret survive; test
   removal/disposal. Record engine and composition behavior explicitly; typed
   Unicode or dispatched synthetic composition events cannot satisfy this cell.
3. **NVDA and VoiceOver:** With Windows NVDA in Firefox/Chrome and macOS
   VoiceOver in Safari, read headings/landmarks and move through controls.
   Confirm spoken field names, English language, required/invalid state,
   linked error text and first-error focus after each server navigation. Confirm
   success content, keyboard operation of islands and absence of duplicate
   announcements after remount. Record actual speech/behavior and settings;
   automated label checks alone cannot satisfy this cell.
4. **WebKit and mobile:** Run SSR forms, Todo local/remote commands, Search
   activation and dashboard independent activation/disposal on desktop Safari,
   real iOS Safari and real Android Chrome. Check touch targets, soft keyboard,
   rotation, text entry, errors, focus and server fallback. Record device/OS
   versions and browser console/CSP outcomes separately. A resized desktop
   Firefox window does not prove a mobile browser.
5. **Zoom/reflow:** At 200% and 400% desktop zoom and a 320 CSS-pixel viewport,
   inspect clipped content, horizontal scrolling, control reachability and
   focus visibility. On mobile increase OS/browser text size and repeat forms
   errors and islands. Record observed defects and exact settings. No general
   WCAG certification follows from these bounded journeys.

## Bounded task acceptance and optional future coverage

The local/browser and hosted run IDs, terminal results, observed versions/architectures and artifact references are recorded in [M10f verification](QUALIFICATION-MATRIX.md#m10-native-browser-and-delivery-reconciliation). The authorized workflow merge and actual hosted run reached terminal success. YT-065 is therefore Done at the bounded automated desktop/native/browser scope.

The manual cells above remain explicitly Not verified and optional. They do not block this bounded task, do not become supported-platform or accessibility-conformance claims, and must not be inferred from typed Unicode or DOM assertions. Supported platform/version policy, real-device/assistive-technology ownership and any application-specific manual evidence remain future decisions. YT-020 release qualification remains Blocked. YT-066 is a separate backlog item.

## Private nine-package preparation and verifier fixtures — 2026-10-11 and 12

The original nine0.1.0 candidates were Core/Actions/Reactive/HTML/DOM/DOM-keyed/
Server Axum/Request-ID/Session, with samples and isolated Redis excluded. On
2026-10-11 Cargo1.88 offline batch packaging failed101 on unpublished core
resolution after creating two archives. Cargo1.99 native temporary-registry
batch packaging with Rust1.88 compiler passed normal extracted verification
for all nine. Both license texts, normalized versions, compatible dependencies
and source bytes matched. cargo-deny0.20.2 license/source and cargo-cyclonedx0.5.9
default-workspace/all-target metadata matched nine inventories. Dev-only, sysroot,
native/system, JS and separately built feature/platform coverage were omitted.
Nine name404 observations did not reserve or qualify names. Private PVR404,
secret-scanning disabled404 and code-scanning unavailable403 remained limitations.
Raw proof is under `target/yt020-private-prep-20261011/`.

PR5 candidate `c7f21b4d620a1b6836fb52a94bf19434af059368` and merge
`633db585f3173f82c512689a09fd9a2065db6ffe` shared tree
`d9d6c4abe08a25ca753647a9149d8e16a80bffa9`. PR38144070219 and master38145022739
required checks passed at those sources. The subsequent owner-selected Rust1.99
decision unified development/MSRV/package metadata before publication. Protected
Rust1.88 check migration required separately authorized delivery and actual
Rust1.99 CI. A changed local YAML alone did not change the hosted gate.

The 2026-10-12 two-host recipe used base633db585 plus recorded Cargo/toolchain
overlays, with source archive SHA-256
`cbcfe22c8008ac66fb0374c5aee566a436d7a1241ab143cb3ee9cc966cb11b00`,
Cargo.toml `10857511488c65b515d4953360b101b628df94d07b9dd131507ad6e4b3ea62df`,
toolchain `b2c19d86958ed6810741488db5cd352856f303c53bcf0ea774e72f94ee4d201e`,
lock `7260eef21ce05abbe90ebccc4b255b3da07cdf5cd0acac51fdbb90ee7b3d4b57`
and54-input mapping `8e5799c319d626812daca22c90feba41a41d23d07ecd642af4d6d3506dc7afb7`.
Both hosts observed Rust1.99.0 `b940084d7`/Cargo1.99.0 `5f94df478`.
They separately fetched locked dependencies and normally packaged nine archives
with fresh host-local state,18 materialized license aliases, no Git/VCS metadata,
default features and epoch1791726849. All nine raw archives matched in size/hash.

Ubuntu GNU and Windows GNU Job `wc_job_hW6_8gK6yQeL2dPx` passed extracted
compilation. Windows used task-local w64devkit and self-contained host linking.
The old Windows checkout3128d235 stayed unchanged. Initial Rust1.88 Job
`wc_job_7LkjxkmPZbcBLYaE` failed101 at host `-lgcc_eh`. Rust1.99 installation
Job `wc_job_r7qfbREmB0pL4ee0` installed binaries then failed1 at rustup self-update.
The final build used verified installed binaries directly. No overlapping retries
or global toolchain change occurred. The WebCodex `validation_source_unproven`
warning remained because its ledger observed a different branch from the
archive/overlay. Explicit54-input bindings supplied the build identity.
Shared owner/agent control prevented an independently controlled trust claim.

Eighteen CycloneDX1.5 inventories matched target-filtered normal/build metadata,
with Linux counts1/12/1/2/2/2/59/60/69 and Windows1/12/1/2/2/2/58/59/68 in the
original candidate order. Generation ran on Ubuntu for both targets. Two unsigned
SLSA-v1 drafts used `urn:yetlet:build:private-cargo-source-package:v1` and local
builder URNs. The final34-artifact unsigned bundle plus manifest differed from
the preserved earlier incomplete bundle. Checksums supplied no authentication.

Public-fixture bootstrap authenticated Cosign3.1.3 via TUF7.0.1/securesystemslib1.5.1
and OpenSSL artifact-key verification. Artifact key SHA-256 was
`59ebf97a9850aecec4bc39c1f5c1dc46e6490a6b5fd2a6cacdcac0c3a6fc4cbf`,
gh2.102.0 original root JSONL
`65ca537f6ed8a47fd0e560c421baa1f6c1efb8b25fc200d8c5c02c0e92eb2b9c`.
Three positive verifier observations covered upstream Cosign Google identity
and gh2.102.0 archive
`bb766f710eef8ede859c18578c72c327597cd4c8a85b06001b1f3843c6019386`
under exact `cli/cli` deployment.yml/trunk/source
`fc4b137cdef0a6bd28fd461b7cf9c84a5812a8cd`. Eleven expected rejections covered
wrong issuer/repository/workflow/ref/source/predicate/root, changed subject,
damaged signature, missing transparency and a valid foreign fixture under Yetlet
policy. Missing ensurepip, TUF bootstrap bytes and incompatible gh selector/root
inputs were retained invocation failures. None signed or uploaded Yetlet bytes.
Current validity/actual Yetlet identity acceptance was a separate future gate.
Raw stores remain `target/yt020-integrity-prep-20261012/`, with later historical
signatures governed by the [private trust record](RELEASE-SUPPLY-CHAIN.md#private-maintainer-key-preparation-policy-and-historical-signatures).

## Local signing, key CLI and Azure evidence — 2026-10-18

Base `c726af4a4f2f3cf3cf6225de0c793541b564249d` plus dirty working inputs bound
the accepted [local OpenSSL/provider contract](SECRET-PROVIDERS.md#local-signing-and-replaceable-provider-contract--yt-093).
Linux passed11 signing tests, core/sample/CLI fmt and strict Clippy,80 CLI library
tests, seven initial key integrations and nine focused integrations after repairs.
Actual Yetlet CLI sample build/run demonstrated PEM/PFX, tamper rejection,
v1→v2 rotation, independent invoices, public-only verification and revoked-v1
rejection. Persistent terminal yes and explicit initialization/rotation preserved
the unselected key, base settings and0700/0600 private modes. Status needed no
password. Temporary private fixtures were removed.

Original sample bootstrap failed on an owned0755 public parent. The correction
allowed owned non-group/world-writable public parents, retaining0700 immutable
versions and0600 files. A0775 parent stayed rejected. Task-created sample group
write caused a separate build refusal. Independent review then found lost TOML
comments/array decorations. The smallest correction covered inline arrays and
arrays of tables in inherited base and selected profiles, with strict reload.
Duplicate-key/revocation substitution was closed during contract review.

Windows MSVC source bundle196files SHA-256
`643e0a90c82355d49874c0320a601d344b47507916f7279d282d32043fe5833a`
and two-file delta `6f236a283c71c493b5afff9ccbc5388e3371dda8736caa83647890f963d5098c`
were independently manifested before execution. Existing Rust1.99/VS/SDK plus
task-local Perl/OpenSSL passed11 signing,72 CLI library and seven key tests,
native sample six outcomes and five actual key/status commands, preserving five
unselected files. All196 final hashes and14 exported logs matched review.
Private fixture/tools/cache/build cleanup and zero owned processes were verified.
An initial core lib command executed zero tests and remained compilation-only.
An incorrect lock-file-deletion assertion failed, then exclusive-open confirmed
released locking without replaying successful rotation.

Actual Windows `yetlet build` supervision failed125 `CLI_SPAWN Other`. The exact
pre-change CLI failed identically on the same sample, baseline bundle
`d073c5bc39f9eb77d049fdcf66be6a58caab61442081e55fc652131dc1e8a493`.
Its cause was unproven then. The later [Windows125 follow-up](QUALIFICATION-MATRIX.md#windows-exit-125-diagnosis-and-native-results)
found unsafe Cargo executable write access and separately qualified protected-path
build/run/cancellation. It did not rewrite the original failure or turn original
native crypto/key PASS into CLI-supervision qualification.

Owner-authorized Azure Standard/software EC-P256 lab used CLI2.91.0. Separate
documents/invoices keys, distinct-version rotation and three signatures verified
through Rust public-only code. Tamper, wrong routing and local revocation failed
as expected. Azure `isValid:false` with exit0 was an expected rejection.
The successful attempt recorded29 native-zero commands. Response-projection and
scratch Rust compilation failures remained with corrections. Both attempts
removed keys/access policy/vault/resource group, left subscription default
unchanged and verified active absence. Two soft-deleted vaults were retained
until scheduled2026-10-25 expiry without purge. No billing or actual expiry
readback is inferred. CLI verification may be local SDK execution. Remote signing
and Rust interoperability supply the proven scope. No Azure adapter was delivered.

Independent final review PASS0 accepted the source/Windows/Azure evidence with
the supervision limit. Raw records stay under `artifacts/yt093-signing-20261018/`,
including Windows manifests, original failures, reports and cleanup. Production
custody, cloud adapters, distributed reload, PKI/timestamps and release activation
were excluded. Later formatting/Guard preparation preserved original source-bound
logs and recorded another11-test pass separately.

## Custody and local rotation evidence — 2026-10-27 and 28

The age integration's authenticated LOCAL roundtrip matched original manifest
`bbbea5d051f243d722963e8b8667eca261df5fcd8f1a97edc62c77ce2e7bdf06`.
Native exit17 evidence remained incomplete. Ciphertext tampering, authentication
failure despite recomputed receipt hash and manifest mismatch were rejected.
Eight integration/two unit tests passed, with three preparation passes retained.
Fmt/Clippy, two YAML/fifteen Bash blocks and direct metadata/ZIP checks passed.
Five bounded repairs/original failures remain in `yt096-age-custody-20261027`.
`complete_product_custody=false` meant no actual sixteen-package/OpenSSL product
build or hosted durability proof. Review PASS0 accepted18 source hashes only
at local integration scope. Later hosted setup/readback was separately approved.

Rotation baseline `47936205fa4a9f05d4a22e41c08f76008b422081` used Linux/GNU
disposable LOCAL A/B keys. Nine tests, syntax, preparation/apply-check, two offline
locked builds and25 real commands covered the
[manual lifecycle contract](RELEASING.md#manual-custody-review-rotation-and-retirement).
All successful restored members matched original manifest
`c83de73f29daa8209ca4e449e6dcfb2120d1ecf0b751180eaa804e0754409fad`.
Receipts stayed `FAILED_INCOMPLETE`/complete=false. The first drill rejected
baseline license symlinks before Cargo. One material correction used checked
internal source symlinks and Python tar data filtering. Corrected real drill and
private modes/cleanup passed. A workflow-state initialization failure remained
operational. Owner-key metadata was checked by stat only, without key reads.
Review PASS0 accepted LOCAL test-copy recovery, excluding owner-key/Secret
recovery, production rotation, platform expansion and release approval.
Original proof remains in the recorded `.pipeline-output/yt096-key-rotation-local-20261028/`
handoffs. No raw evidence store was moved by consolidation.

## Early SSR and handler measurements

The following are **dated source-specific local results**, not new executions.
All 34 M2–M9 original Markdown sources, headings and Git blobs are recoverable
at local Git `ac98cdc127ccd299cb1150fef922b0560453a1d4`; GitHub reachability has
not been verified. Original JSON measurements, failed/reviewer output and raw
logs remain in their separately recorded ignored/external stores, **not in Git**.
Current tooling and published package scope are documented separately.

On 2026-10-08, M2 passed 16 native tests, wasm32/build-time hashing and Chrome 155
SSR/DOM byte parity, two local toggles with zero HTTP/WebSocket, asset/console and
CSP checks. Its debug WASM was 281,324 bytes; it measured no latency, release size
or other browsers. M3 added lazy Search, preserved input/results, bounded
allowlisted server state and corrupt-schema fail-closed SSR. Headless retained
focus made repeated `focus()` unreliable; deterministic input triggered activation.
M4 validated typed local/HTTP commands and 400/403/404/413/415/422 errors with no
failed-mutation revision advance. It was anonymous, process-local, direct HTTP,
not authentication, session CSRF, trusted-proxy or independent security evidence.

M5 passed A01–A10 only at local prototype scope: escaping, no-JS SSR, versioned
island state, local Chrome/Firefox interaction, Chrome byte parity, typed API
errors, strict external-script CSP, wasm32 server-dependency exclusion and a
reproducible baseline. WASM requires `wasm-unsafe-eval`, never general
`unsafe-eval`/`unsafe-inline`. Chrome smoke used debug native artifacts; Firefox
and measurements used matched release server/WASM. Do not mix profiles.

M5/M6 measurements used Linux x86_64, Rust 1.99.0, Python 3.14.4 and Chrome
155.0.8059.39 with cache disabled. M5 Firefox smoke used 157.0/geckodriver 0.37.1.
The 30-sample first-byte clock starts with request write on an already-connected
TCP socket and ends at the first response byte, possibly headers. It excludes
connection establishment, TLS/WAN, concurrency and cold-process startup. Browser
marks are DevTools-injected MutationObservers, five runs per measurement; they
are not LCP/INP. No independent security review or remote GitHub Actions execution
was established by M2–M6, and the then-declared Rust 1.85 was not tested.

| M5 release measurement | Median | p95 | Range |
|---|---:|---:|---:|
| SSR first HTTP byte, 30 | 0.034 ms | 0.039 ms | 0.032–0.052 ms |
| Chrome app ready, 5 | 48.5 ms | 50.6 ms | 47.9–50.6 ms |
| Search trigger to ready, 5 | 3.7 ms | 5.3 ms | 3.0–5.3 ms |

Release digest was `3f3c5f074478c525`. HTML sizes were `/` 532, `/app` 554 and
`/search` 1,431 bytes. Browser WASM was 216,006 bytes and Search 179,716 bytes.
Browser JS+WASM gzip estimate was 81,307 bytes; Search initial loader 551 and
activated JS+WASM 74,481. These are local compressed artifact sums excluding
CSS/favicon, not HTTP compression or measured wire bytes.

M6a (2026-10-09) passed 51 workspace tests, five App integration scenarios,
compiled rustdoc and Chrome/Firefox; debug digest `dbd3b1631850d93a` was unchanged.
It predated `App::page`. Complete M6 passed 66 workspace tests/four doctests,
strict Clippy, minimal/native/wasm32 builds, five typed-result tests, three setup
tests and seven Request ID tests. Todo's migration preserved API errors, CSP,
HTML sizes and all release assets at the M5 digest; Browser/Search excluded
native Axum/Tower/Hyper/RequestContext and optional Request ID.

| Sequential release medians | M5 historical | M6 same-session before | M6 after |
|---|---:|---:|---:|
| SSR first byte | 0.034 ms | 0.019 ms | 0.033 ms |
| App ready | 48.5 ms | 49.5 ms | 49.0 ms |
| Search ready | 3.7 ms | 3.3 ms | 3.8 ms |

**Native SSR increased 0.014 ms** in that same session; Search increased 0.5 ms.
Scheduling/process/browser conditions were not controlled. This is disclosure,
not zero-overhead, statistical equivalence, a speed improvement or a performance
budget. Real IME/AT, WebKit/mobile, persistence/auth/proxy and production assurance
were outside this early scope.

## Keyed renderer and lifecycle evidence

The M7a canonical report is timestamped `2026-10-09T02:02:22+00:00`, not replaced
by later reruns. Linux/Rust 1.99/Python 3.14.4/headless Chrome 155, release mode,
cache disabled: 200 keyed rows, 500 seed-7 toggles, ten measured rounds with
rotating strategy order and a 50-update warm-up per strategy; append/remove/move
followed. Equivalent checksum, outside/owned-input focus and keyed identity were
checked. Times include WASM/DOM work with no per-update JS crossing.

| Strategy | Update median/p95 | Round median/p95 | Identity outcome |
|---|---:|---:|---|
| Subtree replacement | 0.400/0.500 ms | 213.500/228.200 ms | Owned input/focus and untouched row lost |
| Positional View patch | 0.100/0.200 ms | 41.700/42.900 ms | Input retained; semantic keyed identity lost on reorder |
| Keyed direct DOM | 0.000/0.000 ms | 2.000/2.100 ms | Input/focus and keyed row retained |
| Runtime static slots | 0.000/0.000 ms | 1.900/2.600 ms | Input/focus and keyed row retained |

All outside focus and final states passed. Timer resolution makes zero entries
nonliteral. Replacement created 2,015,000 update nodes versus zero for the other
three; each alternative recorded 5,000 text/10,000 attribute mutations. Positional
reorder added 4,000 text/8,012 attribute rewrites. These count explicit/logical
operations, not engine allocation/layout/clone internals. Static slots use runtime
`cloneNode`, not a compiler. Benchmark digest `71182d03b13e4d13`, WASM 165,287
bytes/58,383 local gzip; production before/after stayed `3f3c5f074478c525`.
No layout/paint/heap/GC, long-lived leak, other-browser timing or framework
comparison was performed. One known-shape list cannot select a general winner.

M7b's first reviewer found three P2s: incomplete exact child/text preflight,
reserved-attribute case handling and stale-owner disposal/drop. Repairs preceded
73 workspace tests/five doctests, isolated Chrome and existing debug-profile
Chrome/Firefox regression PASS. M7c passed 17 focused tests (13 graph tests), one
authoring doctest and Chrome batching/re-entrancy/drop/failure cases; nested
subscription, reducer disposal and queued-capture cleanup regressions passed.
Its final byte comparison initially used a wrong dist path; only that harness
comparison was corrected after product PASS. M7d passed 86/five workspace tests
and isolated **release** Chrome/Firefox input/focus/selection/listener tests;
production browser regression remained **debug**, with separate byte-identical
release assets. Isolated digests progressed M7b `63da15682051d80d`, M7c
`150d17ce892eaebd`, M7d `c33a92498904a715`; these are distinct snapshots.

M7e was a **no-decision freeze**, with 86 tests/five doctests, wasm32, Chrome/
Firefox and isolated fixtures PASS. Its aggregate helper failed a sibling-module
import after product checks; bounded asset-reader recovery completed exact bytes
and cleanup without source changes. Repeat benchmark medians 213.8/41.7/1.9/1.9 ms
were variation, not replacement canonical results. Reviewer
`wc_job_9hT23HL0mwaTto4S` passed. Fixed templates/input/conditional slots, general
VDOM/signals/forms/macros/resumability and WebGPU integration were not promoted.

M7f extracted only metadata (5 core + 17 fixture tests, one doctest PASS);
M7g retains its [unaltered original](M7G-VERIFICATION.md). M7h's attempted placement
in the existing DOM crate changed production digests to `6895d575d9c6a494`, then
`fdb6f0049bde5083` even when feature-gated. The independent keyed crate restored
all pre-adoption production bytes. Its isolated Chrome/Firefox release fixture
passed at `6754025153168694`, WASM 272,728/75,818 local-gzip bytes. This is not
zero cost after adoption. M7f/M7h independent continuation reviews passed.

M7i's combined browser Cargo invocation unified web-sys features and changed
Search JS by three bytes; separate entry builds restored Search exactly. Todo
adoption plus final lifetime repair changed production digest to
`faebb0709dccd863`: Browser runtime 317,005 raw/101,349 gzip versus M7e
234,279/81,307, an **increase of 82,726 raw/20,042 gzip bytes**. Search JS
14,699/WASM 179,716 remained byte-identical (SHA256 respectively
`1aca190b9e125e3e09f79c66be7c249f7be0fb794273575f547e6912f91e4f9b` and
`ff46e70c132238db34765e9718692b8246359d2516e344258f464b5d6fc85db9`).

M7i reviewer PASS did not close the broader final gate: M7j reviewer
`wc_job_orP7QpVjLsh07HWI` failed a P2 queued `spawn_local`/late HTTP lifetime race.
Token retirement plus before-start/after-await checks repaired it. Complete
replacement validation passed **91 tests/five doctests**, six wasm32 packages,
Chrome 155/Firefox 157 debug/release production and lifetime-race suites, isolated
release lifecycle and full benchmark. The final cleanup helper matched its own
ancestor command and exited 1 **after product PASS**; bounded ancestor-excluding
cleanup passed. Repeat reviewer `wc_job_XZzdXEOIx6owj1o5` returned terminal exit 0,
PASS, empty issues/followups. These were role-directed local Codex Jobs, not
native managed-role identity or independent production security assurance.

M7j local medians/p95/ranges: SSR 30 samples 0.037/0.073/0.015–0.076 ms; app five
49.0/54.8/48.1–54.8 ms; Search five 3.5/5.9/3.3–5.9 ms. Benchmark round medians
219.0/42.25/2.05/1.8 ms at `2026-10-09T07:08:21+00:00` preserved the canonical
report and production assets. The bounded decision is metadata + optional graph
+ keyed ownership for Todo local only, not general API promotion or superiority.

## Sample and API freeze evidence

M8e (2026-10-09) first passed 113 tests/five doctests and the full debug/release
Chrome/Firefox, REST/Cart, wasm32, benchmark and asset matrix. Final reviewer
`wc_job_dospP8AM0wUmvHFd` found stale Session destroy could send a removal cookie
after another request rotated identity. The repair clears the cookie only after
removing the exact record; retired identity returns Destroyed with no directive.
Focused Session/Cart 16/16, full **114/five**, Cart smoke, dependency isolation and
all production bytes passed. Browser suites were **not replayed** for that
native-only repair. Repeat `wc_job_J_MVNzFaVgTH1Gr1` passed the retained matrix.
M8e digest stayed `faebb0709dccd863`; local SSR/app/Search medians were
0.034/48.8/3.4 ms (30/5/5 samples), p95 0.072/51.4/4.2 ms. No equivalence claim.

M9j (2026-10-10) actually reran the complete 12-application freeze: **190 native
tests/five doctests**, format/strict Clippy/rustdoc, twelve release server builds,
eleven native smoke/cleanup checks, eight wasm32 packages, three release suites
in **each** Chrome 155 and Firefox 157 (Todo/Search, keyed lifecycle, dashboard),
12 catalog entries/four checker tests, 28 normal dependency trees and 170 API
entries in 14 files. All 136 production dist files stayed byte-identical, digest
`faebb0709dccd863`. SQL/Session remained in declared native consumers; no
SQLx/Diesel/SeaORM was present. Counter/Filter remained separately built at
`1eccb326477b59ae`. Current catalog thirteen includes the later Showcase gate.

Local informational M9j medians/p95: SSR 0.034/0.037 ms, app 48.7/89.8 ms,
Search 3.6/4.7 ms; the same loopback/cache-disabled 30/5/5 method applies.
Benchmark medians 215.75/41.75/1.90/1.80 ms retained equivalent states and the
original benchmark digest. Browser activated total was 317,005 raw/101,349 gzip;
Search activated excluding loader 194,415/74,481 plus loader 1,144/551.
These are estimates and source-specific repeats, not Web Vitals, network or SLA.

Two post-product evidence assertions were corrected: Session must occur in its
own dependency tree; API crate labels must accept the documented grouped form.
First final review then failed omitted constants and overbroad upstream API
classification. [Inventory repairs and repeat PASS](API-CONTRACTS.md#original-inventory-acceptance-and-review-limits)
changed documentation only; product matrix remained retained. Original release
YT-020 stayed Blocked at that time. Neither prototype completion nor Stable
Candidate declared a stable/public/production release. Later publication records
control today's distribution identity; broad IME/AT/WebKit/mobile and production
security were not established by these historical runs.

## M10 native, browser and delivery reconciliation

The 2026-10-10 M10 evidence separates source checks from actual native/browser
execution. YT-065 implementation reviewer `wc_job_FH0L7fH4UH6ICPoW` PASS covered
source/local gates; hosted execution was accepted only after PR1 and actual run
`38043565238` at `c38837835dac683b09194b475baab2d8eda5a875`. Candidate
`3128d23591feee15ce976d7b83a56dcd587048d7` and squash tree matched. Candidate
required CI38042614403 and merged baseline38043547271 passed independently.

| Executed lane | Result and preserved failure boundary |
|---|---|
| Local Windows build26300 / PowerShell7.6.6 / Rust1.88 GNU | Nine reusable crates:80 tests/four doctests PASS, Job `wc_job_2fmSUCLQI2b9dOe9`, clean separate checkout. Initial compile passed but test linking lacked GNU prerequisites. User-local w64devkit2.10.0, actual dlltool/assembler/unwind inputs and process-local `-C link-self-contained=yes` resolved it; no stub library, skipped test, elevated workload or global PATH change. This is native GNU, not local MSVC/browser evidence; source logs remained on that runner when artifact transfer was refused |
| Offline Debian12 Linux x86_64 container | Pinned Rust1.88 image `rust@sha256:af306cfa71d987911a781c37b59d7d67d934f49684058f96cf72079c3626bfe0`, exact3128d23 Git archive and locked vendor read-only, no network/ports/socket/credentials, nonroot and bounded resources. Job `wc_job_MUaVCKwQYsrSwseh`:196 tests/five doctests and real loopback SSR/escaping/CSP/404/405 PASS. Owned server/container removed; no orchestration/deployment claim |
| Hosted native qualification | Rust1.99: Ubuntu24.04 x86_64 GNU image20261004.327.1, Windows2025 x86_64 MSVC image20260925.250.1, macOS15 ARM64 image20260907.0337.1; each nine-crate80/four PASS. Named downloaded artifacts, not job titles, establish identity |
| Local/hosted Firefox | Local Job `wc_job_sv9TmUH_rdBF2DdM`: Todo/Search, keyed, multi-island PASS. Added real Tab/Enter/Unicode/labels/error-focus/root-language/stateless303 forms journey:five groups PASS. Hosted Firefox157.0/geckodriver0.37.1/Python3.12.15:all four suites and sanitized report/cleanup PASS. All136 pre-existing production assets were unchanged by local builders |

The initial mocked forms cleanup test failed at an unmocked socket PermissionError;
one fixture correction passed eight mocked cases. Mocked syntax/startup/failure
checks are not real-browser proof. Local form teardown observed driver exit0,
server-15 and closed owned resources. A documentation reviewer later found three
stale hosted-pending claims; correcting those did not rerun platform tests or
convert a document review into a security scan. Real OS IME, spoken AT,
WebKit/mobile, touch/soft keyboard and zoom/reflow remain **Not verified in this
M10 cohort**, optional future scope without WCAG/device/general support claims.
Later Board IME/Orca evidence has its own source and does not rewrite these cells.

Full original M10E/F/G/H, remaining-plan and delivery headings/blobs remain locally
recoverable at `40b66f1ad9bd0eb5666a9e1c8f113310993a3544`; remote reachability is
unverified. Raw evidence stays under its original `target/m10-remaining/`,
`target/m10-delivery/docker/`, `target/qualification/` and runner-local stores.

## M12 migration acceptance and unexecuted paths

The original2026-10-15 M12 migration accepted10 YT-084,30 YT-085 and12 YT-086
file mappings, the transitive Rust fixture caller and three workflow surfaces
after separate reviews. This is bounded tool migration, not current universal
Python-free, platform, browser/GPU or production qualification.

YT-083 actual Linux and exact-source Windows MSVC1.99 process/doctor checks
passed after foundation corrections. The separate GNU import-library attempt
failed before tests, not supported-platform evidence. Doctor/foundation does
not qualify Windows persistent-Service or later browser/qualification commands.
YT-084 compared128 independently generated artifacts and preserved original
Python bytes; early catalog/README/checker/CI adoption followed parity. No
new GPU/browser or long-load acceptance came from those build outputs.

YT-085 initially exhausted four repairs/two operational retries while loopback
allocation returned PermissionDenied; missing Board/scene commands and runtime
gaps stayed explicit. Later separate implementations and host evidence produced
26 source-bound receipts:25 positive scenarios and one **nonzero scene-chrome
GPU-unavailable** result. Default Firefox Support was semantic fallback with
gpu_acceptance=false. Shared-helper/report/Board-retention repairs used focused
checks;16 final Board tests/fmt/Clippy passed. Final independent review at
target/m12/yt085/reviewer-board-retention-20261015/result.json PASS accepted
the30 mappings, while Windows persistent-Service and external Redis fixture
execution remained **Not verified**; their caller migration belonged to YT-086.

YT-086's source/fixture corrections passed57 focused tests (nine measurement,
10 local,21 Docker,17 checksum). The full xtask attempt remained **exit101:
117 passed/four existing server_status loopback-PermissionDenied failures/two
ignored**, later integration binaries/doctests not reached. Earlier compiler,
inspection/edit-tool errors and exhausted/overrun counters were retained, not
reset into a fresh all-green suite. Controlled-PATH sample-dx and checksum
create/verify exited0 with zero Python/Node sentinel invocations, proving only
their executed filesystem/source branches. No hosted workflow was triggered.

Independent reviewer `wc_job_uEQye7iAKqzfoe59` resolved10 of12 prior P2s and
retained two. Separately authorized fixes made corrupt-start inspect recursive
per-attempt native-exit/cleanup receipts, and preserved first SIGINT/SIGTERM
identity through Docker/report finalization. Check/fmt passed;34 focused cases
(11 local/23 Docker) passed; an unused-import Clippy failure preceded corrected
PASS. A whitespace-wrapper failure and its unsuccessful correction were stopped;
ordinary git diff --check passed. No full application/Docker replay followed.
Final independent YT-086 reviewer `wc_job_6UfDDdHnNCBXkMhA` PASS with zero
issues/followups accepted the **bounded migration**; it did not erase those failed
or unexecuted paths, establish complete runtime-graph parity, actual private
rebuild/hosted execution, Windows cancellation here, long-load, physical IME/AT,
cloud or new GPU support.

Full original52-row migration ledger and plan, exact direct cases/options/env,
status transitions and hashes are locally recoverable at
`40b66f1ad9bd0eb5666a9e1c8f113310993a3544`; original YT-082–086 verification
is locally recoverable at `b610acc7e098020b6b59dfa0d1ab2b75b059e656`.
Remote SHA accessibility is unverified. Raw historical target/m12 logs remain
separate and unchanged. Current commands follow [Development](DEVELOPMENT.md#rust-tooling-migration-contracts-and-historical-inventory)
and [xtask usage](../xtask/README.md), with later tools/status assessed separately.

## Browser and Vite feasibility evidence

The Browser DX study dated 2026-11-07 inspected
`bebdd03660ca7dc33fdd1ed4c7e56b5680e055c1`, published SDKalpha.6/CLI0.2.0.
It selected a connected guide over new APIs, event registries or desktop-host
integration and replayed no product checks. The
[browser walkthrough](BROWSER-APP.md#browser-dx-research-and-selected-follow-up)
retains preparation, DOM/listener ownership and unselected ideas.

The 2026-10-17 Vite feasibility base was
`6d1de322739fb3defd32dccd1f6ae3790aecdddd` plus the isolated experiment.
Vite8.3.4, Tailwind/official plugin4.3.3, TypeScript7.0.2,
Node24.21.0/npm12.2.0 and Playwright Core1.64.1 were tooling inputs.
Both plain/plugin production builds and typecheck passed. Nine Rust tests,
format, strict Clippy and build passed. Offline npm ci/rebuild reproduced all 18
output files and package-lock SHA256
`79a124daf75a2a18f164d0ce8707e50531205d116decca4d6cda86dcb198bfff`.

Installed headless Chrome155.0.8059.84 at 840x650 passed both profiles with JS
on/off, escaped useful SSR, computed shared/plain/plugin CSS, external SVG,
two clicks and lazy module/CSS200 with expected MIME. No recorded CSP violation,
console error or external-origin request occurred. Six unsupported paths per
profile returned404, including private manifest/traversal/query cases. The
synthetic backend sentinel was absent from assets/HTML. Browser/server exited,
port closed. The narrow SVG/cycle/CSS fixture establishes no mobile,
accessibility, other-browser or hostile-filesystem qualification. Screenshots
were functional inspection, without visual-design acceptance.

Initial role-directed executor Job wc_job_MXjlDUIEJIPAaSIM lost its Runner
association (`runner_instance_replaced`) and never supplied a terminal role
result. Network-denied npm stalled and its exact owned PID was stopped after
cwd/command inspection. Later absence was verified. Current-session role
emulation completed dependency provisioning and bounded checks after that
failure. It supplied no saved-model/effort/native-identity or Job-success claim.
Independent role-directed Reviewer Job wc_job_SuuEr3YetGlVbs_8 separately
completed exit0, JSON pass, no issues/followups, checking 24 source paths and
permitted evidence hashes. Its saved model was observed as gpt-6-astra, with
inherit mode and no applied effort selector. Both results retain their distinct
identity and scope. The spike's original Todo YT-098–101 plan was later delivered
under separate bounded acceptances, and supplies no stable shared asset API,
WASM/HMR/Windows/deployment or public release by itself.
Raw ignored evidence remains at artifacts/vite-spike-20261017. The
[full original spike](https://github.com/bohewu/yetlet/blob/c0a40af80a48932f3cdf7c593f36fe563e4ec669/docs/VITE-SPIKE-VERIFICATION.md) and
[DX study](https://github.com/bohewu/yetlet/blob/c0a40af80a48932f3cdf7c593f36fe563e4ec669/docs/BROWSER-DX-RESEARCH.md) are locally Git-recoverable only.

## Development HTTPS evidence and original failures

The 2026-10-18 YT-102 follow-up used branch feat/yt090-configuration at
`dbcae2dbb47a9e8a0f1e103b49ec225ee019b63f` plus bounded changes. Public paths,
Development opt-in, optional TLS, private backend and build-without-credentials
are current [development contracts](DEVELOPMENT.md#opt-in-local-https-preparation-and-lifecycle).
Linux lacked mkcert initially. The extracted Ubuntu1.4.4-1ubuntu2 package
matched metadata SHA256
`f2ed2a59b2ebce55a97a8efd218e4a0aea35a6c0c233e411728f3f43b3a5b592`.
The GitHub1.4.4 release predated asset digests. Windows checked the official1.4.4
binary against Scoop manifest SHA256
`d2660b50a9ed59eada480750561c96abc2ed4c9a38c6a24d93e30e0977631398`.
Both used isolated CAROOT and explicit client CA/hostname verification.

| Source-bound check | Actual result and retained limit |
|---|---|
| Linux settings-service and Vite-service builds | Both CLI build0 with certificate directories absent. Explicit local framework paths, separately prepared offline locks and offline npm ci. Generated runtime/service tests passed |
| Linux direct Rust/Vite run | Rust HTTPS200 atlocalhost/127.0.0.1 with wrong-host rejection. Vite page/client/TS HTTPS200, trusted WSS101, private Rust HTTP health200. Certificate/config/foreign-file requests403 and scheme-aware CSP retained |
| Linux invalid material and cancellation | Invalid key exit1, no listener and no PEM log output. Actual controlling-terminal Ctrl-C preserved native SIGINT -2 for both CLI processes |
| Linux final suite |125 passed/0 failed/5 ignored across17 binaries. Node13 passed, four settings profiles across Docker/Compose/Kubernetes passed, strict Clippy/fmt/docs/samples passed. Ignored external/runtime tests were unexecuted |
| Original Linux failures | Docker synthetic lock predated optional TLS dependencies. Exact two optional edges and fixture corrected. Later watch fixture collided with coordinator Vite5173. After stopping that service, full serial suite passed. Both prior failed logs remain |
| Windows source/tools |197 selected files matched, initial ZIP SHA256 c413cb412119844c5fa5063600a8859bf17a17fa00431a45c324ee8058b4235c. Post-build Docker fixture-only delta was outside runtime inputs. Rust/Cargo1.99 MSVC, VS2022 14.44.35207, Node24.18.0/npm12.2.0, Python3.14.3. Process-local protected Cargo/static OpenSSL3.5.9 reused |
| Windows CLI/app builds | CLI build0 with a retained linker warning. Rust TLS, Vite TLS and default HTTP build0. TLS directories absent at build. Source-matched CLI SHA256 94c892aa04fd8dab68cde3c78206ba0f0fa64781cd24d5cfcffc5a1df73816d1 |
| Windows Rust TLS | CA/localhost/IP validation200, wrong hostname refused. Malformed cert/key exit1, no startup listener/listening log/material leak. Real console Ctrl-C preserved CLI/Rust DWORD0xc000013a, later process/listener absence verified |
| Windows Vite/default HTTP | Page and client HTTPS200, token-bound HMR WSS101, private backend HTTP200. Real Ctrl-Break exited six named CLI/Cargo/Rust/Node handles, released3000/5173, sentinel survived,1.0324s. Default HTTP200 and real Ctrl-C cleanup2.0952s |
| Original Windows wrapper failures | AttachConsole failed while already attached. One FreeConsole correction reached valid Rust HTTPS/Ctrl-C, then zero-time descendant wait found unidentified PID26224 active. Its identity/exit delay remain unknown. Later process absence and separate bounded Vite cleanup passed. An earlier PowerShell scalar-splat creation input failed before explicit literal argv succeeded |

Windows Jobs for CLI build, dependency preparation and app builds each reached
terminal0. Direct runtime and closeout commands exited0. Initial GitHub API
prerequisite observation was lost, followed by separately verified host TLS
binary import. No new OpenSSL bootstrap occurred. All 36 exported public files
matched hashes. Independent `/root/https_review` PASS0 reviewed source,
Linux/Windows raw evidence,197 source entries, failed wrappers and later cleanup
without rerunning services/tests. A tls.rs no-index diff returned expected1 for
content difference without whitespace errors.

Closeout removed only owned apps/CA/leaf private keys and confirmed task process
and listener absence. Linux's pre-existing3010 health remained200. Windows
preserved196 prior source files/CLI, global Cargo ACLs and both root stores.
No system trust installation, physical-keyboard proof, full browser HMR,
Windows GNU, production TLS/PKI or release/deployment acceptance follows.
Original logs stay in artifacts/dev-https-20261018. The
[Linux record](https://github.com/bohewu/yetlet/blob/c0a40af80a48932f3cdf7c593f36fe563e4ec669/docs/DEV-HTTPS-VERIFICATION.md),
[native Windows record](https://github.com/bohewu/yetlet/blob/c0a40af80a48932f3cdf7c593f36fe563e4ec669/docs/DEV-HTTPS-WINDOWS-VERIFICATION.md) and
[independent review](https://github.com/bohewu/yetlet/blob/c0a40af80a48932f3cdf7c593f36fe563e4ec669/docs/DEV-HTTPS-REVIEW.md) remain exact local Git sources.

## Secret provider native and review evidence

The initial2026-10-18 native prototype at
`b627d291696fe1aaa4a3db5040da07e2ad44da04` plus scoped changes passed
three provider/four State tests, one doctest, strict Clippy/fmt/dependency
isolation and synthetic State200. `/root/secret_provider_review` independently
accepted object-safe Send lookup, owned/empty/missing values, fixed errors,
selected snapshots and explicit constructor/State composition. It inspected ten
new files and786 baseline hashes with five intended existing-doc deltas.
This initial review did not cover Windows/cloud/browser or release behavior.

The separate Windows package SHA256
`bd15490c6c2fe205ef6d041ed6f7e34d4d19588991a4e2bd72b9b03ca9930140`
contained208 matching imported files before/after native checks on Windows11
Pro10.0.26300.0, x86_64 MSVC, PowerShell7.6.6, Rust/Cargo1.99,
Clippy0.1.99 and rustfmt1.10.0-stable. VS2022 vcvars64 and protected Cargo
were process-local. Job wc_job_dg3H7A3qqxAhZevz completed0 with 13 tests
(one poison, four provider, five State/Loader, three Azure parser), one doctest,
strict Clippy/fmt and configured State200. Loader file rewrite/reload/replace,
removed/empty/missing names, retained owned old values, State injection,
SSR/CSP and profile precedence were exercised. Azure parser tests made no cloud
calls, and Router::oneshot needed no socket listener.

The first offline command failed101 before compilation on missing
signal-hook-registry index metadata. An explicitly authorized locked MSVC fetch
filled the cache, then unchanged source/locks passed offline/locked. A preliminary
PowerShell -Command input was rejected before start. cargo-fmt.exe -V returned2,
while rustfmt -V and actual fmt succeeded. Export readback returned Unknown
resource, followed by corrected bounded text transport after oversized-batch
and object/array-envelope errors. All25 transferred public files matched native
hashes, with original native ZIP retained SHA256
`dd3b67897a0974ec31e1432be20a0d4b6ae75a30f3e57afbf14ed2263b1503b7`.
The later crate README qualification-link delta left 207 files/all compiled
inputs identical. The Work Result's mixed/current-failed cache-miss aggregate
remains unchanged beside final terminal passing command evidence.

Final native preservation checked 208 files, previous196/197 task source trees,
Cargo config/credential hashes, zero private fixture/process/payload-log matches
and unchanged global tools/trust/ACL. Only the task-root ACL was restricted.
Linux's real Azure experiment is separately recorded in the
[retained original rotation proof](SECRET-ROTATION-VERIFICATION.md#actual-azure-result-and-cleanup)
and [operator lifecycle](PROVIDER-OPERATIONS.md#secret-snapshot-adoption-and-operator-credential-lifecycle).
It establishes no Azure-on-Windows authentication or external credential retirement.

`/root/secret_rotation_review` independently passed the rotation/native/cloud
scope with zero findings, checking 13 Linux/native tests, doctest/State200,
Azure latest-v2/pinned-disabled-v1/retained-old semantics and resource cleanup.
It verified208 hashes before the README update and207 plus the documented delta
afterward. The later minimum-local-contract review directly reran 15 tests/one
doctest/Clippy/fmt and accepted competing gated reloads and malformed/invalid
failure retention/recovery. Its registered reviewer/high dispatch had unavailable
trace evidence, leaving effective role/model/effort unverified. This local
follow-up reran neither Windows nor Azure. Production credential lifecycle,
SDK adapter, scheduler, browser and release qualification stay outside scope.
The [Windows source](https://github.com/bohewu/yetlet/blob/c0a40af80a48932f3cdf7c593f36fe563e4ec669/docs/SECRET-PROVIDER-WINDOWS-VERIFICATION.md),
[initial review](https://github.com/bohewu/yetlet/blob/c0a40af80a48932f3cdf7c593f36fe563e4ec669/docs/SECRET-PROVIDER-REVIEW.md) and
[rotation/minimum-contract reviews](https://github.com/bohewu/yetlet/blob/c0a40af80a48932f3cdf7c593f36fe563e4ec669/docs/SECRET-ROTATION-REVIEW.md)
are locally recoverable. Raw logs remain at artifacts/secret-provider-20261018
and artifacts/secret-rotation-20261018 in their original ignored locations.

## Windows exit-125 diagnosis and native results

The 2026-10-18 source was `c726af4a4f2f3cf3cf6225de0c793541b564249d`
plus signing and bounded diagnostics/regression. The same CLI/sample returned125
with default rustup Cargo PATH and0 with byte-identical protected Cargo, SHA256
`6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e`.
Three inherited allow ACEs granted unrelated identities0x1301bf including write,
and physical MSVC Cargo was also writable. Rejection occurred before child
creation/Job assignment through protected policy io::ErrorKind::Other, without
a native Win32 error code. This isolated an executable-permission cause.

The diagnostic exposes fixed CLI_PROCESS stage=executable, kind and optional
numeric OS error without path/argv/environment/arbitrary error text. The
regression deliberately grants Everyone write on a disposable Cargo copy,
proves refusal/no output, then builds with a protected copy. Global unsafe Cargo
and permission policy stayed intact. All 196 selected source files matched;
post-test CLI README link synchronization did not change compiled inputs.

| Native check | Actual result |
|---|---|
| Task-local OpenSSL3.5.9 static MSVC no-shared/no-asm/no-tests and locked CLI/sample fetch |0. This does not run upstream OpenSSL tests |
| Windows windows_build integration suite |6 PASS,0 failed,2 ignored Vite/watch. Default signing sample build125 with kind=Other/native_os_error=None. Same build protected PATH0, sample run0/six demonstration outcomes |
| Finite/frozen/offline/headless cases | App23 propagated. Lock/cache failures101 and preflight/headless refusals retained |
| Real native console cancellation | CREATE_NEW_CONSOLE plus GenerateConsoleCtrlEvent C/Break, both CLI DWORD0xc000013a. Three pinned descendants exited and forwarded Break markers checked before outer cleanup. Unrelated process group survived, lock retained |
| Linux focused library suite |80 PASS with isolated static OpenSSL inputs. Later Guard-preparation byte-equivalent label/config formatting had separate11 signing tests PASS, without a Windows rerun |

Finite sample run used CONIN$ with processed input, file stdout/stderr and NUL
stdin. No stdin control-byte simulation or physical-keyboard claim was made.
The synthetic signing password stayed child-local, generated identities stayed
in memory. Owned task processes were absent, global ACLs/source preserved and
only authorized bootstrap/disposable fixtures removed. Source, protected tools,
OpenSSL/cache/builds/logs remained reusable. Independent final review PASS0
checked196 hashes, seven intended changes against799-file baseline, root-cause
comparison and cleanup, with matching native role/model/effort trace. The
[original report and source-bound PowerShell setup](https://github.com/bohewu/yetlet/blob/c0a40af80a48932f3cdf7c593f36fe563e4ec669/docs/WINDOWS-CLI-125-VERIFICATION.md)
retain exact identities and prior failed signing logs. Separate later Windows
GNU Vite-pair qualification lives in
[the Windows matrix](#historical-windows-native-browser-and-cli-evidence).
This MSVC follow-up did not run those ignored cases or qualify HTTPS, physical
keyboard, crypto performance, deployment or release. Raw evidence remains in
artifacts/windows-cli-125-20261018 and artifacts/yt093-signing-20261018.

## Naming audit results and generator failure

The 2026-10-22 N01–N08 response was based on checkpoint83f417e and later
committed as8c8fba3. [The naming guide](NAMING.md#dated-naming-audit-dispositions)
owns canonical names and wire/source distinctions.

| Focused area | Historical executed result |
|---|---|
| Config | Loading/public-naming18+2 default,23+4 optional provider PASS. Starter3/Admin6 config tests PASS. Config/provider/consumer strict Clippy, Admin all-feature/all-target check and formatting PASS |
| Auth/rate limit | Public naming3+2, README doctests2+1, configured native example7 PASS. Auth/rate/Board all-feature/all-target and isolated YT-125 checks/strict Clippy/format PASS. Existing HTTP/runtime suites compiled where applicable without replay |
| Signing | Certificate-backed/free focused tests2/2 PASS, ten filtered per invocation, all-target checks/Clippy/format PASS with local OpenSSL3.5.5. Original omitted Result test import failed, then smallest import correction passed. No transfer of old3.5.9 qualification |
| CLI | Focused parser/template/removed-option, settings8, help/inspect4, scaffold14, selected generated-Rust compile/test, key9 and service-scaffold1 PASS. All-target check/strict Clippy/format PASS. Other opt-in ignored cases stayed unexecuted |

The separately run generator_selected command failed101 after plain service
Docker/Compose/Kubernetes output, at settings profile
`CLI_SOURCE_UNSUPPORTED generate docker: unsupported config crate layout`.
Existing config src/secrets.rs at83f417e violated the generator's isolated
lib.rs-only export. The same guard covered legacy runtime-settings layouts.
This predated naming edits and remained a failed probe, with registry-only
source unqualified. Independent review first failed P2 for README's overstated
legacy-layout acceptance. Both entry guides were narrowed, then focused
re-review PASS0 confirmed 65 hashes and only three Markdown deltas from the
first review. All 62 other paths stayed identical. Docs75/Backlog126 and584
changed-Markdown links passed. Raw evidence stays in the original external
yetlet-naming-audit-20261022-gh6dtwyj location.

The subsequently selected generator source-closure repair has separate original
failure/repaired acceptance at local Git
`ed7a082b1f9f72430f06866f7d550b0ed9ecbae9`, with current supported shapes in
[the CLI matrix](CLI-GUIDE.md#current-generator-source-matrix).
Naming PASS did not rerun old Auth/Valkey/IdP/browser/CSP, Windows, cloud,
security or release suites. It changed no backlog state or release inclusion.
The [complete original audit response](https://github.com/bohewu/yetlet/blob/c0a40af80a48932f3cdf7c593f36fe563e4ec669/docs/NAMING-AUDIT-20261022.md)
retains original dated In-progress/Blocked states as history.

All twelve Z16 source Markdown blobs/headings are recoverable at exact local Git
`c0a40af80a48932f3cdf7c593f36fe563e4ec669`. GitHub SHA reachability is unverified.
Git recovery does not recover ignored/external raw logs, private credentials or
remote environments. This consolidation reuses dated evidence without rerunning
product/browser/platform/cloud experiments.

## Historical alpha.2 local package verification and PR22 correction

On 2026-10-29, the YT-096/YT-209 alpha.2 local preflight and independent
Reviewer passed with zero issues/required followups. Subsequent PR22 CI failed
at the synthetic rebuild fixture. The following preserves the actual local
checks, the failure and the separately reviewed focused correction. At that
checkpoint Merge remained blocked pending repair delivery and required CI.
[Version/source and remaining gates](RELEASING.md#historical-alpha2-version-package-and-release-gates--2026-10-29).

### Alpha.2 affected verification and actual results

The existing alpha.1 local package recipe and external consumer are reused with
new output/version inputs. Rust/Cargo1.99.0, four build jobs and the local GNU
host/system OpenSSL are used. This is local packaging/linkage preflight, not an
Ubuntu24/OpenSSL3.5.9 hosted build or new Windows/GPU qualification.

| Check | Result and limit |
|---|---|
| Nineteen affected manifest roots | `cargo +1.99.0 metadata --offline --locked --format-version 1` PASS; no lock drift |
| Version/dependency correspondence | Sixteen versions, eighteen internal requirements and 112 local lock fields match; all third-party blocks unchanged |
| Sixteen new-version archives | Normal `cargo package --offline --locked --allow-dirty` verification PASS; no verification-bypass flag on final commands |
| Archive input correspondence | All sixteen normalized manifests and packaged locks differ from retained alpha.1 archives only by the local version; 132 current source/test/example/README/license/original-manifest payload files match |
| External packaged consumer | `cargo run --offline --locked --features integrations` PASS, prints `packaged consumer OK`; Auth/session, Config/secret-provider and Redis/tls compile/integrate through verified directory sources |
| Consumer dependency resolution | Locked metadata resolves all sixteen alpha.2 libraries from final verified directory-source packages, with no checkout paths/patches |

Sixteen `--no-verify --exclude-lockfile` bootstrap collections populate ordinary
directory sources for unpublished siblings; they are not verification PASS.
Final verified archives replace those bootstrap sources before the external
consumer executes. `--allow-dirty` applies only to this uncommitted local
preflight; it does not waive the clean-source release gate. The consumer's
program is byte-identical to the accepted alpha.1 consumer. No live Redis,
Session service, cloud resource, network provider or registry is exercised.

Evidence is retained at `target/yt096-alpha2-version-20261029/` (local, ignored):
master CI receipt, baseline hashes, migration/metadata records,
`manifest-lock-verification.json`, `package-command-results.json`,
`package-inventory.json`, `archive-delta.json`, `package-completion.json` and
original command outputs. The thirty-five package workflow commands include
collection, normal verification, consumer execution and metadata; they are not
thirty-five behavioral tests. No affected package/consumer check failed.

Accepted YT-202/203/204/205, alpha.1 version preflight and YT-209 implementation
evidence retain their exact original sources/inputs/limits. Runtime and release
tool implementation files are unchanged. No native functional suite, browser,
WASM/GPU, CSP, MSVC, security scan, load test, hosted build, custody drill or
unchanged release-tool test is replayed. Old advisory evidence retains its DB
date; this migration does not call it a fresh scan.

### Subsequent PR22 integration failure and local correction

The owner authorized strict/builtin Commit of the reviewed 53 paths as
`dbe1fbbb139fe3d7814ec8188a119a365286464c`, then Push and PR22. The staged and
exact outgoing range/native hook checks PASS. Original
[CI40888941346/1](https://github.com/bohewu/yetlet/actions/runs/40888941346)
passed MSRV but failed Stable's Rust-tool tests before audit/browser execution.
No Merge or automatic retry followed. Earlier archive/consumer PASS results
remain valid at their local preflight scope; they are not a current CI PASS.

The failed existing case is
`private_rebuild::real_cargo_sixteen_alpha_packages_and_normal_verification_refusal`.
Its synthetic manifests, internal requirements and expected archive names were
still literal alpha.1; the reviewed recipe derives alpha.2 from its compiled
root manifest. Cargo collected an alpha.1 fixture archive while staging correctly
looked for alpha.2. This version-sensitive test consumer was missed in the
initial migration/verification selection. It is a fixture mismatch, not a failed
real sixteen-library package check.

The minimal correction changes only `xtask/tests/private_rebuild.rs`: synthetic
versions, dependency requirements, filenames and manifest assertions read the
same compiled workspace-version authority. No public helper, runtime, recipe,
historical trust pin, feature or dependency is changed. Normal verification and
the intentional compile-error refusal remain required. An inventory of the other
alpha.1 literals identifies retained legacy custody pins and deliberate negative
or self-contained synthetic inputs, not additional current-version fixtures.

The one failed case passes locally with `--offline --locked`, including all
sixteen synthetic archives and its expected negative verification refusal.
Owning-test Clippy with `-D warnings` and format PASS. No already accepted package,
consumer, browser or platform check is replayed. Original CI metadata/full log
and focused repair output are retained at `target/yt096-alpha2-pr22-20261029/`.
The expected compile-error output is the passing negative assertion, not a new
product failure. Independent repair Reviewer
`/root/alpha2_version_independent_review_20261029` PASS0; all four source hashes
match before/after read-only review, and the negative block remains byte-identical.
Native reviewer/model/actual high effort are verified; only completion metadata
follows the review. New Commit/Push/CI
delivery requires the applicable authority; conditional Merge remains blocked.


The later [actual alpha.2 hosted outcome](ALPHA2-HOSTED-RECORD.md) has a distinct
C/tag/build and failed custody qualification because its receipt retained
alpha.1 identity fields. [Alpha.3 recovery](ALPHA3-VERSION-PREFLIGHT.md) and its
[accepted hosted record](ALPHA3-HOSTED-RECORD.md) retain the subsequent correction
and publication boundaries. The alpha.2 GNU preflight adds no Windows, browser,
GPU, registry or current-release qualification. Full dated text and original
rendered headings remain at the locally verified
[original source](https://github.com/bohewu/yetlet/blob/8d6fad7e95ee25849872b974f922d10c71b70a95/docs/ALPHA2-VERSION-PREFLIGHT.md#affected-verification-and-actual-results).
Its Git SHA/blob/SHA256 are recorded in the linked release-gate section.
Remote reachability and continued raw evidence availability are unverified.
