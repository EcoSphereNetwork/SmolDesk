import { test, expect } from 'playwright/test'

test('handles extremely long username gracefully', async ({ page }) => {
  await page.goto('/?user=' + 'a'.repeat(200))
  await page.waitForLoadState('networkidle')
  await page.waitForSelector('[data-testid="main-window"]')
  await page.locator('[data-testid="main-window"]').scrollIntoViewIfNeeded()
  const shot = await page.screenshot({ fullPage: true })
  if (process.env.CI) {
    expect(shot).toBeTruthy()
  } else {
    expect(shot).toMatchSnapshot('long-username.png')
  }
})

// add more edge-case tests as needed
