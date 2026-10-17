# Opt-in typed native application configuration

**YT-090 / M16:** Done at the experimental, unpublished, Linux-tested scope after
independent Reviewer PASS. This is an explicitly selected application dependency, outside
the original nine public candidates. It does not change CLI templates, cloud
configuration generators, core, default SSR or browser dependencies. See the
[scope and verification record](YT-090-VERIFICATION.md).

## Choose an application-owned settings type

The isolated [yetlet-config workspace](../crates/yetlet-config/Cargo.toml) loads
ordinary Serde settings once at startup. It adds no DI container, framework-wide
configuration interface, provider registry, middleware or per-request IO.
Applications opt in with an explicit path dependency; callers control when to
load, validate and construct native Axum state.

The [runnable typed example](../crates/yetlet-config/examples/typed.rs) derives
`Deserialize` and `Debug` on app-owned settings, uses `#[serde(deny_unknown_fields)]`
on every settings struct, and stores its synthetic token in
`secrecy::SecretString`. Its explicit `validate()` runs **before binding a
listener**, rejects an empty/whitespace-only required token and invalid
application values, and returns only a fixed error message. Secret presence and
business validation belong to the application: the generic loader cannot know
whether an empty string is required or optional. Base TOML contains `token = ""`
to declare the string key/type; the application requires a later source to fill it.

State wiring uses the existing API:

```rust
App::new()
    .get("/", home)
    .with_state(Arc::new(settings))
    .into_router()
```

The handler extracts `State<Arc<Settings>>` and explicitly selects only
`service.public_message` for escaped SSR. Settings have no Serialize derive.
Secrets must never be serialized into SSR, hydration, browser/WASM, JSON,
telemetry or diagnostics. `ExposeSecret` access is an explicit application
responsibility; this configuration example implements no authentication.

## Sources and deterministic precedence

Low to high, the loader uses exactly:

1. App-provided TOML defaults through `Loader::defaults`, **or** `base.toml` in an
   explicit application directory (not both).
2. Exactly one `development.toml`, `staging.toml` or `production.toml` in that
   directory, selected by `<PREFIX>_ENVIRONMENT` from the OS snapshot.
3. Exactly one explicitly enabled development secret file through
   `development_secrets(path, SecretFormat::Dotenv | SecretFormat::Toml)`.
4. Real prefixed OS environment overrides.

The selector accepts only `Development`, `Staging`, `Production` (case-sensitive).
Absent selection defaults to **Production**. It is resolved and validated before
any file access, cannot come from a dev file, and never becomes a path component.
Staging and Production never inspect/read a configured dev-secret path. No
current-directory ancestor search, implicit HOME discovery, OS environment
mutation, network request, watcher or reload occurs. Explicit relative paths
resolve against the caller's working directory; prefer an app-owned absolute
path for predictable startup, as the example does with `CARGO_MANIFEST_DIR`.

Every selected file is required. Missing, unreadable, invalid UTF-8, malformed,
oversized or nonregular files fail; permission and parse errors are never
silently treated as an absent optional source. Base/selected profile TOML is
application configuration; do not put deployment credentials in checked-in files.

Base TOML declares the available keys and value kinds. Higher TOML layers may
only replace existing keys of the same kind; tables merge recursively. Unknown
keys and table/scalar collisions fail even when Serde would otherwise ignore
them. Arrays may be replaced by TOML layers and are checked by app Serde types;
OS/dotenv overrides support only string, integer, boolean and finite float
leaves. An override's kind comes from the base value: a token declared as a
string remains exactly `000123`, `true`, or any other supplied string. The
adapter never guesses its type from its content. Boolean overrides accept
`true` or `false`; integer/range errors and typed deserialization errors fail.

Prefixes use uppercase ASCII letters/digits/single underscores, start with a
letter, have no trailing underscore, and are at most 64 bytes. TOML keys use
lowercase ASCII letters/digits/single underscores, start with a letter and have
no trailing underscore. Double underscores are reserved for nesting. For
prefix `YTCONFIG`, `YTCONFIG_SERVER__BIND` overrides `server.bind`, while
`YTCONFIG_SERVICE__TOKEN` overrides `service.token`. Related OS keys must use
canonical uppercase spelling. Unknown/malformed keys, non-Unicode related
keys/values, duplicate snapshot keys, duplicate dotenv keys and collisions fail.
Unrelated OS variables are ignored; unrelated dotenv keys fail. The top-level
`environment` key is reserved and rejected in TOML. Keys must be at most 256
bytes and OS values at most 64 KiB.

## Development secrets and ownership

Development secrets are disabled unless the caller explicitly selects a path.
An ignored `.env.local` is a local convenience, **not an encrypted vault**. There
is no secret write/import command. The committed
[dotenv example](../crates/yetlet-config/examples/settings/dotenv-sample.txt)
contains synthetic placeholders only. The commands below require the pinned
Rust 1.99 toolchain and the dependencies from this isolated workspace's lockfile.
Before the first offline build, explicitly fetch those public dependencies into
a writable Cargo cache (this command uses the network):

