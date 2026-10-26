# Optional Starter telemetry collection — YT-136

Status: **Accepted after independent Reviewer PASS**, zero issues and required
followups, at bounded optional reference scope. This is a Linux developer reference for
one-shot collection, not an installed collector. Read the
[bounded contract](../../../../docs/YT-136-TELEMETRY-COLLECTION.md),
[accepted signal/privacy limits](../../../../docs/YT-128-OBSERVABILITY.md)
and [private access reference](../private-operator/README.md).

Start the existing app through your selected lifecycle with
`APP_METRICS__ENABLED=true`; its typed default remains false. Use the existing
Production config and private operator boundary. Do not expose `/metrics` via
the business proxy or forward directly to the app port for business access.
Local users/processes must be trusted; collection has no independent endpoint
authentication. No application Bearer token is needed for metrics.

The commands require Python 3, Linux resource limits and `/usr/bin/curl` 8.4+
(the exercised version is 8.18.0). The local curl manual documents
`--max-filesize` aborting unknown-size transfers from 8.4 onward. An independent
`RLIMIT_FSIZE` also caps each child output file. No tool download is required.
Run under the same trusted UID that owns the private evidence and log file.
Set an existing private collection root and the actual high loopback port:

```bash
umask 077
export TELEMETRY_DIR="$(mktemp -d /tmp/yetlet-telemetry.XXXXXXXX)"
export TELEMETRY_PORT=3010
# For the later log command, select ONLY this owned app's regular JSON output.
# export TELEMETRY_LOG=/absolute/private/path/starter.jsonl
```

## One bounded scrape

Run once; schedule only if you independently select a non-overlapping interval
and retention policy. Every invocation creates a new private observation
directory. Candidate/metadata files are bounded and never treated as samples.
The atomic last-good JSON pointer names a unique immutable successful snapshot;
read that observation's timestamp/hash with the snapshot. Inspect the newest
observation result first: a last-good pointer alone never means current health.
A 404 is reported as disabled-or-missing, requiring settings/route identity to
diagnose. Other HTTP/format/transport/output errors publish no valid snapshot.

