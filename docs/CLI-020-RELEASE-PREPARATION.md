# CLI0.2.0 release preparation — YT-229

**Current state (2026-11-02): YT-229 Done; owner-approved publication PASS.**
The original preparation acceptance is preserved. Publisher41319625709/attempt1
completes GitHub/crates.io publication and the actual registry-installed CLI
checks. Anonymous public-asset, registry-byte and index readbacks also PASS.
Dated earlier checkpoints below retain their original Pending/failure labels;
the final section records the completed publication. SDKalpha.6 is not republished.

## Scope and authority

The owner accepts the next-step recommendation on 2026-11-01: commit the three
SDKalpha.6 publication closeout documents, then prepare independent CLI0.2.0
through the final owner publication gate. The closeout commit is0275291.
[SDKalpha.6 is already published](ALPHA6-RELEASE-PREPARATION.md#owner-approved-publication-and-registry-consumer--2026-11-01);
this item does not rebuild or republish either SDK package. The accepted
[YT-227 plan](YT-227-NEXT-RELEASE-PLAN.md#g2-bind-each-cli-version-to-its-actual-published-sdk)
supplies the version-specific binding requirement.

Exactly one distribution subject is selected: yetlet-cli0.2.0, executable yetlet,
schema3/cohort cli, prospective unsigned tag yetlet-cli-v0.2.0. Candidate C and
distinct signer S are Pending until reviewed source delivery and actual required
CI pass. The owner authorizes necessary strict/builtin Git delivery, PR/CI/Merge,
tag and one execution of each existing build/readback/prepare/sign/result stage,
stopping at release-publish for the owner's approval. Preserve failures; no
automatic failed-job retry, repeated signing/upload or final publication.

## Minimal contract

- Set the independent CLI version to0.2.0 and its registry SDK pin to
  =0.9.0-alpha.6, with no SDK path override or third-party dependency upgrade.
- Refresh the package-owned framework source/workspace/lock and existing browser
  startup templates using the canonical resource-sync script. Generated native
  and browser manifests select the same published SDK version.
- Keep a trusted version-specific release binding: CLI0.1.0/0.1.1 use SDKalpha.5;
  CLI0.2.0 uses SDKalpha.6. Source, archive metadata and installed-consumer checks
  share that binding. Unknown CLI bindings fail closed; schema3 gains no arbitrary
  SDK input. Historical releases and records keep their original meaning.
- Include the already accepted YT-223 static-build command. Preserve normal
  build/run/watch, finite protocol1, protected create-only output, private receipts,
  strict CSP, escaping, optional islands, cancellation and cleanup semantics.
  Do not add a static scaffold command, Markdown generator or new runtime API.
- Keep published installation instructions accurate: CLI0.1.1 remains the current
  registry installation until0.2.0 actually publishes. Development0.2 source and
  static-build availability must be labeled separately. The live guide remains
  its reviewed SDKalpha.5/CLI0.1.1 edition; no automatic edition promotion.

## Affected qualification and reuse

Only changed behavior/inputs receive new qualification: trusted old/new SDK
bindings and rejection, resource correspondence, lock/dependency closure,
normal CLI archive, actual installed archive binary version/help/scaffolding,
registry-SDKalpha.6 generated-project compile and one explicit static export.
Use Rust1.99 GNU and the prepared MSVC runner for affected CLI/resource/binary
checks. Reuse accepted SDK/browser/security/static-failure evidence at its exact
input closure; do not replay old suites merely to accumulate proof. Record
compile, binary execution and browser evidence separately. No new browser or
production/HA support claim follows.

Required PR/master CI runs on the new delivered source. Before marking Done,
independent Reviewer must accept the bounded source, actual missing-platform
evidence and final fixed output/retention/publication plan. The release retains
the existing single owner publication gate and token route; registry OIDC remains
deferred. Five public attachments are expected: one CLI archive, unsigned
provenance, public checksums, public Sigstore bundle and signing acceptance summary.
Private manifests, keys, capabilities and test evidence stay outside public assets.

## Initial implementation checkpoint — 2026-11-01

In progress. The selected version/pin,80-file qualified resource snapshot and
trusted old/new bindings are implemented; no SDK source or third-party entry
changes. Independent review, affected MSVC, immutable C/S and new release stages
remain Pending. Existing SDK, CLI0.1.x tags/releases/signatures and failure
receipts are unchanged.

### Local affected evidence — 2026-11-01

- Python source/registry/publication checks:22 passed, including old/new bindings,
  wrong SDK rejection and closed publication subjects; changed consumer entry
  subsequently passes2 focused tests, including unknown-version rejection before
  any install. These counts overlap by one test and are not summed as24 unique.
- Rust1.99 offline/locked GNU: browser1, real release-interface3, resources2,
  selected-scaffold1 PASS; two opt-in Vite/build tests remain ignored, not counted.
- Normal package verification PASS:142 files; archive-installed actual optimized
  binary reports exactly yetlet0.2.0, help includes static build, and new emits
  registry SDKalpha.6 without path/patch/replace. Its generated native app resolves
  and compiles offline/locked outside the repository.
- The copied standalone static consumer is minimally repinned to the two published
  SDKalpha.6 packages. Third-party lock entries stay identical. Actual installed
  CLI exports exactly six public files at /yetlet/, with escaped text, complete
  documents and CSP. No browser suite is replayed.
- CLI third-party lock entries remain identical; the SDK lock checksum matches
  the already accepted public archive. Resource/source correspondence, format,
  docs75-required/229-item and whitespace checks PASS.

Retain the initial offline cache misses, the new browser assertion's incorrect
root-versus-target dependency lookup, and the copied consumer's omitted two
compile-time frontend inputs. The assertion and copied input closure receive
one bounded correction each; corrected focused checks pass. These are local
setup/assertion failures, not claimed runtime regressions. Original logs and the
actual failed static-build private receipts remain intact. No hosted attempt,
release-stage retry or repeated previously accepted product suite occurs.

All local receipts and the review snapshot are retained outside the repository
under the owner-host evidence run yt229-cli020-20261101. No formal age private key,
publisher token, listener or cloud resource is used for these checks.

### First delivered CI and prerequisite correction — 2026-11-01

PR49/source1ff8b7d contains the accepted local preparation and SDK closeout.
The first Stable job124864220610 in run41312030357 fails in xtask tests before
later SDK/Chrome stages: two current-source CLI custody/private-rebuild fixtures
still expect0.1.1. Their expected current CLI version/tag and fixture SDK pin are
corrected to0.2.0/alpha.6; runtime custody/preparation guards are unchanged.
The existing focused xtask cli_ filter passes4 tests,39 filtered out. Preserve
the original CI failure; no failed hosted job is retried. The source correction
requires independent review before strict/builtin delivery to the same PR.

The first prepared-runner MSVC invocation stops in openssl-sys before any tests:
the invocation omitted the existing documented OpenSSL environment. Reuse the
already provisioned task-protected OpenSSL installation, changing only child
process environment; no SDK feature/dependency change, installation or global
configuration. The failed prerequisite log remains retained; no test had passed
or executed at that checkpoint. Affected MSVC execution is still Pending.

### Owner-requested manifest-driven expectations — 2026-11-01

Before Merge, the owner asks to remove duplicated current-version literals rather
than merely replace0.1.1 with0.2.0. Current-source xtask custody/private fixtures
now derive the CLI version, tag and SDK pin from its manifest. The Python
current-source check and Rust native/browser/scaffold/Windows expectations also
read that manifest; actual binary version checks already use CARGO_PKG_VERSION.
Future manifest bumps do not require editing these current-source expectations.
The canonical Cargo pins and trusted historical CLI-to-SDK release bindings stay
explicit; historical payloads/records never silently follow development HEAD.

Only these changed expectations are checked again: two focused xtask tests, the
one changed Python current-source test, five GNU CLI tests (two other tests stay
ignored), and normal142-file packaging PASS. Earlier unchanged resource/runtime/
SDK/browser evidence remains retained. Run41312478151/1 passes both required
checks at f4cf9b8; the owner-requested expectation changes still need independent
review and required new-source CI before Merge. No old failed job is retried.
The earlier bounded MSVC success is retained with its exact old-source scope;
new Windows expectation code is not yet claimed executed.


## Accepted source and fixed candidate — 2026-11-01

Final source independent Reviewer PASS0 accepts the manifest-driven preparation
and exact-source Windows7-pass/1-ignored evidence at f8cc6be. Required PR CI
41313701348/1 passes; PR49 squash Merge produces C
7fb963a6195f6932f12a8b072f9bad05f85f4f4a, tree
0e0bbd762a777e2555130c4f569832cec60d78dd, identical to reviewed source. Both
required actual-master jobs in41314387731/1 PASS before C selection. Original
41312030357 failure and intermediate green41312478151 remain preserved.

Strict/builtin existing-remote tag Guard passes; the normal hook consumes its
one-time binding when pushing unsigned yetlet-cli-v0.2.0 at C. Remote readback
matches; canonical retirement archives the used binding without changing history.
The existing custody-verifier policy selects exactly this tag; worker/owner gates
and secrets are unchanged. One45-minute build41315130983/attempt1 is dispatched;
its outcome and original custody readback remain Pending at this checkpoint.
The generic release SOP now uses versioned records and manifest-driven test data.
Candidate SDK pin, CLI runtime/templates and third-party closure remain unchanged.
S/V/P and later signing/publication stages are not selected by this paragraph.


## Successful original build and custody — 2026-11-01

[Build41315130983/attempt1](https://github.com/bohewu/yetlet/actions/runs/41315130983)
PASS at fixed C/tag. Public ZIP artifact13086907079 SHA-256 is
894b561d0620212179dec89d253fdd589b11acca8674aa8768ecbb1e3103d617.
Encrypted private ZIP artifact13086852267 SHA-256 is
85eea4f18fcf1b48d5582642366912a040dbe65b670fa94df470b6c2478eacbd.
Both actual downloaded ZIP digests/flat contents match metadata; no local formal
key or decryption is used. The unsigned output has exactly one CLI subject:

| Public build file | Size | SHA-256 |
|---|---|---|
| yetlet-cli-0.2.0.crate | 490696 | d712d9395faa56aa15f7376fad06ac4f86897cbcb1fd910c278d5b2fd02958af |
| PROVENANCE-UNSIGNED.json | 5557 | 6b6c624c4a64d235d9de9450e9067104dc54dcb9d189097a0b58f718168a239c |

The existing hosted staging is not a Git checkout, so its archive contains141
files versus142 in the Git-aware local qualification archive. Every141 common
file is byte-identical; only local .cargo_vcs_info.json is absent. The unchanged
frozen provenance binds C/build. No archive is repacked or replaced.

[Original protected readback41315597052/attempt1](https://github.com/bohewu/yetlet/actions/runs/41315597052)
PASS: complete preparation, original exit0, decryption/inventory/private-manifest
checks true, recipient and C/tag/build bindings match. The safe receipt's existing
notice does not claim builder authentication, durable custody or release approval.
Artifact13087640406 ZIP SHA-256 is
99b3aec55a7a9bc0492b9eaf5d57f154fad303faf457a2e420c2f3716c8aaf03.
Original encrypted bytes, safe receipts, qualification/CI/reviews and failures
are retained outside the repository. A premature run-list observation happened
before download/dispatch completion; its empty result is retained, not a second
readback invocation. No failed release job is retried.

This documents successful product/custody preparation. The next docs-only signer
delivery must pass independent review, strict/builtin Guard and required actual
master CI; select its actual distinct S afterward. Preparation/signing/final
independent verification and owner publication gate remain Pending. Expected
public scope stays five files/one CLI archive, Experimental pre-release, not
Latest. SDK packages, existing releases/tags and the guide edition are preserved.


## Fixed signer and verified outputs — 2026-11-01

PR50 delivers five independently reviewed documents through strict/builtin Guard.
Required PR41316670266/1 and actual master41316894279/1 PASS in the existing
release-only lane. The reviewed tree709c30399c6749dfca0402de0f6aa5c3345ad90d
matches squash Merge S86e5da255a00ba19177c63f6dbab7d376526e421. Select this
actual distinct S as V/P too; C/tag and all product bytes remain fixed.
No runtime, manifest, lock, workflow, permission or key change follows.

| Stage | Actual first attempt | Result |
|---|---|---|
| Prepare/freeze | [41317421694/1](https://github.com/bohewu/yetlet/actions/runs/41317421694) | PASS;37 qualification entries plus closure and all six roles; fixed frozen ciphertext/receipt retained |
| Keyless sign | [41318176509/1](https://github.com/bohewu/yetlet/actions/runs/41318176509) | PASS; same-job verification, exact two frozen payloads and approved S |
| Protected independent result | [41318394905/1](https://github.com/bohewu/yetlet/actions/runs/41318394905) | PASS; one CLI subject and all54 final private manifest entries, both actual bundles, cleanup and record/source/identity bindings |

The original build/readback above and these three stages each use attempt1;
there is no failed release-job retry or repeated signing. Signing inputs are:

- Original provenance SHA-256:
  6b6c624c4a64d235d9de9450e9067104dc54dcb9d189097a0b58f718168a239c.
- Complete final private manifest SHA-256:
  fe3251088f5c250764e0d0b680428adbc84f83709b5d3040e31fe4d5540f462c.
- Public provenance bundle SHA-256:
  33a5ac5f65dd678f54b48c473992c73e44d9b95462197cb089979cc93d2e4980.
- Encrypted private manifest bundle SHA-256:
  9e6a06102b80aeafabc484f5472c4f56990a293693d7b01aa3f4ea27c29489e1.

Frozen artifact13087619508 ZIP SHA-256 is
276d92a5c4c0b3dfaf9aa449b6e57032bc11b403877b6d0bfd3efd86f88aea7d.
Actual signing artifact13087628812 ZIP SHA-256 is
dc479727eaf981acc5b1b0e2ea595ad3acbc684f863b12d893cc61f5b3083411.
Verified-publication artifact13088227669 ZIP SHA-256 is
dee31b500b4b3ebf5ae25dcb14ec12e9f5ef9f6a6f6290e0fb264a472e74bd9a.

Original encrypted bytes, intake/frozen/sign/result artifacts, full safe receipts,
source/qualification/review evidence and existing failures are retained outside
the repository. The owner-local retention wrapper initially assumed two intake
files; the existing canonical contract specifies three. Reuse the verified ZIP
and canonical unpacker once; preserve the wrapper failure, with no source change,
decryption, key access, new prepare or release retry. The temporary encrypted
Draft input is removed only after verified frozen/intake retention. Draft430642547
remains private and empty; no GitHub Release or registry publication has occurred.

The canonical publisher helper prepares exactly five prospective public files:
one CLI archive, original unsigned provenance, public checksums, public Sigstore
bundle and signing acceptance summary. Its preview acceptance is prospective;
it does not claim owner approval. Notes and registry metadata keep SDKalpha.6,
exact original CLI bytes, Experimental pre-release and not-Latest scope. No
private bundle, manifest entries, evidence filenames, keys or capabilities enter
the public set. GitHub promotion, registry upload and actual registry installation
are still unexecuted. Final independent review, publisher preflight and observed
owner gate remain Pending; YT-229 stays In progress at this checkpoint.
These three closing documents stay local/uncommitted to preserve fixed P.


## Owner publication gate ready — 2026-11-01

Final independent Reviewer PASS0 accepts three document hashes,28 retained
output hashes, the unchanged37-entry qualification closure and actual five-stage
results: one subject/all54 private entries. There are zero issues/followups;
registered reviewer/model and effective high effort are verified. Its explicit
conditional preparation acceptance is satisfied by actual
[publisher41319625709/attempt1](https://github.com/bohewu/yetlet/actions/runs/41319625709):
preflight SUCCESS, exact reviewed public/registry plans match hosted artifacts,
and publish is waiting on release-publish Environment25761919341. No owner
approval is performed by the assistant. Safe preflight status is preflight_pass;
GitHub and registry are not_attempted. Private Draft430642547 still has zero
assets. The approved job will upload/read back the five planned public files,
promote the Experimental pre-release without Latest, then publish the one CLI
archive and check its registry installation. That execution is not claimed here.

Preflight artifact13088801243 ZIP SHA-256 is
feda09b05f56c097a6492ea5240469e312df358bc42bb6bf9351ed687d658be0.
Its three safe plan/result files, waiting-job/deployment metadata and the current
private Draft response are retained in the same repository-external evidence run.
No keys, secrets or private evidence are public attachments; no SDK package is
republished. All original failures remain preserved, with no release-job retry.

YT-229 is Done only for the authorized preparation-through-gate scope. Actual
publication/registry-consumer monitoring follows the owner's approval. These
three closing documents remain uncommitted/unpushed so fixed P stays unchanged.

## Owner-approved publication and installed consumer — 2026-11-02

After the owner's release-publish approval,
[publisher41319625709/attempt1](https://github.com/bohewu/yetlet/actions/runs/41319625709)
completes successfully: GitHub first, then the exact CLI archive to crates.io,
then a fresh credential-free registry installation. The GitHub publication time
is2026-11-01T21:47:17Z (2026-11-02 in the owner's Asia/Taipei timezone).
The owner subsequently authorizes the three closing documents' strict/builtin
Commit/Push, PR and conditional Merge after successful CI. The actual default
branch remains master; this does not rename it to main.

The [CLI0.2.0 release](https://github.com/bohewu/yetlet/releases/tag/yetlet-cli-v0.2.0)
is an Experimental pre-release, not Latest. Release430642547 keeps the immutable
tag/C7fb963a and exactly the five reviewed public files: one CLI archive,
PROVENANCE-UNSIGNED.json, RELEASE-CHECKSUMS.json, provenance.sigstore.json and
SIGNING-APPROVAL.json. All names, sizes, SHA-256 digests and notes match the
frozen public plan. Anonymous downloads independently match each original digest.
No private evidence, keys, manifests or capability URLs are release attachments.

| Observed result | Evidence |
|---|---|
| GitHub and registry publication | Safe PUBLICATION-RESULT status complete; both published_and_verified; automatic_retry false |
| CLI archive identity | Original archive, GitHub attachment, crates.io index checksum and anonymous registry download all SHA-256 d712d9395faa56aa15f7376fad06ac4f86897cbcb1fd910c278d5b2fd02958af;490696 bytes; version is not yanked |
| Actual registry installation | Rust1.99 cargo install yetlet-cli --version0.2.0 --locked completes in a fresh Cargo home; version output exactly yetlet0.2.0, help and new each succeed |
| Generated project contract | Generated yetlet requirement is exactly =0.9.0-alpha.6; no path/patch/replace and no forwarded publication credentials |
| Installed source lock | REGISTRY-CONSUMER.lock SHA-256 95d2643786f74250b0aa348747a718a7a6c518e1f197028bee28ff52bff9b759 |
| Retained publication artifact | Artifact13101898819,9480-byte ZIP, SHA-256 016f0cfad6c5b78de05608f4636f0db742798fdc6ea10cee8547bad0fc08afd9;14 safe receipt/plan/consumer-log files |

The actual publisher's install/version/help/new logs and lock, owner approval,
run/jobs, plans and GitHub/registry readbacks are retained outside the repository
in the same yt229-cli020-20261101 evidence run, publication-monitor directory.
The monitoring reads existing execution proof; it does not reinstall the CLI,
rebuild the generated app or replay accepted GNU/MSVC/browser/static tests.
Generated-app compilation and static export retain their earlier qualified scope.
All original failures remain; no release-job retry, duplicate upload, resigning,
tag rewrite, SDK republishing, new key access or live guide-edition promotion
occurs. The three-file closeout changes only recorded status and evidence.
