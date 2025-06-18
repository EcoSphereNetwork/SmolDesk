# Playwright End-to-End Tests

Install dependencies:
```bash
npm install
```

Run tests:
```bash
npm run e2e
```

Configuration resides in `playwright.config.ts`. Tests are stored in `src/e2e/`.

## Mocking WebRTC and Window APIs

When `VITE_USE_MOCK=true` the tests run with simulated WebRTC streams and mocked window controls. The mocks live in `src/ipc/__mocks__/` and are automatically loaded by `src/ipc/index.ts`.

