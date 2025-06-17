# Testing Strategy

The repository contains unit tests for TypeScript and Rust as well as integration and end-to-end tests.

## Running Tests
```bash
npm test          # frontend unit tests
npm run test:e2e  # end-to-end tests
cd src-tauri && cargo test
```

## Known Issues
- Some WebRTC tests rely on mocked Tauri APIs.
- Network tests require a local signaling server.
