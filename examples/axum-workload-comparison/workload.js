import http from 'k6/http';
import exec from 'k6/execution';
import { Counter, Trend } from 'k6/metrics';

const port = Number(__ENV.YT143_PORT);
const rate = Number(__ENV.YT143_RATE);
const seconds = Number(__ENV.YT143_SECONDS);
const workload = __ENV.YT143_WORKLOAD;
const mode = __ENV.YT143_MODE;
const repetition = __ENV.YT143_REPETITION;
const phase = __ENV.YT143_PHASE;
if (!Number.isInteger(port) || port < 1024 || port > 65535 ||
    !['plaintext', 'json', 'ssr'].includes(workload) ||
    !['axum', 'yetlet'].includes(mode) || !['1', '2'].includes(repetition) ||
    !((phase === 'warmup' && rate === 250 && seconds === 2) ||
      (phase === 'measured' && [1000, 4000].includes(rate) && seconds === 10))) {
  throw new Error('Invalid fixed YT143 loopback workload');
}
const CSP = "default-src 'none'; script-src 'self'; style-src 'self'; connect-src 'self'; img-src 'self'; base-uri 'none'; object-src 'none'; form-action 'self'; frame-ancestors 'none'";
const FIRST = `<!doctype html><html lang="en"><head><meta charset="utf-8"><meta name="viewport" content="width=device-width, initial-scale=1"><title>YT143 01 &lt;&amp;&gt; &quot;quoted&quot; &#39;title&#39; café</title></head><body><main data-id="01" title="Item 01 &lt;&amp;&gt; &quot;quoted&quot; &#39;attribute&#39; café"><h1>Item 01 &lt;&amp;&gt; &quot;quoted&quot; &#39;text&#39; café</h1><p>Dynamic request 01</p></main></body></html>`;
const contentType = { plaintext: 'text/plain; charset=utf-8', json: 'application/json', ssr: 'text/html; charset=utf-8' }[workload];
const length = { plaintext: 13, json: 27, ssr: 428 }[workload];

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

const started = new Counter('yt143_started');
const httpDuration = new Trend('yt143_http_ms', true);
const fullDuration = new Trend('yt143_full_call_ms', true);

export default function () {
  const id = workload === 'ssr' ? 1 + exec.scenario.iterationInTest % 20 : 0;
  const number = String(id).padStart(2, '0');
  const expected = workload === 'ssr' ? FIRST.replace(/01/g, number) :
    workload === 'json' ? '{"message":"Hello, World!"}' : 'Hello, World!';
  const path = workload === 'ssr' ? `/ssr/${id}` : `/${workload}`;
  const start = Date.now();
  exec.vu.metrics.metadata.start_ms = String(start);
  exec.vu.metrics.metadata.scenario_anchor_ms = String(exec.scenario.startTime);
  started.add(1, { id: String(id) });
  const response = http.get(`http://127.0.0.1:${port}${path}`, {
    headers: { Accept: contentType.split(';')[0], 'Accept-Encoding': 'identity' },
    redirects: 0, timeout: '8s', responseType: 'text',
  });
  const end = Date.now();
  const valid = response.status === 200 && response.proto === 'HTTP/1.1' &&
    response.body === expected && response.headers['Content-Type'] === contentType &&
    Number(response.headers['Content-Length']) === length &&
    !response.headers['Content-Encoding'] && !response.headers['Set-Cookie'] &&
    !response.headers.Server &&
    (workload === 'ssr' ? response.headers['Content-Security-Policy'] === CSP :
      !response.headers['Content-Security-Policy']);
  exec.vu.metrics.metadata.completion_ms = String(end);
  const tags = { id: String(id), status: String(response.status),
    error_code: String(response.error_code || 0), valid: String(valid),
    proto: response.proto || 'none' };
  httpDuration.add(response.timings.duration, tags);
  fullDuration.add(end - start, tags);
  delete exec.vu.metrics.metadata.start_ms;
  delete exec.vu.metrics.metadata.scenario_anchor_ms;
  delete exec.vu.metrics.metadata.completion_ms;
  if (response.status === 200 && !valid) {
    exec.test.abort('HTTP 200 correctness mismatch; retained raw failure');
  }
}
