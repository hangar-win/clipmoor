import { afterEach, expect, test } from 'vitest'
import { en, tr } from '@/locale'
import { resolveLang, setLang, t } from './locale'

afterEach(() => setLang('tr'))

test('turkish is the default and english switches every label', () => {
  expect(t('paste')).toBe('Yapıştır')
  setLang('en')
  expect(t('paste')).toBe('Paste')
  expect(t('plainPaste')).toBe('Plain paste')
})

test('system language follows the browser locale, an explicit choice wins', () => {
  expect(resolveLang('system', 'tr-TR')).toBe('tr')
  expect(resolveLang('system', 'TR')).toBe('tr')
  expect(resolveLang('system', 'en-GB')).toBe('en')
  expect(resolveLang('tr', 'en-US')).toBe('tr')
  expect(resolveLang('en', 'tr-TR')).toBe('en')
})

test('every language has exactly the turkish keys', () => {
  expect(Object.keys(en).sort()).toEqual(Object.keys(tr).sort())
})
