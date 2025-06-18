# Phase 4 Overview

This phase introduces a testable IPC architecture and sets up Playwright for end-to-end tests.

## Goals
- Mock Tauri IPC APIs for unit and integration tests
- Provide a structure under `src/ipc/__mocks__/` for these mocks
- Add Playwright to run the application in simulated windows

## Structure
- `src/ipc/__mocks__/` – mock implementations
- `src/e2e/` – Playwright specs
- `playwright.config.ts` – Playwright configuration

## IPC Mocking
The frontend loads either a mock implementation or the real Tauri API depending on the `VITE_USE_MOCK` environment variable. The interface lives in `src/ipc/interface.ts` and both `src/ipc/tauri.ts` and `src/ipc/__mocks__/` implement it.

## Playwright Setup
`playwright.config.ts` defines a headless browser environment. Tests live under `src/e2e/` and can be run with `npm run e2e`.

### WebRTC Simulation & Window Control

During unit and e2e tests, WebRTC APIs and Tauri window methods are mocked. This allows verifying UI reactions to connection states and window events without launching a real backend.

### Multi-Window Scenarios

E2E tests switch between a main window and a settings window using mocked window
controls. The navigation button uses `data-testid="open-settings"`.

### Snapshot Strategy

Playwright's `toHaveScreenshot()` is used for basic visual regression. Generated
images are stored under `test-results/` and ignored from Git.

### Error Paths and Offline Mode

Phase 4.4 introduces dedicated mocks for failure cases and offline
simulation. The Playwright specs `connection-error.spec.ts` and
`offline.spec.ts` verify that the UI reacts gracefully when IPC calls fail or no
network is available.

Phase 4.5 finalizes snapshot handling and prepares the e2e suite for CI usage.

**Status:** Phase 4 is complete. Further visual testing continues in Phase 5.
