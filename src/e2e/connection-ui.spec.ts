import { test, expect } from 'playwright/test'

test('should show mocked connection status', async ({ page }) => {
  await page.goto('/')
  await page.waitForLoadState('networkidle')
  await expect(page.getByTestId('ipc-status')).toHaveText(/mocked: online/i)
})
