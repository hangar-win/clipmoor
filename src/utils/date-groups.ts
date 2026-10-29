import type { ItemView } from '@/types/services/items'

export type SectionKey = 'pinned' | 'today' | 'yesterday' | 'thisWeek' | 'older'
export type Section = { key: SectionKey; start: number; items: ItemView[] }

export const pinsFirst = (items: ItemView[]) => (items.some((i) => i.pinned) ? [...items.filter((i) => i.pinned), ...items.filter((i) => !i.pinned)] : items)

const dayStart = (now: Date, daysBack: number) => new Date(now.getFullYear(), now.getMonth(), now.getDate() - daysBack).getTime() / 1000

export function sectionOf(item: ItemView, now: Date): SectionKey {
  if (item.pinned) return 'pinned'
  if (item.lastUsed >= dayStart(now, 0)) return 'today'
  if (item.lastUsed >= dayStart(now, 1)) return 'yesterday'
  return item.lastUsed >= dayStart(now, (now.getDay() + 6) % 7) ? 'thisWeek' : 'older'
}

export const pinAllIds = (section: Section) => (section.key === 'pinned' ? [] : section.items.filter((i) => !i.pinned).map((i) => i.id))

export function groupItems(items: ItemView[], now: Date): Section[] {
  const out: Section[] = []
  items.forEach((item, i) => {
    const key = sectionOf(item, now)
    const last = out[out.length - 1]
    if (last?.key === key) last.items.push(item)
    else out.push({ key, start: i, items: [item] })
  })
  return out
}
