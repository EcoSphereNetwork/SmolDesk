---
title: ConnectionManager
description: src/components/ConnectionManager.tsx
---

## Function
src/components/ConnectionManager.tsx

## Props

| Name | Type |
| --- | --- |
| `onConnected` | `(peerId: string) => void` |
| `onDisconnected` | `() => void` |
| `onStream` | `(stream: MediaStream) => void` |
| `onError` | `(error: Error) => void` |
| `signalingServer` | `string` |
| `autoConnect` | `boolean` |

## Used Hooks

`useState`, `useEffect`, `useCallback`

## Related Features

- [Feature Documentation](../features/remote.md)

## Source

[src/components/ConnectionManager.tsx](https://github.com/EcoSphereNetwork/SmolDesk/blob/main/src/components/ConnectionManager.tsx)
