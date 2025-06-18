import { test, expect } from '@playwright/test'

test('should load landing screen', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByText(/SmolDesk/i)).toBeVisible()
})