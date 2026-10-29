import { useEffect, useRef, useState } from 'react'
import { activateItemService, deleteHistoryService, deleteItemService, getItemImageService, getItemTextService, getRecentSourcesService, listItemsService, moveToGroupService, pinItemsService, onItemsChanged, setLockedService, togglePinService } from '@/services/items'
import { createGroupService, deleteGroupService, listGroupsService, renameGroupService } from '@/services/groups'
import { getAppInfoService, getSettingsService, onLockError, onSettingsChanged, openSettingsService, setKeepOpenService } from '@/services/settings'
import { capturePanelKeysService, dragPanelService, hidePanelService, onPanelHidden, onPanelKey, onPanelShown, panelPaintedService, setPanelGeometryService, setPanelInputService } from '@/services/panel'
import { size } from '@/lib/constants.stylex'
import type { GroupView } from '@/types/services/groups'
import type { GroupFilter, ItemView, Kind, PasteMode, Query } from '@/types/services/items'
import type { DragPhase } from '@/types/services/panel'
import type { Settings } from '@/types/services/settings'
import { INITIAL_QUERY, cycleKind, emptyText, expand, openSourceMenu, pickSource, reduce, requery, shown as onShown, sourceOptions, toggleFilters, withItems, withSources, type Effect, type PanelState } from '@/utils/panel-state'
import { DEFAULT_SHORTCUTS } from '@/utils/shortcuts'
import { layoutById } from './layouts'
import { useThumbs } from './use-thumbs'
import { usePreview } from './use-preview'

export type GroupDialog = { type: 'new'; item: ItemView } | { type: 'rename'; group: GroupView } | { type: 'delete'; group: GroupView } | { type: 'clear'; count: number }
export type NoticeKey = 'lockKeyMissing' | 'settingsReset'
export type PanelMenu = 'more' | 'hints' | null
export type PanelController = ReturnType<typeof usePanelController>

const initial: PanelState = { query: INITIAL_QUERY, items: [], selected: 0 }
const PREVIEW_DELAY = 300
const BAR = { width: parseFloat(size.panelW), height: parseFloat(size.collapsedBarH) }

