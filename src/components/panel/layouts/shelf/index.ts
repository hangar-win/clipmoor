import { size } from '@/lib/constants.stylex'
import type { PanelLayout } from '@/types/panel-layout'
import { ShelfView } from './shelf-view'

export const shelfLayout: PanelLayout = {
  id: 'shelf',
  label: 'layoutShelf',
  window: { width: parseFloat(size.shelfW), height: parseFloat(size.shelfH), align: 'center', fit: { minWidth: parseFloat(size.panelW) } },
  nav: { left: 'prev', right: 'next', up: 'prevKind', down: 'nextKind', home: 'first', end: 'last' },
  View: ShelfView,
}
