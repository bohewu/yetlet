# Development HTTPS — settings, mkcert and CLI lifecycle

Date: 2026-10-18 (Asia/Taipei). Task: YT-102 development HTTPS follow-up.
Status: implementation, Linux/Windows lifecycle checks and
[independent review](DEV-HTTPS-REVIEW.md) passed (zero material issues or required
followups). No push or deployment occurred.

## Authority and scope

The owner authorized development HTTPS through existing settings and mkcert,
for Rust and Vite, with run/stop/build verification, native Windows WebCodex
execution and independent review. The existing secret mechanism remains in use.
This slice does not implement the separately discussed opt-in SecretProvider.

Source: branch `feat/yt090-configuration`, HEAD
`dbcae2dbb47a9e8a0f1e103b49ec225ee019b63f` plus the bounded changes below.
The pre-existing asset/roadmap/backlog planning and YT-103 evidence are preserved.
Evidence is retained under `artifacts/dev-https-20261018/`.

## Selected contract

Settings-enabled scaffolds, including `new --vite`, declare this public section:

```toml
[server.tls]
enabled = false
certificate = ""
private_key = ""
```

Enable it in `development.toml` after separately preparing the certificate:

```toml
[server.tls]
enabled = true
certificate = ".dev/certs/localhost.pem"
private_key = ".dev/certs/localhost-key.pem"
```

The existing loader retains base/profile/scoped-ENV precedence. TLS is an
explicit Development capability; paths are relative to the selected application,
outside its configured frontend source root. Public settings contain paths,
never PEM contents. `.dev/` is excluded from generated version control inputs.
The CLI's public projection does not read the Development secret file.

Generated applications keep TLS dependencies behind the optional
`yetlet-dev-tls` Cargo feature. The CLI selects that feature from the public TLS
settings; no new runtime flag or Cargo metadata settings are introduced. Plain
HTTP and framework/browser defaults remain unchanged. Direct Rust serving uses
the existing Axum router with maintained axum-server/rustls integration.

When Vite automatically starts, the browser-facing origin is HTTPS/WSS on
`127.0.0.1:5173`; Vite reads the prepared leaf certificate/key and bridges the
existing bounded root-page request to private HTTP Rust on `127.0.0.1:3000`.
The CLI fixes this internal backend endpoint and disables backend TLS for that
paired invocation. With Vite auto-start disabled, Rust serves HTTPS directly.
The bridge retains its GET/HEAD, exact Host/Origin, path confinement, nonce/CSP
and no-cookie/no-credential-forwarding boundaries. This does not add a general
forms/API/authentication proxy.

`yetlet build` compiles selected dependencies and runs the existing configured
frontend build. It does not read development TLS credentials, start listeners,
generate certificates or install trust. `run` requires prepared valid material;
configuration/material failures must not leave an active listener or child tree.
The existing native process ownership and cancellation behavior remains in force.

## Prepare development certificates

