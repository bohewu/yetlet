# Yetlet CLI 0.4.0 Experimental signing dossier

## Unsigned checkpoint, 2026-11-12

YT-259 selects one Experimental source package, `yetlet-cli 0.4.0`,
under release record schema 3, cohort `cli`. The build dependency and
embedded SDK snapshot use `0.9.0-alpha.7`. Generated applications select
their registry SDK independently. With the verified alpha.8 core archive
and compatible flags, Native and Native Style use `App::serve()`.
Alpha.7 uses the legacy listener. Settings, Vite, Service and Browser
keep their documented startup profiles. [Scaffold contract](YT-259-CLI-NEW-STARTUP-CONTRACT.md)

At this checkpoint CLI 0.4.0 is unsigned and unpublished. The published
SDK is `0.9.0-alpha.8` and the published CLI is `0.3.1`. The Reader
Edition selects that published pair. The original readback matched all
15 files and eight chapter routes at Pages deployment `7732200772`.
[SDK publication](ALPHA8-HOSTED-RECORD.md), [CLI publication](CLI-031-HOSTED-RECORD.md),
[Reader selection](../site/pages.toml)

## Immutable product candidate

| Binding | Observed value |
|---|---|
| Product source C | `ac03ed1c76c4867c13a47240b1fc93058eed0b65` |
| Product tree | `3e63dd24c7e0ebd6518e9b1685d24683e2560ce3` |
| Lightweight tag | [yetlet-cli-v0.4.0](https://github.com/bohewu/yetlet/tree/yetlet-cli-v0.4.0), with exact remote readback at C |
| Source PR | [PR #91](https://github.com/bohewu/yetlet/pull/91), reviewed head `23895d53d9205b798fe0b1cb3c4e790c0bafdf32` and successful exact-head Rust, Docs and CodeQL |
| Protected master CI at C | [Rust 43189534378](https://github.com/bohewu/yetlet/actions/runs/43189534378), [Docs 43189534339](https://github.com/bohewu/yetlet/actions/runs/43189534339), [CodeQL 43189534286](https://github.com/bohewu/yetlet/actions/runs/43189534286), all successful. The Docs deployment job was skipped. |
| Strict tag Guard | `strict` / `builtin` passed. The native pre-push hook consumed one-use binding `77ec6bf5754d991e75e34a2bbf429cdb7a52ff87970a18587ff6459f667b7791` with zero outgoing commits. |
| Reviewed source manifest | 18 paths match C. Manifest SHA-256 `601a8ccf59fe12ba463d7798ec2dfa478511163c8d3e321cd9a6a6276c123b28` |

Independent source review passed with zero issues or required followups.
The release documentation gate passed at C. The packaged compatibility
policy SHA-256 is
`08272fe5d144b470799e9e4407793dd7637b5d50e6621718b4173b4cac667689`.
Its M26 capability row binds the published alpha.8 `yetlet` archive SHA-256
`3e850fe4320f3511121b37cfa25bb2ce3ef422d5fe11ccbea59908687912fd59`.
[Candidate preparation](CLI-040-RELEASE-PREPARATION.md)

## Original unsigned build and readback

The [unsigned build 43196395769/1](https://github.com/bohewu/yetlet/actions/runs/43196395769/attempts/1)
and [isolated evidence readback 43197052677/1](https://github.com/bohewu/yetlet/actions/runs/43197052677/attempts/1)
passed at C on the selected tag. Readback used the existing
`release-evidence-verify` Environment `25525950662` and tag policy
`64604698` for the exact tag.

| Original Actions transport | Artifact ID | Bytes | SHA-256 |
|---|---:|---:|---|
| `unsigned-preparation-43196395769-1` | `13933606540` | 538250 | `98bb8738a17251d282d28a85f616f71d1a9bb4fbfe5dd2e3ef2849b7011fe0f8` |
| Encrypted private build evidence | `13933411700` | 56495312 | `e78a5f85e557c71b8e7a87e901165310e2801f465307d5935cf805c692bcbd55` |
| `evidence-readback-43197052677-1` | `13934045466` | 688 | `f3048a1d93fe5e6711c72badf6699df214129fc44e31f8b2c84234fc7425c78d` |

| Original unsigned file | Bytes | SHA-256 |
|---|---:|---|
| `yetlet-cli-0.4.0.crate` | 535525 | `f4c7244d23087b44a7e70260842f16e21162cb9da5c1f7bbe224e8c67417efa8` |
| `PROVENANCE-UNSIGNED.json` | 5557 | `72b080a4384925f75816ac9d69b68f0f5e3a342520a6a0ea32573c4a1651354b` |
| `RELEASE-CHECKSUMS.json` | 430 | `68e7a28f79337694af26a57e0b3b3361ee185335e27557908fa781664bef5077` |

Unsigned provenance records the exact package as its sole subject and binds C,
the tagged workflow and the original invocation. The hosted package matches
the source payload at C. Of its 163 files, 162 are byte-identical to the earlier
preparation archive, including the executable sources, packaged Cargo.lock
and SDK resources. The refreshed package README is byte-identical to C.
The original preparation archive was 535492 bytes with SHA-256
`5639c8b34f6aab810269031978d74550a8be2516109dedc84f3ff8336099c36e`.

The safe readback receipt is 878 bytes with SHA-256
`7101f5bf7cb9a4cc64b06b0118d80734091bde6415d82af9872d931778d8a982`.
It records successful decryption and complete inventory and private-manifest
verification with original exit 0. The ciphertext SHA-256 is
`81f20ff0254519de75c819532246d69ac01f7e5c20d98be18b87519896cb95b0`
and the inventory SHA-256 is
`31fc5acd897349125fb3eb92c9fc03fe5641802a723812cc2907ae8efd17e988`.
The unsigned checkpoint's independent review passed with zero issues or
required followups. The three raw ZIPs and original receipts are retained
in owner-controlled custody. The Actions ZIPs expire on 2026-11-19.

## Qualification scope

The private flat qualification selection contains 36 original records and a
schema-1 `QUALIFICATION-CLOSURE.json` bound to C. It covers qualification,
security, inventory, support, review and failures. The checksum manifest
covers 37 entries and has 6277 bytes with SHA-256
`f5f0dac068bd4aad5970addf70eef178f70a4c61af355805ec1aec4ebbf6f993`.
The source map, raw transport ZIPs and release notes are retained separately.
[Supplemental transport contract](YT-096-SIGNER-CONTRACT.md#explicit-supplemental-input-transport)

- Linux used the earlier local preparation archive for six generation variants
  and seven additional locked checks. Native Style passed HTTP, CSS and CSP.
  Browser Style passed Vite/Wasm and Chrome checks. Both preview listeners
  closed and the Chrome profile was removed. The source PR also used that
  archive to exercise the refreshed README procedure with exact alpha.8,
  verifying HTTP 200, CSP and preservation of the lock and receipt.
- Windows x64 MSVC used that same preparation archive for four locked
  consumer checks. Alpha.8 Native, Native Style and Service and alpha.7
  Native matched the expected registry SDK and startup profiles, and retained
  their locks. Earlier YT-259-D Windows Native Style and Service runtime
  results apply to the original source overlay and published SDK alpha.7.
- Security evidence includes the configured CodeQL runs at C, the retained
  CSP and browser observations and the published checksum checks for the
  selected SDKs. The selected original failure records retain the expected
  Linux offline-cache miss and the Windows shell, tool-path, workspace and
  console setup failures with their respective scopes. Successful later
  Windows consumer checks used a standalone install and private consoles.
- Support follows the Experimental capability map and the release readiness
  record. Installation of CLI 0.4.0 from crates.io and the fresh locked
  registry consumer are publication checks. [Capability map](CAPABILITIES.md),
  [Release readiness](RELEASE-READINESS.md)

## Signer selection and publication approval

1. Independently review this dossier, the release notes and the flat evidence
   selection. Deliver this documentation-only record through strict/builtin
   Guard and exact PR and protected master CI. Select the resulting master
   commit as S, distinct from C. Confirm the Reader pair and that the delivery
   skipped Pages deployment. Keep master at S through preparation, signing,
   independent verification and publication preflight.
2. Bind the unpublished Experimental Draft to C and use the existing age
   transport for the selected qualification evidence. Bind the actual Draft
   ID, S, original runs, build artifact and raw provenance digest and size in
   the complete release record. Prepare once at S without OIDC and retain the
   original frozen receipt and both raw payload hashes. Clean up temporary
   handoff assets after verified retention.
3. Sign once at S under `reviewed-automation` with those exact frozen
   inputs. Independently verify at V=S with the separate verifier, covering
   both actual signature bundles, the single CLI package subject and every
   frozen private-manifest entry. Retain the verifier's original publication
   artifact and receipt.
4. Bind the original verifier outputs and reviewed notes to the exact five
   public files and single `yetlet-cli 0.4.0` registry upload. Run the
   no-secret preflight at P=S=V and stop at the protected
   `release-publish` owner approval wait. [Publication procedure](RELEASE-PUBLICATION.md#each-version)

Signer S, the Draft ID and the freeze, signing, independent verification and
preflight results are pending at this checkpoint. After a later verified
publication, the release SOP requires the Reader Edition update and live Pages
readback. [Reader rollover](RELEASING.md#required-reader-update-after-every-sdk-or-cli-publication)

## Owner-approved publication and installed consumer — 2026-11-12

The owner-approved [publication run 43212574155/1](https://github.com/bohewu/yetlet/actions/runs/43212574155/attempts/1)
published [yetlet-cli-v0.4.0](https://github.com/bohewu/yetlet/releases/tag/yetlet-cli-v0.4.0)
as Experimental GitHub release `442869277` at `2026-11-12T12:15:31Z` from
product source C `ac03ed1c76c4867c13a47240b1fc93058eed0b65`. The publisher
verified GitHub and crates.io delivery. Anonymous readback matched all five
GitHub assets and the original single, non-yanked crates.io archive
(`yetlet-cli 0.4.0`, 535525 bytes, SHA-256
`f4c7244d23087b44a7e70260842f16e21162cb9da5c1f7bbe224e8c67417efa8`).

The hosted Linux consumer installed exact CLI `0.4.0` from crates.io and
passed six locked generation cases. The default online and offline-cache
cases selected SDK `0.9.0-alpha.8`. Exact SDK `0.9.0-alpha.7` covered
Native, Native Style, Browser and Browser Style. Native Style passed
HTTP and stylesheet checks. Browser Style passed the Vite/Wasm preview and
Chrome checks, with no console errors, failed responses or inline styles.
Both preview ports closed and the Chrome profile was removed.

A separate clean local crates.io install of CLI `0.4.0` generated and ran the
first app with exact SDK `0.9.0-alpha.8`, no path overrides and receipt
`startup_profile` set to `m26-app-serve`. At `http://127.0.0.1:7878`,
`/` and `/hello/Ada` returned HTTP 200 with the default strict CSP and
scriptless HTML. The generated `Cargo.lock` and `.yetlet/sdk-resolution.json`
were byte-identical after the run, and the listener closed.
