import type { PasteMode } from './items'

export type PanelKey =
  | { type: 'up' }
  | { type: 'down' }
  | { type: 'left' }
  | { type: 'right' }
  | { type: 'home' }
  | { type: 'end' }
  | { type: 'activate'; mode: PasteMode }
  | { type: 'esc' }
  | { type: 'quick'; n: number }
  | { type: 'pin' }
  | { type: 'text'; s: string }
  | { type: 'backspace' }
  | { type: 'deleteWord' }
  | { type: 'selectAll' }
  | { type: 'tab'; back: boolean }
  | { type: 'togglePreview' }
  | { type: 'sourceMenu' }

export type DragPhase = 'grab' | 'move' | 'release'

export type Align = 'start' | 'center'
export type PanelGeometry = { width: number; height: number; align: Align }
