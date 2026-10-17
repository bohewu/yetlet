# YT-098 application-owned asset contract

Experimental and unpublished. This standalone Cargo workspace consumes actual
standalone Tailwind CSS and both existing Vite production profiles through one
application-owned neutral snapshot. No framework crate, ADR, CLI flag/template,
root dependency, browser runtime or VITE-SPIKE source is changed. See the
[contract](../../docs/ASSETS.md) and [verification](../../docs/YT-098-VERIFICATION.md).

Use the already provisioned, pinned tools; these commands install nothing:

```sh
# From the repository root, with the existing Vite node_modules/cache available:
(cd experiments/vite-integration && npm run typecheck && npm run build)
mkdir -p experiments/asset-contract/dist/standalone/assets
/tmp/yt091-tailwindcss-linux-x64-4.3.3 \
  --input experiments/asset-contract/standalone.css \
  --output experiments/asset-contract/dist/standalone/assets/standalone.css --minify
cargo test --offline --locked --manifest-path experiments/asset-contract/Cargo.toml
cargo build --offline --locked --manifest-path experiments/asset-contract/Cargo.toml
node experiments/asset-contract/verify-browser.mjs \
  --backend experiments/asset-contract/target/debug/yetlet-asset-contract \
  --dist "$PWD/experiments/vite-integration/dist" \
  --standalone "$PWD/experiments/asset-contract/dist/standalone" \
  --port 3048 --out /tmp/yt098-browser
```

The standalone binary's verified SHA-256 is
`dc61b3ac6b8c9ca874c0cc4c57b2409791a64c5540404ca5f5367360babc313a`.
It compiles CSS independently of Vite. Its input reuses the pinned package's
local theme/utilities CSS as build input data; this fixture does not execute
Node for standalone compilation, fetch packages, or claim that the input data
is bundled in the binary. Only `src/main.rs` is selected for class scanning.

The server also runs directly with those three argument pairs. Both asset roots
must be absolute and point to operator-owned frozen build trees. Runtime cwd is
irrelevant. Pages are `/plain`, `/tailwind`, and `/standalone`; the last has no
application script. Ctrl-C/SIGTERM stops the loopback server. The focused browser
test owns its server/Chrome and closes them. Local Ubuntu Chrome verification
and independent review passed; the verification record retains the earlier
socket-denied environment evidence and the later successful cleanup.
