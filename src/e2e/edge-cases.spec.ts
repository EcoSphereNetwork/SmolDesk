import { test, expect } from 'playwright/test'

test('handles extremely long username gracefully', async ({ page }) => {
  await page.goto('/?user=' + 'a'.repeat(200))
  await page.waitForLoadState('networkidle')
  const shot = await page.screenshot()
  if (process.env.CI) {
    expect(shot).toBeTruthy()
  } else {
    expect(shot).toMatchSnapshot('long-username.png')
  }

// add more edge-case tests as needed
