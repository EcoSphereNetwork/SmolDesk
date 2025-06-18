import { test, expect } from 'playwright/test'

test('should show offline indicator', async ({ page, context }) => {
  await context.setOffline(true)
  await page.goto('/')
  await expect(page.getByText(/offline mode/i)).toBeVisible()
})
