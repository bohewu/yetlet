import { createStartupShell } from './wasm-startup.mjs';

// Public browser configuration only; no native settings or secrets are imported.
document.getElementById('workspace-label')!.textContent =
  import.meta.env.VITE_WORKSPACE_LABEL || 'Local workspace';
const shell = createStartupShell({
  root: document.getElementById('main')!,
  status: document.getElementById('runtime-status')!,
  indicator: document.getElementById('startup-indicator')!,
  controls: document.getElementById('workspace')! as HTMLFieldSetElement,
  messages: { error: 'Unable to start the browser runtime. Check JavaScript, WebAssembly and the site files, then reload.' },
});
let active = true;
let runtime: { dispose_tasks: () => void } | undefined;
window.addEventListener('pagehide', event => {
  if (event.persisted) return;
  active = false;
  shell.dispose();
  runtime?.dispose_tasks();
  runtime = undefined;
});

async function start() {
  const ticket = shell.begin();
  if (!ticket) return;
  try {
    const module = await import('./generated/tasks.js');
    if (!active) return;
    const { default: wasmUrl } = await import('./generated/tasks_bg.wasm?url');
    if (!active) return;
    await module.default({ module_or_path: wasmUrl });
    if (!active) { module.dispose_tasks(); return; }
    runtime = module;
    module.mount_tasks();
    shell.ready(ticket);
  } catch (error) {
    runtime?.dispose_tasks();
    runtime = undefined;
    if (active) shell.fail(ticket);
    console.error('Task runtime initialization failed', error);
  }
}
void start();
