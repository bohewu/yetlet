# M9h Verification — Security-Bounded Auth Notes

**Date:** 2026-10-10
**Task:** YT-057 / M9h
**Status:** Done. Complete validation and security-focused reviewer `wc_job_EgSLng_VafIvs3ju` PASS; exact catalog entry promoted.

## Chosen authentication strategy

The sample deliberately avoids password handling. At startup the application generates separate OS-random 256-bit synthetic login codes for Alice and Bob and prints them only to the local terminal. Tests/smoke may inject two deterministic valid codes through explicit environment variables. The application never embeds, logs, SSR-renders or serializes the codes into browser assets.

Authentication compares a fixed 64-byte shape in constant time, including a dummy expected value for unknown usernames. This is a loopback development fixture, not an OIDC, password, passkey, MFA, registration or recovery design. It has no production brute-force/rate-limit claim.

## Authentication, Session, authorization and storage boundaries

- `DemoIdentities` authenticates a synthetic identity by code possession.
- `yetlet-session` independently owns opaque cookie identity, absolute expiry, bounded memory and CSRF. Secure is the provider default; this HTTP sample calls the explicit insecure-localhost override only after rejecting every non-loopback listener.
- Login verifies the anonymous CSRF token and uses `replace_and_rotate` to change Session ID, CSRF and state. The old anonymous cookie/token cannot authenticate or mutate.
- Logout verifies authenticated CSRF and rotates to a fresh anonymous Session. The old authenticated cookie is retired.
- Each note handler independently derives the current user from server-side Session state and checks the server-owned note owner. IDs and form fields never grant ownership.
- Notes use a bounded process-local map separate from Session. This is not persistence, an ORM, a repository framework or a shared authentication provider.

## Focused evidence

The first strict Clippy pass found only unused imports/methods before tests ran; they were removed. The initial 10-test run then found one assertion that expected a raw apostrophe even though the shared renderer correctly emitted `&#39;`; the assertion was corrected to verify escaping. A new oversized-form test exposed that the sample collapsed Axum `BytesRejection` into HTTP 400; `parse_form` now preserves HTTP 413 instead of weakening the test.

Current focused **12/12 tests PASS** and cover:

- code shape, distinctness, redacted Debug and unknown-user dummy comparison;
- Secure Session default and loopback-only insecure HTTP exception;
- health outside Session, strict script-free login SSR and no code leakage;
- invalid credentials/CSRF without login or rotation;
- successful login rotation, old cookie retirement and old-CSRF rejection;
- per-user note listing, escaped hostile content and 403 IDOR read/update/delete;
- CSRF on create/update/delete/logout with no partial mutation;
- logout rotation and old authenticated-cookie retirement;
- unknown client Session IDs not adopted, clear 401/403/404/405 behavior;
- duplicate/unknown/oversized forms fail closed;
- per-user note capacity without partial creation.

A real localhost smoke uses two cookie jars and deterministic environment codes. It verifies invalid login, Alice login/session rotation, old CSRF rejection, escaped note creation, Bob list isolation and 403 cross-user access, invalid-CSRF immutability, logout rotation, old/new anonymous access denial and absence of synthetic codes from HTTP and server logs. The owned process is stopped by SIGINT.

## Acceptance commands

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --all-targets --locked
cargo test --doc --workspace --locked
cargo build --release --locked -p yetlet-auth-notes
python3 scripts/smoke_auth_notes.py
cargo tree --locked -p yetlet-auth-notes --edges normal
python3 scripts/check_samples.py
python3 scripts/test_check_samples.py
python3 scripts/check_docs.py
git diff --check
```

Complete evidence must prove application-only Auth/Session dependencies, unchanged production browser assets, no secret occurrence outside synthetic tests/smoke, exact runnable catalog stanza and process/Git cleanup. Keep the sample outside the verified catalog until an independent read-only security-focused reviewer PASS. Do not begin YT-058 before YT-057 is Done.

## Complete validation freeze

The full matrix passed after focused testing. Retained logs, exact source/dist hashes, all package trees, secret-occurrence results, candidate catalog stanza and summary are under Git-ignored `target/m9h-evidence/`.

| Check | Observed result |
|---|---|
| Auth Notes focused tests | 12/12 PASS |
| Workspace native tests | 181/181 PASS |
| Workspace doctests | 5/5 PASS |
| Workspace format and strict Clippy | PASS |
| Auth Notes release build | PASS; 2,547,064 raw bytes |
| Two-user localhost authentication/authorization smoke | PASS; owned process reaped |
| Eight existing wasm32 packages | PASS |
| Production Browser/Search release rebuild | digest `faebb0709dccd863` |
| Production `dist/` | 136/136 files byte-identical |
| Existing catalog/checker | 10 entries and 4/4 tests PASS |
| Candidate Auth Notes catalog | Exact virtual 11-entry catalog PASS |
| Dependency isolation | All 27 workspace package normal trees PASS |
| Documentation/backlog | 49 required documents / 59 task IDs PASS |
| Git/worktree/whitespace | Original master HEAD, one worktree, empty index and untracked whitespace PASS |

`yetlet-auth-notes` directly selects `yetlet-session`, `getrandom`, `subtle`, the server adapter and renderer, and excludes browser/WASM/DOM/reactive, SQLite/SQLx and ORM dependencies. No unrelated package depends on Auth Notes.

The two deterministic 64-character fixtures occur literally only in the Rust test module. The Python smoke constructs them dynamically (`"a" * 64` / `"b" * 64`) rather than embedding literal values, and the optimized application binary contains neither fixture. The first evidence script incorrectly required literal occurrences in both files; its product/build/test gates had already passed. A recovery assertion recorded the truthful fixture shape, rechecked source/dist hashes, Git whitespace and final summary without changing runtime source.

Browser E2E was not added for this script-free SSR application: the real localhost HTTP smoke drives the exact cookie, form, redirect, rotation, CSRF and authorization paths, while shared Session/SSR behavior already has retained evidence. Complete cross-product browser regression remains M9j. This is an ordinary local security-focused evidence gate, not production penetration testing or formal assurance.

## Reviewer gate

Independent security-focused read-only reviewer Job `wc_job_EgSLng_VafIvs3ju` completed at terminal exit 0 with the configured `gpt-6-astra` model, high reasoning effort and read-only sandbox. It inspected authentication/session separation, login/logout fixation rotation, old-cookie/old-CSRF behavior, per-request authorization, IDOR, CSRF ordering, form status handling, secret handling, loopback-only insecure-cookie exception, dependency/asset isolation and the exact candidate catalog stanza. Result:

```json
{
  "overall_status": "pass",
  "issues": [],
  "required_followups": [],
  "optional_notes": [
    "Retained evidence checked successfully: 181 workspace tests plus 5 doctests, real two-user localhost smoke, all 27 normal dependency trees, eight wasm32 checks, and byte-identical production assets.",
    "The fixed test login codes appear only in the Rust test module; the Python smoke constructs fixtures dynamically; neither appears in the optimized application binary.",
    "This is a loopback prototype source/evidence review, not production penetration testing, formal assurance, or approval of a reusable Yetlet auth provider."
  ]
}
```

The Runner still reported no native coding-agent provider, so this was ordinary `codex exec <reviewer prompt>`, not `codex exec review`, native ACP/`agent_type` dispatch or formal assurance. The reviewer made no edits, stages, commits or pushes. Only after PASS was the exact reviewed stanza promoted to `examples/samples.toml`; runtime source stayed unchanged. YT-057 is locally Done and YT-058 remains Todo.
