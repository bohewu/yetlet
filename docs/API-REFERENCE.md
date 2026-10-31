# Current API reference

Date: 2026-10-31. This map describes the published Experimental two-package SDK
(`0.9.0-alpha.5`) in this checkout. `yetlet` contains the shared View and opt-in
native capabilities; `yetlet-browser` contains browser integrations and depends
one-way on the same View. Both packages are available from crates.io at this version.
Earlier release tags retain their original layouts and evidence.

Use guides for behavior and rustdoc from the same revision for signatures.
[The migration contract](YT-211-TWO-PACKAGE-MIGRATION.md) records the complete
source break, feature names and preserved boundaries.

For first authoring, follow [Views/components](VIEW-COMPONENTS.md). For package
features, startup wiring and consumer settings/defaults in one place, use the
[opt-in quick reference](OPT-IN-QUICK-REFERENCE.md); the tables below remain the
API-carrier map, with full contracts in their linked guides.

For source loading, startup validation, typed options and application-owned
diagnostics together, follow [Settings, options and logging](SETTINGS-OPTIONS-LOGGING.md).

## HTML, HTTP and browser modules

These modules provide the main HTML, HTTP and browser building blocks. Their
feature selection remains explicit; see the [first-app guide](FIRST-APP.md). Renderer validation
and native Axum/Tower semantics remain separate responsibilities.

| Package and current guide | Main entry points | Selection, ownership and failure boundary |
|---|---|---|
| [`yetlet::view`](../crates/yetlet/guides/core.md) | `View`, `Element`, `binding` validated vocabulary | Owned renderer-neutral values; construction does not sanitize HTML. Binding validation can return `BindingContractError`; the renderer owns output policy |
| [`yetlet::html`](../crates/yetlet/guides/html.md) | `render_to_string`, `escape_text`, `RenderError` | Borrow a View, return an owned escaped fragment or policy error. Complete buffering; no raw HTML escape hatch or HTTP-header ownership |
| [`yetlet::server_axum`](../crates/yetlet/guides/server-axum.md) | `App`, `Page`, `RequestContext`, `DocumentMetadata`, `LanguageTag`, `middleware` | Native routing/extractors/State/Extensions; fallible rendering maps to generic HTTP 500. Metadata and strict CSP stay bounded; no mandatory Session, configuration, MVC or DI |
| [`yetlet_browser::dom`](../crates/yetlet-browser/guides/dom.md) | `validate`; WASM `mount` | Optional browser subtree replacement, finite DOM policy. Static validation returns `DomPolicyError`; browser failures use `JsValue`. No general identity preservation; mount can reject navigation that static validation accepted |
| [`yetlet_browser::dom_keyed`](../crates/yetlet-browser/guides/dom-keyed.md) | `validate_keyed_order`; WASM `KeyedChildren`, `KeyedEntry`, `DomEventListener` | Explicit keyed/listener lifetime; duplicate-key preflight and browser failures. Callbacks and DOM failures do not provide transactional rollback or a generic sanitizer |
| [`yetlet::reactive`](../crates/yetlet/guides/reactive.md) | `LocalGraph`, `GraphHandle`, `Subscription`, `ReactiveError` | Synchronous thread-local reducer/commit ownership; weak handles and explicit disposal. Snapshot clones published state; faults and queued re-entrant work retain their documented semantics |
| [`yetlet::actions`](../crates/yetlet/guides/actions.md) | `Command`, `ActionExecutor`, `LocalExecutor`, `ActionError`; browser `yetlet_browser::actions::HttpExecutor` | Explicit local versus typed HTTP actions; native HTTP-executor construction is not native fetch execution. WASM executes HTTP; no implicit Auth/CSRF, retries or remote DOM-event circuit |
| [`yetlet::request_id`](../crates/yetlet/guides/request-id.md) | `RequestIds`, `RequestId`, `HEADER`, `InvalidStart` | Explicit scoped/global installation. Clones share a bounded numeric counter; independent providers/restarts do not. Generated IDs replace untrusted inbound values; overflow refuses with 503, not distributed trace identity |
| [`yetlet::session`](../crates/yetlet/guides/session.md) | `SessionConfig`, `SessionManager`, `Session`, typed Session errors | Explicit bounded Memory store, synchronous operations, cookie/CSRF and rotation/revocation. Process-local state disappears on restart; anonymous Session is not authentication or durable business storage |

