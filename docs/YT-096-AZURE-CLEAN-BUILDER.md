# YT-096 Azure clean-builder attempt

Date: 2026-10-20. Candidate remains
`ce56e0814e5423782440acd2c959ca777d572cd4`, original nine unpublished,
experimental 0.1.0 libraries, Rust 1.99/1.99.0.

**Historical SSH-path qualification blocked before source transfer/build.**
The subsequent [Run Command/private Blob continuation](YT-096-AZURE-RUN-COMMAND.md)
passes actual Ubuntu guest/uncached build qualification and deletes its resources.
The remainder of this report preserves the initial SSH checkpoint. Two disposable Azure
VM attempts reached successful control-plane provisioning and host-key reads,
but direct SSH/SCP connections timed out. No Rust installation, Cargo fetch,
package rebuild, native test or wasm build ran on Azure. This is an environment
connectivity failure, not a demonstrated product defect. The accepted
[platform qualification](YT-096-PLATFORM-QUALIFICATION.md) and
[two-host archive comparison](YT-096-TWO-HOST-REBUILD.md) remain unchanged.
YT-096 stays In progress; YT-020 stays Blocked.

## Authorization and intended scope

The owner suggested Azure CLI and explicitly required resource deletion after
verification. The bounded plan was one temporary Ubuntu 24.04 x64 guest with
an empty Cargo home/target, official pinned Rust download and upstream locked
dependency fetch, followed by ordinary nine-package Cargo verification and
selected native/wasm qualification. This would add a real Ubuntu 24.04 kernel
baseline and an uncached build observation. It would still be under shared
maintainer/agent control, not independent builder trust or signed provenance.

Azure CLI 2.91.0 used the existing Visual Studio Professional subscription.
No VM existed in that subscription at preflight. The selected location was
East Asia, size Standard_D2s_v5, Standard SSD 64 GiB, and pinned image
`Canonical:ubuntu-24_04-lts:server:24.04.202610160` from the exact image query.
The initial broad image listing also returned a daily/gen1 image; it was not
the selected image. Each attempt used its own tagged resource group, ephemeral
SSH key, and a port-22 rule restricted to observed workstation egress addresses.
SSH host public keys were read through authenticated Azure Run Command.

## Preserved failures and bounded correction

The first group was `rg-yetlet-yt096-20261020-6c1d34`. Provisioning and the
control-plane host-key command succeeded. An initial SSH directory command
completed, then SCP's separate connection timed out before transferring the
candidate source. No guest qualification command ran.

Read-only egress observations returned 211.23.119.225 through the configured
HTTP route and 118.163.185.45 through a direct route. The initial SSH rule
included only the former. One bounded network-input correction queried both
routes. Their results later changed to the same address, so the second
allowlist was updated in place to retain both already observed /32 addresses.
This correction did not open SSH to all Internet sources. The exact origin of
the timed-out TCP flows was not captured; the route observations do not prove
the complete connection root cause.

After first-group cleanup, the same qualification slice resumed once with
additive evidence under `attempt-2/`, group
`rg-yetlet-yt096-20261020-d39604`. Provisioning and the control-plane host-key
command succeeded; the initial direct SSH directory command timed out despite
the corrected allowlist. No source was transferred and no guest build/test ran.

Two consecutive SSH connection timeout failures end this retry path under the
existing Materiality Gate. There is no third VM attempt, source workaround,
broader firewall rule, model recovery or new validation framework. The first
driver and both attempts' raw logs/configuration are retained.

## Evidence and cleanup

Evidence is additive under
`artifacts/yt096-azure-clean-builder-20261020/`.
[Closeout](../artifacts/yt096-azure-clean-builder-20261020/CLOSEOUT.md) links
the baseline, exact commands, observed image/network inputs, provisioning,
host-key responses, failures and separate cleanup records. Final cleanup
verification PASS confirms both exact resource groups and all 12 owned
resources are absent, including disks and public IPs. The final inventory filter
is case-insensitive because Azure returned some resource-group names in upper
case. Both temporary local SSH key directories were removed. Independent
ordinary slice review is recorded at closeout.

No candidate files, dependency locks, tool pins, product APIs or accepted
artifacts changed. No source or dependency cache reached the guests. The
planned Rust/Cargo commands are unexecuted preparation, not a passing result.
No commit, push, signing, tag, package publication, hosted workflow activation
or production deployment was performed.

## Remaining work at the SSH checkpoint

Actual Ubuntu 24.04 guest qualification, uncached Cargo building and any
build-bound inventory from that guest remain unverified. The earlier container
shared-kernel and historical Chrome evidence limits remain. Independently
controlled builder evidence, final build-bound inventory, formal trust and
registry checks, independent release/security review and maintainer approval
remain open.

The owner later explicitly resumed a different source/evidence transport.
[Azure Run Command with private Blob staging](YT-096-AZURE-RUN-COMMAND.md) now
passes without direct SSH, with evidence retrieval and resource cleanup.
The original SSH slice above still has no executed guest qualification result.

## Independent ordinary slice review

Reviewer `/root/yt096_azure_blocked_cleanup_review_oct20` returned PASS with
zero issues or required followups, accepting report accuracy, preservation and
cleanup only. Azure qualification remains blocked. Native trace verified the
registered reviewer role, gpt-6-astra model and effective high effort; the shared
resolver observation is enforced. The reviewer independently repeated the
exact-group existence checks and case-insensitive resource-inventory check,
confirmed temporary SSH directories/processes absent, and reconciled 510 prior
evidence files, 60 snapshot entries and all 63 source inputs. No provisioning,
SSH, build or test was repeated. [Acceptance](../artifacts/yt096-azure-clean-builder-20261020/REVIEW-ACCEPTANCE.json)
preserves that limited result and the unresolved execution gates.
