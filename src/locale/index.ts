import type { LocalizationKeys } from './tr'

export { default as tr } from './tr'
export { default as en } from './en'

export type LocaleType = 'tr' | 'en'
export type I18NKeys = keyof LocalizationKeys
