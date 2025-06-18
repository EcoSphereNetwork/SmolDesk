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
