import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { AppInfo, Settings } from '@/types/services/settings'

export const getSettingsService = () => invoke<Settings>('get_settings')
export const saveSettingsService = (s: Settings) => invoke<void>('save_settings', { s })
export const setKeepOpenService = (on: boolean) => invoke<void>('set_keep_open', { on })
export const getAppInfoService = () => invoke<AppInfo>('app_info')
export const setAutostartService = (on: boolean) => invoke<void>('set_autostart', { on })
export const openSettingsService = () => invoke<void>('open_settings')
export const openWindowsClipboardSettingsService = () => invoke<void>('open_windows_clipboard_settings')
export const onSettingsChanged = (cb: (s: Settings) => void): Promise<UnlistenFn> => listen<Settings>('settings-changed', (e) => cb(e.payload))
export const onLockError = (cb: (code: string) => void): Promise<UnlistenFn> => listen<string>('lock-error', (e) => cb(e.payload))
