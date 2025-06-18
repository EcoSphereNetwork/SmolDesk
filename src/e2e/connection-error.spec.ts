import { test, expect } from 'playwright/test'

test('should show error fallback when connection fails', async ({ page }) => {
  await page.goto('/')
  await page.waitForLoadState('networkidle')
  await expect(page.getByTestId('ipc-status')).toHaveText(/connection timeout/i)
})
