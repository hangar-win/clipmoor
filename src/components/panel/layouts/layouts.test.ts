import { describe, expect, test, vi } from 'vitest'
import { en, tr } from '@/locale'
import type { NavAction, NavKey } from '@/types/panel-layout'
import type { ItemView } from '@/types/services/items'
import { reduce, type PanelState } from '@/utils/panel-state'
import { LAYOUTS, layoutById } from '.'

vi.mock('@stylexjs/stylex', () => ({ create: (x: unknown) => x, props: () => ({}), defineVars: (x: unknown) => x, defineConsts: (x: unknown) => x, createTheme: () => ({}), keyframes: () => '' }))

const NAV_KEYS: NavKey[] = ['up', 'down', 'left', 'right', 'home', 'end']
const ACTIONS: NavAction[] = ['prev', 'next', 'first', 'last', 'prevKind', 'nextKind']

const mk = (id: number): ItemView => ({ id, kind: 'text', source: null, size: 1, preview: 'x', lastUsed: 100 - id, pinned: false, locked: false, lockedBy: null, group: null, hasImage: false })
const st = (selected = 1): PanelState => ({ query: { text: '', kind: null, group: { type: 'all' }, source: null }, items: [mk(1), mk(2), mk(3)], selected })
const press = (id: string, type: NavKey, s = st()) => reduce(s, { type }, { nav: layoutById(id).nav, keepOpen: false })

describe('layout registry', () => {
  test('ships list, shelf and palette with list first as the default', () => {
    expect(LAYOUTS.map((l) => l.id)).toEqual(['list', 'shelf', 'palette'])
    expect(layoutById('nope').id).toBe('list')
    expect(layoutById('palette').id).toBe('palette')
  })
  test.each(LAYOUTS)('$id is complete', (l) => {
    expect(l.id).toMatch(/^[a-z]+$/)
    expect(tr[l.label]).toBeTruthy()
    expect(en[l.label]).toBeTruthy()
    expect(typeof l.View).toBe('function')
    expect(Number.isFinite(l.window.width) && l.window.width > 0).toBe(true)
    expect(Number.isFinite(l.window.height) && l.window.height > 0).toBe(true)
    expect(['start', 'center']).toContain(l.window.align)
    expect(l.window.fit?.minWidth ?? 0).toBeLessThanOrEqual(l.window.width)
    expect(Object.keys(l.nav).every((k) => NAV_KEYS.includes(k as NavKey))).toBe(true)
    expect(Object.values(l.nav).every((a) => ACTIONS.includes(a))).toBe(true)
    expect(Object.values(l.nav)).toEqual(expect.arrayContaining(['prev', 'next']))
  })
  test('ids are unique', () => {
    expect(new Set(LAYOUTS.map((l) => l.id)).size).toBe(LAYOUTS.length)
  })
  test('window sizes come from the design tokens', () => {
    expect(LAYOUTS.map((l) => [l.id, l.window.width, l.window.height, l.window.align])).toEqual([
      ['list', 448, 640, 'start'],
      ['shelf', 920, 260, 'center'],
      ['palette', 680, 420, 'start'],
    ])
  })
})

describe('layout navigation', () => {
  test('list and palette move with up and down, switch type with left and right', () => {
    for (const id of ['list', 'palette']) {
      expect(press(id, 'down').state.selected).toBe(2)
      expect(press(id, 'up').state.selected).toBe(0)
      expect(press(id, 'right').state).toMatchObject({ selected: 0, query: { kind: 'text' }, filters: true })
      expect(press(id, 'left').state.query.kind).toBe('code')
    }
  })
  test('shelf moves with left and right, switches type with up and down', () => {
    expect(press('shelf', 'right').state.selected).toBe(2)
    expect(press('shelf', 'left').state.selected).toBe(0)
    expect(press('shelf', 'right', st(2)).state.selected).toBe(2)
    expect(press('shelf', 'down').state.query.kind).toBe('text')
    expect(press('shelf', 'up').state.query.kind).toBe('code')
  })
  test('home and end jump to the ends in every layout', () => {
    for (const l of LAYOUTS) {
      expect(press(l.id, 'end').state.selected).toBe(2)
      expect(press(l.id, 'home').state.selected).toBe(0)
    }
  })
  test('arrows still drive the open source menu vertically', () => {
    const open = { ...st(), sources: ['a.exe', 'b.exe'], sourceMenu: 0 }
    expect(press('shelf', 'down', open).state).toMatchObject({ sourceMenu: 1, selected: 1 })
  })
})