```bash
python3 - <<'PY'
import datetime, hashlib, json, os, pathlib, resource, stat, subprocess, tempfile, time
os.umask(0o077)
root = pathlib.Path(os.environ['TELEMETRY_DIR'])
s = root.lstat()
assert stat.S_ISDIR(s.st_mode) and s.st_uid == os.getuid() and s.st_mode & 0o077 == 0
port = int(os.environ['TELEMETRY_PORT'])
assert 1024 <= port <= 65535
obs = pathlib.Path(tempfile.mkdtemp(prefix='scrape-', dir=root))
now = lambda: datetime.datetime.now(datetime.timezone.utc).isoformat()
result = {'started_utc': now(), 'state': 'failed', 'target': f'127.0.0.1:{port}',
          'body_cap_bytes': 65536, 'caller_watchdog_seconds': 5}
body = obs / 'candidate.metrics'
# Exclusive creation inside the new private directory precedes curl's write.
with body.open('xb'): pass
args = ['/usr/bin/curl', '--disable', '--silent', '--noproxy', '*', '--proxy', '',
        '--proto', '=http', '--max-redirs', '0', '--retry', '0',
        '--connect-timeout', '1', '--max-time', '3', '--max-filesize', '65536',
        '--output', str(body), '--write-out', '%{http_code}\n%{content_type}\n',
        f'http://127.0.0.1:{port}/metrics']
def limits():
    resource.setrlimit(resource.RLIMIT_FSIZE, (65536, 65536))
    resource.setrlimit(resource.RLIMIT_CORE, (0, 0))
started = time.monotonic()
try:
    with (obs / 'response.txt').open('xb') as meta:
        try:
            run = subprocess.run(args, stdout=meta, stderr=subprocess.DEVNULL,
                                 timeout=5, preexec_fn=limits)
            result['curl_exit'] = run.returncode
        except subprocess.TimeoutExpired:
            result['curl_exit'] = 'watchdog_timeout_killed_reaped'
    data = body.read_bytes()  # OS file-size limit bounds the local candidate.
    metadata = (obs / 'response.txt').read_bytes()
    result['body_bytes'] = len(data)
    fields = metadata.decode('ascii').splitlines() if len(metadata) <= 256 else []
    code = fields[0] if fields else 'unknown'
    result['http_status'] = code
    if result['curl_exit'] != 0:
        result['reason'] = 'transport_or_output_failure'
    elif code == '404':
        result.update(state='disabled_or_missing_route', reason='verify_setting_and_route')
    elif (code == '200' and fields[1:] == ['text/plain; version=0.0.4; charset=utf-8']
          and 0 < len(data) <= 65536 and data.endswith(b'\n')
          and b'# HELP starter_' in data and b'# TYPE starter_' in data
          and b'\nstarter_readiness_last_probe ' in data):
        data.decode('utf-8')
        snapshot = obs / 'snapshot.metrics'
        body.rename(snapshot)
        result.update(state='fresh', sha256=hashlib.sha256(data).hexdigest(),
                      snapshot=str(snapshot))
    else:
        result['reason'] = 'unexpected_status_or_format'
except (OSError, ValueError) as exc:
    result.update(state='failed', reason=type(exc).__name__)  # No raw messages.
result.update(completed_utc=now(), elapsed_seconds=time.monotonic()-started)
with (obs / 'observation.json').open('x') as out:
    json.dump(result, out, indent=2); out.write('\n')
if result['state'] == 'fresh':
    pointer = obs / 'last-good-candidate.json'
    with pointer.open('x') as out:
        json.dump(result, out, indent=2); out.write('\n')
    os.replace(pointer, root / 'last-good.json')
print(json.dumps({'observation': str(obs), **result}))
raise SystemExit(0 if result['state'] == 'fresh' else 1)
PY
```

A successful source-specific signature is not full Prometheus syntax validation.
No redirects are followed and no environment proxy/curlrc or automatic retry is
used. The body and metadata each have a 64 KiB file cap. A write/access/full-disk
failure can prevent even the failure record; the command then exits nonzero.
Check its exit and missing observation as a gap. Atomic pointer replacement
protects against partial JSON publication, not disk failure or malicious same-UID
writers. This does not bound app-side response allocation or connection admission.

## One bounded JSON log snapshot

Capture the app's own stdout to a private regular file through your existing
lifecycle (the fixture uses an exclusive file opened before child startup).
Do not select a shared journal, Nginx access log, arbitrary service log or
credential file. This command samples the initial file size, reads only that
bounded prefix, records concurrent growth as a gap and rejects oversize or
malformed input. Run after the selected operation or after clean stop for a
stable snapshot. It does not lock, rotate, follow or promise complete history.
Unknown events/fields are excluded; filtered events are counted.

