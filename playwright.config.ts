import { defineConfig } from '@playwright/test'

export default defineConfig({
  testDir: 'src/e2e',
  use: { headless: true }
})
