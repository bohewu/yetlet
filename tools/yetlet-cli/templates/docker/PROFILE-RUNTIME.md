
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
