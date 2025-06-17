import { defineConfig } from 'vitest/config'
import tsconfigPaths from 'vite-tsconfig-paths'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  plugins: [tsconfigPaths({ignoreConfigErrors:true})],
  resolve: {
    alias: {
      '@tauri-apps/api/tauri': fileURLToPath(new URL('./tests/__mocks__/tauri.ts', import.meta.url)),
      '@tauri-apps/api/event': fileURLToPath(new URL('./tests/__mocks__/tauriEvent.ts', import.meta.url)),
    },
  },
  test: {
    environment: 'jsdom',
    globals: true,
    setupFiles: './tests/setup.ts',
    include: ['tests/unit/**/*.{test,spec}.{ts,tsx}'],
    exclude: ['tests/e2e/**', 'tests/integration/**'],
  },
})
