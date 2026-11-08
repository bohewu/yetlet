# Security Policy

## Supported versions

Yetlet is currently a pre-release prototype. No version is production-supported and there is no security-fix service-level agreement. Reports about the current `master` branch and the published Experimental SDK/CLI are welcome. See [Release readiness](docs/RELEASE-READINESS.md) for distribution identity; publication does not establish production support.

The following is the historical pre-public policy checkpoint, not current distribution status. The maintainer selected [Private preparation](docs/YT-020-PRIVATE-RELEASE-PREP.md) for experimental `0.1.0` and the original nine library candidates. After separately approved activation, the latest `0.1.x` patch would be maintained on a best-effort basis; older patches would be upgrade targets, not separate maintained branches. This prepared policy does not mean a version has been released or create a response/remediation deadline.

## Reporting a vulnerability

Do **not** open a public GitHub Issue for a suspected vulnerability.

- While the repository is private, use an existing private collaborator channel to contact the maintainer.
- Use GitHub Private Vulnerability Reporting through the repository's Security → Advisories → Report a vulnerability entry. The [accepted reporting verification](docs/YT-206-PUBLIC-PREVIEW-DISCLOSURE.md#reporting-completion) records the form and actual maintainer notification delivery at its original scope.
- If reporting configuration changes, verify the form and maintainer notification route before advertising the changed route.
- A separate email address, custom domain or email-forwarding service is optional and is not a prerequisite for the source preview.
- If the private form is unavailable, a public Issue may request a private reporting route only. Do not include vulnerability details, exploit steps, secrets or personal data in that Issue.

Include a clear impact statement, affected commit/version, reproduction steps, prerequisites, and any suggested mitigation. Avoid sending secrets, personal data, destructive payloads, or public exploit details.

The maintainer will handle reports on a best-effort basis, confirm the affected boundary, and coordinate disclosure after a fix or documented risk decision. This policy does not promise a particular response or remediation deadline while Yetlet remains a prototype.

## Scope and non-claims

The bounded sample security evidence, threat model, automated tests and local reviewers are not penetration testing or formal assurance. Production deployments must satisfy [docs/THREAT-MODEL.md](docs/THREAT-MODEL.md), [docs/RELEASE-READINESS.md](docs/RELEASE-READINESS.md), and the application's own provider and operational requirements.
