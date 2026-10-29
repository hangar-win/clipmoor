export type Kind = 'text' | 'image' | 'link' | 'color' | 'code'
export type PasteMode = 'paste' | 'plainPaste' | 'copy'
export type LockSource = 'all' | 'group'
export type ItemView = { id: number; kind: Kind | null; source: string | null; size: number | null; preview: string | null; lastUsed: number; pinned: boolean; locked: boolean; lockedBy: LockSource | null; group: number | null; hasImage: boolean }
export type GroupFilter = { type: 'all' } | { type: 'pinned' } | { type: 'group'; id: number }
export type Query = { text: string; kind: Kind | null; group: GroupFilter; source: string | null }
export type LockTarget = { type: 'item'; id: number } | { type: 'group'; id: number }
