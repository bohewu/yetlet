import init, { mount_counter, dispose_counter } from "./yetlet_dashboard_counter.js";
try { await init(); mount_counter(); } catch (_) {
  document.getElementById("counter-status").textContent = "Counter assets unavailable; HTML records remain available.";
}
window.addEventListener("pagehide", dispose_counter, { once: true });
