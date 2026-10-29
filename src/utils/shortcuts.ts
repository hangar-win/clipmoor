import { t } from '@/lib/locale'
import type { Chord, Mods, Settings, Shortcuts } from '@/types/services/settings'
import type { NavAction, NavKey, NavMap } from '@/types/panel-layout'

const NONE: Mods = { ctrl: false, shift: false, alt: false, win: false }
const ENTER = 0x0d

export const DEFAULT_SHORTCUTS: Shortcuts = {
  open: { ...NONE, win: true, key: 0x56 },
  paste: { ...NONE, key: ENTER },
  copy: { ...NONE, shift: true, key: ENTER },
  plainPaste: { ...NONE, ctrl: true, shift: true, key: ENTER },
  pin: { ...NONE, ctrl: true, key: 0x50 },
  preview: { ...NONE, ctrl: true, key: 0x20 },
  close: { ...NONE, key: 0x1b },
  source: { ...NONE, ctrl: true, key: 0x4b },
  quick: { ...NONE, ctrl: true },
}

const NAMES: Record<number, string> = {
  0x08: 'Backspace', 0x09: 'Tab', 0x0d: 'Enter', 0x13: 'Pause', 0x1b: 'Esc', 0x21: 'PgUp', 0x22: 'PgDn', 0x23: 'End', 0x24: 'Home',
  0x25: '←', 0x26: '↑', 0x27: '→', 0x28: '↓', 0x2c: 'PrtSc', 0x2d: 'Ins', 0x2e: 'Del', 0x5d: 'Menu',
  0x6a: 'Num *', 0x6b: 'Num +', 0x6d: 'Num -', 0x6e: 'Num .', 0x6f: 'Num /',
  0xba: ';', 0xbb: '=', 0xbc: ',', 0xbd: '-', 0xbe: '.', 0xbf: '/', 0xc0: '`', 0xdb: '[', 0xdc: '\\', 0xdd: ']', 0xde: "'", 0xe2: '<',
}

export function keyName(vk: number): string {
  if (vk === 0x20) return t('keySpace')
  if ((vk >= 0x30 && vk <= 0x39) || (vk >= 0x41 && vk <= 0x5a)) return String.fromCharCode(vk)
  if (vk >= 0x60 && vk <= 0x69) return `Num ${vk - 0x60}`
  if (vk >= 0x70 && vk <= 0x87) return `F${vk - 0x6f}`
  return NAMES[vk] ?? `0x${vk.toString(16).toUpperCase()}`
}

const modNames = (m: Mods) => [m.win && 'Win', m.ctrl && 'Ctrl', m.alt && 'Alt', m.shift && 'Shift'].filter(Boolean) as string[]

export const formatChord = (c: Chord) => [...modNames(c), keyName(c.key)].join('+')
export const formatQuick = (m: Mods) => [...modNames(m), '1–9'].join('+')
export const quickHint = (m: Mods, n: number) => `${modNames(m).join('+')} ${n}`

const ARROWS: [NavKey, string][] = [['up', '↑'], ['down', '↓'], ['left', '←'], ['right', '→']]
export const navHint = (nav: NavMap, ...actions: NavAction[]) => ARROWS.filter(([k]) => actions.includes(nav[k]!)).map(([, glyph]) => glyph).join('')

const MODIFIER_KEYS = [16, 17, 18, 91, 92]

export function chordFromKey(e: { keyCode: number; ctrlKey: boolean; shiftKey: boolean; altKey: boolean; metaKey: boolean }): Chord | null {
  if (!e.keyCode || MODIFIER_KEYS.includes(e.keyCode)) return null
  return { ctrl: e.ctrlKey, shift: e.shiftKey, alt: e.altKey, win: e.metaKey, key: e.keyCode }
}

export type Action = keyof Shortcuts
export type Verdict = { error?: 'scReserved' | 'scNeedsModifier' | 'scNoWin' | 'scQuickDigit' | 'scTyping' | 'scFixed' | 'scConflict'; clash?: Action; warn?: 'scCommon' }

const chord = (spec: string): Chord => {
  const parts = spec.split('+')
  const key = parts.pop()!
  const has = (m: string) => parts.includes(m)
  return { win: has('Win'), ctrl: has('Ctrl'), alt: has('Alt'), shift: has('Shift'), key: key.startsWith('0x') ? Number(key) : key.charCodeAt(0) }
}
const sameMods = (a: Mods, b: Mods) => a.ctrl === b.ctrl && a.shift === b.shift && a.alt === b.alt && a.win === b.win
const same = (a: Chord, b: Chord) => a.key === b.key && sameMods(a, b)
const isDigit = (vk: number) => (vk >= 0x31 && vk <= 0x39) || (vk >= 0x61 && vk <= 0x69)
const typable = (vk: number) => vk === 0x20 || (vk >= 0x30 && vk <= 0x39) || (vk >= 0x41 && vk <= 0x5a) || (vk >= 0x60 && vk <= 0x6f) || (vk >= 0xba && vk <= 0xc0) || (vk >= 0xdb && vk <= 0xdf) || vk === 0xe2

