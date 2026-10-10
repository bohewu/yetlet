# M9e Data-access Comparison — Direct SQLite Options

**Date:** 2026-10-09
**Scope:** A bounded local spike supporting YT-054 dependency selection, not a universal database benchmark or API stability decision.

## Candidates

- `rusqlite 0.40.2`, `default-features = false`, `bundled`: direct synchronous SQLite statements and row closures.
- `sqlx 0.8.6`, `default-features = false`, features `runtime-tokio`, `sqlite`, `macros`: asynchronous pool/query API and derive-based row mapping.

At the time of this M9e comparison, the workspace declared an unverified MSRV of 1.85; current `sqlx 0.9.0` registry metadata declared Rust 1.94, so the bounded spike used `sqlx 0.8.6`. A later CI/security hardening round raised the workspace baseline to Rust 1.88 to consume the patched `time 0.3.47` transitive dependency. That later change does not alter this comparison's dependency choice or retroactively turn the spike into a broad MSRV claim.

## Measured fixture

Two disposable standalone crates under Git-ignored `target/m9e-spike/` each opened an in-memory SQLite database, created one table, inserted a parameterized row and mapped one row into Rust. Builds used the installed Rust 1.99 toolchain. Package count comes from `cargo metadata`; binary bytes are local unstripped release executables. Build time is a single machine observation and must not be treated as statistically comparable performance.

| Candidate | Resolved packages | Release binary | Initial release build | Clean offline release rebuild |
|---|---:|---:|---:|---:|
| `rusqlite` | 12 | 2,600,400 bytes | 28.800 s | 27.770 s |
| `sqlx` | 117 | 4,365,424 bytes | 31.837 s | 30.460 s |

The SQLx fixture resolved 109 packages not present in the rusqlite fixture. These results reflect the selected features and tiny fixture only; SQLx provides capabilities—async pooling, multiple databases, derives/macros—that this single-file SQLite sample does not need.

## Selection

M9e selects **application-local `rusqlite`** because:

1. the task requires one embedded SQLite file, explicit SQL, typed row mapping and transaction behavior—not a multi-database abstraction;
2. direct statements and `Row::get` keep every query and mapping visible beside the feature code;
3. the bounded fixture has substantially fewer resolved packages and a smaller executable;
4. blocking behavior can be stated honestly and isolated through a bounded semaphore plus `tokio::task::spawn_blocking`;
5. bundled SQLite produces a self-contained sample without requiring an operator-installed SQLite library.

This selection does **not** promote `rusqlite` into Yetlet core or define a universal data provider. An application may instead opt into SQLx, another mapper, or an external ORM when its requirements justify the cost. Such dependencies remain absent from unrelated Yetlet applications and browser graphs.

## Explicit non-goals

No Yetlet ORM, query DSL, implicit `DbContext`, generated models, repository/unit-of-work requirement, migration framework, connection-pool abstraction or service locator is introduced. Migrations are sample-owned SQL. The sample opens one connection per bounded blocking operation; this is an intentionally simple deployment choice rather than a claim that pooling is never useful.
