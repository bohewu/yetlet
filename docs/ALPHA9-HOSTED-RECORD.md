# Yetlet SDK 0.9.0-alpha.9 — hosted evidence

## Immutable SDK source and scope

YT-296 covers the Experimental SDK cohort `yetlet` and `yetlet-browser` at
`0.9.0-alpha.9`, with Rust 1.99. The controlled publisher uploads Core
first and verifies its crates.io index and download before uploading Browser.
Browser declares an exact dependency on the matching Core version with
Core defaults disabled. [Release and migration notes](ALPHA9-RELEASE-PREPARATION.md).

| Identity | Result |
|---|---|
| Product source C | `29b2663671db59b530c01be17ff48120f5c445d8` |
| Lightweight tag | `v0.9.0-alpha.9` points to C |
| Source release preparation | [PR #103](https://github.com/bohewu/yetlet/pull/103) delivered the reviewed SDK inputs. The SDK manifests, locks, crate directories, release build workflow and xtask match the reviewed M28 source. |
| Protected master CI at C | [Rust](https://github.com/bohewu/yetlet/actions/runs/45093778854) and [CodeQL](https://github.com/bohewu/yetlet/actions/runs/45093778827) passed. Both required Rust jobs ran the approved tooling-only scope. |
| Full reviewed product CI | [Rust and Chrome](https://github.com/bohewu/yetlet/actions/runs/45028782147), [CodeQL](https://github.com/bohewu/yetlet/actions/runs/45028782136) and [documentation](https://github.com/bohewu/yetlet/actions/runs/45028782129) passed at `4625e0151b3e19976932a6a02c3d91ff9dba39ac`. |
| Tag and evidence policy | Strict/builtin native Commit Guard passed with zero outgoing commits using the owner-authorized one-use history baseline. The protected evidence environment's sole tag rule is `v0.9.0-alpha.9` and administrator bypass is disabled. |

The release source keeps Core defaults empty. View and HTML are separate
opt-ins. Native Web HTTP, windows, UDP, command ingress and GPU rendering are
optional. Browser defaults to DOM. [Capabilities](CAPABILITIES.md).

Pre-freeze qualification passed 229 source tests. The archives passed six
external consumer families and Windows MSVC cross-target compilation from Linux.
CLI 0.4.2 exercised those pre-freeze SDK archives with Linux Native and
Service and with live Chrome/Wasm. The exact scope and original failures
are retained in the [M28 preparation record](ALPHA9-RELEASE-PREPARATION.md).
Windows runtime, physical network and production qualification remain
outside the release evidence.

The two pre-freeze package lock audits reported zero vulnerabilities.
The Core graph has the scoped informational `rustix` warning and the
workspace's separately configured OIDC applicability exception remains
limited to the source-only application. See the [RustSec scope](ALPHA9-RELEASE-PREPARATION.md#rustsec-scope).

The independent CLI `0.4.2` is already available on crates.io. Its
[publication run](https://github.com/bohewu/yetlet/actions/runs/45093965886)
verified a public registry install and Native and Chrome/Wasm consumers on
published SDK alpha.8. A separate public installation also passed Service
runtime and both health endpoints. SDK signing and registry verification
are distinct. Pages remains paused.

## Original build and protected readback

| Identity | Exact retained result |
|---|---|
| [Unsigned SDK build](https://github.com/bohewu/yetlet/actions/runs/45095661132/attempts/1) | Run `45095661132`, attempt 1, success at C |
| Unsigned public preparation ZIP | Artifact `14783097466`, 379608 bytes, SHA-256 `4959a4058ed7a88a5a63bbbb7f8566ed6d1ecd6cac4d565cd10e3b3279531c63` |
| Encrypted private build ZIP | Artifact `14783411722`, 56433835 bytes, SHA-256 `baa63d2a0cb82b68b60567cac91dbf6013107f90d29a2634174441b75fba76c3` |
| Original unsigned provenance | 5732 bytes, SHA-256 `c76c97c77ddf266abb514080bcd8d96235cf493268a17ea287e40468640f73ea` |
| [Protected evidence readback](https://github.com/bohewu/yetlet/actions/runs/45096393887/attempts/1) | Run `45096393887`, attempt 1, preflight and verification succeeded at C |
| Reduced readback ZIP | Artifact `14783626198`, 688 bytes, SHA-256 `2c0e38b4d0ca8152e7ef852a83ebd968137b687c5bad20403cb0cf7c61423887` |
| Reduced readback receipt | 872 bytes, SHA-256 `2d59d2c6d9be4fb2b0f7ca2e1d8397fec9e467d3fbf10accb17b854f1a44f3b7` |

| SDK archive in upload order | Size in bytes | SHA-256 |
|---|---:|---|
| `yetlet-0.9.0-alpha.9.crate` | 323136 | `76731d596b60ad9caa42e40dc4b01dccb93958e67e2606370a52442927694b57` |
| `yetlet-browser-0.9.0-alpha.9.crate` | 53504 | `4fe13fdaa45323cdfe1221a70f0332f70a1bdb52d925fd294b62bd98c9a4d404` |

All 117 Core payload files match the qualified pre-freeze archive after
excluding local VCS metadata. The Browser payload matches its pre-freeze
archive except for the Core checksum in its packaged lock. The new checksum identifies the exact Core
archive built in the same run. Parsed Browser locks match after that single
checksum substitution. The original provenance binds both archive hashes and
the exact build source. The encrypted package and reduced readback receipt
agree on source, ciphertext and inventory. The protected verifier confirmed
decryption and full private manifest verification. Original transport bytes
remain in owner-controlled local custody.

## Signing and publication gate

Signing requires a distinct, reviewed master commit and successful CI.
The release-only readback change uses authenticated GitHub metadata and
anonymous canonical download URLs for the two SDK packages and their four
companion files. All six file sizes and hashes must match the owner-approved
publication plan
while the release is public, before the first registry upload.
Core upload and public byte readback precede Browser upload. The final exact
publication run requires the owner's approval in the protected
`release-publish` environment.
