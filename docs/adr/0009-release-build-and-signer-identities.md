# ADR 0009 — Separate Release Build and Signer Identities

- **Status:** Accepted design; local implementation independently reviewed PASS; setup and execution Pending.
- **Date:** 2026-10-28.
- **Task:** YT-096; D1/D3 amendment. D6 Pending; YT-020 Blocked.
- **Related:** [Signer contract](../YT-096-SIGNER-CONTRACT.md),
  [D1–D5 policy](../YT-096-D1-D5-RELEASE-TRUST-PLAN.md),
  [CI integration](../YT-096-RELEASE-CI-INTEGRATION.md),
  [hosted checkpoint](../YT-096-HOSTED-DRILL-PLAN.md).

The [later alpha integration](../YT-096-SIGNER-CONTRACT.md#local-workflow-implementation--2026-10-28)
applies this unchanged design to actual source `bf33638`, unsigned alpha tag and
retained build/verifier. S and execution authority remain Pending. The original
`0.1.0` context below keeps its historical source/run identities.

## Context

The owner-selected C `3eacfe31f5e3281411d3bc4653a6baccab668b30`, immutable
unsigned `v0.1.0`, build `40628943583` / attempt1 and verifier `40631789238` /
attempt1 already bind retained sixteen-package outputs and unsigned provenance.
That source's `release.yml` is build-only. A later signer cannot truthfully claim
the original workflow/source identity by adopting its certificate SAN.

## Decision

Preserve C, tag and exact product/provenance bytes. Use a separate future reviewed
full forty-hex signer commit S, different from C, containing proposed
`.github/workflows/release-sign.yml`. S remains Pending; documentation HEAD does
not select it. At execution, checkout/event/workflow SHA must all equal approved S.
Require OIDC issuer `https://token.actions.githubusercontent.com`, exact SAN
`https://github.com/bohewu/yetlet/.github/workflows/release-sign.yml@refs/heads/master`
and cryptographically authenticated certificate/workflow digest S. Preserve
repository ID1409587171 and owner ID35250002. A protected branch or SAN alone
does not bind the reviewed signer bytes; check the Environment OIDC subject
separately. Product provenance continues to describe C and its original build.

The proposed protected `release-sign` Environment uses exact `master` branch
policy, reviewer `bohewu`, prevent self-review `false` and admin bypass `false`.
Nothing is configured by this ADR. Existing verifier Environment/tag policy and
Secret remain unchanged; new secret placement/access requires separate custody
authority. Keep decryption isolated, manually approved and cleaned before OIDC.

Separate closed `prepare` and `sign` modes. Preparation has read-only permissions
and no OIDC/signing authority. It verifies original retained inputs, creates a
complete allowlisted final assembly within existing bounds, freezes its manifest
and emits encrypted bytes plus a safe external receipt. Later D6 approval must
bind S, exact identities and both raw payload hashes. Signing adds OIDC permission
only in its protected job, retrieves the same frozen assembly and verifies hashes
before issuance. No rebuild, reassembly or provenance rewriting follows approval.

Sign the unchanged original public provenance with supplied-statement DSSE and
the final complete private manifest with a detached blob signature. The old
preparation manifest is an input with its own scope, not automatically the final
signing target. Keep receipt, both bundles and final approval outside the frozen
manifest to avoid cycles. The signer contract specifies scope and SHA acquisition.

## Alternatives and consequences

Keeping strict build/sign co-location would require a new candidate/tag and fresh
build/custody, without moving `v0.1.0`; that alternative is not selected. Reusing
the original SAN for different signer code, or checking only a mutable branch,
would obscure the actual authority and is rejected.

Separate identities preserve accepted bytes and avoid unnecessary rebuilds, but
require verification of both original builder C and approved signer S. Final
evidence closure may exceed existing bounds; stop for disposition rather than
silently omit proof or expand limits. D2 disclosure/custody limits and all original
failure labels remain unchanged. No SLSA level or production qualification follows.

## Acceptance boundary

This accepts the design only. Independent document review, signer implementation,
tool/root bootstrap and compatibility, Environment/custody setup, payload freeze,
exact D6 authority, OIDC/signing and publication remain separate gates. No workflow,
certificate, signature, hosted execution, Commit or Push is produced by this ADR.

## Local ciphertext handoff amendment — 2026-10-29

A separately owner-authorized additional run `40789535074/1` at selected S
`0e8fb60` fails at Draft metadata HTTP403 before protected work. The owner requests
a working data path while preserving the read-only signer and existing custody.
The [handoff contract](../YT-096-SIGNER-CONTRACT.md#encrypted-ciphertext-capability-handoff--2026-10-29)
uses an owner-local read to obtain a short-lived, specific-asset read capability,
verifies unauthenticated ciphertext retrieval, and encrypts the URL to the
existing age public recipient. CI receives no owner credential or plaintext URL.

The protected isolated phase now decrypts this bounded capability and performs
one restricted HTTPS ciphertext retrieval before private evidence decryption.
Host/path, no-redirect, expiry, ZIP and safe-receipt checks constrain that transfer.
Private-key access remains confined to this existing approved phase; no OIDC,
repository write, new Secret or changes to original C/tag/build are introduced.
Owner-generated input replaces direct CI Draft API access. Frozen signing bytes,
D6 approval and later signing/verification contracts remain unchanged.

This avoids a new upload workflow and does not grant write access to a key-bearing
job. Its tradeoff is a short operator-to-job time window: generate immediately
before dispatch, enforce stage-specific freshness and stop on expiry without
retry. Real local transfer/test-key proof and focused cases exist; independent
review PASS0 accepts the local implementation. This amendment records the local correction, not new Git,
source-S, hosted, formal-key, signing or publication authority. Original failures
and earlier independent results retain their historical scope.

## Single publication policy amendment — 2026-10-30

The owner selected assistant-operated build, preparation, signing and independent
verification, with one human approval for final combined GitHub/crates.io
publication. [The implementation and rollout contract](../RELEASE-PUBLICATION.md)
supersedes the earlier requirement for an owner click on each key-bearing job
for future executions using this reviewed policy. It does not rewrite the
historical manual approvals or completed release evidence above.

Keep immutable C, distinct S, both frozen payload hashes, public identity/log
disclosure, isolation/key cleanup before OIDC and a separate no-OIDC independent
verifier V. Standing automation authority is limited to the already reviewed
identity/disclosure scope; request data cannot change the committed policy.
The final publisher P has a separate owner-reviewed Environment and registry
token, no age identity or OIDC. It rechecks exact-source CI and original verifier
artifacts, publishes/readbacks GitHub first, then promotes identical registry
bytes. Partial publication is retained as incomplete without automatic retries.

This is local policy/code integration. Existing remote Environment reviewers
remain until separately delivered configuration rollout; no tag, hosted job,
signing or publication is executed by this amendment. YT-020 remains Blocked.
