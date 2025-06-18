# Testing Strategy

The repository contains unit tests for TypeScript and Rust as well as integration and end-to-end tests.

## Test Types

- **Unit** – validate isolated functions and components
- **Integration** – ensure IPC between frontend and backend works
- **E2E** – run the packaged app and interact via browser automation

Current coverage focuses on React hooks and a handful of Rust utilities. Additional tests for WebRTC flows are planned.

## Running Tests
```bash
npm test          # frontend unit tests
npm run e2e       # end-to-end tests
cd src-tauri && cargo test
```

## Known Issues
- Some WebRTC tests rely on mocked Tauri APIs.
- Network tests require a local signaling server.
- Window controls are simulated when using the mock IPC layer.
- Vitest is optional and must be installed with `scripts/install-vitest.sh` in Codex environments.

See [CI overview](./ci-overview.md) for planned automation steps.
See [coverage instructions](./coverage.md) for generating reports.
See [Playwright guide](./playwright.md) to run E2E tests.

