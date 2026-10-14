// Application-owned lifecycle; assets remain inert until explicit activation.
const root = document.getElementById("scene-island");
const canvas = document.getElementById("scene-canvas");
const status = document.getElementById("scene-status");
const activateButton = document.getElementById("activate-scene");
const disposeButton = document.getElementById("dispose-scene");
let current = null;
let serial = 0;
let modulePromise = null;
let starts = 0;
let pending = 0;
let lateSettled = 0;
let lastRetired = null;
let retainedCPU = null;
let moduleTimes = null;
const alive = owner => current === owner && !owner.retired;
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
    // wgpu 27's WebSurface Drop is a no-op. Release this generation's context
    // explicitly, before a replacement can configure it; pending owners never do.
    if (owner.configuredCanvas) {
      canvas.getContext("webgpu")?.unconfigure();
      owner.configuredCanvas = false;
    }
    owner.runtime.dispose();
  }
  if (owner.settled && owner.completions === 0) { owner.runtime.free(); owner.runtime = null; }
}
function dispose() {
  const owner = current;
  current = null; // Invalidate generation before cancel/destroy and late completion.
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
window.addEventListener("pagehide", () => {
  dispose();
  activateButton.removeEventListener("click", activate);
  disposeButton.removeEventListener("click", dispose);
}, { once: true });
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
    retainedCPU = data.slice();
    invalidate(current);
  },
  update: (start, data) => {
    if (!current?.runtime || !alive(current)) throw new Error("GPU not ready");
    if (!retainedCPU) throw new Error("Dynamic replacement required before updates");
    current.runtime.update(start, data);
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
