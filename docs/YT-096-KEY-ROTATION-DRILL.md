# YT-096 — Manual key rotation preparation and local drill contract

Date: 2026-10-28. **Contract first; implementation delivered; local drill PASS;
independent Reviewer acceptance PASS (zero issues and required followups).** D6 Pending; YT-096 In progress;
YT-020 Blocked. No commit or push is authorized.

## Scope and baseline

The thin manual [rotation tool](../scripts/release-evidence-rotation.py) implements
`prepare` and `drill` commands.
Baseline: `47936205fa4a9f05d4a22e41c08f76008b422081`.
The drill is Linux amd64/GNU only, with fresh disposable LOCAL synthetic test
identities and `complete_product_custody=false`. It must execute the existing
custody CLI using authenticated age; a mocked round trip is insufficient.

The existing [custody contract](YT-096-AGE-CUSTODY-INTEGRATION.md) records the
adopted scheme, authenticated tool bootstrap, historical local evidence and
pending hosted authority. The retained run `yt096-age-custody-20261027` contains
`generation-result.json` and `local-roundtrip-proof.json`; its authenticated
`tools` binaries may be reused without rerunning bootstrap. These are historical
results, not evidence that this rotation drill passed.

Do not read, copy or use the existing real owner identity. Do not change the
current checkout's recipient, pins or keys. No remote access, GitHub settings,
Secret installation, key upload, hosted execution or Git index/history/ref
mutations are in scope. Read-only source inspection/archive and isolated fixture
preparation are allowed. Never move immutable `v0.1.0`. No automatic schedule, general
keyring, new cryptographic algorithm/dependency or hash-bypass option is allowed.

## Preparation contract

`prepare` takes a repository root, exact expected HEAD and one new public
recipient. It compares operational bytes with Git objects at that commit and
emits only `rotation.patch` and `manifest.json`; there is no apply option or
identity input. Public recipient validation checks line shape only; obtain and
review a valid age recipient separately. Drill recipients came from age-keygen.

All four operational sites must agree:

| Site | Required update |
|---|---|
| [.github/release-evidence-recipient.txt](../.github/release-evidence-recipient.txt) | New public recipient, exact one-line bytes |
| [xtask custody constant](../xtask/src/release/custody.rs) `RECIPIENT_SHA` | SHA-256 of those exact public-file bytes |
| Same custody module, literal age encryption recipient | Same public recipient |
| [release.yml](../.github/workflows/release.yml) independent SHA check | Same public-file SHA-256 |

The preparation manifest binds the declared baseline, exact input file hashes,
expected old recipient/pins, new recipient/hash and resulting patch/file hashes.
Expected-old checks cover every site and reject missing, duplicated, inconsistent
or changed anchors before producing an applicable result. Do not infer success
from a syntactically valid recipient alone or silently patch another baseline.

Any supported application must recheck the entire expected-old binding, refuse
existing destinations and publish only a fully prepared new isolated snapshot.
Failure must leave no partial application or overwritten input/output. Never
apply to the current checkout. Keep the tool small; no general patch framework.

## Local drill contract

`drill` creates fresh disposable A and B identities using authenticated
`age-keygen`, private directories0700 and files0600. It records tool hashes and
the existing authentication evidence. Key bytes and plaintext diagnostics must
not appear in public reports/artifacts; raw diagnostics remain private. Avoid tracing.

Create isolated source snapshots from the bound baseline, then use preparation
to produce coherent A and B public bindings. Build the actual xtask CLI offline
with locked dependencies in those snapshots. Missing cached dependencies or
authenticated tools is a blocker, not permission to fetch or bootstrap again.
Record snapshot hashes, commands, exit status and original source/workflow/run/
attempt bindings. Synthetic bindings must be labeled as local fixture values.

Use an actual synthetic private assembly, supported extras and the full original
LOCAL checksum manifest. Preserve the original manifest bytes and hash before
capture, then compare every restored member using the existing full-manifest
verification path. Retain ciphertext, transport receipt and actual readback
receipt. No product build or release qualification is implied.

