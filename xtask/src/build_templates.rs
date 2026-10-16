// Frozen literal browser assets and HTML from the YT-084 legacy source snapshot.
pub const BROWSER_JS: &str = r#####"import init, { dispose, mount } from "./yetlet_browser.js";
await init();
globalThis.__yetletTodoRuntime = Object.freeze({ dispose, mount });
mount();
"#####;
pub const BROWSER_CSS: &str = r#####"
:root { color-scheme: light dark; font-family: system-ui, sans-serif; }
body { max-width: 48rem; margin: 0 auto; padding: 1.5rem; line-height: 1.6; }
.is-complete { text-decoration: line-through; opacity: .65; }
button { font: inherit; padding: .5rem .9rem; cursor: pointer; }
button:focus-visible { outline: 3px solid Highlight; outline-offset: 2px; }
"#####;
pub const BROWSER_ICON: &str = r#####"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><rect width="16" height="16" fill="#263238"/><path d="M3 4l5 8 5-8" stroke="white" stroke-width="2" fill="none"/></svg>
"#####;
pub const BROWSER_PAGE: &str = r#####"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>Yetlet | Browser Todo</title>
<link rel="stylesheet" href="@@styles.css@@">
<link rel="icon" href="@@favicon.svg@@" type="image/svg+xml">
<script type="module" src="@@app.js@@"></script>
</head>
<body>
<div id="app"><p>Loading browser runtime...</p></div>
<noscript>JavaScript is required. <a href="/">Open the SSR page</a>.</noscript>
</body>
</html>
"#####;
pub const KEYED_BOOTSTRAP: &str = r#####"import init, * as api from "./yetlet_keyed_bindings.js";
await init();
window.yetletBindings = api;
document.documentElement.setAttribute("data-yetlet-m7b-ready", "true");
document.documentElement.setAttribute("data-yetlet-m7c-ready", "true");
document.documentElement.setAttribute("data-yetlet-m7d-ready", "true");
"#####;
pub const KEYED_STYLES: &str = r#####"
:root { color-scheme: light dark; font-family: system-ui, sans-serif; }
body { max-width: 52rem; margin: 0 auto; padding: 1.5rem; line-height: 1.5; }
.m7b-list { display: grid; gap: .5rem; padding: 0; list-style: none; }
.m7b-row { display: grid; grid-template-columns: minmax(12rem, 1fr) minmax(10rem, 1fr) auto; gap: .5rem; align-items: center; }
.m7b-details { grid-column: 1 / -1; font-size: .9rem; opacity: .8; }
button { font: inherit; }
input { font: inherit; min-width: 0; }
"#####;
pub const KEYED_ICON: &str = r#####"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 16 16"><rect width="16" height="16" fill="#263238"/><path d="M3 4l5 8 5-8" stroke="white" stroke-width="2" fill="none"/></svg>
"#####;
pub const KEYED_PAGE: &str = r#####"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta http-equiv="Content-Security-Policy" content="default-src 'self'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; connect-src 'self'; object-src 'none'; base-uri 'none'">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Yetlet M7b/M7c/M7d Keyed Bindings, Reactivity and Lifecycle</title>
  <link rel="stylesheet" href="@@styles.css@@">
  <link rel="icon" href="@@favicon.svg@@" type="image/svg+xml">
  <script type="module" src="@@prototype.js@@"></script>
</head>
<body>
  <h1>Yetlet M7b/M7c/M7d Keyed Binding, Reactivity and Lifecycle Prototype</h1>
  <p>This fixture is isolated from production Browser and Search bundles.</p>
  <label for="outside-focus">Outside focus sentinel</label>
  <input id="outside-focus" type="text" value="outside">
  <div id="prototype-host">@@ssr@@</div>
