import { expect, test } from 'vitest'
import type { ColorSet } from '@/types/services/settings'
import { PRESETS, applyPreset, colorSet, colorWarnings, contrast, mixOklab, normalizeHex, palette, presetState, resolveColors } from './colors'

test('contrast follows the WCAG ratio', () => {
  expect(contrast('#FFFFFF', '#000000')).toBeCloseTo(21, 2)
  expect(contrast('#000000', '#FFFFFF')).toBeCloseTo(21, 2)
  expect(contrast('#777777', '#FFFFFF')).toBeCloseTo(4.48, 2)
  expect(contrast('#FFD23F', '#1B1B1E')).toBeCloseTo(11.9, 1)
  expect(contrast('#ABC', '#AABBCC')).toBeCloseTo(1, 5)
})

test('a hex is accepted with or without # in 3 or 6 digits, anything else is refused', () => {
  expect(normalizeHex('#ffd23f')).toBe('#FFD23F')
  expect(normalizeHex(' abc ')).toBe('#AABBCC')
  expect(normalizeHex('#12345')).toBeNull()
  expect(normalizeHex('red')).toBeNull()
  expect(normalizeHex('')).toBeNull()
})

test('a set fills missing or invalid colors from the mode defaults', () => {
  const dark = colorSet('dark', { ground: '#101820', text: 'nope' })
  expect(dark).toMatchObject({ ground: '#101820', card: '#1B1B1E', text: '#EDEDF0', selection: '#FFD23F', typeLink: '#D9531E', typeLocked: '#8A8A92' })
  expect(colorSet('light', {})).toMatchObject({ ground: '#F6F5F2', card: '#FFFFFF', text: '#1B1B1E' })
})

test('untouched sets never warn, even where the stock palette sits under a floor', () => {
  expect(colorWarnings('dark', {})).toEqual([])
  expect(colorWarnings('light', {})).toEqual([])
})

test('a dim text color warns on the ground and on the card at 4.5:1', () => {
  const w = colorWarnings('dark', { text: '#4A4A4A' })
  expect(w).toMatchObject([{ key: 'text', on: 'ground', min: 4.5 }, { key: 'text', on: 'card', min: 4.5 }])
  expect(w[0].ratio).toBeCloseTo(contrast('#4A4A4A', '#1B1B1E'), 5)
})

test('changing the ground re-checks every pair that sits on it', () => {
  const keys = colorWarnings('light', { ground: '#FFD23F' }).map((w) => `${w.key}/${w.on}`)
  expect(keys).toContain('selection/ground')
  expect(keys).not.toContain('text/ground')
})

test('selection ring and type bands warn at 3:1', () => {
  const w = colorWarnings('dark', { selection: '#3A3A40', typeCode: '#2A2A30' })
  expect(w).toMatchObject([{ key: 'selection', on: 'ground', min: 3 }, { key: 'typeCode', on: 'ground', min: 3 }])
})

test('a light ground in the dark set warns, the same ground in the light set does not', () => {
  expect(colorWarnings('dark', { ground: '#E0E0E0' })[0]).toEqual({ key: 'ground', on: 'ground', lightGround: true })
  expect(colorWarnings('light', { ground: '#E0E0E0' }).some((w) => w.lightGround)).toBe(false)
  expect(colorWarnings('dark', { ground: '#2B2B33' }).some((w) => w.lightGround)).toBe(false)
})

const on = (dark: ColorSet = {}, light: ColorSet = {}) => resolveColors({ enabled: true, dark, light })

test('switched off, nothing is overridden', () => {
  expect(resolveColors({ enabled: false, dark: { ground: '#101820' }, light: {} })).toBeNull()
})

test('switched on with untouched sets, every token keeps its stock value', () => {
  const r = on()!
  expect(r.ink).toMatchObject({ ground: 'light-dark(#F6F5F2, #1B1B1E)', card: 'light-dark(#FFFFFF, #1B1B1E)', hairline: 'light-dark(#8C887F, #6A6A74)', strip: 'light-dark(#ECE9E2, #2A2A30)', selection: '#FFD23F', selectionEdge: 'light-dark(#1B1B1E, #00000000)' })
  expect(r.hue).toEqual({ text: 'light-dark(#2F55B4, #3F67C8)', image: '#178A6B', link: '#D9531E', code: 'light-dark(#7A45C9, #834FD3)', locked: '#8A8A92' })
})

