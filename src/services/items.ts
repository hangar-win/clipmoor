import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { ItemView, LockTarget, PasteMode, Query } from '@/types/services/items'

export const listItemsService = (query: Query) => invoke<ItemView[]>('list_items', { query })
export const activateItemService = (id: number, mode: PasteMode | null) => invoke<void>('activate', { id, mode })
export const getItemImageService = (id: number, thumb: boolean): Promise<Blob> => invoke<ArrayBuffer>('item_image', { id, thumb }).then((b) => new Blob([b], { type: 'image/png' }))
export const getItemTextService = (id: number) => invoke<string>('item_text', { id })
export const togglePinService = (id: number) => invoke<boolean>('toggle_pin', { id })
export const pinItemsService = (ids: number[]) => invoke<void>('pin_items', { ids })
export const moveToGroupService = (id: number, group: number | null) => invoke<void>('move_to_group', { id, group })
export const setLockedService = (target: LockTarget, locked: boolean) => invoke<void>('set_locked', { target, locked })
export const getRecentSourcesService = () => invoke<string[]>('recent_sources')
export const deleteItemService = (id: number) => invoke<void>('delete_item', { id })
export const deleteHistoryService = () => invoke<void>('delete_history')
export const deleteAllService = () => invoke<void>('delete_all')
export const onItemsChanged = (cb: () => void): Promise<UnlistenFn> => listen('items-changed', () => cb())
