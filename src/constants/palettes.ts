export const PALETTES = [
  {
    id: 'grafit',
    name: {
      tr: 'Grafit',
      en: 'Graphite'
    },
    default: true,
    dark: {
      ground: '#1B1B1E',
      card: '#1B1B1E',
      text: '#EDEDF0',
      selection: '#FFD23F',
      typeText: '#3F67C8',
      typeImage: '#178A6B',
      typeLink: '#D9531E',
      typeCode: '#834FD3',
      typeLocked: '#8A8A92'
    },
    light: {
      ground: '#F6F5F2',
      card: '#FFFFFF',
      text: '#1B1B1E',
      selection: '#FFD23F',
      typeText: '#2F55B4',
      typeImage: '#178A6B',
      typeLink: '#D9531E',
      typeCode: '#7A45C9',
      typeLocked: '#8A8A92'
    }
  }
] as const

export type PaletteId = (typeof PALETTES)[number]['id']

export const STOCK_INK = {
  dark: {
    ground: '#1B1B1E',
    card: '#1B1B1E',
    strip: '#2A2A30',
    active: '#34343C',
    overlay: '#1B1B1EB8',
    text: '#EDEDF0',
    textMuted: '#B5B5BC',
    textControl: '#D4D4D9',
    textStrip: '#DADADF',
    textStripMuted: '#ABABB3',
    textOnActive: '#FFFFFF',
    frame: '#6A6A74',
    hairline: '#6A6A74',
    activeEdge: '#EDEDF0',
    swatchEdge: '#FFFFFF29',
    selection: '#FFD23F',
    selectionEdge: '#00000000',
    focusRing: '#EDEDF0'
  },
  light: {
    ground: '#F6F5F2',
    card: '#FFFFFF',
    strip: '#ECE9E2',
    active: '#1B1B1E',
    overlay: '#F6F5F2B8',
    text: '#1B1B1E',
    textMuted: '#55555C',
    textControl: '#3A3A40',
    textStrip: '#2A2A30',
    textStripMuted: '#55555C',
    textOnActive: '#F6F5F2',
    frame: '#86827A',
    hairline: '#8C887F',
    activeEdge: '#1B1B1E',
    swatchEdge: '#0000001F',
    selection: '#FFD23F',
    selectionEdge: '#1B1B1E',
    focusRing: '#1B1B1E'
  }
} as const
