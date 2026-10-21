import init, { mount_triage, dispose_triage } from './generated/yetlet_board_triage.js';
import wasmUrl from './generated/yetlet_board_triage_bg.wasm?url';
import type { StatusChart } from './status-chart';
import type { DescriptionEditor } from './description-editor';

let stop: (() => void) | undefined;

// Keep dynamic imports in the entry to avoid a shared Vite preload helper
// introducing a cycle in the application's bounded asset manifest.
export async function start(
  loadChart: () => Promise<typeof import('./status-chart')>,
  loadEditor: () => Promise<typeof import('./description-editor')>,
) {
  stop?.();
  let active = true;
  let chart: StatusChart | undefined;
  let editor: DescriptionEditor | undefined;
  const retire = () => {
    if (!active) return;
    active = false;
    window.removeEventListener('pagehide', onPageHide);
    dispose_triage();
    chart?.destroy();
    chart = undefined;
    editor?.destroy();
    editor = undefined;
    document.getElementById('triage-workspace')?.removeAttribute('data-ready');
  };
  const onPageHide = (event: PageTransitionEvent) => {
    if (!event.persisted) retire();
  };
  stop = retire;
  // Guard the entire asynchronous startup, including imports before Rust mounts.
  window.addEventListener('pagehide', onPageHide);
  try {
    await init({ module_or_path: wasmUrl });
    if (!active) return;
    try {
      const { createStatusChart } = await loadChart();
      if (!active) return;
      const host = document.getElementById('triage-status-chart');
      if (host) chart = createStatusChart(host);
    } catch {
      // Chart failure leaves the native summary and Rust workspace available.
    }
    try {
      const nonce = document.querySelector<HTMLMetaElement>('meta[property="csp-nonce"]')?.nonce;
      if (nonce && /^[a-f0-9]{32}$/.test(nonce)) {
        const { createDescriptionEditor } = await loadEditor();
        if (!active) return;
        const host = document.getElementById('triage-description-editor');
        if (host) editor = createDescriptionEditor(host, nonce);
      }
    } catch {
      // Missing packages/styles leave the native description editable in Rust.
    }
    if (!active) { chart?.destroy(); editor?.destroy(); return; }
    mount_triage(chart, editor);
    chart = undefined; // The Rust runtime now owns the adapter's destruction.
    editor = undefined;
  } catch (error) {
    retire();
    throw error;
  }
}
