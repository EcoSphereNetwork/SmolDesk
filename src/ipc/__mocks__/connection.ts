import type { IConnectionAPI } from '../interface'

export const ConnectionAPI: IConnectionAPI = {
  getStatus: async () => 'mocked: online',
  restart: async () => console.log('[mock] restart')
}
