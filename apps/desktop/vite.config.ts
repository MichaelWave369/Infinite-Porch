import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
// The daemon serves the live operator UI on the same origin. Vite is a static
// design preview: there is deliberately no privileged cross-origin API proxy.
export default defineConfig({plugins:[react()],build:{target:'es2022'},server:{host:'127.0.0.1'}});
