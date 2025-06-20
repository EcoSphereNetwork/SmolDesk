import type { IConnectionAPI } from './interface'
import type { IWindowAPI } from './window.interface'

const mockMode = import.meta.env.VITE_USE_MOCK
const useMock = mockMode === 'true' || mockMode === 'error'

export const ConnectionAPI: Promise<IConnectionAPI> =
  mockMode === 'error'
    ? import('./__mocks__/connection.error').then(m => m.ConnectionAPI)
    : useMock
      ? import('./__mocks__/connection').then(m => m.ConnectionAPI)
      : import('./tauri').then(m => m.ConnectionAPI)

export const WindowAPI: Promise<IWindowAPI> =
  useMock
    ? import('./__mocks__/window').then(m => m.WindowAPI)
    : import('./tauri').then(m => m.WindowAPI)

if (useMock) {
  import('./__mocks__/webrtc').then(m => m.setupWebRTCMocks())
}
