import init, { mount_triage, dispose_triage } from './generated/yetlet_board_triage.js';
import wasmUrl from './generated/yetlet_board_triage_bg.wasm?url';

export async function start() {
  await init({ module_or_path: wasmUrl });
  mount_triage();
  window.addEventListener('pagehide', event => {
    if (!event.persisted) {
      dispose_triage();
      document.getElementById('triage-workspace')?.removeAttribute('data-ready');
    }
  });
}
