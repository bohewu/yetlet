# YT-221 — Controlled DOM navigation and image policies

Date: 2026-10-31. Status: Done at selected unreleased-source scope; contract
and final independent Reviewer PASS0, zero issues/required followups.
Baseline: `4b88f3fec2325170fa4b1109cb08d2a53518c55c`, the reviewed YT-220
implementation committed with strict/builtin and the native hook, no Push.
One nonblocking PATH_UNC warning is an unchanged hostile-URL test literal;
no blocking findings or Guard bypass. The owner selects contract first, then
minimal opt-in implementation and new-behavior evidence; new work is not to be
committed/pushed. YT-222–224 remain Todo.

## Purpose and fixed boundaries

Implement the explicit URL/image branch of the accepted
[YT-219 contract](YT-219-HTML-DOM-CONTRACT.md#yt-221--explicit-options-and-resource-boundaries).
Keep public View/Element, existing validate/mount/error signatures, old strict
URL defaults, same-origin form submission and YT-220's89 ordinary elements.
Image selection adds img only, yielding90 elements only in that selected mode.
No generic resource loader, new package/dependency, server capability, automatic
retry, Settings provider, DI, hydration or live-site replacement is selected.

Use the browser's [URL/origin model](https://url.spec.whatwg.org/#origin),
the HTML [noopener semantics](https://html.spec.whatwg.org/multipage/links.html#link-type-noopener)
and [image loading model](https://html.spec.whatwg.org/multipage/images.html#updating-the-image-data).
Native validation remains spelling/structure validation; actual browser origin,
effective-base and configuration-origin decisions happen before mounting.

## Typed API contract

New exports live under `yetlet_browser::dom` (native types/validation, Wasm-only
mounting), with one canonical name per API:

| API | Selected shape and meaning |
|---|---|
| NavigationPolicy | Enum: SameOrigin (Default); HttpsOrigins(Vec<String>) for additional explicitly named HTTPS origins |
| ImagePolicy | Enum: Disabled (Default); SameOrigin; HttpsOrigins(Vec<String>) for same-origin plus named HTTPS image origins |
| DomOptions | Clone/Debug/Default/Eq value with private fields; construct with `DomOptions::default() -> DomOptions`, no additional constructor. Getters: `navigation_policy(&self) -> &NavigationPolicy`, `image_policy(&self) -> &ImagePolicy`. Infallible consuming builders: `with_navigation_policy(self, policy: NavigationPolicy) -> Self`, `with_image_policy(self, policy: ImagePolicy) -> Self` |
| validate_with_options | `(view: &View, options: &DomOptions) -> Result<(), DomPolicyError>`; static structural/spelling check, not origin/configuration acceptance |
| mount_with_options | `(target: &web_sys::Element, view: &View, options: &DomOptions) -> Result<(), JsValue>` on Wasm; complete policy/URL preflight, then owned construction/replacement |

The original validate/mount functions use default options. No existing public
error enum variant/signature is changed. Origin lists are public configuration
values, not prevalidated origin objects: native validation does not validate
their contents or prove destination membership. It may accept an HTTPS spelling
that mounting later rejects. This is consistent with existing native validation
not proving the effective browser base. Do not use native validation as an
authorization gate. Invalid option origins always make mounting fail before
creating nodes or assigning resource attributes, even for a View with no URLs.
No constructor pretends to validate origins without a standard URL parser.

Wasm preflight parses each configured origin with URL, requires HTTPS, no user
information, query or fragment, and accepts only its canonical origin spelling
with an optional single trailing slash. This deliberately rejects paths,
wildcards, noncanonical spellings and ambiguous origin-like strings. Compare
parsed canonical origins exactly; no prefix or suffix hostname matching.
Lists are independent: navigation permission does not enable image loading,
and image permission never enables navigation or form actions.

## Admission matrix

| Surface | Default | Explicit option |
|---|---|---|
| Anchor href | Existing root-relative path or fragment spelling | HttpsOrigins adds absolute lowercase https:// destinations, provisionally at static validation, exact allowed origin at mounting |
| Anchor extra attributes | Existing admission unchanged; target/rel/hreflang/referrerpolicy stay rejected | HttpsOrigins, including an empty list, selects the finite extra-anchor vocabulary |
| Form action/method | Existing root-relative, browser same-origin and literal get/post | No widening, even with navigation/image origins configured |
| img | Rejected | ImagePolicy other than Disabled admits img with required src and alt, no children |
| Image src | No image capability | Root-relative same-origin path; HttpsOrigins additionally admits absolute HTTPS at named image origins |

Keep existing control/whitespace/backslash/protocol-relative rejection. An
explicit HTTPS spelling is not a URL parser; only browser parsing decides its
meaning, credentials and origin. Reject credentials, unselected schemes and
cross-origin root/fragment resolution caused by a foreign base. Absolute URLs
selected through an origin list remain absolute; all accepted href/action/src
destinations are frozen before element construction, so later base changes do
not retarget attributes. Existing URL-free default Views retain their previous
ability to mount without an HTTP(S) navigation context.

For anchors, allow target only `_self`/`_blank`; `_blank` requires an explicit
rel token noopener (noreferrer alone is not the selected authoring spelling).
Do not silently rewrite the author's rel value. Allowed ASCII-insensitive rel
tokens are noopener, noreferrer, nofollow and external; empty rel is harmless.
Reject opener and resource-hint/unknown tokens. Referrerpolicy, when explicitly
selected, is no-referrer or same-origin; hreflang is inert literal metadata.
Ping/download and named target browsing contexts remain deferred. Existing
anchors without these attributes keep their current behavior.

For img, require src and an alt attribute; empty alt is valid decoration.
Admit nonnegative integer width/height, loading=eager|lazy,
decoding=auto|sync|async, fetchpriority=auto|high|low, and the selected referrerpolicy
values. Ordinary existing global attributes remain available. Srcset/sizes,
picture/source, crossorigin, ismap/usemap, event-handler attributes and inline
CSS are not selected. Loading/decoding hints use native browser behavior; the
framework does not await decoding, guarantee display or introduce retries.

## Ordered preflight and failure semantics

1. Perform full structural/attribute/spelling validation without network or DOM
   mutation. Invalid later markup cannot create an earlier image.
2. Parse both option-origin lists; resolve/check every href/action/src against
   the document and effective base before creating any node or assigning any
   resource-bearing attribute. Reject any invalid/denied result and retain the
   old target. No resource requests occur during this phase.
3. Construct the detached replacement with those frozen destinations. Apply
   non-resource image attributes before src. Accepted image loading may start
   before insertion; failure after admission is not rollback of network effects.
4. Clear/append only after construction succeeds. Final browser DOM errors keep
   the existing nontransactional limit; no stronger rollback claim is made.

An accepted image may send browser-managed credentials/referrer under normal
browser rules. CSP can further restrict it; allowlisting is not authentication,
privacy isolation, redirect-chain enforcement or SSRF protection for a server.
This policy controls assigned URLs and their initial parsed origin. Native HTTP
redirects remain browser-owned, subject to CSP, and are not claimed to preserve
the initial origin allowlist across the redirect chain. Application endpoints
still own authorization, safe GET semantics and asset content. Failure status
messages do not echo configured origins/credentials/raw URLs.

No server settings/secrets enter browser Views. The optional application Settings
loader may map public settings into typed options explicitly. Native types and
validation introduce no mandatory URL-parser/HTTP/Settings dependency; browser
origin checks reuse existing web-sys URL support. No global policy registry or
background worker is added.

## Minimal consumer and new verification

Add one explicit opt-in surface to an existing sample, with local/external link
and image selection owned by that application. Keep existing consumers on
default policies; the new sample activation must not enable global policy.
Document the APIs with a minimal typed-options example, unreleased-source
availability and the distinction between static and browser acceptance.

Only add checks for changed behavior: default versus selected admission; required
image attributes/values/void shape; independent link/image/form rules; malformed
origin configuration, credentials, deceptive URLs, effective-base changes and
frozen destinations; noopener/token policy; allowed same-origin and isolated
HTTPS-origin image loads and a native image failure; invalid late markup/URL/
option producing zero denied requests and retaining the old target.

Reuse existing browser fixtures/ownership techniques, with test-local HTTPS
certificates and isolated browser trust scoped to those sessions. No system or
user trust-store modification, external service, test-framework expansion or
TLS/deployment qualification is selected. Retain exact fixture/requests/error/
source/cleanup receipts and first failures. Supplement native/Wasm compilation,
focused tests/Clippy/rustdoc/docs and actual Chrome/Firefox new cases; do not
replay YT-220 or earlier passed suites. Clean owned servers/browsers/profiles.

Contract review must pass before implementation. Final independent review must
pass before marking YT-221 Done. New work remains uncommitted/unpushed; no
package/version, public site, tag, hosted workflow, signing or publication change.

Contract independent review: **PASS0**, zero issues/followups, four input hashes
stable and registered reviewer model/effective high effort verified. The first
P2 for incomplete getter/builder types is retained; exact signatures above close
that gap. Implementation may proceed under the existing owner selection; final
implementation review is still required before Done.

## Implemented slice and compatibility

The implementation is in [DOM mounting](../crates/yetlet-browser/src/dom/mod.rs),
[typed options](../crates/yetlet-browser/src/dom/options.rs) and
[focused tests](../crates/yetlet-browser/src/dom/tests_yt221.rs).
The existing [Todo View source](../examples/todo-ui/src/controls.rs) adds
controlled_links_and_image as an explicit helper, not a default route installation.
Its native rendering/validation and actual Wasm consumer use the same helper.
The [DOM guide](../crates/yetlet-browser/guides/dom.md#explicit-links-and-images-unreleased-source)
and [compatibility note](COMPATIBILITY.md) distinguish these unreleased APIs
from the published SDK0.9.0-alpha.5.

Existing validate/mount delegate to default options; DomPolicyError variants,
public View/Element, SSR, native controls, default89-tag vocabulary and feature/
dependency graphs are unchanged. The only additional element is opt-in img.
Origin parsing reuses browser web-sys::Url; std-only native options add no parser.
No manifests, lockfiles, loader, CSP producer, Settings integration or keyed DOM
API changes are included. Host CSP and redirect-chain behavior remain independent.

## New-behavior evidence — 2026-10-31

Run-local evidence is retained at `target/yt221-20261031/`, with the exact native/
Wasm fixture, sources, screenshots and logs saved outside the repository under
`${XDG_DATA_HOME:-$HOME/.local/share}/yetlet-release-evidence/runs/yt221-20261031/`.
The new five native cases and consumer case are selected by the yt221 test
filter, while the single new DomOptions doctest is selected separately.
Earlier YT-220 suites and other accepted platform/release evidence are reused,
not replayed or relabeled as current-source qualification.

| Evidence | Actual result and limit |
|---|---|
| Native options/policy tests |5 passed,13 earlier tests filtered out; defaults, independent policies, provisional origin boundary, finite anchor/image attributes, shapes and unchanged forms |
| DomOptions doctest |1 passed,1 earlier doctest filtered out; public typed construction |
| Todo consumer test |1 passed,13 earlier tests filtered out; explicit helper, escaping and separation from existing controls |
| Native/Wasm lint and format | Strict scoped browser-library Clippy on native and wasm32 PASS; scoped SDK formatting PASS |
| Consumer builds | Native actual helper rendered/validated; real wasm32 fixture built offline/locked and matching wasm-bindgen0.2.129 glue generated |
| Chrome156.0.8078.73 |16 default/configuration/URL/credential/form/base/late-node rejection cases before any element creation; target identity retained and URL errors redacted. Allowed link popup and local/remote images, frozen destinations, configuration spelling and native404 image behavior PASS |
| Firefox159.0 | Same16 preflight cases PASS in first execution; retained as evidence. Corrected continuation completes actual popup, local/remote images, frozen URLs, configuration and native404 failure; no replay of passed16 cases |
| HTTP/resource observations |32 requests across three owned origins; observer C and denied paths receive zero requests. Popup requests omit Referer; one expected missing-image request per completed browser case, no framework retry |
| CSP and screenshots | No captured unexpected app exception/rejection/CSP violation; the admitted404 is expected native image failure. Chrome desktop/mobile and Firefox desktop inspected; Chrome emulated narrow viewport has no horizontal overflow. Not physical-mobile or accessibility certification |
| Cleanup | Both Firefox processes/profiles and Chrome context/browser closed; three loopback TLS servers stopped and ports closed. Ephemeral private keys deleted; no system/user trust-store change |

The fixture uses three loopback HTTPS origins and the real selected sample/Wasm
module. Chrome trusts only the exact ephemeral leaf SPKI in its isolated process;
Firefox trusts the ephemeral CA in its disposable profile with
acceptInsecureCerts=false. The test-only image CSP includes observer C so CSP does
not conceal a renderer denial. These changes do not alter production CSP or
qualify deployment TLS. Browser request observations cover the owned fixture,
not all browser networking or third-party privacy behavior.

### Retained failures and bounded continuation

1. The first contract review returned P2 because getter/builder signatures were
   incomplete. Only the exact public signature table was corrected; independent
   followup PASS0 preceded implementation. Both verdicts remain recorded.
2. Firefox's first popup check read the newly selected window URL before
   navigation completed. Its16 preflight cases already passed. This was a harness
   timing failure, not evidence of a renderer defect. One in-place navigation
   wait correction completed only the unfinished portion in a fresh owned
   browser; the original script, failure, cleanup and logs are retained.
   Recreating the valid local-image state is setup, not a framework retry.

No old native/browser suite was replayed. No Windows, GPU, browser-history,
redirect-chain, hosted, signing, publication or live-site qualification is claimed.

## Final independent acceptance

**PASS0**, zero issues/required followups. The independent reviewer inspected all14
changed paths, actual product/fixture code, retained native/browser evidence,
compatibility and cleanup. All23 frozen input hashes match; the two announced
Backlog/Roadmap status corrections were explicitly included in review. Registered
role/model gpt-6-astra and effective high effort are verified. No passed tests or
browser runs were replayed by the reviewer. Its verdict permits recording Done;
subsequent edits only record this acceptance and next-item status.

YT-221 is Done at the selected unreleased-source scope. New14 paths remain
unstaged/uncommitted/unpushed, the existing SDK/site stay unchanged, and original
contract/browser failures remain in the retained receipts. YT-222 is the next
unblocked contract item; YT-222–224 remain Todo and are not implemented here.
