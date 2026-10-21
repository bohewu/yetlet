import './app.css';
import init, { mount_tasks } from './generated/tasks.js';
import wasmUrl from './generated/tasks_bg.wasm?url';

// This value is public browser configuration. Native APP_* settings are not imported.
document.getElementById('workspace-label')!.textContent =
  import.meta.env.VITE_WORKSPACE_LABEL || 'Local workspace';

try {
  await init({ module_or_path: wasmUrl });
  mount_tasks();
} catch (error) {
  console.error("Task runtime initialization failed", error);
  document.getElementById('runtime-status')!.textContent =
    'Unable to start the browser runtime. Check that JavaScript, WebAssembly and static assets are available.';
}
