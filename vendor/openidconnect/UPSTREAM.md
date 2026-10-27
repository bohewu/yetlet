# Application-owned OpenID Connect dependency patch

This is a temporary runtime-source snapshot of the official `openidconnect`
4.0.1 release. It preserves the existing public API and OIDC implementation while
moving its P-256/P-384 dependencies to the 0.14 series for
[RUSTSEC-2026-0387](https://rustsec.org/advisories/RUSTSEC-2026-0387.html).
It is an application dependency, excluded from Yetlet workspace membership and
publication. The Board's `oidc` feature and Admin's startup setting still control
their existing compilation and runtime boundaries.

## Provenance and license

- Upstream: <https://github.com/ramosbugs/openidconnect-rs>
- Published source: `openidconnect` 4.0.1 from crates.io.
- Archive SHA-256: `0d8c6709ba2ea764bbed26bce1adf3c10517113ddea6f2d4196e4851757ef2b2`.
- Upstream VCS commit: `b639b5d39eac6903238867aeb2b29326502e6b26`.
- License: MIT; the original text is retained in [LICENSE](LICENSE).
- The original README follows a local notice in [README.md](README.md).

## Local delta

The active manifest keeps the upstream normal dependencies and features except
for `p256`/`p384` 0.14.0. Its Rust requirement is the application's 1.99 baseline,
and publication is disabled. Existing SHA-2 0.10 dependencies for RSA, HMAC and
token hashes remain unchanged; the newer EC stack may carry its own digest
generation.

All 22 runtime Rust source files are retained. Only upstream `cfg(test)` module
declarations and test-only suffixes are removed; protocol, verification and
runtime signing APIs remain present. Examples, external certification tests,
development dependencies, upstream test modules/private-key fixtures, original
lockfiles, CI metadata and registry vendor-checksum metadata are intentionally
omitted. This is a runtime dependency snapshot, not the complete upstream test
distribution. Focused application compatibility tests use public EC verification
vectors from the original MIT-licensed source without retaining private keys.

The root and Admin workspaces each select this directory through their own
`[patch.crates-io]` section. A patch in the Board member manifest alone would not
cover both workspaces. The existing RSA-specific audit decision remains separate;
this patch adds no advisory exemption.

## Removal condition

Prefer a reviewed official release that supplies the fixed dependency closure.
After migrating both application workspaces and validating their affected graph,
remove both source overrides and this directory. Do not publish this snapshot
under the upstream crate name or treat it as an official fixed release.
