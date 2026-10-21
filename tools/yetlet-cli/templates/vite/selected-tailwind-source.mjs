import { resolve } from 'node:path';

// CLI execution supplies its selected app root, independent of frontend nesting.
// Direct npm at the default frontend/ location uses the literal ../src fallback.
const rustSource = {
  name: 'yetlet-rust-source',
  enforce: 'pre',
  transform(source, id) {
    const normalize = value => process.platform === 'win32' ? value.replaceAll('\\', '/') : value;
    const css = normalize(fileURLToPath(new URL('app.css', import.meta.url)));
    if (id.split('?')[0] !== css || !process.env.YETLET_PROJECT_ROOT) return null;
    const path = normalize(resolve(process.env.YETLET_PROJECT_ROOT, 'src/main.rs'));
    const escaped = path.replace(/["\\\n\r\f]/g, char => `\\${char.codePointAt(0).toString(16)} `);
    return source.replace('@source "../src/main.rs";', `@source "${escaped}";`);
  },
};
