# Security Policy

## Supported versions

Yetlet is currently a pre-release prototype. No version is production-supported and there is no security-fix service-level agreement. Reports about the current `master` branch are still welcome so defects can be assessed before a public release.

The maintainer selected [Private preparation](docs/YT-020-PRIVATE-RELEASE-PREP.md) for experimental `0.1.0` and the original nine library candidates. After separately approved activation, the latest `0.1.x` patch would be maintained on a best-effort basis; older patches would be upgrade targets, not separate maintained branches. This prepared policy does not mean a version has been released or create a response/remediation deadline.

## Reporting a vulnerability

Do **not** open a public GitHub Issue for a suspected vulnerability.

- While the repository is private, use an existing private collaborator channel to contact the maintainer.
- When the repository becomes public, use GitHub Private Vulnerability Reporting when that feature is enabled.
- Before public activation, the maintainer must verify a reachable private fallback and document it alongside the GitHub reporting route. No unverified contact address is advertised here; while Private, use the existing private collaborator channel.

Include a clear impact statement, affected commit/version, reproduction steps, prerequisites, and any suggested mitigation. Avoid sending secrets, personal data, destructive payloads, or public exploit details.

The maintainer will handle reports on a best-effort basis, confirm the affected boundary, and coordinate disclosure after a fix or documented risk decision. This policy does not promise a particular response or remediation deadline while Yetlet remains a prototype.

## Scope and non-claims

The bounded sample security evidence, threat model, automated tests and local reviewers are not penetration testing or formal assurance. Production deployments must satisfy [docs/THREAT-MODEL.md](docs/THREAT-MODEL.md), [docs/RELEASE-READINESS.md](docs/RELEASE-READINESS.md), and the application's own provider and operational requirements.
