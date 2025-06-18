import { test, expect } from 'playwright/test'

test('should load landing screen', async ({ page }) => {
  await page.goto('/')
  await page.waitForLoadState('networkidle')
  await expect(page.locator('[data-testid="main-window"]')).toBeVisible()
  const shot = await page.screenshot()
  if (process.env.CI) {
    expect(shot).toBeTruthy()
  } else {
    expect(shot).toMatchSnapshot('main-screen.png')
  }
})
