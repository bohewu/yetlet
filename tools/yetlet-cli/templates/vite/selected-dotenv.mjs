import { lstatSync } from 'node:fs';
import { join as dotenvPath } from 'node:path';

// Runs on every config resolution, before Vite reads frontend dotenv files.
const frontendDotenv = {
  name: 'yetlet-frontend-dotenv',
  enforce: 'pre',
  config(_config, { mode }) {
    for (const name of ['.env', '.env.local', `.env.${mode}`, `.env.${mode}.local`]) {
      if (name.includes('/') || name.includes('\\')) {
        throw new Error('Yetlet frontend dotenv must stay in the frontend root');
      }
      const stat = lstatSync(dotenvPath(root, name), { throwIfNoEntry: false });
      if (stat && (stat.isSymbolicLink() || !stat.isFile())) {
        throw new Error(`Yetlet frontend dotenv must be a regular, unlinked file: ${name}`);
      }
    }
  },
};
