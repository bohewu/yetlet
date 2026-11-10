# YT-245 — SDK alpha.7 local release preparation

Date: 2026-11-09. Status: **Original preparation and C/build/readback PASS; later owner-approved SDK alpha.7 publication and registry-only consumer PASS** (see [final release record](ALPHA7-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-09)).
Accepted baseline: `14e40614ab723a8a5b6f2de0cf85a8db762b331f`, branch
`feature/sdk-alpha7-preparation`. PR62 is squash merged; its reviewed tree
`d099f3e54ae427aadc00de257935371d768a5554` equals the original preparation
baseline tree. PR #63 delivered the alpha.7 preparation and was squash merged to
`master` as `1beb4d3cf2aea76b0cb29ca634a52e55bc55f088` (tree
`9e710eff0a7b1ce70b6b0778a354654fdabd16dc`). All PR exact-head checks and all
three post-merge master workflows passed. The Windows Runner pulled that exact
merged source and passed the accepted MSVC native SDK/tests/SSR and wasm32 compile
gate; its scope and unverified limits are recorded in
[Windows gate acceptance](../artifacts/verification/yt245-alpha7-preparation-20261108/WINDOWS-GATE-ACCEPTANCE.md).
At the original C/readback checkpoint, S/V/P and signature/publication were still pending. They were subsequently completed in the linked owner-approved release record; this preparation result retains its original bounded scope.
No mint-game or unrelated project was modified.
## Selected SDK and compatibility boundary

Exactly `yetlet`, then `yetlet-browser`, schema2/`consolidated`, locally selected
version `0.9.0-alpha.7`. Workspace and necessary same-checkout path dependencies
advance together, including inherited application package versions in independent
locks. Third-party versions, checksums and dependency selections are preserved.
Registry-only sample consumers, CLI resources/dependency/publication bindings and
`site/pages.toml` retain their existing versions. Current public quickstarts
continue SDKalpha.6/CLI0.2.0; alpha.7 package README registry recipes require
actual version availability, while reviewed local path/archive consumers can
exercise the selected source now.

This SDK includes the YT-234/235 typed Tag Builder API: 101 ordinary HTML
constructors, `TagElement<T>`, scoped attribute setters and typed void-child
restrictions over the same owned `Element`/`View`. Named setters upsert;
`.id`/`.class` replace and `.add_class` merges distinct tokens. Generic
`.attribute` retains append semantics and duplicate validation by the renderer.
HTML/DOM admission, per-mount URL/image permissions, defaults (`html` / `dom`),
native feature isolation, Rust 1.99 MSRV and Experimental API limits are unchanged.
No public Style crate, implicit styling, runtime CSS injection, renderer redesign
or production certification is included. Private app-owned Style generation and
Showcase remain separately scoped checkout/tooling surfaces.

## Independent CLI prerequisite

