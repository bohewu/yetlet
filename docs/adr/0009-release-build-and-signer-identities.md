# ADR 0009 — Separate Release Build and Signer Identities

- **Status:** Accepted design only; implementation, setup and execution Pending.
- **Date:** 2026-10-28.
- **Task:** YT-096; D1/D3 amendment. D6 Pending; YT-020 Blocked.
- **Related:** [Signer contract](../YT-096-SIGNER-CONTRACT.md),
  [D1–D5 policy](../YT-096-D1-D5-RELEASE-TRUST-PLAN.md),
  [CI integration](../YT-096-RELEASE-CI-INTEGRATION.md),
  [hosted checkpoint](../YT-096-HOSTED-DRILL-PLAN.md).

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
