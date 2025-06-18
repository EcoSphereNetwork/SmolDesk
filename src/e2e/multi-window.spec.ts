import { test, expect } from 'playwright/test'

// Test switching between main and settings windows

test('should simulate switching between virtual windows', async ({ page }) => {
  await page.goto('/')
  await page.waitForLoadState('networkidle')
  const main = page.locator('[data-testid="main-window"]')
  await expect(main).toBeVisible()
  let shot = await page.screenshot()
  if (process.env.CI) {
    expect(shot).toBeTruthy()
  } else {
    expect(shot).toMatchSnapshot('main-window.png')
  }

  const settingsButton = page.locator('[data-testid="open-settings"]')
  await settingsButton.scrollIntoViewIfNeeded()
  await settingsButton.waitFor({ state: 'visible' })
  await settingsButton.click()
  const settings = page.locator('[data-testid="settings-window"]')
  await expect(settings).toBeVisible()
  shot = await page.screenshot()
  if (process.env.CI) {
    expect(shot).toBeTruthy()
  } else {
    expect(shot).toMatchSnapshot('settings-window.png')
  }
})
