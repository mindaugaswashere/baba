import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

// The frontend dev server runs on :5174 (5173 is taken) and proxies every
// /api/* request to the Rust backend on :8080. Because the browser only ever
// talks to :5174, the auth cookie stays same-origin — no CORS, no cross-site
// cookie headaches.
export default defineConfig({
  plugins: [react()],
  server: {
    port: 5174,
    strictPort: true,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
})
