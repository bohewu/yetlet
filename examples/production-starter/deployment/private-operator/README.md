# Optional private Starter reference (YT-133)

This is a small, editable **developer integration example**, not a required
Yetlet deployment stack, installer or production qualification. Consumers choose
their own deployment and operations. It changes no framework or application API.
It illustrates one Linux host, one native Starter process, persistent local
SQLite, native systemd supervision and private SSH access. Same-UID user units
in a development experiment do not prove a dedicated production service account.

The path is trusted workstation `127.0.0.1:LOCAL_PORT` → verified SSH local
forward → server `127.0.0.1:EDGE_PORT` Nginx → `127.0.0.1:APP_PORT` Starter.
Use one upstream. Trust host-local processes and the workstation. SSH access is
not product identity: SSR has anonymous Memory Session/CSRF, while the API still
requires its Bearer token and rejects every Origin header. Local HTTP cookies
remain HttpOnly/SameSite=Lax but **not Secure**. Reload forms after each app
restart. Public TLS, IdP, Valkey, multiple writers and HA are separate choices.

See the [selected reference contract](../../../../docs/YT-133-STARTER-SINGLE-INSTANCE-CONTRACT.md),
[application instructions](../../README.md),
[write/deadline limits](../../../../docs/YT-127-ADMISSION-OVERLOAD.md) and
[schema-2 forward-fix rules](../../../../docs/YT-131-MIXED-VERSION-COMPATIBILITY.md).
Historical load, backup, identity and browser evidence is unchanged; this example
does not rerun or promote it to release/storage/deployment qualification.

## Prepare owned copies

Build from the repository root, keeping the lock and offline dependency cache:

```sh
nice -n 10 cargo build --manifest-path examples/production-starter/Cargo.toml --release --locked --offline -j 2
```

Record the source, lockfile, release executable and rendered configuration hashes.
Choose a private absolute fixture root on the intended persistent filesystem;
check its actual mount rather than assuming `/tmp` is persistent. Create owned
configuration, data and runtime directories with mode 0700. Copy the existing
`base.toml` and `production.toml` into the configuration directory. Use the existing
`APP_ENVIRONMENT=Production`, `YETLET_CONFIG_ROOT`, `APP_SERVER__PORT` and
`APP_DATABASE__PATH` overrides. No new settings are introduced.

Generate a random API token without printing it, for example into an owned 0600
environment file (run with the output path substituted):

```sh
python3 - <<'PY'
import os, secrets
fd = os.open('/ABSOLUTE/OWNED/token.env', os.O_WRONLY | os.O_CREAT | os.O_EXCL, 0o600)
with os.fdopen(fd, 'w') as output:
    output.write('APP_SERVICE__API_TOKEN=' + secrets.token_hex(32) + '\n')
PY
```

Do not pass the token value on command lines, commit it, print it or retain it in
request logs. `EnvironmentFile` provides the existing application environment
variable; it is not a secret vault and cannot hide it from the same UID/root.
The file must contain only that token variable; do not override the selected
database/configuration through it. Remove development credentials at teardown.

Substitute all `@PLACEHOLDER@` values in private copies of `nginx.conf.in` and
`starter.service.in`. Use absolute paths without whitespace, quotes, backslashes,
Nginx variables or systemd `%` specifiers. `@REFERENCE@` points to this directory,
`@BINARY@` to the newly built release executable, `@RECEIPT@` to the storage
receipt, and `@AUTHORITY@` is the exact browser Host including its local port,
for example `starter.private.test:43001`. Resolve that name to workstation
loopback. Pick unused ports; a port check is not a reservation. Create runtime
`body/` and `proxy/` directories. The business-open file must initially be absent.
Validate the rendered Nginx configuration with the actual selected Nginx
executable (`nginx -t -p OWNED_RUNTIME/ -c ABSOLUTE_CONFIG`).

The service template is optional input for an operator's unit installation.
For an isolated experiment, create a uniquely named transient `systemd-run
--user` unit with the same service properties instead of installing units or
reloading the existing manager. Bound CPU, memory, tasks and file descriptors as
shown; these are example budgets, not measured capacity. Leave `Restart=no`.
`TOKIO_WORKER_THREADS=2` bounds the existing Tokio worker setting; blocking SQL
still uses its existing four permits and the unit's overall task/resource caps.
Do not change existing services to try this example.

## Explicit initialization and ordinary restart

Keep Nginx business access closed throughout initialization:

1. Confirm the chosen empty fixture, persistent parent/mount, sole writer and
   configuration authority. Run `python3 preflight.py init-check DATABASE`.
   This requires an absent database and absent WAL/SHM sidecars; it creates none.
2. For this intentional first initialization only, start the selected binary
   with the template's settings/resources/readiness hook but **without** its
   restart `ExecStartPre`. The unchanged app performs schema initialization.
   `python3 preflight.py ready APP_PORT` must pass before any business access.
3. Stop the app cleanly and confirm its entire process scope has exited. Then
   run `python3 preflight.py record DATABASE RECEIPT --allow-empty` to explicitly
   accept this newly initialized empty fixture. Without `--allow-empty`, an
   empty database is refused. Protect the receipt with the data/configuration.
