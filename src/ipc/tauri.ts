import { invoke } from '@tauri-apps/api/tauri'
import type { IConnectionAPI } from './interface'

export const ConnectionAPI: IConnectionAPI = {
  getStatus: () => invoke<string>('get_status'),
  restart: () => invoke('restart_connection')
}
