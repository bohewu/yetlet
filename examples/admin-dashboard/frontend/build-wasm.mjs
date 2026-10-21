// Application build step, not a new Yetlet CLI mode or framework compiler.
import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { homedir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
const workspace = fileURLToPath(new URL('../../../', import.meta.url));
const output = fileURLToPath(new URL('generated/', import.meta.url));
function run(command, args, capture = false) {
  const result = spawnSync(command, args, { cwd: workspace, stdio: capture ? 'pipe' : 'inherit', encoding: 'utf8' });
  if (result.error || result.status !== 0) throw new Error(`${command} failed; install the wasm32-unknown-unknown Rust target and matching wasm-bindgen-cli 0.2.129`, { cause: result.error });
  return result.stdout;
}
const fallback = join(homedir(), '.local/share/yetlet-tools/bin', process.platform === 'win32' ? 'wasm-bindgen.exe' : 'wasm-bindgen');
const bindgen = process.env.WASM_BINDGEN || (existsSync(fallback) ? fallback : 'wasm-bindgen');
if (run(bindgen, ['--version'], true).trim() !== 'wasm-bindgen 0.2.129') throw new Error('This application requires wasm-bindgen-cli 0.2.129');
run('cargo', ['build', '--locked', '--release', '--target', 'wasm32-unknown-unknown', '-p', 'yetlet-board-triage']);
const metadata = JSON.parse(run('cargo', ['metadata', '--locked', '--no-deps', '--format-version', '1'], true));
run(bindgen, [join(metadata.target_directory, 'wasm32-unknown-unknown/release/yetlet_board_triage.wasm'), '--target', 'web', '--out-dir', output]);
