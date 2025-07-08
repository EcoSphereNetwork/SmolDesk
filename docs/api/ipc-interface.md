---
title: IPC Interface
description: Übersicht der in SmolDesk verfügbaren Tauri-Kommandos
---

SmolDesk verwendet Tauri zur Kommunikation zwischen Frontend (React) und Backend (Rust). Die folgenden Befehle können über `invoke` aufgerufen werden.

## Commands

| Command | Parameters | Returns | Zugehörige Features |
|--------|------------|---------|--------------------|
| `get_display_server` | – | `String` | [Remote](../features/remote.md) |
| `get_monitors` | – | `Result<Vec<MonitorInfo>, String>` | [Monitors](../features/monitors.md) |
| `start_capture` | `monitorIndex: usize`, `config: ScreenCaptureConfig` | `Result<(), String>` | [Remote](../features/remote.md) |
| `stop_capture` | – | `Result<(), String>` | [Remote](../features/remote.md) |
| `send_input_event` | `event: InputEvent` | `Result<(), String>` | [Remote](../features/remote.md) |
| `set_input_enabled` | `enabled: bool` | `Result<(), String>` | [Remote](../features/remote.md) |
| `configure_input_forwarding` | `config: InputForwardingConfig` | `Result<(), String>` | [Monitors](../features/monitors.md) |
| `get_video_codecs` | – | `Vec<String>` | [Remote](../features/remote.md) |
| `get_hardware_acceleration_options` | – | `Vec<String>` | [Remote](../features/remote.md) |
| `get_clipboard_text` | – | `Result<String, String>` | [Clipboard](../features/clipboard.md) |
| `set_clipboard_text` | `text: String` | `Result<(), String>` | [Clipboard](../features/clipboard.md) |
| `initialize_security` | `secretKey: String` | `Result<(), String>` | [Security](../features/security.md) |

Weitere Kommandos wie Dateiübertragung oder OAuth befinden sich in der Entwicklung und sind aktuell als experimentell gekennzeichnet.

### Beispiel

```ts
import { invoke } from '@tauri-apps/api/tauri'

await invoke('start_capture', { monitorIndex: 0, config: { fps: 30 } })
```

## Events

Das Backend sendet Ereignisse über Tauri's Event-System. Relevante Events sind unter anderem `transfer-started`, `transfer-progress`, `transfer-completed` sowie `clipboard-changed`. Weitere Eventnamen finden sich in den jeweiligen Komponenten.
