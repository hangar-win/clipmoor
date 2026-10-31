import * as stylex from '@stylexjs/stylex'

export const ink = stylex.defineVars({
  ground: 'light-dark(#F6F5F2, #1B1B1E)',
  card: 'light-dark(#FFFFFF, #1B1B1E)',
  strip: 'light-dark(#ECE9E2, #2A2A30)',
  active: 'light-dark(#1B1B1E, #34343C)',
  overlay: 'light-dark(#F6F5F2B8, #1B1B1EB8)',
  text: 'light-dark(#1B1B1E, #EDEDF0)',
  textMuted: 'light-dark(#55555C, #B5B5BC)',
  textControl: 'light-dark(#3A3A40, #D4D4D9)',
  textStrip: 'light-dark(#2A2A30, #DADADF)',
  textStripMuted: 'light-dark(#55555C, #ABABB3)',
  textOnActive: 'light-dark(#F6F5F2, #FFFFFF)',
  frame: 'light-dark(#86827A, #6A6A74)',
  hairline: 'light-dark(#8C887F, #6A6A74)',
  activeEdge: 'light-dark(#1B1B1E, #EDEDF0)',
  swatchEdge: 'light-dark(#0000001F, #FFFFFF29)',
  selection: '#FFD23F',
  selectionEdge: 'light-dark(#1B1B1E, #00000000)',
  focusRing: 'light-dark(#1B1B1E, #EDEDF0)',
})

export const radius = stylex.defineConsts({
  dot: '2px',
  key: '5px',
  item: '6px',
  control: '8px',
  card: '10px',
  window: '12px',
  full: '9999px',
})

export const hue = stylex.defineVars({
  text: 'light-dark(#2F55B4, #3F67C8)',
  image: '#178A6B',
  link: '#D9531E',
  code: 'light-dark(#7A45C9, #834FD3)',
  locked: '#8A8A92',
  color: 'conic-gradient(#E74C3C, #F1C40F, #2ECC71, #3498DB, #E74C3C)',
})

export const syntax = stylex.defineVars({
  keyword: 'light-dark(#8A3FB8, #C79BF2)',
  string: 'light-dark(#357027, #9CCB8A)',
  comment: 'light-dark(#68686F, #9A9AA2)',
  number: 'light-dark(#9A4A12, #E8A06E)',
  function: 'light-dark(#2D5BA6, #8DB0F0)',
  type: 'light-dark(#0E716B, #6CCBC0)',
})

export const font = stylex.defineConsts({
  sans: '"Segoe UI Variable Text", "Segoe UI Variable", "Segoe UI", system-ui, sans-serif',
  mono: '"Cascadia Code", "Cascadia Mono", Consolas, monospace',
})
