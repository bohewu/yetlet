
## Frontend

`--vite` includes application settings and the complete production/development
Vite scaffold. Sources, package locks and frontend `.env*` live under
`frontend/`. Its checked-in `.env.example` contains public placeholders only.
Vite loads dotenv exclusively from that frontend root. Every imported VITE_*
value is public; never copy backend credentials into this directory.

Provision frontend tools explicitly, then prepare a production snapshot before
starting Rust, including development:

```sh
cd frontend
npm ci --ignore-scripts --no-audit --no-fund
npm run typecheck
npm run build
cd ..
yetlet build
yetlet run
```

`cli.vite.enabled`, `auto_start` and `auto_build` default to true. Run selects
only auto_start; build selects only auto_build. A skipped frontend build needs
separately prepared assets. No command implicitly installs dependencies. Set
auto_start=false for two foreground terminals: start Rust separately and run
`npm run dev` from frontend with backend credentials removed. The bridge accepts
only read-only `/` from loopback Rust port 3000; Vite uses strict loopback port
5173, optionally selected with YETLET_VITE_PORT. Stop both terminals explicitly.
No Rust watcher, general proxy, actions, authentication or production HMR exists.

The generated entry, shared module and lazy module accept development HMR.
The button counter and existing DOM survive module updates; entry replacement
stores the counter in Vite hot data, and shared replacement removes the old
listener before mounting the new handler. Lazy handler changes apply on the next
click. A click still awaiting its import when its mount is disposed is dropped.
Production keeps ordinary module loading without the development HMR hooks.

TypeScript is the default source format. Add your own JS/TS modules, including
subdirectories, beneath the frontend root and import them normally; no filename
registration is needed. JS/MJS, TS/MTS, CSS and SVG source requests stay confined
to that physical root. Dotfiles, tooling configs, build/dependency directories
and links outside the root are refused. The dev document still loads `app.ts`.
Vite owns dependency tracking and HMR; the CLI owns startup, shutdown and builds.

`cli.vite.root` selects a protected project-relative directory. When changing it,
move the entire frontend tree; the CLI subprocess cwd, Vite root/envDir and Rust
`dist` loading follow that root. Rust resolves it against the build-time app
manifest directory, independently of invocation cwd or relocated settings.
Runtime Node is unnecessary for production: Rust loads the bounded versioned
CSS/JS/SVG snapshot with strict production CSP before listening. Missing dist
fails before binding. Backend secrets stay outside the frontend child environment.

Tailwind, when selected, uses the official Vite plugin and requires no standalone
CSS compiler. CLI frontend children receive YETLET_PROJECT_ROOT as the selected
app directory so Rust source scanning follows custom nested frontend roots.
Direct npm from default frontend/ needs no extra value; when manually starting
npm from a relocated root, explicitly set YETLET_PROJECT_ROOT to the absolute
app directory. This is execution context, not another persistent settings file.
Pinned frontend versions remain experimental, not release proof.
