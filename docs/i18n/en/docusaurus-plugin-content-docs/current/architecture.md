---
title: Architecture Overview
description: ""
---
# Architecture Overview

SmolDesk consists of a Rust backend built with Tauri and a React frontend written in TypeScript. A separate Node.js signaling server coordinates WebRTC peers.

## System Components
- **Frontend (React + Tauri)** – UI, state management and interaction with backend through Tauri IPC commands.
- **Backend (Rust)** – Screen capture, input forwarding, clipboard access and security utilities.
- **Signaling Server** – WebSocket based server for exchanging WebRTC offer/answer and ICE candidates.
- **Tests** – Vitest for TypeScript and `cargo test` for Rust modules.

## Data Flow
1. Frontend invokes Tauri commands for screen capture or input forwarding.
2. Backend streams frames via WebRTC using the signaling server for connection establishment.
3. Input events from the client are sent back through the same peer connection.

IPC between the frontend and backend uses Tauri's `invoke` API. Critical commands are defined in `src-tauri/src/main.rs`.