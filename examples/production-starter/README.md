# Issue Tracker production starter (YT-092)

A small, copyable native Rust SSR/API reference application. Create an issue,
read its list/detail, change its status, then restart the process: SQLite retains
committed issues. All pages/forms work without JavaScript, WASM, Vite or a browser
server circuit. This application explicitly selects SQLite, settings, anonymous
CSRF sessions and request IDs; none becomes a framework default.

This is an experimental reference, not supported hosting, HA, OIDC or security
qualification. The anonymous operator pages bind **127.0.0.1 only** in every
profile. Run locally or behind an operator-managed access-controlled boundary.
The command API has a separate shared Bearer token; it does not authenticate the
anonymous SSR pages and is not a user/role system. Do not expose this board as a
public service. Sessions are process-local and expire on restart; issue data is
persistent.

## Run with existing tools

Requires Rust 1.99+ and a native C compiler for bundled SQLite. The repository
CLI is unpublished; use its locally built executable directly. From this
directory, prepare the CLI and app's dependency cache once. On Linux, the app directory
and CLI inputs must be owned by you and not group/world-writable, as required
by the existing [CLI filesystem contract](../../tools/yetlet-cli/README.md):

```bash
cargo build --locked
cargo build --locked --manifest-path ../../tools/yetlet-cli/Cargo.toml
```

Create a private `.env.development` containing `APP_SERVICE__API_TOKEN=` followed
by your unique 64-character lowercase hexadecimal secret. For example, with an
existing OpenSSL executable on Unix, create the file once without overwriting:

```bash
(
  umask 077
  set -C
  {
    printf '%s' 'APP_SERVICE__API_TOKEN='
    openssl rand -hex 32
  } > .env.development
)
```

Select the existing Development secret source, then invoke only the Yetlet CLI:

```bash
export APP_ENVIRONMENT=Development
export YETLET_DEV_SECRETS="$PWD/.env.development"
../../tools/yetlet-cli/target/debug/yetlet run
```

On Windows PowerShell, prepare the same private secret file, protect it with
an operator-owned ACL, then set `$env:APP_ENVIRONMENT = 'Development'` and
`$env:YETLET_DEV_SECRETS = (Join-Path $PWD '.env.development')`. Run the same
locally built CLI with `& ..\..\tools\yetlet-cli\target\debug\yetlet.exe run`.
Python and an outer `cargo run` are not required to launch the application.
If the CLI is already on PATH, the launch command is simply `yetlet run`.

Open <http://127.0.0.1:3010/>. Ctrl-C gracefully drains HTTP work and exits.
`yetlet watch --manifest-path <absolute-app-directory>/Cargo.toml` selects the existing Watchexec
watch/rebuild/restart path; Watchexec must already be installed. `yetlet build`
uses the app-owned lockfile and offline Cargo cache:

```bash
../../tools/yetlet-cli/target/debug/yetlet build
```

The existing CLI reads `[cli]` in the same base/profile settings; Vite is disabled.
There are no new CLI options. CLI generators still accept only their declared
scaffolds: this edited reference app does not extend `new --service`, Docker,
Compose or Kubernetes generation. See [CLI contract](../../docs/CLI-CONTRACT.md).

## Manual acceptance

Use this short journey to check the application yourself. Keep the CLI terminal
open so you can stop and restart the same app with the same settings/database.

| Step | Action | Expected result |
|---|---|---|
| 1 | Create an issue with a recognizable title and description | The detail page shows your exact text, status `open` and version 1 |
| 2 | Follow "All issues", then reopen your issue | The list and detail show the same saved issue |
| 3 | Select "In progress" and submit "Update status" | The detail shows `in_progress` and version 2 |
| 4 | Refresh the browser page | The same issue, status and version remain |
| 5 | Press Ctrl-C in the CLI terminal, run the same `yetlet run` command, then refresh | The committed issue remains after restart |

The automated tests separately cover stale-write conflicts, invalid input/body
limits, CSRF rejection, escaping/CSP, configuration precedence and secret-safe
errors/logs. This journey checks visible functionality; formal visual,
accessibility and cross-browser qualification remain separate work.

## Settings and secrets

`yetlet-config` loads `base.toml` → selected `development.toml`, `staging.toml` or
`production.toml` → explicitly selected Development secret file → `APP_*` OS
overrides. `APP_ENVIRONMENT` accepts exactly `Development`, `Staging`,
`Production`; omission selects Production. Every application settings structure
rejects unknown fields. There is no automatic `.env` discovery.

- `YETLET_CONFIG_ROOT`: optional absolute settings directory; parent traversal is
  rejected. Copy all four TOML files there for runtime deployment. It also anchors
  relative database paths, independently of the process working directory.
