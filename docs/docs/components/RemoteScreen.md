# RemoteScreen

[Zur Statusübersicht](./status.md)

✅ Phase 2 abgeschlossen

Displays the incoming media stream. Handles toggling of input events and exposes an `onInputToggle` callback.

## Props

| Name | Type | Description |
| --- | --- | --- |
| `stream` | MediaStream | video stream to display |
| `isConnected` | boolean | indicates active session |
| `inputEnabled` | boolean | forward user input when true |
| `onInputToggle` | function | called when user toggles input |

### Example

```tsx
<RemoteScreen stream={stream} isConnected={true} onInputToggle={setInput} />
```

### Teststatus

Die Bedienelemente werden in `tests/unit/RemoteScreen.test.tsx` getestet.
