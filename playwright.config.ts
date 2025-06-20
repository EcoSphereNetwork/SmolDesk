import { defineConfig } from 'playwright/test'

const port = Number(process.env.PORT || 1420)

export default defineConfig({
  testDir: './src/e2e',
  timeout: 10000,
  webServer: {
    command: 'npm run dev',
    port,
    reuseExistingServer: !process.env.CI,
    timeout: 20000
  },
  use: {
    headless: true,
    viewport: { width: 1280, height: 720 },
    baseURL: `http://localhost:${port}`
  }
})
