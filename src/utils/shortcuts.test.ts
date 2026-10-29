import { afterEach, describe, expect, test } from 'vitest'
import { setLang } from '@/lib/locale'
import type { Chord, Settings } from '@/types/services/settings'
import { DEFAULT_SHORTCUTS, check, chordFromKey, formatChord, formatQuick, isDefaultShortcut, navHint, quickHint, shortcutText, withShortcut } from './shortcuts'

afterEach(() => setLang('tr'))

const k = (keyCode: number, mods: Partial<{ ctrlKey: boolean; shiftKey: boolean; altKey: boolean; metaKey: boolean }> = {}) => ({ keyCode, ctrlKey: false, shiftKey: false, altKey: false, metaKey: false, ...mods })

describe('formatChord', () => {
  test('names modifiers in Windows order and keys by their caps', () => {
    expect(formatChord({ ctrl: true, shift: true, alt: false, win: false, key: 0x56 })).toBe('Ctrl+Shift+V')
    expect(formatChord(DEFAULT_SHORTCUTS.open)).toBe('Win+V')
    expect(formatChord(DEFAULT_SHORTCUTS.plainPaste)).toBe('Ctrl+Shift+Enter')
    expect(formatChord({ ctrl: false, shift: false, alt: true, win: true, key: 0x71 })).toBe('Win+Alt+F2')
    expect(formatChord(DEFAULT_SHORTCUTS.close)).toBe('Esc')
  })
  test('space is localised', () => {
    expect(formatChord(DEFAULT_SHORTCUTS.preview)).toBe('Ctrl+Boşluk')
    setLang('en')
    expect(formatChord(DEFAULT_SHORTCUTS.preview)).toBe('Ctrl+Space')
  })
  test('quick numbers show the modifier with the fixed digit range', () => {
    expect(formatQuick(DEFAULT_SHORTCUTS.quick)).toBe('Ctrl+1–9')
    expect(formatQuick({ ctrl: true, shift: true, alt: false, win: false })).toBe('Ctrl+Shift+1–9')
  })
})

describe('chordFromKey', () => {
  test('takes the Windows key code and the held modifiers', () => {
    expect(chordFromKey(k(0x56, { ctrlKey: true, shiftKey: true }))).toEqual({ ctrl: true, shift: true, alt: false, win: false, key: 0x56 })
    expect(chordFromKey(k(0x43, { metaKey: true, altKey: true }))).toEqual({ ctrl: false, shift: false, alt: true, win: true, key: 0x43 })
  })
  test('a lone modifier is not a shortcut yet', () => {
    for (const code of [16, 17, 18, 91, 92]) expect(chordFromKey(k(code, { ctrlKey: true }))).toBeNull()
  })
})

const c = (key: number, mods: Partial<Chord> = {}): Chord => ({ ctrl: false, shift: false, alt: false, win: false, key, ...mods })
const sc = DEFAULT_SHORTCUTS