const RESERVED = ['Win+L', 'Ctrl+Alt+0x2E', 'Ctrl+Shift+0x1B', 'Ctrl+0x1B', 'Alt+0x09', 'Alt+Shift+0x09', 'Alt+0x73', 'Alt+0x1B', 'Win+Shift+S', 'Win+Ctrl+D', 'Win+Ctrl+0x73', 'Win+Ctrl+0x25', 'Win+Ctrl+0x27', 'Win+Ctrl+0x0D', 'Win+Ctrl+O', 'Win+Ctrl+Shift+B', 'Win+Shift+0x25', 'Win+Shift+0x26', 'Win+Shift+0x27', 'Win+Shift+0x28', 'Win+Alt+R', 'Win+Alt+G', 'Win+Alt+B', 'Win+Alt+0x2C'].map(chord)
const COMMON = [...'ACFNOPRSTVWXYZ'].map((k) => chord(`Ctrl+${k}`)).concat(['Ctrl+Shift+Z', 'Ctrl+Shift+T', 'Ctrl+Shift+N', 'Ctrl+0x09', 'Ctrl+Shift+0x09', 'Alt+0x20', 'Alt+0x0D'].map(chord))
const FIXED_KEYS = [0x08, 0x09, 0x23, 0x24, 0x26, 0x28]
const FIXED = ['Ctrl+A', 'Ctrl+V'].map(chord)

const reserved = (c: Chord) => (c.win && !c.ctrl && !c.alt && !c.shift && c.key !== 0x56 && !(c.key >= 0x70 && c.key <= 0x87)) || RESERVED.some((r) => same(r, c))

function clash(sc: Shortcuts, action: Action, c: Chord): Action | undefined {
  if (action === 'quick') return (Object.keys(sc) as Action[]).find((a) => a !== 'quick' && isDigit((sc[a] as Chord).key) && sameMods(sc[a], c))
  if (isDigit(c.key) && sameMods(sc.quick, c)) return 'quick'
  return (Object.keys(sc) as Action[]).find((a) => a !== action && a !== 'quick' && same(sc[a] as Chord, c))
}

export function check(sc: Shortcuts, action: Action, c: Chord): Verdict {
  if (action !== 'open' && c.win) return { error: 'scNoWin' }
  if (reserved(c)) return { error: 'scReserved' }
  if (action === 'open') {
    if (!c.ctrl && !c.alt && !c.win) return { error: 'scNeedsModifier' }
  } else {
    if (action === 'quick' && !isDigit(c.key)) return { error: 'scQuickDigit' }
    if (typable(c.key) && c.ctrl === c.alt) return { error: 'scTyping' }
    if (FIXED_KEYS.includes(c.key) || FIXED.some((f) => same(f, c))) return { error: 'scFixed' }
  }
  const other = clash(sc, action, c)
  if (other) return { error: 'scConflict', clash: other }
  return action === 'open' && COMMON.some((x) => same(x, c)) ? { warn: 'scCommon' } : {}
}

export const isWinV = (c: Chord) => same(c, DEFAULT_SHORTCUTS.open)

export const shortcutChord = (sc: Shortcuts, action: Action): Chord => (action === 'quick' ? { ...sc.quick, key: 0x31 } : sc[action])
export const shortcutText = (sc: Shortcuts, action: Action) => (action === 'quick' ? formatQuick(sc.quick) : formatChord(sc[action]))
export const isDefaultShortcut = (sc: Shortcuts, action: Action) => same(shortcutChord(sc, action), shortcutChord(DEFAULT_SHORTCUTS, action))

export function withShortcut(s: Settings, action: Action, c: Chord): Settings {
  if (action === 'quick') return { ...s, shortcuts: { ...s.shortcuts, quick: { ctrl: c.ctrl, shift: c.shift, alt: c.alt, win: c.win } } }
  const takeover = action !== 'open' ? s.takeover : isWinV(c) || (!isWinV(s.shortcuts.open) && s.takeover)
  return { ...s, takeover, shortcuts: { ...s.shortcuts, [action]: c } }
}
