# Settings, typed options and application logging

Choose a settings source at native startup, deserialize application-owned
`Settings`, validate them, map the selected values into plain options, then
construct services and native state. Logging follows the same ownership rule:
the application chooses and initializes its subscriber.

These are existing experimental, unpublished source-checkout integrations.
They add no required DI container, settings service or new runtime.

## Prerequisites

Run the command below from the repository root with the pinned Rust toolchain
and native target. The optional `yetlet-config` package is a separate native-only
workspace; it does not support browser/WASM builds. `--offline` requires its
locked dependencies in an already populated Cargo cache. See the
[configuration guide](CONFIGURATION.md#prerequisites) for setup and source rules.

The typed example uses its committed base/profile files and a synthetic token.
Admin and Starter have their own settings schemas and preparation requirements;
do not copy the small example's keys into those applications. Follow the
[Admin README](../examples/admin-dashboard/README.md#prepare-and-execute) or
[Starter README](../examples/production-starter/README.md#run-with-existing-tools)
before running either application.

## Minimal integration

### Run the existing typed configuration example

This command is an instruction, not a fresh execution receipt. Its environment
assignments are scoped to the subshell and contain only a synthetic placeholder:

```sh
(
  export YTCONFIG_ENVIRONMENT=Production
  export YTCONFIG_SERVICE__TOKEN=example
  cargo run --offline --locked --manifest-path crates/yetlet-config/Cargo.toml --example typed -- --check
)
```

The [complete typed example](../crates/yetlet-config/examples/typed.rs) defines
its own `Deserialize` settings structs, uses `SecretString` for its token and
rejects unknown fields. Its startup wiring contains:

```rust
let settings: Settings = loader.load()?;
settings.validate()?;
```

This is a wiring fragment from that example, not a standalone program.
`Settings`, `loader` and `validate()` belong to the application. The surrounding
`main` handles errors and checks `--check` before constructing a listener.
The loader binds the **whole merged root** to the requested Serde type; there
is no public section-selection or section-binding API.

### Map settings into source-independent options

Admin's [complete startup source](../examples/admin-dashboard/src/main.rs)
loads and validates its settings, constructs selected Session/frontend inputs,
then uses this application-owned mapping for both demo and OIDC constructors:

```rust
fn board_options(
    settings: &config::Settings,
    assets: &vite::Snapshot,
    sessions: yetlet_lab_operations_board::SessionBackend,
) -> BoardOptions {
    BoardOptions {
        stylesheets: assets.stylesheet_paths(),
        triage_script: assets.entry_script_path(),
        sessions,
        write_authorization: settings.write_authorization.enabled,
        request_tracing: settings.request_tracing.enabled,
    }
}
```

This wiring fragment assumes Admin's `config` and `vite` modules and its
`BoardOptions` import. The same source passes the result to
`Board::open_with_options` or `Board::open_with_oidc_and_options`.
The [Board constructors](../examples/lab-operations-board/src/lib.rs) consume
plain options without reading environment variables or invoking a loader.
Direct callers may construct those options themselves.

The [Shop startup](../examples/modular-shop/src/main.rs) instead parses its
native environment input into `DatabaseOptions { catalog_cache }`, then calls
[`Database::open_with_options`](../examples/modular-shop/src/database.rs).
It needs no `Loader`. Keep each consumer's external keys and validation intact.

### Construct only the state handlers need

Use native `State<S>` with constructed runtime dependencies or explicitly
selected settings. [Starter startup](../examples/production-starter/src/main.rs)
consumes settings to configure logging, open its store and select metrics, then
passes the store, protected API token and readiness flag into
[its web state](../examples/production-starter/src/web.rs).
It does not require handlers to retain the complete settings object.

The small typed example intentionally demonstrates `State<Arc<Settings>>` and
renders only `service.public_message`. That teaching example is a choice, not a
global all-settings-state convention. Keep secrets out of SSR, JSON, hydration
and browser builds regardless of how native state is arranged.

### Initialize application logging explicitly

For developer diagnostics and production operation, use `tracing` events for
discrete outcomes and spans for related work. Choose `info!` for ordinary
lifecycle/outcome events, `warn!` for conditions needing attention and `error!`
for failures. Select safe fields deliberately; a severity level does not redact
data. Application-owned code decides which events to emit and what they mean.

Starter's [complete entry point](../examples/production-starter/src/main.rs)
initializes its subscriber after validated configuration and before service
construction. This existing wiring fragment assumes that source's `settings`:

```rust
tracing_subscriber::fmt()
    .json()
    .with_target(false)
    .with_max_level(settings.logging.level.filter())
    .try_init()
    .map_err(|_| "logging_unavailable")?;
```

Starter emits lifecycle events and an error for incomplete shutdown; its
[HTTP middleware](../examples/production-starter/src/web.rs) instruments an
`http_request` span with selected request ID/method fields and a response event.
Use those sources as concrete event/span examples. The application initializes
the subscriber once; a library constructor must not install a global subscriber.

Admin uses a different, deliberately narrow
[subscriber](../examples/admin-dashboard/src/tracing_output.rs): JSON to stderr,
INFO for `yetlet_lab_operations_board::REQUEST_TRACING_TARGET`, with current
span context.
Its [startup](../examples/admin-dashboard/src/main.rs) installs this only when
request tracing is compiled and selected in settings. It is not a general
application-wide logging configuration.

## Expected result

The typed example prints only `Configuration valid; listener not started.` and
exits before binding. It has loaded the ordinary sources and passed app-owned
validation; it has not authenticated a user or tested a deployment.

For a prepared host application, the selected constructors receive ordinary
Rust values. Handlers obtain the application's chosen native state. Enabled
diagnostics flow through the host's subscriber and filter. Admin request tracing
distinguishes response construction, dropped HTTP futures and SQL-worker
outcomes; it does not prove response-body delivery or durable audit storage.

## Configuration and errors

### Sources, defaults and startup validation

`Loader::defaults(toml)` replaces `base.toml` as the lowest source. Then comes
the required selected profile, an explicitly selected Development-only secret
file, and prefixed OS overrides. The case-sensitive environment selector accepts
`Development`, `Staging` or `Production`; omission selects Production.
`load_with_env` uses supplied overrides but still reads the selected files.

Base/default TOML defines allowed keys and value kinds. Overlay tables merge
recursively; overlays cannot introduce keys or change kinds. TOML layers can
replace arrays; OS/dotenv overrides support scalar leaves only. Preserve the
application's declared schema, Serde defaults and exact boolean parsing. See
[source precedence](CONFIGURATION.md#sources-and-deterministic-precedence).

Loader errors redact source details, but application errors and arbitrary
`Debug` implementations need their own protection. Do not print settings,
credentials or secret values. Deserialization is not business validation:
applications must explicitly validate required values and allowed combinations
before exposing the service. Constructors retain validation for direct callers.
There is no automatic business-validation hook, watcher or reload. Options
select startup behavior; they do not reconfigure an existing instance.

### Compile availability and instance selection

Cargo features determine compiled capabilities and may add optional dependencies.
Startup settings choose which compiled capabilities an instance installs.
`enabled = false` means no installation of that capability, not a blanket
guarantee of zero compile cost. No required `Loader`, DI, `IOptions` registry or
monitor is involved.

Admin's `request_tracing.enabled` defaults to false. Selecting it through
`APP_REQUEST_TRACING__ENABLED=true` requires the `request-tracing` Cargo feature;
otherwise startup fails with a fixed configuration error. Write authorization
is independently selected. Board options preserve both checks for embedded
callers; unsupported selections fail before database/provider work.

Starter's `logging.level` / `APP_LOGGING__LEVEL` accepts `error`, `warn`, `info`
or `debug`; its base defaults to `info` and output is JSON. Neither example
provides a pretty/JSON switch, `RUST_LOG`/`EnvFilter` configuration or live level
reload. Those would be explicit application changes, not existing settings.

### Diagnostic and security boundaries

Core requires no logger. Keep subscriber initialization, filtering and sink
ownership in the native host. Preserve Admin's target allowlist and explicit
worker context propagation when extending diagnostics. Never dump credentials,
cookies, headers, bodies, query strings or complete requests/settings into logs.
Do not turn arbitrary inbound values into trusted correlation identifiers.

Request spans are local diagnostics; they do not establish distributed tracing,
an exporter, audit completeness or delivery guarantees. Authorization, CSRF,
secret ownership, persistence and retry policy remain application concerns.
The [opt-in secret integration](SETTINGS-SECRET-PROVIDER.md) is separate from
ordinary loading and introduces no automatic refresh or secret logging.

## API reference

See [Loader source/rustdoc](../crates/yetlet-config/src/lib.rs),
[configuration details](CONFIGURATION.md),
[Board options and constructors](../examples/lab-operations-board/src/lib.rs),
[Shop options](../examples/modular-shop/src/database.rs),
[Starter settings](../examples/production-starter/src/config.rs) and
[Starter usage](../examples/production-starter/README.md).
The [current API reference](API-REFERENCE.md),
[publication scope](DOCUMENTATION-SCOPE.md) and
[ADR 0006](adr/0006-handler-first-mvc-optional.md) define current availability,
compatibility/licensing navigation and native handler/state boundaries.
