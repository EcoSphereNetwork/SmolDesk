# Project Insights

SmolDesk is a Linux focused remote desktop tool that combines a Rust/Tauri backend with a React and TypeScript frontend. Screen capture for X11 and Wayland is streamed via WebRTC using a Node based signaling server. Security features include OAuth2 PKCE and HMAC signed messages. Hardware accelerated encoding through VAAPI or NVENC is planned. Development documents outline a modular architecture with phases for core features, security and performance.

Testing strategies mention unit and integration tests with mocks for Tauri APIs and browser features. Offline or restricted network environments rely on these mocks to run tests.
