# YT-094 — opt-in OIDC application slice

Status: implementation, focused native checks, independent code review and the
bounded Linux real-provider token/access/local-logout workflow pass. Independent
returned-evidence review PASS has zero issues or required followups. Existing SSR,
Rust/WASM and returned Windows evidence retain their original source boundaries.
The owner waived separate Windows OIDC verification; it was not run. The
reference does not establish production authentication/deployment support.

## Selected contract

The Admin reference explicitly enables the Board's optional `oidc` Cargo feature.
The ordinary Board and browser dependency graphs exclude the OIDC client.
Runtime `oidc.enabled=false` retains the terminal-code demonstration; `true`
installs OIDC login and disables terminal-code authentication. There is no core
authentication module, DI container or provider registry.

`openidconnect` 4.0.1 owns Discovery, Authorization Code with S256 PKCE, ID Token
signature/issuer/audience/expiry/nonce verification and optional access-token-hash
verification. Only `openid` is requested. The backchannel retains HTTPS
verification, disables redirects and uses five-second connect/fifteen-second
request timeouts. Discovery, keys and credentials are startup snapshots; changes
require restarting this consumer. Automatic refresh, IdP logout, introspection,
revocation, PAR, provisioning and multi-provider selection are not selected.

Native `POST /board/oidc/login` requires the existing session CSRF token,
rotates the session and installs fresh state/nonce/PKCE material valid for five
minutes. Only the selected OIDC login page adds the discovered authorization
origin to `form-action`. Other CSP directives and pages remain unchanged. This
permits the native form's redirect without arbitrary form destinations or JS.

`GET /board/oidc/callback` atomically consumes the pending transaction before
token exchange. Missing, mismatched, expired, rejected and replayed transactions
cannot sign in. A returned issuer, when present, must match the configured
issuer. Errors do not echo code, state, tokens, nonce, verifier, provider
descriptions or credentials. Callback responses use `no-referrer` and
`no-store`. Logs contain fixed failure-stage categories and numeric HTTP status.

Authentication grants no permissions by itself. Verified issuer/subject must
match an explicit setting for existing `alice`, `bob` or `viewer` permissions.
Username, email and provider roles do not assign access. An unassigned account
receives a native 403 page with its own escaped account reference and no data
access. An empty map grants nobody access. Assigned login rotates the existing
session and CSRF token again. Project/write checks, audit history and
CSRF-protected local logout remain authoritative; local logout does not end the
IdP's SSO session.

## Dependency audit applicability — 2026-10-21

