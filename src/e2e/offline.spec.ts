import { test, expect } from 'playwright/test'

test('should show offline indicator', async ({ page, context }) => {
  await page.goto('/')
  await page.waitForLoadState('load')
  await context.setOffline(true)
  await expect(page.getByText(/offline mode/i)).toBeVisible()
})
