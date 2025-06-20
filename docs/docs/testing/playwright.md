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
