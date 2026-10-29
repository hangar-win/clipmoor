import { size } from '@/lib/constants.stylex'
import type { PanelLayout } from '@/types/panel-layout'
import { VERTICAL_NAV } from '@/utils/panel-state'
import { ListView } from './list-view'

export const listLayout: PanelLayout = {
  id: 'list',
  label: 'layoutList',
  window: { width: parseFloat(size.panelW), height: parseFloat(size.panelH), align: 'start' },
  nav: VERTICAL_NAV,
  View: ListView,
}
