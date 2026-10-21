# YT-096 two-host candidate archive rebuild

Date: 2026-10-20. Candidate remains
`ce56e0814e5423782440acd2c959ca777d572cd4`, tree
`0c5fa71179741f7b52dead078ff6d8a9efc30241`, original nine experimental,
unpublished 0.1.0 libraries; Rust MSRV 1.99 and qualification pin 1.99.0.

**PASS at the two-host source-archive comparison scope:** Windows MSVC rebuilt
and Cargo-verified all nine packages. Every returned `.crate` is byte-identical
to its accepted Linux original. Independent ordinary slice review PASS has
zero issues or required followups.
This does not satisfy independently controlled trusted rebuilding, compiled
binary reproducibility or final release acceptance. YT-096 remains In progress;
YT-020 remains Blocked.

## Inputs and execution

The [original package preflight](YT-096-CANDIDATE-QUALIFICATION.md) records the
first build on Ubuntu 26.04.1 GNU, Rust/Cargo 1.99.0, using the selected Git
archive and a fresh target with existing cached dependencies. Those artifacts,
their successful verification and all earlier [platform results](YT-096-PLATFORM-QUALIFICATION.md)
were reused without rerunning their test suites.

The existing WebCodex MSVC Project is
`agent:ncut-bohe-244143825ae3488e:localtasks`, Session
`wc_sess_y8CJQjwRbo4kknP1`. The actual host is Windows 11 x64 build 26300,
with Visual Studio 2022 Professional and Rust/Cargo 1.99.0 MSVC. The first
bootstrap briefly reported runner_unavailable; a focused health observation
then reported online/connected with zero active Jobs. The same session resumed;
no host replacement, service installation or global setting change occurred.

The owned root is `C:\repos\localtasks\yt096-platform-20261020\rebuild`.
Cargo uses the existing source export under its parent, whose Git archive
SHA-256 is
`9096cabe9a7b6d8b809c5bd337c2a8ab88f33ff19b6b4c8b59f31267c7335546`.
All 63 selected source input hashes match, including the existing 18 materialized
license aliases. Cargo/rustc/rustdoc binary hashes match the earlier MSVC
qualification. No source overlay, manifest, lockfile, dependency or toolchain
change was made.

A new private Cargo home and package target were created. Standard Cargo
directory-source replacement uses the existing 331-directory vendor staging.
It includes inactive-target resolution inputs and the prepublication fixtures;
it is not the release dependency inventory or an independent cache. The source
workspace is used because this operation rebuilds its packages; the previous
standalone test lane remains separate.

The initial ordinary Cargo package command exited 101 before producing any
archive: a directory replacement source requires explicit publication registry
selection. Its log requested `--registry crates-io`. One bounded command
correction supplied that option, reusing the same private home/target after
the pre-package refusal. The original result/log/script and ZIP are retained.
This does not enable network access: `--locked --offline` and the directory
source remain in force. No package was uploaded and no registry rights were
tested.

The successful command was:

```sh
cargo.exe package --locked --offline --registry crates-io \
  --manifest-path <candidate-source>/Cargo.toml --target-dir <owned-package-target> \
  -p yetlet-core -p yetlet-actions -p yetlet-reactive -p yetlet-html \
  -p yetlet-dom -p yetlet-dom-keyed -p yetlet-server-axum \
  -p yetlet-request-id -p yetlet-session
```

No feature or profile override, --no-verify or package-content workaround was
used. This is normal Cargo package verification, not a rerun of the native test
suites. Source inputs and the workspace lock remain unchanged after execution.
Native Job `wc_job_lqv0xpcrmBaguR95` is terminal, exit 0. The preceding failure
was a synchronous native exit 101 with no promoted Job. Native executable
commands do not have an agent model or reasoning effort.

## Archive comparison

The Windows evidence transfer contains the actual nine newly rebuilt archives.
Local comparison checked their sizes, SHA-256 values and complete bytes against
the retained Linux originals. This is not a rehash of the old Windows-consumed
archives. Cargo's raw log records nine packaging and nine verification steps.