test('a custom dark ground re-derives the dark lines and strips from it and leaves the light side stock', () => {
  const r = on({ ground: '#101820' })!
  expect(r.ink.ground).toBe('light-dark(#F6F5F2, #101820)')
  for (const k of ['hairline', 'frame', 'textMuted', 'strip'] as const) {
    expect(r.ink[k]).toMatch(/^light-dark\(#[0-9A-F]{6}, #[0-9A-F]{6}\)$/)
  }
  const dark = (v: string) => v.slice(-8, -1)
  expect(contrast(dark(r.ink.hairline), '#101820')).toBeGreaterThanOrEqual(3)
  expect(contrast(dark(r.ink.textMuted), '#101820')).toBeGreaterThanOrEqual(4.5)
  expect(contrast(dark(r.ink.strip), '#101820')).toBeLessThan(1.5)
  expect(r.ink.hairline.startsWith('light-dark(#8C887F,')).toBe(true)
})

test('a custom text color drives the derived muted text and active edge', () => {
  const r = on({}, { text: '#202A44' })!
  expect(r.ink.activeEdge).toBe('light-dark(#202A44, #EDEDF0)')
  const muted = r.ink.textMuted.slice(11, 18)
  expect(muted).not.toBe('#55555C')
  expect(contrast(muted, '#F6F5F2')).toBeGreaterThan(4.5)
  expect(contrast(muted, '#F6F5F2')).toBeLessThan(contrast('#202A44', '#F6F5F2'))
})

test('accent and type colors map onto the selection and hue tokens per mode', () => {
  const r = on({ selection: '#00C2FF', typeLink: '#FF8800' })!
  expect(r.ink.selection).toBe('light-dark(#FFD23F, #00C2FF)')
  expect(r.hue.link).toBe('light-dark(#D9531E, #FF8800)')
})


test('mixing in oklab matches the browser color-mix', () => {
  expect(mixOklab('#000000', '#FFFFFF', 50)).toBe('#636363')
  expect(mixOklab('#FF0000', '#00FF00', 0)).toBe('#00FF00')
  expect(mixOklab('#FF0000', '#00FF00', 100)).toBe('#FF0000')
})

const OFF = { enabled: false, dark: {}, light: {} }

test('the five presets are Grafit first, then Gri, Haki gri, Gece and Kum', () => {
  expect(PRESETS.map((p) => p.id)).toEqual(['grafit', 'gri', 'haki-gri', 'gece', 'kum'])
})

test('every preset holds the contrast floor in both modes', () => {
  for (const preset of PRESETS) {
    for (const mode of ['dark', 'light'] as const) {
      const f = palette(mode, preset[mode])
      const at = (fg: string, bg: string) => [`${preset.id}/${mode} ${fg}:${bg}`, contrast(f[fg as keyof typeof f]!, f[bg as keyof typeof f]!)] as const
      const text = [at('text', 'ground'), at('text', 'card'), at('textMuted', 'ground'), at('textControl', 'ground'), at('textStrip', 'strip'), at('textStripMuted', 'strip'), at('textOnActive', 'active')]
      const lines = [at('hairline', 'ground'), at('hairline', 'card'), at('frame', 'ground')]
      const marks = preset.id === 'grafit' ? (mode === 'dark' ? [at('selection', 'ground')] : []) : [at('selection', 'ground'), at('typeText', 'ground'), at('typeImage', 'ground'), at('typeLink', 'ground'), at('typeCode', 'ground')]
      for (const [name, r] of text) expect(r, name).toBeGreaterThanOrEqual(4.5)
      for (const [name, r] of [...lines, ...marks]) expect(r, name).toBeGreaterThanOrEqual(3)
      if (preset.id !== 'grafit') expect(colorWarnings(mode, preset[mode]), `${preset.id}/${mode}`).toEqual([])
    }
  }
})

test('the light Grafit ring rides on the graphite selected-chip edge instead of the yellow alone', () => {
  const f = palette('light', PRESETS[0].light)
  expect(contrast(f.selectionEdge, f.ground)).toBeGreaterThanOrEqual(3)
})

test('dark presets use dark surfaces only', () => {
  const dark = (hex: string) => contrast(hex, '#000000') < contrast(hex, '#FFFFFF')
  for (const preset of PRESETS) {
    const f = palette('dark', preset.dark)
    for (const k of ['ground', 'card', 'strip', 'active'] as const) expect(dark(f[k]), `${preset.id} ${k}`).toBe(true)
  }
})

test('choosing a preset fills both sets with its palette and turns custom colours on', () => {
  const c = applyPreset({ enabled: false, dark: { ground: '#123456' }, light: {} }, 'kum')
  expect(c).toMatchObject({ enabled: true, preset: 'kum' })
  expect(c.dark).toMatchObject({ ground: '#221D18', text: '#F1E9DF', selection: '#F2B544' })
  expect(c.light).toMatchObject({ ground: '#F4EEE4', text: '#22190F' })
})

test('Grafit without changes renders exactly like custom colours off', () => {
  const r = resolveColors(applyPreset(OFF, 'grafit'))!
  expect(r.ink).toMatchObject({ ground: 'light-dark(#F6F5F2, #1B1B1E)', card: 'light-dark(#FFFFFF, #1B1B1E)', text: 'light-dark(#1B1B1E, #EDEDF0)', hairline: 'light-dark(#8C887F, #6A6A74)', selection: '#FFD23F' })
  expect(r.hue.text).toBe('light-dark(#2F55B4, #3F67C8)')
  expect(presetState(OFF)).toEqual({ id: 'grafit', modified: false })
  expect(presetState(applyPreset(OFF, 'grafit'))).toEqual({ id: 'grafit', modified: false })
})

test('a tweaked colour marks the preset modified, putting it back clears the mark', () => {
  const gri = applyPreset(OFF, 'gri')
  expect(presetState(gri)).toEqual({ id: 'gri', modified: false })
  const tweaked = { ...gri, dark: { ...gri.dark, text: '#ffffff' } }
  expect(presetState(tweaked)).toEqual({ id: 'gri', modified: true })
  expect(presetState({ ...tweaked, dark: { ...gri.dark, text: '#f3f3f3' } })).toEqual({ id: 'gri', modified: false })
})

test('custom colours saved before presets existed count as a modified Grafit', () => {
  expect(presetState({ enabled: true, dark: { ground: '#101820' }, light: {} })).toEqual({ id: 'grafit', modified: true })
  expect(presetState({ enabled: true, dark: {}, light: {}, preset: 'nope' })).toEqual({ id: 'grafit', modified: false })
})
