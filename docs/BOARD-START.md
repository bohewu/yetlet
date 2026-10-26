# Start the Lab Operations Board

This guide applies to a Yetlet checkout containing `examples/lab-operations-board`.
The Board implementation is committed; available source still depends on the
revision checked out, and no registry release is assumed. This is an unpublished,
local synthetic-identity demonstration. The [current Board README](../examples/lab-operations-board/README.md)
owns opt-in options and safety limits; the [API reference](API-REFERENCE.md)
distinguishes application APIs from framework packages.

## Local startup

Run from the repository root with the repository-pinned Rust 1.99 toolchain:

```sh
cargo xtask build board-assets --check
cargo run --release --locked -p yetlet-lab-operations-board
```

Cargo may need to download already locked dependencies on a fresh development machine. Add `--offline` only when that exact dependency set is cached. No Node, browser WASM build, Bootstrap JavaScript, Redis or GPU is required to start this application. The fixed CSS and its same-origin images are already included; `--check` verifies them without fetching or rewriting assets.

Open `http://127.0.0.1:3012/board` in the browser on the same machine. Use the exact IP address and port: the application intentionally checks Host/Origin. Do not replace the URL with `localhost` while the server is configured for `127.0.0.1`.

Startup prints three fresh `LOCAL_LOGIN` lines. Enter the lowercase name in **Identity**, and copy that name's current generated value into **Terminal code**:

| Identity | Scope |
|---|---|
| `alice` | Operator for Project 1; create/update permitted |
| `bob` | Operator for Project 2; no access to Project 1 |
| `viewer` | Read-only access to Project 1 |

These are generated demonstration codes, not fixed passwords or real organization identities. Keep them local and do not paste them into reports. They change on every process restart. Memory login state is lost on restart; persistent work records are independent.

A new database starts with an empty list. Select **New work item**, enter a title/description, choose status/priority/assignee, and submit. Open a record to see its transactional history. To demonstrate a conflict, open the same edit page twice, save one, then submit the older draft: it returns HTTP 409 and preserves that draft rather than overwriting the newer version.

**Enable optional live status** explicitly loads the local status module. Updates show current counts and a notice to refresh the authoritative work-item list; the module does not silently overwrite an open draft or replay writes. **Stop live status** and **Reconnect live status** control the owned connection. Ordinary forms and lists remain usable without JavaScript.

## Data location and shutdown

The default SQLite file is `target/lab-operations.sqlite3`. Keep it to retain work records and history. Use `YETLET_BOARD_DB` to place it in a different private local directory; create that parent directory before starting. Never select a production database for this demo.

POSIX example:

```sh
mkdir -p .local-board
chmod 700 .local-board
YETLET_BOARD_DB=.local-board/board.sqlite3 \
YETLET_BOARD_LISTEN=127.0.0.1:3012 \
cargo run --release --locked -p yetlet-lab-operations-board
```

PowerShell equivalent configuration. Retained [Windows native execution](YT-094-WINDOWS-VERIFICATION.md)
includes Board tests and Admin HTTP/restart/drain at its recorded source; it is
not new qualification of every later feature or this checkout:

```powershell
New-Item -ItemType Directory -Force .local-board | Out-Null
$env:YETLET_BOARD_DB = '.local-board/board.sqlite3'
$env:YETLET_BOARD_LISTEN = '127.0.0.1:3012'
cargo run --release --locked -p yetlet-lab-operations-board
```

Press **Ctrl+C** in the application's terminal. The server signals live streams to stop and drains admitted database work. Do not remove database/WAL files while it is running. The demo bounds records to 10,000 and history rows to 100,000; reaching a limit returns an explicit rejection rather than deleting old history automatically.

## Viewing an Ubuntu checkout from another computer

Keep the application bound to Ubuntu loopback. With an existing authorized SSH account, forward the same port from the browsing computer:

```sh
ssh -N -L 3012:127.0.0.1:3012 <user>@<ubuntu-host>
```

Then open `http://127.0.0.1:3012/board` locally. This is a suggested operator command, not a tunnel started by the verification. Keep local and server port numbers equal to preserve the exact authority check; if that port is occupied, select a different matching port at both ends. Do not expose the synthetic login service by changing the listener to `0.0.0.0`.

## Verification and screenshots

The application README documents focused commands. [Qualification](YT-074-QUALIFICATION.md) distinguishes actual browser/native/load evidence from unverified real OS IME, spoken assistive technology and production claims. The [screenshot gallery](screenshots/m11-board/README.md) contains real browser captures of synthetic records. Those records belonged to an isolated verification database which was cleaned up; they are not silently seeded into your normal database.
