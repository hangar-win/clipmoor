import { size } from '@/lib/constants.stylex'
import type { PanelLayout } from '@/types/panel-layout'
import { VERTICAL_NAV } from '@/utils/panel-state'
import { PaletteView } from './palette-view'

export const paletteLayout: PanelLayout = {
  id: 'palette',
  label: 'layoutPalette',
  window: { width: parseFloat(size.paletteW), height: parseFloat(size.paletteH), align: 'start' },
  nav: VERTICAL_NAV,
  View: PaletteView,
}
