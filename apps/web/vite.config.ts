import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'
import path from 'path'
import { readFileSync } from 'fs'

const backend = process.env.KINEMA_BACKEND ?? 'https://127.0.0.1:3000'
const root = JSON.parse(readFileSync(path.resolve(__dirname, '../../package.json'), 'utf8'))

export default defineConfig({
  define: {
    __APP_VERSION__: JSON.stringify(root.version),
  },
  plugins: [react()],
  resolve: {
    alias: { '@': path.resolve(__dirname, './src') },
  },
  server: {
    port: 5173,
    proxy: {
      '/api': { target: backend, changeOrigin: true, secure: false, ws: true },
    },
  },
  build: {
    target: 'es2022',
    rollupOptions: {
      output: {
        manualChunks: {
          react: ['react', 'react-dom', 'react-router-dom'],
        },
      },
    },
  },
})
