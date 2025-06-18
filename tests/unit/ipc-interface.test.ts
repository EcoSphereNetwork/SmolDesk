import { describe, it, expect, vi } from 'vitest'

describe('ConnectionAPI loader', () => {
  it('loads mock when env flag enabled', async () => {
    vi.stubEnv('VITE_USE_MOCK', 'true')
    const { ConnectionAPI } = await import('../../src/ipc')
    const api = await ConnectionAPI
    const status = await api.getStatus()
    expect(status).toBe('mocked: online')
    vi.unstubAllEnvs()
  })
})
