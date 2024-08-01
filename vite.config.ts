import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import vuetify from 'vite-plugin-vuetify'
import eslintPlugin from 'vite-plugin-eslint'

// https://vitejs.dev/config/
export default defineConfig({
  plugins: [
    vue(),
    vuetify({ autoImport: true }),
		eslintPlugin({
			include: ['src/**/*.ts', 'src/**/*.vue', 'src/*.ts', 'src/*.vue'],
		}),
  ],
  resolve: {
    alias: {
      '@': '/src'
    }
  }
})