## Additional opt-in modules

Each native capability below is a feature of `yetlet`; WebGPU belongs to
`yetlet-browser`. Selecting a feature compiles an API, while startup installation
and application-owned configuration remain explicit. Native modules are absent
on wasm32 even in a broad feature union.

| Package and current guide/source | Main entry points | Selection, ownership and failure boundary |
|---|---|---|
| [`yetlet::auth`](../crates/yetlet/guides/auth.md) | `Principal`, `Authentication`, `authenticate`, `Authenticated`, resolver types | Native route-scoped identity resolution; required versus optional extraction, missing-layer refusal and responses follow the guide. Verified identity does not grant business authorization. Optional `auth-session` explicitly bridges Memory Session only |
| [`yetlet::rate_limit`](../crates/yetlet/guides/rate-limit.md) | `Policy`, `Budget`, `Key`, `RateLimitLayer`, `InvalidRateLimitInput` | Native per-process memory admission. Policy reuse shares configuration; Budget/layer clones share accounting. Trusted keys, bounded capacity and rejection semantics are explicit; no distributed quota |
| [`yetlet::config`](../crates/yetlet/guides/config.md) / [full guide](CONFIGURATION.md) | `Loader`, `Environment`, `Error::Key`; optional `SecretBinding` | Opt-in startup loading with finite precedence. `load_with_env` applies a supplied snapshot after ordinary sources; optional `secret-provider` resolves selected protected fields. No ambient Settings service, live reload or browser secrets |
| [`yetlet::secrets`](../crates/yetlet/guides/secrets.md) | `SecretProvider`, `SecretFuture`, `SecretError`, `SettingsSecrets` | Injected native async lookup: missing `None` differs from access/unavailable errors. Settings adapter owns an explicitly replaceable snapshot; no automatic rotation, registry or browser dependency |
| [`yetlet::signing`](../crates/yetlet/guides/signing.md) / [source](../crates/yetlet/src/signing/mod.rs) | `SigningBackend`, `PublicKeySource`, `PublicKeySet`, `Policy`, `Verifier`, `SigningService` | Explicit exact-byte operations, versioned keys and caller-owned trust. Public material membership does not establish trust; local OpenSSL support does not activate release signing or imply general PKI validation |
| [`yetlet::session_redis`](../crates/yetlet/guides/session-redis.md) | `RedisSessionConfig`, `RedisSessionManager`, `RedisSession`, typed operation/errors | Separately selected async provider; configured namespace, limits, lifecycle and uncertain-outcome handling remain explicit. Optional native `session-redis-tls` selects checked `rediss://`; Redis names the protocol API and Valkey the selected reference server. No generic HA/failover/restore promise |
| [`yetlet_browser::wgpu`](../crates/yetlet-browser/guides/wgpu.md) / [support](../crates/yetlet-browser/guides/wgpu-support.md) | Bounded scene/batch/lifetime and picking APIs | Separate optional flat-scene renderer. CPU-only defaults; explicit `wgpu-browser` plus WASM target selects WebGPU. Scene/device/lifecycle errors remain explicit; no DOM acceleration, stable API, 3D or performance guarantee |

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

From the repository root, select features and target explicitly:

```sh
cargo doc --offline --locked --no-deps -p yetlet --features server-axum,auth-session,rate-limit,config,secret-provider
cargo doc --offline --locked --no-deps -p yetlet-browser --target wasm32-unknown-unknown --all-features
```

The native entry page is `target/doc/yetlet/index.html`; browser output is under
`target/wasm32-unknown-unknown/doc/yetlet_browser/index.html`. Signing separately
requires its documented native OpenSSL dependency. Local rustdoc instructions
are not fresh platform qualification or a docs.rs publication claim.
