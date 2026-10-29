import { invoke } from '@tauri-apps/api/core'
import type { GroupView } from '@/types/services/groups'

export const listGroupsService = () => invoke<GroupView[]>('list_groups')
export const createGroupService = (name: string) => invoke<number>('create_group', { name })
export const renameGroupService = (id: number, name: string) => invoke<void>('rename_group', { id, name })
export const deleteGroupService = (id: number) => invoke<void>('delete_group', { id })
