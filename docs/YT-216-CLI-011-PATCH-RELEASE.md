# YT-216 — CLI0.1.1 version-contract patch release

Date:2026-10-31. Status:In progress. Prepare through the final owner publication
gate after independent Reviewer PASS; no CLI0.1.1 upload is authorized yet.

## Trigger and smallest scope

[CLI0.1.0 publication41169732818/1](https://github.com/bohewu/yetlet/actions/runs/41169732818)
publishes GitHub/crates.io successfully and anonymously installs the executable,
but its exact version-output assertion fails on `yetlet 0.1.0 (unpublished preview)`.
Help/new checks are not reached. Keep that failed run, all partial receipts and
actual logs, published0.1.0 archive, tag, signatures and release unchanged. This
is a new patch version, not a retry or replacement of0.1.0.

Selected package:only `yetlet-cli`0.1.1, executable `yetlet`. SDK dependency stays
exactly `yetlet = "=0.9.0-alpha.5"`; no SDK upload, dependency upgrade or runtime/
generator redesign. Remove the stale unpublished suffix from `--version`/`-V`
and the same stale global help label. Version output is exactly `yetlet VERSION`
with one newline and the CLI manifest's version, independently of SDK version.

The real binary integration test executes both version flags, help and a minimal
new-project command with an isolated temporary parent. Verify the generated
SDK registry pin and absence of local path/patch/replace overrides. No mock
subprocess output substitutes for this binary check. Add the owning test to the
existing required native MSRV CI command list. No app compilation/server or old
runtime/platform matrix replay is needed for these label/metadata changes.

Only the CLI manifest and its own lock entry advance to0.1.1. Other lock entries
must remain identical. Adjust only helper tests that read the current CLI
manifest; historical0.1.0 record fixtures remain valid and unchanged.

## Existing release contract

Reuse closed schema3/cli and the existing [single-gate path](RELEASE-PUBLICATION.md):
new immutable `yetlet-cli-v0.1.1` at green merged C; distinct later green S, fixed
certificate identity/issuer/roots, age custody, protected separate no-OIDC result
verification and final owner-only `release-publish` gate. Keep all existing SDK
and CLI tags immutable. The exact original CLI archive becomes the sole registry
upload subject; the public set remains one crate plus the four existing reduced
verification files. GitHub Experimental pre-release first, then identical bytes
to crates.io after owner approval; not Latest, no prebuilt binary.

Owner authority covers necessary reviewed strict/builtin Git delivery through
PR/required CI/squash Merge, source/tag selection, bounded build/readback,
prepare/sign/independent verification and no-secret publication preflight. Stop
before final owner approval. Keep key/Secret custody and disabled admin bypass;
no new Secret, local age-private-key or crates.io-token access. Existing owner-
local GitHub authentication is used in memory only for the reviewed encrypted
ciphertext handoff. No credentials or capability URL reach logs or public assets.

Each new-version hosted stage has one attempt under existing bounds: build45m;
readback5/15m; prepare/sign5/30m each; result5/30m; preflight10m then owner-gated
publication60m. Keep previous version's consumed attempts. Preserve any failure
and stop; no automatic job, signing or publication retry.

## Reuse and missing proof

Reuse YT-214/YT-215 GNU runtime/generator and SDK/native/browser evidence only at
original source/input scopes. Published SDK/template/resource inputs are unchanged;
only CLI labels/self-version and its new interface check change. Retain original
HTTP evidence limits and all earlier failed receipts. New native source interface
and normal packaging proof do not claim fresh CLI MSVC/browser/GPU qualification.

The CLI dependency lock differs only in its own package version; reuse accepted
CLI61-entry audit/license/source evidence at its original advisory snapshot and
policy scope. The existing non-applicable RSA exception remains disclosed; no new
ignore or security-policy weakening. New archive/provenance/actual source custody,
fixed payloads, signatures, result and public/registry projection need new proof.
Do not treat an old signature or a green job with wrong receipt fields as proof.

Initial baseline:master `ed68b840dc445d9a95c03692f9b32c2e07c0cb6b`.
New C/S/tag/run/output identities remain unselected until their actual stages
succeed. Run-local evidence: `target/yt216-cli-011-release-20261031/`; encrypted
and long-term evidence stay outside the repository. Only complete actual results
may update the following execution record; final approval remains with the owner.

## Selected candidate and source delivery — 2026-10-31

Implementation independent Reviewer PASS0 accepts the frozen13-file boundary.
Real binary version/help checks PASS; one Linux scaffold fixture correction to
private0700 parent PASS without replaying the other two cases. Four affected
release identity/cohort/custody checks, one own-manifest Python check, normal
136-file package verification, docs/format/whitespace PASS. Only the CLI self-lock
entry changes; SDK/dependencies remain identical. Optional standalone CLI Clippy
retains three unrelated existing style diagnostics in unchanged files; no policy
suppression or unrelated source repair. Initial offline-cache and fixture failures
remain recorded. Normal local dirty-package proof is not the final archive identity.

Strict/builtin staged/index-tree and exact outgoing/native Guard PASS0 deliver
commit3303911 in [PR39](https://github.com/bohewu/yetlet/pull/39). Required PR
CI41171706263/1 passes both checks, including the real CLI binary command in native
MSRV. Squash-merged source C is `ae5dd4c55e1124f56fcc3b07a9fc9e5bf083b25f`, tree
`f269070be7da68f6355606f0b217f7e14fb4f144`, identical to the reviewed index tree.
The source branch is removed after external verified backup/tree correspondence.

Original master CI41172082238/1 fails the unchanged Lab SQLite concurrency test:
expected303/409, observed303/503; MSRV passes. Preserve its raw log and exact
failure, without app/test changes. The owner separately authorizes one failed-
Stable-only rerun. Attempt2 succeeds; MSRV retains its original execution timestamp.
Both checks therefore pass at exact C. That exceptional CI retry is consumed and
does not reset any old result or authorize another job retry.

New unsigned immutable `yetlet-cli-v0.1.1` is pushed/read back exactly at C through
strict/builtin accepted-remote-history/native Guard PASS. The consumed binding is
explicitly retired; old SDK/CLI tags and history remain untouched. Existing evidence
Environment advances only its single exact-tag policy to0.1.1, preserving its key
and protection rules. No new Secret or local private-key access.

One [build41173099066/1](https://github.com/bohewu/yetlet/actions/runs/41173099066)
is dispatched at this tag under the45minute bound. Actual archive/custody/payload/
signature/result identities remain Pending at this checkpoint. This narrow three-
document record delivery provides later S distinct from C; select only its actual
green merged SHA. It changes no CLI/builder/signer/SDK inputs. Continue the same
ledger, preserving the old0.1.0 publication/consumer failure and first master CI
failure. No0.1.1 publication or final owner approval is claimed.
