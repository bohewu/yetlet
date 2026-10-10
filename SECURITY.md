# Security Policy

## Supported versions

Yetlet is currently a pre-release prototype. No version is production-supported and there is no security-fix service-level agreement. Reports about the current `master` branch are still welcome so defects can be assessed before a public release.

## Reporting a vulnerability

Do **not** open a public GitHub Issue for a suspected vulnerability.

- While the repository is private, use an existing private collaborator channel to contact the maintainer.
- When the repository becomes public, use GitHub Private Vulnerability Reporting when that feature is enabled.
- If the GitHub private-reporting form is unavailable, use the private contact method listed on the maintainer's GitHub profile and include the repository name.

Include a clear impact statement, affected commit/version, reproduction steps, prerequisites, and any suggested mitigation. Avoid sending secrets, personal data, destructive payloads, or public exploit details.

The maintainer will handle reports on a best-effort basis, confirm the affected boundary, and coordinate disclosure after a fix or documented risk decision. This policy does not promise a particular response or remediation deadline while Yetlet remains a prototype.

## Scope and non-claims

The bounded sample security evidence, threat model, automated tests and local reviewers are not penetration testing or formal assurance. Production deployments must satisfy [docs/THREAT-MODEL.md](docs/THREAT-MODEL.md), [docs/RELEASE-READINESS.md](docs/RELEASE-READINESS.md), and the application's own provider and operational requirements.
