import type { IConnectionAPI } from './interface'

export const ConnectionAPI: Promise<IConnectionAPI> =
  import.meta.env.VITE_USE_MOCK === 'true'
    ? import('./__mocks__/connection').then(m => m.ConnectionAPI)
    : import('./tauri').then(m => m.ConnectionAPI)
