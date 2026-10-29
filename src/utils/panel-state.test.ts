import { describe, expect, test } from 'vitest'
import type { ItemView } from '@/types/services/items'
import { VERTICAL_NAV, collapse, emptyText, expand, reduce, shown, toggleFilters, withItems, withSources, type PanelState } from './panel-state'

const mk = (id: number): ItemView => ({ id, kind: 'text', source: 'notepad.exe', size: 1, preview: `ö${id}`, lastUsed: id, pinned: false, locked: false, lockedBy: null, group: null, hasImage: false })
const st = (n = 3, selected = 0): PanelState => ({ query: { text: '', kind: null, group: { type: 'all' }, source: null }, items: Array.from({ length: n }, (_, i) => mk(i + 1)), selected })

describe('panelState', () => {
  test('down and up clamp', () => {
    expect(reduce(st(3, 2), { type: 'down' }).state.selected).toBe(2)
    expect(reduce(st(3, 0), { type: 'up' }).state.selected).toBe(0)
    expect(reduce(st(3, 0), { type: 'down' }).state.selected).toBe(1)
  })
  test('enter activates selected with mode', () => {
    expect(reduce(st(3, 1), { type: 'activate', mode: 'plainPaste' }).effect).toEqual({ type: 'activate', id: 2, mode: 'plainPaste' })
  })
  test('quick number pastes nth visible', () => {
    expect(reduce(st(), { type: 'quick', n: 3 }).effect).toEqual({ type: 'activate', id: 3, mode: 'paste' })
  })
  test('quick number beyond list does nothing', () => {
    expect(reduce(st(), { type: 'quick', n: 7 }).effect).toBeUndefined()
  })
  test('enter on empty list does nothing', () => {
    expect(reduce(st(0), { type: 'activate', mode: 'paste' }).effect).toBeUndefined()
  })
  test('text appends, resets selection, refetches', () => {
    const r = reduce(st(3, 2), { type: 'text', s: 'ş' })
    expect(r.state.query.text).toBe('ş')
    expect(r.state.selected).toBe(0)
    expect(r.effect).toEqual({ type: 'refetch' })
  })
  test('backspace removes last char', () => {
    const s = { ...st(), query: { text: 'ab', kind: null, group: { type: 'all' as const }, source: null } }
    expect(reduce(s, { type: 'backspace' }).state.query.text).toBe('a')
  })
  test('tab cycles kind tabs both ways', () => {
    expect(reduce(st(), { type: 'tab', back: false }).state.query.kind).toBe('text')
    expect(reduce(st(), { type: 'tab', back: true }).state.query.kind).toBe('code')
  })
  test('esc hides, pin targets selected', () => {
    expect(reduce(st(), { type: 'esc' }).effect).toEqual({ type: 'hide' })
    expect(reduce(st(3, 1), { type: 'pin' }).effect).toEqual({ type: 'pin', id: 2 })
  })
  const typed = (text: string, extra: Partial<PanelState> = {}): PanelState => ({ ...st(), query: { text, kind: null, group: { type: 'all' }, source: null }, ...extra })
  test('select all marks the query, next text replaces it', () => {
    const s = reduce(typed('abc'), { type: 'selectAll' }).state
    expect(s.textSelected).toBe(true)
    expect(s.query.text).toBe('abc')
    const r = reduce(s, { type: 'text', s: 'x' })
    expect(r.state.query.text).toBe('x')
    expect(r.state.textSelected).toBe(false)
    expect(r.effect).toEqual({ type: 'refetch' })
  })
  test('select all on empty query selects nothing', () => {
    expect(reduce(typed(''), { type: 'selectAll' }).state.textSelected).toBeFalsy()
  })
  test('backspace and delete word clear a selected query', () => {
    const s = typed('foo bar', { textSelected: true })
    expect(reduce(s, { type: 'backspace' }).state).toMatchObject({ query: { text: '' }, textSelected: false })
    expect(reduce(s, { type: 'deleteWord' }).state).toMatchObject({ query: { text: '' }, textSelected: false })
  })
  test('delete word removes the last word with trailing spaces', () => {
    expect(reduce(typed('foo bar'), { type: 'deleteWord' }).state.query.text).toBe('foo ')
    expect(reduce(typed('foo bar  '), { type: 'deleteWord' }).state.query.text).toBe('foo ')
    expect(reduce(typed('şöyle'), { type: 'deleteWord' }).state.query.text).toBe('')
    expect(reduce(typed(''), { type: 'deleteWord' }).effect).toBeUndefined()
  })
  test('pasted text appends like typing', () => {
    expect(reduce(typed('a'), { type: 'text', s: 'merhaba' }).state.query.text).toBe('amerhaba')
  })
  test('refreshed list keeps the selected item selected after it moves to the top', () => {
    expect(withItems(st(3, 2), [mk(3), mk(1), mk(2)]).selected).toBe(0)
    expect(withItems(st(3, 2), [mk(1), mk(2)]).selected).toBe(1)
    expect(withItems(st(3, 1), []).selected).toBe(0)
  })
  test('showing the panel selects the newest item and clears the query', () => {
    const before = { ...st(3, 2), query: { text: 'ab', kind: 'code' as const, group: { type: 'pinned' as const }, source: null }, textSelected: true }
    const r = shown(before, false)
    expect(r.state).toMatchObject({ selected: 0, query: { text: '', kind: null, group: { type: 'all' }, source: null }, textSelected: false })
    expect(r.effect).toEqual({ type: 'refetch' })
    expect(reduce(withItems(r.state, [mk(9), mk(1)], false), { type: 'activate', mode: 'paste' }).effect).toEqual({ type: 'activate', id: 9, mode: 'paste' })
  })
  test('keep-open shows keep the query and selection', () => {
    const before = { ...st(3, 2), query: { text: 'ab', kind: null, group: { type: 'all' as const }, source: null } }
    expect(shown(before, true).state).toBe(before)
  })
  const img = (id: number): ItemView => ({ ...mk(id), kind: 'image', preview: '', hasImage: true })
  const withImage = (selected = 1): PanelState => ({ ...st(), items: [mk(1), img(2), mk(3)], selected })
  test('ctrl+space toggles the preview of a selected image only', () => {
    const open = reduce(withImage(), { type: 'togglePreview' })
    expect(open).toEqual({ state: { ...withImage(), preview: true } })
    expect(reduce(open.state, { type: 'togglePreview' }).state.preview).toBe(false)
    expect(reduce(withImage(0), { type: 'togglePreview' }).state.preview).toBeFalsy()
  })
  test('esc closes the preview first, then the panel', () => {
    const r = reduce({ ...withImage(), preview: true }, { type: 'esc' })
    expect(r).toEqual({ state: { ...withImage(), preview: false } })
    expect(reduce(r.state, { type: 'esc' }).effect).toEqual({ type: 'hide' })
  })
  test('the preview follows the selection across images and closes on any other item', () => {
    const open: PanelState = { ...st(), items: [img(1), img(2), mk(3)], selected: 0, preview: true }
    expect(reduce(open, { type: 'down' }).state).toMatchObject({ selected: 1, preview: true })
    expect(reduce({ ...open, selected: 1 }, { type: 'down' }).state).toMatchObject({ selected: 2, preview: false })
    expect(reduce({ ...open, selected: 2, preview: false }, { type: 'up' }).state.preview).toBe(false)
  })
  test('moving to a non-image, filtering or reopening closes the preview', () => {
    const open = { ...withImage(), preview: true }
    expect(reduce(open, { type: 'down' }).state.preview).toBe(false)
    expect(reduce(open, { type: 'home' }).state.preview).toBe(false)
    expect(reduce(open, { type: 'tab', back: false }).state.preview).toBe(false)
    expect(reduce(open, { type: 'text', s: 'a' }).state.preview).toBe(false)
    expect(shown(open, true).state.preview).toBe(false)
  })
  test('a refreshed list keeps the preview only while the same item stays selected', () => {
    const open = { ...withImage(), preview: true }
    expect(withItems(open, [mk(9), mk(1), img(2), mk(3)]).preview).toBe(true)
    expect(withItems(open, [mk(1), mk(3)]).preview).toBe(false)
  })
  test('a new item arriving while hidden does not drag the selection to the old one', () => {
    const hidden = withItems(st(3, 0), [mk(4), mk(1), mk(2), mk(3)], false)
    expect(hidden.selected).toBe(0)
    expect(withItems(st(3, 0), [mk(4), mk(1), mk(2), mk(3)]).selected).toBe(1)
  })
  test('the empty list explains the filter that emptied it', () => {
    const q = st(0).query
    expect(emptyText(q)).toEqual(['emptyTitle', 'emptyHint'])
    expect(emptyText({ ...q, kind: 'code' })).toEqual(['noMatchTitle', 'noKindHint'])
    expect(emptyText({ ...q, group: { type: 'pinned' } })).toEqual(['noPinsTitle', 'noPinsHint'])
    expect(emptyText({ ...q, kind: 'code', group: { type: 'group', id: 1 } })).toEqual(['noGroupTitle', 'noGroupHint'])
    expect(emptyText({ ...q, text: 'x', group: { type: 'pinned' } })).toEqual(['noMatchTitle', 'noMatchHint'])
  })
  const withSrc = (extra: Partial<PanelState> = {}): PanelState => ({ ...st(), sources: ['Code.exe', 'explorer.exe', 'floorp.exe'], ...extra })
  test('the source shortcut opens the menu on the current source and closes the preview', () => {
    expect(reduce(withSrc({ preview: true }), { type: 'sourceMenu' }).state).toMatchObject({ sourceMenu: 0, preview: false })
    const filtered = withSrc({ query: { ...st().query, source: 'EXPLORER.EXE' } })
    expect(reduce(filtered, { type: 'sourceMenu' }).state.sourceMenu).toBe(2)
  })
  test('arrows move inside the open menu and clamp to all sources plus each app', () => {
    const open = withSrc({ sourceMenu: 0 })
    expect(reduce(open, { type: 'up' }).state).toMatchObject({ sourceMenu: 0, selected: 0 })
    expect(reduce(open, { type: 'down' }).state.sourceMenu).toBe(1)
    expect(reduce(open, { type: 'end' }).state.sourceMenu).toBe(3)
    expect(reduce({ ...open, sourceMenu: 3 }, { type: 'down' }).state.sourceMenu).toBe(3)
    expect(reduce({ ...open, sourceMenu: 3 }, { type: 'home' }).state.sourceMenu).toBe(0)
  })
  test('enter picks the highlighted source, keeps the other filters and refetches', () => {
    const open = withSrc({ sourceMenu: 1, selected: 2, query: { text: 'ab', kind: 'code', group: { type: 'pinned' }, source: null } })
    const r = reduce(open, { type: 'activate', mode: 'copy' })
    expect(r.state).toMatchObject({ sourceMenu: undefined, selected: 0, query: { text: 'ab', kind: 'code', group: { type: 'pinned' }, source: 'Code.exe' } })
    expect(r.effect).toEqual({ type: 'refetch' })
    expect(reduce({ ...r.state, sourceMenu: 0 }, { type: 'activate', mode: 'paste' }).state.query.source).toBeNull()
  })
  test('esc and the source shortcut close only the menu', () => {
    const open = withSrc({ sourceMenu: 2 })
    expect(reduce(open, { type: 'esc' })).toEqual({ state: { ...open, sourceMenu: undefined } })
    expect(reduce(open, { type: 'sourceMenu' })).toEqual({ state: { ...open, sourceMenu: undefined } })
  })
  test('any other key closes the menu and acts as usual', () => {
    const r = reduce(withSrc({ sourceMenu: 2 }), { type: 'text', s: 'a' })
    expect(r.state).toMatchObject({ sourceMenu: undefined, query: { text: 'a' } })
    expect(r.effect).toEqual({ type: 'refetch' })
  })
  test('the source filter resets on show unless the panel is kept open', () => {
    const filtered = withSrc({ query: { ...st().query, source: 'floorp.exe' }, sourceMenu: 1 })
    expect(shown(filtered, false).state).toMatchObject({ query: { source: null }, sourceMenu: undefined })
    expect(shown(filtered, true).state).toMatchObject({ query: { source: 'floorp.exe' }, sourceMenu: undefined })
  })
  test('a source that leaves the history falls back to all sources', () => {
    const filtered = withSrc({ query: { ...st().query, source: 'floorp.exe' } })
    expect(withSources(filtered, ['FLOORP.EXE']).effect).toBeUndefined()
    const r = withSources(filtered, ['Code.exe'])
    expect(r.state).toMatchObject({ sources: ['Code.exe'], query: { source: null } })
    expect(r.effect).toEqual({ type: 'refetch' })
  })
  const kept = { nav: VERTICAL_NAV, keepOpen: true }
  test('esc on a kept-open panel collapses it and releases the keys instead of hiding', () => {
    const r = reduce({ ...st(3, 1), sourceMenu: undefined }, { type: 'esc' }, kept)
    expect(r.state).toMatchObject({ collapsed: true, selected: 1 })
    expect(r.effect).toEqual({ type: 'collapse' })
    expect(reduce({ ...withImage(), preview: true }, { type: 'esc' }, kept)).toEqual({ state: { ...withImage(), preview: false } })
  })
  test('the open source menu closes first, then esc collapses', () => {
    const menu = reduce(withSrc({ sourceMenu: 1 }), { type: 'esc' }, kept)
    expect(menu.state.collapsed).toBeFalsy()
    expect(reduce(menu.state, { type: 'esc' }, kept).state.collapsed).toBe(true)
  })
  test('a collapsed panel ignores keys until it expands', () => {
    const bar = collapse(st(3, 1)).state
    expect(reduce(bar, { type: 'down' }, kept)).toEqual({ state: bar })
    expect(reduce(bar, { type: 'activate', mode: 'paste' }, kept).effect).toBeUndefined()
    expect(expand(bar)).toMatchObject({ collapsed: false, selected: 1 })
    expect(expand(st())).toEqual(st())
  })
  test('the open shortcut expands a collapsed kept-open panel and keeps its query', () => {
    const bar = { ...collapse(typed('ab')).state }
    expect(shown(bar, true).state).toMatchObject({ collapsed: false, query: { text: 'ab' } })
    expect(shown(bar, false).state).toMatchObject({ collapsed: false, query: { text: '' } })
  })
  test('collapsing closes the image preview and the source menu', () => {
    expect(collapse({ ...withImage(), preview: true, sourceMenu: 1 }).state).toMatchObject({ collapsed: true, preview: false, sourceMenu: undefined })
  })
  test('switching type reveals the filter row, a fresh show hides it again', () => {
    const r = reduce(st(), { type: 'tab', back: false })
    expect(r.state.filters).toBe(true)
    expect(shown(r.state, false).state.filters).toBe(false)
    expect(shown(r.state, true).state.filters).toBe(true)
    expect(toggleFilters(toggleFilters(st())).filters).toBe(false)
  })
  test('pins come first so the selection and quick numbers follow what is shown', () => {
    const items = [mk(1), { ...mk(2), pinned: true }, mk(3)]
    const s = withItems(st(0, 0), items)
    expect(s.items.map((i) => i.id)).toEqual([2, 1, 3])
    expect(reduce(s, { type: 'quick', n: 1 }).effect).toEqual({ type: 'activate', id: 2, mode: 'paste' })
  })
  test('unmapped navigation keys do nothing', () => {
    expect(reduce(st(3, 1), { type: 'left' }, { nav: { up: 'prev', down: 'next' }, keepOpen: false })).toEqual({ state: st(3, 1) })
  })
  test('an empty filtered list points at the source filter', () => {
    expect(emptyText({ ...st().query, source: 'floorp.exe' })).toEqual(['noMatchTitle', 'noSourceHint'])
  })
})