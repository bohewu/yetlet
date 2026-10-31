# Opt-in integration quick reference

Choose an existing capability, then follow its linked guide. This lookup page
concentrates compile selection, explicit startup installation, consumer-owned
settings and defaults. It describes the published Experimental SDK
`0.9.0-alpha.5` and the explicitly identified application-owned integrations;
it does not introduce another configuration schema or a generic
capability builder. Read source/rustdoc and guides at the same revision.

## Three separate decisions

1. **Package and target:** add the specific dependency to the application that
   uses it. Keep server-only providers out of browser WASM and hydration state.
2. **Cargo feature, where defined:** compile an optional adapter/integration.
   Features named below belong to the exact package/consumer in that row; there
   is no workspace-global Auth/Tracing/Cache switch.
3. **Startup wiring:** construct/install the selected layer, provider or typed
   application option. A compiled feature does not select that installation.

Settings are inputs to step 3 only when the application chooses a loader or
explicit environment parsing. `yetlet::config::Loader` is an optional settings
loader, separate from `SecretProvider`; no SettingsProvider trait exists.
Admin already depends on Loader for its own settings. That application choice
is not a requirement for Board embedding or other Yetlet consumers.

For the complete startup sequence and existing Config, Admin/Board, Shop and
Starter examples, follow [Settings, options and logging](SETTINGS-OPTIONS-LOGGING.md).

## Framework capabilities

“No built-in key” means the package does not read an ambient enable variable;
application-owned inputs can still configure its explicit API. “Not installed”
means the capability is absent from a basic core/server app unless selected,
not that a listed sample using it disables its own store or middleware.

