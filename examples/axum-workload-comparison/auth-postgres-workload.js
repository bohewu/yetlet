import http from 'k6/http';
import exec from 'k6/execution';
import { Counter, Trend } from 'k6/metrics';

// Explicitly selected form-authenticated application; no credential acquisition here.
const mode = __ENV.YT170_MODE;
const staircase = __ENV.YT171_STEP !== undefined;
const timeline = __ENV.YT173_STEP !== undefined;
if (timeline && (staircase || !['observe', 'burst'].includes(__ENV.YT173_STEP))) throw new Error('Invalid fixed YT173 selection');
const steps = {
  admission: { rate: 125, seconds: 2, phase: 'warmup' },
  'load-250': { rate: 250, seconds: 20, phase: 'measured' },
  'load-500': { rate: 500, seconds: 20, phase: 'measured' },
  'load-1000': { rate: 1000, seconds: 20, phase: 'measured' },
  'load-2000': { rate: 2000, seconds: 20, phase: 'measured' },
  recovery: { rate: 100, seconds: 10, phase: 'recovery' },
};
if (staircase && !Object.prototype.hasOwnProperty.call(steps, __ENV.YT171_STEP)) throw new Error('Invalid fixed YT171 step');
const selected = staircase ? steps[__ENV.YT171_STEP] : null;
const phase = timeline ? __ENV.YT173_STEP : (staircase ? selected.phase : __ENV.YT170_PHASE);
const port = Number(__ENV.YT170_PORT);
if (!['json', 'ssr'].includes(mode) || !(timeline ? ['observe', 'burst'] : (staircase ? ['warmup', 'measured', 'recovery'] : ['warmup', 'measured'])).includes(phase) || !Number.isInteger(port) || port < 1024 || port > 65535) {
  throw new Error('Invalid fixed YT170 workload');
}
const credentials = JSON.parse(open(__ENV.YT170_CREDENTIAL_FILE));
if (!Array.isArray(credentials.cookies) || credentials.cookies.length !== 8 || !credentials.cookies.every(value => /^yt170_session=[a-f0-9]{64}$/.test(value))) {
  throw new Error('Invalid private fixture credentials');
}
const rate = timeline ? (phase === 'burst' ? 0 : (mode === 'json' ? 500 : 2000)) : (staircase ? selected.rate : (phase === 'warmup' ? 25 : 100));
const seconds = timeline ? (phase === 'burst' ? 4 : 5) : (staircase ? selected.seconds : (phase === 'warmup' ? 2 : 20));
const vus = timeline ? (phase === 'burst' ? 16 : 128) : (staircase ? 128 : 8);
const CSP = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";
const oracles = Array.from({ length: 8 }, (_, index) => {
  const id = index + 1;
  if (mode === 'json') return JSON.stringify({ id, title: `Report ${id} <&>`, body: `Private <&> "quoted" 'café' ${id}` });
  return `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>Report ${id} &lt;&amp;&gt;</title></head><body><main id="private-document"><h1>Report ${id} &lt;&amp;&gt;</h1><p data-document="${id}">Private &lt;&amp;&gt; &quot;quoted&quot; &#39;café&#39; ${id}</p></main></body></html>`;
});
export const options = {
  scenarios: { requests: timeline && phase === 'burst' ? { executor: 'per-vu-iterations', vus: 16, iterations: 1, maxDuration: '4s', gracefulStop: '4s' } : { executor: 'constant-arrival-rate', rate, timeUnit: '1s', duration: `${seconds}s`, preAllocatedVUs: vus, maxVUs: vus, gracefulStop: '4s' } },
  noConnectionReuse: false, noVUConnectionReuse: false, maxRedirects: 0,
  discardResponseBodies: false,
  summaryTrendStats: ['count', 'min', 'med', 'p(95)', 'p(99)', 'max'],
  systemTags: ['status', 'method', 'name', 'proto', 'error_code', 'scenario'],
  tags: { mode, phase, rate: String(rate), ...(timeline ? { step: __ENV.YT173_STEP } : (staircase ? { step: __ENV.YT171_STEP } : {})) },
};
const started = new Counter('yt170_started');
const valid = new Counter('yt170_valid');
const fullDuration = new Trend('yt170_full_call_ms', true);
const httpDuration = new Trend('yt170_http_ms', true);
export default function () {
  const index = exec.scenario.iterationInTest % 8;
  const id = index + 1;
  const path = mode === 'json' ? `/api/documents/${id}` : `/private/documents/${id}`;
  const contentType = mode === 'json' ? 'application/json' : 'text/html; charset=utf-8';
  const start = Date.now();
  const probe = timeline ? (phase === 'burst' ? 20000 + exec.vu.idInTest : exec.scenario.iterationInTest + 1) : exec.scenario.iterationInTest;
  exec.vu.metrics.metadata.probe_id = String(probe);
  exec.vu.metrics.metadata.scenario_anchor_ms = String(exec.scenario.startTime);
  exec.vu.metrics.metadata.start_ms = String(start);
  started.add(1);
  const response = http.get(`http://127.0.0.1:${port}${path}`, {
    headers: { Cookie: credentials.cookies[(exec.vu.idInTest - 1) % 8], Accept: contentType.split(';')[0], 'Accept-Encoding': 'identity', ...(timeline ? { 'X-Yetlet-Probe': String(probe) } : {}) },
    timeout: '3s', redirects: 0, tags: { name: mode === 'json' ? '/api/documents/{id}' : '/private/documents/{id}' },
  });
  const expected = oracles[index];
  const ok = response.status === 200 && response.proto === 'HTTP/1.1' && response.body === expected &&
    response.headers['Content-Type'] === contentType && Number(response.headers['Content-Length']) === expected.length + 1 && // One é contributes two UTF-8 bytes.
    response.headers['Cache-Control'] === 'no-store' && String(response.headers.Vary || '').split(',').some(value => value.trim().toLowerCase() === 'cookie') &&
    response.headers['X-Content-Type-Options'] === 'nosniff' && response.headers['Content-Security-Policy'] === CSP &&
    !response.headers['Set-Cookie'] && !response.headers['Content-Encoding'] && !response.headers.Server;
  const end = Date.now();
  exec.vu.metrics.metadata.completion_ms = String(end);
  const result = { status: String(response.status), valid: String(ok), error_code: String(response.error_code || 0) };
  httpDuration.add(response.timings.duration, result);
  fullDuration.add(end - start, result);
  valid.add(ok ? 1 : 0, result);
  delete exec.vu.metrics.metadata.probe_id;
  delete exec.vu.metrics.metadata.scenario_anchor_ms;
  delete exec.vu.metrics.metadata.start_ms;
  delete exec.vu.metrics.metadata.completion_ms;
  if (response.status === 200 && !ok) exec.test.abort('HTTP200 oracle mismatch; raw failure retained');
}
