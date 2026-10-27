# Developer integration guide template

Use this structure for a task-oriented integration guide. Keep the root README
as the product entry and link to guides for details. API inventories, naming and
compatibility policies, reference catalogs and dated evidence have their own
formats; they are not tutorials.

Begin with a short purpose and truthful availability/target scope. Then use these
five top-level headings in this order:

## Prerequisites

State where commands run, the required toolchain/target and explicit package or
feature selection. Identify prepared settings, providers and other inputs. Link
to exact tool setup, especially for browser builds. Explain when `--offline`
requires a populated Cargo cache. Do not assume an unpublished registry package
or install a tool/service implicitly.

## Minimal integration

Show the smallest existing runnable path or wiring snippet for the task, with
its complete source link. Label a setup fragment and identify application-owned
types/values it assumes; a Router fragment is not a running listener or complete
authentication product. Rustdoc context lines are not standalone Rust statements.
Keep required validation and safe defaults in the example. Prefer one clear path
before alternatives. Commands shown are instructions, not evidence of execution.

## Expected result

Describe observable success: URL/status, a fixed output, constructed value or
documented effect. For intentionally anonymous/disabled examples, state that
result explicitly. Separate output construction from delivery, and identity from
authorization. Do not infer performance, platform or deployment qualification.

## Configuration and errors

Explain settings/feature separation, defaults, lifecycle/ownership, relevant
limits and errors. Preserve failure/redaction/security and uncertain-outcome
semantics. Name the consumer that owns authorization, CSRF, secrets, persistence
and retries. Point to deeper subsections instead of silently weakening a contract
to shorten a guide. Retain useful existing subsection titles so anchors survive.

## API reference

Link to current package source/rustdoc guidance and relevant usage references
from the same revision. Avoid duplicating an exhaustive signature inventory.
Keep license/compatibility information available. Put dated validation/delivery
notes in an explicitly labeled historical section after current guidance; keep
their dates, source identities, original results and limits unchanged.

The [current API entry](API-REFERENCE.md) and
[publication scope](DOCUMENTATION-SCOPE.md) remain authoritative navigation and
source-selection rules. Applying this template does not publish a page, renew
its historical qualification or require new runtime tests.
