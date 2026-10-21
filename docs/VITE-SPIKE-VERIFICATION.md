# VITE-SPIKE — Asset integration feasibility and follow-up plan

Date: 2026-10-17. Base commit:
`6d1de322739fb3defd32dccd1f6ae3790aecdddd` (the owner-authorized YT-091 commit).
Branch: `feat/yt090-configuration`. **The bounded experiment passed actual Linux
build/native/browser checks and separate independent review of the experiment
and planning.** Formal integration items below remain Todo, not implemented.

## Authority and execution history

The owner requested committing accepted YT-091, then experimenting with Vite and
registering future items if the experiment succeeds. YT-091 was locally committed
after a strict/builtin staged guard returned zero blockers and one illustrative
Windows-path warning. Two trailing blank lines were removed; the scaffold's
existing `trim_end()` was checked to preserve generated handler bytes. No push,
merge, auto-merge, release or deployment was authorized or performed.

The initial role-directed executor Job `wc_job_MXjlDUIEJIPAaSIM` lost its Runner
association when the Runner process was replaced (`runner_instance_replaced`).
It produced source and local Rust evidence, but no terminal role result; it is
**not accepted as a successful agent Job**. Its isolated environment denied
network access, and npm could not reach the public registry. The coordinator
stopped only the exact owned stalled npm PID after validating its cwd/command.
After the lost Job, a fresh process inspection confirmed that no original
executor or npm work remained active. Only then were the canonical generalist
role and repository fallback policy reread.

The current ChatGPT session completed the same bounded task as
**current-session role emulation**, using ordinary authorized Runner dependency
provisioning and validation. No replacement executor was launched. This emulated
work makes no claim to the saved role model, resolver effort, native identity or
Job telemetry. Public packages were explicitly installed into the isolated
experiment, without changing a global toolchain or weakening sandbox policies.
The independent-review gate is separate and cannot be fulfilled by this
emulation or by its self-inspection.

## Source boundary and choices

The [standalone experimental workspace](../experiments/vite-integration/README.md)
contains only application-owned Rust, Vite frontend sources, pinned package/Cargo
lockfiles and a focused browser test. Existing core/renderer/CLI/template and root
Cargo files are unchanged after the YT-091 commit. The experiment is not added to
the canonical 12-sample catalog and does not activate `--assets vite`.

Two Vite 8.3.4 production builds use the same Rust loader: plain CSS/TypeScript,
and `@tailwindcss/vite` 4.3.3 plus Tailwind 4.3.3. TypeScript 7.0.2 checking is
separate. Node 24.21.0/npm 12.2.0 were used only for tooling, with Playwright Core
1.64.1 and installed Chrome for QA. The plain build does not activate Tailwind;
the shared experimental package still contains both sets of devDependencies.

The manifest's static imports, dynamic imports, CSS and SVG references are
validated and loaded into a bounded Rust startup snapshot. HTML comes from
Yetlet, links external same-origin CSS/scripts and preserves `DEFAULT_CSP` and
escaping. All HTTP responses use `no-store` and `nosniff`; this proof makes no
immutable-cache or filename-hash authenticity claim. No public directory copy,
data-URI inlining, source map or dotenv discovery is enabled. Node/Vite does not
run inside HTTP handling or perform Rust SSR.

## Actual checks

| Boundary | Observed result |
|---|---|
| Public dependency installation | Local pinned npm installation with `--ignore-scripts`, no global install; five direct versions confirmed |
| TypeScript | `npm run typecheck`: exit 0 |
| Actual Vite plain and Tailwind builds | `npm run build`: exit 0; both emit entry, shared, lazy JS/CSS and external SVG in real manifests |
| Rust tests | `cargo test --offline --locked`: 9 passed, no ignored or filtered tests, including both emitted manifests |
| Rust formatting, strict Clippy, build | `cargo fmt --all -- --check`, `cargo clippy --offline --locked --all-targets -- -D warnings`, `cargo build --offline --locked`: all exit 0 |
| Reprovision and repeat build | Offline `npm ci --ignore-scripts`, typecheck and build: exit 0; all 18 output files and package-lock bytes unchanged |
| Synthetic backend-secret exclusion | Sentinel absent from both builds and served HTML; no actual secret inspected |
| Actual Chrome production journey | Both plain and Tailwind profiles PASS at 840 x 650 using Chrome 155.0.8059.84 |
| JS-disabled SSR | Both profiles retain escaped useful SSR text and unloaded lazy state |
| CSS, image and interaction | Actual computed plain/Tailwind/shared CSS, external SVG natural width, two clicks, lazy module text and lazy CSS changes PASS |
| CSP and requests | Zero recorded CSP violations/console errors, all observed requests same-origin; lazy JS/CSS return 200 with expected MIME |
| Invalid HTTP asset requests | Six unsupported paths per profile return 404, including private manifest/traversal/query cases |
| Resource cleanup | Browser closed; owned Rust server exited 0; selected port independently confirmed closed |

