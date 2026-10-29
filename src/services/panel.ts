import { invoke } from '@tauri-apps/api/core'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import type { DragPhase, PanelGeometry, PanelKey } from '@/types/services/panel'

export const setPanelInputService = (on: boolean) => invoke<void>('panel_input', { on })
export const hidePanelService = () => invoke<void>('hide_panel')
export const panelPaintedService = () => invoke<void>('panel_painted')
export const dragPanelService = (phase: DragPhase) => invoke<void>('drag_panel', { phase })
export const setPanelGeometryService = (g: PanelGeometry) => invoke<void>('set_panel_geometry', g)
export const capturePanelKeysService = (on: boolean) => invoke<void>('capture_panel_keys', { on })
export const setRecordingService = (on: boolean) => invoke<void>('set_recording', { on })
export const onPanelKey = (cb: (k: PanelKey) => void): Promise<UnlistenFn> => listen<PanelKey>('panel-key', (e) => cb(e.payload))
export const onPanelShown = (cb: () => void): Promise<UnlistenFn> => listen('panel-shown', () => cb())
export const onPanelHidden = (cb: () => void): Promise<UnlistenFn> => listen('panel-hidden', () => cb())
