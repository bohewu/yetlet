// Explicit enhancement of the existing shared SSR View; no DOM replacement.
import init, { enhance } from './showcase.js';
try {
  await init(new URL('./showcase_bg.wasm', import.meta.url));
  enhance();
} catch (error) {
  document.getElementById('enhancement-status').textContent =
    'Local enhancement could not load. Theme, density and selection still work through native links.';
  console.error(error);
}
