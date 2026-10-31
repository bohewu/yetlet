# GitHub Pages minimum publication plan — YT-218

Date: 2026-10-31. Status: YT-218 Done after final independent Reviewer PASS0.
The owner-adopted delivery and first-publication sequence has completed:
[the public guide](https://bohewu.github.io/yetlet/) is live at source
`64d10742d62059c91dd2e46931aab1cb3b65d897`. The historical preparation checkpoints
below remain intact; the final section records actual settings, CI and readback.

## Historical planning baseline and exact public scope

[YT-217](YT-217-DOCS-PREVIEW.md) is Done at API-doc/local reader-preview scope.
Local committed source is `9ef215c2bf9d754f587fc5b3130b107ff0664c68`;
the observed remote `master` is `729fab315f7ea5c13da42cef87cf899a73d347f2`.
The two local documentation commits have not been pushed. Repository metadata
reports Public, default branch `master`, `has_pages:false`, no homepage URL and
no `github-pages` Environment. Existing release Environments are separate.

Recommend one English guide at **`https://bohewu.github.io/yetlet/`**, with the
existing mdBook project base `/yetlet/`, no custom domain and no additional Wiki.
Reuse the exact [eight selections and excerpt markers](../site/pages.toml):

| Chapter | Canonical source | Export boundary |
|---|---|---|
| Overview | Root `README.md` | Before `## Project and license` |
| First app | `docs/FIRST-APP.md` | Whole reader guide |
| Views/components | `docs/VIEW-COMPONENTS.md` | Whole reader guide |
| HTML/HTTP | `crates/yetlet/guides/server-axum.md` | Before historical `<details>` |
| Browser DOM | `crates/yetlet-browser/guides/dom.md` | Before historical `<details>` |
| Opt-in features | `docs/OPT-IN-QUICK-REFERENCE.md` | Whole reader guide |
| API reference | `docs/API-REFERENCE.md` | Whole reader guide |
| Troubleshooting | `docs/TROUBLESHOOTING.md` | Whole reader guide |

Publish generated HTML, its built-in search index and mdBook CSS/JS/fonts/icons.
The initial edition references SDK `0.9.0-alpha.5` and CLI `0.1.1`. Repository
links outside these chapters remain links to the exact documentation-source
commit; they do not recursively export their targets. Keep the
[documentation classification](DOCUMENTATION-SCOPE.md): no task/review reports,
benchmarks, screenshots, private evidence, raw source/Markdown/TOML, staging
directory or `EXPORT-MANIFEST.json` in the site artifact. Local native/WASM
rustdoc retains its accepted scope; uploading generated rustdoc or asserting
docs.rs availability is not part of this first site.

Upload **`target/docs-site/yetlet`**, whose root contains `index.html`.
Uploading its parent would add an incorrect second `/yetlet/` path and include
staging/manifest material. Use Actions artifact deployment, without a generated
`gh-pages` branch or direct publication of the repository's mixed `docs/` tree.
GitHub [supports this custom build/deployment path](https://docs.github.com/en/pages/getting-started-with-github-pages/configuring-a-publishing-source-for-your-github-pages-site).

## Minimum CI design

Propose one `.github/workflows/docs.yml`, using Ubuntu24.04, Python3.11+ and the
already qualified mdBook0.5.4 binary/digest. No Node/npm application install,
Cargo SDK build, browser download, signing or registry job is needed to render
the guide. Keep [the existing builder](../tools/build_docs.py) as its entry point.

| Event | Build | Deploy |
|---|---|---|
| `pull_request` | Build selected pages and upload a downloadable artifact | Never; no Environment, Pages or OIDC permission |
| `push` to `master` | Same document build | Only after initial enablement below, with `PAGES_DEPLOY_ENABLED == 'true'` |
| `workflow_dispatch` on `master` | Same document build, useful for the first publication | Same enablement guard; reject other branches/tags |

Trigger the automatic builds for the eight source paths above, `site/**`,
`tools/build_docs.py`, the two version inputs `Cargo.toml` and
`tools/yetlet-cli/Cargo.toml`, and the new workflow itself. Update these filters
when changing the source selection. Do not use `pull_request_target` or promote
an artifact from a PR run into a privileged deployment.

The build job has `contents:read`, checks out the exact event commit with
credentials persistence disabled, builds the selected book and checks its
artifact-root/selection/link correspondence. Upload one same-run `github-pages`
artifact with seven-day retention. PR artifacts are downloadable previews, not
public PR websites. Store the export manifest/hash receipt as workflow evidence
outside the uploaded site root. A proposed ten-minute build limit is a starting
bound, not an observed hosted timing.

The deploy job has `needs:build`, `pages:write` and `id-token:write` only, and uses
Environment `github-pages` with its deployment URL. It reads the same run's
artifact, never an arbitrary URL or run ID. On eligible deployments only,
`configure-pages` reads the existing Pages configuration with `enablement:false`;
verify its base path before deployment. Initial Pages creation remains an owner-
authorized settings operation. Use a separate Pages concurrency group with no
in-progress deployment cancellation; newer queued updates can supersede older
pending work. Proposed deploy timeout: ten minutes.

GitHub documents the [artifact, permissions, job dependency and Environment
requirements](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages).
The OIDC token here authorizes Pages deployment; it is not Sigstore signing or
crates.io publishing. No PAT, custom Secret, age key or release Environment is
needed. The build job receives no deployment permissions.

### Existing source CI stays intact

Protected `master` currently requires **Rust 1.99 MSRV** and **Stable native,
samples, and Chrome**, with administrator enforcement enabled. Keep these names,
rules and [the existing CI classifier](../scripts/release_ci.py). Merge only
after the required checks and new document build pass. The Pages check need not
be added to branch protection in this initial scope.

Pages deployment depends on its own successful document build. Ordinary master
Rust CI continues independently; this design does not make Pages wait for that
second Rust run or launch release workflows. Before the first publication,
observe green merged-master CI as part of the authorized activation sequence.
The initial delivery includes Rust source doc comments and new `site/`/workflow
paths outside the current light-lane allowlist, so expect existing full PR CI.
A light Pages build does not make that whole delivery light or authorize
loosening the classifier. Later accepted proof remains reusable at its scope.

### Action pins observed for implementation

Use complete immutable SHAs, not floating major tags. These official release
pins were inspected; resolve/check them again when implementing rather than
assuming a documentation example is current:

| Action | Version / SHA | Runtime |
|---|---|---|
| [configure-pages](https://github.com/actions/configure-pages/blob/45bfe0192ca1faeb007ade9deae92b16b8254a0d/action.yml) | v6.0.0 / `45bfe0192ca1faeb007ade9deae92b16b8254a0d` | Node24 |
| [upload-pages-artifact](https://github.com/actions/upload-pages-artifact/blob/fc324d3547104276b827a68afc52ff2a11cc49c9/action.yml) | v5.0.0 / `fc324d3547104276b827a68afc52ff2a11cc49c9` | Composite; pinned nested upload-artifact v7.0.0 uses Node24 |
| [deploy-pages](https://github.com/actions/deploy-pages/blob/368f82528645a54fb793d4d04e342629a3f51346/action.yml) | v5.0.1 / `368f82528645a54fb793d4d04e342629a3f51346` | Node24 |

Reuse the repository's accepted checkout pin. Do not change the existing direct
upload-artifact v7.0.2 pins or override Node runtimes to hide compatibility issues.

## Repository settings and update policy

Recommend accepting these settings as one initial publication decision:

- **Pages:** Settings → Pages → Source: GitHub Actions. Verify HTTPS enforcement
  at the default `github.io` URL; no DNS or custom domain setup. GitHub documents
  [HTTPS behavior and enforcement](https://docs.github.com/en/pages/getting-started-with-github-pages/securing-your-github-pages-site-with-https).
- **Environment:** create `github-pages`, allow deployments from the exact
  branch `master` only, with no tag patterns, Secrets or per-deployment reviewer.
  Protected-branch PR review and the one-time owner publication authorization
  remain in force; release Environment protections are unchanged.
- **Preparation switch:** an Actions repository variable `PAGES_DEPLOY_ENABLED`,
  absent/false while preparing the workflow, true only under the initial Pages
  enablement authority. This is a CI setting, not Yetlet Settings provider or a
  Cargo feature. A merged preparation workflow can build without publishing.
- **Routine updates after activation:** accepted relevant master changes build
  and deploy automatically. No new human Environment approval for each wording
  change. Turning the variable off stops new deployments; it does not unpublish
  a site already live. Unpublishing or selecting an old source needs separate
  direction, not a claim that disabling updates rolls back published content.

## Small implementation delta and first-publication sequence

Before activation, add only the workflow and a public-build label path to the
existing builder. Today its banner always says `Local documentation preview`.
Keep that for local/PR previews; a clean master publication must identify a
deployed guide, full documentation-source commit and the guide's validated
SDK/CLI edition. Workspace version strings alone are not proof of published
availability. Reject dirty publication inputs; do not alter guide bodies,
renderer behavior, package versions or the eight-page selection to change a label.

Validate only the new label/workflow/artifact-root/permission conditions. Reuse
YT-217's unchanged guide, layout, search and touch proof; verify the real hosted
URL/assets after the first actual deployment, since localhost cannot prove
GitHub routing. No existing SDK/platform suite replay is needed for the site job.

Future authorized delivery sequence:

1. Implement the bounded delta locally, validate it and obtain independent
   Reviewer PASS. Keep the deployment switch off during preparation.
2. With Git authority, strict/builtin Commit/Push one coherent PR containing
   the existing local documentation commits and new site tooling. Required CI
   and document build must pass before an authorized Merge; record the actual
   merged source SHA. Local `9ef215c` is the proof anchor, not an already hosted
   deployment or the eventual merged documentation-source identity.
3. Under explicit initial-publication authority, confirm merged-master CI,
   configure Pages/Environment, turn the switch on and dispatch one first build/
   deploy at master. Retain failures; do not automatically retry a failed run.
4. Read back the public HTTPS home, First app and API chapter, local assets and
   search/menu journey; reconcile source/edition and exact artifact root. Keep
   the deployment/run/manifest receipts and prior failures. Only then describe
   the site as published; repository homepage metadata can be updated separately.

## Owner-adopted choices

| Decision | Adopted recommendation | Authority accepted for this batch |
|---|---|---|
| Destination and disclosure | Default HTTPS project URL; exact eight-page edition and generated assets only; Wiki/domain/rustdoc upload deferred | Initial public website publication |
| Update policy | One initial authorization; thereafter protected-master document builds deploy automatically, without a repeated Environment gate | Persistent Pages settings, Environment restriction and CI variable policy |
| Delivery and first execution | One bounded reviewed PR/CI/Merge batch, then settings and a single first deploy/readback; failures retained without automatic retries | Git operations, external settings and hosted execution, when explicitly authorized |

The owner's subsequent instruction to follow these recommendations adopts all
three choices. Preserve the staged order: review before Git delivery, required
CI before Merge, then settings and one first deployment/readback. No domain,
new token or release-key rotation is needed. Initial failures are retained and
failed hosted runs are not automatically retried.

## Implementation checkpoint — 2026-10-31

The dedicated workflow uses the pinned actions above and leaves the deployment
switch absent/off. Preview jobs have read-only repository access; the deployment
job is limited to eligible master push/manual events, the true switch and a
successful public build. Its Environment is `github-pages`, with Pages/OIDC
permissions only. The configure action cannot enable Pages itself. No existing
Rust/release workflow or CI classifier is modified.

The builder adds `--public`, explicit SDK/CLI edition metadata, a full-commit
source link and clean-checkout/edition refusal before replacing any output.
Its bounded output check validates chapter HTML, local links/anchors and the
actual upload root, rejecting source/manifest files, symbolic links and paths
outside that root. Public and preview pages keep identical guide bodies.

New local evidence under `target/yt218-pages-20261031/validation/`:

- Nine focused label/output tests PASS. A real dirty `--public` CLI invocation
  is correctly rejected before changing the existing preview; this expected
  negative check is not a failed publication.
- The new builder renders eight unchanged YT-217 chapter sources and checks454
  local links/assets/anchors. Accepted doctest/native/WASM/browser/layout proof
  is reused; no old product suite is replayed locally.
- Eight cases executing the actual workflow mode script PASS: PR, disabled
  switch, feature/tag refs and master push/manual outcomes are distinguished.
- Official actionlint1.7.12 syntax/context check PASS. Its downloaded tool is
  outside tracked source and matches upstream release digest
  `8aca8db96f1b94770f1b0d72b6dddcb1ebb8123cb3712530b08cc387b349a3d8`.
- All three official action release pins were re-observed and match this plan.

These checks do not claim hosted Pages acceptance. Final clean-source public
integration precedes Push; actual public URL/artifact/source readback follows
the one authorized deployment. Mark YT-218 Done only after those outcomes exist.

## Completed delivery and first publication — 2026-10-31

Implementation review PASS0 accepted the ten changed inputs without required
followups. Nine new tests, eight actual workflow-mode cases, actionlint, the
clean public build and454 local references passed. The eight YT-217 guide body
hashes were preserved; no accepted local SDK/platform suite was replayed.

Strict/builtin staged and exact-outgoing checks, plus native hooks, passed
before delivery. Three inherited generic Windows-drive path warnings were
nonblocking; no Guard policy was weakened. [PR43](https://github.com/bohewu/yetlet/pull/43)
squash-merged as `64d10742d62059c91dd2e46931aab1cb3b65d897`, tree
`10223623f4fb67108dc08cf62a8756dc0d10dc20`, exactly matching the qualified
`85cdadc37804e344a1beca164beb6a6c33a46008` source tree. Original local branches
and a verified repo-external Git bundle preserve pre-merge history.

The PR's required Rust CI41195957987, document build41195958007 and all four
CodeQL analyses41195956270 passed. Merged-master Rust41197101236, document
build41197101341 and all four CodeQL analyses41197100937 also passed before
activation. Their preparation deploy jobs were intentionally skipped; no
product release workflow was dispatched.

Actual configured settings:

- Pages uses GitHub Actions at **https://bohewu.github.io/yetlet/**, no CNAME,
  `https_enforced:true`.
- `github-pages` allows exactly the branch `master`, with no reviewer, tag
  pattern or Secret. The four existing release Environment protections remain
  unchanged.
- Repository variable `PAGES_DEPLOY_ENABLED` is `true`. Future relevant protected-
  master pushes build and deploy automatically; PRs only build previews.
- No product version, release tag, signing identity or crates.io state changed.

[First publication41198529190/1](https://github.com/bohewu/yetlet/actions/runs/41198529190)
completed both build and deploy successfully, with no hosted retry. Its
`github-pages` artifact13040884335 has ZIP SHA256
`685ed585fe825b17b5970184dfd7f5a3b36bed67dd3b35bbc7afbbd8a9a51b07`.
Inspection confirms42 generated files at the correct upload root, eight selected
chapters and no raw Markdown/Rust/TOML, staging or export manifest. The logged
manifest identifies clean public mode, SDK0.9.0-alpha.5, CLI0.1.1, the actual
merged source and all eight matching source hashes.

Anonymous HTTPS readback of home, First app, API and the actual hashed search
index returned200 and matched their artifact bytes. Existing Chromium at
1440x1000 verified the public source/edition, navigation and SessionConfig search.
A390x844 touch-emulated journey verified the menu, Views navigation and menu
closure. No captured page, console or HTTP errors occurred; all three screenshots
were visually inspected. This adds actual Pages routing/assets acceptance to the
reused local proof, not physical-mobile or broader browser qualification.

Two operational/harness observations remain recorded, without deployment retry:

1. A redundant HTTPS-enforcement write returned404 (`The certificate does not
   exist yet`). Readback already showed the desired `https_enforced:true` on
   the default github.io site; no second write or weaker setting was needed.
2. An extra asset comparison assumed `searchindex.js` and returned404. The
   canonical artifact instead contains `searchindex-071e7429.js`; one smallest
   harness correction compared that file successfully, reusing the three
   successful HTML readbacks. Browser search had already passed.

Raw CI/settings/artifact/manifest/readback/browser/error receipts are retained
under `target/yt218-pages-20261031/` and copied outside the repository to the
`yt218-pages-20261031` release-evidence set. Browser contexts were closed; no
preview server or temporary cloud resources were created for this readback.
Final independent hosted-evidence/document Reviewer PASS0 accepts the five
closeout document hashes, actual publication evidence and bounded completion.
There are zero issues or required followups; registered reviewer model
`gpt-6-astra` and effective high effort were verified. YT-218 is Done.
Wiki, custom domain, generated-rustdoc hosting and further editions remain deferred choices,
not incomplete first-publication criteria.
