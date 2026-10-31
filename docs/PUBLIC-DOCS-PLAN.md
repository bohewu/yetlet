# Public documentation and rustdoc plan

Date: 2026-10-31. YT-217 is Done at selected API-doc/local reader-preview
scope after independent final Reviewer PASS0. Its [execution record](YT-217-DOCS-PREVIEW.md)
retains the original passing proof, stopped harness and authorized correction.
YT-218 subsequently delivered the separate Pages workflow/settings and first
publication; [the guide](https://bohewu.github.io/yetlet/) is live at source
64d1074. [Actual publication evidence](GITHUB-PAGES-PUBLICATION.md#completed-delivery-and-first-publication--2026-10-31)
records green CI, source/artifact/edition matching and hosted reader acceptance,
accepted by final independent Reviewer PASS0; YT-218 is Done. No product API/runtime/version change
or expansion of signed release-candidate scope follows from website publication.

## Recommended destinations

Use **GitHub Pages** for the public guide site, the existing repository for
canonical Markdown/source, and **rustdoc** for version-matched API reference.
GitHub Pages serves static HTML/CSS/JavaScript and is available for public
repositories on GitHub Free; the published project URL is
[bohewu.github.io/yetlet](https://bohewu.github.io/yetlet/). See [GitHub Pages documentation](https://docs.github.com/en/pages/getting-started-with-github-pages/what-is-github-pages).

Recommend [mdBook](https://rust-lang.github.io/mdBook/) for a small Markdown-based
guide with navigation and search. Keep one repository and existing guide
sources; the installed site build stages explicitly selected files into its book
input. Do not copy the guides into a second manually edited source tree.
YT-217 pins mdBook0.5.4 as local ignored tooling; the checked-in builder validates
that version. No site tooling becomes a Yetlet runtime dependency.

Defer GitHub Wiki. It can host long-form documentation, but a second editable
API reference would duplicate the repository contract. If a community need
appears later, consider a short landing page linking to canonical guides or
clearly labeled community recipes. GitHub also recommends Pages for content
that needs search-engine indexing; see [About wikis](https://docs.github.com/en/communities/documenting-your-project-with-wikis/about-wikis).
No organization split, new repository, custom domain, or Wiki setting change
is needed for this first scope.

## Reader navigation

The README introduces the product, current availability and shortest useful
path. The guide teaches integration; rustdoc supplies callable API details.
Readers should not need release investigation or backlog history to begin.

| Navigation group | Existing source inputs | Purpose |
|---|---|---|
| Start | [Root README](../README.md), [First app](FIRST-APP.md) | Install CLI0.1.1 without cloning, create a project using SDK0.9.0-alpha.5 and follow explicit build/run steps |
| Write UI | [Views/components](VIEW-COMPONENTS.md), [server guide](../crates/yetlet/guides/server-axum.md), [browser DOM guide](../crates/yetlet-browser/guides/dom.md) | HTML-first composition, native handlers and optional browser interaction |
| Choose features | [Opt-in quick reference](OPT-IN-QUICK-REFERENCE.md), [Settings/options/logging](SETTINGS-OPTIONS-LOGGING.md), selected linked module guides | Cargo features, startup installation, configuration, ownership and failure behavior |
| API reference | [API map and Rust reading guide](API-REFERENCE.md), same-version generated rustdoc | Locate a type/method/error and understand feature/target availability |
| Examples and help | [Sample catalog](SAMPLES.md), [CLI guide](../tools/yetlet-cli/README.md), [Troubleshooting](TROUBLESHOOTING.md) | Choose an existing example or recover from a concrete symptom |
| Status and policies | [Compatibility](COMPATIBILITY.md), [Security policy](../SECURITY.md), [Releases](https://github.com/bohewu/yetlet/releases) | Experimental limits, reporting and actual distribution |

Retain the [guide template](GUIDE-TEMPLATE.md): prerequisites, minimal
integration, expected result, configuration/errors, API reference. Explain new
Rust terms at first use; avoid assuming readers already know Cargo feature
unions, owned versus borrowed values, or the package-name/import-name difference.
Prefer the existing working snippet over a second, subtly different example.

These groups define information architecture, not the final export manifest.
The [documentation classification](DOCUMENTATION-SCOPE.md) remains authoritative:
selection is not recursive. Do not export all `docs/**` or all Markdown links.
Backlog, Roadmap, YT-* task/review/research records, release internals, benchmark
logs, local paths and private evidence stay outside the default reader edition.
Necessary behavior/security limitations must survive reader editing; historical
research links can remain explicit GitHub links without copying whole reports.

## Version and API contract

- Label the first guide edition with SDK `0.9.0-alpha.5`, CLI `0.1.1`, and its
  actual documentation source commit. Documentation may improve after a release;
  that must not imply that the later checkout was the signed product candidate.
- Link API signatures for the exact published package version when available.
  Keep checkout rustdoc labeled with source commit, target and selected features.
  A native page cannot stand in for browser-only APIs. Local build commands are
  in the [API reference](API-REFERENCE.md#local-rustdoc); this plan asserts no new
  docs.rs build or hosted rustdoc qualification.
- If a later development edition is useful, label it Development and separate
  it from released APIs. Do not invent archived editions for unverified tags or
  expose two versions of the same page without a clear version label.
- Keep public documentation in English. This first site needs no translations,
  analytics, application login, external JavaScript service or custom UI runtime.

## Implementation order — YT-217

1. **Improve the API entry points first.** The [focused source inventory](API-REFERENCE.md#current-doc-comment-coverage-and-gaps)
   selects two crate homepages, View/Element/Attribute and SessionConfig/errors.
   Add purpose, the smallest useful example, applicable feature/target gates,
   ownership and error/security behavior. Preserve API/output and existing
   contracts. Only new/changed examples and affected rustdoc links/builds need
   validation; do not replay unrelated SDK/platform qualification. Do not enable
   a repository-wide missing-docs failure policy as part of this slice.
2. **Build a local static reader edition.** Review complete chosen pages/assets,
   record the exact export list and source, then wire the existing Markdown into
   mdBook. Check navigation/search, internal links, code-block presentation,
   mobile reading and the `/yetlet/` project base path. Excluded source/evidence
   files must not enter the built output. Keep native/browser API links explicit.
3. **Enable publication only after accepting the preview.** A later dedicated
   Pages workflow should build a preview for relevant documentation changes and
   deploy only accepted default-branch content. Keep it separate from SDK/CLI
   release jobs and their Secrets, signing and registry permissions. Use the
   existing bounded documentation CI lane; documentation edits should not
   dispatch product release workflows. The initial Pages enablement/deployment
   and any `github-pages` Environment configuration need separate owner authority.

Use GitHub's [custom Pages workflow guidance](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)
when implementing the last step. Pages deployment permissions are site-hosting
permissions; they do not authorize artifact signing or crates.io publication.
Do not add repeated release-style manual gates for each wording correction.

## Evidence and current limits

The original planning batch reconciled existing release receipts and source
comments without product verification replay. The later owner-authorized
YT-217 slice adds only its new examples, selected rustdoc and preview checks.
[CLI0.1.1's completed consumer](YT-216-CLI-011-PATCH-RELEASE.md#completed-publication-and-installed-consumer--2026-10-31)
and the [SDK release record](ALPHA5-HOSTED-RECORD.md#completed-publication-and-subsequent-cleanup)
keep their original sources and limits. The API inventory is a manual sample of
public entry points, not a full coverage score. Source comments and the local
static build retain their accepted scope. Navigation/search/mobile/screenshot
checks pass at local Chromium/emulated-mobile scope after final independent review.
The later YT-218 publication adds actual public routing/assets/reader evidence
at its recorded source and Chromium/emulated-mobile scope. Physical mobile,
additional browsers and hosted generated rustdoc are not newly qualified.