export function usePanelController() {
  const [state, setState] = useState(initial)
  const ref = useRef(state)
  const thumbs = useThumbs(state.items)
  const [groups, setGroups] = useState<GroupView[]>([])
  const [allItems, setAllItems] = useState<ItemView[]>([])
  const [shown, setShown] = useState(0)
  const [dialog, setDialog] = useState<GroupDialog | null>(null)
  const [dragging, setDragging] = useState(false)
  const [notice, setNotice] = useState<NoticeKey | null>(null)
  const [keepOpen, setKeepOpen] = useState(false)
  const [shortcuts, setShortcuts] = useState(DEFAULT_SHORTCUTS)
  const [layoutId, setLayoutId] = useState('list')
  const [compact, setCompact] = useState(false)
  const [menu, setMenu] = useState<PanelMenu>(null)
  const menuRef = useRef<PanelMenu>(null)
  menuRef.current = menu
  const keepOpenRef = useRef(false)
  const visible = useRef(false)
  keepOpenRef.current = keepOpen
  const layout = layoutById(layoutId)
  const layoutRef = useRef(layout)
  layoutRef.current = layout
  const showPreview = usePreview(!!state.preview, state.items[state.selected]?.id)

  const commit = (next: PanelState) => {
    ref.current = next
    setState(next)
  }

  const refetch = (query: Query, follow = false) =>
    listItemsService(query).then((items) => {
      if (ref.current.query === query) commit(withItems(follow ? ref.current : { ...ref.current, items: [] }, items))
    }, console.error)

  const refetchFilters = () => {
    listItemsService(INITIAL_QUERY).then(setAllItems, console.error)
    getRecentSourcesService().then((sources) => apply(withSources(ref.current, sources)), console.error)
    return listGroupsService().then((gs) => {
      setGroups(gs)
      const g = ref.current.query.group
      if (g.type === 'group' && !gs.some((x) => x.id === g.id)) apply(requery(ref.current, { ...ref.current.query, group: { type: 'all' } }))
    }, console.error)
  }

  const closePreview = () => {
    showPreview(null)
    if (ref.current.preview) commit({ ...ref.current, preview: false })
  }

  const run = (effect?: Effect) => {
    if (effect?.type === 'activate') activateItemService(effect.id, effect.mode).catch(console.error)
    else if (effect?.type === 'hide') hidePanelService().catch(console.error)
    else if (effect?.type === 'pin') togglePinService(effect.id).catch(console.error)
    else if (effect?.type === 'refetch') refetch(ref.current.query)
    else if (effect?.type === 'collapse') {
      showPreview(null)
      setMenu(null)
      capturePanelKeysService(false).catch(console.error)
    }
  }

  const apply = ({ state: next, effect }: { state: PanelState; effect?: Effect }) => {
    commit(next)
    run(effect)
  }

  const applySettings = (s: Settings) => {
    setKeepOpen(s.keepPanelOpen)
    setShortcuts(s.shortcuts)
    setLayoutId(s.layout)
    setCompact(s.compact)
  }

  useEffect(() => {
    refetch(ref.current.query)
    refetchFilters()
    getSettingsService().then(applySettings, console.error)
    const subs = [
      onPanelKey((k) => (menuRef.current && k.type === 'esc' ? setMenu(null) : apply(reduce(ref.current, k, { nav: layoutRef.current.nav, keepOpen: keepOpenRef.current })))),
      onPanelShown(() => {
        requestAnimationFrame(() => requestAnimationFrame(() => panelPaintedService().catch(console.error)))
        visible.current = true
        setShown((n) => n + 1)
        setDialog(null)
        setMenu(null)
        setNotice(null)
        showPreview(null)
        commit(onShown(ref.current, keepOpenRef.current).state)
        refetch(ref.current.query)
        refetchFilters()
        getAppInfoService().then((i) => i.settingsReset && setNotice((n) => n ?? 'settingsReset'), console.error)
      }),
      onPanelHidden(() => {
        visible.current = false
        setMenu(null)
        if (ref.current.sourceMenu !== undefined || ref.current.collapsed) commit(expand({ ...ref.current, sourceMenu: undefined }))
      }),
      onSettingsChanged(applySettings),
      onLockError((code) => code === 'lockKeyMissing' && setNotice(code)),
      onItemsChanged(() => {
        refetch(ref.current.query, visible.current)
        refetchFilters()
      }),
    ]
    return () => subs.forEach((p) => p.then((off) => off()))
  }, [])

  const [fitted, setFitted] = useState<number | null>(null)
  const { width, height, align, fit } = layout.window
  const geometry = state.collapsed ? { ...BAR, align } : { width: (fit && fitted) || width, height, align }
  useEffect(() => {
    setPanelGeometryService(geometry).catch(console.error)
  }, [geometry.width, geometry.height, geometry.align])

  const dialogOpen = dialog !== null
  useEffect(() => {
    if (!dialogOpen) return
    setPanelInputService(true).catch(console.error)
    return () => void setPanelInputService(false).catch(console.error)
  }, [dialogOpen])

  const toggleKeepOpen = () => {
    const on = !keepOpen
    setKeepOpen(on)
    setKeepOpenService(on).catch((e) => {
      console.error(e)
      setKeepOpen(!on)
    })
  }

  const move = (id: number, group: number | null) => void moveToGroupService(id, group).catch(console.error)

  return {
    state,
    layout,
    compact,
    groups,
    unpinned: allItems.filter((i) => !i.pinned).length,
    latest: allItems[0] ?? null,
    total: allItems.length,
    shown,
    dialog,
    dragging,
    notice,
    keepOpen,
    shortcuts,
    menu,
    thumbs,
    now: Date.now() / 1000,
    empty: emptyText(state.query),
    setMenu,
    fitTo: setFitted,
    openSettings: () => {
      setMenu(null)
      openSettingsService().catch(console.error)
    },
    setDialog,
    toggleKeepOpen,
    toggleFilters: () => commit(toggleFilters(ref.current)),
    expand: () => {
      commit(expand(ref.current))
      capturePanelKeysService(true).catch(console.error)
    },
    nextKind: () => apply(cycleKind(ref.current, false)),
    pickKind: (kind: Kind | null) => apply(requery(ref.current, { ...ref.current.query, kind })),
    pickGroup: (group: GroupFilter) => apply(requery(ref.current, { ...ref.current.query, group })),
    sourceOptions: sourceOptions(state),
    openSources: (open: boolean) => commit(open ? openSourceMenu(ref.current) : { ...ref.current, sourceMenu: undefined }),
    highlightSource: (i: number) => commit({ ...ref.current, sourceMenu: i }),
    pickSource: (source: string | null) => apply(pickSource(ref.current, source)),
    drag: (phase: DragPhase) => {
      if (phase === 'grab') closePreview()
      dragPanelService(phase).catch(console.error)
    },
    startDrag: () => {
      closePreview()
      setDragging(true)
    },
    endDrag: () => setDragging(false),
    hover: (i: number) => i !== ref.current.selected && commit({ ...ref.current, selected: i, preview: false }),
    hoverPreview: (id: number | null) => ref.current.preview || showPreview(id, PREVIEW_DELAY),
    activate: (item: ItemView) => void activateItemService(item.id, null).catch(console.error),
    activateWith: (item: ItemView, mode: PasteMode) => void activateItemService(item.id, mode).catch(console.error),
    pin: (item: ItemView) => void togglePinService(item.id).catch(console.error),
    pinAll: (ids: number[]) => void pinItemsService(ids).catch(console.error),
    lockItem: (item: ItemView) => void setLockedService({ type: 'item', id: item.id }, !item.locked).catch(console.error),
    lockGroup: (group: GroupView) => void setLockedService({ type: 'group', id: group.id }, !group.locked).catch(console.error),
    move,
    deleteItem: (item: ItemView) => void deleteItemService(item.id).catch(console.error),
    createGroup: (item: ItemView, name: string) => void createGroupService(name).then((g) => moveToGroupService(item.id, g)).catch(console.error),
    renameGroup: (group: GroupView, name: string) => void renameGroupService(group.id, name).catch(console.error),
    deleteGroup: (group: GroupView) => void deleteGroupService(group.id).catch(console.error),
    clearUnpinned: () => void deleteHistoryService().catch(console.error),
    loadText: getItemTextService,
    loadImage: (id: number) => getItemImageService(id, false),
  }
}
