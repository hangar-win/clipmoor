import { en, tr, type I18NKeys, type LocaleType } from '@/locale'

let dict: Record<I18NKeys, string> = tr

export function resolveLang(language: 'system' | LocaleType, locale: string): LocaleType {
  return language !== 'system' ? language : locale.toLowerCase().startsWith('tr') ? 'tr' : 'en'
}

export function setLang(l: LocaleType): void {
  dict = l === 'en' ? en : tr
  if (typeof document !== 'undefined') document.documentElement.lang = l
}

export function t(key: I18NKeys): string {
  return dict[key]
}
