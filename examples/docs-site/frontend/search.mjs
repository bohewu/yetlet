import { createStartupShell } from './wasm-startup.mjs';
const root = document.getElementById('site-search');
const status = document.getElementById('site-search-status');
const button = document.getElementById('site-search-enable');
const shell = createStartupShell({ root, status, messages: {
  loading: 'Loading optional Wasm search…', ready: 'Search ready. All content stays local.',
  error: 'Search unavailable. Keep browsing the guide, or reload to retry.',
} });
let runtime, initialized = false;
button.addEventListener('click', async () => {
  const ticket = shell.begin();
  if (!ticket) return;
  button.disabled = true;
  try {
    const [module, data] = await Promise.all([import('./docs_search.js'), import('./search-data.mjs')]);
    runtime = module;
    await runtime.default();
    initialized = true;
    if (shell.state === 'disposed') { runtime.dispose(); return; }
    runtime.activate(JSON.stringify(data.default));
    shell.ready(ticket);
    button.hidden = true;
  } catch {
    if (initialized) { try { runtime.dispose(); } catch { /* Static navigation remains available. */ } }
    shell.fail(ticket);
  }
});
button.disabled = false;
window.addEventListener('pagehide', event => {
  if (!event.persisted) { shell.dispose(); if (initialized) runtime.dispose(); }
});
