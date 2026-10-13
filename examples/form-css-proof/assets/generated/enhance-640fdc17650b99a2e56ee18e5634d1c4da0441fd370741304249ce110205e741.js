// Application-owned, opt-in preview. Never sets HTML or replaces form controls.
const owners = new WeakMap();

export function dispose(root) {
  const owner = owners.get(root);
  if (!owner) return;
  owner.active = false;
  owner.abort.abort();
  for (const timer of owner.timers) clearTimeout(timer);
  owner.timers.clear();
  owners.delete(root);
  root.dataset.previewState = "disposed";
}

export function mount(root) {
  dispose(root);
  const form = root.querySelector("form");
  const summary = root.querySelector("[data-preview-summary]");
  if (!form || !summary) throw new Error("invalid form proof boundary");
  const owner = { active: true, abort: new AbortController(), composing: new Set(), timers: new Set() };
  owners.set(root, owner);
  const current = () => owner.active && owners.get(root) === owner && root.isConnected;
  const render = () => {
    if (!current() || owner.composing.size) return;
    const title = form.elements.namedItem("title");
    const notes = form.elements.namedItem("notes");
    const priority = form.elements.namedItem("priority");
    const notify = form.elements.namedItem("notify");
    summary.textContent = `${title.value || "Untitled"}\nPriority: ${priority.value}; notifications: ${notify.checked ? "on" : "off"}\n${notes.value}`;
    root.dataset.previewUpdates = String(Number(root.dataset.previewUpdates || "0") + 1);
  };
  const listen = (target, type, fn) => target.addEventListener(type, fn, { signal: owner.abort.signal });
  listen(form, "compositionstart", event => owner.composing.add(event.target));
  listen(form, "compositionend", event => { owner.composing.delete(event.target); render(); });
  listen(form, "input", render);
  listen(form, "change", render);
  // Reset's default action follows dispatch. Re-read the live controls afterwards.
  listen(form, "reset", () => queueMicrotask(render));
  const later = root.querySelector("[data-preview-later]");
  if (later) listen(later, "click", () => {
    const timer = setTimeout(() => { owner.timers.delete(timer); render(); }, 180);
    owner.timers.add(timer);
  });
  root.dataset.previewState = "mounted";
  render();
}

for (const root of document.querySelectorAll("[data-form-proof]")) {
  const activate = root.querySelector("[data-preview-activate]");
  const stop = root.querySelector("[data-preview-dispose]");
  activate?.addEventListener("click", () => mount(root));
  stop?.addEventListener("click", () => dispose(root));
  // These two loader controls own no form values; only mount owns form listeners.
  window.addEventListener("pagehide", () => dispose(root), { once: true });
}
