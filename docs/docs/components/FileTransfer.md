---
title: FileTransfer
description: 
---
# FileTransfer

[Zur Statusübersicht](./status.md)

✅ Phase 2 abgeschlossen

Allows uploading and downloading of files over the data channel.

## Events
- `onTransferComplete(id)` – called when a transfer finishes.
- `onError(message)` – called on failures.

## Props

| Name | Type | Description |
| --- | --- | --- |
| `maxSize` | number | maximum file size in bytes |

### Example

```tsx
<FileTransfer maxSize={10_000_000} onTransferComplete={handleDone} />
```

### Teststatus

Die Komponente wird in `tests/unit/FileTransfer.test.tsx` gerendert.
