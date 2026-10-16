import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// A GitHub Pages project site lives at /Infinite-Porch/, while localhost
// development keeps the normal root path. A misspelled path should fail the
// build instead of publishing blank HTML that cannot load its assets.
const base = process.env.HOUSE_BASE_PATH ?? '/';
if (base !== '/' && base !== '/Infinite-Porch/') {
  throw new Error('Unsupported Infinite House public base path: ' + base);
}
export default defineConfig({ plugins: [react()], base });
