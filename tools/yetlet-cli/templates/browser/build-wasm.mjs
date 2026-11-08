import { spawn } from 'node:child_process';
import { resolve, join } from 'node:path';
import { fileURLToPath, pathToFileURL } from 'node:url';

const frontend = fileURLToPath(new URL('.', import.meta.url));
export const projectRoot = process.env.YETLET_PROJECT_ROOT || resolve(frontend, '..');

// Watch supplies a private output directory and cancellation; production retains
// its existing generated/ output. Children stay in the CLI-owned process tree.
export async function buildWasm(outDir = join(frontend, 'generated'), signal) {
  const css = await styleStage('prepare', undefined, signal);
  const bindgen = process.env.WASM_BINDGEN || 'wasm-bindgen';
  function run(command, args, capture = false) {
    return new Promise((accept, reject) => {
      const child = spawn(command, args, {
        cwd: projectRoot, signal, windowsHide: true,
        stdio: capture ? ['ignore', 'pipe', 'inherit'] : ['ignore', 'inherit', 'inherit'],
        env: { ...process.env, CARGO_NET_OFFLINE: 'true', RUSTUP_AUTO_INSTALL: '0', RUSTUP_SKIP_UPDATE_CHECK: '1' },
      });
      let stdout = '', error;
      child.stdout?.on('data', (data) => { stdout += data.toString(); });
      child.on('error', (cause) => { error = cause; });
      child.on('close', (code) => {
        if (error || code !== 0) reject(new Error(`${command} failed; prepare the Rust wasm32 target, locked dependencies and wasm-bindgen-cli 0.2.129 separately.`, { cause: error }));
        else accept(stdout);
      });
    });
  }
  if ((await run(bindgen, ['--version'], true)).trim() !== 'wasm-bindgen 0.2.129') throw new Error('Requires wasm-bindgen-cli 0.2.129.');
  await run('cargo', ['build', '--frozen', '--release', '--target', 'wasm32-unknown-unknown', '--lib']);
  const metadata = JSON.parse(await run('cargo', ['metadata', '--frozen', '--no-deps', '--format-version', '1'], true));
  await run(bindgen, [join(metadata.target_directory, 'wasm32-unknown-unknown/release/tasks.wasm'), '--target', 'web', '--out-name', 'tasks', '--out-dir', outDir]);
  return css;
}

if (process.argv[1] && import.meta.url === pathToFileURL(resolve(process.argv[1])).href) await buildWasm();

// Private Rust stage protocol. The CLI passes only its validated finite public
// selection; frontend environments never regain APP_* or secret selectors.
export function styleStage(operation, candidate, signal) {
  if (!process.env.YETLET_STYLE_CLI || !process.env.YETLET_INTERNAL_STYLE) return Promise.resolve(undefined);
  return new Promise((accept, reject) => {
    const child = spawn(process.env.YETLET_STYLE_CLI, ['__yetlet_style_worker', operation], {
      cwd: projectRoot, signal, windowsHide: true, stdio: ['pipe', 'pipe', 'inherit'],
    });
    const chunks = []; let length = 0, error;
    child.stdout.on('data', (chunk) => {
      length += chunk.length;
      if (length > 8 * 1024 * 1024) { error = new Error('Private style stage output exceeded its bound'); child.kill(); }
      else chunks.push(chunk);
    });
    child.stdin.on('error', (cause) => { error ??= cause; });
    child.on('error', (cause) => { error = cause; });
    child.on('close', (code) => {
      if (error || code !== 0) reject(new Error('Rust CSS stage failed; waiting for source correction', { cause: error }));
      else {
        try { accept(operation === 'prepare' ? JSON.parse(Buffer.concat(chunks).toString('utf8')) : undefined); }
        catch (cause) { reject(new Error('Invalid private CSS candidate', { cause })); }
      }
    });
    child.stdin.end(candidate === undefined ? undefined : JSON.stringify(candidate));
  });
}
