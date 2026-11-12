# Yetlet SDK 0.9.0-alpha.8 signing dossier

## Owner-approved publication and registry consumer — 2026-11-12

Owner `bohewu` approved `release-publish` Environment `25761919341` for
[run 43177100178, attempt 1](https://github.com/bohewu/yetlet/actions/runs/43177100178).
The [Experimental GitHub pre-release](https://github.com/bohewu/yetlet/releases/tag/v0.9.0-alpha.8)
is Release ID `442617970`, published `2026-11-12T07:44:09Z` at immutable C.
The publisher staged six approved files and verified their public readback.
Independent anonymous readback matches every file's SHA-256 and size.

The signer, verifier and publisher source was
`ed08de5c02a943f862b8e36e41ceba0665957b5d`, distinct from C.
[PR #89](https://github.com/bohewu/yetlet/pull/89) passed the independent dossier
review, strict/builtin Guard and exact PR and post-merge CI. The no-OIDC
[freeze](https://github.com/bohewu/yetlet/actions/runs/43176020791),
[signing](https://github.com/bohewu/yetlet/actions/runs/43176662929) and
[independent verification](https://github.com/bohewu/yetlet/actions/runs/43176918278)
all passed on attempt 1. The verifier checked both package subjects and all 79
private manifest entries. The temporary encrypted Draft attachment was removed
before publication.

| Published package, in upload order | Bytes | SHA-256 |
|---|---:|---|
| `yetlet 0.9.0-alpha.8` | 173026 | `3e850fe4320f3511121b37cfa25bb2ce3ef422d5fe11ccbea59908687912fd59` |
| `yetlet-browser 0.9.0-alpha.8` | 52559 | `26c1a93d91d5ab574f213a13048a8cd6a680539042d3d3166d0a4de2c475983f` |

Both crates.io version entries are non-yanked. The version API, sparse index
and anonymous downloads match the original signed package identities. A fresh
registry-only consumer pinned both exact versions and passed `cargo +1.99.0
check --locked`, with no path patches or forwarded credentials. Its lock
SHA-256 is `857d88325b320f552ad009f3a7a720b93aaa4f0d17b95e17fd40062ae7d547b3`.
The original publication result artifact `13924811286` is retained with ZIP
SHA-256 `4c9b9396889163bdf5fb90ec19033ea0a1ad5502423c5a8a441e69dc7b0efb08`.
The independent CLI remains `0.3.1`. The Reader Edition rollover uses that
published CLI with SDK alpha.8 under the [release SOP](RELEASING.md#required-reader-update-after-every-sdk-or-cli-publication).

The Reader source selects SDK alpha.8 and CLI `0.3.1`. Both standalone guide
consumers were reconciled with Cargo, changing only the two SDK entries in each
lock. A registry-installed CLI `0.3.1` generated the exact alpha.8 first app.
The frozen build and run returned both HTTP routes, escaped HTML and strict
CSP. The app lock and resolution receipt were unchanged after shutdown. Local
native and Wasm verification produced the eight-chapter, fifteen-file Reader
preview and the separate ten-file static reference export. The public artifact
and live Pages delivery require readback at the deployed source.

## Historical signer preparation checkpoint — 2026-11-12

The hosted unsigned build and isolated readback of its original encrypted
evidence passed. At this checkpoint SDK `0.9.0-alpha.7` and CLI `0.3.1` were the
public distributions and live Reader Edition. The checkpoint recorded no
alpha.8 distribution and read back all eight Reader pages at Pages deployment
`7728960490`. [Current distribution](RELEASE-READINESS.md)

## Immutable product candidate

The release record selects schema 2, cohort `consolidated`, and exactly two
packages in dependency order, `yetlet` and `yetlet-browser`. Their defaults
remain `html` and `dom`, respectively. The CLI is independently released.

| Binding | Observed value |
|---|---|
| Product source C | `96255ce339ef7ea184d662a400dadffe476f6c6e` |
| Product tree | `406258a22ee881472b20662fb17809e53f37bab3` |
| Root Cargo.lock SHA-256 | `15e5c1618cc8f4277aacd09026d250e7a22e76ad0e19723673449b7b2de1b8c5` |
| Lightweight tag | `v0.9.0-alpha.8`, with remote readback at C |
| Strict tag Guard | `strict` / `builtin` PASS, native pre-push consumed the one-use binding `c29cb47382386d1f3c51d65416e3499c7d102e025baf81e9be52721140b1c7b0` |
| Source delivery | [PR #88](https://github.com/bohewu/yetlet/pull/88), exact reviewed head `0c30cb845f9e4893ce072b5d6c65fb3e2342dc66` and successful PR CI |
| Master CI at C | [Rust 43166885036](https://github.com/bohewu/yetlet/actions/runs/43166885036), [Docs 43166885139](https://github.com/bohewu/yetlet/actions/runs/43166885139), [CodeQL 43166885107](https://github.com/bohewu/yetlet/actions/runs/43166885107), all successful |

The independent source, archive and consumer review returned PASS with zero
issues or required followups. The retained source-manifest check at C matches
all 66 reviewed paths. The release-document gate also passed at C. The source
change introduces native `App<()>::serve()` on loopback plain HTTP at
`127.0.0.1:7878`, explicit `serve_on(address)` and the optional native
`yetlet::server_axum::prelude`. The application owns Tokio. Callers retain
explicit paths and `into_router()` for native listener, TLS and lifecycle
control. [Startup contract](adr/0011-application-startup-api.md),
[Prelude contract](adr/0012-server-axum-prelude.md)

## Original unsigned build and custody

The [unsigned build 43172046728/1](https://github.com/bohewu/yetlet/actions/runs/43172046728/attempts/1)
and [isolated original evidence readback 43173069184/1](https://github.com/bohewu/yetlet/actions/runs/43173069184/attempts/1)
completed successfully at C on the selected tag. The readback used
`release-evidence-verify` Environment ID `25525950662` with exact tag policy
ID `64604698`.

| Original Actions transport ZIP | Artifact ID | Bytes | SHA-256 |
|---|---:|---:|---|
| `unsigned-preparation-43172046728-1` | `13922238472` | 228506 | `7d7240afa17f240799a7490f16b4445f082d494660398a0559da15d8c2619ca4` |
| `encrypted-private-96255ce339ef7ea184d662a400dadffe476f6c6e-43172046728-1` | `13922038677` | 56259653 | `317cc1fb3b3d354b8578c6889831710dc6f7fcd5383b1ce51d8f69e18f3ab578` |
| `evidence-readback-43173069184-1` | `13923310419` | 685 | `0644a3861a87af9271b43892401e5715834f633b76b34e00794b9b48cf221408` |

| Original raw unsigned file | Bytes | SHA-256 |
|---|---:|---|
| `yetlet-0.9.0-alpha.8.crate` | 173026 | `3e850fe4320f3511121b37cfa25bb2ce3ef422d5fe11ccbea59908687912fd59` |
| `yetlet-browser-0.9.0-alpha.8.crate` | 52559 | `26c1a93d91d5ab574f213a13048a8cd6a680539042d3d3166d0a4de2c475983f` |
| `PROVENANCE-UNSIGNED.json` | 5732 | `fa3e9d7dab219d7d7f0d8e1509d3bcdcb3693cf7a2f2b06afba6160c83626fdf` |
| `RELEASE-CHECKSUMS.json` | 603 | `5b85b0a95bddfd524953075893802640084d82f6bb89b4d2b3a84ee68096249e` |

The two provenance subjects match the actual archives. The archives are
byte-identical to local preparation and their source payloads match C. The
browser package lock pins the base package to the original registry-format
`yetlet` archive SHA-256 above. Local checksum verification passed at unsigned
byte-integrity scope.

The original safe readback receipt is 872 bytes with SHA-256
`846afe8b8b8be6ded5744c627006ebb82f22e51d56bfcea2a06c05f955d32ce3`.
It records complete decryption and private-manifest/inventory verification
with original exit zero, ciphertext SHA-256
`1dd73fb368d0a921511bf347b0b74bf0f6fdf26af74d9051cf44ba63702f7739` and
inventory SHA-256
`1e859411a7e8ae0710ad8127369d9b5989e63c4437033aee9ffdebf5baf7daab`.
Its scope is isolated readback. The owner-side ignored unsigned verification
directory retains the original transport ZIPs, raw public files and receipt.
The hosted ZIPs expire on 2026-11-19. Long-term off-host retention and signing
authentication are separate gates.

## Scoped supplemental evidence

The owner-local flat qualification selection contains 60 retained originals
and a schema-1 `QUALIFICATION-CLOSURE.json` bound to C. The closure covers
qualification, security, inventory, support, review and failures. Its
`RELEASE-CHECKSUMS.json` covers 61 entries and is 10347 bytes with SHA-256
`1b4869621fdf0925bd8e2aa57a693b29367318b9d9fa535f675203043df48e84`.
The reviewable original-file mapping is retained alongside the flat directory.
The canonical Cargo xtask checksum create and verify both passed.
[Supplemental transport contract](YT-096-SIGNER-CONTRACT.md#explicit-supplemental-input-transport)

- The local exact-archive native consumer built and exercised both loopback
  startup paths, JSON and 404 responses, strict CSP and escaped HTML. The
  browser consumer compiled for `wasm32-unknown-unknown`. Those consumers used
  the original archives staged as local directory sources. Registry-only
  resolution and download are publication gates.
- Native Windows x64 MSVC evidence comes from merged M26
  `9d1075ab6442a9d8cb58606c170327aa32799bf7`. The original formal job
  `wc_job_vhSbJEX_GSFt-M-5` exited zero with 35 SDK Axum tests and 113 tests
  across 13 samples. Scoped Git source parity at C and SDK-version
  normalization for the affected manifests and root lock bound the reuse.
  This is Windows runtime evidence at M26 source scope.
- Ubuntu Chrome smoke and sample CSP checks come from accepted YT-257
  `caa3758c94b47d09f273cb3d3743398210c78da6` for Todo/Search, Multi-Island
  Dashboard and Style Showcase. The retained source comparison links the
  relevant examples and browser smoke source through M26 to C. These are
  historical sample checks. New alpha.8 verification covers the native
  exact-archive consumer and Wasm compile.
- Security evidence includes successful configured CodeQL at C, the retained
  CSP and escaping observations, and the reviewed locked dependency
  reconciliation. The original historical-document sync drift, four baseline
  lock failures, Windows wrapper stop, local tag Guard receipt projection
  mismatch and rejected custody tag-policy update remain in the selection.
  Each original record retains its disposition and scope.
- Package features, supported surfaces and maturity follow the existing
  capability and support records. This scope is Experimental. Stable API,
  production readiness and broader platform or security certification remain
  outside the release qualification. [Capability map](CAPABILITIES.md),
  [Release readiness](RELEASE-READINESS.md)

## Gates recorded at the historical checkpoint

1. Independently review this dossier, release notes and the flat original-file
   selection. Deliver the docs-only signer change through strict/builtin Guard,
   exact-head PR checks and post-merge master CI. Select the resulting protected
   master commit as S, distinct from C, and keep master at S through prepare
   and sign.
2. Create the unpublished Experimental prerelease Draft at C and bind the
   actual Release ID, S, both original runs, the build artifact and the raw
   provenance digest and size in the complete release record. Use the existing
   authenticated age transport for the selected flat qualification and the
   short-lived handoff. Bind actual retained transport IDs and hashes in the
   prepare request.
3. Run protected prepare at S with no OIDC and retain the original frozen
   artifact and receipt. Preparation fixes both signing inputs, the original
   raw provenance and the resulting full private manifest. Sign once at S
   under the standing `reviewed-automation` policy using those exact input
   hashes and the authenticated `release-sign.yml@refs/heads/master` identity.
4. Independently verify the actual signing results at V, including both package
   subjects, the signer identity and all frozen private-manifest entries. Keep
   the private evidence and private bundle under controlled custody.
5. Bind the actual verified public output, release notes and two-crate registry
   plan at P. Wait for the owner's protected `release-publish` Environment
   approval before publishing. Publication acceptance requires GitHub asset
   readback, exact registry archive readback in dependency order and a fresh
   locked registry-only consumer. Update the Reader Edition and verify live
   Pages after publication acceptance. [Publication procedure](RELEASE-PUBLICATION.md),
   [Reader rollover](RELEASING.md#required-reader-update-after-every-sdk-or-cli-publication)

S, V, P, the Draft ID and the resulting prepare, signature and publication
identities are pending at this unsigned checkpoint.
