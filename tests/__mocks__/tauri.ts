import { vi } from 'vitest'

export const invoke = vi.fn(() => Promise.resolve())
export const listen = vi.fn(() => Promise.resolve(() => {}))
export const emit = vi.fn()