</body>
</html>
"#####;
pub const DASHBOARD_COUNTER_LOADER: &str = r#####"function installCounterIsland() {
  const root = document.querySelector('[data-yetlet-island="counter"]');
  if (!root) return;
  const dashboard = globalThis.__yetletDashboard ??= {};
  let modulePromise;
  let active = false;
  const load = () => modulePromise ??= import("./yetlet_dashboard_counter.js")
    .then(async module => { await module.default(); return module; });
  const activate = async () => {
    if (active) return;
    try {
      const module = await load();
      module.mount_counter();
      active = true;
      root.removeAttribute("data-yetlet-error");
    } catch (error) {
      active = false;
      root.setAttribute("data-yetlet-error", "mount-failed");
      console.error("Yetlet counter island failed", error);
      throw error;
    }
  };
  const dispose = async () => {
    if (modulePromise) {
      try { (await modulePromise).dispose_counter(); } catch (error) {
        root.setAttribute("data-yetlet-error", "dispose-failed");
        throw error;
      }
    }
    active = false;
  };
  dashboard.counter = Object.freeze({ activate, mount: activate, dispose });
  root.addEventListener("focusin", () => { activate().catch(() => {}); });
  root.addEventListener("yetlet-activate", () => { activate().catch(() => {}); });
  root.setAttribute("data-yetlet-loader-ready", "true");
}
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", installCounterIsland, { once: true });
} else {
  installCounterIsland();
}
"#####;
pub const DASHBOARD_FILTER_LOADER: &str = r#####"function installFilterIsland() {
  const root = document.querySelector('[data-yetlet-island="filter"]');
  if (!root) return;
  const dashboard = globalThis.__yetletDashboard ??= {};
  let modulePromise;
  let active = false;
  const load = () => modulePromise ??= import("./yetlet_dashboard_filter.js")
    .then(async module => { await module.default(); return module; });
  const activate = async () => {
    if (active) return;
    try {
      const module = await load();
      module.mount_filter();
      active = true;
      root.removeAttribute("data-yetlet-error");
    } catch (error) {
      active = false;
      root.setAttribute("data-yetlet-error", "mount-failed");
      console.error("Yetlet filter island failed", error);
      throw error;
    }
  };
  const dispose = async () => {
    if (modulePromise) {
      try { (await modulePromise).dispose_filter(); } catch (error) {
        root.setAttribute("data-yetlet-error", "dispose-failed");
        throw error;
      }
    }
    active = false;
  };
  dashboard.filter = Object.freeze({ activate, mount: activate, dispose });
  root.addEventListener("focusin", () => { activate().catch(() => {}); });
  root.addEventListener("yetlet-activate", () => { activate().catch(() => {}); });
  root.setAttribute("data-yetlet-loader-ready", "true");
}
if (document.readyState === "loading") {
  document.addEventListener("DOMContentLoaded", installFilterIsland, { once: true });
} else {
  installFilterIsland();
}
"#####;
pub const DASHBOARD_CSS: &str = r#####":root { color-scheme: light dark; font-family: system-ui, sans-serif; }
body { max-width: 64rem; margin: 0 auto; padding: 1.5rem; line-height: 1.5; }
#dashboard-page { display: grid; gap: 1rem; }
section { border: 1px solid CanvasText; border-radius: .5rem; padding: 1rem; }
button, input { font: inherit; margin: .25rem; padding: .4rem .7rem; }
:focus-visible { outline: 3px solid Highlight; outline-offset: 2px; }
"#####;
pub const BENCH_BOOTSTRAP: &str = r#####"import init, { run_benchmark } from "./yetlet_render_bench.js";
await init();
window.runYetletBenchmark = (rows, updates, rounds, seed) =>
  JSON.parse(run_benchmark(rows, updates, rounds, seed));
document.documentElement.dataset.benchmarkReady = "true";
document.getElementById("run-default").addEventListener("click", () => {
  const output = document.getElementById("output");
  output.textContent = "Running...";
  setTimeout(() => {
    try {
      output.textContent = JSON.stringify(
        window.runYetletBenchmark(200, 500, 10, 7), null, 2
      );
    } catch (error) {
      output.textContent = String(error);
    }
  }, 0);
});
"#####;
pub const BENCH_STYLES: &str = r#####":root { color-scheme: light dark; font-family: system-ui, sans-serif; }
body { max-width: 68rem; margin: 0 auto; padding: 1.5rem; line-height: 1.5; }
button { font: inherit; padding: .5rem .9rem; }
#bench-root { position: absolute; left: -100000px; top: 0; }
pre { max-height: 36rem; overflow: auto; padding: 1rem; border: 1px solid; }
"#####;
pub const BENCH_PAGE: &str = r#####"<!doctype html>
<html lang="en">
<head>
  <meta charset="utf-8">
  <meta name="viewport" content="width=device-width, initial-scale=1">
  <title>Yetlet M7a Render Benchmark</title>
  <link rel="icon" href="data:,">
  <link rel="stylesheet" href="@@styles.css@@">
  <script type="module" src="@@bench.js@@"></script>
</head>
<body>
  <h1>Yetlet M7a Render Benchmark</h1>
  <p>This isolated fixture is not a production route or public renderer API.</p>
  <button type="button" id="run-default">Run default benchmark</button>
  <pre id="output">Ready after the benchmark module loads.</pre>
  <div id="bench-root" aria-hidden="true"></div>
</body>
</html>
"#####;
