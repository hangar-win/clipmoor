import { PALETTES, STOCK_INK } from '@/constants/palettes'
import type { ColorKey, ColorSet, CustomColors, Mode } from '@/types/services/settings'

export const COLOR_KEYS: readonly ColorKey[] = ['ground', 'card', 'text', 'selection', 'typeText', 'typeImage', 'typeLink', 'typeCode', 'typeLocked']
type Full = Record<ColorKey, string>

const GRAFIT = PALETTES.find((p) => p.default)!
export const DEFAULT_COLORS: Record<Mode, Full> = { dark: { ...GRAFIT.dark }, light: { ...GRAFIT.light } }


export function colorSet(mode: Mode, set: ColorSet): Full {
  const full = { ...DEFAULT_COLORS[mode] }
  for (const k of COLOR_KEYS) {
    const v = normalizeHex(set[k] ?? '')
    if (v) full[k] = v
  }
  return full
}

export function normalizeHex(input: string): string | null {
  const m = /^#?([0-9a-f]{3}|[0-9a-f]{6})$/i.exec(input.trim())
  if (!m) return null
  const h = m[1].length === 3 ? [...m[1]].map((c) => c + c).join('') : m[1]
  return `#${h.toUpperCase()}`
}

const toLinear = (c: number) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4)
const toGamma = (c: number) => (c <= 0.0031308 ? c * 12.92 : 1.055 * c ** (1 / 2.4) - 0.055)

function luminance(hex: string): number {
  const h = normalizeHex(hex)!
  const [r, g, b] = [1, 3, 5].map((i) => toLinear(parseInt(h.slice(i, i + 2), 16) / 255))
  return 0.2126 * r + 0.7152 * g + 0.0722 * b
}

function toOklab(hex: string): number[] {
  const h = normalizeHex(hex)!
  const [r, g, b] = [1, 3, 5].map((i) => toLinear(parseInt(h.slice(i, i + 2), 16) / 255))
  const l = Math.cbrt(0.4122214708 * r + 0.5363325363 * g + 0.0514459929 * b)
  const m = Math.cbrt(0.2119034982 * r + 0.6806995451 * g + 0.1073969566 * b)
  const s = Math.cbrt(0.0883024619 * r + 0.2817188376 * g + 0.6299787005 * b)
  return [0.2104542553 * l + 0.793617785 * m - 0.0040720468 * s, 1.9779984951 * l - 2.428592205 * m + 0.4505937099 * s, 0.0259040371 * l + 0.7827717662 * m - 0.808675766 * s]
}

function fromOklab([L, A, B]: number[]): string {
  const l = (L + 0.3963377774 * A + 0.2158037573 * B) ** 3
  const m = (L - 0.1055613458 * A - 0.0638541728 * B) ** 3
  const s = (L - 0.0894841775 * A - 1.291485548 * B) ** 3
  const rgb = [4.0767416621 * l - 3.3077115913 * m + 0.2309699292 * s, -1.2684380046 * l + 2.6097574011 * m - 0.3413193965 * s, -0.0041960863 * l - 0.7034186147 * m + 1.707614701 * s]
  return `#${rgb.map((c) => Math.round(Math.min(1, Math.max(0, toGamma(c))) * 255).toString(16).padStart(2, '0')).join('').toUpperCase()}`
}

export function mixOklab(a: string, b: string, percentA: number): string {
  const [x, y] = [toOklab(a), toOklab(b)]
  return fromOklab(x.map((v, i) => (v * percentA + y[i] * (100 - percentA)) / 100))
}

export function contrast(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x)
  return (hi + 0.05) / (lo + 0.05)
}

export type ColorWarning = { key: ColorKey; on: 'ground' | 'card'; ratio?: number; min?: number; lightGround?: true }

const PAIRS: [ColorKey, 'ground' | 'card', number][] = [
  ['text', 'ground', 4.5], ['text', 'card', 4.5], ['selection', 'ground', 3],
  ['typeText', 'ground', 3], ['typeImage', 'ground', 3], ['typeLink', 'ground', 3], ['typeCode', 'ground', 3],
]

export function colorWarnings(mode: Mode, set: ColorSet): ColorWarning[] {
  const f = colorSet(mode, set)
  const touched = (k: ColorKey) => normalizeHex(set[k] ?? '') !== null
  const out: ColorWarning[] = []
  if (mode === 'dark' && touched('ground') && contrast(f.ground, '#000000') > contrast(f.ground, '#FFFFFF')) out.push({ key: 'ground', on: 'ground', lightGround: true })
  for (const [key, on, min] of PAIRS) {
    if (!(touched(key) || touched(on))) continue
    const ratio = contrast(f[key], f[on])
    if (ratio < min) out.push({ key, on, ratio, min })
  }
  return out
}

const DERIVED_KEYS = ['textMuted', 'textControl', 'textStrip', 'textStripMuted', 'textOnActive', 'frame', 'hairline', 'strip', 'active', 'activeEdge', 'selectionEdge'] as const
type Derived = Record<(typeof DERIVED_KEYS)[number], string>

const stockInk = (mode: Mode): Derived => Object.fromEntries(DERIVED_KEYS.map((k) => [k, STOCK_INK[mode][k]])) as Derived

