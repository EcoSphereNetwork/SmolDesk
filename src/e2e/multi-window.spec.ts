import { test, expect } from 'playwright/test'

// Test switching between main and settings windows

test('should simulate switching between virtual windows', async ({ page }) => {
  await page.goto('/')
  await expect(page.locator('[data-testid="main-window"]')).toBeVisible()
  await expect(page).toHaveScreenshot('main-window.png')

  await page.click('[data-testid="open-settings"]')
  await expect(page.locator('[data-testid="settings-window"]')).toBeVisible()
  await expect(page).toHaveScreenshot('settings-window.png')
})
