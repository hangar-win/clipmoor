import { renderToString } from 'react-dom/server'
import { describe, expect, test, vi } from 'vitest'
import { LAYOUTS } from '@/components/panel/layouts'
import { samplePanel } from '@/components/settings/sample-panel'
import { t } from '@/lib/locale'
import { groupItems } from '@/utils/date-groups'
import { DEFAULT_SHORTCUTS } from '@/utils/shortcuts'
import { CollapsedBar } from './collapsed-bar'
import { startsDrag } from './panel-tools'

vi.mock('@stylexjs/stylex', () => ({ create: (x: unknown) => x, props: () => ({}), defineVars: (x: unknown) => x, defineConsts: (x: unknown) => x, createTheme: () => ({}), keyframes: () => '' }))
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))

const el = (area: boolean) => ({ dataset: area ? { dragArea: '' } : {} }) as unknown as EventTarget
const tags = (html: string) => html.match(/<[a-z0-9]+\b[^>]*>/g) ?? []
const areas = (html: string) => tags(html).filter((t) => t.includes('data-drag-area'))

describe('panel drag', () => {
  test('starts only on a primary press over an empty drag area', () => {
    expect(startsDrag({ button: 0, target: el(true) })).toBe(true)
    expect(startsDrag({ button: 0, target: el(false) })).toBe(false)
    expect(startsDrag({ button: 2, target: el(true) })).toBe(false)
  })

  test.each(LAYOUTS)('$id header has a drag area and no control is one', (layout) => {
    const html = renderToString(<layout.View panel={samplePanel(layout, false, DEFAULT_SHORTCUTS)} />)
    expect(areas(html).length).toBeGreaterThan(0)
    expect(areas(html).filter((t) => /^<(button|input)\b|role="(searchbox|tab|tablist|option)"/.test(t))).toEqual([])
  })

  test.each(LAYOUTS)('$id header ends with the more button and drops keep-open and hints', (layout) => {
    const html = renderToString(<layout.View panel={samplePanel(layout, false, DEFAULT_SHORTCUTS)} />)
    const buttons = tags(html).filter((b) => b.startsWith('<button') && b.includes('aria-label='))
    const more = buttons.find((b) => b.includes(`aria-label="${t('more')}"`))
    expect(more).toBeDefined()
    expect(more).not.toContain('data-drag-area')
    expect(buttons.some((b) => b.includes(`aria-label="${t('keepOpen')}"`) || b.includes(`aria-label="${t('secShortcuts')}"`))).toBe(false)
  })

  test('list date-group headers carry a pin-all button that is no drag area, Pinned does not', () => {
    const list = LAYOUTS[0]
    const panel = samplePanel(list, false, DEFAULT_SHORTCUTS)
    const html = renderToString(<list.View panel={panel} />)
    const pinAll = tags(html).filter((b) => b.startsWith('<button') && b.includes(`aria-label="${t('pinAll')}"`))
    expect(pinAll.length).toBe(groupItems(panel.state.items, new Date(panel.now * 1000)).filter((g) => g.key !== 'pinned').length)
    expect(pinAll.length).toBeGreaterThan(0)
    expect(pinAll.filter((b) => b.includes('data-drag-area'))).toEqual([])
  })

  test('collapsed bar is a drag area', () => {
    const html = renderToString(<CollapsedBar panel={samplePanel(LAYOUTS[0], false, DEFAULT_SHORTCUTS)} />)
    expect(areas(html).length).toBe(1)
  })
})
