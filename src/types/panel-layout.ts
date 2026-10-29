import type { ComponentType } from 'react'
import type { I18NKeys } from '@/locale'
import type { PanelGeometry } from '@/types/services/panel'
import type { PanelController } from '@/components/panel/use-panel-controller'

export type NavKey = 'up' | 'down' | 'left' | 'right' | 'home' | 'end'
export type NavAction = 'prev' | 'next' | 'first' | 'last' | 'prevKind' | 'nextKind'
export type NavMap = Partial<Record<NavKey, NavAction>>

export type LayoutViewProps = { panel: PanelController }

export type PanelWindow = PanelGeometry & { fit?: { minWidth: number } }

export interface PanelLayout {
  id: string
  label: I18NKeys
  window: PanelWindow
  nav: NavMap
  View: ComponentType<LayoutViewProps>
}
