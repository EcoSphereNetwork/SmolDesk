# RemoteScreen

Displays the incoming media stream. Handles toggling of input events and exposes an `onInputToggle` callback.

## Props
- `stream` – MediaStream to render.
- `isConnected` – indicates active session.
- `inputEnabled` – whether mouse and keyboard input are forwarded.
- `onInputToggle(enabled)` – informs parent when the user toggles input forwarding.
