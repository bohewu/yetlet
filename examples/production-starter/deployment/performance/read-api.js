import http from 'k6/http';
import exec from 'k6/execution';
import { Counter, Trend } from 'k6/metrics';

// One app-specific request per scheduled arrival; no custom scheduler or retry.
const port = Number(__ENV.YT141_PORT);
const rate = Number(__ENV.YT141_RATE);
const seconds = Number(__ENV.YT141_SECONDS);
const phase = __ENV.YT141_PHASE;
const legacy = [100, 500].includes(rate) && [5, 10].includes(seconds) &&
  /^(warmup|r[12]_(baseline|higher|recovery))$/.test(phase || '');
const warmup142 = phase === 'yt142_warmup' && rate === 250 && seconds === 5;
const paired142 = [1000, 2000, 4000, 8000, 16000].includes(rate) &&
  phase === `yt142_${rate}` && seconds === (rate === 16000 ? 5 : 10);
if (!Number.isInteger(port) || port < 1024 || port > 65535 ||
    !(legacy || warmup142 || paired142)) {
  throw new Error('Invalid YT141 loopback fixture or phase settings');
}
const token = open(__ENV.YT141_TOKEN_FILE).trim();
const expected = open(__ENV.YT141_EXPECTED_FILE);
if (!/^[0-9a-fA-F]{64}$/.test(token) || JSON.parse(expected).length !== 20) {
  throw new Error('Invalid YT141 private fixture input');
}

export const options = {
  scenarios: paired142 ? {
    load: {
      executor: 'constant-arrival-rate', rate, timeUnit: '1s',
      duration: `${seconds}s`, preAllocatedVUs: 64, maxVUs: 64,
      gracefulStop: '8s',
    },
    recovery: {
      executor: 'constant-arrival-rate', rate: 250, timeUnit: '1s',
      startTime: `${seconds}s`, duration: '30s',
      preAllocatedVUs: 16, maxVUs: 16, gracefulStop: '8s',
    },
  } : {
    reads: {
      executor: 'constant-arrival-rate', rate, timeUnit: '1s',
      duration: `${seconds}s`, preAllocatedVUs: 16, maxVUs: 16,
      gracefulStop: '8s',
    },
  },
  noConnectionReuse: false,
  noVUConnectionReuse: false,
  maxRedirects: 0,
  discardResponseBodies: false,
  systemTags: ['status', 'method', 'proto', 'scenario', 'error_code'],
  summaryTrendStats: ['count', 'min', 'med', 'p(95)', 'p(99)', 'max'],
  tags: { phase },
};

const started = new Counter('yt141_started');
const elapsed = new Trend('yt141_full_call_ms', true);
const successful = new Trend('yt141_valid_200_ms', true);

export default function () {
  const start = Date.now();
  exec.vu.metrics.metadata.start_ms = String(start);
  exec.vu.metrics.metadata.scheduled_start_ms = String(exec.scenario.startTime);
  started.add(1);
  const response = http.get(`http://127.0.0.1:${port}/api/issues`, {
    headers: { Authorization: `Bearer ${token}` },
    redirects: 0, timeout: '8s', responseType: 'text',
  });
  const duration = Date.now() - start;
  const valid = response.status === 200 && response.body === expected;
  // Per-point metadata is retained by JSON output without making each start
  // timestamp an aggregate metric label. Never include headers or body.
  const tags = {
    status: String(response.status), error_code: String(response.error_code || 0),
    valid: String(valid), proto: response.proto || 'none',
  };
  elapsed.add(duration, tags);
  if (valid) successful.add(duration, tags);
  delete exec.vu.metrics.metadata.start_ms;
  delete exec.vu.metrics.metadata.scheduled_start_ms;
}
