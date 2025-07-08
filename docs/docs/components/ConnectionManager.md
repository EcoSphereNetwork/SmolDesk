---
title: ConnectionManager
description: ""
---
# ConnectionManager

[Zur Statusübersicht](./status.md)

✅ Phase 2 abgeschlossen

Manages peer connections using WebRTC. It handles creating rooms, joining rooms and relaying streams to the RemoteScreen component.

## Props

| Name | Type | Description |
| --- | --- | --- |
| `signalingServer` | string | URL of the WebSocket signaling server |
| `onConnected` | function | called with peerId on connect |
| `onDisconnected` | function | called when connection closes |
| `onStream` | function | receives incoming MediaStream |
| `onError` | function | error handler |

### Example

```tsx
<ConnectionManager signalingServer="ws://localhost:5173" onStream={setStream} />
```

### Teststatus

Weitere Details findest du in `tests/unit/ConnectionManager.test.tsx`, der grundlegende Render- und Ereignistests enthält.
