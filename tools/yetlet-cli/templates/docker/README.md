# Linux container configuration for @NAME@

This preview writes configuration only. It does not run Cargo, Docker, buildx,
kubectl, a shell, or network clients, and does not copy app/framework sources.
It targets **Linux OCI amd64**, even when generation runs on Windows. Only the
standalone, unedited `new --cloud` native-ssr-v1 profile is supported. The main
source is parsed as Rust and matched to the accepted cloud template, including
pre-bind signal registration. Metadata alone is insufficient. Extra sources,
build scripts, Cargo configuration, dependencies, features and manual layouts
are rejected. This is a bounded profile, not a general Cargo dependency solver.

## Prepare three physically separate contexts

Use three fresh protected directories, with **no ancestor/descendant overlap**:
`DIR` (this generated configuration), `APP_EXPORT`, and `FRAMEWORK_EXPORT`.
Never pass the live app or dirty framework checkout as any build context.
`DIR` is the ONLY default positional context. Its deny-by-default `.dockerignore`
allows Dockerfile and the app overlay. It does **not** protect named contexts.
Named context ignore behavior must not be relied on to sanitize source exports;
physical allowlists and exact inventory comparison are mandatory before upload.
No source-export automation is provided by this slice.

1. Choose private operator-owned parents (0700 on Linux; protected current-user
   ACLs on local fixed NTFS on Windows). Refuse symlinks, junctions, reparse points,
   traversal, untrusted writable ancestry, special files and hardlinks to mutable
   source. Freeze source writers during export, inventory verification and build.
   If a source changes or cannot be inspected safely, stop and regenerate into a
   new output directory. Never replace an existing generated directory.
2. Manually create `APP_EXPORT/src`. Copy **only** the selected app's existing
   `Cargo.lock` and `src/main.rs`, byte for byte, then copy `DIR/app-src.dockerignore`
   to `APP_EXPORT/.dockerignore`. Do not copy raw app `Cargo.toml`: it contains
   local absolute paths. Do not generate or refresh Cargo.lock as part of export.
3. Manually create `FRAMEWORK_EXPORT/crates/{yetlet-core,yetlet-html,yetlet-server-axum}/src`
   as individual directories. Copy **only** the three package `Cargo.toml` files
   and the exact source paths listed in `build-context-inventory.json` under
   `yetlet-src`. Copy `DIR/framework-workspace.toml` to `FRAMEWORK_EXPORT/Cargo.toml`,
   and `DIR/yetlet-src.dockerignore` to `FRAMEWORK_EXPORT/.dockerignore`. The
   generated minimal workspace has resolver 3, exactly three members and exact
   supported inherited workspace.package and workspace.dependencies values.
   Do not copy the monorepo root Cargo.toml or Cargo.lock. Tests, examples, unrelated
   crates and unknown files are outside this export profile.
4. Audit every exported file as public, non-secret build input. Recursively list
   every entry including hidden files and directories **without following links**;
   compare file paths exactly with its context's inventory keys (directories must
   be only their necessary parents). Reject extra/missing files, link/reparse or
   special entries, unsafe ownership/permissions, altered byte lengths or hashes.
   Calculate each regular file's SHA-256 with your OS's trusted hash utility
   (`sha256sum -- FILE` or PowerShell `Get-FileHash -LiteralPath FILE -Algorithm SHA256`)
   and compare all entries, not just Cargo.lock. Also verify every generated file
   against `generated_files_except_inventory` before using DIR. The inventory
   records no timestamps, source absolute paths, environment values or unknown
   files. It is a local snapshot contract, not a signed authenticity guarantee.
5. Explicitly confirm `.env`, `.env.*`, SSH keys, credentials, `.git`, `target`,
   caches and planted secret fixtures are absent from **all** three contexts.
   Do not recursively copy a source tree and rely on ignore rules to remove them.
   No blanket `cp -r`, `COPY . .`, shell evaluation, sed replacement, Git archive
   or live-directory build context is part of this recipe. Keep DIR unchanged.

The app overlay is reconstructed by TOML parsing. Exactly two dependency path
values change, to `/work/yetlet/crates/yetlet-core` and
`/work/yetlet/crates/yetlet-server-axum`; all other supported manifest semantics
remain equal. Internal HTML/core relative dependencies retain their source
values. App sources and the source Cargo.lock are never modified by generation.

## Explicit operator build and local runtime verification

Only after all three inventories pass, replace the capitalized placeholders
with your chosen protected directory paths and local image tag. Pass paths as
literal arguments; quote each whole argument if it contains spaces or Unicode.
This single-line example is suitable for a shell or PowerShell after literal
replacement; it contains no app-controlled shell text:

```text
docker buildx build --platform linux/amd64 --build-context "app-src=APP_EXPORT" --build-context "yetlet-src=FRAMEWORK_EXPORT" --file "DIR/Dockerfile" --tag "LOCAL_IMAGE:TAG" --load "DIR"
```

This is a separate operator action. BuildKit named contexts and Dockerfile syntax
1.7 support are required; record actual Docker/buildx/BuildKit versions when
verifying. The Dockerfile first runs `cargo metadata --locked`, then
`cargo build --locked --release`. These may download declared registry crates;
this is **not an offline build**. Base images/frontend may also require network.
No credentials, SSH forwarding, secrets, Cargo config, registry tokens, arbitrary
build arguments or caches are baked into this configuration. Check the source
Cargo.lock hash after the build and retain actual build/context-transfer logs.

The builder is `rust:1.99.0-bookworm` (Rust GNU/glibc) and runtime is
`debian:bookworm-slim` (Debian 12 glibc), both explicitly Linux/amd64. Tags specify
versions but remain mutable; image availability, downloaded digests and runtime
compatibility must be checked in the separate disposable Docker gate. For an
immutable build, the operator must review and pin verified image/frontend
SHA-256 digests in a separately reviewed configuration. Generation does not
resolve registry tags or assert that a base image was downloaded or tested.
Windows generation does not build Windows containers or cross-compile locally.

The runtime uses numeric UID/GID 65532, listens explicitly on
`YETLET_HOST=0.0.0.0`, `YETLET_PORT=8080`, and starts the binary in exec form.
`STOPSIGNAL SIGTERM` uses the app's accepted readiness/drain contract; arbitrary
long requests can extend graceful shutdown. Choose an appropriate stop timeout.
No HEALTHCHECK is emitted: no curl/wget or probe binary is installed/assumed.
Verify `/health/live`, `/health/ready`, SSR, CSP and HTML escaping externally using
a bounded disposable container with localhost-only host publishing. Confirm
SIGTERM drains an in-flight request and removes the listener/container. No TLS,
HA, external-dependency readiness, Compose, Kubernetes, cluster operations, image
push or production qualification is implied by these files.

Docker references: [build contexts](https://docs.docker.com/build/concepts/context/)
and [Dockerfile](https://docs.docker.com/reference/dockerfile/).
