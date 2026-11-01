# CLI0.2.0 release preparation — YT-229

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

## Current checkpoint

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
