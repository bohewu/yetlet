# YT-232 — Public guide edition and static-export introduction

Date: 2026-11-02. Status: Done at local guide-source update scope.

## Scope and acceptance contract

The owner selects published SDK `0.9.0-alpha.6` and independent CLI `0.2.0`
for the public introductory guide's next source edition. Update the reader
instructions, exact edition admission and registry-only documentation consumer
together. Introduce static export from the first-app path through the existing
application-owned exporter example; align that example's SDK pin with the guide.
Register the work in Backlog/Roadmap and require independent Reviewer PASS
before Done. No Commit/Push or Pages deployment is authorized.

- Preserve eight selected chapters and existing rendering/search/navigation
  behavior. A guide edition is an explicit reviewed choice, independent of
  future workspace versions. Public mode still requires a clean source.
- Installation, expected CLI output, generated SDK dependency and guide labels
  must agree. SDK and CLI retain independent version numbers.
- A plain native SSR scaffold does not become a static exporter automatically.
  Explain the explicit trusted native export entry, prepared lock/cache,
  create-only output, public/private artifact boundary and optional Wasm islands.
  Do not promise a static scaffold flag, route crawling or automatic deployment.
- Keep existing SDK/CLI runtime, package versions, external dependencies,
  renderer policies and Pages workflow/settings unchanged. Only the two
  nonpublishable guide/reference consumers advance their exact registry SDK
  requirements and corresponding lock entries.
- Preserve dated historical evidence, old release identities and the current
  live Pages edition. Local updated sources/output do not qualify live URLs or
  publish a package.

## Evidence reuse and necessary delta

[SDKalpha.6 publication/registry consumers](ALPHA6-RELEASE-PREPARATION.md#owner-approved-publication-and-registry-consumer--2026-11-01)
and [CLI0.2.0 publication/installed consumer](CLI-020-RELEASE-PREPARATION.md#owner-approved-publication-and-installed-consumer--2026-11-02)
already verify actual distribution, checksum, installation, version/help/new and
the generated exact SDK dependency. CLI preparation also verifies the minimal
SDKalpha.6 static consumer. Reuse these results without reinstalling or
repackaging the releases.

[YT-224](YT-224-DOCS-PILOT.md), [YT-225](YT-225-DOCS-PUBLIC-BUILD.md) and
[YT-226](YT-226-PAGES-CUTOVER-PLAN.md) retain their original navigation, search,
touch, public artifact and HTTPS evidence. The eight-page shape, frontend and
exporter source stay unchanged. Do not replay their whole browser/platform
matrix or relabel old-source observations as new-source proof.

Necessary delta: changed edition/consumer-admission checks; registry-only lock
correspondence without third-party upgrades; one local guide build with the new
native/Wasm SDK consumer; the static-reference build only where its exact prior
input correspondence cannot be established; changed content/link review and
the existing docs/release-document/whitespace checks. Use a disposable protected
consumer/output and keep private manifests/receipts outside the site root.
No hosted run, new clean-source public qualification or live readback is selected.

## Progress and limits

Contract recorded before edits. Current first-app/install/expected output,
Views/HTTP/DOM, opt-in, troubleshooting and entry-point text select SDKalpha.6
and CLI0.2.0. First-app's static section links the existing explicit exporter;
the eight-page selection is unchanged. Historical guide details retain their
original editions and outcomes.

The existing guide-edition gate now selects the reviewed published pair.
Preparation shares that gate instead of duplicating version literals and
requires all declared guide SDK dependencies to pin the selected registry
version exactly. Public mode still rejects dirty or unadmitted sources;
new workspace versions cannot silently promote the reader.

| Necessary local delta | Result and scope |
|---|---|
| Edition/admission checks | 11 affected Python tests PASS: new labels, dirty/unadmitted refusal, independent workspace versions, preview and consumer mismatch before output |
| Registry consumer locks | Both docs-site/static-site update only yetlet and yetlet-browser to alpha.6 with registry source/checksums; every third-party lock entry is identical |
| Docs reader build | Rust1.99 locked/offline native exporter and release Wasm consumer PASS with existing CLI0.2.0/wasm-bindgen0.2.129; local preview produces eight chapters and exactly15 site files with219 checked local links, private manifest/receipts outside output |
| Reader content | Install command/expected CLI0.2.0, exact SDKalpha.6, static section/anchor and separate public HTTP root verified; no old first-app version remains |
| Static reference | New locked/offline wasm32 library check and explicit HTML-only export PASS; six public files, escaped text and meta CSP; private receipts remain outside output |
| Documentation gates | Release-document check, docs75/232 and tracked/new-file whitespace checks PASS; final acceptance/status checks follow review |

The reader's JavaScript, CSS, search implementation, Rust exporters and eight
routes are unchanged. Browser/search/navigation/touch and SDKalpha.6 platform
results remain reused at their original scope. The attempted in-app browser
check returned "browser access host is unavailable" before creating a tab;
no new browser interaction or live Pages qualification is claimed. Its owned
loopback server is closed. Earlier preflight also found wasm-bindgen missing
from PATH; the existing0.2.129 binary was selected without installation.
Both operational failures are retained, without a browser retry or product fix.

Raw new logs/manifests, SDK-only lock comparison, cleanup and reused-evidence
references are retained outside the repository under
yt232-public-guide-edition-20261102. Independent read-only Reviewer PASS,
zero issues/required followups, verifies all24 frozen source hashes, Git status,
artifact/source hashes and both SDK-only lock deltas. It accepts Done at local
guide-source scope, followed only by acceptance/status text and the required
documentation/whitespace checks.
The live guide is still the separately deployed edition; later delivery and
deployment require their own owner authorization. No clean-source public build,
hosted job, reinstall, package publication, Commit/Push or Pages change occurs.