| Archive | Bytes | Linux and rebuilt Windows SHA-256 |
|---|---:|---|
| yetlet-core-0.1.0.crate | 9103 | 664505f181f28e502ae02cdd7cb757044780c3059399cbc73863417082cd184e |
| yetlet-actions-0.1.0.crate | 11580 | dbd7f3d37ae977bdc03537bbfaa569040c8d9eb2ae9288f3d258b209c9ea1cd0 |
| yetlet-reactive-0.1.0.crate | 14487 | 652d881a49c7bbfa260675c0d22d6249abadb559d958f35b0c8c0cded6b6cdd3 |
| yetlet-html-0.1.0.crate | 8673 | 09d306cf4e9b2cc85811e215e10bdbbfeb874db9e3a770678b19703d5a61cb23 |
| yetlet-dom-0.1.0.crate | 9651 | 22826cb54e7ec3ae41a83240a76877d11e425c5da8894c6df0ca4172fb045668 |
| yetlet-dom-keyed-0.1.0.crate | 11561 | 1bc2c891286713c0552e827e0092213252c57099d3e4aa886cf487a090d4a05b |
| yetlet-server-axum-0.1.0.crate | 25173 | bdfe803d8e4a67b94bbf6ee3a237a972740cbaf8ec0cbcff3383ecfb952ab380 |
| yetlet-request-id-0.1.0.crate | 13274 | 3d0d547ef1d93f5e492b0e2975ce26ff3a06ddc710a5923b91eae48fdb958f60 |
| yetlet-session-0.1.0.crate | 18520 | 927525c423dcf77f9046f648c7de02d82f802c0979cac396dd225a67e1274534 |

## Evidence, cleanup and remaining gates

[New closeout](../artifacts/yt096-two-host-rebuild-20261020/CLOSEOUT.md) links
the initial and successful raw logs, full argv/environment, tool/input hashes,
terminal observations, returned archives and local comparison. The successful
evidence ZIP SHA-256 is
`0b297393631cfe21377fd6fc14e5c3d2bec22b8c1a9d80aa8e5bab1f276a3b90`;
its 15 extracted members match the transfer. The initial ZIP has six members
and SHA-256 `db21c1edb1a31301e766f3f15c709bd2f9400f6869faa35838a2b0c19deec8de`.
All 462 preexisting files in the three earlier qualification evidence roots
remain byte-identical. No historical closeout/review record was overwritten.

Both packaging invocations are terminal and started no persistent application
or listener. Owned source, cache, package target and evidence remain for
continuation. There was no detached worktree, staging, commit, push, signing,
tagging, publication, hosted workflow activation or deployment.

The two-host archive-comparison subgate now passes. Both hosts remain under
shared maintainer/agent control and reuse the staged dependency sources. This
cannot satisfy the [independently controlled builder requirement](RELEASE-SUPPLY-CHAIN.md#independent-rebuild-provenance-and-approvals).
The selected Ubuntu shared-kernel/Chrome evidence boundaries, build-bound
inventory and SBOM omissions, formal provenance/trust/signature/registry checks,
final independent release/security review and final maintainer approval remain
open. No existing waiver, support classification, signature or unsigned bundle
is expanded or reassigned by this result.

## Independent ordinary slice review

Reviewer `/root/yt096_two_host_rebuild_review_oct20` independently returned
PASS with zero issues, required followups or optional notes. Native trace
matched the registered reviewer role, gpt-6-astra model and effective high
effort; the shared reasoning resolver observation is enforced. The reviewer
checked full archive bytes and every tar member against the accepted inventory,
both ZIPs and their 21 members, all 462 prior evidence files, 48 review snapshot
entries, 63 source identities, tool hashes, raw packaging/verification logs,
terminal native results, preservation and the scoped documentation. No build
or test was repeated. The [acceptance record](../artifacts/yt096-two-host-rebuild-20261020/REVIEW-ACCEPTANCE.json)
accepts this bounded slice and leaves all stated release gates open.
