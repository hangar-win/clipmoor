import type { ItemView, Kind, PasteMode, Query } from '@/types/services/items'
import type { PanelKey } from '@/types/services/panel'
import type { NavAction, NavMap } from '@/types/panel-layout'
import { pinsFirst } from './date-groups'

export type PanelState = { query: Query; items: ItemView[]; selected: number; textSelected?: boolean; preview?: boolean; sources?: string[]; sourceMenu?: number; filters?: boolean; collapsed?: boolean }
export type Effect = { type: 'activate'; id: number; mode: PasteMode } | { type: 'hide' } | { type: 'pin'; id: number } | { type: 'refetch' } | { type: 'collapse' }
export type Step = { state: PanelState; effect?: Effect }
export type KeyContext = { nav: NavMap; keepOpen: boolean }

export const KIND_TABS: (Kind | null)[] = [null, 'text', 'image', 'link', 'color', 'code']

export const INITIAL_QUERY: Query = { text: '', kind: null, group: { type: 'all' }, source: null }

export const VERTICAL_NAV: NavMap = { up: 'prev', down: 'next', left: 'prevKind', right: 'nextKind', home: 'first', end: 'last' }

export const emptyText = (q: Query) =>
  q.text ? (['noMatchTitle', 'noMatchHint'] as const) : q.group.type === 'group' ? (['noGroupTitle', 'noGroupHint'] as const) : q.group.type === 'pinned' ? (['noPinsTitle', 'noPinsHint'] as const) : q.kind ? (['noMatchTitle', 'noKindHint'] as const) : q.source ? (['noMatchTitle', 'noSourceHint'] as const) : (['emptyTitle', 'emptyHint'] as const)

const isImage = (item?: ItemView) => item?.kind === 'image' && item.hasImage
const select = (s: PanelState, i: number): PanelState => {
  const selected = Math.max(0, Math.min(i, s.items.length - 1))
  return { ...s, selected, preview: !!s.preview && isImage(s.items[selected]) }
}
export const withItems = (s: PanelState, fetched: ItemView[], follow = true): PanelState => {
  const items = pinsFirst(fetched)
  const id = s.items[s.selected]?.id
  const kept = follow ? items.findIndex((i) => i.id === id) : -1
  const selected = kept >= 0 ? kept : Math.min(s.selected, Math.max(0, items.length - 1))
  return { ...s, items, selected, preview: !!s.preview && items[selected]?.id === id }
}
export const requery = (s: PanelState, query: Query) => ({ state: { ...s, query, selected: 0, preview: false }, effect: { type: 'refetch' } as Effect })
export const sameSource = (a: string | null, b: string | null) => a?.toLowerCase() === b?.toLowerCase()
export const sourceOptions = (s: PanelState): (string | null)[] => [null, ...(s.sources ?? [])]
export const openSourceMenu = (s: PanelState): PanelState => ({ ...s, preview: false, sourceMenu: Math.max(0, sourceOptions(s).findIndex((o) => sameSource(o, s.query.source))) })
export const pickSource = (s: PanelState, source: string | null) => requery({ ...s, sourceMenu: undefined }, { ...s.query, source })
export const withSources = (s: PanelState, sources: string[]): Step => {
  const next = { ...s, sources, sourceMenu: s.sourceMenu === undefined ? undefined : Math.min(s.sourceMenu, sources.length) }
  const gone = !!s.query.source && !sources.some((x) => sameSource(x, s.query.source))
  return gone ? pickSource(next, null) : { state: next }
}
export const shown = (s: PanelState, keepOpen: boolean): Step => {
  if (s.sourceMenu !== undefined) s = { ...s, sourceMenu: undefined }
  if (keepOpen) return { state: s.preview || s.collapsed ? { ...s, preview: false, collapsed: false } : s }
  const r = requery(s, INITIAL_QUERY)
  return { ...r, state: { ...r.state, textSelected: false, filters: false, collapsed: false } }
}
export const collapse = (s: PanelState): Step => ({ state: { ...s, collapsed: true, preview: false, sourceMenu: undefined }, effect: { type: 'collapse' } })
export const expand = (s: PanelState): PanelState => (s.collapsed ? { ...s, collapsed: false } : s)
export const toggleFilters = (s: PanelState): PanelState => ({ ...s, filters: !s.filters })

