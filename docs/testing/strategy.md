---
title: Testing Strategy
description: ''
---
# Testing Strategy

The project provides unit, integration and end to end tests. Vitest is used for the frontend and mocks Tauri as well as browser APIs. All Tauri calls in tests are replaced with stubs in `tests/__mocks__` and loaded via the global setup file `tests/setup.ts`.

Integration tests rely on mocked WebRTC and security components. In environments without network access the tests can still run because no real signaling server or backend is started.

Developers can inspect tests using `npm run test:ui` which launches the Vitest UI. When running offline all external network requests are blocked and the suite uses the provided mocks.