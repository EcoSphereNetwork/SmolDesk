# Continuous Integration Overview

## Zielsetzung

Automate builds and run tests for every pull request. Lint sources and package artifacts on successful runs.

## Tools

- **vitest** – unit tests for React components
- **jest-axe** – accessibility checks
- **playwright** (optional) – end-to-end testing
- **cargo test** – Rust backend tests

## Strategien
- **Unit-Tests:** via `vitest` in a jsdom environment
- **Accessibility:** optional `jest-axe` checks for UI components
- **Integration/IPC:** to be added for frontend ↔ backend communication
- **Rust-Tests:** run `cargo test` for the Tauri backend

CI runs locally for developers and remotely on pull requests. Matrix jobs handle Node and Rust separately. Browser APIs and MediaStream mocks are considered risk areas.

## Phasen-Ziele
- **Lokal**: schnelle Testläufe und Linting vor Commits
- **Remote**: komplette CI in GitHub Actions für Pull Requests und Merges

\nSee [Phase 3 Report](../development/phase-3-report.md) for the initial CI implementation.

