# YT-094 — reference application closeout

Date: 2026-10-19. Implementation baseline:
`2a7202db2fea12f498b048b9be9f0521d848dcb8`.

**Done at the selected reference-application scope.** The owner requested an
existing-evidence reconciliation, explicit accepted limits, independent review
and a local commit. This closeout changes documentation only. It does not rerun
passed application tests, browser journeys, containers or external services.

## Completion conditions and existing evidence

The original [YT-094 acceptance criteria](BACKLOG.md) and the subsequently
selected Rust/WASM, OIDC and shared-storage slices have the following evidence.
Each result belongs to the source, platform and procedure recorded in its linked
report; this table does not turn earlier results into a fresh cross-platform
run of the baseline above.

| Condition | Accepted evidence and result |
| --- | --- |
| Existing CLI, settings and application composition | The [Admin guide](../examples/admin-dashboard/README.md) records `yetlet new` with Vite/Tailwind/service/settings, explicit reuse of the Board library, CLI build/run/stop and selectable frontend builds. No duplicate authorization/storage layer or mandatory framework dependency was introduced |
| Tailwind/Vite and meaningful no-JS SSR | The first-slice Linux Chrome journey passes native login, list/filter/pagination, empty/detail/edit/history, validation and conflict screens. Ordinary pages retain meaningful HTML without browser code |
| Typed forms, permissions and audit history | Native and actual HTTP/browser checks cover resource/role denial, CSRF, escaped content, preserved 422/409 drafts, persisted writes and transactional actor/history records; see the Admin guide and [OIDC evidence](YT-094-OIDC.md) |
| Responsive navigation, keyboard and CJK input | Linux desktop 1440×1000 and mobile 390×844 checks cover layout, focus and browser-protocol composition. The [Windows WASM report](YT-094-WINDOWS-WASM-VERIFICATION.md) adds actual browser interaction and returned operator-attested physical Bopomofo, keyboard, Narrator and save/reload journeys, with the evidence-strength limits below |
| Highly interactive Rust/WASM with SSR fallback | The [triage slice](YT-094-WASM-TRIAGE.md) passes local multi-draft/filter/reorder behavior without HTTP, explicit saves, pending edits, 409/422 draft retention and disposal checks. Windows follows at its recorded source; the corrected queued-save finding and passing follow-up review remain visible |
| Real external identity integration | The [opt-in OIDC slice](YT-094-OIDC.md) passes actual Linux IdP token exchange and verified identity, unmapped/replayed login denial, explicit project permissions, persisted edits/history, CSRF denial and local logout with JavaScript disabled. Independent code and returned-evidence reviews pass |
| Persistence and session failure behavior | Initial storage outage/recovery, restart persistence and session retirement pass. [Valkey sessions](YT-094-VALKEY-SESSIONS.md) pass actual two-app login/logout, app restart, bounded fail-closed pause/stop and recovery. The configurable absolute session lifetime defaults to 1800 seconds; its focused follow-up passes |
| Shared business storage | The [shared-SQLite slice](YT-094-SHARED-SQLITE.md) passes two independent app connections and two actual containers using one local volume: cross-instance reads/updates, stale/concurrent conflicts, busy recovery, restart and shared logout. This extends the initial fixture's replica-local databases |
| Bounded workload and cleanup | The first slice records 40 authenticated reads at concurrency 8. CLI/process, Windows console, browser and Compose reports record their own stop/restart and owned-resource cleanup. These are bounded application checks, without a throughput or availability commitment |
| Independent review | The linked slice reports retain their independent PASS results and material fixes. Windows returned evidence has a separate source-side reconciliation PASS; the earlier Windows reviewer failure and routing mismatch are preserved rather than relabeled |

## Accepted limitations and exclusions

- **Evidence scope:** historical checks retain their own source identities.
  Selected source hashes are not a complete reproducible-build proof. Earlier
  dated statements that YT-094 was In progress remain historical; this document
  supersedes their overall task status, without changing any result or limit.
- **Windows physical input and assistive technology:** physical IME and spoken
  Narrator results rely on explicit operator attestation. Returned page state
  corroborates the saved text; screenshots corroborate drain and exit 0. There
  is no independent physical-keystroke observation or recorded spoken-output
  transcript. Fourteen returned evidence hashes and 310 selected source hashes
  matched. The original missing-exit review failure and configured/observed-model
  mismatch remain disclosed; the separate reconciliation review does not repair
  that historical routing claim.
- **Owner-selected platform boundary:** separate Windows OIDC verification was
  waived and the dedicated test client/account were deleted. Routine repeated
  IME/assistive-technology matrices are not required; focused Ubuntu checks may
  accompany relevant input/accessibility changes. No general all-OS support
  claim follows from these examples.
- **Authentication:** OIDC protocol/cryptographic behavior belongs to the
  maintained client library; evidence covers the selected application wiring.
  Global IdP logout, refresh/revocation, public TLS/proxy deployment and production
  authorization qualification were not selected. Real OIDC callback completion
  across Valkey replicas was not exercised; pending-state encoding has native
  coverage. Credentials, real provider identifiers and private browser evidence
  remain outside tracked files; no test identity is recreated for this closeout.
- **Sessions and configuration:** Memory and terminal-code login remain the
  defaults. Valkey and OIDC require explicit opt-in settings/features. Valkey
  outages return 503 without Memory fallback. Session expiry is absolute, not
  sliding; settings/secrets are loaded at startup and existing clients retain
  their copied values. The adjacent [settings/SecretProvider integration](SETTINGS-SECRET-PROVIDER.md)
  does not introduce automatic reload or rotation.
- **Storage and deployment:** Compose evidence is single-host, with a local
  SQLite volume and a single nonpersistent Valkey primary. TLS/ACL hardening,
  backup/restore, cross-host storage, HA, rolling schema/first-run migration
  qualification and distributed event delivery are outside this closeout.
  SQLite engine ACID, durability and performance are not being recertified.
  `/healthz` is stateless and `/readyz` checks SQLite, not every optional service.
  The shared-SQLite journey is HTTP evidence, not a new rendered-browser run.
- **Frontend and lifecycle:** authenticated Vite dev bridging is not selected;
  the Admin serves built assets. Triage drafts are tab-local, with no offline
  persistence or automatic conflict merge. Disposal suppresses future client
  work but cannot roll back an already admitted server operation. Linux
  cancellation exit 1 and Windows console exit 0 retain their documented
  platform/procedure boundaries. Service-generator delivery of the edited Admin
  layout is not qualified by these local checks.

These are accepted boundaries of the selected reference application, not
remaining YT-094 blockers. Public release and production qualification remain
separate: YT-020 stays Blocked and the M20 release gates are unchanged. M19 as a
whole remains partial.

## Documentation closeout and next work

Only documentation/link/structure checks and the required staged Commit Guard
are run for this closeout. Existing passing runtime evidence is reused as
requested. Independent Reviewer `/root/yt094_closeout_review_oct19` returned PASS
with zero issues or required followups after inspecting all six documentation
targets, linked acceptance reports and selected SQLite, Valkey/TTL and Windows
reconciliation receipts. The review was read-only and repeated no runtime checks.

[YT-104](BACKLOG.md) is the next independently unblockable discussion: a small,
opt-in browser-only Rust/WASM starter with settings-first CLI creation/build/
run/stop and static output, without a native Yetlet page server. The existing
[API observations](YT-094-API-OBSERVATIONS.md) feed separate YT-105 authoring and
YT-106 third-party component research. None is a prerequisite for this Done
status; no implementation of those tasks is authorized by this closeout.