- `YETLET_DEV_SECRETS`: absolute dotenv file, read only in Development. Staging
  and Production do not inspect even a selected missing secret file.
- `APP_SERVICE__API_TOKEN`: required 64-character lowercase hexadecimal value.
  No usable credential is shipped. Use a unique secret for each installation.
- `APP_DATABASE__PATH`: persistent database path. Default `data/issues.sqlite3`
  is relative to the selected configuration root; an absolute operator-owned
  volume path is supported. Parent traversal, `:memory:` and Windows drive-relative/root-only paths are rejected.
- `APP_SERVER__PORT`: integer 1–65535 (default 3010). Host remains loopback-only.
- `APP_LOGGING__LEVEL`: `error`, `warn`, `info` (default), or `debug`.

For Staging/Production, supply the API secret through the process environment
from the operator's secret manager. Never copy `.env.development` into images,
public assets or version control. The server does not serialize settings, tokens,
paths or database diagnostics. Configure the selected settings root explicitly
when moving the built executable: its default is the original manifest directory.

## HTTP contract

| Route | Behavior |
|---|---|
| `GET /` | Latest 100 issues and create form |
| `POST /issues` | CSRF-protected create; 303 redirect to detail |
| `GET /issues/{id}` | Detail and status form |
| `POST /issues/{id}/status` | CSRF-protected versioned status update; 303 redirect |
| `GET /api/issues` | Latest 100 issues as JSON |
| `POST /api/issues` | JSON `{title, body}`; 201 with issue/location |
| `GET /api/issues/{id}` | JSON issue |
| `PATCH /api/issues/{id}` | JSON `{status, expected_version}` |
| `GET /health/live` | Process liveness independent of database |
| `GET /health/ready` | Lifecycle readiness and real schema/connection probe |

All `/api` routes require `Authorization: Bearer <token>` and reject browser
`Origin` headers. They are command-oriented; there is no CORS or browser token
injection. Status values are `open`, `in_progress`, `closed`. An issue contains
`id`, `title`, `body`, `status`, `version`. Both forms and JSON cap request bodies
at 16 KiB, title at 120 characters and body at 4,000. Concurrent updates use the
expected version: a stale write returns 409 without changing stored data.
Invalid requests return bounded errors; unknown database failures return generic
500, busy/unavailable storage returns 503, missing issues return 404.

Pages use the existing escaped renderer, English document metadata and strict
CSP. Anonymous sessions use HttpOnly/SameSite cookies, explicit local HTTP mode
and hidden CSRF tokens. API credentials never enter HTML or session state.

## Operate, copy and extend

The app initializes and checks its versioned SQL schema before binding. SQLite
uses WAL, a two-second busy timeout, and at most four blocking operations. Each
operation opens/closes a connection on Tokio's blocking pool; no connection is
held across an async await. Startup rejects unsupported or malformed schemas.
Readiness probes the real database. Shutdown registers native signals before
bind, marks the process unready, drains Axum handlers, closes their connections,
and emits `stopped`. Linux SIGINT/SIGTERM and Windows Ctrl-C/Ctrl-Break are handled.

Logs are JSON tracing events containing a fixed event name, process-local numeric
request ID, method, status and duration. No request URI/query, bodies, headers,
secret values or raw error chains are logged. `x-yetlet-request-id` is generated
by the server, overwrites inbound IDs, and resets on process restart. Configure
log collection/rotation outside this application. The sample has no metrics,
distributed tracing or persistent sessions.

Keep the database and its WAL/SHM siblings on writable persistent local storage.
Use SQLite's supported backup workflow or stop the process before copying the
complete database; do not copy only a live main file or delete WAL files. Restore
into a separate local directory and verify the application can open it before
switching an instance. This example does not qualify network filesystems,
multiple replicas, migration rollback, long-load performance or HA.

Copy this directory and retain its `Cargo.lock`. Until Yetlet is published, adjust
the five Yetlet dependency paths in `Cargo.toml` to your framework checkout;
`yetlet-server-axum` resolves its companion local crates there. No registry
release is assumed. Application-specific SQL lives in `migrations/`, settings in
`src/config.rs`, handlers/views in `src/web.rs`, persistence in `src/store.rs`,
and startup/signal ownership in `src/main.rs`. There is no ORM, DI container or
required Controller. Authentication, pagination and deletion are later app choices.

```bash
cargo test --locked --offline
cargo clippy --locked --offline --all-targets -- -D warnings
cargo fmt --check
```

See [YT-092 verification](../../docs/YT-092-STARTER-VERIFICATION.md) for actual
platform evidence, limitations and the lessons that should inform later
scaffolding work.
