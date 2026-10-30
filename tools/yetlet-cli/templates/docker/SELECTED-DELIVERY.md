# Container delivery for @NAME@ (experimental CLI profile)

This command writes configuration into a new protected directory. Prepare
dependencies, locks, frontend builds, physical source exports and images
explicitly. Generation does not run Cargo/npm/Docker, install tools, export
sources, publish an image or deploy anything.

Supported inputs are the current unedited `yetlet new --service`, optionally
`--settings`, or `--vite` with optional `--tailwind`. Vite implies settings.
Standalone Tailwind and arbitrary edited Rust/frontend/dependency layouts remain
outside this bounded profile. Generated registry version declarations are not a
claim that Yetlet is published: prepare explicit compatible absolute Cargo path
dependencies to one qualified local framework checkout, retaining the generated
versions, then prepare Cargo.lock separately. No framework-source CLI option or
custom source environment variable is needed.

## Prepare the application

1. Keep generated source/dependency features and public runtime TOMLs exact.
   Declared public CLI settings use the existing base/profile mechanism and
   strict validation. The Production public Vite root selects the production
   build inputs; valid Development/Staging roots may differ. Generation never
   starts the configured watcher or dev server and never builds the frontend,
   regardless of watch, auto_start or auto_build values.
2. For Vite, explicitly run npm ci, npm run typecheck and npm run build in the
   configured Production frontend root. Freeze writers before generation.
   Missing, stale or extra dist files fail. Timestamp checks indicate freshness,
   not authenticated build provenance. Use trusted, quiescent build trees.
3. Generate Docker configuration into a NEW protected directory DIR. Prepare
   two more physically separate protected directories APP_EXPORT and
   FRAMEWORK_EXPORT. None may be an ancestor or descendant of another. Never
   use the live application/framework checkout as a Docker context or recursively
   copy it into one.

## Export only inventoried bytes

`build-context-inventory.json` names the finite exported files and their byte
counts/SHA-256 values. For this current profile, `app-src-source-paths` maps each
non-synthesized app-src destination to its original project-relative source.
For example, exported `dist/assets/app.js` can map to
`web-ui/dist/assets/app.js`. Read the mapped original path, verify its bytes,
then copy it to the inventory's canonical destination. Do not assume that an
exported `dist/` exists at the application root. The map is explicit; the CLI
does not perform this export.

- APP_EXPORT contains only app-src inventory keys. Copy each mapped original
  file byte for byte. Copy DIR/app-src.dockerignore to APP_EXPORT/.dockerignore.
  Public CLI TOMLs may be inventoried even when the application has no runtime
  settings feature. Their presence does not select `yetlet/config` for that app.
- FRAMEWORK_EXPORT contains only yetlet-src inventory keys. Copy the named
  complete `crates/yetlet` source closure, including resources/guides/tests/examples,
  DIR/framework-workspace.toml as Cargo.toml, and DIR/yetlet-src.dockerignore as
  .dockerignore. The workspace has one SDK member; Settings selects its `config`
  feature. No separate Config workspace or Browser SDK is added to native delivery.
- DIR remains the default generated-configuration context. Check its
  generated_files_except_inventory hashes as well as default_context.

Physically enumerate all entries, including hidden ones, before upload. Permit
only inventory file keys and necessary parent directories. Reject missing/extra
files, traversal, links, junctions/reparse points, special files, hardlinks to
mutable sources and untrusted writable ownership/ancestry. Verify every byte
count and SHA-256 with trusted OS tools. Named-context ignore rules do not
replace these physical checks. No live dotenv, secrets, node_modules, .git, target,
frontend sources/dev scripts or source maps belong in an exported context.
An exactly inventoried SDK `.env.example` is synthetic package input only.
Abort and regenerate if any inspected input changes.

Asset limits retain the existing contract: flat assets/, 128 KiB manifest,
128 nodes, 512 references, 256 files, 1 MiB per native-read input and 16 MiB
aggregate. Only the checked complete JS/CSS/SVG graph is included. Public runtime
token declarations stay empty. The inventory is deterministic and contains no
developer absolute paths or process environment values; it is not a signature.

## Build and run the selected image separately

After all physical inventories pass:

```text
docker buildx build --platform linux/amd64 --build-context "app-src=APP_EXPORT" --build-context "yetlet-src=FRAMEWORK_EXPORT" --file "DIR/Dockerfile" --tag "LOCAL_IMAGE:TAG" --load "DIR"
```

BuildKit uses Rust 1.99 bookworm, cargo metadata --locked and cargo build --locked
--release. Base images and registry dependencies may need network; generation
does not qualify an offline container build or reproducible provenance.
The Debian runtime uses UID/GID 65532 and port 8080 with an exec entrypoint and
SIGTERM. It contains the compiled binary, selected built assets and optional
public runtime settings, with no Node runtime.

Vite images use explicit YETLET_ASSET_ROOT=/opt/yetlet/assets; that root supplies
the immutable startup graph independently of the build-time project path.
Settings-enabled images use YETLET_CONFIG_ROOT=/opt/yetlet/config, Production,
APP_SERVER__HOST=0.0.0.0 and APP_SERVER__PORT=8080. Supply APP_SERVICE__TOKEN only
at runtime. Compose uses required host interpolation; Kubernetes references the
existing Secret PACKAGE-runtime, key service-token, and creates no Secret.
Service-only apps need no runtime token or settings dependency. Never bake
YETLET_DEV_SECRETS, credentials or dotenv into an image.

Use this same image for Compose/Kubernetes. Verify actual SSR, CSP, asset bytes,
health, shutdown and cleanup separately from configuration generation. Assets
retain no-store; the default image contains one release's snapshot. Retention,
rolling availability and production support require a separately selected
deployment policy and qualification.
