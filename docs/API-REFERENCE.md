# Current API reference

Date: 2026-10-27. Scope: the current checkout, including the existing opt-in
application options. All packages remain unpublished and experimental. This
entry map does not promise SemVer stability or requalify the current source for
every platform. All sixteen implemented libraries are included in the selected
experimental 0.1.0 candidate; package selection does not install a capability.

Use repository guides for behavior and integration, and source/rustdoc from the
same revision for signatures and feature/target availability. The
[documentation scope](DOCUMENTATION-SCOPE.md) defines publication inputs.
[Compatibility](COMPATIBILITY.md) and [naming](NAMING.md) govern change discipline;
the [historical M9 classification](API-CONTRACTS.md) retains its original scope.
The [capability map](CAPABILITIES.md) distinguishes framework packages,
application integrations and research decisions.

The packages have compatible 0.1.0 metadata, licenses and package guides. They
remain unpublished; the selected candidate is undergoing technical evidence
reconciliation. Support remains specific to each package, target and feature.
Historical nine-package evidence keeps its original source and support limits.

For first authoring, follow [Views/components](VIEW-COMPONENTS.md). For package
features, startup wiring and consumer settings/defaults in one place, use the
[opt-in quick reference](OPT-IN-QUICK-REFERENCE.md); the tables below remain the
API-carrier map, with full contracts in their linked guides.

## HTML, HTTP and browser packages

These packages provide the main HTML, HTTP and browser building blocks. Their
presence here is not registry availability; start from the existing checkout. Renderer validation
and native Axum/Tower semantics remain separate responsibilities.

| Package and current guide | Main entry points | Selection, ownership and failure boundary |
|---|---|---|
| [yetlet-core](../crates/yetlet-core/README.md) | `View`, `Element`, `binding` validated vocabulary | Owned renderer-neutral values; construction does not sanitize HTML. Binding validation can return `BindingContractError`; the renderer owns output policy |
| [yetlet-html](../crates/yetlet-html/README.md) | `render_to_string`, `escape_text`, `RenderError` | Borrow a View, return an owned escaped fragment or policy error. Complete buffering; no raw HTML escape hatch or HTTP-header ownership |
| [yetlet-server-axum](../crates/yetlet-server-axum/README.md) | `App`, `Page`, `RequestContext`, `DocumentMetadata`, `LanguageTag`, `middleware` | Native routing/extractors/State/Extensions; fallible rendering maps to generic HTTP 500. Metadata and strict CSP stay bounded; no mandatory Session, configuration, MVC or DI |
| [yetlet-dom](../crates/yetlet-dom/README.md) | `validate`; WASM `mount` | Optional browser subtree replacement, finite DOM policy. Static validation returns `DomPolicyError`; browser failures use `JsValue`. No general identity preservation; mount can reject navigation that static validation accepted |
| [yetlet-dom-keyed](../crates/yetlet-dom-keyed/README.md) | `validate_keyed_order`; WASM `KeyedChildren`, `KeyedEntry`, `DomEventListener` | Explicit keyed/listener lifetime; duplicate-key preflight and browser failures. Callbacks and DOM failures do not provide transactional rollback or a generic sanitizer |
| [yetlet-reactive](../crates/yetlet-reactive/README.md) | `LocalGraph`, `GraphHandle`, `Subscription`, `ReactiveError` | Synchronous thread-local reducer/commit ownership; weak handles and explicit disposal. Snapshot clones published state; faults and queued re-entrant work retain their documented semantics |
| [yetlet-actions](../crates/yetlet-actions/README.md) | `Command`, `ActionExecutor`, `LocalExecutor`, `HttpExecutor`, `ActionError` | Explicit local versus typed HTTP actions; native HTTP-executor construction is not native fetch execution. WASM executes HTTP; no implicit Auth/CSRF, retries or remote DOM-event circuit |
| [yetlet-request-id](../crates/yetlet-request-id/README.md) | `RequestIds`, `RequestId`, `HEADER`, `InvalidStart` | Explicit scoped/global installation. Clones share a bounded numeric counter; independent providers/restarts do not. Generated IDs replace untrusted inbound values; overflow refuses with 503, not distributed trace identity |
| [yetlet-session](../crates/yetlet-session/README.md) | `SessionConfig`, `SessionManager`, `Session`, typed Session errors | Explicit bounded Memory store, synchronous operations, cookie/CSRF and rotation/revocation. Process-local state disappears on restart; anonymous Session is not authentication or durable business storage |

## Additional opt-in packages

These seven packages are also included in the selected sixteen-package
candidate and are available for explicit source-checkout use. A separate
workspace/lock is a build boundary, not an interchangeable provider or
platform-support guarantee. Their default installation and support do not
follow from coordinated versioning.