```sh
cargo fetch --locked --manifest-path crates/yetlet-config/Cargo.toml
```

A restricted environment may instead use an explicitly selected, user-owned
`CARGO_HOME`; use that same cache for the subsequent commands. The loader itself
does not fetch dependencies or access the network. To exercise the explicit
development file without creating a secret file:

```sh
YTCONFIG_ENVIRONMENT=Development cargo run --offline --locked \
  --manifest-path crates/yetlet-config/Cargo.toml --example typed -- \
  --check --dev-secrets crates/yetlet-config/examples/settings/dotenv-sample.txt
```

Dotenv parsing uses `dotenvy::from_read_iter`, never its process-environment
loading APIs. Dotenvy ordinarily expands `$NAME`/`${NAME}` against ambient OS
variables and preceding entries. This adapter rejects **all dollar signs before
parsing**, including quoted/escaped dollar signs, so there is no expansion or
shell sourcing/command execution. Use the explicit TOML secret-file option for
literal dollar-containing values. Quotes/escapes/comments otherwise follow
pinned dotenvy parsing. Duplicate keys are rejected rather than last-wins.

An application may explicitly supply a protected user-private file, for example
a path under `$HOME/.config/myapp/development.toml` or a Windows user-owned path built
with `PathBuf`, using `SecretFormat::Toml`. There is no automatic lookup there.
The operator must own/protect the file and **all parent directories**, keep the
path outside VCS/backups shared with others, and set appropriate file/directory
permissions (for example Unix 0600/0700) or Windows ACLs. The loader does not set,
audit or guarantee permissions, ACLs, encryption, keychain storage or ownership.

Reads are bounded to 64 KiB per file, including a one-byte over-limit check if
the file grows. Nonregular final inputs are rejected. Unix opens use
`O_NOFOLLOW | O_NONBLOCK`; Windows opens use `FILE_FLAG_OPEN_REPARSE_POINT` and
reject final reparse points. Final symlinks are rejected for every input.
Intermediate path components must remain operator-controlled; this is not a
hostile-filesystem/TOCTOU certification. Windows runtime execution and ACL
behavior are **Not verified** here; portable path joining and explicit paths are
covered by source/tests on Linux. Other native targets have only the portable
metadata checks, without a certified atomic no-follow guarantee.

Adapter errors/Debug contain fixed categories only, no input/key/value/path or
upstream source chain. Loader Debug omits all fields; SecretString redacts the
example token's Debug. Application-defined Debug/errors can still expose other
fields; the application must decide which values are sensitive. No claim is
made that all temporary strings, parser allocations or environment snapshots
are zeroized. No actual private secret is needed for these examples/tests.

## Run without starting a listener

From the repository root, use synthetic placeholders with each finite profile:

```sh
YTCONFIG_ENVIRONMENT=Development YTCONFIG_SERVICE__TOKEN=000123-placeholder \
  cargo run --offline --locked --manifest-path crates/yetlet-config/Cargo.toml --example typed -- --check
YTCONFIG_ENVIRONMENT=Staging YTCONFIG_SERVICE__TOKEN=000123-placeholder \
  cargo run --offline --locked --manifest-path crates/yetlet-config/Cargo.toml --example typed -- --check
YTCONFIG_ENVIRONMENT=Production YTCONFIG_SERVICE__TOKEN=000123-placeholder \
  cargo run --offline --locked --manifest-path crates/yetlet-config/Cargo.toml --example typed -- --check
```

Each exits before binding and prints only the fixed summary
`Configuration valid; listener not started.` Remove `--check` only when you
intend to run the foreground example on the configured loopback address.
PowerShell callers can set the same process-local environment keys with `$env:`;
there is no Windows execution receipt for this workspace yet.

## Dependency and scope boundary

The adapter pins `toml = 0.9.12` with only std/parse/serde, Serde 1.0.229 and
dotenvy 0.15.7; libc 0.2.186 is Unix-only for the no-follow/nonblocking file open.
Secrecy 0.10.3, Axum/Tokio/Tower and Yetlet's native SSR crates are example/test
dependencies only. The nested Cargo.lock pins the resolved graph. Cargo fetched
and inspected `config` 0.15.27, but its deserializer permits implicit scalar
coercion. The smaller direct TOML adapter preserves strict Serde scalar types
and implements only bounded loading, known-key merging and typed overrides;
it does not implement a format parser or general provider system.

External vaults, encrypted secret storage, certificates, hot reload, Tailwind
(YT-091), DI, signing, production starters/admin, formal release and CI/CD are
deferred. A distinct follow-up, [YT-097](BACKLOG.md#m16--opt-in-typed-application-configuration),
proposes opt-in CLI scaffolding and is not implemented by this crate; no existing
CLI command or template was changed. YT-020 stays Blocked. The separate
independent read-only Reviewer passed the bounded YT-090 implementation; that
acceptance does not activate publication, production qualification or any later task.
