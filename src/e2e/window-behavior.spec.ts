import { test, expect } from 'playwright/test'

test('should show close button and simulate action', async ({ page }) => {
  await page.goto('/')
  await page.waitForLoadState('networkidle')
  await page.locator('[data-testid="window-close"]').scrollIntoViewIfNeeded()
  await page.click('[data-testid="window-close"]')
  await expect(page.getByText(/SmolDesk/i)).toBeVisible()
})
