// Application-owned lifecycle; assets remain inert until explicit activation.
const root = document.getElementById("scene-island");
const canvas = document.getElementById("scene-canvas");
const status = document.getElementById("scene-status");
const activateButton = document.getElementById("activate-scene");
const disposeButton = document.getElementById("dispose-scene");
const selection = document.getElementById("scene-selection");
const detailLink = document.getElementById("scene-detail-link");
const navigateButton = document.getElementById("scene-navigate");
const resetButton = document.getElementById("scene-reset");
let focusedRecord = 1;
let selectedRecord = null;
let tap = null;
let rejectedPicks = 0;
const recordCount = () => current?.runtime && !current.runtimeDisposed ? current.runtime.cpu_bytes() / 32 : 120;
function resetSelection() {
  tap = null;
  focusedRecord = 1;
  selectedRecord = null;
  navigateButton.textContent = "Select record 001";
  selection.textContent = "No scene record selected.";
  detailLink.hidden = true;
  detailLink.removeAttribute("href");
}
function showSelection(id) {
  if (!Number.isInteger(id) || id < 1 || id > recordCount()) return false;
  selectedRecord = id;
  focusedRecord = id;
  navigateButton.textContent = `Select record ${String(id).padStart(3, "0")}`;
  selection.textContent = `Selected work record ${String(id).padStart(3, "0")}.`;
  // The SSR product owns 120 records. Larger research batches have no SSR URL.
  detailLink.hidden = id > 120;
  if (id <= 120) detailLink.setAttribute("href", `/scene?record=${id}`);
  else detailLink.removeAttribute("href");
  return true;
}
function capturePick(clientX, clientY) {
  const owner = current;
  if (!alive(owner) || owner.runtimeDisposed || !owner.runtime || owner.state !== "ready") return null;
  const box = canvas.getBoundingClientRect();
  return Array.from(owner.runtime.pick(clientX - box.left, clientY - box.top, box.width, box.height, window.devicePixelRatio));
}
function commitPick(token) {
  const owner = current;
  if (!Array.isArray(token) || token.length !== 3 || !token.every(value => Number.isInteger(value) && value >= 0 && value <= 0xffffffff)
      || !alive(owner) || owner.runtimeDisposed || !owner.runtime
      || !owner.runtime.accepts_pick(...token)) { rejectedPicks++; return false; }
  return showSelection(token[2]);
}
let current = null;
let serial = 0;
let modulePromise = null;
let starts = 0;
let pending = 0;
let lateSettled = 0;
let lastRetired = null;
let retainedCPU = null;
let moduleTimes = null;
const alive = owner => !!owner && current === owner && !owner.retired;
const setStatus = (owner, state, message) => {
  if (!alive(owner)) return;
  owner.state = state;
  root.dataset.sceneState = state;
  status.textContent = message;
};
const stats = owner => owner.runtime && !owner.runtimeDisposed ? {
  configurations: owner.runtime.configurations(), draws: owner.runtime.draws(),
  deviceLimit: owner.runtime.device_limit(), bufferBytes: owner.runtime.buffer_bytes(),
  cpuCapacityBytes: owner.runtime.cpu_capacity_bytes(), metrics: Array.from(owner.runtime.metrics())
} : owner.stats;
function stopWatching(owner) {
  if (owner.raf !== null) cancelAnimationFrame(owner.raf);
  owner.raf = null;
  for (const remove of owner.cleanup.splice(0)) remove();
  owner.observers = 0;
  owner.listeners = 0;
}
function releaseRuntime(owner) {
  if (!owner.runtime) return;
  if (!owner.runtimeDisposed) {
    owner.stats = stats(owner);
    owner.runtimeDisposed = true;
    // The extracted renderer releases its own configured surface exactly once.
    owner.configuredCanvas = false;
    owner.runtime.dispose();
  }
  if (owner.settled && owner.completions === 0) { owner.runtime.free(); owner.runtime = null; }
}
function dispose() {
  const owner = current;
  current = null; // Invalidate generation before cancel/destroy and late completion.
  resetSelection();
  if (owner) {
    owner.retired = true;
    stopWatching(owner);
    releaseRuntime(owner);
    lastRetired = { generation: owner.id, rafPending: false, observers: 0, listeners: 0,
      bufferBytes: 0, resourcesReleased: owner.settled && owner.completions === 0 };
  }
  root.dataset.sceneState = "disposed";
  status.textContent = "GPU disposed. HTML records remain available.";
}
function fail(owner, category) {
  if (!alive(owner) || ["lost", "error", "unavailable"].includes(owner.state)) return;
  owner.fault = category;
  tap = null;
  stopWatching(owner);
  releaseRuntime(owner);
  const state = category.endsWith("lost") ? "lost" : "error";
  setStatus(owner, state, `GPU ${state} (${category}). HTML records remain available. Activate GPU overview to retry explicitly.`);
}
// Rust's callbacks retain their own generation, never an ambient renderer.
window.__yt076Fault = (generation, category) => queueMicrotask(() => {
  if (current?.id === generation) fail(current, category);
});
function paused(owner) {
  return document.hidden || !owner.visible || !root.isConnected || !canvas.isConnected;
}
function invalidate(owner) {
  if (!alive(owner) || !owner.settled || !owner.runtime || !["ready", "suspended"].includes(owner.state)) return;
  if (paused(owner)) {
    if (owner.raf !== null) cancelAnimationFrame(owner.raf);
    owner.raf = null;
    setStatus(owner, "suspended", "GPU paused while hidden, offscreen or detached. HTML records remain available.");
    return;
  }
  if (owner.raf !== null) { owner.coalesced++; return; }
  owner.raf = requestAnimationFrame(() => {
    owner.raf = null;
    if (!alive(owner) || paused(owner)) { invalidate(owner); return; }
    try {
      const box = canvas.getBoundingClientRect();
      owner.css = { width: box.width, height: box.height };
      owner.dpr = Math.min(2, Math.max(1, window.devicePixelRatio));
      if (!owner.runtime.resize(box.width, box.height, window.devicePixelRatio, canvas.isConnected)) {
        setStatus(owner, "suspended", "GPU paused for a zero-sized target. HTML records remain available.");
        return;
      }
      owner.configuredCanvas = true;
      owner.runtime.draw();
      owner.completedDraw = performance.now();
      setStatus(owner, "ready", `GPU overview ready: ${owner.runtime.cpu_bytes() / 32} synthetic records. Use the HTML list for detail.`);
    } catch (error) { fail(owner, String(error)); }
  });
}
function watch(owner) {
  const listen = (target, name, handler) => {
    target.addEventListener(name, handler);
    owner.listeners++;
    owner.cleanup.push(() => target.removeEventListener(name, handler));
  };
  const resize = new ResizeObserver(() => invalidate(owner));
  resize.observe(canvas);
  const intersection = new IntersectionObserver(entries => {
    owner.visible = entries[entries.length - 1].isIntersecting;
    invalidate(owner);
  });
  intersection.observe(canvas);
  let connected = canvas.isConnected && root.isConnected;
  const detach = new MutationObserver(() => {
    const next = canvas.isConnected && root.isConnected;
    if (next !== connected) { connected = next; invalidate(owner); }
  });
  detach.observe(document.documentElement, { childList: true, subtree: true });
  owner.observers = 3;
  owner.cleanup.push(() => resize.disconnect(), () => intersection.disconnect(), () => detach.disconnect());
  listen(document, "visibilitychange", () => invalidate(owner));
  listen(window, "resize", () => invalidate(owner));
  // A resolution media query detects monitor DPR changes without a perpetual RAF.
  let query = null;
  const changed = () => {
    query?.removeEventListener("change", changed);
    query = matchMedia(`(resolution: ${window.devicePixelRatio}dppx)`);
    query.addEventListener("change", changed);
    invalidate(owner);
  };
  changed();
  owner.listeners++;
  owner.cleanup.push(() => query?.removeEventListener("change", changed));
}
async function activate() {
  if (current && !["lost", "error", "unavailable"].includes(current.state)) return;
  if (current) dispose(); // Explicit recovery alone creates a fresh generation.
  const owner = { id: ++serial, runtime: null, retired: false, settled: false,
    state: "initializing", visible: true, raf: null, cleanup: [], observers: 0, listeners: 0,
    coalesced: 0, completions: 0, css: null, dpr: null, fault: null, stats: null };
  current = owner;
  starts++;
  pending++;
  root.dataset.initializations = String(starts);
  setStatus(owner, "initializing", "Initializing optional WebGPU overview…");
  watch(owner);
  try {
    if (!navigator.gpu) throw new Error("WebGPU unavailable in this context");
    modulePromise ??= loadModule();
    const module = await modulePromise;
    if (!alive(owner)) return;
    owner.runtime = new module.SceneOwner(owner.id);
    if (retainedCPU) owner.runtime.replace(retainedCPU);
    const adapter = await owner.runtime.initialize(canvas);
    if (!alive(owner) || owner.state !== "initializing") return;
    root.dataset.adapter = adapter;
    setStatus(owner, "ready", "GPU overview initialized; waiting for a visible target.");
  } catch (_) {
    if (alive(owner) && owner.state === "initializing") {
      stopWatching(owner);
      releaseRuntime(owner);
      setStatus(owner, "unavailable", "WebGPU or its assets are unavailable. Search and open records in the HTML list. Activate GPU overview to retry explicitly.");
    }
  } finally {
    owner.settled = true;
    pending--;
    if (owner.retired || ["lost", "error", "unavailable"].includes(owner.state)) {
      releaseRuntime(owner);
      if (owner.retired) {
        lateSettled++;
        if (lastRetired?.generation === owner.id) lastRetired.resourcesReleased = true;
      }
    } else { invalidate(owner); }
  }
}
async function loadModule() {
  const start = performance.now();
  try {
    const module = await import("./scene.js");
    const imported = performance.now();
    await module.default();
    moduleTimes = { importMs: imported - start, wasmInitMs: performance.now() - imported };
    return module;
  } catch (error) { modulePromise = null; throw error; }
}
activateButton.addEventListener("click", activate);
disposeButton.addEventListener("click", dispose);
// Input remains application-local, with no text/IME interception or drag/capture.
document.getElementById("selection-controls").hidden = false;
const selectFocused = () => showSelection(focusedRecord);
const navigate = event => {
  if (event.isComposing || event.altKey || event.ctrlKey || event.metaKey) return;
  const count = recordCount();
  const delta = { ArrowLeft: -1, ArrowRight: 1, ArrowUp: -12, ArrowDown: 12 }[event.key];
  if (delta !== undefined) focusedRecord = Math.max(1, Math.min(count, focusedRecord + delta));
  else if (event.key === "Home") focusedRecord = 1;
  else if (event.key === "End") focusedRecord = count;
  else return;
  event.preventDefault();
  navigateButton.textContent = `Select record ${String(focusedRecord).padStart(3, "0")}`;
};
const pointerDown = event => {
  tap = null;
  if (!event.isPrimary || event.button !== 0) return;
  const token = capturePick(event.clientX, event.clientY);
  if (token) tap = { pointerId: event.pointerId, x: event.clientX, y: event.clientY, token };
};
const pointerMove = event => {
  if (tap?.pointerId === event.pointerId && Math.hypot(event.clientX - tap.x, event.clientY - tap.y) > 6) tap = null;
};
const cancelTap = () => { tap = null; };
const pointerUp = event => {
  const pendingTap = tap;
  tap = null;
  if (!pendingTap || pendingTap.pointerId !== event.pointerId
      || Math.hypot(event.clientX - pendingTap.x, event.clientY - pendingTap.y) > 6) return;
  const token = capturePick(event.clientX, event.clientY);
  if (!token || token[0] !== pendingTap.token[0] || token[1] !== pendingTap.token[1]) { rejectedPicks++; return; }
  // A deferred result must pass the generation/version check at the text sink.
  queueMicrotask(() => commitPick(token));
};
navigateButton.addEventListener("keydown", navigate);
navigateButton.addEventListener("click", selectFocused);
resetButton.addEventListener("click", resetSelection);
canvas.addEventListener("pointerdown", pointerDown);
canvas.addEventListener("pointermove", pointerMove);
canvas.addEventListener("pointerup", pointerUp);
canvas.addEventListener("pointercancel", cancelTap);
canvas.addEventListener("lostpointercapture", cancelTap);
window.addEventListener("scroll", cancelTap, { passive: true, capture: true });
window.addEventListener("pagehide", () => {
  dispose();
  activateButton.removeEventListener("click", activate);
  disposeButton.removeEventListener("click", dispose);
  navigateButton.removeEventListener("keydown", navigate);
  navigateButton.removeEventListener("click", selectFocused);
  resetButton.removeEventListener("click", resetSelection);
  canvas.removeEventListener("pointerdown", pointerDown);
  canvas.removeEventListener("pointermove", pointerMove);
  canvas.removeEventListener("pointerup", pointerUp);
  canvas.removeEventListener("pointercancel", cancelTap);
  canvas.removeEventListener("lostpointercapture", cancelTap);
  window.removeEventListener("scroll", cancelTap, { capture: true });
}, { once: true });
window.__yt078Selection = Object.freeze({ capturePick, commitPick,
  snapshot: () => ({ focusedRecord, selectedRecord, rejectedPicks, tapPending: tap !== null,
    generation: current?.id ?? null, dataVersion: current?.runtime && !current.runtimeDisposed ? current.runtime.data_version() : null,
    reducedMotion: matchMedia("(prefers-reduced-motion: reduce)").matches }) });