describe('check', () => {
  test('windows-reserved combinations cannot be chosen', () => {
    for (const r of [c(0x4c, { win: true }), c(0x45, { win: true }), c(0x47, { win: true }), c(0x09, { alt: true }), c(0x73, { alt: true }), c(0x1b, { ctrl: true, shift: true }), c(0x2e, { ctrl: true, alt: true }), c(0x53, { win: true, shift: true })]) {
      expect(check(sc, 'open', r).error, formatChord(r)).toBe('scReserved')
    }
  })
  test('the open shortcut needs Ctrl, Alt or Win and may use Win', () => {
    expect(check(sc, 'open', c(0x56)).error).toBe('scNeedsModifier')
    expect(check(sc, 'open', c(0x56, { shift: true })).error).toBe('scNeedsModifier')
    expect(check(sc, 'open', c(0x43, { win: true, shift: true }))).toEqual({})
    expect(check(sc, 'open', c(0x56, { win: true }))).toEqual({})
    expect(check(sc, 'open', c(0x56, { ctrl: true, shift: true }))).toEqual({})
  })
  test('a common shortcut as the open shortcut warns but is allowed', () => {
    expect(check(sc, 'open', c(0x43, { ctrl: true }))).toEqual({ warn: 'scCommon' })
    expect(check(sc, 'open', c(0x56, { ctrl: true }))).toEqual({ warn: 'scCommon' })
  })
  test('panel actions cannot use Win, typing keys or fixed keys', () => {
    expect(check(sc, 'close', c(0x1b, { win: true })).error).toBe('scNoWin')
    expect(check(sc, 'pin', c(0x50)).error).toBe('scTyping')
    expect(check(sc, 'pin', c(0x50, { shift: true })).error).toBe('scTyping')
    expect(check(sc, 'close', c(0x41, { ctrl: true })).error).toBe('scFixed')
    expect(check(sc, 'paste', c(0x09)).error).toBe('scFixed')
    expect(check(sc, 'close', c(0x28, { ctrl: true })).error).toBe('scFixed')
    expect(check(sc, 'close', c(0x71))).toEqual({})
    expect(check(sc, 'pin', c(0x50, { alt: true }))).toEqual({})
  })
  test('a clash with another action names it and is not allowed', () => {
    expect(check(sc, 'pin', c(0x20, { ctrl: true }))).toEqual({ error: 'scConflict', clash: 'preview' })
    expect(check(sc, 'pin', c(0x33, { ctrl: true }))).toEqual({ error: 'scConflict', clash: 'quick' })
    expect(check(sc, 'pin', c(0x63, { ctrl: true }))).toEqual({ error: 'scConflict', clash: 'quick' })
    expect(check(sc, 'open', c(0x0d, { ctrl: true, shift: true }))).toEqual({ error: 'scConflict', clash: 'plainPaste' })
    expect(check(sc, 'pin', sc.pin)).toEqual({})
    expect(check(sc, 'pin', c(0xbc, { ctrl: true }))).toEqual({})
  })
  test('the source filter defaults to a free Ctrl+K and takes part in clash checks', () => {
    expect(formatChord(sc.source)).toBe('Ctrl+K')
    expect(check(sc, 'source', sc.source)).toEqual({})
    expect(check(sc, 'pin', c(0x4b, { ctrl: true }))).toEqual({ error: 'scConflict', clash: 'source' })
    expect(check(sc, 'source', c(0x50, { ctrl: true }))).toEqual({ error: 'scConflict', clash: 'pin' })
    expect(check(sc, 'source', c(0x4b)).error).toBe('scTyping')
  })
  test('the quick-number modifier is recorded with a digit and must not type', () => {
    expect(check(sc, 'quick', c(0x33, { alt: true }))).toEqual({})
    expect(check(sc, 'quick', c(0x58, { alt: true })).error).toBe('scQuickDigit')
    expect(check(sc, 'quick', c(0x31, { shift: true })).error).toBe('scTyping')
    expect(check({ ...sc, pin: c(0x32, { alt: true }) }, 'quick', c(0x35, { alt: true }))).toEqual({ error: 'scConflict', clash: 'pin' })
  })
})

describe('withShortcut', () => {
  const settings = { takeover: true, shortcuts: sc } as Settings
  test('leaving Win+V hands it back to Windows, returning to it catches it again', () => {
    const moved = withShortcut(settings, 'open', c(0x56, { ctrl: true, shift: true }))
    expect(moved.takeover).toBe(false)
    expect(moved.shortcuts.open).toEqual(c(0x56, { ctrl: true, shift: true }))
    const also = withShortcut({ ...moved, takeover: true }, 'open', c(0x42, { alt: true }))
    expect(also.takeover).toBe(true)
    expect(withShortcut(moved, 'open', sc.open).takeover).toBe(true)
  })
  test('quick keeps only the modifiers', () => {
    expect(withShortcut(settings, 'quick', c(0x33, { alt: true })).shortcuts.quick).toEqual({ ctrl: false, shift: false, alt: true, win: false })
  })
})

describe('shortcut rows', () => {
  test('quick shows its modifiers with the digit range, others their chord', () => {
    expect(shortcutText(DEFAULT_SHORTCUTS, 'quick')).toBe('Ctrl+1–9')
    expect(shortcutText(DEFAULT_SHORTCUTS, 'pin')).toBe('Ctrl+P')
  })
  test('a row is default until its chord or quick modifiers change', () => {
    expect(isDefaultShortcut(DEFAULT_SHORTCUTS, 'pin')).toBe(true)
    expect(isDefaultShortcut(DEFAULT_SHORTCUTS, 'quick')).toBe(true)
    expect(isDefaultShortcut({ ...DEFAULT_SHORTCUTS, pin: { ...DEFAULT_SHORTCUTS.pin, shift: true } }, 'pin')).toBe(false)
    expect(isDefaultShortcut({ ...DEFAULT_SHORTCUTS, quick: { ...DEFAULT_SHORTCUTS.quick, alt: true } }, 'quick')).toBe(false)
  })
})

describe('panel hints', () => {
  test('the quick number hint names the configured modifier', () => {
    expect(quickHint(DEFAULT_SHORTCUTS.quick, 1)).toBe('Ctrl 1')
    expect(quickHint({ ctrl: true, shift: false, alt: true, win: false }, 9)).toBe('Ctrl+Alt 9')
  })
  test('the arrow hint follows the layout navigation', () => {
    const vertical = { up: 'prev', down: 'next', left: 'prevKind', right: 'nextKind' } as const
    const horizontal = { left: 'prev', right: 'next', up: 'prevKind', down: 'nextKind' } as const
    expect(navHint(vertical, 'prev', 'next')).toBe('↑↓')
    expect(navHint(horizontal, 'prev', 'next')).toBe('←→')
    expect(navHint(horizontal, 'prevKind', 'nextKind')).toBe('↑↓')
    expect(navHint({}, 'prev', 'next')).toBe('')
  })
})
