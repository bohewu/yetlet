
## Standalone Tailwind

This selection writes `tailwind.css` and the app-owned Rust CSS adapter without
Node/package files. Separately provision Tailwind 4.3.3 CLI and explicitly build
`dist/tailwind.css` before running Rust. The adapter resolves this path against the app
manifest directory independently of runtime cwd. No compiler or dependency install runs during creation.
The Vite selection instead owns CSS through its official Tailwind plugin.
