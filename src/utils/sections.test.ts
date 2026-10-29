import { expect, test } from 'vitest'
import { reasonSection, railStep, storedSection } from './sections'

const calm = { lockKeyMissing: false, unreadable: 0, settingsReset: false }

test('the last section is remembered, anything unknown falls back to Genel', () => {
  expect(storedSection('history')).toBe('history')
  expect(storedSection('nope')).toBe('general')
  expect(storedSection(null)).toBe('general')
})

test('a reason to open Settings picks the section that explains it', () => {
  expect(reasonSection(calm)).toBeNull()
  expect(reasonSection({ ...calm, lockKeyMissing: true })).toBe('security')
  expect(reasonSection({ ...calm, unreadable: 3 })).toBe('security')
  expect(reasonSection({ ...calm, settingsReset: true })).toBe('general')
  expect(reasonSection({ ...calm, settingsReset: true, unreadable: 1 })).toBe('security')
})

test('arrows walk the rail and wrap, Home and End jump to the ends, other keys do nothing', () => {
  expect(railStep('general', 'ArrowDown')).toBe('shortcuts')
  expect(railStep('security', 'ArrowDown')).toBe('general')
  expect(railStep('general', 'ArrowUp')).toBe('security')
  expect(railStep('look', 'Home')).toBe('general')
  expect(railStep('look', 'End')).toBe('security')
  expect(railStep('look', 'ArrowRight')).toBeNull()
})
