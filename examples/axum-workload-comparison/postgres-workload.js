import http from 'k6/http';
import exec from 'k6/execution';
import { Counter, Trend } from 'k6/metrics';

const started = new Counter('yt144_started');
const fullCall = new Trend('yt144_full_call_ms', true);
const httpDuration = new Trend('yt144_http_ms', true);
const ids = new Trend('yt144_request_id');
const valid = new Counter('yt144_valid');
const tags = { mode: __ENV.YT144_MODE, repetition: __ENV.YT144_REPETITION, phase: __ENV.YT144_PHASE, offered: __ENV.YT144_RATE };
export const options = {
  scenarios: { fixed: { executor: 'constant-arrival-rate', rate: Number(__ENV.YT144_RATE), timeUnit: '1s', duration: `${__ENV.YT144_SECONDS}s`, preAllocatedVUs: 64, maxVUs: 64, gracefulStop: '3s' } },
  maxRedirects: 0,
  discardResponseBodies: false,
  summaryTrendStats: ['min', 'med', 'p(95)', 'p(99)', 'max'],
  systemTags: ['status', 'method', 'name', 'proto', 'error_code', 'expected_response', 'scenario'],
  tags,
};
export default function () {
  const id = 1 + (exec.scenario.iterationInTest % 10000);
  const start = Date.now();
  exec.vu.metrics.metadata.start_ms = String(start);
  exec.vu.metrics.metadata.scenario_anchor_ms = String(exec.scenario.startTime);
  started.add(1);
  const response = http.get(`http://127.0.0.1:${__ENV.YT144_PORT}/db/${id}`, {
    headers: { Accept: 'application/json', 'Accept-Encoding': 'identity' },
    timeout: '2s', redirects: 0, tags: { name: '/db/{id}' },
  });
  const ok = response.status === 200 && response.body === `{"id":${id},"randomNumber":${1 + ((id * 7919) % 10000)}}` &&
    response.headers['Content-Type'] === 'application/json' && response.headers['Content-Length'] === String(response.body.length) &&
    !response.headers['Content-Encoding'] && !response.headers['Set-Cookie'] && !response.headers.Server && response.proto === 'HTTP/1.1';
  const end = Date.now();
  exec.vu.metrics.metadata.completion_ms = String(end);
  const result = { status: String(response.status), valid: String(ok), error_code: String(response.error_code || 0) };
  fullCall.add(end - start, result);
  httpDuration.add(response.timings.duration, result);
  ids.add(id, result); // Numeric value retains each ID without 10,000 native metric series.
  valid.add(ok ? 1 : 0, result);
  delete exec.vu.metrics.metadata.start_ms;
  delete exec.vu.metrics.metadata.scenario_anchor_ms;
  delete exec.vu.metrics.metadata.completion_ms;
  if (response.status === 200 && !ok) { exec.test.abort("HTTP 200 correctness mismatch; raw failure retained"); }
}
