export const SECTIONS = ['general', 'shortcuts', 'look', 'history', 'security'] as const
export type Section = (typeof SECTIONS)[number]

export const storedSection = (v: string | null): Section => SECTIONS.find((s) => s === v) ?? 'general'

const SECTION_KEY = 'settings.section'

export function loadSection(): Section {
  try {
    return storedSection(localStorage.getItem(SECTION_KEY))
  } catch {
    return storedSection(null)
  }
}

export function saveSection(s: Section): void {
  try {
    localStorage.setItem(SECTION_KEY, s)
  } catch {}
}

export function reasonSection(info: { lockKeyMissing: boolean; unreadable: number; settingsReset: boolean }): Section | null {
  if (info.lockKeyMissing || info.unreadable > 0) return 'security'
  return info.settingsReset ? 'general' : null
}

export function railStep(current: Section, key: string): Section | null {
  const i = SECTIONS.indexOf(current)
  const n = SECTIONS.length
  const to = { ArrowDown: (i + 1) % n, ArrowUp: (i + n - 1) % n, Home: 0, End: n - 1 }[key]
  return to === undefined ? null : SECTIONS[to]
}