| Required case | Acceptance evidence |
|---|---|
| A capture and A verify | Actual CLI success and complete LOCAL manifest match |
| B capture and B verify | Actual CLI success and complete LOCAL manifest match |
| Wrong identity | Actual authenticated decryption/readback rejection |
| B verifier given authentic A transport and A identity | Rejection by new public pin; an old identity cannot bypass it |
| Retained A tooling reads A transport | Success using retained A public file, helper and original bindings/key |
| Rollback | Select retained A snapshot/tooling and perform another actual A capture/readback; full manifest matches |
| LOCAL TEST identity loss | Remove the active disposable test copy, demonstrate failure, restore from a separately held local TEST copy, then verify and match the preserved full original manifest |

Retain the A helper/binary, public file, original bindings, ciphertext and test
identity while A readback is needed. Merely retaining an old key does not make a
new-pin verifier accept old transport. The loss exercise proves only local test
copy recovery; it makes no claim about recovering a GitHub Secret or owner key.

## Actual commands and prerequisites

Use Linux amd64/GNU, Git, Cargo/Rust with cached locked xtask dependencies, and
Python supporting the tar `data` extraction filter. Provide the authenticated
age/age-keygen binaries and prior bootstrap PASS record; both hardcoded binary
digests and record hashes must match. No download or bootstrap is performed.
Variables are operator placeholders. `$REPO` must be the repository root with
HEAD equal to `$EXACT_HEAD`; its three operational files must match committed
bytes. Destinations must be new; the private drill directory must be outside it.

```sh
python3 scripts/release-evidence-rotation.py prepare \
  --source-tree "$REPO" --base "$EXACT_HEAD" \
  --recipient-public "$NEW_PUBLIC_RECIPIENT" --output "$NEW_PREPARATION_DIR"

python3 scripts/release-evidence-rotation.py drill \
  --source-tree "$REPO" --base "$EXACT_HEAD" \
  --age "$AUTHENTICATED_AGE" --age-keygen "$AUTHENTICATED_AGE_KEYGEN" \
  --authentication-evidence "$TOOL_AUTHENTICATION_RECORD" \
  --work-dir "$NEW_PRIVATE_LOCAL_DRILL_DIR"
```

Preparation prints `Public preparation created; patch not applied`; its manifest
contains exact old/new file, recipient and patch hashes plus a retention checklist.
Drill prints the new private `proof.json` location and retains snapshots, helpers,
test identities, original manifest, ciphertext/receipts and private diagnostics.
Handled refusals exit1 with `Rotation refused:`; invalid CLI usage exits2. A
started failed drill retains FAILED proof; inspect private diagnostics locally.

## Recorded implementation and local evidence

Run `yt096-key-rotation-local-20261028` retains
[generation-result.json](../.pipeline-output/yt096-key-rotation-local-20261028/handoffs/generation-result.json)
and [local-rotation-drill-proof.json](../.pipeline-output/yt096-key-rotation-local-20261028/handoffs/local-rotation-drill-proof.json).
Python syntax and nine focused tests PASS; public preparation and read-only
`git apply --check` PASS. Two offline locked A/B builds and 25 actual commands
cover every table case, including expected refusals without partial readback.
Every successful readback matches the complete original LOCAL manifest and members;
manifest SHA-256 is `c83de73f29daa8209ca4e449e6dcfb2120d1ecf0b751180eaa804e0754409fad`.
Receipts remain `FAILED_INCOMPLETE`/`complete=false`; drill PASS does not relabel them.

The first drill rejected baseline license symlinks before Cargo/custody execution.
That failure is retained; one material product repair allowed checked internal
source symlinks with Python's tar data filter. Corrected real drill PASS; zero
operational retries. Private-mode audit PASS: directories0700/files0600,
helpers0700 and checked internal source symlinks; no scope processes remain.
Checkout operational files are unchanged. Parent verified the owner identity
inode/mode/size/mtime/uid unchanged by stat only, without reading key bytes. LOCAL TEST-copy recovery
proves neither owner-key nor GitHub Secret recovery. A separate workflow-state
initialization failure is retained as an operational observation; no canonical
workflow run completion is claimed. Historical custody proof fields are unchanged.

Independent Reviewer acceptance PASS, zero issues and required followups.
Next: seek separate source selection, protected Environment/Secret setup and
bounded hosted verification authority. New changes remain uncommitted/unpushed. Current-host
LOCAL synthetic proof provides no platform qualification, sixteen-package/OpenSSL
production build, durable custody, production rotation or release approval.
D6 approval, signing and publication require separate authority and are not
unlocked automatically by any future hosted run; YT-020 remains Blocked.
