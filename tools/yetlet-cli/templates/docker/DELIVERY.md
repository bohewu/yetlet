# Optional cloud delivery for @NAME@ (YT-101 preview)

Configuration generation only: no source export, Cargo/npm execution, image
build, network, publication or deployment. Linux/amd64 images; Linux/Windows
native generators. Supports the accepted generated service layouts with optional
runtime Settings and Vite assets. Current creation uses `--service`, `--settings`,
`--vite` and `--tailwind`; removed argv aliases are not supported. The selected
CLI profile emits its own guide. Standalone Tailwind and Vite dev bridge exports
are outside this delivery profile. Plain service generation retains its own recipe.

## Explicit operator preparation

1. Generate the app and its own Cargo.lock explicitly. For Vite, run npm ci,
   npm run typecheck and npm run build explicitly in that app. The CLI never
   executes them. Keep exact generated Rust/frontend/public TOML inputs;
   arbitrary edited sources, extra Cargo targets and dependencies are refused.
   Public settings may be overridden using typed runtime APP keys.
2. Freeze writers. Generate docker into a NEW protected directory DIR. Missing,
   stale or extra dist files are refused. Build timestamps must be at least as
   new as generated inputs; timestamps are a freshness check, not authenticated
   build provenance. Use only operator-owned, trusted, quiescent build trees.
3. Prepare three physically separate protected directories DIR, APP_EXPORT and
   FRAMEWORK_EXPORT, with no ancestor/descendant overlap. Never pass the live
   checkout/app as any Docker context and never recursively copy a source tree.
   This is an explicit operator action; there is no automatic export helper.
4. APP_EXPORT: copy only the app files enumerated under app-src in
   build-context-inventory.json, byte for byte. These are Cargo.lock, exact Rust
   sources, optional dist/.vite/manifest.json and its bounded JS/CSS/SVG files,
   and optional four public TOMLs. Copy DIR/app-src.dockerignore as
   APP_EXPORT/.dockerignore. Do not copy raw app Cargo.toml or frontend inputs.
5. FRAMEWORK_EXPORT: copy only enumerated yetlet-src crate sources/manifests.
   The closure is the complete compiled `crates/yetlet` member, including
   inventoried resources, guides, tests and examples. Settings selects `config`
   on the same SDK edge, not another crate or workspace. Copy
   DIR/framework-workspace.toml as FRAMEWORK_EXPORT/Cargo.toml and
   DIR/yetlet-src.dockerignore as FRAMEWORK_EXPORT/.dockerignore.
6. Physically enumerate ALL entries, including hidden entries, without following
   links. Only inventory file keys and their necessary parent directories may
   exist in each context. Reject any missing/extra file, traversal, symlink,
   junction/reparse point, special file, hardlink to mutable source or untrusted
   writable ownership/ancestry. Compare every byte count and SHA-256 with the
   inventory using trusted OS tools (sha256sum or Get-FileHash). Check DIR's
   generated_files_except_inventory as well. Named-context ignore processing is
   not a security boundary: physical allowlists are mandatory before upload.
   No live .env, credentials, node_modules, .git, target, source maps or arbitrary
   files may enter ANY context. An exactly inventoried SDK `.env.example` is
   synthetic package input, not permission to export live dotenv files.
   Abort and regenerate if any input changes.

Limits: flat dist/assets only, 128 KiB manifest, 128 nodes, 512 references,
256 distinct asset files, 1 MiB per native-read input, 16 MiB aggregate assets.
Manifest schemas, imports/dynamic imports, cycles, CSS types and all file paths
are checked. Only manifest-listed artifacts are selected; unrelated files
inside dist are rejected. Other root files (including dotenv) are never read
or copied. Public config token declarations must remain empty. The inventory
is deterministic, contains no developer root/environment values and is not a
signature or proof of frontend producer authenticity.

## Separate operator image build

After all three physical inventories pass, use literal selected paths:

```text
docker buildx build --platform linux/amd64 --build-context "app-src=APP_EXPORT" --build-context "yetlet-src=FRAMEWORK_EXPORT" --file "DIR/Dockerfile" --tag "LOCAL_IMAGE:TAG" --load "DIR"
```

Requires BuildKit named contexts / Dockerfile syntax 1.7. The Rust 1.99
bookworm builder runs cargo metadata --locked and cargo build --locked
--release --bin @NAME@. Registry dependencies/base images may require network;
this is not an offline Docker build. Review/pin actual base/frontend digests
separately; mutable tags are not reproducible provenance. The Debian bookworm
runtime contains only the selected Rust binary, built assets and public config
under separate absolute roots, with no Node runtime. It uses UID/GID 65532,
port 8080, SIGTERM and exec-form entrypoint. Verify actual SSR/CSP/assets/lazy
chunks/health and signal/drain/cleanup separately before claiming runtime
success. Compose/Kubernetes generation must select this same image. Kubernetes
keeps non-root/read-only rootfs and no service-account credential mount.

## Selected optional cloud delivery profile (YT-101)

Use the exact image built from this app's separately generated Docker inventory.
Vite requires an explicit operator `npm ci` and `npm run build` before generation;
there is no Node/Vite runtime. YETLET_ASSET_ROOT=/opt/yetlet/assets selects the
immutable bytes in that image. No frontend source, node_modules or source maps
belong in any exported context. No-assets profiles require no asset environment.

Config profiles require YETLET_CONFIG_ROOT=/opt/yetlet/config, Production and
APP_SERVER__HOST=0.0.0.0 / APP_SERVER__PORT=8080. Only exact generated public
base/development/staging/production TOMLs are delivered; override public values
through typed APP environment keys. The root selector uses the YETLET namespace
so it cannot collide with typed APP configuration keys. Without the selector,
ordinary local generated configuration retains its app-root default. Never
set YETLET_DEV_SECRETS or bake dotenv/credential files into an image.

Compose requires the operator's host APP_SERVICE__TOKEN using required
interpolation; it never reads a secret while generating files. Kubernetes
references only an already-existing Secret named PACKAGE-runtime with key
service-token (replace PACKAGE with the unchanged app package name). Select
that name/key deliberately before using the manifest; it creates no Secret.
The operator may review/edit this reference to an independently managed Secret.
Do not put token values into YAML, Dockerfiles, inventories, logs or command
history. No-config profiles require no token or configuration environment.

Assets retain the existing Cache-Control: no-store policy. An immutable image
and startup snapshot do not provide rolling asset retention. Keep old images
and all static/lazy files available for old HTML/JS for the full client/rollout
lifetime, or use a coordinated stop-and-replace preview with no surviving old
clients. This bounded generator does not implement a shared asset store,
immutable HTTP caching, rolling/HA safety or production qualification.
