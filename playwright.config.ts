import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: './src/e2e',
  timeout: 10000,
  use: {
    headless: true,
    viewport: { width: 1280, height: 720 },
    baseURL: 'http://localhost:5173'
  }
})