| Capability / dependency | Cargo feature or target selection | Explicit startup API | Settings and default | Existing integration |
|---|---|---|---|---|
| Memory Session: `yetlet::session` | `yetlet/session` on native targets | Validate SessionConfig, create SessionManager and install `scoped` on the intended routes | No built-in key; not installed automatically. Independent managers/processes have independent Memory state | [Session cart and guide](../crates/yetlet/guides/session.md#minimal-integration) |
| Authentication: `yetlet::auth` | `yetlet/auth` on native targets | Authentication plus resolver/state/anonymous response, then the native authenticate middleware | No built-in key or login provider; installing it resolves identity, while business authorization stays explicit | [Auth wiring](../crates/yetlet/guides/auth.md#native-composition) |
| Memory Session → Auth bridge: `yetlet::auth` | `yetlet/auth-session` adds the Memory bridge | `session::memory` inside the resolver; Session middleware outside Auth | No built-in key; the feature provides the helper, not automatic Session/Auth installation or a Valkey adapter | [Bridge order and failures](../crates/yetlet/guides/auth.md#optional-memory-session-bridge) |
| Request correlation: `yetlet::request_id` | `yetlet/request-id` on native targets | Construct RequestIds; use `scoped` or final `global` wrapping | No built-in key; no automatic counter. IDs are provider-local, not distributed tracing or identity | [Request ID wiring](../crates/yetlet/guides/request-id.md#install-a-scoped-provider) |
| Rate limiting: `yetlet::rate_limit` | `yetlet/rate-limit` on native targets | Validated Policy → Budget → RateLimitLayer with a trusted key/scope | Library has no enable key. Its **native example** loads `RATE_LIMIT_ENABLED` (false) and `RATE_LIMIT_POLICY__INTERVAL_MS` / `RATE_LIMIT_POLICY__BURST` / `RATE_LIMIT_POLICY__MAX_KEYS` (2000 / 5 / 64) | [Settings example](../crates/yetlet/guides/rate-limit.md#opt-in-settings-loader-in-the-native-example) |
| Typed settings: `yetlet::config` | `yetlet/config` on native targets | Construct Loader with the app's directory/prefix, load and validate typed settings at startup | App declares keys/prefix/defaults; absent profile selector means Production. Adding the dependency performs no load | [Configuration](CONFIGURATION.md#minimal-integration) |
| Secret lookup: `yetlet::secrets` | `yetlet/secrets` on native targets | Construct/inject `Arc<dyn SecretProvider>` into app-owned state or clients | No enable key or automatic vault; app chooses required values. SettingsSecrets refresh is explicit | [Secret providers](SECRET-PROVIDERS.md#minimal-integration) |
| Secret injection into settings: `yetlet::config` | Explicit `secret-provider` feature | Existing Loader plus injected provider and typed SecretBinding selections | Only explicitly bound fields use the provider; missing/access/unavailable fail without fallback. Ordinary load does not call it | [Typed binding contract](SETTINGS-SECRET-PROVIDER.md#contract) |
| Remote Session: `yetlet::session_redis` | `yetlet/session-redis` on native targets | Validated RedisSessionConfig, then explicit async RedisSessionManager connect/install | No built-in key; app owns endpoint, namespace, limits and credentials. This async provider does not replace the synchronous Memory type transparently | [Provider API](../crates/yetlet/guides/session-redis.md#api-and-application-responsibilities) |
| TLS for that remote provider | Its explicit `session-redis-tls` feature plus selected `rediss://` endpoint | Same provider connect API; standard certificate/hostname verification stays enabled | TLS is default-off; compiling it alone does not choose an endpoint or deployment. Redis is the protocol/API name; Valkey is the reference server | [TLS requirements](../crates/yetlet/guides/session-redis.md#optional-native-tls--yt-184-reference) |
| Browser DOM: `yetlet_browser::dom` | `yetlet-browser/dom`, WASM target for mount; native static validate is separate | Explicit `mount` on an owned browser host after the selected asset build | No enable key or automatic activation. Linking core/SSR does not install a browser runtime | [DOM guide](../crates/yetlet-browser/guides/dom.md#minimal-integration) |

These rows cover common integration paths rather than every `pub` item or every
optional renderer/signing operation. Use the [current API map](API-REFERENCE.md)
for the remaining implemented packages and rustdoc target/feature commands.
Policy reuse does not share rate-limit accounting: clone/share a Budget when
that group total is intended. Auth, Session, CSRF, response caching and tracing
remain separate responsibilities even when installed on the same route group.

## Application-owned integrations

These are existing **example application** interfaces, not framework-wide
packages, default features or settings keys. Embedded hosts can construct typed
options directly without the Settings loader. Admin maps its already selected
settings; Shop's binary maps its own explicit environment value.

| Existing consumer | Compiled feature | Typed selection / startup | External selection and default | Existing guide |
|---|---|---|---|---|
| Board/Admin sensitive write authority | `commit-write-auth` on the selected app | BoardOptions.write_authorization through `Board::open_with_options` or `open_with_oidc_and_options` | Admin `write_authorization.enabled` / `APP_WRITE_AUTHORIZATION__ENABLED`, false. Board default options stay false | [Write authority](../examples/admin-dashboard/README.md#opt-in-commit-time-write-authorization--yt-181) |
| Board/Admin request and SQL-worker tracing | `request-tracing` on the selected app | BoardOptions.request_tracing through the same options constructors; embedding host owns subscriber setup | Admin `request_tracing.enabled` / `APP_REQUEST_TRACING__ENABLED`, false. Compiling the feature alone stays off | [Safe tracing](../examples/admin-dashboard/README.md#opt-in-request-and-sql-worker-tracing--yt-182) |
| Shop catalog display cache | `catalog-cache` on `yetlet-modular-shop` | DatabaseOptions.catalog_cache through `Database::open_with_options`, then the existing app constructor | Shop binary `YETLET_SHOP_CATALOG_CACHE_ENABLED`, absent/false stays off; only exact true/false accepted. It uses no Settings loader for this flag | [Display cache](../examples/modular-shop/README.md#optional-catalog-display-cache--yt-183) |
| Board/Admin Valkey-backed Auth resolver | `auth-valkey` (includes the app's `valkey` feature) | BoardOptions.sessions selects SessionBackend::AuthValkey with its required inputs; BoardOptions default is Memory | Admin `session.backend` / `APP_SESSION__BACKEND=valkey`, required Valkey settings, and `auth.enabled` / `APP_AUTH__ENABLED=true` with absolute `auth.policy_file` / `APP_AUTH__POLICY_FILE`. Auth default false; local authority selection is Linux-only | [Auth adoption](../examples/admin-dashboard/README.md#optional-auth-adoption-yt-126) |

The constructors, full settings types and safe setup sequences in those guides
remain authoritative. In particular, the write-authority reference needs a new,
nonexistent private SQLite file in its fresh-v2 schema; do not enable it against
an existing v1/real database. Reopening v2 denies and advances authority until
trusted current-policy confirmation explicitly re-enables it. Session/account/
resource authorization and CSRF still apply; write authority alone grants none.
Tracing does not imply audit delivery or SQL rollback, and display cache does
not become an authority for prices, checkout, Session or permissions.

Admin validates its feature/settings combination during configuration loading;
Board/Shop reject unsupported true options before opening storage. Defaults preserve the consumer's existing
behavior. Do not infer that every package has a false boolean switch, identical
options type, shared namespace or hot reload.

## Which guide to follow next

- Starting from a basic app: [View/component authoring](VIEW-COMPONENTS.md), then
  the smallest selected package guide above.
- Choosing an API carrier: [current API reference](API-REFERENCE.md) and
  [capability availability](CAPABILITIES.md). Audit/output-cache research does
  not imply implemented general-purpose crates.
- Diagnosing selection/wiring: [troubleshooting](TROUBLESHOOTING.md). Resolve the
  actual source/feature/settings failure without weakening validation or trust.

This is a source-backed quick reference. It adds no settings aliases, constructor,
runtime behavior, installation, executed verification or publication decision.