The first [PR #10 CI run](https://github.com/bohewu/yetlet/actions/runs/39532401659)
failed its full-lockfile RustSec gate. The root workspace, standalone Admin and
xtask lockfiles now select rustls-webpki 0.103.16 and webpki-roots 1.0.10, fixing
[RUSTSEC-2026-0382](https://rustsec.org/advisories/RUSTSEC-2026-0382.html) and
[RUSTSEC-2026-0381](https://rustsec.org/advisories/RUSTSEC-2026-0381.html).
HTTPS certificate verification remains enabled; no trust bypass is introduced.
The adjacent xtask lockfile also advances rustls from 0.23.45 to 0.23.46,
removing its RUSTSEC-2026-0346 through RUSTSEC-2026-0349 findings.

The optional openidconnect 4.0.1 dependency also includes rsa 0.9.10, reported by
[RUSTSEC-2023-0071](https://rustsec.org/advisories/RUSTSEC-2023-0071.html). That
advisory concerns private-key disclosure through observable timing and currently
has no patched version. The application is an OIDC client: Oidc::finish calls
ID Token claims verification with the client's verifier. The pinned dependency's
RSA verification path constructs RsaPublicKey from the issuer's public n/e and
calls verify. This consumer does not construct RSA private keys, sign tokens or
decrypt RSA ciphertext. Client-secret authentication is not an RSA private-key
operation. The separate local signing provider uses ES256, not this RSA path.

The single-ID exception in [.cargo/audit.toml](../.cargo/audit.toml) records this
bounded applicability decision. It does not patch rsa, certify arbitrary uses of
openidconnect, disable auditing of other advisories or change the historical
YT-096 package-audit results. Reassess the exception before changing the OIDC/RSA
dependency or consumer, especially before introducing private-key operations.

Local follow-up passes the 25 Board tests with oidc enabled, 17 Admin tests,
documentation validation, and root/Admin RustSec audits with only that
single-ID exception. The supplemental xtask audit reports no vulnerabilities
and, at d963494, retained the pre-existing xml-rs 0.8.19 yanked-version warning
without silencing it. These results do not rerun or expand the YT-096 candidate
gates.

The owner subsequently selected removal of that warning. The xtask manifest and
lockfile now pin [xml-rs 0.8.29](https://docs.rs/crate/xml-rs/0.8.29), a non-yanked
0.8 patch with the same MIT license and no additional dependencies. The existing
SVG/CSS safety and exact-output tests pass within the full xtask test suite
(141 passed, two pre-existing ignored). The xtask RustSec audit with --deny
warnings exits 0 with zero vulnerabilities and zero warnings. Historical
YT-084/YT-085 cached-dependency limitations remain unchanged as source-bound
records; this update does not modify application/library dependencies or assets.

## Existing settings and secret-provider composition

Real provider/client settings, subjects, test credentials, authorization/callback
URLs, logs and browser evidence remain outside the Git working tree. These are
synthetic examples only:

```toml
[oidc]
enabled = true
issuer = "https://identity.example/"
client_id = "admin-development"
client_secret = ""

[oidc.subjects]
alice = "verified-subject-for-project-one-operator"
bob = ""
viewer = ""
```

Use a private TOML overlay with declared keys. Explicitly select its absolute
path with the application's `YETLET_DEVELOPMENT_SECRET_FILE` and select
`APP_ENVIRONMENT=Development`. The existing `yetlet-config::Loader` preserves
base/profile, Development overlay, then OS override precedence. Staging and
Production do not read the Development overlay. No home-directory discovery or
second application configuration format is added.

Admin selects only loaded `oidc.client_secret` for `SettingsSecrets` under the
application-owned name `oidc-client-secret`. Its startup constructor accepts
`Arc<dyn SecretProvider>`, resolves once and supplies a protected credential to
the Board. Other providers can be injected without changing the read contract.
Browser builds and hydration state receive no secret, token or provider handle.
Test-account credentials belong to a separate private browser-test file, not
application settings.

The callback follows the direct listener:
`http://127.0.0.1:<port>/board/oidc/callback`, registered exactly. This preserves
the reference's loopback-only HTTP boundary; it does not add public deployment,
proxy-origin inference or a new development-TLS mechanism.

## Validation boundary

Focused checks cover overlay precedence/isolation, injected-provider failure,
explicit mapping, fresh transaction material, expiry, CSRF/session rotation,
demo-login rejection, invalid/replayed callbacks and CSP scope. Ordinary Board
tests remain default-feature compatibility evidence. Browser dependency inspection
must exclude OIDC, secrets and the HTTP client.

Real-provider tests use CLI build/run/stop, a disposable database and an isolated
browser. Detailed evidence stays in a private run directory outside the repo.
Login and consent alone do not qualify token verification or assigned access.
Preserve failed observations separately; complete the real callback, explicit
mapping, local workflow/logout, cleanup and independent review before claiming
this slice passed.

### Initial local working-tree checks — 2026-10-19

| Check | Result and scope |
| --- | --- |
| Board, optional OIDC enabled | `cargo test --offline --locked -p yetlet-lab-operations-board --features oidc --all-targets`: 22 passed |
| Board, default features | `cargo test --offline --locked -p yetlet-lab-operations-board --all-targets`: 17 passed |
| Admin | `cargo test --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --all-targets`: 16 passed |
| Strict Clippy | Board with and without OIDC, plus Admin, all targets, `-- -D warnings`: passed |
| Browser graph | `cargo tree` for the Admin browser manifest excludes `openidconnect`, `reqwest` and `yetlet-secrets` |
| CLI lifecycle | Actual `yetlet build` and `yetlet run` reached the loopback listener. Ctrl-C drained the app and closed the listener; native Linux CLI cancellation exited 1 |
| Native browser login | JS-disabled sign-in renders; real-provider login and the requested consent page were reached |
| Initial real callback | Reached the app, but token exchange returned an unusable response. No verified subject or assigned access was established in that attempt |
| Documentation and whitespace | `cargo xtask check docs` and focused whitespace checks passed |
| Independent code review | `/root/yt094_oidc_code_review_oct19`: PASS, zero issues or required code followups, including final cfg compatibility confirmation |

A separate diagnostic token request with an invalid code and a loopback
`redirect_uri` returned HTTP 403 with a non-JSON body. This historical observation
does not prove the exact rule. The operator subsequently reported completing the
WAF adjustment. Required redirect parameters and TLS verification remained
enabled. Private run evidence retains the initial failure separately.

These checks cover the edited Linux source, not the earlier Windows snapshot.

The isolated browser and CLI-owned application were stopped; the loopback
listener was confirmed closed. Private credentials and disposable run evidence
remain outside the repo. No commit or external configuration change was made by
this slice. The initial code-review PASS alone did not close the real-provider
gate.

### Real-provider follow-up after operator WAF adjustment — 2026-10-19

The existing CLI rebuilt the edited source and ran the same isolated application
with a disposable SQLite database. System Chrome used bundled Playwright because
the Browser plugin was not available. JavaScript was disabled throughout the
1366 × 900 browser flow, including provider login and consent. Only `openid` was
requested; the client and S256 challenge were checked before accepting consent.
Real configuration, verified subjects, account credentials and callback URLs
were held outside the working tree and removed after verification. Retained
browser evidence is anonymized.

| Check | Observed result |
| --- | --- |
| Actual token exchange | HTTP 200 in both unassigned and mapped runs; the client verified the ID Token before returning a subject or assigning access |
| No implicit permissions | Empty subject mapping returned the native unassigned 403 page; protected Project 1 data still returned 401 |
| Callback replay and response isolation | Replaying the consumed callback returned 401 with `no-referrer` and `no-store` |
| Explicit settings mapping | The verified test-account subject was mapped only to the existing Project 1 operator in private test settings; restarting the app applied it |
| Assigned login | Native Project 1 list rendered after successful token exchange; session cookie and CSRF token rotated; the old anonymous cookie returned 401 |
| Resource permissions | Project 2 listing, create form and POST each returned 403 |
| Native persisted workflow | Created a Project 1 work item with selected priority/assignee, then changed its title/status to version 2; two audit rows and local actor were visible; reload retained the update |
| Local logout | Incorrect CSRF returned 403 and preserved the session. Native sign-out returned to login; current browser and the old authenticated cookie both received 401 for protected data |
| Rendered browser checks | Correct page titles, meaningful content, no framework overlay, no page errors and no CSP errors. Three HTTP error console messages corresponded to deliberate denial/replay navigations. Private screenshots record login, assigned list, persisted detail/history and logout |
| CLI and cleanup | Build exited 0; both CLI-run instances stopped through Ctrl-C with database-drain and owned-child cleanup markers, native cancellation exit 1. Browser closed; listener confirmed closed |
| Source identity | All 13 selected implementation/configuration/lock inputs matched the source hashes retained with the earlier code review |
| Independent evidence review | `/root/yt094_oidc_code_review_oct19`: PASS, zero issues or required followups; independently checked all 13 source hashes, four screenshot hashes, build log and screenshots |

This closes the requested real-provider token, explicit access and local-logout
workflow at the stated Linux loopback scope. No runtime implementation repair
was needed after the WAF adjustment.
Global IdP logout, refresh/revocation, Windows OIDC, public TLS/proxy deployment
and production authentication remain outside this slice. YT-094 remains In
progress for its broader application/deployment gates; prior passing SSR/WASM and
Windows workloads are not repeated or relabeled as coverage of this OIDC source.

After evidence review, the dedicated Development-secret and browser-test-account
files, private environment, raw Discovery/token probe responses and disposable
database were removed. The browser runtime was reset. Checks found zero real
provider/client/account/subject/credential matches in Git-visible files or the
retained run evidence. Only anonymized observations, source hashes, safe local
application screenshots and the build log are retained. The external test client
and account were not deleted by the agent; their removal remains an operator
action. No commit, push or external configuration change was made.

## Verification ownership and owner-selected scope

The maintained `openidconnect` dependency owns its documented protocol,
cryptographic and platform behavior. This reference verifies the selected
application wiring: settings, redirects/CSP, session/CSRF, permissions and local
logout. It does not repeat the dependency's full protocol or per-platform matrix
and does not expose every upstream feature as a configured application flow.
The owner explicitly declined additional Windows OIDC verification and removed
the dedicated client/account. Preserve the accepted Linux proof; do not request
new credentials to reopen that waived check.