```bash
python3 - <<'PY'
import datetime, json, os, pathlib, stat, tempfile
os.umask(0o077)
root = pathlib.Path(os.environ['TELEMETRY_DIR'])
s = root.lstat()
assert stat.S_ISDIR(s.st_mode) and s.st_uid == os.getuid() and s.st_mode & 0o077 == 0
obs = pathlib.Path(tempfile.mkdtemp(prefix='logs-', dir=root))
result = {'completed_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(),
          'state': 'failed', 'gap': True, 'input_cap_bytes': 131072}
records = []
try:
    fd = os.open(os.environ['TELEMETRY_LOG'], os.O_RDONLY | os.O_NOFOLLOW | os.O_NONBLOCK)
    with os.fdopen(fd, 'rb') as source:
        before = os.fstat(source.fileno())
        assert stat.S_ISREG(before.st_mode) and before.st_uid == os.getuid()
        assert before.st_mode & 0o077 == 0
        raw = source.read(min(before.st_size, 131073))
        after = os.fstat(source.fileno())
    result.update(source_size=before.st_size, bytes_read=len(raw))
    lines = raw.splitlines()
    if before.st_size > 131072 or len(raw) > 131072:
        raise ValueError('oversize')
    if len(lines) > 256 or any(len(line) > 4096 for line in lines):
        raise ValueError('line_limit')
    events = {'started', 'shutdown_requested', 'stopped', 'shutdown_incomplete',
              'http_request', 'sql_finished', 'readiness_checked', 'http_admission_rejected'}
    methods = {'GET','POST','PATCH','PUT','DELETE','HEAD','OPTIONS','OTHER'}
    outcomes = {'ok','not_found','conflict','unavailable','invalid_schema','internal'}
    filtered = 0
    for line in lines:
        entry = json.loads(line)
        fields = entry.get('fields', {})
        event = fields.get('event')
        if event not in events:
            filtered += 1; continue
        out = {'event': event}
        spans = entry.get('spans', []) + [entry.get('span', {})]
        context = next((s for s in spans if s.get('name') == 'http_request'), {})
        for key in ('request_id', 'status', 'duration_ms'):
            value = fields.get(key, context.get(key))
            if type(value) is int and 0 <= value <= 2**64-1:
                out[key] = value
        method = fields.get('method', context.get('method'))
        if method in methods: out['method'] = method
        if fields.get('outcome') in outcomes: out['outcome'] = fields['outcome']
        if fields.get('reason') in {'capacity','shutdown','deadline'}:
            out['reason'] = fields['reason']
        if type(fields.get('ready')) is bool: out['ready'] = fields['ready']
        records.append(out)
    encoded = b''.join((json.dumps(r, separators=(',', ':'))+'\n').encode() for r in records)
    if len(encoded) > 131072: raise ValueError('output_limit')
    with (obs / 'events.jsonl').open('xb') as output: output.write(encoded)
    changed = (before.st_size, before.st_mtime_ns) != (after.st_size, after.st_mtime_ns)
    result.update(state='captured', gap=changed or filtered > 0, filtered_events=filtered,
                  records=len(records), output_bytes=len(encoded), complete_history=False)
except (OSError, ValueError, AssertionError, TypeError, AttributeError):
    result['reason'] = 'input_unavailable_oversize_or_invalid'
with (obs / 'observation.json').open('x') as output:
    json.dump(result, output, indent=2); output.write('\n')
print(json.dumps({'observation': str(obs), **result}))
raise SystemExit(0 if result['state'] == 'captured' else 1)
PY
```

Only fixed event names, generated numeric request ID, finite method/outcome/reason,
boolean readiness and numeric status/duration are exported. No arbitrary string
values, timestamps from input, messages, URLs, request/response headers, token,
cookie, query, request body, SQL or raw errors are exported. The private original
app file remains subject to the source's accepted YT-128 exclusion boundary;
this filter cannot establish audit completeness or make arbitrary logs safe.
Retain collection time with the bounded snapshot; IDs reset per process.

For sink, alert, diagnostic and retention guidance, see the
[contract and observation limits](../../../../docs/YT-136-TELEMETRY-COLLECTION.md#diagnosis-and-optional-operations).
No Prometheus/promtool, remote sink, real alert delivery, journal retention or log
rotation service was started or qualified. The exercised result remains pending
independent review in the linked contract.

The focused local integration exercised these exact Python command blocks once
against the verified existing release: fresh metrics 200 (3,998 bytes), clean
stop followed by curl exit 7, and explicitly disabled metrics 404 with healthy
readiness. Both failed observations preserved the last-good pointer. Bounded
JSON projection correlated the synthetic 201 with SQL completion; an oversized
131,073-byte log source was rejected with an explicit gap. Both application
children exited 0, were reaped and released the loopback listener; disposable
fixture data was removed. See the linked contract for exact identities, evidence
and limits. This does not exercise a remote sink or qualify ongoing operations.
