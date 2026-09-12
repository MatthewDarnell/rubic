import react from '@vitejs/plugin-react';
import { defineConfig } from 'vite';

export default defineConfig({
  base: './', // Ensures relative paths for assets in production
  build: {
    outDir: 'dist', // Match your Electron build configuration
  },
  plugins: [react()],
  server: {
    // Tauri's devUrl is http://localhost:5173 and WebView2 resolves that to IPv4;
    // Node binds "localhost" to ::1 by default, so pin the address.
    host: '127.0.0.1',
    port: 5173,
    strictPort: true,
    proxy: {
      '/api': {
        // If your frontend requests /api/qubic-price
        target: 'https://rpc.qubic.org', // This is the target CoinGecko API
        changeOrigin: true, // Needed for virtual hosted sites
        rewrite: (path) => path.replace(/^\/api/, ''), // Remove /api prefix when sending to target
        secure: true, // Use false if you're targeting http, true for https (default)
      },
    },
  },
});
