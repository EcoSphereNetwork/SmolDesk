import { test, expect } from 'playwright/test'

test('should show error fallback when connection fails', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByText(/connection timeout/i)).toBeVisible()
})
