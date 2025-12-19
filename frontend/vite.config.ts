import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

export default defineConfig({
  plugins: [sveltekit()],
  
  // Tauri expects a fixed port for development
  server: {
    port: 5173,
    strictPort: true,
  },
  
  // Prevent vite from obscuring rust errors
  clearScreen: false,
  
  // Tauri expects environment variables to be prefixed with VITE_
  envPrefix: ['VITE_', 'TAURI_'],
});
