# Yetlet CLI 0.4.2 Experimental — hosted evidence

## Original signing preparation checkpoint — 2026-11-22

YT-295 selects the independent schema-3 `yetlet-cli` package at version
`0.4.2`. Its Build SDK is the exact published `yetlet =0.9.0-alpha.8` with
default features disabled and the existing embedded alpha.8 snapshot.
Generated applications select their SDK separately through `sdk-latest-v2`
and retain exact dependency pins, locked checksums and a resolution receipt.
The accepted public pair and Reader Edition at this checkpoint are SDK
`0.9.0-alpha.8` and CLI `0.4.1`. Signing, publication and installed registry
verification for CLI `0.4.2` remain pending.

## Immutable source and qualification

| Identity | Result |
|---|---|
| Product source C | `4625e0151b3e19976932a6a02c3d91ff9dba39ac` |
| Source tree | `40b832650c0fb03b9e56869d7c2dfe7fabad6a64` |
| Immutable lightweight Git tag | `yetlet-cli-v0.4.2` points to C |
| Reviewed source delivery | [PR #103](https://github.com/bohewu/yetlet/pull/103); exact merged tree matches the reviewed PR head |
| Protected master CI at C | [Rust/MSRV, Native, samples, Chrome and RustSec](https://github.com/bohewu/yetlet/actions/runs/45028782147), [CodeQL](https://github.com/bohewu/yetlet/actions/runs/45028782136), [documentation](https://github.com/bohewu/yetlet/actions/runs/45028782129) passed; Pages deploy was skipped |
| Commit Guard | The owner accepted C as the existing remote history base solely for this tag. The strict/builtin native pre-push hook passed with zero outgoing commits. The consumed binding and exact GitHub tag readback were retained. |

This patch ships the SDK-aware Native, Service and Browser templates from M28.
Native and Service detect the selected SDK API layout and can use `web,html`
with alpha.9. Browser selects the matching `yetlet-browser` version and
requests Core View explicitly where the selected SDK exposes it. SDK alpha.9
publication is tracked independently in YT-296.
[Template comparison and scope](ALPHA9-RELEASE-PREPARATION.md#cli-release-notes-and-compatibility).

- Linux CLI `0.4.2` installed from the tested local archive passed Native, Service
  and Browser checks against checksum-bound local alpha.9 SDK archives.
  Native and Service covered build, HTTP runtime and watch; Browser covered
  Chrome/Wasm interactions and watch reload. Public alpha.8 controls passed
  explicit and default generation for all three templates, plus locked fetch,
  build and runtime for Native and Service. The hosted CLI archive matches
  all 165 content files of that tested local CLI archive. The local archive
  also contains optional Cargo VCS metadata. The hosted provenance binds C.
- Source qualification retains the original CLI `0.4.1` alpha.9 preflight
  failures and the bounded fixture/setup corrections for the candidate tests.
  Independent source review found and then verified corrections to two
  documentation anchors. Neither correction changed the CLI package payload.
- The scoped CLI package audit recorded zero vulnerabilities and the RustSec
  informational warning `RUSTSEC-2026-0486` for `rustix 1.1.5` reached through
  `tempfile`. Scoped source review found no use of the affected xattr interfaces.
  See the [audit disposition](ALPHA9-RELEASE-PREPARATION.md#rustsec-scope).
- This qualification covers Linux CLI execution and normal-path Chrome/Wasm
  behavior. Windows CLI execution, deliberate inline-handler CSP denial, GPU
  presentation and production qualification are outside this release's evidence.

## Original build and independent readback

| Evidence | Exact result |
|---|---|
| [Unsigned build](https://github.com/bohewu/yetlet/actions/runs/45037632538/attempts/1) | Run `45037632538`, attempt 1, success at C |
| Public preparation ZIP | Artifact `14759514407`, 548632 bytes, SHA-256 `c740c29056e8e46db16d36578080dc1c7426e250ef08efb9e03c19ea80755e92` |
| Encrypted private transport ZIP | Artifact `14760770194`, 56505550 bytes, SHA-256 `a6dc37e446165f4a2d5d3b7e76f26e3b7e22ae67d88039404efa545a87a94494` |
| CLI archive | `yetlet-cli-0.4.2.crate`, 546579 bytes, SHA-256 `b7f196f5adfdf09063180270553e819035e67d888ba893d641fb3c2ad65ce515` |
| Original provenance | `PROVENANCE-UNSIGNED.json`, 5557 bytes, SHA-256 `0e83db6e3db502fbee2860b1cf18f7dd8459a4c8e69f9d1ba9379c40eba116ca` |
| Original checksums | `RELEASE-CHECKSUMS.json`, 430 bytes, SHA-256 `89a0e6ae7b05b1d548582a9a93ef9cc2496411c58c266f3fb0df514ccc1d45b0` |
| [Protected original evidence verification](https://github.com/bohewu/yetlet/actions/runs/45038788811/attempts/1) | Run `45038788811`, attempt 1, preflight and verify success at C; Environment `25525950662` |
| Safe readback ZIP | Artifact `14760413843`, 686 bytes, SHA-256 `72946d4a21abbb6ef60a57aebb8b25050303bd229ba82f77e53a632cbcbf643d` |
| Safe readback receipt | `yt096-safe-readback-receipt.json`, 878 bytes, SHA-256 `516bee11ff28216dd6f4cbc18c27618a15a69fc8274b2892b9ef2e1c10d4a3fa` |

The owner authorized updating the Environment's sole allowed tag in place
to `yetlet-cli-v0.4.2`. The rule ID and tag type were retained, the
Environment protection configuration matched the previous readback, and
administrator bypass remained disabled. The verifier reported
`PREPARATION_SUCCESS`, exit 0, complete inventory and private manifest
verification. Its receipt matches the original source/run, recipient,
ciphertext and inventory digests. The three original ZIPs and receipt were
downloaded and verified again in an owner-controlled local store.
GitHub's transport copies expire on 2026-11-29 UTC. Off-host retention has yet to be verified.

## Signing and publication gate

A later reviewed master commit S must pass exact-source CI before it can sign
the frozen CLI package evidence from C. The standing automation prepares the
two frozen payloads, signs them and independently verifies the signing result.
The owner then approves the exact `release-publish` run. The post-publication
check installs the CLI from crates.io and exercises generated apps against the
published SDK before SDK alpha.9 publication begins. Pages deployment remains
paused. [Publication procedure](RELEASE-PUBLICATION.md#each-version).

## Owner-approved publication and installed consumer — 2026-11-23

The [Experimental GitHub release](https://github.com/bohewu/yetlet/releases/tag/yetlet-cli-v0.4.2)
and crates.io serve the same approved CLI archive. The owner-approved
[continuation run](https://github.com/bohewu/yetlet/actions/runs/45093965886)
verified the published archive, installed it from crates.io and passed
Native and Browser consumer checks on Linux, including live Chrome/Wasm
on independently selected SDK alpha.8. A separate public CLI installation
passed the Service HTTP and health checks. The
[SDK alpha.9 public consumer](ALPHA9-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-23)
subsequently verified this same CLI with the new SDK. The CLI package build
continues to use its exact alpha.8 SDK dependency.
