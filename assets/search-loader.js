// Lightweight external bootstrap: no remote event bridge and no eager Wasm.
// Register only after the SSR island exists, then lazy-load on local interaction.
function installSearchIsland() {
  const root = document.querySelector('[data-yetlet-island="search"]');
  if (!root) return;
  let activation;
  const activate = () => {
    if (!activation) {
      activation = import("./yetlet_search.js")
        .then(async ({ default: init, mount_search }) => {
          await init();
          mount_search();
        })
        .catch(error => {
          // Leave existing SSR markup available if the island fails to boot.
          root.setAttribute("data-yetlet-error", "load-failed");
          console.error("Yetlet search island failed", error);
        });
    }
    return activation;
  };
  root.addEventListener("focusin", activate, { once: true });
  root.addEventListener("input", activate, { once: true });
  root.setAttribute("data-yetlet-loader-ready", "true");
}
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", installSearchIsland, { once: true });
} else {
  installSearchIsland();
}
