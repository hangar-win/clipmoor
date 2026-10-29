import { hue, ink } from '@/tokens.stylex'
import type { CustomColors, Theme } from '@/types/services/settings'
import { resolveColors } from '@/utils/colors'

const varName = (token: unknown) => /var\((--[^),\s]+)/.exec(String(token))![1]

function put<G, K extends keyof G & string>(out: Record<string, string>, group: G, values: Record<K, string>) {
  for (const k of Object.keys(values) as K[]) out[varName(group[k])] = values[k]
}

export function colorVars(c: CustomColors): Record<string, string> {
  const r = resolveColors(c)
  if (!r) return {}
  const out: Record<string, string> = {}
  put(out, ink, r.ink)
  put(out, hue, r.hue)
  return out
}

let applied: string[] = []

export function applyColors(c: CustomColors): void {
  const style = document.documentElement.style
  for (const p of applied) style.removeProperty(p)
  const vars = colorVars(c)
  for (const [p, v] of Object.entries(vars)) style.setProperty(p, v)
  applied = Object.keys(vars)
}

export function applyTheme(t: Theme): void {
  if (t === 'system') document.documentElement.removeAttribute('data-theme')
  else document.documentElement.dataset.theme = t
}
