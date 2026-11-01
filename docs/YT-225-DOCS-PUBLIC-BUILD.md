# YT-225 — Public-mode guide artifact preparation

Date: 2026-11-01. Status: Done at preparation scope after independent Reviewer PASS0.
Source baseline: a3067149167997ed48cc831c05532be1131b75cf.

## Selected scope

The owner selects formal output and a build-only CI path for the accepted
[YT-224 reader](YT-224-DOCS-PILOT.md). Existing Pages remains the mdBook guide.
This item grants no Commit/Push, Merge, hosted execution or deployment authority.
It adds no SDK/runtime API, package version, dependency or CLI release.

The owner explicitly declines compatibility aliases for the early site's old
first-app.html and other mdBook URLs. Canonical routes remain /yetlet/ and the
seven /yetlet/chapter/ directory routes. Old bookmarks are allowed to stop working
after a separately authorized replacement; no redirect/alias layer is added.

## Public output contract

examples/docs-site/prepare.py gains explicit --public. It reuses the existing
guide builder's clean-checkout and reviewed-edition rules, captures the full
source commit, and keeps the same eight selections/excerpt/source hashes from
site/pages.toml. SDK0.9.0-alpha.5 and CLI0.1.1 remain the selected edition.
Dirty inputs or a workspace/edition mismatch fail before replacing generated
chapter input. Default local preparation remains a preview.

The native public shell links its abbreviated source label to the full commit
and removes only local-preview footer explanations. Experimental SDK/CLI labels,
canonical source attribution, readable HTML and the owned header search remain.
Public mode describes artifact content, not a claim that deployment occurred.

tools/build_yetlet_docs.py prepares selected inputs, runs the existing frozen
Wasm builder, copies a protected temporary consumer, and invokes the existing
source CLI static-export path. Tools and locked dependency caches must already
be prepared; the builder installs or downloads nothing. Fresh output is required.
It reuses the existing local-link/anchor/source-exclusion checker, checks the
exact15-file site inventory, and writes source/edition/input/output hashes and
private CLI receipts beside the site root. Those records are never site assets.
Public mode also rejects a changed source identity or dirty checkout at closeout.
The temporary consumer is removed; diagnostic receipts remain outside the site.

## Build-only CI contract

The new .github/workflows/docs-preview.yml is independent of the unchanged
live .github/workflows/docs.yml. PRs, relevant master pushes and manual dispatch
build an artifact. No job deploys, selects a GitHub Environment, uses Secrets,
or receives pages:write/id-token:write. Only contents:read is granted.

The workflow prepares Rust1.99.0/wasm32 and wasm-bindgen0.2.129 explicitly,
using the repository's existing full checkout/cache/upload action pins. It fetches
the two existing locked standalone dependency graphs and builds the source CLI;
published CLI0.1.1 lacks static mode. No dependency or tool version is upgraded.
The existing Node runtime executes the dependency-free bootstrap build script.
Timeout is30 minutes; actual hosted duration remains unmeasured.

Only the generated site root is uploaded as yetlet-guide-preview, including its
explicit .nojekyll. EXPORT-MANIFEST.json and STATIC-RECEIPTS are retained in a
separate build-record artifact. Retention is seven days. PR artifacts are
downloadable files, not public websites or authorization to replace Pages.

## Focused verification and reuse

- Six new Python tests PASS: public source/edition binding, dirty-source refusal,
  version mismatch, local preview preservation, exact output boundary and public
  disclosure. No accepted mdBook/parser/platform test suite is replayed.
- One new native shell test PASS verifies public full-source links/edition and
  absence of local-only copy, while retaining the default preview wording.
  Native strict Clippy and formatting PASS at the changed exporter scope.
- One actual local build-tool integration PASS produces15 files, preserved
  static-export receipts and a source/input/output manifest outside the upload
  root. This is deliberately preview mode because the working source is dirty;
  it does not assert a clean public-source candidate or hosted acceptance.
- actionlint initially rejects runner.temp at job-env scope. The bounded repair
  sets WASM_BINDGEN through GITHUB_ENV in a step. Focused actionlint then PASS;
  the initial failure is retained.

The canonical sources and search/browser runtime are unchanged. Reuse YT-224's
eight-chapter/no-JS/Chrome/mobile/keyboard/loading/failure/lifetime evidence,
YT-223's GNU/MSVC static-export evidence, and YT-218's existing Pages acceptance.
There is no new benchmark, Firefox suite, Windows run or live URL probe here.
Retained evidence: repository-external run yt225-guide-artifact-20261101.

