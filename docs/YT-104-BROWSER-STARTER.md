# YT-104 — browser-only task starter

Date: 2026-10-19. Base checkout `82174f94d4e10b6c684ae28aa1631cb67662114c`
plus the bounded source changes described here. This is an opt-in static
reference application, not a new default renderer, general SPA framework or
production hosting service. **Done at this selected local scope.** Independent
Reviewer `/root/yt104_browser_review_oct19` PASS has zero issues or required
followups.

## Selected contract

The existing `new`, `build` and `run` commands are reused. Creation selects a
finite built-in starter through `--template native|browser` (default `native`).
Execution continues to use settings.

```sh
yetlet new tasks --template browser --tailwind
```

Template selection belongs to creation arguments: `APP_CLI__TARGET` does not
select a template or affect `new`. No parent configuration or framework checkout
is inferred. Unknown, missing or duplicate template values and browser with
`--service` fail as usage errors before publishing a project. The resulting
`base.toml` persists the execution target:

```toml
[cli]
target = "browser"

[cli.rust]
watch = false

[cli.vite]
enabled = true
root = "frontend/"
auto_start = true
auto_build = true

[cli.generate.k8s]
replicas = 1
```

Execution uses the existing shared Loader, base/profile precedence, declared
APP overrides (including `APP_CLI__TARGET`) and protected standalone manifest/lock
checks. Missing target
defaults to `native` for existing applications. Browser mode requires Vite
enabled and native watch/TLS disabled. The public generator namespace above
retains the existing projection shape; native delivery generators are not a
browser deployment workflow.

| Operation | Browser behavior |
| --- | --- |
| `new` | Publish Rust library sources, static HTML, public settings and frontend sources/lock; no install, lock generation, compile or listener |
| `build` | With `auto_build=true`, supervise only the existing npm build stage: frozen optimized Cargo WASM, matching wasm-bindgen web bindings and Vite static output; no native Rust build |
| `run` | With `auto_start=true`, require prepared protected HTML/manifest and supervise only Vite preview on loopback 5173 with strict-port behavior; no build, native server or dev bridge |
| Ctrl-C | Reuse the existing Linux foreground group / Windows Job and console cleanup; preserve the documented native cancellation result |
| Disabled automatic stage | `auto_build=false` or `auto_start=false` returns a visible skip and exit 0 without starting children |
| Rust watch / `watch` | Refused for this browser contract; rebuild explicitly after source changes |

`--release` retains its native meaning; browser WASM builds are optimized with
or without it. Root renaming uses `cli.vite.root`; keep CSS relative source
paths aligned if changing the frontend's depth. Package scripts remain
trusted application code, not a sandbox. Rust target, matching wasm-bindgen CLI,
Node/npm, dependencies and application Cargo.lock are prepared separately.
Generated crate versions remain unpublished; local evaluation explicitly uses
trusted Cargo path dependencies rather than a hidden framework source selector.

## Application and browser boundary

The CLI-created [browser task reference](../examples/browser-tasks/README.md)
adds, edits, completes and filters at most 200 tab-local tasks. A page reload
restores the three sample items. There is no account, persistence, API request,
hydration state or backend-generated page shell. The TS entry only initializes
WASM, imports CSS and selects a public workspace label; Rust owns task behavior.

The existing DOM subset is sufficient: `li`, `label`, `input`, `span`, `button`
and inert class/type/id/data attributes render each row through `yetlet-dom`.
`KeyedChildren` retains row/control identity; application code explicitly writes
checkbox properties, accessible edit names and native form bindings with
`web-sys`. Static semantic HTML supplies forms, navigation-to-content, loading
and no-JS explanations. A local `LocalGraph` reduces typed commands; no transport
or changes to Core/DOM/reactive APIs are needed. The edit form remains native
and retains an unfinished draft while filtering. Existing YT-105 covers general
authoring ergonomics; no new runtime abstraction is inferred from this sample.

Only public frontend `VITE_*` values belong in frontend dotenv. Backend APP
settings and the existing config/Development-secret selectors are removed from
the frontend process environment. Settings/SecretProvider, sessions, signing,
SQLite and server crates are absent from the browser application's dependency
graph and output. This does not make arbitrary application build scripts safe
to run with unrelated credentials in their environment.

The ship directory is **only `frontend/dist/`**: static HTML plus versioned
JS/CSS/WASM/SVG and the build manifest. Relative asset URLs support root hosting
and a directory prefix ending in `/`. JavaScript/WASM and an HTTP(S) host with
correct MIME types are required; `file://`, history routes and automatic SPA
fallback are not selected. HTML CSP permits same-origin assets/fetches and
`wasm-unsafe-eval`, without inline script or general JS eval. A deployment host
owns additional response headers and TLS. Vite preview is a local verification
tool, not a production server.

