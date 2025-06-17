# ClipboardSync

Synchronises clipboard contents between host and client using Tauri IPC commands.

## Events
- `onSync(entry)` – fired when clipboard data is exchanged.
- `onError(message)` – emitted when synchronization fails.
