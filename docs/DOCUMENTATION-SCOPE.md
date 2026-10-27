# Documentation sources and publication scope

Date: 2026-10-26. This is a documentation preparation contract. No website,
Wiki, package publication or repository-visibility change is activated.
Yetlet remains unpublished and experimental; the original release-candidate
scope and approval gates are unchanged.

## Canonical sources

Use [the current API reference](API-REFERENCE.md) to choose a package and its
usage guide. Rust source and its rustdoc describe signatures, ownership and
target/feature availability; the linked current guides describe behavior and
integration limits. Read both at the same source revision. A discrepancy is a
documentation or implementation defect, not permission to weaken a contract.

A documentation website or Wiki should render or link those versioned sources.
Do not maintain a second independently edited API contract. Record the source
revision and feature/target selection of any generated rustdoc. Existing tests
and qualification records remain bound to their own source identities; publishing
documentation does not renew that evidence or make an API stable.

## Classification and selected reader sources

These exact source selections define the first reader-documentation scope.
They are manuscript inputs, not permission to publish their complete contents
without the separate publication decision below. Selection is not recursive:
linked reports, screenshots, assets and source files are not silently included.

| Class | Selected sources | Reader purpose and boundary |
|---|---|---|
| Current reference | [API-REFERENCE.md](API-REFERENCE.md), this page | Current API carriers, source ownership, opt-in selection and documentation boundaries |
| First application | [FIRST-APP.md](FIRST-APP.md) | Existing native CLI/source-checkout onboarding, app-owned dependencies/lock/cache, expected routes and common failures; no new runtime qualification |
| View authoring | [VIEW-COMPONENTS.md](VIEW-COMPONENTS.md) | Existing first-app continuation: ordinary function components, typed data, lists/conditions, owned Views and renderer boundaries |
| Opt-in selection | [OPT-IN-QUICK-REFERENCE.md](OPT-IN-QUICK-REFERENCE.md) | Existing package/features, startup APIs/options and exact consumer settings/defaults; no generic registry or new keys |
| Troubleshooting | [TROUBLESHOOTING.md](TROUBLESHOOTING.md) | Symptom/cause/correction lookup linking existing setup and capability guides; no new execution qualification |
| Guide authoring | [GUIDE-TEMPLATE.md](GUIDE-TEMPLATE.md) | Shared five-part integration guide format; current source, prerequisites, expected result and failure boundaries stay explicit |
| Framework guides | [ARCHITECTURE.md](ARCHITECTURE.md), [HTTP-PIPELINE.md](HTTP-PIPELINE.md), [EXTENSIONS.md](EXTENSIONS.md), [NAMING.md](NAMING.md), [COMPATIBILITY.md](COMPATIBILITY.md), [CAPABILITIES.md](CAPABILITIES.md) | Architecture, composition, naming, current availability and explicit compatibility limits; dated task/review paragraphs remain historical annotations |
| Settings guides | [CONFIGURATION.md](CONFIGURATION.md), [SECRET-PROVIDERS.md](SECRET-PROVIDERS.md), [SETTINGS-SECRET-PROVIDER.md](SETTINGS-SECRET-PROVIDER.md) | Current startup loading, secret lookup and protected-field injection; omit historical execution/review bookkeeping from a reader edition |
| Package guides | The sixteen package-guide/source links in the current API reference tables | Public entry points, examples, errors, ownership, targets and limitations; seven later packages remain outside the original nine release candidates |
| Application and CLI guides | [SAMPLES.md](SAMPLES.md), [BOARD-START.md](BOARD-START.md), [sample index](../examples/README.md), [Board README](../examples/lab-operations-board/README.md), [Admin README](../examples/admin-dashboard/README.md), [Shop README](../examples/modular-shop/README.md), [Starter README](../examples/production-starter/README.md), [CLI README](../tools/yetlet-cli/README.md) | Existing runnable usage and application-owned options; follow the exact sample-local README links in the sample catalog when selecting additional sample pages |
| Root overview and policies | [README](../README.md), [license choice](../LICENSE-MIT), [Apache license](../LICENSE-APACHE), [security policy](../SECURITY.md), [contribution policy](../CONTRIBUTING.md) | Introduction, license and existing reporting/contribution rules; selection does not activate reporting services or change governance |

YT-201 adds package-local configuration and signing READMEs to the current API
map. They provide archive-local introductions and licenses; the maintained
configuration guide and signing source/rustdoc retain their respective ownership.
A historical signing contract is context, not an additional current reader-source
selection or automatic publication input.

The current API reference is a selection map, not a dump of every `pub` item.
Rustdoc can contain adapter/support items and broad upstream reexports;
visibility alone does not promote them to recommended or supported Yetlet APIs.
Generated output for optional or browser APIs must use the relevant feature and
target; one native default build cannot represent every configuration.

The [YT-205 current-C reconciliation](YT-205-RELEASE-EVIDENCE-CONSOLIDATION.md#reader-inventory-and-decision-reconciliation)
binds46 exact reader manuscripts at technical candidate118f3d0 to the previously
selected draft and retains native/WASM rustdoc input limits. Both Session README
corrections are now in C's actual archives. This is a versioned source selection,
not a sanitized ready-to-publish site. Historical annotations and unselected-target
links still need an explicit reader edition/destination mapping before export.
No image asset is selected and selection is not recursive.

The [YT-206 public-source inventory](YT-206-PUBLIC-PREVIEW-DISCLOSURE.md) separately
lists repository/history/Actions exposure and selected GitHub private-reporting
activation checks; a separate mailbox/domain is optional.
Source-preview preparation does not publish a reader site or resolve formal
signing/registry decisions. Repository visibility remains Private.

## Retained evidence and maintainer material

The [repository index](README.md) is intentionally a mixed navigation record,
not a page to export wholesale. The following material is retained in the
repository but excluded from the default reader-site selection:

- [Backlog](BACKLOG.md), [Roadmap](ROADMAP.md), agent/workflow instructions and
  task-delivery records, including this documentation delivery's own record.
- `docs/YT-*`, milestone verification/planning/review documents, research
  experiments, API classification snapshots, performance measurements and
  screenshot/evidence inventories. A current guide may cite an exact historical
  result without making the whole report current or a publication input.
- Private release/trust/signing decisions, cloud allocation and cleanup history,
  machine-specific evidence paths, reviewer/job identifiers and local operational
  details. Do not reproduce those details in newly authored reader pages.
- Unlisted files. Being under `docs/`, being tracked, or being linked from a
  selected guide does not itself select publication. Architecture decisions and
  threat-model context can be selected explicitly later; no directory glob does so.

Keep original dates, source IDs, PASS/FAIL results, failures and qualification
limits. Label superseded usage instead of deleting evidence or rewriting an old
verdict. This classification controls audience and publication inputs; it does
not declare every excluded file confidential or every selected file sanitized.

## Separate publication decisions

Before an actual publication, choose the destination and an exact page/asset
list, read the selected complete content for private or machine-specific details,
and reconcile links to material that will not be published. Keep necessary
behavior and security limits in the reader edition; do not remove them to make
an experimental feature appear complete. This preparation does not certify a
ready-to-host site or authorize another validation campaign.

Website content and repository visibility are different boundaries. A future
public repository exposes its tracked files and accessible history regardless
of site navigation or a directory named `internal`. Moving a file or hiding a
link does not make it private. Any history/visibility or private-archive decision
requires separate owner direction; no removal or history rewrite is selected here.

The existing release/trust decisions stay pending. Documentation preparation
does not publish packages, sign artifacts, enable hosting or expand support.
