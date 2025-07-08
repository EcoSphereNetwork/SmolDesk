---
title: Internal API
description: 
---
# Internal API

The backend exposes several Tauri commands which the frontend uses. Commands reside in `src-tauri/src/main.rs`.

## Common Commands
- `start_capture(monitorIndex, config)` – begin screen capture.
- `stop_capture()` – stop current capture.
- `send_input_event(event)` – forward mouse or keyboard input.
- `get_clipboard_text()` / `set_clipboard_text(text)` – clipboard access.
- `initialize_security(secretKey)` – prepare security manager.

All commands return `Result` types and may produce error strings.

### Example invocation

```ts
import { invoke } from '@tauri-apps/api/tauri'

await invoke('start_capture', { monitorIndex: 0, config: { fps: 30 } })
```

### Events

The backend emits events over Tauri's event system:

- `capture-error` – emitted when screen capture fails
- `file-transfer-progress` – progress updates for file transfers
