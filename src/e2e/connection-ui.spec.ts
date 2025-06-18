import { test, expect } from 'playwright/test'

test('should show mocked connection status', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByText(/mocked: online/i)).toBeVisible()
})
