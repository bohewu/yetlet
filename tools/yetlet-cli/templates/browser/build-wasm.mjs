import { spawnSync } from 'node:child_process';
import { resolve, join } from 'node:path';
import { fileURLToPath } from 'node:url';

const frontend = fileURLToPath(new URL('.', import.meta.url));
const root = process.env.YETLET_PROJECT_ROOT || resolve(frontend, '..');
const bindgen = process.env.WASM_BINDGEN || 'wasm-bindgen';
function run(command, args, capture = false) {
  const result = spawnSync(command, args, { cwd: root, stdio: capture ? 'pipe' : 'inherit', encoding: 'utf8', env: { ...process.env, CARGO_NET_OFFLINE: 'true', RUSTUP_AUTO_INSTALL: '0' } });
  if (result.error || result.status !== 0) throw new Error(`${command} failed; prepare the Rust wasm32 target, locked dependencies and wasm-bindgen-cli 0.2.129 separately.`, { cause: result.error });
  return result.stdout;
}
if (run(bindgen, ['--version'], true).trim() !== 'wasm-bindgen 0.2.129') throw new Error('Requires wasm-bindgen-cli 0.2.129.');
run('cargo', ['build', '--frozen', '--release', '--target', 'wasm32-unknown-unknown', '--lib']);
const metadata = JSON.parse(run('cargo', ['metadata', '--frozen', '--no-deps', '--format-version', '1'], true));
run(bindgen, [join(metadata.target_directory, 'wasm32-unknown-unknown/release/tasks.wasm'), '--target', 'web', '--out-name', 'tasks', '--out-dir', join(frontend, 'generated')]);