This follows the maintained [Vite static deployment guidance](https://vite.dev/guide/static-deploy.html)
and [wasm-bindgen web deployment contract](https://wasm-bindgen.github.io/wasm-bindgen/reference/deployment.html).

## Verification and preserved failures

Linux implementation checks:

```sh
cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --lib --test browser --test commands
cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --test frontend_process --test selected_scaffold
cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets -- -D warnings
yetlet build --manifest-path examples/browser-tasks/Cargo.toml
npm run typecheck --prefix examples/browser-tasks/frontend
cargo test --offline --locked --manifest-path examples/browser-tasks/Cargo.toml
cargo clippy --offline --locked --manifest-path examples/browser-tasks/Cargo.toml --target wasm32-unknown-unknown -- -D warnings
cargo xtask check docs
```

The CLI's pre-existing signing dependency needs a separately prepared OpenSSL
prefix; the local check reused that existing static prerequisite. The browser
app does not require OpenSSL. `wasm-bindgen` 0.2.129 was selected explicitly on
PATH. The reference was created through `new` in a protected temporary parent,
moved into `examples/`, then given explicit relative Cargo path dependencies
and a separately prepared offline lock/npm install.

| Check | Result and boundary |
| --- | --- |
| CLI native focused suites | Library 82/82, browser creation 1/1, commands 3/3; frontend/process 15/15; selected scaffold 1 passed, 2 existing explicitly opt-in cases ignored |
| App model | 2/2; invalid/missing/oversized titles and the 200-item bound preserve existing state; add/edit/complete/filter retain identity |
| Build/static checks | CLI browser build exit 0, optimized WASM and versioned static files; TypeScript, strict native CLI/WASM Clippy, format, docs and whitespace pass |
| Actual Chrome | System Chrome on Linux; desktop 1440×1000 and mobile 390×844; title, mount, add/edit/complete/filter/empty, preserved edit draft, keyed checkbox identity, validation, escaped markup, CJK text, reload reset and no mobile overflow pass |
| Requests/CSP/MIME | Five initial same-origin HTML/CSS/JS/WASM/SVG requests; zero requests during task operations; WASM `application/wasm`; zero relevant console, page or CSP errors |
| No JavaScript | Explicit requirement explanation visible and task controls remain disabled; no meaningful no-JS task execution is claimed |
| Independent static host | Only dist copied outside the app tree and served by Python's existing HTTP server at `/demo/`; the same browser interaction and MIME/CSP checks pass, with no Yetlet native backend, Node runtime, settings or source files in the served tree |
| Linux stop/cleanup | Actual CLI PTY Ctrl-C prints `CLI_CANCELLED ... owned children stopped`, tool-observed exit 1; the CLI/notifier are absent, no process retains app/frontend cwd and ports 5173/4187 close. The independent static host was also stopped |
| Windows GNU | Current CLI compiled and executed natively with Rust 1.99; 1/1 browser-creation test passes. Actual CLI new/build exit 0, TypeScript exit 0, static HTML/JS/CSS/WASM/SVG generated without a native application entry point |
| Windows run/stop | New owned hidden console: actual `run` serves HTML 200 and WASM `application/wasm`; native `CTRL_C_EVENT` gives CLI status `0xc000013a` (3221225786), all five observed owned processes stop and port 5173 closes |
| Windows source/cleanup | All 879 exported input hashes rechecked; seven generated runtime files match the exported reference byte-for-byte. Zero remaining task processes/listeners; older host checkout remains at its original branch/HEAD. This is GNU native/HTTP evidence, not a separate Windows browser, MSVC or physical-keyboard claim |

The Browser plugin was not listed and the available computer-use inventory had
no browser surfaces, so browser qualification used existing Playwright with
system Chrome. Screenshots and receipts stay outside Git. The full-screen
design concept and rendered desktop/mobile views were visually inspected.

Initial failures are retained separately: missing default Linux OpenSSL headers
was a prerequisite issue; the first new static-entry read rejected ordinary
group-writable build directories even inside a private frontend, corrected by
reusing the protected output-read policy. The first browser mount failed because
the sample returned a row still attached to its temporary DOM mount; it now
detaches that row before transferring it to the existing keyed owner. The
passing browser results above follow that application fix.

Windows used a private source fixture, preserving the older host checkout. The
CLI binary SHA-256 was
`334c42257113815f02c43014c3c4b386fdd3aa3b03dafa356c1a4cd86816c153`.
Owned receipt archive SHA-256 was
`70b81f9bfdfb50a2b6631f70efaeac2e56bffcfb98f16c8fd561e17a656f5acc`.
The source fixture is a working-tree export, not a claim that the old Windows
checkout was updated or that output is bit-reproducible across hosts.

Windows prerequisites were prepared separately: the official matching
wasm-bindgen release, Rust WASM target and task-local
[MSYS2 OpenSSL SDK](https://packages.msys2.org/packages/mingw-w64-x86_64-openssl).
The SDK archive was checked against its published SHA-256. The default Node
selection traversed an fnm junction and was refused by existing CLI policy;
a byte-identical protected task-local Node/npm copy passed without changing
global PATH, ACLs or toolchain defaults. The initial artifact-parent creation
refusal is retained; creation in the normal temporary parent passed and the
owned result was moved to the private fixture. No filesystem rule was weakened.
Native console collection first needed unsigned Win32 argument typing and
detaching the collector's existing console before attaching the owned one;
these were harness corrections, not application shutdown fixes. Returned build,
source, console-stop and cleanup receipts stay outside Git.

## Creation-template amendment — 2026-10-19

The maintainer selected `yetlet new tasks --template browser --tailwind` in place
of the original environment-driven creation selector. `--template=native` keeps
the existing default. The parser accepts both separate and equals forms; rejects
unknown, missing and duplicate values, misplaced options and browser/service
combinations; and passes a typed template directly to publication. Runtime
`cli.target` and its declared APP override retain their settings behavior.

Focused Linux verification passes: template parser 1/1, actual browser creation
1/1, command integration 3/3, key integration 9/9, selected scaffold 1 passed
with two existing opt-in cases ignored, strict all-target CLI Clippy, format,
docs and whitespace. Creation tests prove that runtime APP overrides cannot
select the template, including native creation with a browser override and
browser creation with a native override. A real CLI creation through the new
flag reproduces all seven reviewed reference runtime inputs byte-for-byte.

Source-matched Windows GNU verification through WebCodex also passes: template
parser 1/1, native browser creation 1/1 and CLI compilation exit 0. The eight
updated source/template/test hashes match this checkout; CLI SHA-256 is
`c9635dc50ad2816d97d637684dbd746a59cbebe356a7967125b14319350f142e`.
The older Windows checkout remains unchanged. Existing Rust 1.99, GNU binutils,
C compiler and OpenSSL SDK were selected only for the task process; no global
PATH, toolchain default or product code was changed for prerequisite recovery.
The initial missing-toolchain/binutils/compiler and linker-path failures remain
operational results rather than product failures. The unchanged runtime journeys
were not repeated.

A broader local replay recorded 82/83 library tests and 11/15 frontend/process
tests passing. The watch fixture cannot bind port 5173 while the separate
maintainer-requested preview is running; the frontend cancellation fixture also
refuses the occupied port and poisons three subsequent shared-lock tests. A
single isolated watch replay confirmed the same readiness failure, so further
replays stopped. These are occupied-port test-environment results, not passing
current-suite claims. The earlier complete library/process PASS and actual
Linux/Windows run/stop/browser/static-host receipts remain qualified to their
recorded source. Execution, process adapters, browser runtime and packaging are
unchanged by this amendment; the requested preview remains running.

Final amendment Reviewer `/root/yt104_template_closeout_review_oct19` PASS has
zero issues or required followups. It verified all 47 final review-target hashes,
inspected the creation/settings/publication paths and accepted the focused
Linux/Windows evidence, seven identical runtime inputs and explicit occupied-port
replay boundaries. Review was read-only with no repeated tests or services.

## Closeout boundary

Independent Reviewer `/root/yt104_browser_review_oct19` inspected the implementation,
templates, standalone example, tests and adjacent contracts, verified all 44
review-target hashes and 137 compiled-input hashes, and reviewed Linux browser/
static-host and Windows build/run/stop/cleanup receipts. Review was read-only;
no passing test or service journey was repeated. Its PASS supports the selected
bounded Done status.

Implementation files are the CLI target/settings/execution/scaffold modules,
the protected static-output read seam, browser templates and creation/process
tests under `tools/yetlet-cli/`, plus the standalone `examples/browser-tasks/`
sources/settings/locks. Backlog, Roadmap, CLI contract and guide/index documents
record the same scope. No framework crate or workspace-default dependency changed.

Cloud deployment, publication, framework default changes, HMR, persistent task
storage and production support remain outside this qualification. YT-105 and YT-106 remain separate research
items; the completed YT-094 evidence is not reopened.
