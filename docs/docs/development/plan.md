# Development Plan

This iterative plan guides future Codex runs when enhancing SmolDesk.

## 1. Initial Analysis
- Build all components using `make build` or the scripts under `scripts/`.
- Run existing tests: `npm test` and `cargo test`.
- Review documentation in `docs/`.

## 2. Module Validation
- Inspect each package (`src/`, `src-tauri/`, `signaling-server/`).
- Ensure dependencies compile and lint cleanly.
- Document missing pieces or outdated code.

## 3. Component Completion
- Finish incomplete React components and Rust modules.
- Verify IPC commands are implemented on both sides.

## 4. Test Strategy Implementation
- Expand unit tests for critical paths.
- Add integration tests between backend and frontend.
- Provide basic end-to-end coverage using the signaling server.

## 5. Automate CI/CD
- Configure GitHub Actions to build and test on push.
- Package artifacts for Linux in deb/rpm/AppImage formats.

## 6. Refactoring and Cleanup
- Remove dead code and unused assets.
- Apply formatting and lint rules.

## 7. Feature Expansion
- Implement roadmap items such as multi-monitor support and advanced security.

Each phase should end with a successful build and passing tests before moving on.
