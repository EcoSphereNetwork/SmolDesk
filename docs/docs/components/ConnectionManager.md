# ConnectionManager

Manages peer connections using WebRTC. It handles creating rooms, joining rooms and relaying streams to the RemoteScreen component.

## Props
- `signalingServer` – URL of the WebSocket signaling server.
- `onConnected(peerId)` – callback when a peer is connected.
- `onDisconnected()` – called when the connection closes.
- `onStream(stream)` – provides the incoming media stream.
- `onError(error)` – error handler.
