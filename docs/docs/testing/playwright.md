# Playwright End-to-End Tests

Install dependencies:
```bash
npm install
```

Run tests:
```bash
npm run e2e
```

The dev server is started automatically by Playwright via `webServer` in the
config. Make sure `npm run dev` works locally. When not running in CI the
existing server is reused to speed up tests.

Configuration resides in `playwright.config.ts`. Tests are stored in `src/e2e/`.


## Mocking WebRTC and Window APIs

When `VITE_USE_MOCK=true` the tests run with simulated WebRTC streams and mocked window controls. The mocks live in `src/ipc/__mocks__/` and are automatically loaded by `src/ipc/index.ts`.

## Snapshot Tests

Visual regressions are checked with `toHaveScreenshot()` which stores images in
`test-results/`.

## Error Simulation

You can switch mocks to an error mode by setting `VITE_USE_MOCK=error`. This
loads `src/ipc/__mocks__/connection.error.ts` and allows tests to verify that
the UI shows proper fallback messages when IPC calls fail.

## Offline Mode

Playwright's browser context can simulate offline conditions via
`context.setOffline(true)`. Tests under `offline.spec.ts` ensure the interface
displays an offline indicator.

## Edge Cases

Additional tests cover unusual states such as extremely long user names or
empty API responses. These help harden the UI against unexpected input.

## Snapshot Stability Tips

- Call `page.waitForLoadState('networkidle')` before taking screenshots.
- Ensure target elements are visible using `scrollIntoViewIfNeeded()`.
- In CI the tests fall back to simple `page.screenshot()` checks via
  `process.env.CI` because pixel output may vary.

To run e2e tests in CI mode locally:

```bash
npm run test:ci:e2e
```

Component-level snapshots will be added in **Phase 5** via Storybook and
Playwright component tests.