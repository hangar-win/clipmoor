import type { PanelLayout } from '@/types/panel-layout'
import { listLayout } from './list'
import { paletteLayout } from './palette'
import { shelfLayout } from './shelf'

export const LAYOUTS: PanelLayout[] = [listLayout, shelfLayout, paletteLayout]

export const layoutById = (id: string): PanelLayout => LAYOUTS.find((l) => l.id === id) ?? LAYOUTS[0]
