import { createStartupShell } from './wasm-startup.mjs';
const root = document.getElementById('island-panel');
const status = document.getElementById('island-status');
const button = document.getElementById('enable-counter');
const shell = createStartupShell({ root, status, messages: {
  loading: 'Loading the optional counter…', ready: 'Counter ready.',
  error: 'Counter unavailable. The guide remains readable; reload to retry.',
} });
let runtime;
let initialized = false;
button.addEventListener('click', async () => {
  const ticket = shell.begin();
  if (!ticket) return;
  button.disabled = true;
  try {
    runtime = await import('./static_island.js');
    await runtime.default();
    initialized = true;
    if (shell.state === 'disposed') { runtime.dispose(); return; }
    runtime.activate();
    shell.ready(ticket);
    button.hidden = true;
  } catch {
    if (initialized) { try { runtime.dispose(); } catch { /* Keep static failure guidance usable. */ } }
    shell.fail(ticket);
  }
});
window.addEventListener('pagehide', (event) => {
  if (!event.persisted) { shell.dispose(); if (initialized) runtime.dispose(); }
});
