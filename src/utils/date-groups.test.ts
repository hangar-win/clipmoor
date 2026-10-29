import { describe, expect, test } from 'vitest'
import type { ItemView } from '@/types/services/items'
import { groupItems, pinAllIds, pinsFirst, sectionOf } from './date-groups'

const at = (...d: [number, number, number, number?, number?]) => new Date(d[0], d[1] - 1, d[2], d[3] ?? 0, d[4] ?? 0).getTime() / 1000
const mk = (id: number, lastUsed: number, pinned = false): ItemView => ({ id, kind: 'text', source: null, size: 1, preview: 'x', lastUsed, pinned, locked: false, lockedBy: null, group: null, hasImage: false })

const THURSDAY = new Date(2026, 9, 29, 15, 30)
const MONDAY = new Date(2026, 9, 26, 9, 0)

describe('date groups', () => {
  test('local midnight splits today from yesterday', () => {
    expect(sectionOf(mk(1, at(2026, 10, 29, 0, 0)), THURSDAY)).toBe('today')
    expect(sectionOf(mk(1, at(2026, 10, 28, 23, 59)), THURSDAY)).toBe('yesterday')
    expect(sectionOf(mk(1, at(2026, 10, 28, 0, 0)), THURSDAY)).toBe('yesterday')
  })
  test('this week starts on monday, anything before it is older', () => {
    expect(sectionOf(mk(1, at(2026, 10, 27, 12)), THURSDAY)).toBe('thisWeek')
    expect(sectionOf(mk(1, at(2026, 10, 26, 0, 0)), THURSDAY)).toBe('thisWeek')
    expect(sectionOf(mk(1, at(2026, 10, 25, 23, 59)), THURSDAY)).toBe('older')
    expect(sectionOf(mk(1, at(2026, 9, 1)), THURSDAY)).toBe('older')
  })
  test('on a monday last week is older and yesterday is still yesterday', () => {
    expect(sectionOf(mk(1, at(2026, 10, 25, 20)), MONDAY)).toBe('yesterday')
    expect(sectionOf(mk(1, at(2026, 10, 24, 20)), MONDAY)).toBe('older')
  })
  test('pins form their own group whatever their age', () => {
    expect(sectionOf(mk(1, at(2025, 1, 1), true), THURSDAY)).toBe('pinned')
  })
  test('pins move to the top in a stable order, an unpinned list is returned as is', () => {
    const items = [mk(1, 9), mk(2, 8, true), mk(3, 7), mk(4, 6, true)]
    expect(pinsFirst(items).map((i) => i.id)).toEqual([2, 4, 1, 3])
    const plain = [mk(1, 9), mk(2, 8)]
    expect(pinsFirst(plain)).toBe(plain)
  })
  test('groups follow pinned, today, yesterday, this week, older and skip empty ones', () => {
    const items = pinsFirst([mk(1, at(2026, 10, 29, 15)), mk(2, at(2026, 10, 29, 9), true), mk(3, at(2026, 10, 28, 8)), mk(4, at(2026, 10, 10)), mk(5, at(2026, 10, 9))])
    const groups = groupItems(items, THURSDAY)
    expect(groups.map((g) => [g.key, g.start, g.items.map((i) => i.id)])).toEqual([
      ['pinned', 0, [2]],
      ['today', 1, [1]],
      ['yesterday', 2, [3]],
      ['older', 3, [4, 5]],
    ])
  })
  test('pin all takes every listed item of a date group, locked ones included, and nothing from Pinned', () => {
    const items = pinsFirst([mk(1, at(2026, 10, 29, 15)), mk(2, at(2026, 10, 29, 9), true), { ...mk(3, at(2026, 10, 29, 8)), locked: true }, mk(4, at(2026, 10, 10))])
    expect(groupItems(items, THURSDAY).map((g) => [g.key, pinAllIds(g)])).toEqual([
      ['pinned', []],
      ['today', [1, 3]],
      ['older', [4]],
    ])
  })
  test('an empty list has no groups', () => {
    expect(groupItems([], THURSDAY)).toEqual([])
  })
})
