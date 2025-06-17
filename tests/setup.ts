import { vi } from 'vitest'

// localStorage mock
class LocalStorageMock {
  store: Record<string, string> = {}
  clear() { this.store = {} }
  getItem(key: string) { return this.store[key] || null }
  setItem(key: string, value: string) { this.store[key] = value }
  removeItem(key: string) { delete this.store[key] }
}
Object.defineProperty(global, 'localStorage', {
  value: new LocalStorageMock(),
  writable: true,
})

// fetch mock
global.fetch = vi.fn(async () => ({ ok: true, json: async () => ({}) })) as any

// matchMedia mock
Object.defineProperty(window, 'matchMedia', {
  writable: true,
  value: vi.fn().mockImplementation((query) => ({
    matches: false,
    media: query,
    onchange: null,
    addListener: vi.fn(),
    removeListener: vi.fn(),
    addEventListener: vi.fn(),
    removeEventListener: vi.fn(),
    dispatchEvent: vi.fn(),
  })),
})

// ------------------------------------------------------------
// Tauri API mocks
// ------------------------------------------------------------

vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: vi.fn(() => Promise.resolve()),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
  emit: vi.fn(),
}))

// ------------------------------------------------------------
// Browser and WebRTC API mocks
// ------------------------------------------------------------

Object.defineProperty(global.navigator, 'mediaDevices', {
  value: {
    getUserMedia: vi.fn().mockResolvedValue({
      getTracks: () => [{ kind: 'video', id: 'mock-track' }],
    }),
    getDisplayMedia: vi.fn().mockResolvedValue({
      getTracks: () => [{ kind: 'video', id: 'mock-track' }],
    }),
  },
  writable: true,
})

class GlobalMockRTCPeerConnection {
  localDescription: any = null
  remoteDescription: any = null
  addTrack = vi.fn()
  removeTrack = vi.fn()
  close = vi.fn()
  addIceCandidate = vi.fn()
  getStats = vi.fn().mockResolvedValue(new Map())
  createOffer = vi.fn().mockResolvedValue({ type: 'offer', sdp: '' })
  createAnswer = vi.fn().mockResolvedValue({ type: 'answer', sdp: '' })
  setLocalDescription = vi.fn()
  setRemoteDescription = vi.fn()
  createDataChannel = vi.fn().mockReturnValue({
    readyState: 'open',
    send: vi.fn(),
    close: vi.fn(),
  })
}

(global as any).MockRTCPeerConnection = GlobalMockRTCPeerConnection

Object.defineProperty(global, 'RTCPeerConnection', {
  value: GlobalMockRTCPeerConnection,
  writable: true,
})

Object.defineProperty(global, 'RTCSessionDescription', {
  value: vi.fn().mockImplementation((init) => init),
  writable: true,
})

Object.defineProperty(global, 'RTCIceCandidate', {
  value: vi.fn().mockImplementation((init) => init),
  writable: true,
})
