# Streamed Jobs

Service A reads a local synthetic file in 64 KiB chunks and streams one HTTP
request to service B. B writes each incoming chunk to an anonymous temporary
file. At upload EOF it rewinds the file and sends the response headers. It then
reads the file back in chunks, counting bytes and computing SHA-256. A publishes
B's receipt in the terminal job snapshot.

A owns one job, `demo`, per process. `POST /jobs/demo` starts the job and returns
202. Every connection to `GET /jobs/demo/events` gets the latest snapshot,
followed by live changes until `done` or `failed`. A subscriber can disconnect
without cancelling the transfer. Intermediate snapshots can coalesce. The SSE
`sent` field counts bytes A supplied to the HTTP body. B's first-write log
records bytes independently at the receiver.

## Run the exercise

From the repository root on Linux, run this Bash block with Cargo, Python 3.11+
and coreutils. It builds the sample, starts both services on ephemeral
loopback ports, interrupts an SSE connection during the upload, and reconnects
to the same job. It compares B's receipt with the source file, then removes
the temporary directory and stops both services.

```bash
(
  set -eu
  cargo build --locked -p yetlet-streamed-jobs
  bin="$(cargo metadata --locked --no-deps --format-version 1 |
    python3 -c 'import json,sys; print(json.load(sys.stdin)["target_directory"])')/debug/yetlet-streamed-jobs"
  scratch=$(mktemp -d)
  pids=()
  trap 'for pid in "${pids[@]}"; do kill "$pid" 2>/dev/null || :; wait "$pid" 2>/dev/null || :; done; rm -rf -- "$scratch"' EXIT
  python3 - "$scratch/input.bin" <<'PY'
import random, sys
rng = random.Random(42)
with open(sys.argv[1], "wb") as output:
    for _ in range(256):
        output.write(rng.randbytes(65536))
PY
  listen_url() {
    for ((i=0; i<100; i++)); do
      url=$(awk '/^listening / {print $2; exit}' "$1")
      if [[ -n "$url" ]]; then echo "$url"; return; fi
      sleep 0.05
    done
    return 1
  }
  "$bin" receiver 0 >"$scratch/b.log" &
  pids+=("$!")
  b=$(listen_url "$scratch/b.log")
  "$bin" sender 0 "${b##*:}" "$scratch/input.bin" >"$scratch/a.log" &
  pids+=("$!")
  a=$(listen_url "$scratch/a.log")
  python3 examples/streamed-jobs/client.py "$a" "$scratch/input.bin"
  cat "$scratch/b.log"
)
```

The sample accepts a stable
regular file of 1 byte to 64 MiB and accepts one job per sender launch. B admits
one transfer at a time. Both file passes include a 12 ms pause per chunk to make
the phases visible. The demo uses trusted loopback peers, process-local progress
and a synthetic checksum scan. Restarting A discards the snapshot.

For client uploads into A, see the [File Server](../file-server/README.md).
For bounded replay of individual events by ID, see
[SSE Notifications](../sse-notifications/README.md).

## Measured loopback run

In the 2026-11-28 run on Ubuntu 26.04.1 x86-64 with Linux 7.0.0-44 and Rust
1.99.0, the sample ran as a `0.9.0-alpha.9` debug build from a checkout based on
`fc9f7aa`. B's first write held 16,309 of 16,777,216 bytes. The SSE client
disconnected at 65,536 bytes and reconnected to a snapshot at 1,507,328 bytes.
It then received `scan` and `done` on the second connection. The source and B's
scanned file both yielded SHA-256
`486692853f1d2a318df9bddf8db20dd038c0e0f285058c499b99e6e4db9bf3d6`.

| Peak memory | A (sender) | B (receiver) | A + B in the same sample |
|---|---:|---:|---:|
| Upload, sampled process RSS (KiB) | 6,832 | 6,820 | 13,652 |
| Scan, sampled process RSS (KiB) | 6,824 | 6,692 | 13,516 |
| Ready through client exit, sampled process RSS (KiB) | 6,832 | 6,820 | 13,652 |
| Complete service-unit lifetime, kernel cgroup peak (bytes) | 2,428,928 | 18,956,288 | — |

The observer read each service PID's `/proc/PID/status` (`VmRSS`) in pairs,
with a 10 ms sleep between pairs (median interval 10.36 ms). Upload samples
start at client launch. B's upload-complete marker starts the scan phase, and
B's scan-complete marker ends it. The observer labeled each pair using B's
latest marker and collected 334 upload and 407 scan samples. Shorter spikes
can fall between reads. Adding the RSS readings counts pages shared by the
processes twice.

Each service ran in its own transient systemd user unit with memory accounting.
The lifetime row uses systemd's cgroup v2 `MemoryPeak` after process exit.
Each unit contained its Rust service alone. The input generator, SSE client
and sampler ran outside both units. Cgroup accounting includes charged
file-cache and kernel memory, including B's temporary-file pages.
Process RSS counts resident mappings, including shared code and libraries.
These figures describe one paced, single-job run.