function derive(mode: Mode, f: Full): Derived {
  const d = DEFAULT_COLORS[mode]
  if (f.ground === d.ground && f.card === d.card && f.text === d.text) return stockInk(mode)
  const on = (base: string) => (p: number) => mixOklab(f.text, base, p)
  const g = on(f.ground)
  const c = on(f.card)
  const line = g(45)
  const dark = mode === 'dark'
  return {
    textMuted: g(70), textControl: g(85), textStrip: c(90), textStripMuted: c(70), frame: line, hairline: line, strip: g(8),
    active: dark ? g(16) : f.text, textOnActive: dark ? f.text : f.ground, activeEdge: f.text, selectionEdge: dark ? 'transparent' : f.text,
  }
}

export const palette = (mode: Mode, set: ColorSet) => {
  const f = colorSet(mode, set)
  return { ...f, ...derive(mode, f) }
}

export type PresetId = 'grafit' | 'gri' | 'haki-gri' | 'gece' | 'kum'
export type Preset = { id: PresetId; dark: ColorSet; light: ColorSet }

const PRESET_IDS: PresetId[] = ['grafit', 'gri', 'haki-gri', 'gece', 'kum']

const FALLBACK_PRESETS: Record<string, Pick<Preset, 'dark' | 'light'>> = {
  gri: {
    dark: { ground: '#202020', card: '#2B2B2B', text: '#F3F3F3', selection: '#FFD23F', typeText: '#7C9CE8', typeImage: '#3DB38F', typeLink: '#F07A45', typeCode: '#B08CF0' },
    light: { ground: '#F3F3F3', card: '#FFFFFF', text: '#1C1C1C', selection: '#8A6100', typeText: '#2F55B4', typeImage: '#13795D', typeLink: '#C2410C', typeCode: '#7A45C9' },
  },
  'haki-gri': {
    dark: { ground: '#1E201A', card: '#262920', text: '#ECEDE4', selection: '#E3C75A', typeText: '#86A3D9', typeImage: '#6DB58A', typeLink: '#E5875A', typeCode: '#B39AD8' },
    light: { ground: '#F0EFE6', card: '#FBFAF4', text: '#1A1C15', selection: '#6E6A1E', typeText: '#3D5A9E', typeImage: '#3C7A4E', typeLink: '#B4532A', typeCode: '#6E4E9E' },
  },
  gece: {
    dark: { ground: '#0E0E10', card: '#151518', text: '#F2F2F5', selection: '#FFD23F', typeText: '#6F8FE6', typeImage: '#2FB088', typeLink: '#F0763F', typeCode: '#A986F0' },
    light: { ground: '#E8E8EC', card: '#F7F7F9', text: '#0E0E10', selection: '#7A5A00', typeText: '#2F55B4', typeImage: '#13795D', typeLink: '#C2410C', typeCode: '#7A45C9' },
  },
  kum: {
    dark: { ground: '#221D18', card: '#2B2520', text: '#F1E9DF', selection: '#F2B544', typeText: '#8AA2D8', typeImage: '#5CB394', typeLink: '#F08A5D', typeCode: '#BC97E0' },
    light: { ground: '#F4EEE4', card: '#FCF9F3', text: '#22190F', selection: '#9A5B13', typeText: '#345AA8', typeImage: '#1F7A5E', typeLink: '#B8481A', typeCode: '#7444B8' },
  },
}

export const PRESETS: Preset[] = PRESET_IDS.map((id) => {
  const p = PALETTES.find((x) => x.id === id) ?? FALLBACK_PRESETS[id]
  return { id, dark: { ...p.dark }, light: { ...p.light } }
})

export const presetOf = (id: string | null | undefined) => PRESETS.find((p) => p.id === id) ?? PRESETS[0]
export const sameSet = (mode: Mode, a: ColorSet, b: ColorSet) => COLOR_KEYS.every((k) => colorSet(mode, a)[k] === colorSet(mode, b)[k])

export function applyPreset(c: CustomColors, id: PresetId): CustomColors {
  const p = presetOf(id)
  return { ...c, enabled: true, preset: p.id, dark: { ...p.dark }, light: { ...p.light } }
}

export function presetState(c: CustomColors): { id: PresetId; modified: boolean } {
  if (!c.enabled) return { id: 'grafit', modified: false }
  const p = presetOf(c.preset)
  return { id: p.id, modified: !sameSet('dark', c.dark, p.dark) || !sameSet('light', c.light, p.light) }
}

const ld = (light: string, dark: string) => (light === dark ? light : `light-dark(${light}, ${dark})`)

export type ResolvedColors = { ink: Record<'ground' | 'card' | 'text' | 'selection' | keyof Derived, string>; hue: Record<'text' | 'image' | 'link' | 'code' | 'locked', string> }

export function resolveColors(c: CustomColors): ResolvedColors | null {
  if (!c.enabled) return null
  const L = colorSet('light', c.light)
  const D = colorSet('dark', c.dark)
  const dl = derive('light', L)
  const dd = derive('dark', D)
  return {
    ink: {
      ground: ld(L.ground, D.ground), card: ld(L.card, D.card), text: ld(L.text, D.text), selection: ld(L.selection, D.selection),
      ...(Object.fromEntries(DERIVED_KEYS.map((k) => [k, ld(dl[k], dd[k])])) as Derived),
    },
    hue: { text: ld(L.typeText, D.typeText), image: ld(L.typeImage, D.typeImage), link: ld(L.typeLink, D.typeLink), code: ld(L.typeCode, D.typeCode), locked: ld(L.typeLocked, D.typeLocked) },
  }
}
