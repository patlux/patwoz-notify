import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
import { VitePWA } from 'vite-plugin-pwa'

// https://vitejs.dev/config/
export default defineConfig({
  server: {
    host: '0.0.0.0',
    proxy: {
      '/api': {
        target: 'http://localhost:3000',
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    manifest: true,
  },
  plugins: [
    react(),
    VitePWA({
      filename: 'sw.ts',
      srcDir: '.',
      injectRegister: false,
      manifest: false,
      strategies: 'injectManifest',
      injectManifest: { injectionPoint: undefined },
      registerType: 'autoUpdate',
      devOptions: {
        enabled: true,
      },
    }),
  ],
})
