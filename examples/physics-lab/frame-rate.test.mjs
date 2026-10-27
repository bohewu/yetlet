import test from 'node:test';
import assert from 'node:assert/strict';
import { createFrameRateCounter } from './frame-rate.mjs';

test('warmup is unknown; one second of sixty submitted draws estimates sixty', () => {
  const counter = createFrameRateCounter();
  counter.reset(0);
  for (let i = 1; i < 60; i++) counter.record(i * 1000 / 60);
  assert.equal(counter.fps, null);
  counter.record(1000);
  assert.equal(counter.fps, 60);
});
test('only submitted draws count, and reset discards idle time and old samples', () => {
  const counter = createFrameRateCounter();
  counter.reset(0);
  counter.record(250);
  counter.record(1000);
  assert.equal(counter.fps, 2);
  counter.reset(10000);
  assert.equal(counter.fps, null);
  counter.record(11000);
  assert.equal(counter.fps, 1);
});
test('invalid or backward time clears the estimate without poisoning later samples', () => {
  const counter = createFrameRateCounter();
  counter.reset(0);
  counter.record(1000);
  counter.record(NaN);
  assert.equal(counter.fps, null);
  counter.record(2000);
  counter.record(3000);
  assert.equal(counter.fps, 1);
  counter.record(2500);
  assert.equal(counter.fps, null);
  counter.record(3500);
  assert.equal(counter.fps, 1);
});
