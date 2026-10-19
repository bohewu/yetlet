# YT-094 — two Admin replicas and Valkey

This opt-in Linux Compose fixture uses the existing provider and app settings.
See [implementation and observed results](../../docs/YT-094-VALKEY-SESSIONS.md).
It does not start an IdP or change the default Memory backend.

## Prepare the application image

Build the existing frontend first if absent, using the Admin guide. Then build
the native app with the explicit capability:

```sh
cargo build --offline --locked --manifest-path examples/admin-dashboard/Cargo.toml --features valkey
```

Prepare a disposable Docker build directory outside the repository. Its
`rootfs/` contains only:

- `/app/admin-dashboard`: that executable, mode 0555.
- Runtime libraries and ELF loader at their original absolute paths, as reported
  by `ldd` for that executable, copied as ordinary files with mode 0555.
- `/app/config/{base,development,staging,production}.toml`: public application
  settings, mode 0444. Never copy a Development-secret file.
- `/app/assets/`: the existing `frontend/dist` snapshot, readable by UID 10001.
- An empty `/data` directory, mode 0777, for Docker's initial named-volume copy.
  SQLite files are subsequently created with the application's private mode.

Copy this fixture's `Dockerfile` into that directory and build a unique local
image. Packaging uses the host's matching runtime libraries, not an assumption
that a host-built executable runs against an arbitrary image's libc. No source,
toolchain, login codes or credentials belong in the image. This fixture is not a
replacement for the separately qualified generated cloud-image contract.

## Select settings and run

Set `YT094_APP_IMAGE` to the prepared image, `YT094_NAMESPACE` to a unique
deployment namespace, and `YT094_VALKEY_PORT`, `YT094_APP_A_PORT` and
`YT094_APP_B_PORT` to three unused loopback ports. Use an isolated project name:

```sh
docker compose --project-name "$YT094_PROJECT" --file qualification/yt094-valkey/compose.yaml up -d
```

All settings use the existing `APP_SESSION__...` overrides. The `redis://` URL
denotes the compatible protocol; the actual selected server is Valkey. Default
builds reject selecting the remote backend without feature `valkey`. The fixture
defaults to the app's 1800-second (30-minute) absolute session lifetime.
`YT094_SESSION_TTL_SECONDS` explicitly overrides that setting for both replicas;
for example, use `60` for a short qualification run. The app accepts 1 second to
30 days. Reads do not renew expiry; rotation starts a new configured lifetime.

Use one private browser context for both loopback ports. Read A's fresh
`LOCAL_LOGIN` code from its private container log, sign in through A's native
form, and visit B's Project 1 list. Sign out on B and confirm the old cookie is
rejected by both. Never publish the log/code, cookie, CSRF token or payload.

Restart A while preserving Valkey and verify B and restarted A still accept the
same unexpired session. Pause Valkey and verify protected routes fail with 503
without new cookies, then unpause and verify recovery. Stop/start Valkey to
exercise deliberate data loss: old cookies must remain unauthorized after
reconnection, and a fresh login must work across replicas. Liveness remains
available during a session-store outage. SQLite volumes are per replica and are
not a shared business database.

## Optional shared SQLite directory

For the separately selected single-host business-storage slice, add the
[volume override](compose.shared-sqlite.yaml):

```sh
docker compose --project-name "$YT094_PROJECT" --file qualification/yt094-valkey/compose.yaml --file qualification/yt094-valkey/compose.shared-sqlite.yaml up -d
```

Both app containers now mount the same local `shared-data` volume at `/data`,
including `board.sqlite3` and its WAL/SHM files. The existing
`APP_DATABASE__PATH` setting and SQLite implementation are reused. Without the
override, the original per-replica data selection remains unchanged. Use the
same two `--file` arguments for lifecycle commands and `down --volumes`.

Check that A-created records appear through B, B updates appear through A,
stale/same-version writes report conflicts and restarting an app retains the
shared record. The [focused application evidence](../../docs/YT-094-SHARED-SQLITE.md)
also checks existing busy/error recovery through two independent native Board
connections. This is a local-volume application integration, not a qualification
of SQLite's transaction engine, distributed storage, network filesystems or HA.

## Stop and clean up

```sh
docker compose --project-name "$YT094_PROJECT" --file qualification/yt094-valkey/compose.yaml down --volumes
```

Verify all three listeners and project resources are gone. Remove the uniquely
owned application image if no longer needed. Preserve unrelated containers,
images, volumes and prior evidence. No registry push or remote deployment is
part of this fixture.
