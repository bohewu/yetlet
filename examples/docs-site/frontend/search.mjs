import { createStartupShell } from './wasm-startup.mjs';
const root = document.getElementById('site-search');
const status = document.getElementById('site-search-status');
const input = document.getElementById('site-search-input');
const results = document.getElementById('site-search-results');
const shell = createStartupShell({ root, status, messages: {
  loading: 'Loading search…', ready: '',
  error: 'Search unavailable. Reload to retry.',
} });
let runtime, initialized = false;
const closeResults = () => { results.hidden = true; };
const outside = event => { if (!root.contains(event.target)) closeResults(); };
const escape = event => {
  if (event.key === 'Escape') { event.preventDefault(); input.focus({ preventScroll: true }); closeResults(); }
};
const blur = event => { if (!root.contains(event.relatedTarget)) closeResults(); };
const focus = () => { if (input.value.trim() && results.childElementCount) results.hidden = false; };
document.addEventListener('pointerdown', outside);
root.addEventListener('keydown', escape);
root.addEventListener('focusout', blur);
root.addEventListener('focusin', focus);
async function startSearch() {
  const ticket = shell.begin();
  if (!ticket) return;
  try {
    const [module, data] = await Promise.all([import('./docs_search.js'), import('./search-data.mjs')]);
    runtime = module;
    await runtime.default();
    initialized = true;
    if (shell.state === 'disposed') { runtime.dispose(); return; }
    runtime.activate(JSON.stringify(data.default));
    shell.ready(ticket);
  } catch {
    if (initialized) { try { runtime.dispose(); } catch { /* Static navigation remains available. */ } }
    shell.fail(ticket);
  }
}
window.addEventListener('pagehide', event => {
  if (!event.persisted) {
    document.removeEventListener('pointerdown', outside);
    root.removeEventListener('keydown', escape);
    root.removeEventListener('focusout', blur);
    root.removeEventListener('focusin', focus);
    closeResults(); shell.dispose(); if (initialized) runtime.dispose();
  }
});
void startSearch();
