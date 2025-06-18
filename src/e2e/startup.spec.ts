import { test, expect } from '@playwright/test'

test('startup screen shows', async ({ page }) => {
  await page.goto('http://localhost:1420')
  await expect(page).toHaveTitle(/SmolDesk/)
})
