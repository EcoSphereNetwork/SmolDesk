# ClipboardSync

[Zur Statusübersicht](./status.md)

✅ Phase 2 abgeschlossen

Synchronizes clipboard content between host and client via Tauri IPC calls.
This component listens for clipboard changes and forwards entries over an
optional `WebRTCConnection`.

## Props

| Name               | Type                              | Description |
| ------------------ | --------------------------------- | ----------- |
| `webrtcConnection` | `WebRTCConnection?`               | connection used to broadcast entries |
| `pollInterval`     | number                            | milliseconds between checks |
| `onSync`           | `(entry: ClipboardEntry) => void` | callback when a new entry was synced |
| `onError`          | `(msg: string) => void`           | reports initialization or sync errors |

## Events

- `onSync(entry)` – fired when clipboard data was processed
- `onError(message)` – emitted on any failure

## Example

```tsx
<ClipboardSync pollInterval={1000} onSync={(e) => console.log(e)} />
```

### Teststatus

Unit tests run with Vitest under `tests/unit/ClipboardSync.test.tsx`.
