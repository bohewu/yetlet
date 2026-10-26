import http from 'k6/http';
import exec from 'k6/execution';
import { Counter, Trend } from 'k6/metrics';

const port = Number(__ENV.YT157_PORT);
const rate = Number(__ENV.YT157_RATE);
const seconds = Number(__ENV.YT157_SECONDS);
const workload = __ENV.YT157_SIZE;
const mode = __ENV.YT157_MODE;
const repetition = __ENV.YT157_REPETITION;
const phase = __ENV.YT157_PHASE;
const targetIpv4 = __ENV.YT164_TARGET_IPV4 === undefined ? '127.0.0.1' : __ENV.YT164_TARGET_IPV4;
const octets = /^(0|[1-9]\d{0,2})\.(0|[1-9]\d{0,2})\.(0|[1-9]\d{0,2})\.(0|[1-9]\d{0,2})$/.exec(targetIpv4);
const ipv4 = octets && octets.slice(1).map(Number);
if (!ipv4 || ipv4.some(value => value > 255) ||
    !(targetIpv4 === '127.0.0.1' || ipv4[0] === 10 ||
      (ipv4[0] === 172 && ipv4[1] >= 16 && ipv4[1] <= 31) ||
      (ipv4[0] === 192 && ipv4[1] === 168))) {
  throw new Error('YT164_TARGET_IPV4 requires loopback or private IPv4');
}
if (!Number.isInteger(port) || port < 1024 || port > 65535 ||
    workload !== 'medium' ||
    !['reference', 'current'].includes(mode) || !['1', '2'].includes(repetition) ||
    !((phase === 'warmup' && rate === 125 && seconds === 2) ||
      (phase === 'measured' && rate === 1000 && seconds === 10))) {
  throw new Error('Invalid fixed YT157 loopback workload');
}
const CSP = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";
const rows = { small: 8, medium: 128, large: 1024 }[workload];
const contentType = 'text/html; charset=utf-8';
const length = { small: 2383, medium: 31063, large: 245207 }[workload];
// Independent already-escaped literal oracle; prepared outside every HTTP iteration.
let expected = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>YT145 public report &lt;&amp;&gt; &quot;quoted&quot; &#39;café&#39;</title></head><body><main id="ssr-report"><header><h1>SSR report &lt;&amp;&gt; &quot;title&quot; &#39;café&#39;</h1><p>Public synthetic rows</p></header><section aria-label="Records"><ul>`;
for (let index = 0; index < rows; index++) {
  const i = String(index).padStart(4, '0');
  expected += `<li data-row="${i}" title="Row ${i} &lt;&amp;&gt; &quot;quoted&quot; &#39;café&#39;"><h2>Record ${i}</h2><p>Public &lt;&amp;&gt; &quot;value&quot; &#39;café&#39; ${i}</p><a href="/records/${i}?view=full&amp;public=1">Open record</a></li>`;
}
expected += `</ul></section><footer><p>End of public report</p></footer></main></body></html>`;

export const options = {
  scenarios: { requests: {
    executor: 'constant-arrival-rate', rate, timeUnit: '1s',
    duration: `${seconds}s`, preAllocatedVUs: 64, maxVUs: 64,
    gracefulStop: '8s',
  } },
  noConnectionReuse: false, noVUConnectionReuse: false,
  maxRedirects: 0, discardResponseBodies: false,
  systemTags: ['status', 'method', 'proto', 'scenario', 'error_code'],
  summaryTrendStats: ['count', 'min', 'med', 'p(95)', 'p(99)', 'max'],
  tags: { workload, mode, repetition, phase, rate: String(rate) },
};

const started = new Counter('yt157_started');
const httpDuration = new Trend('yt157_http_ms', true);
const fullDuration = new Trend('yt157_full_call_ms', true);

export default function () {
  const path = `/ssr/${workload}`;
  const probeId = `${phase === 'warmup' ? 'w' : 'm'}:${exec.scenario.iterationInTest}`;
  exec.vu.metrics.metadata.probe_id = probeId;
  const start = Date.now();
  exec.vu.metrics.metadata.start_ms = String(start);
  exec.vu.metrics.metadata.scenario_anchor_ms = String(exec.scenario.startTime);
  started.add(1);
  const response = http.get(`http://${targetIpv4}:${port}${path}`, {
    headers: { Accept: contentType.split(';')[0], 'Accept-Encoding': 'identity', 'X-Yt157-Id': probeId },
    redirects: 0, timeout: '8s', responseType: 'text',
  });
  const end = Date.now();
  const valid = response.status === 200 && response.proto === 'HTTP/1.1' &&
    response.body === expected && response.headers['Content-Type'] === contentType &&
    Number(response.headers['Content-Length']) === length &&
    !response.headers['Content-Encoding'] && !response.headers['Set-Cookie'] &&
    !response.headers.Server &&
    response.headers['Content-Security-Policy'] === CSP;
  exec.vu.metrics.metadata.completion_ms = String(end);
  const tags = { status: String(response.status),
    error_code: String(response.error_code || 0), valid: String(valid),
    proto: response.proto || 'none' };
  httpDuration.add(response.timings.duration, tags);
  fullDuration.add(end - start, tags);
  delete exec.vu.metrics.metadata.start_ms;
  delete exec.vu.metrics.metadata.scenario_anchor_ms;
  delete exec.vu.metrics.metadata.completion_ms;
  delete exec.vu.metrics.metadata.probe_id;
  if (response.status === 200 && !valid) {
    exec.test.abort('HTTP 200 correctness mismatch; retained raw failure');
  }
}
