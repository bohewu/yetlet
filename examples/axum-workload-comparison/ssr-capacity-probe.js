import http from 'k6/http';
import exec from 'k6/execution';
import { Counter, Trend } from 'k6/metrics';

// YT-165 fixed capacity protocol; same SSR response oracle as the path study.
const plan = {
  'warmup-soak-5000': { rate: 125, seconds: 2, index: 0 },
  'soak-5000': { rate: 5000, seconds: 1800, index: 1 },
  'warmup-sustained-5000': { rate: 125, seconds: 2, index: 0 },
  'sustained-5000': { rate: 5000, seconds: 300, index: 1 },
  'warmup-sustained-6000': { rate: 125, seconds: 2, index: 0 },
  'sustained-6000': { rate: 6000, seconds: 300, index: 1 },
  'warmup-5000': { rate: 125, seconds: 2, index: 0 },
  'refine-5000': { rate: 5000, seconds: 10, index: 1 },
  'warmup-6000': { rate: 125, seconds: 2, index: 2 },
  'refine-6000': { rate: 6000, seconds: 10, index: 3 },
  'warmup-7000': { rate: 125, seconds: 2, index: 4 },
  'refine-7000': { rate: 7000, seconds: 10, index: 5 },
  warmup: { rate: 125, seconds: 2, index: 0 },
  'ramp-2000': { rate: 2000, seconds: 10, index: 1 },
  'ramp-4000': { rate: 4000, seconds: 10, index: 2 },
  'ramp-8000': { rate: 8000, seconds: 10, index: 3 },
  'ramp-12000': { rate: 12000, seconds: 10, index: 4 },
  'ramp-16000': { rate: 16000, seconds: 10, index: 5 },
  'recovery-immediate': { rate: 1000, seconds: 10, index: 6 },
  'recovery-after-idle': { rate: 1000, seconds: 10, index: 7 },
};
const step = __ENV.YT165_STEP;
if (!Object.prototype.hasOwnProperty.call(plan, step)) throw new Error('Invalid fixed YT165 step');
const { rate, seconds, index: stepIndex } = plan[step];
const port = Number(__ENV.YT157_PORT);
const targetIpv4 = __ENV.YT164_TARGET_IPV4;
const octets = /^(0|[1-9]\d{0,2})\.(0|[1-9]\d{0,2})\.(0|[1-9]\d{0,2})\.(0|[1-9]\d{0,2})$/.exec(targetIpv4 || '');
const ipv4 = octets && octets.slice(1).map(Number);
if (port !== 39262 || !ipv4 || ipv4.some(value => value > 255) ||
    !(ipv4[0] === 10 || (ipv4[0] === 172 && ipv4[1] >= 16 && ipv4[1] <= 31) ||
      (ipv4[0] === 192 && ipv4[1] === 168))) {
  throw new Error('YT165 requires the declared high port and private IPv4');
}
const workload = 'medium';
const mode = 'current';
const repetition = '1';
const phase = (step === 'warmup' || step.startsWith('warmup-')) ? 'warmup' : 'measured';
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
    duration: `${seconds}s`, preAllocatedVUs: 256, maxVUs: 256,
    gracefulStop: '8s',
  } },
  noConnectionReuse: false, noVUConnectionReuse: false,
  maxRedirects: 0, discardResponseBodies: false,
  systemTags: ['status', 'method', 'proto', 'scenario', 'error_code'],
  summaryTrendStats: ['count', 'min', 'med', 'p(95)', 'p(99)', 'max'],
  tags: { workload, mode, repetition, phase, step, rate: String(rate) },
};

const started = new Counter('yt157_started');
const validResponse = new Counter('yt165_valid_response');
const httpDuration = new Trend('yt157_http_ms', true);
const fullDuration = new Trend('yt157_full_call_ms', true);

export default function () {
  const path = `/ssr/${workload}`;
  const probeId = `${phase === 'warmup' ? 'w' : 'm'}:${exec.scenario.iterationInTest}`;
  exec.vu.metrics.metadata.probe_id = probeId;
  const iteration = exec.scenario.iterationInTest;
  const sampleEvery = step === 'soak-5000' ? 16384 : 4096;
  const serverProbeId = iteration % sampleEvery === 0
    ? `${phase === 'warmup' ? 'w' : 'm'}:${(stepIndex * 64 + Math.floor(iteration / sampleEvery)) * 32}`
    : null;
  if (serverProbeId) exec.vu.metrics.metadata.server_probe_id = serverProbeId;
  const start = Date.now();
  exec.vu.metrics.metadata.start_ms = String(start);
  exec.vu.metrics.metadata.scenario_anchor_ms = String(exec.scenario.startTime);
  started.add(1);
  const headers = { Accept: contentType.split(';')[0], 'Accept-Encoding': 'identity' };
  if (serverProbeId) headers['X-Yt157-Id'] = serverProbeId;
  const response = http.get(`http://${targetIpv4}:${port}${path}`, {
    headers,
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
  validResponse.add(valid ? 1 : 0, tags);
  delete exec.vu.metrics.metadata.start_ms;
  delete exec.vu.metrics.metadata.scenario_anchor_ms;
  delete exec.vu.metrics.metadata.completion_ms;
  delete exec.vu.metrics.metadata.probe_id;
  delete exec.vu.metrics.metadata.server_probe_id;
  if (response.status === 200 && !valid) {
    exec.test.abort('HTTP 200 correctness mismatch; retained raw failure');
  }
}
