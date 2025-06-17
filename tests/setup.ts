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
