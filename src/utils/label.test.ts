import { afterEach, describe, expect, test } from 'vitest'
import { setLang } from '@/lib/locale'
import type { ItemView } from '@/types/services/items'
import { chipLabel, codeLines } from './label'

const NOW = 1_800_000_000
const item = (p: Partial<ItemView>): ItemView => ({ id: 1, kind: 'text', source: 'notepad.exe', size: 84, preview: 'x', lastUsed: NOW, pinned: false, locked: false, lockedBy: null, group: null, hasImage: false, ...p })

afterEach(() => setLang('tr'))

describe('chipLabel', () => {
  test('text shows source without exe, character count and relative time', () => {
    expect(chipLabel(item({ source: 'chrome.exe', lastUsed: NOW - 125 }), NOW)).toEqual({ source: 'Chrome', size: '84 karakter', time: '2 dk' })
  })
  test('time steps from now to minutes, hours and days', () => {
    expect(chipLabel(item({ lastUsed: NOW - 20 }), NOW).time).toBe('şimdi')
    expect(chipLabel(item({ lastUsed: NOW - 3 * 3600 - 59 }), NOW).time).toBe('3 sa')
    expect(chipLabel(item({ lastUsed: NOW - 2 * 86400 }), NOW).time).toBe('2 gün')
  })
  test('image size is in bytes, color size names the notation', () => {
    expect(chipLabel(item({ kind: 'image', size: 1_572_864 }), NOW).size).toBe('1,5 MB')
    expect(chipLabel(item({ kind: 'image', size: 40_000 }), NOW).size).toBe('39 KB')
    expect(chipLabel(item({ kind: 'color', preview: ' rgb(1, 2, 3)' }), NOW).size).toBe('RGB')
    expect(chipLabel(item({ kind: 'color', preview: '#E74C3C' }), NOW).size).toBe('HEX')
  })
  test('english singular and plural', () => {
    setLang('en')
    expect(chipLabel(item({ size: 1 }), NOW).size).toBe('1 character')
    expect(chipLabel(item({ size: 1234, lastUsed: NOW - 61 }), NOW)).toMatchObject({ size: '1,234 characters', time: '1 min' })
  })
  test('locked item without meta has empty labels', () => {
    expect(chipLabel(item({ source: null, size: null, kind: null, preview: null }), NOW)).toEqual({ source: '', size: '', time: 'şimdi' })
  })
})

describe('codeLines', () => {
  test('first two non-blank lines without their shared indent', () => {
    expect(codeLines('\n    if (x) {\n\n        y();\n    }')).toEqual(['if (x) {', '    y();'])
  })
  test('tabs count as indent and a single line stays single', () => {
    expect(codeLines('\t\tSELECT 1')).toEqual(['SELECT 1'])
  })
})

describe('source names', () => {
  test('all-caps stems are title-cased and known dev apps get their product name', () => {
    expect(chipLabel(item({ source: 'OUTLOOK.EXE' }), NOW).source).toBe('Outlook')
    expect(chipLabel(item({ source: 'Code.exe' }), NOW).source).toBe('VS Code')
    expect(chipLabel(item({ source: 'msedge.exe' }), NOW).source).toBe('Edge')
  })
})