| Package and current guide/source | Main entry points | Selection, ownership and failure boundary |
|---|---|---|
| [yetlet-auth](../crates/yetlet-auth/README.md) | `Principal`, `Authentication`, `authenticate`, `Authenticated`, resolver types | Native route-scoped identity resolution; required versus optional extraction, missing-layer refusal and responses follow the guide. Verified identity does not grant business authorization. Optional `session` explicitly bridges Memory Session only |
| [yetlet-rate-limit](../crates/yetlet-rate-limit/README.md) | `Policy`, `Budget`, `Key`, `RateLimitLayer`, `InvalidRateLimitInput` | Native per-process memory admission. Policy reuse shares configuration; Budget/layer clones share accounting. Trusted keys, bounded capacity and rejection semantics are explicit; no distributed quota |
| [yetlet-config](../crates/yetlet-config/README.md) / [full guide](CONFIGURATION.md) | `Loader`, `Environment`, `Error::Key`; optional `SecretBinding` | Opt-in startup loading with finite precedence. `load_with_env` applies a supplied snapshot after ordinary sources; optional `secret-provider` resolves selected protected fields. No ambient Settings service, live reload or browser secrets |
| [yetlet-secrets](../crates/yetlet-secrets/README.md) | `SecretProvider`, `SecretFuture`, `SecretError`, `SettingsSecrets` | Injected native async lookup: missing `None` differs from access/unavailable errors. Settings adapter owns an explicitly replaceable snapshot; no automatic rotation, registry or browser dependency |
| [yetlet-signing](../crates/yetlet-signing/README.md) / [source](../crates/yetlet-signing/src/lib.rs) | `SigningBackend`, `PublicKeySource`, `PublicKeySet`, `Policy`, `Verifier`, `SigningService` | Explicit exact-byte operations, versioned keys and caller-owned trust. Public material membership does not establish trust; local OpenSSL support does not activate release signing or imply general PKI validation |
| [yetlet-session-redis](../crates/yetlet-session-redis/README.md) | `RedisSessionConfig`, `RedisSessionManager`, `RedisSession`, typed operation/errors | Separately selected async provider; configured namespace, limits, lifecycle and uncertain-outcome handling remain explicit. Optional native `tls` selects checked `rediss://`; Redis names the protocol API and Valkey the selected reference server. No generic HA/failover/restore promise |
| [yetlet-wgpu](../crates/yetlet-wgpu/README.md) / [support](../crates/yetlet-wgpu/SUPPORT.md) | Bounded scene/batch/lifetime and picking APIs | Separate optional flat-scene renderer. CPU-only defaults; explicit `browser` plus WASM target selects WebGPU. Scene/device/lifecycle errors remain explicit; no DOM acceleration, stable API, 3D or performance guarantee |

Native Auth, configuration, secrets, signing and remote Session capabilities
must not enter browser builds or serialized hydration state. Package selection
and Cargo features determine compiled availability; application startup decides
whether an available capability is installed for an instance. Workspace-wide
builds are not the default dependency graph of a small SSR application.

## Application options and tools

| Existing consumer | Current integration API | Contract boundary |
|---|---|---|
| [Board](../examples/lab-operations-board/README.md) / [Admin](../examples/admin-dashboard/README.md) | `BoardOptions`; `Board::open_with_options` and `open_with_oidc_and_options` | Independent default-false `write_authorization` / `request_tracing`, plus frontend/Session inputs. Uncompiled selections fail before storage; existing simpler constructors stay off. Admin maps its existing typed Settings; Board loads no ambient settings or global subscriber |
| [Shop](../examples/modular-shop/README.md) | `DatabaseOptions`; `Database::open_with_options` | Default-false `catalog_cache`; the binary maps its existing environment key. Display freshness is bounded; Session, authorization and checkout SQL stay authoritative. No mandatory Settings loader or framework cache crate |
| [CLI](../tools/yetlet-cli/README.md#current-command-and-capability-inventory) | Existing parser, commands and configured generators | Separate unpublished tool; settings, supported source layouts and platform limits belong to its current guide. Generators do not silently install or deploy services |

These options are public Rust interfaces of example applications, not methods
on a framework-wide capability builder. Their public fields do not grant trust
or remove the existing fresh-v2 write-authority/reopen restrictions. Feature
names and external settings keys remain as documented in each current guide.
The [naming guide](NAMING.md#application-opt-in-options--yt-185) records migration
from the former capability-specific constructors without aliases.

OIDC, opaque Bearer checks, native tracing and catalog caching have existing
application-owned integrations. Audit history and output-cache studies do not
imply `yetlet-audit`, `yetlet-output-cache` or other extracted packages. Do not
infer an implemented API from a research item marked Done.

## Local rustdoc

Generate the reference for the package/configuration being used. For example,
from the repository root with the pinned toolchain and required dependencies:

```sh
cargo doc --locked --no-deps -p yetlet-core -p yetlet-html -p yetlet-server-axum
cargo doc --locked --no-deps -p yetlet-auth --features session
cargo doc --locked --no-deps -p yetlet-rate-limit
```

The first command writes the server entry page under
`target/doc/yetlet_server_axum/index.html`. Features remain explicit; `--no-deps`
selects documented packages, not a promise that dependencies need no compilation.
For an isolated workspace, use its manifest and its own lock/target directory:

```sh
cargo doc --locked --no-deps --manifest-path crates/yetlet-config/Cargo.toml
cargo doc --locked --no-deps --manifest-path crates/yetlet-secrets/Cargo.toml
cargo doc --locked --no-deps --manifest-path crates/yetlet-signing/Cargo.toml
cargo doc --locked --no-deps --manifest-path crates/yetlet-session-redis/Cargo.toml --features tls
```

A native default reference omits cfg-gated browser items. With the documented
WASM prerequisites, select that target (and the GPU feature only if desired):

```sh
cargo doc --locked --no-deps --target wasm32-unknown-unknown -p yetlet-dom -p yetlet-dom-keyed -p yetlet-actions
cargo doc --locked --no-deps --manifest-path crates/yetlet-wgpu/Cargo.toml --target wasm32-unknown-unknown --features browser
```

These are local documentation-generation instructions, not qualification commands
or a claim that every combination has been executed. No hosted rustdoc, docs.rs
package, separate Wiki or documentation website is selected by this reference.
