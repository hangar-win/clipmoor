import type { PanelController } from '@/components/panel/use-panel-controller'
import { t } from '@/lib/locale'
import type { PanelLayout } from '@/types/panel-layout'
import type { ItemView, Kind } from '@/types/services/items'
import type { Shortcuts } from '@/types/services/settings'
import { pinsFirst } from '@/utils/date-groups'
import { INITIAL_QUERY, emptyText, sourceOptions, type PanelState } from '@/utils/panel-state'

const DAY = 86400

const item = (id: number, lastUsed: number, kind: Kind | null, preview: string | null, source: string, extra: Partial<ItemView> = {}): ItemView => ({ id, kind, source, size: preview?.length ?? null, preview, lastUsed, pinned: false, locked: false, lockedBy: null, group: null, hasImage: false, ...extra })

export const sampleItems = (now: number): ItemView[] =>
  pinsFirst([
    item(1, now - 60, 'link', 'https://github.com/tauri-apps/tauri', 'chrome.exe'),
    item(2, now - 3 * DAY, 'text', t('colSample'), 'OUTLOOK.EXE', { pinned: true }),
    item(3, now - 300, 'code', 'SELECT o."Id", o."Total" FROM "Orders" o', 'Code.exe'),
    item(4, now - 900, 'image', null, 'SnippingTool.exe', { size: 248_311, hasImage: true }),
    item(5, now - 1800, 'color', '#3F67C8', 'Figma.exe'),
    item(6, now - DAY, null, null, 'KeePassXC.exe', { locked: true }),
    item(7, now - DAY - 600, 'text', 'hello@example.com', 'WINWORD.EXE'),
  ])

const noop = () => undefined
const pending = () => new Promise<never>(noop)

export function samplePanel(layout: PanelLayout, compact: boolean, shortcuts: Shortcuts, now = Date.now() / 1000): PanelController {
  const items = sampleItems(now)
  const state: PanelState = { query: INITIAL_QUERY, items, selected: 0, filters: true, sources: [...new Set(items.map((i) => i.source!))] }
  return {
    state,
    layout,
    compact,
    groups: [],
    unpinned: items.filter((i) => !i.pinned).length,
    latest: items[0],
    total: items.length,
    shown: 0,
    dialog: null,
    dragging: false,
    notice: null,
    keepOpen: false,
    shortcuts,
    menu: null,
    thumbs: () => undefined,
    now,
    empty: emptyText(INITIAL_QUERY),
    setMenu: noop,
    fitTo: noop,
    openSettings: noop,
    setDialog: noop,
    toggleKeepOpen: noop,
    toggleFilters: noop,
    expand: noop,
    nextKind: noop,
    pickKind: noop,
    pickGroup: noop,
    sourceOptions: sourceOptions(state),
    openSources: noop,
    highlightSource: noop,
    pickSource: noop,
    drag: noop,
    startDrag: noop,
    endDrag: noop,
    hover: noop,
    hoverPreview: noop,
    activate: noop,
    activateWith: noop,
    pin: noop,
    pinAll: noop,
    lockItem: noop,
    lockGroup: noop,
    move: noop,
    deleteItem: noop,
    createGroup: noop,
    renameGroup: noop,
    deleteGroup: noop,
    clearUnpinned: noop,
    loadText: pending,
    loadImage: pending,
  }
}
