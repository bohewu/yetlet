# YT-249 — Reader Edition publication follow-through

Date: 2026-11-10. Status: Done at independently reviewed source and local consumer scope.

## Owner-selected scope

The owner authorizes promotion to published SDK `0.9.0-alpha.7` and CLI `0.3.0`,
including the public Reader Edition, README and Pages. Future SDK or CLI
publications must include the same reader update in their closeout. This
supersedes the previous hold on reader promotion; it does not change historical
[YT-232](YT-232-PUBLIC-GUIDE-EDITION.md) proof or authorize package republication.

Accepted publication records remain the source of published versions.
`site/pages.toml` remains the reviewed consumer selection. The required existing
release-document check now rejects disagreement between these two authorities,
independently of development manifests. Existing sync renders tutorial versions;
explicit `new --sdk-version` preserves the selected SDK, while the generated
Cargo.lock and resolution receipt survive subsequent frozen builds.

Only the docs-site and static reference SDK pins/lock entries advance. Preserve
third-party lock entries, checksums, eight chapters, rendering/search/navigation,
SDK/CLI archives, tags, signatures, historical CLI bindings and Runner permissions.
The [release SOP](RELEASING.md#required-reader-update-after-every-sdk-or-cli-publication)
requires compatible published-CLI consumers, native/Wasm/static export, independent
review, strict/builtin Guard, exact-head CI, merge, master CI and live readback
before closeout. If the new SDK is not supported by the published CLI, report the
blocker and keep closeout open; never silently select an unverified pair.

## Verification and delivery

The accepted CLI0.3.0 registry installation and six native/browser resolver cases
retain their original publication scope. New local execution reuses the accepted
published-equivalent candidate archive installation; it is not a fresh registry
installation. No SDK/CLI source, archive or release workflow changes.

| Changed surface | New evidence |
|---|---|
| Reader consumers | Both locks advance only yetlet/yetlet-browser to alpha.7, preserving all third-party entries and registry sources/checksums |
| Native/Wasm reader | Full actual native renderer and Wasm build PASS; eight chapters and exactly15 public files, private manifests/receipts excluded |
| Static reference | Frozen wasm32 library and native release export PASS, six public files and unchanged app lock |
| First-app walkthrough | CLI0.3.0 exact alpha.7 new, generated Cargo.lock/resolution receipt, explicit fetch, frozen build/run and HTTP SSR PASS; lock/receipt unchanged and owned server stopped |
| Existing gates | 14 version-sync,21 release-document and23 reader-build Python tests PASS; release-doc sync idempotent/check PASS; cargo xtask docs75/backlog246 and whitespace PASS |

Original sync omissions/regex failure and the local static-output-parent protection
failure are retained. The single run-local correction uses an owned0700 output
parent; no shared permissions or product protections change. The reader export
and static Wasm check already passed and were not replayed. Temporary compiler,
consumer and server workspaces are removed. This round's isolated Rust cache is
cleaned after delivery, while logs, original failures and accepted outputs stay.

No new Windows or browser interaction/platform qualification is claimed. Existing
published SDK/CLI browser and platform evidence keeps its original scope. Public
mode, exact-head CI and live Pages correspondence will be checked through the
existing guarded delivery; source preview is not evidence of a completed deployment.

Independent Reviewer PASS, zero issues/required followups, verifies all26 frozen
source identities and the new/reused evidence boundaries. Only acceptance/status
text follows this review, with the existing docs and whitespace checks. Guarded
public delivery still requires exact-head PR/master CI and live Pages readback;
those results are recorded separately and are not inferred from this source review.
