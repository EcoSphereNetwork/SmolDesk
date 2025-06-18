import { test, expect } from 'playwright/test'

test('handles extremely long username gracefully', async ({ page }) => {
  await page.goto('/?user=' + 'a'.repeat(200))
  await expect(page).toHaveScreenshot('long-username.png')
})

// add more edge-case tests as needed
