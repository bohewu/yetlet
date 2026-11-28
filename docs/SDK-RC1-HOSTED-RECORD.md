# Yetlet SDK 1.0.0-rc.1 — hosted evidence

The Experimental SDK release contains `yetlet` and `yetlet-browser` with
Rust 1.99 as the minimum supported toolchain. Tag `v1.0.0-rc.1` points to
product source `45ee6b3d7990ba106f03a74a5be7cbf90e6668f3` from
[PR #122](https://github.com/bohewu/yetlet/pull/122).
[Release readiness](RELEASE-READINESS.md#sdk-10-release-candidate-acceptance)
records the compatibility scope and the earlier source and package checks.

## Owner-approved publication and registry consumer — 2026-11-28

The [independent signature verification](https://github.com/bohewu/yetlet/actions/runs/46148143062)
and [publication run](https://github.com/bohewu/yetlet/actions/runs/46148263004)
passed. The [GitHub pre-release](https://github.com/bohewu/yetlet/releases/tag/v1.0.0-rc.1)
and crates.io delivered the same approved archives. The publication receipt
(`15264249081`) records Core upload and readback before Browser, both registry
index and download checksums, and a successful locked consumer of both exact
versions from crates.io. Both package versions are non-yanked.

| Package | SHA-256 |
|---|---|
| [yetlet 1.0.0-rc.1](https://crates.io/crates/yetlet/1.0.0-rc.1) | `72d9fbdcf7cf0c8a029ab2ac83f6c33759fb3b19d12586057311fced5740531a` |
| [yetlet-browser 1.0.0-rc.1](https://crates.io/crates/yetlet-browser/1.0.0-rc.1) | `53201337df415b80b717337988014c271326cf5c360dcd0672e8fb6a70d3419b` |

An Ubuntu x86-64 consumer used the independently published CLI `0.4.2` with
exact registry SDK pins and locks. Service HTTP, native SSR and both health
endpoints passed. Chrome loaded real Wasm and exercised keyed row reordering with
`moveBefore()` and the `insertBefore()` fallback, including CSP enforcement.
The [CLI publication record](CLI-042-HOSTED-RECORD.md) retains the CLI archive
and installation evidence. The Reader Edition closeout still requires reviewed
source delivery, CI and live Pages readback.
