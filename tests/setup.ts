import { vi } from 'vitest'
import '@testing-library/jest-dom'

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

// ------------------------------------------------------------
// Additional browser API mocks for canvas and encoding
// ------------------------------------------------------------

// atob/btoa
Object.defineProperty(global, 'atob', {
  value: (str: string) => Buffer.from(str, 'base64').toString('binary'),
  writable: true,
})

Object.defineProperty(global, 'btoa', {
  value: (str: string) => Buffer.from(str, 'binary').toString('base64'),
  writable: true,
})

// Mock canvas element creation
const globalCanvas = {
  width: 1920,
  height: 1080,
  getContext: vi.fn().mockReturnValue({ drawImage: vi.fn() }),
  captureStream: vi.fn().mockReturnValue({
    getVideoTracks: vi.fn().mockReturnValue([
      { id: 'mock-video-track', stop: vi.fn() }
    ]),
    getTracks: vi.fn().mockReturnValue([
      { kind: 'video', id: 'mock-video-track', stop: vi.fn() }
    ])
  }),
}

const originalCreate = document.createElement.bind(document)
document.createElement = vi.fn((tag: string) => {
  if (tag === 'canvas') return globalCanvas as any
  if (tag === 'video') return { autoplay: true, muted: true } as any
  if (tag === 'img') return {
    onload: null,
    src: '',
    width: 1920,
    height: 1080,
    addEventListener: vi.fn(),
  } as any
  return originalCreate(tag)
}) as any

Object.defineProperty(global, 'VideoEncoder', {
  value: vi.fn(() => ({
    configure: vi.fn(),
    encode: vi.fn(),
    close: vi.fn(),
  })),
  writable: true,
  configurable: true,
})

Object.defineProperty(global, 'VideoDecoder', {
  value: vi.fn(() => ({
    configure: vi.fn(),
    decode: vi.fn(),
    close: vi.fn(),
  })),
  writable: true,
  configurable: true,
})

Object.defineProperty(global, 'EncodedVideoChunk', {
  value: vi.fn().mockImplementation((data) => data),
  writable: true,
  configurable: true,
})

Object.defineProperty(global, 'VideoFrame', {
  value: vi.fn().mockImplementation((source, init) => ({
    codedWidth: 1920,
    codedHeight: 1080,
    duration: init?.duration || 33333,
    close: vi.fn(),
  })),
  writable: true,
  configurable: true,
})
