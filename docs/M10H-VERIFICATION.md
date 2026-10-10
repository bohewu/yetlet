# M10h Verification — Codex-Assisted Internal Security Scan

**Date:** 2026-10-10
**Task:** YT-067 / M10h
**Status:** Blocked: the requested installed security-scan started but did not produce a complete report. No external-audit or security-clean claim.

## Maintainer scope revision

The maintainer explicitly selected a Codex security-scan instead of an expensive external assessment and requested that the project not claim separate external verification. This supersedes the earlier YT-067 requirement for an appointed outside assessor. The [external-assessment template](EXTERNAL-SECURITY-REVIEW.md) is retained only for optional future use, not as a current milestone requirement.

YT-067 now requires an identified current-source scan, a complete canonical report with honest scope/coverage, findings disposition and focused verification of any accepted fixes. A process exit code, dependency audit, blank report, limited inspection or ordinary document-review PASS is not a completed security scan. This is internal automated/Codex-assisted assessment, not certification, penetration testing or formal assurance.

## Actual attempt and blocker

The installed Codex Security 0.1.32 standard security-scan was invoked on candidate `3128d23591feee15ce976d7b83a56dcd587048d7`. Job `wc_job_oCTu5__mlpWYcsLr` reached terminal exit 0 with an explicitly incomplete scan outcome. Scan ID `580f6c9a-ded9-4f69-a3c9-16bbcc29ab97` passed capability preflight, but the first canonical checkpoint failed due to writable ancestor permissions. Ten fully reviewed files were reported; repository coverage and findings counts were not finalized, and no report was generated. The source-review workers terminated.

The owner-verified Codex directory's group-write bit was removed, changing mode 0775 to 0755. No credential/configuration file was edited. A same-scan continuation was then rejected by the platform safety check before returning a Job identity. No alternative scanner, reworded retry, role emulation or fabricated report was used to bypass that refusal. The registered scan remains incomplete with no active coordinator claimed.

See [delivery verification](M10-DELIVERY-VERIFICATION.md) for the exact CI, Windows, Docker and scan evidence boundaries. YT-067 remains Blocked because the internal scan is incomplete. Paid external assessment is not required by this revised task.

## Historical review and current acceptance

Earlier local reviewers `wc_job__-PsA9lkTP2U5_D2` and repeat `wc_job_tHTDY79-fLUFKJ_G` reviewed the former handoff/status documents, not source-security coverage or an external report. The repeat PASS repaired stale next-work wording; it remains valid only as historical document evidence.

Current documentation validation must preserve the maintainer's revised scope, explicit incomplete scan outcome and independent production limitations. Any later scan continuation must be permitted by the platform and retain its correct scan ownership; current-session emulation cannot override a platform refusal. YT-020 remains Blocked by its remaining release/support/operational decisions, and no scanner or green CI automatically authorizes publication or deployment.
