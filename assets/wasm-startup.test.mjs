import test from 'node:test';
import assert from 'node:assert/strict';
import { createStartupShell } from './wasm-startup.mjs';

function elements() {
  const root = { dataset: {}, attributes: {}, setAttribute(k, v) { this.attributes[k] = v; } };
  const status = { textContent: 'Initial custom loading', hidden: false };
  const indicator = { hidden: false }, controls = { disabled: true };
  return { root, status, indicator, controls };
}
test('static shell stays intact until explicitly begun, then ready enables only owned controls', () => {
  const e = elements(), other = elements();
  const shell = createStartupShell({ ...e, messages: { ready: 'My application is ready' } });
  assert.equal(e.status.textContent, 'Initial custom loading');
  assert.deepEqual(e.root.dataset, {});
  const ticket = shell.begin();
  assert.equal(shell.begin(), null);
  assert.equal(e.root.attributes['aria-busy'], 'true');
  assert.equal(e.controls.disabled, true);
  assert.equal(shell.ready(ticket), true);
  assert.equal(e.status.textContent, 'My application is ready');
  assert.equal(e.controls.disabled, false);
  assert.equal(other.controls.disabled, true);
  assert.equal(e.indicator.hidden, true);
});
test('a late ready cannot erase error; retries reject old tickets and cancellation retires the current one', () => {
  const e = elements(), shell = createStartupShell(e), old = shell.begin();
  assert.equal(shell.fail(old), true);
  assert.equal(shell.ready(old), false);
  const fresh = shell.begin('Explicit retry');
  assert.equal(shell.fail(old), false);
  assert.equal(shell.ready(fresh), true);
  assert.equal(shell.cancel(fresh), true);
  assert.equal(shell.fail(fresh), false);
});
test('dispose invalidates late work and permits an explicitly new root owner', () => {
  const e = elements(), first = createStartupShell(e), old = first.begin();
  assert.throws(() => createStartupShell(e), /already has an owner/);
  first.dispose();
  assert.equal(first.begin(), null);
  const replacement = createStartupShell(e), fresh = replacement.begin('New owner');
  assert.equal(first.ready(old), false);
  assert.equal(first.fail(old), false);
  first.dispose();
  assert.equal(e.status.textContent, 'New owner');
  assert.equal(replacement.ready(fresh), true);
});
test('custom status is text, with no HTML sink or asynchronous resource management', () => {
  const e = elements();
  Object.defineProperty(e.status, 'innerHTML', { set() { throw Error('HTML sink used'); } });
  const shell = createStartupShell(e), ticket = shell.begin('<img src=x onerror=run()>');
  assert.equal(e.status.textContent, '<img src=x onerror=run()>');
  shell.cancel(ticket);
  assert.equal(shell.state, 'idle');
  assert.equal(e.controls.disabled, true);
});
