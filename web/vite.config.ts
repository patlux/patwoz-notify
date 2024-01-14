import react from '@vitejs/plugin-react-swc'
import { defineConfig } from 'vite'
import { VitePWA } from 'vite-plugin-pwa'
import tsconfigPaths from 'vite-tsconfig-paths'

// https://vitejs.dev/config/
export default defineConfig({
  server: {
    host: '0.0.0.0',
    proxy: {
      '/api': {
        target: 'http://127.0.0.1:3000',
        changeOrigin: true,
      },
    },
  },
  build: {
    outDir: 'dist',
    manifest: true,
  },
  plugins: [
    tsconfigPaths(),
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
