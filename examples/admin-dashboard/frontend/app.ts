import './app.css';

// Only this route loads the Rust interaction implementation and WASM bytes.
if (document.getElementById('triage-workspace')) {
  import('./triage').then(module => module.start()).catch(() => {
    const status = document.getElementById('triage-message');
    if (status) status.textContent = 'Interactive workspace unavailable. Open an item to use the native view and editor.';
  });
}