The nine Rust tests cover schema and metadata validation, unsafe paths/types,
missing static/dynamic references, missing files/entry, cycles, size bounds,
symlink escape rejection, deterministic CSS dedup/traversal and actual manifests.
The fixture is intentionally narrower than full Vite semantics: one SVG,
dependency-first CSS order and cycle refusal. It is not a hostile-filesystem
race guarantee. CSS order and compatibility are explicit design work for YT-098.

The remote Browser control runtime/skill was not available for the Ubuntu test;
the connected Browser surface exposes read-only tabs. The announced test used
installed headless Chrome through Playwright, not the user's open browser. It
performed one bounded successful end-to-end run and was not repeated for redundant
proof. Screenshots were inspected as a functional fixture, not a visual-design
approval. Mobile, accessibility and other browsers were not tested.

Retained ignored evidence: `artifacts/vite-spike-20261017/browser.json`,
`plain.png`, `tailwind.png`, `backend.log`, and `build-reproducibility.json`.
The repeat-build package-lock SHA-256 is
`79a124daf75a2a18f164d0ce8707e50531205d116decca4d6cda86dcb198bfff`.
Sources and lockfiles, not node_modules, target, dist or screenshots, are the
reviewable experimental deliverables. No server, browser or watcher remains.

## Decision and newly registered work

The experiment supports **asset-producer-neutral boundaries plus optional Vite**,
not mandatory Vite or removal of YT-091's no-Node Tailwind mode. The existing
application-owned approach remains authoritative until YT-098 supplies a
reviewed replacement decision. The following are future work, all Todo:

| Item | Purpose | Required distinction |
|---|---|---|
| YT-098 | Optional producer-neutral asset contract | Validate asset graph/paths/types, settle CSS cascade/order, define snapshot/cache/retention and prove Tailwind standalone plus Vite consumers before promoting a shared API |
| YT-099 | Vite production CLI/profile integration | Optional `--assets vite`; no Tailwind dependency unless selected; use the official Tailwind plugin when selected, not two CSS compilers; explicit lock/build and Linux/Windows/browser qualification |
| YT-100 | Vite development server/HMR integration | Separate dev-only origin/WebSocket/nonce policy, secure environment handling and owned process lifecycle; do not weaken production CSP |
| YT-101 | Configuration and assets in cloud-generated delivery | Explicit runtime asset/config roots, sanitized build contexts, secret exclusion, immutable image assets and limited Docker/Compose/Kubernetes qualification |

Suggested order is YT-098 then YT-099; YT-100 and YT-101 are distinct follow-ups.
No future item is implemented or accepted by this spike. No stable assets crate,
WASM bundling, Windows Vite journey, HMR, production rollout, CI/CD or public
release is claimed. YT-020 remains Blocked.

## Primary references

- [Vite backend integration](https://vite.dev/guide/backend-integration): manifest-driven HTML and static/dynamic dependency records; this is not Vite SSR.
- [Vite build options](https://vite.dev/config/build-options): manifest, CSS splitting and `assetsInlineLimit: 0`.
- [Vite shared options](https://vite.dev/config/shared-options): `envDir: false`, public assets and nonce integration.
- [Vite CSP](https://vite.dev/guide/features#content-security-policy-csp): nonce handling and the data-URI default.
- [Tailwind Vite integration](https://tailwindcss.com/docs/installation/using-vite): official build-time plugin.

## Independent review and final source reconciliation

Separate read-only role-directed Reviewer Job `wc_job_SuuEr3YetGlVbs_8`
completed with exit 0 and JSON `overall_status=pass`, `issues=[]`,
`required_followups=[]`. It inspected all 24 declared changed/new files, adjacent
SSR/CSP APIs and ADRs, the nine Rust tests, the browser-test source and the five
explicitly permitted proof files. All 24 source hashes and both JSON evidence
hashes matched at the beginning and end; Git HEAD/branch and empty staging were
unchanged. No material P0-P2 finding was reported.

The saved Reviewer model was independently observed as `gpt-6-astra` in a
read-only Codex process. Reasoning mode was `inherit`: no effort selector was
passed or claimed as applied. This is an independent ad-hoc role-directed Job,
not native managed-role dispatch or formal assurance. The Reviewer did not rerun
builds/browser journeys, change files or make external calls. Its acceptance
covers the bounded feasibility and actionable Todo plan, not the failed original
executor or any unimplemented future item.

After the review only this factual closeout and the roadmap review-status text
were updated. Rust/TypeScript/CSS, package manifests/lockfiles, Vite configuration,
tests and browser-verified assets stayed unchanged. The current-session emulation
result remains distinct from the independent review result. YT-098 is the next
dependency-ready planned item; YT-098 through YT-101 are all Todo. No further
implementation or commit is implied. YT-020 remains Blocked.