Install [mkcert](https://github.com/FiloSottile/mkcert) separately. Installing its
local CA with `mkcert -install` is an explicit operator trust action. `new`,
`run` and `build` never perform it. Prepare certificate directories with private
ownership/permissions; keep private material outside frontend sources and Git.

From the application directory, after preparing `.dev/certs/`:

```text
mkcert -cert-file .dev/certs/localhost.pem -key-file .dev/certs/localhost-key.pem localhost 127.0.0.1 ::1
```

Select Development, supply the application's existing secret inputs, then use
the normal `yetlet build` and `yetlet run`. No private signing key or PFX password
from YT-093 is needed to generate these TLS identities.

## Verification boundary

Native commands, results, source identities and cleanup are recorded below.
Test CAs are isolated with `CAROOT`; clients explicitly trust the
test root and validate the hostname. No system trust-store installation or
certificate-error bypass is used for these lifecycle checks. This proves the
configured TLS endpoints without claiming system-wide browser trust.

The initial Linux mkcert preparation found no existing binary. The official
GitHub v1.4.4 release predates asset digests, so the verification uses the
Ubuntu `mkcert 1.4.4-1ubuntu2` package extracted into the task directory. Its
SHA-256 matches package metadata:
`f2ed2a59b2ebce55a97a8efd218e4a0aea35a6c0c233e411728f3f43b3a5b592`.
No global tool installation occurred.

## Linux results

The local CLI generated a service application with `new --settings --service`
and another with `new --vite --service`. Both used explicit local framework
dependency paths, separately generated offline lockfiles, existing synthetic
secret inputs and the isolated mkcert CA. The Vite frontend used `npm ci
--offline --ignore-scripts --no-audit --no-fund`.

| Check | Observed result | Evidence |
| --- | --- | --- |
| Rust and Vite `yetlet build` | Both exit 0 while configured certificate directories do not exist | `local-builds.json`, `tls-*-cli-build-missing-certs.log` |
| Rust `yetlet run` | CA and hostname verified HTTPS 200 on `127.0.0.1:3000` and `localhost:3000`; wrong hostname rejected | `rust-cli-run.json` |
| Vite `yetlet run` | CA and hostname verified HTTPS 200 for page, client and TypeScript entry; trusted WSS handshake 101; internal Rust HTTP health 200 | `vite-cli-run.json` |
| Development source boundaries | Certificate/configuration/foreign filesystem requests rejected with 403; scheme-aware CSP retained | `vite-cli-run.json`, `vite-smoke.json` |
| Actual controlling-terminal Ctrl-C | Both CLI processes retain native SIGINT return code -2 | `tls-rust-stop.json`, `tls-vite-stop.json` |
| Invalid private key | Rust and Vite exit 1, no listener remains and no PEM material appears in logs | `invalid-material.json`, `tls-*-invalid-key.log` |
| Final CLI tests | 125 passed, 0 failed, 5 explicitly ignored across 17 test binaries | `cli-all-targets-final.log`, `checks.json` |
| Vite Node tests | 13 passed, 0 failed | `vite-node-tests.log` |
| Existing generator compatibility | Four settings scaffold profiles across Compose, Docker and Kubernetes passed | `generator-selected.log`, `compose-tests.log` |

Validation commands:

```text
OPENSSL_DIR=/tmp/yt093-native-tools/openssl-install OPENSSL_STATIC=1 cargo test --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets
OPENSSL_DIR=/tmp/yt093-native-tools/openssl-install OPENSSL_STATIC=1 cargo clippy --offline --locked --manifest-path tools/yetlet-cli/Cargo.toml --all-targets -- -D warnings
cargo fmt --manifest-path tools/yetlet-cli/Cargo.toml --all -- --check
node --test tools/yetlet-cli/tests/vite-dev.mjs
cargo xtask check docs
cargo xtask check samples
git diff --check
```

Clippy, formatting, documentation and sample checks passed. The generated Rust
service compiled with the optional TLS feature, and generated runtime tests
also passed. `generated-runtime.log` and `generated-service.log` retain those
separate observations. The ignored tests remain opt-in external/runtime cases;
the final suite's pass count does not claim they executed.

Two earlier full-suite failures are retained. A Docker fixture had a synthetic
lockfile predating the selected manifest's optional TLS dependencies; the
canonical selected-manifest lock validator and this fixture now recognize the
two exact optional dependency edges. A later watch fixture collided with the
coordinator's simultaneously running Vite on port 5173. The service was stopped
and the complete suite passed serially. See `cli-all-targets.log` and
`cli-all-targets-fixed.log`; neither failure is presented as a passing run.

Local closeout verified ports 3000/5173 closed, no task application process,
and the pre-existing starter's port 3010 health still 200. Task-local CA and
leaf private keys were removed. HEAD and the staged index are unchanged; the
pre-existing asset, roadmap and YT-103 plan/evidence files retain their original
bytes. See `local-cleanup.json`.

## Native Windows results

The [native WebCodex record](DEV-HTTPS-WINDOWS-VERIFICATION.md) retains source,
tool identities, exact commands, original harness failures and public logs.
All 197 selected files match the current checkout; the only post-build source
delta is the Docker test fixture, outside compiled runtime inputs.

Rust TLS, Vite TLS and default HTTP builds exited 0; the TLS builds ran before
certificate directories existed. Actual Rust HTTPS validated the isolated CA,
localhost and IP names, and rejected a wrong hostname. Actual Vite HTTPS
returned 200, WSS returned 101, and the fixed private HTTP backend returned 200.
The Rust console Ctrl-C retained native status `0xc000013a`; later inspection
found no task processes/listeners. Vite console Ctrl-Break released six identified
CLI/Cargo/Rust/Node processes and both ports within 1.0324 seconds while an
unrelated sentinel survived. Default HTTP startup/Ctrl-C also passed.

An initial console helper failed while already attached; its one correction
reached real Rust HTTPS/Ctrl-C but then an immediate all-descendant wait failed
on an unidentified PID. The failed wrapper remains recorded. Its identity and
instantaneous cleanup are not claimed; the later process inspection and separate
bounded Vite check supply their explicitly stated evidence.

Native closeout removed the disposable applications, CA private key and leaf
private keys. Task processes/listeners were absent, both Windows root stores and
global Cargo permissions were unchanged, and the previous task's source/CLI
remained intact. This verifies lifecycle and TLS through explicit CA trust;
system browser trust, physical keyboard behavior and general Vite HMR remain
outside this slice.

## Changed files and remaining scope

The implementation changes CLI public-settings projection and run/build/watch
selection (`src/settings.rs`, `src/app.rs`, `src/platform/process/mod.rs`),
selected scaffold generation (`src/scaffold.rs`, `templates/config/tls.rs`,
`templates/config/SELECTED-README.md`), Vite's development configuration and
bridge, and the exact selected-manifest dependency validator (`src/compose.rs`).
Focused Rust/Node tests and the Docker synthetic fixture follow those changes;
`src/platform/mod.rs` updates its static signature assertion. All source paths
in this paragraph are relative to `tools/yetlet-cli/`. The CLI contract,
configuration guide, documentation index and backlog link this bounded slice.

This is the separately authorized development HTTPS follow-up associated with
YT-102. The broader production TLS deployment guidance in YT-102 remains open.
No secret-provider API, automatic secret/certificate rotation, cloud provider,
OS trust installation or production TLS configuration is introduced. YT-094
remains the next separate core backlog discussion; this task does not start it.

## References

- [mkcert setup and certificate generation](https://github.com/FiloSottile/mkcert).
- [axum-server feature selection](https://docs.rs/crate/axum-server/0.8.0/features).
- [Vite server HTTPS and WebSocket options](https://vite.dev/config/server-options).
