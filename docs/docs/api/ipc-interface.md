---
title: IPC Interface
description: ""
---
# IPC Interface

The frontend communicates with the backend using the `IConnectionAPI` interface.

```ts
export interface IConnectionAPI {
  getStatus(): Promise<string>
  restart(): Promise<void>
}
```

Depending on the `VITE_USE_MOCK` flag either `src/ipc/tauri.ts` or `src/ipc/__mocks__/connection.ts` is loaded.

## Window Control

```ts
export interface IWindowAPI {
  minimize(): void
  close(): void
  isFocused(): Promise<boolean>
}
```

The window API is available via `WindowAPI` from `src/ipc/index.ts` and uses Tauri's window API in production.
