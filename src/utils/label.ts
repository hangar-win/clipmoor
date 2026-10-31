import { t } from '@/lib/locale'
import type { ItemView } from '@/types/services/items'

const fill = (template: string, n: number) => template.replace('{n}', new Intl.NumberFormat(t('locale'), { maximumFractionDigits: 1 }).format(n))

const time = (age: number) => {
  if (age < 60) return t('now')
  if (age < 3600) return fill(t('minutes'), Math.floor(age / 60))
  if (age < 86400) return fill(t('hours'), Math.floor(age / 3600))
  return fill(t('days'), Math.floor(age / 86400))
}

export const bytes = (n: number) => (n < 1048576 ? `${fill('{n}', Math.round(n / 1024))} KB` : `${fill('{n}', Math.round(n / 104857.6) / 10)} MB`)

const size = ({ kind, size, preview }: ItemView) => {
  if (size === null) return ''
  if (kind === 'color') return (preview ?? '').trim().startsWith('#') ? 'HEX' : (preview ?? '').trim().slice(0, 3).toUpperCase()
  if (kind === 'image') return bytes(size)
  return fill(t(size === 1 ? 'char' : 'chars'), size)
}

const KNOWN: Record<string, string> = { code: 'VS Code', devenv: 'Visual Studio', msedge: 'Edge', windowsterminal: 'Terminal', snippingtool: 'Snipping Tool' }

export const sourceName = (s: string | null) => {
  const stem = (s ?? '').replace(/\.exe$/i, '')
  const name = KNOWN[stem.toLowerCase()] ?? (stem === stem.toUpperCase() ? stem.toLowerCase() : stem)
  return name.charAt(0).toUpperCase() + name.slice(1)
}

export function codeLines(preview: string, max = 2): string[] {
  const lines = preview.split(/\r?\n/).filter((l) => l.trim()).slice(0, max)
  const indent = Math.min(...lines.map((l) => l.length - l.trimStart().length))
  return lines.map((l) => l.slice(indent).trimEnd())
}

export function chipLabel(item: ItemView, now: number) {
  return { source: sourceName(item.source), size: size(item), time: time(Math.max(0, now - item.lastUsed)) }
}

export const oneLine = (preview: string | null) => (preview ?? '').replace(/\s+/g, ' ').trim()

export const kindName = (item: ItemView) => t(item.kind ?? 'locked')

export const itemMeta = (item: ItemView, now: number, withSource = false) => [kindName(item), chipLabel(item, now).time, withSource && item.source ? sourceName(item.source) : ''].filter(Boolean).join(' · ')

export function urlParts(url: string): [string, string, string] {
  const host = URL.canParse(url) ? new URL(url).host : ''
  const i = host ? url.indexOf(host) : -1
  return i < 0 ? [url, '', ''] : [url.slice(0, i), host, url.slice(i + host.length)]
}
