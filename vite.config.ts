import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vuetify from 'vite-plugin-vuetify'
import wasm from 'vite-plugin-wasm'

// https://vitejs.dev/config/
export default defineConfig((configEnv) => ({
  plugins: [
    vue(),
    vuetify({ autoImport: true }),
    wasm(),
  ],
  resolve: {
    alias: {
      '@': '/src'
    }
  }
}))
