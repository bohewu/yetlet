# Troubleshooting common problems

Find the symptom below, check the possible cause, then follow the linked guide
for the complete correction. Read guides and API source from the same checkout.
The SDK is an Experimental published alpha; the CLI is a source-installed preview. These are source-backed explanations of
existing behavior, not new execution or platform qualification results.

Start with the **first failing stage**: CLI build → app creation/dependencies →
settings → build/assets → HTTP request. A status alone does not identify its
cause; a 503 from a Session store is different from one returned by a limiter.
Use the selected application's response and safe stage diagnostics. CLI-owned
messages are bounded, but child stdout/stderr may contain sensitive data; do not
share credentials, cookies, CSRF tokens or complete environment/settings dumps.

- [Create and start an application](#create-and-start-an-application)
- [CLI settings and execution](#cli-settings-and-execution)
- [Browser builds and interaction](#browser-builds-and-interaction)
- [Runtime settings and secrets](#runtime-settings-and-secrets)
- [HTTP middleware, authentication and Session](#http-middleware-authentication-and-session)
- [Rate limiting](#rate-limiting)

## Create and start an application

| Symptom | Possible cause | Correction | Corresponding guide |
|---|---|---|---|
| Building the CLI cannot find OpenSSL headers/libraries | The CLI's native signing dependency needs development files; this does not mean every generated app needs OpenSSL | Prepare the documented native toolchain/OpenSSL development files or your selected OpenSSL prefix before building the CLI | [CLI prerequisites](../tools/yetlet-cli/README.md#first-use-prerequisites) |
| `new` refuses an app name or destination | Invalid/reserved name, existing destination, missing parent or unsafe input path | Use an unused valid name and an existing owned, protected physical parent. Do not merge into an existing project or disable filesystem checks | [First-app creation](FIRST-APP.md#2-create-the-plain-native-app) · [CLI creation](../tools/yetlet-cli/README.md#build-locally-and-create-an-application) |
| Cargo cannot resolve the generated Yetlet package versions | Registry/network/cache preparation is missing, or the manifest was changed to an incompatible version | Keep the generated exact published SDK version and selected features. Explicitly generate this app's lock and fetch dependencies before frozen execution; normal build/run needs no local SDK path | [Registry dependency preparation](FIRST-APP.md#3-prepare-the-app-lockcache-then-build-and-run) |
| Frozen build/run refuses a lock or cannot find a cached dependency | This standalone app has no lock, the dependency graph changed, or its locked dependencies were not prepared | After an intentional dependency change, explicitly generate this app's lock and fetch its locked dependencies. Use the same prepared Cargo cache; do not copy another workspace's lock or expect frozen execution to repair it | [App lock and cache](FIRST-APP.md#3-prepare-the-app-lockcache-then-build-and-run) |
| `doctor` or `inspect` succeeds, but build/run still fails | Doctor covers the local Rust/Cargo environment; inspect explains settings-derived plans. Neither establishes app readiness | Follow the actual failing stage's setup. Prepare app dependencies, required secrets and selected assets separately; a skipped stage is not execution evidence | [CLI inspection](../tools/yetlet-cli/README.md#read-only-project-inspection-yt-111) · [Diagnostics](../tools/yetlet-cli/README.md#diagnostics-and-platform-evidence) |
| The plain first app cannot bind port 3000 | Another listener owns the port | Stop the owned conflicting process or choose an unused `YETLET_PORT` as shown in the walkthrough. The plain scaffold remains loopback-only; other samples have their own bind settings | [First-app configuration](FIRST-APP.md#configuration-and-errors) |
| A source edit is absent from an already running plain app | Ordinary native `run` does not watch by default | Stop, build and run again, or deliberately prepare/select the separate Rust watch path. Native frozen `cargo run` may compile changed source once; it is not a live reload promise | [First-app expected result](FIRST-APP.md#expected-result) · [Rust watch](../tools/yetlet-cli/README.md#rust-watch) |

## CLI settings and execution

| Symptom | Possible cause | Correction | Corresponding guide |
|---|---|---|---|
| CLI returns a settings `source`, `profile` or `values` hint | These are coarse hints: unsafe/missing public sources, invalid profile, or invalid public schema/values | Select the intended protected app manifest and public settings root; check required base/profile files, exact `Development`/`Staging`/`Production` spelling, declared keys and types. Do not infer a precise IO/parser cause or print secret values from the hint | [CLI diagnostic hints](../tools/yetlet-cli/README.md#diagnostics-and-platform-evidence) |
| `--release` still uses Production settings, or the wrong public settings directory is read | Cargo build mode and CLI settings profile are independent; changing the public root does not move the app or frontend root | Select `APP_ENVIRONMENT` explicitly when another profile is intended (absent means Production). Use the documented absolute `YETLET_CONFIG_ROOT` only for public TOMLs and select the app with `--manifest-path` | [Settings decision table](../tools/yetlet-cli/README.md#settings-selection-decision-table) |
| Browser build/run/watch exits 0 without starting the expected work | The selected frontend auto flag intentionally skips that stage | Check the merged plan: browser build uses auto_build, run uses auto_start, watch requires both. Native Vite gates differ; select the intended configuration instead of treating exit 0 as a completed build | [Execution decision table](../tools/yetlet-cli/README.md#execution-decision-table) |
| A former `dev`, `--cloud` or `--with-config` command is rejected | Legacy CLI vocabulary is no longer the current interface | Follow current `new`/`build`/`run`/`watch` choices and the migration diagnostic; use current `--service`/`--settings` selections where applicable | [Current CLI inventory](../tools/yetlet-cli/README.md#current-command-and-capability-inventory) |

## Browser builds and interaction

| Symptom | Possible cause | Correction | Corresponding guide |
|---|---|---|---|
| Browser build reports a missing tool or wasm-bindgen identity/version mismatch | The WASM target or matching CLI is missing, or another executable is selected from PATH | Prepare `wasm32-unknown-unknown` and the wasm-bindgen CLI matching the applicable lock (currently 0.2.129), with the documented installation root/PATH. Use the selected app's guide for its Node/npm requirements; native SSR needs neither | [Browser tool setup](../README.md#choose-your-next-step) · [Todo build](../examples/todo-server/README.md#run) |
| Todo `/app` returns 503 with an instruction to build browser assets | The server cannot read `dist/app.html`; starting Cargo does not produce browser assets | From the repository root, follow Todo's explicit browser build before serving `/app`. Keep its intended CSP; this route is distinct from readable SSR pages such as `/search` | [Todo setup](../examples/todo-server/README.md#run) |
| Generated browser watch returns 503 before its first successful generation, or a Rust edit resets tab state | No good WASM generation exists yet, or the selected loop performs a full reload | Prepare the frozen build inputs and fix compilation errors. The last good JS/WASM pair survives later compile failures; successful Rust builds reload the page and do not restore Rust/tab state. CSS HMR remains Vite-owned | [Browser development loop](../tools/yetlet-cli/README.md#browser-rust-development-loop--yt-112) |
| Native DOM validation succeeds but an island does not mount, or a navigation/control View is rejected | Static validation does not execute browser APIs; mount has additional document-base/navigation rules and a finite supported vocabulary | Use the documented WASM caller/owned host and supported tags/attributes. Follow mount's actual browser error and keep URL/CSP restrictions; do not bypass them with arbitrary HTML or event handlers | [DOM integration](../crates/yetlet-browser/guides/dom.md#minimal-integration) · [DOM limits](../crates/yetlet-browser/guides/dom.md#supported-subset-and-lifetime) |

## Runtime settings and secrets

| Symptom | Possible cause | Correction | Corresponding guide |
|---|---|---|---|
| The runtime Loader rejects a key/type, or an environment override is ignored | Wrong declared schema/prefix/nesting, wrong scalar kind, or an unrelated OS variable. This is separate from CLI tool settings | Check base-declared keys and types and canonical prefixed names; double underscores express nesting. Select the exact profile and all required sources. Boolean overrides accept `true`/`false`; unrelated variables do not create configuration keys | [Source order and schema](CONFIGURATION.md#sources-and-deterministic-precedence) |
| A development secret file is not read, or a copied secret stays old after refresh | Secret files require explicit Development selection/path. A provider lookup and a client's previously copied value have different lifecycles | Use the application's explicit Development-secret setup; Staging/Production do not read that file. On rotation, follow the consumer's documented reload/reconfiguration boundary; do not place backend secrets in frontend dotenv or hydration state | [Development secrets](CONFIGURATION.md#development-secrets-and-ownership) · [Rotation](SECRET-PROVIDERS.md#refresh-and-rotation) |
| Secret lookup/injection fails, or an empty secret passes lookup but startup rejects it | Missing (`None`), Unavailable and AccessDenied are different outcomes; an empty `Some` value still requires consumer validation | Repair the selected provider/input according to its actual category. Bound settings fields have no literal/environment fallback on lookup failure; validate required values before constructing clients/listeners. Do not log plaintext or silently select a weaker source | [Secret read contract](SECRET-PROVIDERS.md#read-contract) · [Typed injection](SETTINGS-SECRET-PROVIDER.md#contract) |

## HTTP middleware, authentication and Session

| Symptom | Possible cause | Correction | Corresponding guide |
|---|---|---|---|
| Middleware seems absent on a sibling route/404, or after-processing runs before a stream completes | Scope, registration order and response-return timing differ from handler/body completion | Check the documented matched-route/fallback rules, ServiceBuilder versus repeated Router layers, and route registration before wrapping. Response extensions do not automatically inherit request extensions | [HTTP execution order](HTTP-PIPELINE.md#scope-and-execution-order) |
| Auth returns generic 500, including on an optional route | Possible missing/duplicate Auth layer, authentication-name mismatch, missing Session bridge layer, or an invalid 401 response without a nonempty challenge | Check the selected route's wiring and exact authentication name; install Session outside Auth for the Memory bridge. Supply an applicable nonempty `WWW-Authenticate` for 401. Do not turn setup/provider errors into anonymous success | [Resolver/failure mapping](../crates/yetlet/guides/auth.md#principal-and-resolver) · [Memory bridge](../crates/yetlet/guides/auth.md#optional-memory-session-bridge) |
| A protected request is anonymous, returns 401/local login redirect, or returns 403 after identity resolution | Anonymous and invalid credentials differ from application policy denial; verified identity alone grants no permission | Follow the resolver's actual result and the app's explicit anonymous response. Validate credentials/current account mapping and resource authorization independently. Never redirect a provider outage or policy denial, and do not trust arbitrary identity headers | [Auth result semantics](../crates/yetlet/guides/auth.md#principal-and-resolver) |
| Auth/Session returns 503 | The selected provider may be unavailable; Memory admission/entropy/store failures also reject. A status alone does not determine which | Inspect the selected provider's safe failure category and its documented recovery behavior. Keep failures closed, preserve uncertain mutation outcomes and do not automatically repeat a write or downgrade to anonymous | [Auth bridge failures](../crates/yetlet/guides/auth.md#optional-memory-session-bridge) · [Session failures](../crates/yetlet/guides/session.md#cookies-mutations-and-failures) |
| Memory Session disappears after restart, differs across processes, or is lost on local HTTP | Memory state is process-local; cookies default to Secure. Expiry, destruction and browser cookie handling can also end a Session | Use the documented local-HTTP override only for the loopback demo. Retain Secure for deployment; use an explicitly selected shared provider for cross-instance state rather than treating independent Memory managers as shared storage | [Session cookie/storage boundaries](../crates/yetlet/guides/session.md#cookies-mutations-and-failures) · [Deployment boundary](../crates/yetlet/guides/session.md#deployment-boundary) |
| A previously open form fails CSRF validation after login/rotation/logout | Tokens are Session-bound and rotation retires the old identity/token | Reload a current protected form under the current Session, then perform the intended action with explicit CSRF validation. Do not reuse a stale token, disable CSRF or automatically replay a rejected write | [Session mutations](../crates/yetlet/guides/session.md#cookies-mutations-and-failures) · [Auth login boundary](../crates/yetlet/guides/auth.md#optional-memory-session-bridge) |

## Rate limiting

| Symptom | Possible cause | Correction | Corresponding guide |
|---|---|---|---|
| Rate limits do nothing, or startup rejects configuration even while disabled | The native example defaults off; files/schema/types are still validated. This switch belongs to that app's optional settings integration | Load the example's declared settings once at startup and deliberately select `RATE_LIMIT_ENABLED` when needed. Use `RATE_LIMIT_POLICY__INTERVAL_MS`, `RATE_LIMIT_POLICY__BURST` and `RATE_LIMIT_POLICY__MAX_KEYS`; the interval refills one cell. It is not a global library environment switch or live reload | [Rate-limit settings example](../crates/yetlet/guides/rate-limit.md#opt-in-settings-loader-in-the-native-example) |
| Reusing one Policy does not produce a group total, or different instances allow independent traffic | Policy reuses configuration; separately constructed Budgets and separate processes have independent accounting | Clone/share the same Budget for the same key to share a total within a process. Include route/method only when intentional partitioning is wanted. This implementation provides no distributed quota | [Budget wiring](../crates/yetlet/guides/rate-limit.md#shared-budget-wiring) · [Accounting ownership](../crates/yetlet/guides/rate-limit.md#admission-ownership-and-errors) |
| Requests receive 429 or a limiter-generated 503 | 429 means a deficit; 503 means retained-key capacity/internal failure, not a quota wait | A 429 has a finite `Retry-After` opportunity, not reserved credit. Diagnose 503 separately (no limiter Retry-After); retain application retry/idempotency policy. Do not infer an extra queue, automatic retry, Session failure or authentication denial | [Limiter failure semantics](../crates/yetlet/guides/rate-limit.md#admission-ownership-and-errors) |

For integration choices beyond a specific failure, use the
[current API reference](API-REFERENCE.md), [first-app walkthrough](FIRST-APP.md)
and [sample catalog](SAMPLES.md). Application-specific diagnostics remain in the
selected sample guide; this catalog adds no runtime diagnostics, generic default
provider, deployment policy or new validation claim.
