import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'

export const previewItemService = (id: number | null) => invoke<void>('preview_item', { id })
export const placePreviewService = (id: number, width: number, height: number) => invoke<void>('place_preview', { id, width, height })
export const onPreviewItem = (cb: (id: number | null) => void): Promise<UnlistenFn> => listen<number | null>('preview-item', (e) => cb(e.payload))
