import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { ItemView } from '@/types/services/items'

export const previewItemService = (id: number | null) => invoke<void>('preview_item', { id })
export const placePreviewService = (id: number, width: number, height: number, min: number) => invoke<void>('place_preview', { id, width, height, min })
export const onPreviewItem = (cb: (item: ItemView | null) => void): Promise<UnlistenFn> => listen<ItemView | null>('preview-item', (e) => cb(e.payload))