4. Subsequent starts use the unmodified template: `restart DATABASE RECEIPT`
   checks the existing file/parent device and inode, schema fingerprint, schema
   version 2 and explicitly accepted emptiness before the app can open it.
   A missing, zero-byte or replaced file is refused without creating one.
   `ExecStartPost` uses a 60-second external readiness gate; unit-active alone
   does not mean ready. A failed hook fails/stops the unit. Business access
   remains closed until the hook succeeds and the selected data is checked.

The receipt is a narrow developer fixture identity guard. It does not attest
the underlying disk, mount source, backups, business truth or complete integrity.
Create it only from a known app-initialized schema after readiness and clean stop.
After storing the first expected issue, stop cleanly, preserve/remove the old
receipt deliberately and `record` a new one **without** `--allow-empty`; later
unexpectedly empty issue tables will then be refused. That deliberate re-record
is a data-authority decision, never an automatic restart fallback. A legitimate
restore changes file identity and requires separate full data/schema verification
before a new receipt. Never remove a receipt merely to bypass a failed check.
No database operation runs while sealing a stopped fixture other than read-only
inspection. The parent directory must remain private and stable; same-UID races
or malicious local replacement are outside this trusted-local example.

## Open private business access

Run the rendered Nginx in the foreground under an owned process/unit. Once app
readiness and selected data checks pass, create `RUNTIME/business-open` as a
regular owned file. Establish a workstation local forward with verified host
identity, for example:

```sh
ssh -N -T -o ExitOnForwardFailure=yes -o StrictHostKeyChecking=yes \
  -o UserKnownHostsFile=VERIFIED_KNOWN_HOSTS -i PRIVATE_KEY \
  -L 127.0.0.1:LOCAL_PORT:127.0.0.1:EDGE_PORT USER@SSH_HOST
```

Use the selected SSH account/key policy to allow only this destination and local
forwarding (`AllowTcpForwarding local`, `PermitOpen 127.0.0.1:EDGE_PORT`, no agent,
X11 or tunnel forwarding). Do not forward the app/probe port. A development test
may run a separate high-port loopback sshd with disposable keys and restricted
forwarding; it must not alter the host SSH service or personal authorized keys.
The canonical HTTP Host must equal `@AUTHORITY@`; other Hosts return 421. The
business proxy denies `/health`, `/health/*`, `/metrics` and `/metrics/*`. Readiness
is a private direct loopback request and still depends on trusted local callers.

The proxy preserves Origin, Authorization, cookies, CSRF forms and upstream
security/no-store/Set-Cookie/Vary headers. It has no cache or automatic upstream
retry. One worker's 128 connections include upstream sockets, not 128 clients.
Headers use a 1 KiB initial buffer and two 8 KiB large buffers; one header/request
line cannot exceed a large buffer. Header arrival is limited to 5 seconds.
Bodies are capped at 16 KiB, buffered within 32 KiB, with a 5-second body idle
timeout. Responses use an 8 KiB header buffer plus four 8 KiB buffers, no response
temporary files; larger responses stream as those buffers fill. The listener
backlog is 64. Connect is 1 second, upstream read idle 7 seconds, upstream send
idle/downstream send idle/keepalive idle 5 seconds. Idle bounds are **not** whole
HTTP transaction deadlines or minimum transfer rates. No requests/second limit
is imposed. These values do not bound every kernel/process allocation.

The unchanged app retains 32 HTTP/four SQL permits, zero application queues and
a cooperative 5-second timer through Response creation, excluding response body
transmission. A late blocking write can outlive that response. No timing here
creates a whole-request deadline or safe mutation retry.

## Withdrawal, stop and re-entry

Remove `RUNTIME/business-open` first. Nginx evaluates it on every new request,
including existing keepalive/tunnel routes, and returns 503 while already
admitted requests may finish. Verify rejection through every owned business
path, including an existing connection. This is the withdrawal boundary; a
request that passed the gate before removal remains active work. Readiness
alone is not an admission fence. Trusted direct-local business clients must
also be stopped/accounted for; no proxy can fence a deliberately bypassing
trusted local process.

Then send SIGQUIT to the owned Nginx master and wait for its process scope to
exit: graceful quit closes listeners/idle keepalives and waits for active work.
Do not fabricate drain evidence by killing a response in flight. Choose and
record a separate bounded withdrawal wait; expiry is a failed drain needing
investigation, not permission for silent request loss. Stop the owned tunnel
after its requests complete. Only then stop the app with SIGTERM via its owned
systemd unit. `KillMode=control-group` covers its process scope; the initial
20-second supervisor stop budget is separate from withdrawal. The app's actual
10-second grace begins at native shutdown notification, not gate removal.
Require exit 0 and `stopped`, and confirm all old writer processes are gone
before any schema change. `shutdown_incomplete`, forced termination or unknown
client outcomes require preserving data and reconciling writes before re-entry.

Use ordinary restart preflight/readiness before reopening the gate. Never retry
a mutation blindly after timeout/disconnect; inspect the issue and expected
version first. Keep schema 2 and use a compatible forward fix after writes.
Old-binary restoration is not a safe rollback; a separately verified backup
restore can discard later acknowledged work and needs an explicit loss decision.
Confirm owned ports are closed, units/processes exited and disposable API/SSH
credentials removed at the end of an experiment. Consumers must choose their
own ongoing monitoring, SLO, retention, RPO/RTO and operational authority before
using any analogous arrangement for retained real data.
