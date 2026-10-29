import { renderToString } from 'react-dom/server'
import { describe, expect, test, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { LAYOUTS } from '@/components/panel/layouts'
import { groupItems } from '@/utils/date-groups'
import { DEFAULT_SHORTCUTS } from '@/utils/shortcuts'
import { sampleItems, samplePanel } from './sample-panel'

vi.mock('@stylexjs/stylex', () => ({ create: (x: unknown) => x, props: () => ({}), defineVars: (x: unknown) => x, defineConsts: (x: unknown) => x, createTheme: () => ({}), keyframes: () => '' }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

const NOON = new Date(2026, 9, 29, 12).getTime() / 1000

describe('sample panel', () => {
  test('items cover pinned, today and yesterday with every kind and a locked item', () => {
    const items = sampleItems(NOON)
    expect(groupItems(items, new Date(NOON * 1000)).map((s) => s.key)).toEqual(['pinned', 'today', 'yesterday'])
    expect(new Set(items.map((i) => i.kind))).toEqual(new Set(['text', 'image', 'link', 'code', 'color', null]))
    expect(items.some((i) => i.locked)).toBe(true)
  })

  test.each(LAYOUTS.flatMap((l) => [[l.id, false], [l.id, true]] as const))('%s (compact %s) renders without calling a service', (id, compact) => {
    const layout = LAYOUTS.find((l) => l.id === id)!
    const c = samplePanel(layout, compact, DEFAULT_SHORTCUTS, NOON)
    const html = renderToString(<layout.View panel={c} />)
    expect(html).toContain('github.com/tauri-apps/tauri')
    c.activate(c.state.items[0])
    c.pin(c.state.items[0])
    c.clearUnpinned()
    expect(invoke).not.toHaveBeenCalled()
  })
})
