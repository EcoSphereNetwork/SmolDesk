# IPC Interface

The frontend communicates with the backend using the `IConnectionAPI` interface.

```ts
export interface IConnectionAPI {
  getStatus(): Promise<string>
  restart(): Promise<void>
}
```

Depending on the `VITE_USE_MOCK` flag either `src/ipc/tauri.ts` or `src/ipc/__mocks__/connection.ts` is loaded.
