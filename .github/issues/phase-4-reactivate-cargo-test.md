# Phase 4: Reaktivierung cargo test

The Tauri backend still fails to compile in CI after adding GTK and libsoup packages. The build script exits with an error.

- Ensure a dedicated container with all Tauri dependencies
- Re-enable `cargo test` once the container is prepared