The intended later independent release is CLI `0.3.0` for additive
`native-style` / `browser-style` templates and existing Style build/watch support,
with registry SDK `=0.9.0-alpha.7`. This migration and package are **not implemented
or published** by YT-245. The CLI manifest stays `0.2.0`, its exact registry SDK
pin stays `=0.9.0-alpha.6`, and current publication bindings/resources stay intact.
Do not invoke `sync_resources.py` before the SDK is actually available. Normal
registry dependency preparation requires that publication; arbitrary registry
patches/overrides or fabricated CLI0.3 packages cannot substitute for it. This
preserves [YT-227 G2](YT-227-NEXT-RELEASE-PLAN.md#g2-bind-each-cli-version-to-its-actual-published-sdk)
and the SDK-first/CLI-later sequence in [alpha.6 preparation](ALPHA6-RELEASE-PREPARATION.md).

The SDK alpha.7 has since been owner-approved and published. The independent
CLI 0.3.0 Style release is now tracked under [YT-248](CLI-030-RELEASE-PREPARATION.md),
after [YT-247 Dynamic SDK Version Resolution](YT-247-SDK-VERSION-RESOLUTION-CONTRACT.md)
Phase B. Phase A delivers only the proposed contract/ADR and independent review;
this original SDK preparation remains historical and does not include that CLI release.
The later Owner-authorized [YT-248 candidate preparation](YT-248-CLI030-CANDIDATE.md)
qualifies the separate CLI archive and consumes the immutable registry SDK. It
does not republish SDK alpha.7 or upgrade the public Reader Edition. The later
[CLI publication and standalone consumer](CLI-030-HOSTED-RECORD.md) complete the
independent distribution while preserving the original preparation evidence.

## Evidence and local drafts

Retained evidence root: `artifacts/verification/yt245-alpha7-preparation-20261108/`.
The English preparation inputs `YT-245-SDK-ALPHA7-NOTES-DRAFT.md` and
`YT-245-CLI-030-SCOPE-DRAFT.md` are ignored local files, not public Drafts.
Before exact future notes are selected, bind actual accepted C, package hashes,
feature/target scope and new/reused evidence. No placeholder identity is an
accepted source or successful release.

The implementation handoff in that root records commands/exits, source hashes,
changed lock identities, exact local package/consumer checksums and cleanup.
Local package hashes establish only preparation artifacts, not future immutable-C
or hosted artifact identity. Normal packaging must verify Yetlet first and stage
its exact digest before Browser verification. No `--no-verify`, dirty-source
hosted guard bypass or new validation framework is permitted.

Reuse accepted YT-234/235 semantics and [YT-243](YT-243-RELEASE-QUALIFICATION.md)
Linux browser/CSP and Windows evidence only at identical relevant product source
hashes and their original declared scope. The accepted YT-245 Windows Runner gate
passed MSVC native SDK/tests/SSR and wasm32 compilation for the exact PR #63 merge;
that bounded result is not a Windows browser-runtime or GNU claim.
[YT-244](YT-244-PUBLICATION-SURFACE-API-AUDIT.md) retains package link/API audit
evidence. New version/docs/archive checks are fresh and separate; old signatures,
candidate PASS or registry receipts do not cover new bytes. Firefox390,
macOS/runtime, Safari/mobile, hosted Windows/OpenSSL image and other previously
unqualified claims are not promoted. No browser GUI replay or additional Windows
work is performed for alpha.7.

## Remaining release gates

1. Retain the accepted independent preparation review and original finding below.
   Later changes require source/evidence reconciliation and review of their
   affected scope; this local acceptance is not final release approval.
2. YT-245 Git delivery and its platform gate are complete: PR #63 merged to
   `master` as `1beb4d3cf2aea76b0cb29ca634a52e55bc55f088`; PR exact-head checks
   and all three post-merge master workflows passed. The Windows Runner pulled
   that exact source and passed the accepted MSVC native SDK/tests/SSR and wasm32
   compile gate (see the Windows gate acceptance above). Do not rerun this accepted
   YT-245 Windows verification. YT-246 PR #64 is also merged as `a9ec562`; all
   exact-head PR checks and all three post-merge master workflows passed. The final
   release-document gate at C passed with zero sync changes.
3. Immutable C `a9ec562eed5a0a08ab894a76a04fe859416a8ba1`, tree
   `7df26e82cd2fe00e7f92902dac8b5f834877012c`, and lightweight tag
   `v0.9.0-alpha.7` are selected. Hosted schema2/consolidated build run
   `42589716426/1` and protected original-evidence readback run `42591012250/1`
   passed; exact artifact/archive identities and the Environment tag-policy
   rollover are recorded in [the hosted record](ALPHA7-HOSTED-RECORD.md).
4. Select a reviewed signer commit S, distinct from C, via a normal PR with
   independent Reviewer, Strict Commit Guard, exact-head CI, Merge and post-merge
   master CI. V and P may equal S if the reviewed source and exact policy remain
   unchanged. Follow protected preparation/signing/independent verification, frozen
   payloads, complete private-manifest and two-subject verification, authenticated
   signatures and cleanup requirements. No historical signature is reusable.
5. Bind final English SDK notes to accepted C/evidence and derive the existing
   six-file SDK public allowlist and joined two-crate registry plan. Retain actual
   verified output hashes and the publication plan. Stop before the owner-protected
   publication workflow; GitHub/crates.io publication requires explicit owner approval.
6. Only after SDK availability, implement/review CLI0.3.0 manifest/pin/resources
   and current command inventory, installation/version expectations and generated
   native/browser Style consumers. Run its separate schema3/`cli` package,
   delivery, immutable source, hosted/signing/verifier and owner publication gates.
   Do not combine it into a three-package SDK cohort.
7. Any public guide-edition promotion remains a separate accepted consumer/source
   and Pages decision; it is not triggered by either version bump.

## Resumed local results — 2026-11-08

All fresh receipts use `resume-*` names in the retained evidence root. The
original release-doc PASS, 69-file pause snapshot, retained stash and terminal
stopped-job logs remain intact. Before this continuation, all 69 restored files
matched their original hashes; this continuation changes only this report's
baseline, ordering and actual results. The other 68 restored files remain
byte-identical. No version migration is replayed.

| Check | Actual local result |
|---|---|
| Source and lock reconciliation | Ten changed lock roots resolve with full `cargo metadata --locked --offline`; all third-party entries, including versions/checksums/dependency selections, are unchanged. Local changes are only alpha.6 to alpha.7 versions, including inherited application packages. CLI manifest/lock/resources and public guide inputs are unchanged |
| Release and guide metadata | Release-doc gate PASS; 118 existing regression tests run successfully with one existing skip, covering affected release/doc/public-guide metadata. Docs/DAG and 13-entry sample catalog PASS |
| Typed API and compile closure | Eight typed-tag unit tests, three passing doctests and four compile-fail doctests PASS. Both SDK defaults compile for wasm32 at alpha.7; native normal archive verification and external consumer also PASS |
| Normal two-package archives | Existing Rust `private::build_packages` invocation, rebuilt against selected metadata, verifies Yetlet before Browser with the exact staged predecessor digest. Both commands exit0; no verification bypass or hosted guard override |
| Archive readback | Actual package versions, features, Rust 1.99 requirement, licenses, READMEs, 13/4 carried guides and included source/tests/examples match the selected source. Third-party package-lock entries match retained YT-243 alpha.6 archives. Browser's archive lock binds the verified base digest |
| External typed-Tag consumer | Independent registry-shaped workspace selects both exact local archives through the existing directory-source/checksum recipe. Native execution and wasm32 compilation PASS, including escaping, input/textarea builders and shared Browser/base View identity; metadata and lock checksums bind both archives without checkout path dependencies |
| Reused behavior evidence | All 58 SDK source/test file hashes match accepted baseline and retained YT-243 qualified source. Its original behavior/browser/CSP and local Windows scope is retained separately; no browser or Windows execution is replayed |
| Cleanup and authority | Packaging subprocess records report cleanup success; owned Cargo/invocation sessions complete. No server/browser/watcher is started. Source remains unstaged, no active or uncertain local task execution remains, and no cache is deleted |

Current local preparation archives after the bounded Browser documentation correction
(not immutable-C, hosted or published artifacts):

| Archive | Bytes | SHA-256 |
|---|---:|---|
| `yetlet-0.9.0-alpha.7.crate` | 169595 | `7f63ac7ddd10dfba42704b54c2a2310b8251440cede638e51bbd5054abf6f1ea` |
| `yetlet-browser-0.9.0-alpha.7.crate` | 52559 | `5892542ba76e1d001903c3a745bfa70eb2a4cf756c70d7bf6cd9f0a110475154` |

The original independent review found one material P2: the Browser keyed-DOM
guide incorrectly claimed alpha.7 distribution. The corrected sentence describes
package contents without claiming publication. Fresh `review-correction-*`
receipts and `REVIEW-CORRECTION-HANDOFF.md` in the retained evidence root record
source-doc gates, normal packaging, archive readback and the exact-archive
native/WASM consumer, all exit0. Every non-generated archive member matches
current source; normalized manifests and package locks match the reviewed
archives. Yetlet remains byte-identical; Browser differs only at
`guides/dom-keyed.md`. Its corrected archive is 52559 bytes with the digest above.
The consumer metadata and lock bind both current digests; only Browser's checksum
changes in the consumer lock.

Original `resume-*`, source hashes and independent review receipts remain intact.
The original Browser archive (52565 bytes,
`e1277db626cdd4030549da99681a666e009d00e2b6b87a239e9b22b582d8efb8`) and its
guide/readback/consumer bindings are historical and superseded only at this
Browser documentation/archive boundary. The other reviewed facts, ten lock roots,
58 unchanged product-input hashes, source tests and original browser/CSP/platform
scope are reused unchanged. No general suite, browser GUI or Windows work is
replayed. Independent corrective Reviewer `wc_job_138KD3ElAAfaxxap` returned
PASS with zero issues and zero required followups after inspecting the corrected
source, actual archive and checksum-bound consumer. The parent accepts YT-245
only at bounded local SDK preparation scope; all 67 other reviewed paths and
58 product inputs remain unchanged.

The requested prepublication sequence is now complete: PR #63 exact-head CI
passed, the PR was squash merged as `1beb4d3cf2aea76b0cb29ca634a52e55bc55f088`,
all three post-merge master workflows passed, and the Windows Runner verified
that exact merged source at the bounded scope above. The durable CI and Windows
receipts are retained in the ignored verification directory.

The original hosted source/build/readback checkpoint is recorded in
[ALPHA7-HOSTED-RECORD.md](ALPHA7-HOSTED-RECORD.md). C and tag were immutable,
with both unsigned archives and encrypted evidence read back successfully.
At that checkpoint, S/V/P, signatures and independent result verification were
pending; those gates and the owner-approved GitHub/crates.io distribution are now
completed in [the final alpha.7 receipt](ALPHA7-HOSTED-RECORD.md#owner-approved-publication-and-registry-consumer--2026-11-09).
At the SDK publication checkpoint CLI 0.3.0 was a separate future release; it is
now [independently published and verified](CLI-030-HOSTED-RECORD.md). The public
Reader Edition continues to use SDK alpha.6 / CLI 0.2.0 until separately promoted.
