import { invoke } from '@tauri-apps/api/tauri'
import { appWindow } from '@tauri-apps/api/window'
import type { IConnectionAPI } from './interface'
import type { IWindowAPI } from './window.interface'

export const ConnectionAPI: IConnectionAPI = {
  getStatus: () => invoke<string>('get_status'),
  restart: () => invoke('restart_connection')
}

export const WindowAPI: IWindowAPI = {
  minimize: () => appWindow.minimize(),
  close: () => appWindow.close(),
  isFocused: () => appWindow.isFocused()
}
