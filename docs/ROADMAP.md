# Yetlet Roadmap — Evidence-Gated Milestones

**Status:** Evidence-gated prototype roadmap; developer entry/status refresh 2026-10-12. Historical qualification dates and source identities remain unchanged.

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
| **M13 — Done at bounded Linux CLI preview scope** | Unpublished Rust `yetlet` binary, minimal handler-first scaffold and local doctor/build/dev | YT-087 | [CLI contract](YT-087-CLI-PLAN.md), fail-closed templates and processes, actual local HTTP/PTY evidence and final independent Reviewer PASS; no registry publication, Windows qualification or release activation |
| **M14 — Done at bounded Linux/Windows CLI preview scope** | Two internal OS adapters for filesystem-safe scaffolding and foreground process ownership | YT-088 | [Cross-platform plan](YT-088-CROSS-PLATFORM-CLI-PLAN.md), real GNU/MSVC positive new/build/dev HTTP/console/cleanup and per-slice independent reviews; no public release |
| **M15 — Done at bounded Linux OCI/Kubernetes configuration-generator preview scope** | Opt-in CLI emission of Dockerfile, Compose and portable Kubernetes manifests without platform SDK coupling | YT-089 | [Generator plan](YT-089-CLOUD-NATIVE-GENERATORS-PLAN.md), source-path-aware image build, app-owned bind/health/SIGTERM, offline generation and per-slice independent verification; no image publication or cluster deployment |
| **M16 — Done at bounded YT-090 / YT-097 scope** | Opt-in typed native configuration and optional CLI scaffolding | YT-090, YT-097 | [Guide](CONFIGURATION.md), [focused Linux evidence and independent Reviewer PASS](YT-090-VERIFICATION.md); [CLI integration Linux/Windows GNU/MSVC and independent Reviewer PASS](YT-097-VERIFICATION.md); no default/core/browser or release change |
| **M17 — Done at bounded YT-091 Tailwind scope** | Opt-in build-time Tailwind CSS 4 application scaffolding | YT-091 | [Guide](../tools/yetlet-cli/README.md#opt-in-tailwind-css-4-yt-091-independently-reviewed), [evidence](YT-091-VERIFICATION.md); actual Linux HTTP/CSS/CSP/signals and native Windows GNU/MSVC Tailwind source-bound builds PASS; independent Reviewer `wc_job_T7Blu0CYQUOmaQTL` PASS, no core/browser change or release activation |
| **M18 — Done at bounded YT-098–YT-101 scopes** | Optional asset contract, Vite production/dev profiles and cloud delivery | YT-098–YT-101 | [Bounded Linux Vite spike](VITE-SPIKE-VERIFICATION.md) passed build/native/browser checks; [YT-098 application-owned producer proof](YT-098-VERIFICATION.md) passed 11 native tests, four local Chrome checks, cleanup and independent Reviewer `/root/yt098_browser_closeout_review` (zero issues/followups). [YT-099 production CLI integration](YT-099-VERIFICATION.md) Done at bounded experimental scope: Linux/Windows GNU/MSVC, local Chrome JS on/off, cleanup and independent Reviewer `/root/yt099_local_browser_acceptance_review` PASS; [YT-100 opt-in dev bridge](YT-100-VERIFICATION.md) Done: pinned Vite8 env P1 fixed; native Windows GNU/MSVC source-bound tests, Linux Plain/Tailwind Chrome 11/11 HMR/CSP/WS, actual dual-terminal Ctrl-C/restart/503/cleanup PASS; independent Reviewer `wc_job_MprRYhxeijOu6xG5` PASS (zero issues/followups). Windows live HMR remains unverified; [YT-101 cloud delivery](YT-101-VERIFICATION.md) Done: real Linux Docker/Compose/K3s Pod/Service/asset PASS, Windows MSVC native/source PASS, subsequent Windows GNU 726/726 source and native tests/generator PASS after official MinGW64 GCC/Binutils installation, independent Reviewer `wc_job_fYtD9F8KXkXyzvYQ` PASS; no stable shared asset API, default/core/browser or release change |
| **M19 — Production-oriented application and developer experience (partial)** | Native Issue Tracker starter, research-only developer workflow and separately opt-in rolling asset qualification; planned browser-only starter | YT-092, YT-102–YT-104 | [YT-092 starter](YT-092-STARTER-VERIFICATION.md): Linux/Windows native HTTP, persistence, CLI lifecycle and independent Reviewer PASS. DX research and rolling qualification remain separate; no production support or release claim. YT-104 remains Todo at registration-only scope. |

**YT-094 first slice (2026-10-19):** the
[SSR administration reference](../examples/admin-dashboard/README.md) starts from
the existing CLI's Vite/Tailwind/settings scaffold and reuses the persisted Lab
Operations Board through explicit application composition. The selected CSS-only
Vite build covers responsive list/filter/pagination, native forms, detail/history
and consistent failure states. Local Linux Chrome and CLI evidence is recorded in
the application guide; independent Reviewer `/root/yt094_slice_review` PASS has
zero material issues or required followups. The later
[Windows native slice](YT-094-WINDOWS-VERIFICATION.md) passes source-matched MSVC
tests, CLI lifecycle and HTTP, with independent Reviewer PASS and zero issues or
required followups; browser/OS input remain open. YT-094 remains In progress:
OS-level IME/accessibility qualification, wider native/browser coverage and
production authentication/deployment gates are not claimed by this first slice.

**YT-093 bounded closeout (2026-10-18):** the
[signing verification](YT-093-SIGNING-VERIFICATION.md) records the opt-in signing
crate, local multi-key CLI, confirmed automatic settings updates and independent
Reviewer PASS. Linux and source-matched Windows native signing/key operations
pass; actual Linux CLI sample build/run and terminal confirmation pass. The
separate Azure CLI experiment verifies three remote signatures locally and
removes active lab resources, with disclosed soft-delete retention. The original
Windows exit-125 evidence remains retained; the separately reviewed
[CLI follow-up](WINDOWS-CLI-125-VERIFICATION.md) identifies the Cargo ACL refusal
and passes build/run/Ctrl-C cleanup with protected task-local tools. No cloud adapter, HTTPS feature or production
security/release qualification follows from this bounded Done status.

**YT-092 bounded closeout (2026-10-18):** the independent
[Issue Tracker starter](../examples/production-starter/README.md) exercises
SSR forms, a command JSON API and explicit SQLite persistence using existing
settings/CLI and optional modules. [Qualification](YT-092-STARTER-VERIFICATION.md)
records Linux 15/15 and native Windows GNU 16/16 tests, actual HTTP, persistence
across process restart, native console Ctrl-C, frozen CLI build/run, strict
Linux lint/format/docs checks and independent Reviewer `/root/yt092_review` PASS
with zero issues/followups. YT-092 is Done only at this reference-app scope;
public hosting/auth/HA, broader browser qualification and generator source
contract changes remain outside this closeout.

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

## M18 — Asset contract and Vite integration (Done at bounded scopes)

The [Vite feasibility spike](VITE-SPIKE-VERIFICATION.md) and
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
[implementation and real runtime evidence](YT-101-VERIFICATION.md) cover
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

The 2026-10-18 [CLI contract](CLI-CONTRACT.md) now has an explicitly authorized
[local implementation slice](YT-102-CLI-IMPLEMENTATION.md). The earlier
[design-only verification](YT-102-CLI-CONTRACT-VERIFICATION.md) remains historical.
Current Linux tests and Windows cross-compilation do not close native Windows
qualification, broader DX research, registry release or new generator formats.
YT-102's real journey audit, measurements and DX Extension Contract remain Todo.

**YT-103, opt-in HA and rolling asset compatibility:** independently of the
YT-102 developer workflow study, follow YT-101 with an application/deployment
strategy for multi-replica readiness/draining and for older SSR pages that still
request versioned CSS, JavaScript or lazy imports during and after a rolling
update. Compare object storage/CDN, a separate static asset service and a bounded
multi-version image approach before selecting a provider-neutral contract.
A future task execution must demonstrate two-version K3s rolling replacement,
old-tab lazy loading, rollback, a single-Pod disruption, retention and eventual
safe cleanup using real HTTP/browser evidence with measured 404/5xx outcomes and
independent Reviewer PASS. This registration authorizes **no** deployment,
provider integration, core coupling, release or production HA claim; YT-096
continues to record unsupported guarantees unless explicitly included in its
owner-approved supported candidate.

## M17 — Explicit build-time Tailwind CSS

YT-091 adds `yetlet new NAME --with-tailwind`, optionally combined with cloud
and typed configuration. Developers explicitly build root CSS with their
selected verified standalone Tailwind 4.3.3 executable before Cargo/run;
scaffolding, build/dev and requests never run or fetch a compiler. Literal Rust
classes feed a bounded startup CSS snapshot served at its SHA-256 URL with
strict CSP, CSS MIME, nosniff and immutable asset caching. Existing templates,
core/browser APIs and the twelve-sample catalog remain unchanged. All generated
source variants stay outside the deployment generators' accepted contract.
[Implementation evidence](YT-091-VERIFICATION.md) records actual checks and
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
[Evidence](YT-097-VERIFICATION.md) preserves these limits. Existing
Docker/Compose/Kubernetes generators reject the separate configuration cloud app.
Tailwind YT-091 is now separately authorized under M17 above. Certificates,
external vaults, hot reload, formal release and CI/CD remain deferred;
YT-020 remains Blocked.

## M15 — Opt-in Docker, Compose and Kubernetes generators

[YT-089d Kubernetes generator](YT-089D-VERIFICATION.md) is independently accepted by Reviewer `wc_job_1SJSI37ZToF_od0U` after actual Linux K3s client/server dry-run, Ready Pod, ClusterIP HTTP/CSP/health, scale-down, test image removal and namespace cleanup, with exact native GNU/MSVC byte parity. These were parent-operated limited tests separate from the coding Job. Final YT-089e current-source/documentation review independently passed (`wc_job_ci3bjlyeh2KHRhta`, zero issues/followups); M15 is Done at the limited unpublished generator scope, not production release.

Following reviewed YT-087/YT-088 commit `d24a05033c57e5b98dd57bf83189bc1297f6f28c`, the maintainer selected **generated configuration files over runtime platform coupling**. [YT-089](YT-089-CLOUD-NATIVE-GENERATORS-PLAN.md) is independently reviewed M15 design only (Reviewer `wc_job_9IplTI1HHkyZK9hv` PASS); [YT-089a opt-in cloud runtime](YT-089A-VERIFICATION.md) independently passed Linux bind/health/real SIGTERM/SIGINT/drain and native GNU/MSVC generated cloud builds (Reviewer `wc_job_0dvTDNAV22Vbpc5j` PASS). [YT-089b Docker configuration generation](YT-089B-VERIFICATION.md) is independently accepted by Reviewer `wc_job_IC92mEXoQMsG4nTg` after actual three-context Linux image build, HTTP/CSP/SIGTERM cleanup and bit-identical Windows GNU/MSVC output. Compose is independently accepted by Reviewer `wc_job_OCVGSYl5uAGET8w3`; Kubernetes is separately accepted by Reviewer `wc_job_1SJSI37ZToF_od0U` with real isolated Multipass K3s evidence and full cleanup. This does not authorize image publication, persistent K8s deployment or production release. YT-020 remains Blocked.

[YT-089c Compose](YT-089C-VERIFICATION.md) is independently accepted after source-bound
Linux Compose HTTP/SIGTERM/cleanup and exact native GNU/MSVC two-file parity.
[YT-089d Kubernetes](YT-089D-VERIFICATION.md) is independently accepted after actual schema/native parity and a parent-operated isolated Linux K3s deployment/HTTP/cleanup. The generator itself performs no cluster action. [YT-089e current-source closeout](YT-089E-VERIFICATION.md) passes Linux and GNU/MSVC Rust checks and preserves exactly the accepted source runtime semantics; independent final Reviewer `wc_job_ci3bjlyeh2KHRhta` PASS completes M15 at bounded unpublished config-generator scope.

## M13 — CLI developer preview

After independently accepted M12 commit `94cf8f555b554a26c2416842eab06110b229904d`, the maintainer authorized beginning [YT-087](YT-087-CLI-PLAN.md). Its Linux local `new`/`doctor` (087a) and frozen `build`/foreground `dev` (087b) slices have each received distinct independent Reviewer PASS after their bounded corrections. The 087c CLI documentation and isolated validation also passed final independent Reviewer `wc_job_avwDfY3wH233pvN7` (zero issues/followups). M13 is Done only at the unpublished Linux preview scope; none of these previews has been published. This is not a crates.io release or YT-020 qualification.

A later [two-runner native Windows verification](YT-087-WINDOWS-VERIFICATION.md) initially failed on offline toolchain prerequisites and, after explicit maintainer authorization for installation, **passed Windows GNU/MSVC compilation, Windows-applicable tests and expected fail-closed non-Linux CLI behavior**. This validates native portability of the currently implemented source, not functioning Windows `new/build/dev` features or a broader M13 release-support envelope.

## M14 — Platform-native Yetlet CLI parity

The maintainer clarified that Linux and Windows should share the same working CLI command contract. The previously verified Windows GNU/MSVC **compilation and expected refusals** are not functional Windows CLI acceptance. [YT-088](YT-088-CROSS-PLATFORM-CLI-PLAN.md) implemented two narrow internal OS adapter boundaries (safe filesystem publication and owned-process/terminal execution). Distinct 088a–d independent Reviewer PASS receipts now cover Linux preservation and native Windows GNU/MSVC positive new/build/dev, full SSR HTTP/CSP, console Ctrl-C/Break and resistant-child Job cleanup. Final 088e documentation/current-source regressions passed on Linux and Windows GNU/MSVC, with independent Reviewer `wc_job_IPC2lyaZyAz72KlT` PASS (zero issues/followups). M14 is Done at the bounded unpublished Linux/Windows x86-64 developer CLI scope, not at production or public-release scope. Linux YT-087 historical acceptance stays unchanged and YT-020 remains Blocked.

## M12 — Repository-toolchain planning checkpoint

The [Rust toolchain plan](M12-RUST-TOOLCHAIN-PLAN.md), [complete migration ledger](M12-PYTHON-MIGRATION.md) and [YT-082 verification](YT-082-VERIFICATION.md) file the dependency order `YT-081 → YT-082 → YT-083 → YT-084 → YT-085 → YT-086`. The independent directional backlog review passed; this does not accept implementation. YT-082 is Done after separate independent document review PASS; YT-083 is Done after corrected Linux/Windows MSVC tests and fresh independent Reviewer PASS; YT-084 is Done after [128-file builder parity, Rust checks and independent reviewer PASS](YT-084-VERIFICATION.md); YT-085 is Done after its separate independent Reviewer PASS, retaining its scoped evidence and unavailable environment boundaries. YT-086 is Done at bounded source/fixture migration scope after independent Reviewer `wc_job_6UfDDdHnNCBXkMhA` PASS ([evidence](YT-086-VERIFICATION.md)); full runtime-graph and environment qualifications remain unverified. The original YT-082 checkpoint changed documents only. YT-087 was Deferred in the original M12 plan; its separately authorized M13 unpublished Linux preview is now Done after its own independent review. YT-020 remains Blocked.

- [x] YT-082 plan and exact inventory independently accepted; documentation checks are recorded separately from software evidence.
- [x] YT-083 isolated unpublished xtask/doctor/process foundation independently accepted, including actual Linux and Windows owned-child cleanup without global host changes.
- [x] YT-084 seven builders and docs/catalog checks/tests accepted after parity and a separate independent Reviewer PASS.
- [x] YT-085 thirty scenario/helper mappings accepted after scoped runtime evidence, status/cleanup repairs and a separate independent Reviewer PASS.
- [x] YT-086 benchmarks/qualification/release/inline private rebuild and CI migration independently accepted at bounded source/fixture scope; controlled PATH proves executed commands interpreter-free, while unexecuted runtime graph remains Not verified.

Each later gate needs its own independent Reviewer PASS. Historical receipts and failed qualification remain immutable. A bounded language-port check cannot claim full-load, physical IME/AT, GPU performance, production qualification or release acceptance. No Rust/CI implementation, downloads/installs, hosted trigger, new validation framework, commit or push occurs in YT-082.

## M11 — Product readiness and optional scene rendering (selected delivery)

[M11 plan](M11-PLAN.md) files YT-069–YT-081 after the completed bounded research. M11-P targets one small persisted, authorized, usable SSR-first product with existing CSS; M11-G independently targets browser proof and possible first-party opt-in Experimental `yetlet-wgpu`. Neither line depends on public release, and product readiness does not depend on GPU. [Product research](M11-PRODUCT-READINESS-RESEARCH.md) preserves handoff workstreams A–E and exactly three priority gaps; [GPU plan](M11-WGPU-PLAN.md) preserves native/browser, lifecycle, performance and fallback boundaries. [ADR 0008](adr/0008-product-readiness-and-opt-in-scene-renderer.md) accepts only the bounded Experimental opt-in separation after owner approval and scoped implementation evidence, not stable/public qualification.

**Historical planning checkpoint:** YT-069 covers documentation and owner-permitted planning review only. At that checkpoint, YT-070–YT-081 were Todo and required separate activation. Documentation/static checks and [self-review](M11-PLANNING-REVIEW.md) cannot qualify a runtime or satisfy later independent implementation/security gates. The original nine candidates and YT-020 remain unchanged. The YT-068/68-item status below is a retained pre-M11 checkpoint, not the current expanded backlog count.

**Current non-GPU continuation (2026-10-14):** YT-070–YT-073 remain Done at authorized local scopes. [YT-074](YT-074-QUALIFICATION.md) is Done at the owner-amended local scope, with retained real Chewing IME PASS, repaired [Firefox/Orca spoken AT PASS](YT-074-DESKTOP-ACCEPTANCE.md) and final Chrome/Firefox PASS. The owner accepted five staggered cohorts for a new full-duration mixed workload; the original synchronized-burst failure remains a limitation on that Ubuntu host. The full 62-minute mixed run passed with 45,000 measured requests and zero errors; fresh independent review passed with no material findings. [Cross-environment synchronized evidence](YT-074-CROSS-ENVIRONMENT-VERIFICATION.md) adds native Windows/Azure Ubuntu full PASS at the original gates, with 44,850/45,000 actual formal requests, while retaining original Ubuntu FAIL and all instrumentation limits. The evidence supports an original-host system/storage limitation; no original-host repair or broader acceptance is claimed. [Handoff](M11-P-IMPLEMENTATION-HANDOFF.md) records completed documentation/static review and the subsequent conditional one-commit local delivery authority, with no new load or push. No GPU, release or deployment activation follows.

**Subsequent GPU activation (2026-10-14):** The owner activated YT-075 with dependent-item sequencing only after independent review. The application-owned GPU example passes focused native/WASM build, strict lint and Todo/Search byte-isolation gates; seven Linux Chrome configurations retained their unsuccessful GPU scene drawing attempts. Separate Windows Chrome and Edge baseline Javascript scenes using the same YT-075 WGSL passed 120/120 actual pixels but did not run the app's Wasm. The **selected Ubuntu Firefox 158 Stable (experimental WebGPU opt-in only) X11/Intel Vulkan** browser now PASSES the **actual Rust/wgpu scene Wasm**, 120/120 canvas pixels, semantic HTML fallback, CSP, independent counter, late-device/owner retirement, missing-asset and injected GPU failure checks. [Selected browser qualification](YT-075-UBUNTU-BROWSER-VERIFICATION.md) and [all attempts](YT-075-VERIFICATION.md) preserve this scope. YT-075 is **Done at the selected Ubuntu Firefox 158 X11 experimental-opt-in scope** after [fresh independent reviewer PASS](YT-075-INDEPENDENT-REVIEW.md), zero blocking issues; Chrome/NVIDIA/default Firefox, loss/recovery, dynamic performance and extraction remain outside this slice. YT-076 is now [Done at selected lifecycle scope](YT-076-VERIFICATION.md): 11 real Firefox lifecycle checks, resized/recovered 120/120 pixel samples, verified cleanup and independent reviewer PASS (0 issues). YT-077 dynamic comparison [implemented and measured](YT-077-MEASUREMENT.md): all 24 matched Canvas2D/WebGPU lanes and 126 real pixel checks passed in the selected Firefox X11 environment; no GPU speedup was demonstrated; [fresh independent reviewer](YT-077-INDEPENDENT-REVIEW.md) returned PASS (zero issues), so YT-077 is Done at this bounded scope. The owner then approved [Experimental opt-in promotion](M11-G-EXPERIMENTAL-PROMOTION-DECISION.md), without a performance guarantee, and an initial strict/builtin-guarded local commit `8c52452`. [YT-078](YT-078-VERIFICATION.md) picking and [YT-079](YT-079-VERIFICATION.md) extraction are Done with their individual independent reviewer PASS. [YT-080](YT-080-VERIFICATION.md) is Done at the selected Experimental scope after exact runtime-byte correspondence, focused final checks, a distinct no-WebGPU-preference default-profile fallback PASS and [separate independent Reviewer PASS](YT-080-INDEPENDENT-REVIEW.md), zero issues and zero required followups. YT-075–YT-080 are complete at their declared scopes; the YT-078–YT-080 source was additionally committed as `e499d24` under strict/builtin Commit Guard. [YT-081 M11 closeout](YT-081-M11-CLOSEOUT.md) and [cross-crate API audit](YT-081-API-CONTRACT-AUDIT.md) are Done at the selected scopes after independent documentary review and a separately reviewed [Experimental GPU typed-error/first-retirement refinement](YT-081-WGPU-API-REFINEMENT.md), with [new-source Reviewer PASS](YT-081-WGPU-API-INDEPENDENT-REVIEW.md) (zero issues). No push, release or YT-020 activation follows.

## Developer documentation refresh — YT-068

The [developer documentation index](README.md) and root README now separate native SSR setup, optional browser behavior and the nine unpublished candidates from retained qualification history. The nine candidate guides, current/history navigation and [final Private source/package boundary](YT-020-PRIVATE-RELEASE-PREP.md#current-preparation-boundary--t08--yt-068-2026-10-12) are reconciled. YT-068 is Done at documentation scope after bounded validation; the independent delivery gate/result is recorded separately in the run; documentation-stage checks do not attest that gate; YT-020 stays Blocked and established YT-030/YT-065/YT-066/YT-067 bounds are retained. YT-029 is Done at bounded deadline/overhead research scope; YT-021 is Done at bounded incremental SSR comparison scope, retaining the app-owned prototype and rejecting core streaming/suspense promotion. YT-024 is Done at bounded typed network adapter proof scope, adopting the app-owned adapter pattern and rejecting core transport framework promotion. YT-022 is Done at bounded data-only handler/capture research scope, retaining explicit application-owned descriptors and rejecting generic/core resumability promotion. YT-023 is Done at bounded native wgpu/Vulkan scene research scope, retaining an opt-in app-owned prototype with actual draw/readback and frame/allocation-byte observations while rejecting core/browser-portability promotion. The next unblockable YT-020 preparation action is the explicit immutable-candidate/source/package and formal release-policy decision record; any new preflight/rebuild/provenance/SBOM/license/signing execution needs separate authority. Historical `8e557e7` signed bytes do not attest policy HEAD `abb2cce` plus current dirty research, root manifest/lock and candidate READMEs. A later local commit alone cannot activate release. No release gate or historical qualification result changes through this entry refresh.

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

**M2 evidence (2026-10-08):** 16/16 Rust tests and Clippy PASS at the M2 gate. `scripts/build_browser.py` generated a hashed manifest and browser modules. The headless Chrome `scripts/smoke_browser.py` confirmed byte-identical SSR/DOM TodoList, state changes, and no remote event circuit. See [M2 verification](M2-VERIFICATION.md). Remote GitHub Actions execution was not verified.

### M3 — SSR islands and bounded state v1

- [x] `GET /search` renders a complete usable SSR input and result list without eagerly importing Search WASM.
- [x] Small external `search-loader.js` and versioned assets activate Search WASM on local interaction.
- [x] Input events remain in the browser; initial SSR results/input/heading are preserved.
- [x] `YT-017`: Explicit allowlisted, size-bounded Search Island v1 JSON state, escaped as an HTML attribute. Invalid versions fail closed and preserve readable SSR markup.

**M3 evidence (2026-10-08):** Native/WASM builds, `scripts/build_browser.py`, and headless Chrome `scripts/smoke_browser.py` verified lazy loading, DOM preservation, local search/empty/reset, versioned state acceptance, and incompatible-state fallback. See [M3 verification](M3-VERIFICATION.md). YT-011–YT-013 and YT-017 are complete at **prototype** scope; generalized resumability is not claimed.

### M4 — Explicit actions, API, and security

- [x] `YT-014`: Typed `Command`/`ActionExecutor` and injected `LocalExecutor` with unit and browser event-boundary evidence.
- [x] `YT-015`: Explicit same-origin-path `HttpExecutor` sends only semantic JSON commands; no raw DOM-event forwarding or automatic mutation retries.
- [x] `YT-016`: `GET/POST /api/todos` in-memory snapshot API validates body, command, and error status; failed mutations preserve state.
- [x] `YT-018`: Prototype strict CSP and browser checks, typed HTTP 4xx responses, nonmatching Origin rejection, and bounded JSON body handling.

**M4 evidence (2026-10-08):** Native Cargo/Clippy, browser WASM build and headless Chrome E2E demonstrate the local/HTTP action separation, one typed Create POST, server snapshot rendering, visible typed 422 error, and regression of SSR/Islands. See [M4 verification](M4-VERIFICATION.md). This is an **anonymous single-origin HTTP demo**, not an authenticated production security review.

### M4.1 — User-approved HTTP pipeline integration

This bounded extension comes from the Middleware Pipeline & RequestContext handoff and preserves the existing implementation sequence. See [ADR 0004](adr/0004-request-context-and-middleware.md) and [HTTP pipeline](HTTP-PIPELINE.md).

- [x] `YT-026`: Request-only context, request-local typed extensions and distinct application state.
- [x] `YT-027`: Native Tower pre-routing global layer, nested groups, route layers and explicit 404/405/error semantics.
- [x] `YT-028`: Actual API/SSR integration, streaming/request-isolation tests and browser boundary regression.

**M4.1 evidence (2026-10-08):** 46/46 tests, strict Clippy/format, English docs, browser compile and one headless regression PASS. Two browser WASM artifacts remain byte-identical. Native overhead/production assurance remain unmeasured; see [HTTP pipeline](HTTP-PIPELINE.md).
- [x] `YT-029`: [Opt-in deadline and measured native overhead research](YT-029-HTTP-DEADLINE-RESEARCH.md), completed 2026-10-12 at application-owned example scope. Adopt native Tower/Axum composition for explicitly selected use; general providers, supported core API promotion and production qualification remain deferred. Original P0 evidence above is unchanged.

### M5 — MVP release-evidence freeze

- [x] Reproducible **local** results for A01–A09, mapped to actual Rust/Chrome/Firefox tests.
- [x] Release-mode baseline measured for local SSR first-byte, HTML, gzip artifact estimates, Browser/WASM and Search Island readiness; environment and limitations recorded.
- [x] Documentation distinguishes locally tested prototype capabilities from production/public release gates.
- [ ] **Public-release authorization:** The license and bounded qualification work are complete. [Private experimental 0.1.0 preparation](YT-020-PRIVATE-RELEASE-PREP.md) selects the original nine libraries and prepared API/MSRV/maintenance scope; immutable-source/registry, trusted release integrity, public reporting/scanner activation and explicit approval remain open. Application deployment qualification is separate.

**M5 local evidence (2026-10-08):** [M5 acceptance and measurements](M5-VERIFICATION.md) documents A01–A10, release Rust/Chrome 155, Firefox 157 physical-keyboard E2E, 30 SSR timing samples, 5 Chrome navigation/activation samples, and asset sizes. `YT-019` is locally Done; `YT-020` is Blocked for publication, with explicit owners in [Release readiness](RELEASE-READINESS.md). GitHub CI has not executed the new M5 workflow changes.

**Later delivery/preparation update (2026-10-11):** The earlier hosted-CI statement above records the M5 checkpoint. Maintainer PR #4 and post-merge `a61f409` required CI passed. YT-065/066/067 retain their bounded completion and historical source identities. The maintainer authorized private `0.1.0` preparation only; local package verification is not a hosted run on these uncommitted changes, signed release evidence or publication approval. YT-020 remains Blocked for actual activation.

**Private integrity update (2026-10-12):** PR #5 delivered the original package preparation as `633db585`; required PR/master runs `38144070219` / `38145022739` passed with the original check names. The maintainer then selected Rust 1.99 before first publication. [Integrity preparation](YT-020-INTEGRITY-PREPARATION.md) records byte-identical nine-crate source archives from separate Ubuntu/Windows hosts, honest shared-control limits, target inventories, unsigned provenance drafts and public verification fixtures. These working overlays have no new hosted CI or release authority. Protected-check migration and the existing concrete activation gates remain open; YT-020 stays Blocked.

**Later authorized Private follow-up (2026-10-12):** Reviewed integrity preparation was committed as `9ec9b7b`. PR #6 first-head run `38193149046` passed both new jobs on tree-equal synthetic checkout `cdc67bb6`; protected checks were then atomically migrated to Rust 1.99 MSRV plus the unchanged Stable job, with all other controls retained. [Hosted rebuilding](YT-020-HOSTED-REBUILD.md) prepares a single manual GitHub-operated builder for the same immutable source as the primary. Later candidate/merge/rebuild evidence is retained separately and never attributed to the first-head run. Signing and actual release activation remain open; YT-020 stays Blocked.

### M6 — Handler-first developer API and opt-in modules (locally complete)

**Goal:** Let developers build a small SSR site or typed JSON API with ordinary Rust handlers and components, without mandatory MVC Controllers, source layouts, model binders, DI containers or result hierarchies. Expose a **thin Axum/Tower facade**, not a second router. Keep all optional capabilities explicitly installed and browser-safe. See [ADR 0006](adr/0006-handler-first-mvc-optional.md) and [ADR 0005](adr/0005-opt-in-capability-modules.md).

- [x] `YT-030 / M6a`: Thin `App<S>` facade supports ordinary GET/POST handlers, native `MethodRouter`, nested groups, scoped middleware, explicit state and raw Router escape hatches.
- [x] `YT-031 / M6b`: Page and async View route convenience; native typed inputs/results/rejections, redirects and errors; RequestContext remains optional.
- [x] `YT-032 / M6c`: configure/try_configure execute typed setup functions once; no plugin registry, service locator or mandatory provider dependency.
- [x] `YT-033 / M6d`: Independent yetlet-request-id crate proves opt-out, scoping, error/short-circuit/concurrent behavior and core/browser dependency isolation.
- [x] `YT-034 / M6e`: Todo migration preserves SSR, API and middleware contracts; Chrome/Firefox regression and release before/after measurement completed. No zero-overhead performance claim.

**M6a local evidence (2026-10-09):** Five App Builder integration tests and a compiling rustdoc example passed; the existing RequestContext/Tower integration remains intact. [M6a verification](M6A-VERIFICATION.md) documents API surface, native semantics and remaining gaps. This is neither the M6b result API nor a performance/production release claim.

**Complete M6 local evidence (2026-10-09):** 66 workspace tests and 4 doctests, strict Clippy, native/minimal/wasm32 builds, Chrome 155 and Firefox 157 regressions passed. Release asset hashes and HTML sizes are unchanged. The same-session native first-byte median changed from 0.019 to 0.033 ms; historical M5 was 0.034 ms. See [M6 verification](M6-VERIFICATION.md) for unrounded methodology/limits and [Extensions](EXTENSIONS.md) for the actual composition contract. Production/release gates remain separate.

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

**M7a local evidence (2026-10-09, retained report `2026-10-09T02:02:22+00:00`):** 200 rows, 500 updates, 10 measured rounds in Chrome 155. Median total round: subtree replacement 213.500 ms; positional View patch 41.700 ms; keyed direct DOM 2.000 ms; static template slots 1.900 ms. All strategies produced equivalent final states. Only keyed/slot prototypes preserved the untouched keyed row after reorder; whole replacement also lost owned input identity/focus. Production Browser/Search digest remained `3f3c5f074478c525`. See [M7a verification](M7A-VERIFICATION.md). This is architecture evidence, not a selected production renderer or framework comparison.

**M7b local evidence (2026-10-09):** Four native authoring/SSR tests, one compiling rustdoc example, wasm32/release build and an isolated Chrome 155 E2E passed. Mismatched instance/key or actual child-node/slot shape fails before binding; successful bind preserves SSR root/row/label/text identity, handles local typed events and conditional insertion/removal, preserves keyed rows across reverse/append/remove, and disposes listeners explicitly without stale handles affecting newer owners. Production digest remains `3f3c5f074478c525`. See [M7b verification](M7B-VERIFICATION.md).

**M7c local evidence (2026-10-09):** Thirteen reactive-core tests plus the existing four M7b tests and one doctest pass. The isolated Chrome fixture verifies a three-message/one-commit batch, deferred re-entrant dispatch, explicit observer/owner disposal, recoverable reducer failure, faulted renderer failure with keyed DOM identity preserved, and no HTTP/WebSocket circuit. Reviewer regressions prove nested subscription cleanup, caller iterator/state release and reducer/commit failure cleanup occur outside graph borrows without retaining replayable queue work; reducer-triggered owner disposal skips remaining messages/commit. See [M7c verification](M7C-VERIFICATION.md).

**M7d local evidence (2026-10-09):** The fixed keyed row adds one allowlisted controlled text input. Chrome 155 and Firefox 157 preserve its node identity, live value, focus and selection through keyed reorder; focused row removal and owner disposal remove input/click listeners so detached events cannot update the graph. Native SSR escaping/authoring checks and strict CSP remain intact. See [M7d verification](M7D-VERIFICATION.md).

**M7e local evidence (2026-10-09):** 86 workspace tests, 5 doctests, strict Clippy/format, four-package wasm32 checks, production Chrome 155 and Firefox 157 regressions, the complete M7a workload, and isolated M7b/M7c/M7d Chrome/Firefox release regressions pass. Production Browser/Search assets remain byte-identical at `3f3c5f074478c525`, and dependency graphs exclude both M7 experimental crates. The outcome is a no-decision evidence freeze: several small primitives are candidates for later extraction, but no current M7 API or renderer is promoted. See [M7e verification](M7E-VERIFICATION.md).

**M7f local evidence (2026-10-09):** `yetlet-core` now owns validated `ComponentIdentity`, `BindingKey`, `SlotName`, `InertAttributeName` and `SlotDescriptor` contracts. The isolated authoring prototype consumes those types while retaining its experimental fixed shape. Five core tests, seventeen prototype tests, one doctest, strict targeted Clippy and wasm32 compile pass. See [M7f verification](M7F-VERIFICATION.md).

**M7g local evidence (2026-10-09):** The exact M7c graph implementation and its thirteen ownership/batching/re-entrancy/failure/teardown tests now live in dependency-free `yetlet-reactive`. The keyed-binding fixture uses a compatibility re-export and retains four authoring tests plus one doctest. Strict targeted Clippy and native/wasm32 checks pass. See [M7g verification](M7G-VERIFICATION.md).

**M7h local evidence (2026-10-09):** Optional `yetlet-dom-keyed` now owns unique-order validation, keyed entry state, reconciliation and disposable listener lifetimes. The existing fixed-shape fixture consumes it and passes Chrome 155/Firefox 157 reorder, form-state and teardown regressions. Direct/feature-gated placement in `yetlet-dom` changed production bytes; the independent crate restored the canonical production digest `3f3c5f074478c525`. See [M7h verification](M7H-VERIFICATION.md).

**M7i local evidence (2026-10-09):** The production Todo Browser local list uses `yetlet-reactive` plus `yetlet-dom-keyed`, while the remote typed HTTP path and Search Island remain separate. Chrome 155 and Firefox 157 preserve local list/row identity, verify queued click→dispose retirement, ignore a delayed HTTP completion after disposal/remount, and retain SSR/CSP/API/Search regressions. The repaired production digest is `faebb0709dccd863`; the Browser runtime increases by 82,726 raw / 20,042 local-gzip bytes relative to M7e, while Search JS/WASM remain byte-identical after separate package builds. See [M7i verification](M7I-VERIFICATION.md).

**M7j final evidence (2026-10-09):** Replacement validation passes 91 workspace tests, 5 doctests, strict Clippy/format, six-package wasm32, debug/release Chrome 155 and Firefox 157 regressions including queued/in-flight lifetime races, the full M7a workload, isolated keyed lifecycle suites, dependency checks and a repeated release baseline. ADR 0007 selects only bounded identity/graph/keyed-DOM primitives and the Todo local-list adoption. Production digest is `faebb0709dccd863`; Search remains byte-identical. The first final reviewer found the lifetime P2 that this source repairs; repeat reviewer `wc_job_XZzdXEOIx6owj1o5` returned PASS with no issues or follow-ups. See [M7j verification](M7J-VERIFICATION.md).

### M8 — Sample-driven application experience

**Goal:** Validate that Yetlet's existing handler-first, component-first and opt-in capability model produces understandable applications with low accidental boilerplate. Samples are evidence, not excuses to add Controllers, DI, a macro DSL or a general client framework.

- [x] `YT-044 / M8a`: Standardize a versioned runnable-sample catalog, local READMEs and validation gate.
- [x] `YT-045 / M8b`: Add an API-only REST sample with no browser/WASM dependencies.
- [x] `YT-046 / M8c`: Add an SSR shopping cart with an explicit opt-in Session provider and bounded security contract.
- [x] `YT-047 / M8d`: Audit repeated boilerplate and dependency cost across the three applications.
- [x] `YT-048 / M8e`: Run the complete regression and freeze the M8 evidence/architecture decision.

**M8a local evidence (2026-10-09):** `examples/samples.toml`, sample-local READMEs and `scripts/check_samples.py` now validate repository-contained paths, exact launch commands, complete documented routes, companion/browser mappings and direct optional dependencies. The focused negative suite covers false declarations and malformed types; 10 Todo tests pass. Final reviewer `wc_job_vQ6pGDE0slh9gV8c` returned PASS with no issues. M8a changes no runtime behavior. See [M8a verification](M8A-VERIFICATION.md).

**M8b local evidence (2026-10-09):** The API-only Book CRUD sample passes 7 tests, strict Clippy, a real localhost smoke, catalog/docs gates and native dependency isolation. Independent reviews found routing-contract, authority-parser and no-Origin Host-validation defects; all material findings were repaired with focused regressions and complete M8b validation. Final reviewer `wc_job_d911MsTpbwT6fZY3` returned PASS with no issues or required follow-ups. See [M8b verification](M8B-VERIFICATION.md).

**M8c local evidence (2026-10-09):** The independent `yetlet-session` provider and native-only SSR cart implement random opaque identity, absolute expiry/capacity, fixation resistance, cookie lifecycle, session-bound CSRF, refresh persistence, bounded cart/form state and explicit absence from unrelated dependency graphs. Stage reviewers repaired atomic-rotation, stale-cookie and non-loopback-listener defects; M8e later found and repaired the stale-destroy removal-cookie variant. Final replacement validation passes 114 tests and 5 doctests. See [M8c verification](M8C-VERIFICATION.md).

**M8d local evidence (2026-10-09):** A reproducible audit compares application-owned Rust source, tests, routes, direct dependencies and optional capabilities across Todo, REST and Cart. Repeated patterns are either already handled, small native bootstrap or application/security-specific; no runtime API promotion is justified. Reviewer `wc_job_lZSDSBgMKWu1X5X7` returned PASS with no issues or required follow-ups. See [M8d verification](M8D-VERIFICATION.md).

**M8e final evidence (2026-10-09):** The complete workspace/native/browser/sample matrix, release artifact/dependency isolation, retained render/keyed fixtures, local baseline and M8d metrics pass. Final reviewer `wc_job_dospP8AM0wUmvHFd` found one stale-destroy cookie race; it was repaired with a deterministic two-request regression and 114-test replacement validation. Repeat reviewer `wc_job_J_MVNzFaVgTH1Gr1` returned PASS with no issues or required follow-ups. M8 is locally complete. See [M8e verification](M8E-VERIFICATION.md).

### M9 — Sample expansion and API contract qualification (implementation underway)

**Goal:** Challenge M8 application ergonomics using the original nine-sample catalog sequence plus three targeted supplements, then classify real API compatibility maturity. The [M9 implementation plan](M9-PLAN.md) is the authoritative scope and test matrix; M9p planning is complete, and implementation begins with bounded independently reviewed samples.

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

**Explicit data-access boundary:** No built-in Yetlet ORM, ORM runtime, mandatory repository/DbContext, query DSL, migration framework or hidden database provider. YT-054 evaluates existing SQL-first `rusqlite` or `sqlx`-style direct/typed query libraries as **app-selected optional dependencies**. Any third-party ORM is external and opt-in only, never a default dependency. See [M9 plan](M9-PLAN.md#3-sql-first-data-access-policy-no-yetlet-orm).

**Planning evidence (2026-10-09):** [M9 planning verification](M9-PLANNING-VERIFICATION.md) records 40 documents, 59 backlog items, the unchanged three-entry runnable catalog, SQL-first opt-in policy, complete task DAG and reviewer `wc_job_fr93PI1YLOtR3KjE` PASS with zero findings. At the planning freeze YT-049 was Done and YT-050–YT-059 were Todo; current per-stage progress follows below.
**M9a / YT-050 evidence (2026-10-09):** A distinct, cataloged native-only `yetlet-hello-world` proves deterministic SSR, an escaped dynamic route, strict CSP, native 404/405 and absence of optional providers or browser dependencies. Workspace tests 117/117, doctests 5/5, strict Clippy, real HTTP smoke, four-sample catalog and native release build pass; independent reviewer `wc_job_Htk-Quo_aFExMJs3` returned PASS. See [M9a verification](M9A-VERIFICATION.md).
**M9b / YT-051 evidence (2026-10-09):** The native-only Middleware Lab demonstrates global/group/method routing, short-circuit, typed RequestContext state, real third-party Tower interoperability and streamed/failing bodies. Workspace tests 124/124, doctests 5/5, strict Clippy, localhost smoke, five-sample catalog and optional-dependency isolation PASS; independent reviewer `wc_job_UnKflDaA8ZQHXCce` returned PASS with no issues. See [M9b verification](M9B-VERIFICATION.md).
**M9c / YT-052 evidence (2026-10-09):** Native SSR form validation and semantic error associations, escaped untrusted attributes, bounded 2 KiB bodies, strict CSP and no-side-effect 303 PRG pass 130 tests, five doctests and a real localhost smoke. First reviewer `wc_job_7D2rtHdGEfwW1Aet` found an inherited-language accessibility P2; a sample-owned `lang="en"` override on both main views and GET/error/success regressions repaired it. Repeat reviewer `wc_job_lGtk_ZIoAvK4F6PW` returned PASS with no required followups. No DB/ORM/Session/CSRF claim. See [M9c verification](M9C-VERIFICATION.md).

**M9d / YT-053 evidence (2026-10-09):** The bounded File Server passes 14 focused tests, 144 workspace tests, five doctests, full Clippy/format, six WASM packages and one localhost smoke including interrupted-upload cleanup. Browser/Search release digest remains `faebb0709dccd863`, all 136 retained assets byte-identical; file dependencies stay application-local. Independent read-only reviewer `wc_job_g8JSu80-Jseg6lmG` returned PASS with no findings, and runtime telemetry matched the saved `gpt-6-astra`/high configuration. The exact reviewed seventh catalog entry was promoted. See [M9d verification](M9D-VERIFICATION.md).
**M9e / YT-054 evidence (2026-10-09):** A bounded `rusqlite` versus compatible `sqlx` spike selected direct application-local `rusqlite`; explicit SQL/migration/transactions run behind a bounded `spawn_blocking` gate. Nine focused tests, 153 workspace tests, five doctests, three-process persistence/concurrency smoke, 22 dependency trees, six WASM packages and 136 byte-identical production assets pass. First reviewer `wc_job_Q3pBfUqPYGYNSIrD` found incomplete required-column validation; the fail-closed schema check and regressions were repaired, the full matrix rerun, and repeat reviewer `wc_job_aVBprnEsYc8o3ubi` returned PASS. The exact eighth catalog entry was then promoted. See [comparison](M9E-DATA-ACCESS-COMPARISON.md) and [M9e verification](M9E-VERIFICATION.md).
**M9f / YT-055 evidence (2026-10-10):** Native HTTP SSE uses an application-owned bounded Hub with finite IDs, a 16-event replay buffer, eight subscribers, four-event per-client queues, heartbeat comments and disconnect cleanup. Ten focused tests, 163 workspace tests, five doctests, real heartbeat/live/reconnect smoke, all 23 package trees, six wasm32 packages and 136 byte-identical production assets pass. First reviewer `wc_job_LF-i2IsQxFXO-obd` found control characters could be trimmed before validation; raw-input rejection and regressions repaired it, the full matrix reran, and repeat reviewer `wc_job_fa8KJiTxGRuWIEDF` returned PASS. See [M9f verification](M9F-VERIFICATION.md).
**M9g / YT-056 evidence (2026-10-10):** Complete SSR hosts separately built Counter/Filter islands with independent manifests, versions, listeners and disposal. Six server tests, 169 workspace tests, five doctests, release Chrome/Firefox identity/focus/late-callback/failure-recovery and missing/malformed-manifest fallback pass. Eight wasm32 packages, all 26 dependency trees and 136 byte-identical production assets pass. Reviewers `wc_job_63A__nJFK3BgNWPk` and `wc_job_6pk1_UnsnH8WR4Ns` exposed missing browser fallback and retained-log evidence; both were repaired without runtime expansion, and final reviewer `wc_job_OMstvGrtDiKZ5p6C` returned PASS. See [M9g verification](M9G-VERIFICATION.md).
**M9h / YT-057 evidence (2026-10-10):** Loopback-only synthetic terminal codes authenticate Alice/Bob; `yetlet-session` separately owns opaque identity/CSRF; every note route enforces server-owned ownership. Login/logout rotate Session ID and CSRF. Twelve focused tests, 181 workspace tests, five doctests, a real two-user smoke, all 27 dependency trees, eight wasm32 packages and 136 byte-identical production assets pass. Security-focused reviewer `wc_job_EgSLng_VafIvs3ju` returned PASS with no findings; the exact eleventh catalog entry was promoted. See [M9h verification](M9H-VERIFICATION.md).
**M9i / YT-058 evidence (2026-10-10):** Identity, catalog, Session cart and direct-SQL orders compose through ordinary Rust modules. Nine focused tests, 190 workspace tests, five doctests, a three-process restart/authorization/disabled-module smoke, 28 dependency trees, eight wasm32 packages and 136 byte-identical production assets pass. First reviewer `wc_job_a_WnhFSftzNoYYgf` found a readiness-failure leak in the Python smoke harness; terminate/kill/reap/stream-close regressions repaired it, and repeat reviewer `wc_job_zbWS7OSbUtaKzYcn` returned PASS. The exact twelfth catalog entry was promoted. See [M9i verification](M9I-VERIFICATION.md).
**M9j / YT-059 evidence (2026-10-10):** The [API inventory](API-CONTRACTS.md) classifies 170 public declarations/constants. Complete regression passes 190 tests, five doctests, 11 native smokes, eight wasm32 packages, three Chrome and three Firefox suites, 28 dependency trees and 136 byte-identical production files. First reviewer `wc_job_BzkzTPcKr4FKATTr` found omitted constants/errors and over-broad upstream reexport classification; both were repaired without runtime changes, and repeat reviewer `wc_job_QkQkovlBlvrhPxCE` returned PASS. Stable Candidates remain pre-SemVer evidence; YT-020 remains Blocked. See [M9j verification](M9J-VERIFICATION.md).

**Historical trust/registry policy checkpoint (2026-10-12):** [Private public-key preparation and future crates.io distribution](YT-020-PRIVATE-TRUST-POLICY.md) are selected for the original nine experimental `0.1.0` libraries. Only documentation and independent review are authorized now. Actual key/signing, real registry rights/resolution and formal release integrity/approval remain unverified; OIDC, certificate/chain, transparency/inclusion and builder/source guarantees are preserved. YT-020 stays Blocked and no repository visibility transition is implied.

**Later authorized Private signing (2026-10-12):** The reviewed policy was locally committed as `abb2cce`. [Dedicated-key signing and separate verification](YT-020-PRIVATE-SIGNING-VERIFICATION.md) now pass for three preparation signatures, 37 artifacts and six negative cases. Formal log-required verification rejects the no-log bundle; all release guarantees remain mandatory. Source/build/inventory identities are retained without broad reruns. YT-020 stays Blocked for formal activation; no push, publication or visibility change occurred.

### M10 — Release engineering and production-security qualification

**Goal:** turn prototype evidence into explicit support, governance and production-security decisions without expanding Yetlet core or implying release approval.

- [x] **YT-060 / M10a:** Dual `MIT OR Apache-2.0` license, package metadata, security policy and maintainer-directed/no-external-PR contribution policy; independent reviewer PASS.
- [x] **YT-061 / M10b:** Collaborator-only PRs, protected `master`, required checks, SHA-pinned GitHub-owned Actions and Dependabot configured; unsupported private-state security services recorded; repeat reviewer PASS.
- [x] **YT-062 / M10c:** Minimal title/language document metadata, provisional compatibility policy, legacy generic compile fixtures, complete stable/MSRV validation and repeat reviewer PASS.
- [x] **YT-063 / M10d:** Product-neutral production edge/authentication threat model, private-cache/CSRF repair, documentation validation and repeat security-focused reviewer PASS.
- [x] **YT-064 / M10e:** Release design/preflight, actual license/source checker, 41 native CycloneDX SBOMs, 16 checksum tests and independent reviewer PASS; not signed/reproducible release activation.
- [x] **YT-065 / M10f — Done at automated qualification scope:** Hosted run `38043565238` passed Ubuntu, Windows MSVC, macOS ARM64 and Linux Firefox jobs; local Windows GNU and offline Docker passed. Real IME, assistive-technology speech, WebKit/mobile and zoom remain explicitly Not verified optional coverage, not supported-platform or conformance claims.
- [x] **YT-066 / M10g — Done at single-host Docker multi-instance and selected Redis-provider scope:** Actual local Session isolation/restart and SQLite operations passed with reviewer. YT-066a is complete at single-host Docker/Memory proof scope; b2 implementation and [b3](YT-066B3-VERIFICATION.md) single-host/single-primary/no-persistence Redis qualification passed independent review and were locally committed. The [responsibility clarification](YT-066-PERSISTENCE-RECOVERY-PLAN.md) separates framework semantics/configuration from application operations. The status and limitations are reconciled in [M10g closeout](M10G-VERIFICATION.md#yt-066-bounded-framework-closeout--2026-10-11), with a separate final documentation review gate. Production TLS/ACL policy, backups, RPO/RTO and operational ownership do not block YT-066. Broader durability/HA/load/deployment guarantees remain unverified, with no new execution.
- [x] **YT-067 / M10h — Done (internal source/fix scope):** Official Codex Security Standard scan `fc4d4af8-5864-4ace-b098-23ebfe249b31` completed on the pre-patch baseline with 227/227 regular files reviewed and one Low finding. Tiny endpoint-validation correction passed Rust/MSRV/WASM/Chrome regressions and read-only reviewer `wc_job_sTA8TdP3N3Veu9uC` PASS. The historical earlier scan remains incomplete; no post-patch complete rescan or external certification is claimed.

**Post-M9 hardening evidence (2026-10-10):** [CI/security review](CI-SECURITY-REVIEW.md) records the first hosted CI failure, bounded browser-harness repair, full-SHA Action pins, read-only permissions, patched `RUSTSEC-2026-0009`, Rust 1.88 verification and the remaining GitHub/security/release gaps. GitHub Actions run `38024582602` completed successfully for commit `86c30a42114ca74f4197f0389707861308a844a7`, including both required jobs and all Chrome release suites.

**M10 sequencing decision (2026-10-10):** The maintainer selected `MIT OR Apache-2.0` and a maintainer-directed model that does not accept public pull requests. YT-060/YT-061 now implement those policies without changing repository visibility. YT-062 retains the tiny/handler-first contract: title plus validated root language only, no arbitrary head manager, CSP DSL, DI, MVC or plugin subsystem. See [M10a](M10A-VERIFICATION.md), [M10b](M10B-VERIFICATION.md), and [M10c planning verification](M10C-PLANNING-VERIFICATION.md).

**M10a/M10b evidence (2026-10-10):** All 28 workspace packages carry `MIT OR Apache-2.0` and the canonical repository URL; all 19 examples are nonpublishable, and all nine library package archives contain both license texts. Reviewer `wc_job_MPF4l2t8JaJXBguv` returned PASS. GitHub remains Private with collaborator-only PR creation, squash-only merging, admin-enforced protected `master`, both required CI checks, GitHub-owned SHA-pinned Actions, read-only workflow tokens and Dependabot alerts/security fixes. First governance reviewer found missing check-run evidence and stale monitoring wording; both were repaired, and repeat reviewer `wc_job_lu2WynqvKYemLlmd` returned PASS. Public-only PVR/secret-scanning/CodeQL activation remains a visibility-transition checklist item.

**M10c / YT-062 evidence (2026-10-10):** Legacy page/render signatures remain; `LanguageTag`, `DocumentMetadata`, metadata-aware render functions and one compile compatibility fixture add explicit root-language ownership. Forms Validation adopts root `lang="en"` and removes its nested workaround. First reviewer `wc_job_VWOJoR-lV3PAyS_-` found a narrowed `Page::new` generic bound; the exact `Into<Cow<'static, str>>` contract and borrowed/owned generic regression were restored, complete stable/MSRV evidence reran, and repeat reviewer `wc_job_rz9Ci3gWOLfLKdqN` returned PASS. The bounded API remains Experimental; see [M10c verification](M10C-VERIFICATION.md).

**M10d / YT-063 evidence (2026-10-10):** Planning reviewer `wc_job__wg1bHF6DG3ztjjg` passed. The [Threat Model](THREAT-MODEL.md) defines a product-neutral TLS edge, forwarding-header, OIDC, Session/CSRF, authorization, rate-limit, CORS, secrets, telemetry and availability contract while keeping those capabilities external/opt-in. First completed reviewer `wc_job_UPiINEJpZ28tLFyl` found missing private-cache isolation and an overbroad CSRF-token prohibition; both were repaired, documentation validation reran, and repeat reviewer `wc_job_AlEmAzdabqC_tuHv` returned PASS. The document remains requirements evidence, not production certification; see [M10d verification](M10D-VERIFICATION.md).

**Remaining M10 execution evidence (2026-10-10):** [Approved plan](M10-REMAINING-PLAN.md), [release preflight](M10E-VERIFICATION.md), [compatibility matrix](M10F-VERIFICATION.md), [local operations](M10G-VERIFICATION.md) and [internal security scan verification](M10H-VERIFICATION.md) keep executed evidence separate from optional unperformed manual coverage, the distributed-provider work required at that historical stage and release gates. The later YT-066 closeout above qualifies only its selected experimental provider; broader deployments remain unverified. No default runtime dependencies, core subsystems or ORM were added.

## Post-MVP research

| Track | Research question |
|---|---|
| Streaming SSR | [YT-021 comparison](YT-021-STREAMING-SSR-RESEARCH.md) retains app-owned incremental fragments; large/delayed first-body gains trade off lower throughput, so supported-core streaming/suspense promotion is rejected |
| Compile-time template IR | M7b–M7e prove one fixed shape; M7f–M7i promote only low-level contracts and one production keyed-list path. General trees remain required before any macro decision |
| Resumability | [YT-022 study](YT-022-RESUMABILITY-RESEARCH.md) proves two finite versioned local handlers with bounded numeric captures and native size/cost evidence; retain explicit app-owned descriptors, reject generic/core resumability promotion |
| Custom transport | [YT-024 proof](YT-024-CUSTOM-TRANSPORT-RESEARCH.md) uses existing ActionExecutor for one opt-in typed TCP adapter with bounded loopback/error/cancellation evidence; adopt the app-owned pattern, reject core framework promotion |
| WebGPU renderer | [YT-023 study](YT-023-WEBGPU-RESEARCH.md) proves opt-in native wgpu/Vulkan draws at 10k–100k with raw completion-wall-time and explicit allocation-byte observations; retain the app-owned prototype, reject core/browser-portability promotion; no matched DOM/CPU speedup is established |
| Worker compute | Which computations can leave the main thread without compromising DOM correctness? |
| Tooling | When do HMR, bundler integration, and devtools justify their complexity? |

## Decision rule

For every proposed feature, identify its owner: kernel, renderer, transport, server adapter, or optional tooling. If ownership is unclear or responsibilities overlap, document the decision or run a bounded spike before adding core abstractions.
