import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import tailwindcss from '@tailwindcss/vite'

export default defineConfig({
  plugins: [react(), tailwindcss()],
  server: {
    port: 5173,
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:8080',
        changeOrigin: true,
      },
    },
  },
  build: {
    rollupOptions: {
      output: {
        manualChunks(id) {
          if (id.includes('cytoscape')) {
            return 'cytoscape-vendor'
          }
          if (id.includes('recharts')) {
            return 'chart-vendor'
          }
          if (id.includes('node_modules')) {
            return 'vendor'
          }
        },
      },
    },
    chunkSizeWarningLimit: 500,
  },
})