const setText = (s: PanelState, text: string) => {
  const r = requery(s, { ...s.query, text })
  return { ...r, state: { ...r.state, textSelected: false } }
}
const activateAt = (s: PanelState, i: number, mode: PasteMode) => {
  const item = s.items[i]
  return { state: s, effect: item ? ({ type: 'activate', id: item.id, mode } as Effect) : undefined }
}
export const cycleKind = (s: PanelState, back: boolean): Step => {
  const n = KIND_TABS.length
  const i = (KIND_TABS.indexOf(s.query.kind) + (back ? n - 1 : 1)) % n
  const r = requery(s, { ...s.query, kind: KIND_TABS[i] })
  return { ...r, state: { ...r.state, filters: true } }
}

function navigate(s: PanelState, action: NavAction | undefined): Step {
  switch (action) {
    case 'prev': return { state: select(s, s.selected - 1) }
    case 'next': return { state: select(s, s.selected + 1) }
    case 'first': return { state: select(s, 0) }
    case 'last': return { state: select(s, s.items.length - 1) }
    case 'prevKind': return cycleKind(s, true)
    case 'nextKind': return cycleKind(s, false)
    default: return { state: s }
  }
}

function reduceSourceMenu(s: PanelState, menu: number, k: PanelKey, ctx: KeyContext): Step {
  const options = sourceOptions(s)
  const at = (i: number) => ({ state: { ...s, sourceMenu: Math.max(0, Math.min(i, options.length - 1)) } })
  switch (k.type) {
    case 'up': return at(menu - 1)
    case 'down': return at(menu + 1)
    case 'home': return at(0)
    case 'end': return at(options.length - 1)
    case 'activate': return pickSource(s, options[menu] ?? null)
    case 'esc':
    case 'sourceMenu': return { state: { ...s, sourceMenu: undefined } }
    default: return reduce({ ...s, sourceMenu: undefined }, k, ctx)
  }
}

export function reduce(s: PanelState, k: PanelKey, ctx: KeyContext = { nav: VERTICAL_NAV, keepOpen: false }): Step {
  if (s.collapsed) return { state: s }
  if (s.sourceMenu !== undefined) return reduceSourceMenu(s, s.sourceMenu, k, ctx)
  switch (k.type) {
    case 'sourceMenu': return { state: openSourceMenu(s) }
    case 'up':
    case 'down':
    case 'left':
    case 'right':
    case 'home':
    case 'end': return navigate(s, ctx.nav[k.type])
    case 'activate': return activateAt(s, s.selected, k.mode)
    case 'quick': return activateAt(s, k.n - 1, 'paste')
    case 'esc': return s.preview ? { state: { ...s, preview: false } } : ctx.keepOpen ? collapse(s) : { state: s, effect: { type: 'hide' } }
    case 'pin': {
      const item = s.items[s.selected]
      return { state: s, effect: item ? { type: 'pin', id: item.id } : undefined }
    }
    case 'text': return setText(s, (s.textSelected ? '' : s.query.text) + k.s)
    case 'selectAll': return { state: { ...s, textSelected: !!s.query.text } }
    case 'backspace':
      if (!s.query.text) return { state: s }
      return setText(s, s.textSelected ? '' : Array.from(s.query.text).slice(0, -1).join(''))
    case 'deleteWord':
      if (!s.query.text) return { state: s }
      return setText(s, s.textSelected ? '' : s.query.text.replace(/\S*\s*$/u, ''))
    case 'togglePreview': {
      const item = s.items[s.selected]
      return { state: s.preview || isImage(item) ? { ...s, preview: !s.preview } : s }
    }
    case 'tab': return cycleKind(s, k.back)
  }
}