// Typed application diagnostics: explicit handles/bytes, never measured VRAM.
window.__yt075Scene = Object.freeze({
  // Application-owned dynamic entry; data never enters clone-per-dispatch LocalGraph.
  prepareModule: () => { modulePromise ??= loadModule(); return modulePromise.then(() => moduleTimes); },
  records: async (count, seed) => { const module = await modulePromise; return module.scene_records(count, seed); },
  patch: async (data, start, length, seed, step) => {
    const module = await modulePromise; return module.status_patch(data, start, length, seed, step);
  },
  replace: data => {
    // Rust validates before retaining a replacement or scheduling affected GPU work.
    if (!current?.runtime || !alive(current)) throw new Error("GPU not ready");
    current.runtime.replace(data);
    resetSelection();
    retainedCPU = data.slice();
    invalidate(current);
  },
  update: (start, data) => {
    if (!current?.runtime || !alive(current)) throw new Error("GPU not ready");
    if (!retainedCPU) throw new Error("Dynamic replacement required before updates");
    current.runtime.update(start, data);
    tap = null;
    retainedCPU.set(data, start * 32);
    invalidate(current);
  },
  completion: async () => {
    const owner = current;
    if (!owner?.runtime || !alive(owner)) throw new Error("GPU not ready");
    owner.completions++;
    try { await owner.runtime.completion(); }
    finally {
      owner.completions--;
      if (owner.runtimeDisposed) {
        releaseRuntime(owner);
        if (lastRetired?.generation === owner.id) {
          lastRetired.resourcesReleased = owner.settled && owner.completions === 0;
        }
      }
    }
  },
  disposeDynamic: () => { dispose(); retainedCPU = null; },
  dispose,
  activate,
  moduleTimes: () => moduleTimes,
  draw: () => { if (current) invalidate(current); },
  snapshot: () => ({ state: root.dataset.sceneState ?? "inert", starts,
    generation: current?.id ?? null, pendingInitializations: pending, lateSettled,
    pendingCompletions: current?.completions ?? 0,
    rafPending: current?.raf !== null && current?.raf !== undefined,
    observers: current?.observers ?? 0, listeners: current?.listeners ?? 0,
    visible: current?.visible ?? null, hidden: document.hidden,
    css: current?.css ?? null, dpr: current?.dpr ?? null,
    backing: { width: canvas.width, height: canvas.height },
    coalesced: current?.coalesced ?? 0, fault: current?.fault ?? null,
    ...(current ? stats(current) : null),
    cpuRecordBytes: current?.runtime?.cpu_bytes() ?? retainedCPU?.byteLength ?? 3840,
    retainedJSBytes: retainedCPU?.byteLength ?? 0, completedDraw: current?.completedDraw ?? null,
    moduleTimes,
    ownedSurfaces: current?.configuredCanvas ? 1 : 0,
    ownedBufferBytes: current?.runtime && current.settled && ["ready", "suspended"].includes(current.state) ? current.runtime.buffer_bytes() : 0,
    lastRetired })
});