## Remaining delivery and replacement gates

Independent Reviewer PASS0 accepts this bounded preparation as Done. Later
authorized Git delivery permits the new workflow to validate an actual clean
source and produce its hosted artifact. New reader Firefox smoke remains a
recommended unmeasured consumer check; it does not invalidate framework proof.
Replacing Pages requires separate owner direction, review of the actual artifact
and a deployment integration change. Only after that deployment can HTTPS,
project-directory routing, JS/Wasm MIME/CSP and artifact readback be accepted.
The current website, settings, automatic mdBook flow and local user preview are
not changed by this preparation.

## Final independent acceptance

Reviewer PASS0 has zero issues/required followups. All10 source/status bindings,
15 output hashes and the CLI input/output receipts match. The temporary copied
consumer is removed and its private receipts retained. Actual dirty --public
refusal leaves generated input unchanged. Documentation/link/structure/DAG,
formatting and tracked/untracked whitespace checks pass. Registered reviewer
gpt-6-astra/effective high are verified; review reruns no tests or browsers.
All10 changed paths remain unstaged/uncommitted, HEAD remains a306714, and no
Push, Merge, hosted execution, signing, publication or Pages change occurred.

## Authorized delivery and hosted artifact — 2026-11-01

The owner subsequently authorizes strict/builtin commit of the10 paths and Push
to the existing [PR44](https://github.com/bohewu/yetlet/pull/44), followed by clean
hosted artifact verification. Commit0544f4a096f31c76a352e99192d8e6baf3998272 has
tree2b50a607db322c7503ec92008866960394220b2f. Staged Guard/native hook pass with
zero findings; exact9099879→0544f4a outgoing/native-hook checks cover4 commits
and45 file operations, with3 nonblocking generic Windows-drive path warnings
from the earlier CLI guide. No Guard policy, hook or history is weakened.

[Guide preview41265011134/1](https://github.com/bohewu/yetlet/actions/runs/41265011134)
passes on its first attempt. The actual clean public-mode checkout is GitHub's
PR merge-ref67ae378fa9238795a94e3768a0a064a1e577e824, whose parents are the
unchanged master64d1074 and delivered head0544f4a; its tree exactly matches the
reviewed head. This is a PR build source, not a merged master or deployed edition.
The hosted build reports219 local references, unchanged SDK0.9.0-alpha.5 and
CLI0.1.1, complete temporary-consumer cleanup and separate diagnostic receipts.

Downloaded artifact readback PASS:

| Artifact | Exact ZIP SHA256 | Observed contents |
|---|---|---|
| yetlet-guide-preview /13067016568 | b0bf10bef73ba81dbb114e042ef9fa221e933df401f82f03cc1dd105da77ae88 | Exactly15 selected site files, including .nojekyll; no build records or old aliases |
| yetlet-guide-build-record /13066407992 | 50d2f9c2c3ec453a53d53f1ab944b5f85da7f0913c0987489bed29b3c300cfb1 | EXPORT-MANIFEST.json and two private CLI receipts, outside the site root |

Both ZIP digests match GitHub metadata. All15 file hashes/sizes match the actual
manifest and CLI receipt. All8 canonical sources/excerpts and10 tracked copied
consumer inputs match delivered Git blobs; generated JS/Wasm and copied CSS/
bootstrap assets bind to the same recorded inputs. Every HTML page has the full
PR-source link and correct public edition without local-only footer copy.
Receipts report process cleanup, temporary cleanup and completed create-only
filesystem publication; this does not mean website deployment. Archives are
inspected in memory, without extraction, browser/server startup or test replay.

The unchanged mdBook PR build41265010880 passes and its deployment job skips.
Both MSRV and Stable/native/samples/Chrome pass in source CI41265010842.
No rerun or retry is requested. Original failed local checks remain retained.
Public artifact, source/log/receipt readbacks and Guard/Git outcomes are saved
outside the repository in the authorized-delivery subdirectory of the same
YT-225 evidence set. PR44 remains open; no Merge, Pages replacement, settings
change, release or signing occurs. These hosted closeout edits remain uncommitted.

Final independent hosted closeout Reviewer PASS0 has zero issues/followups.
All3 document/status hashes, both downloaded ZIP digests,15 output hashes/sizes,
8 source/excerpt and10 tracked consumer bindings reconcile. The reviewer inspects
actual artifacts, receipts and source CI without rerunning tests or extracting
archives. Registered reviewer gpt-6-astra/effective high are verified. Only the
three hosted closeout documents remain unstaged/uncommitted after acceptance.
