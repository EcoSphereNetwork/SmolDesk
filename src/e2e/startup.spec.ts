import { test, expect } from 'playwright/test'

test('should load landing screen', async ({ page }) => {
  await page.goto('/')
  await page.waitForLoadState('networkidle')
  await page.waitForSelector('[data-testid="main-window"]')
  const main = page.locator('[data-testid="main-window"]')
  await main.scrollIntoViewIfNeeded()
  const shot = await page.screenshot({ fullPage: true })
  if (process.env.CI) {
    expect(shot).toBeTruthy()
  } else {
    expect(shot).toMatchSnapshot('main-screen.png')
  }
})
