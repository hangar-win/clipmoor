import { useEffect, useState } from 'react'
import * as stylex from '@stylexjs/stylex'
import { RadioGroup } from '@base-ui/react/radio-group'
import { Radio } from '@base-ui/react/radio'
import { Check, TriangleAlert } from 'lucide-react'
import { icon } from '@/lib/stylex-utils'
import { Button } from '@/components/ui/button'
import { border } from '@/lib/constants.stylex'
import { t } from '@/lib/locale'
import type { I18NKeys } from '@/locale'
import type { ColorKey, CustomColors, Mode } from '@/types/services/settings'
import { PRESETS, applyPreset, colorSet, colorWarnings, normalizeHex, palette, presetOf, presetState, sameSet, type ColorWarning, type Preset, type PresetId } from '@/utils/colors'
import { s } from './styles'

const ratio = (n: number) => new Intl.NumberFormat(document.documentElement.lang || 'tr', { maximumFractionDigits: 1 }).format(Math.floor(n * 10) / 10)
const warningText = (w: ColorWarning) => (w.lightGround ? t('colWarnLight') : t('colWarn').replace('{on}', t(w.on === 'card' ? 'colCard' : 'colGround')).replace('{ratio}', ratio(w.ratio!)).replace('{min}', ratio(w.min!)))

function ColorField({ label, value, warnings, onCommit }: { label: string; value: string; warnings: string[]; onCommit: (hex: string) => void }) {
  const [draft, setDraft] = useState(value)
  useEffect(() => setDraft(value), [value])
  const invalid = draft.trim() !== '' && normalizeHex(draft) === null
  const commit = (v: string) => {
    const hex = normalizeHex(v)
    if (hex && hex !== value) onCommit(hex)
  }
  const edit = (v: string) => {
    setDraft(v)
    if (v.replace('#', '').trim().length === 6) commit(v)
  }
  const done = () => {
    commit(draft)
    setDraft(value)
  }
  return (
    <div {...stylex.props(s.colorRow)}>
      <div {...stylex.props(s.text)}>
        <span>{label}</span>
        {invalid && <span role="alert" {...stylex.props(s.hint, s.error)}>{t('colInvalid')}</span>}
        {warnings.map((w) => <span key={w} {...stylex.props(s.hint, s.contrastWarn)}><TriangleAlert {...stylex.props(icon.sm, s.warnIcon)} />{w}</span>)}
      </div>
      <div {...stylex.props(s.keys)}>
        <input type="color" aria-label={label} value={(value || '#E74C3C').toLowerCase()} onChange={(e) => edit(e.target.value)} {...stylex.props(s.swatch, s.focus)} />
        <input aria-label={`${label} hex`} aria-invalid={invalid} spellCheck={false} maxLength={7} value={draft} onChange={(e) => edit(e.target.value)} onBlur={done} onKeyDown={(e) => e.key === 'Enter' && done()} {...stylex.props(s.input, s.focus, s.hex)} />
      </div>
    </div>
  )
}

const BASE_ROWS: [ColorKey, I18NKeys][] = [['ground', 'colGround'], ['card', 'colCard'], ['text', 'colText'], ['selection', 'colSelection']]
const TYPE_ROWS: [ColorKey, I18NKeys][] = [['typeText', 'text'], ['typeImage', 'image'], ['typeLink', 'link'], ['typeCode', 'code']]

const ld = (light: string, dark: string) => `light-dark(${light}, ${dark})`
const HUES = ['typeText', 'typeImage', 'typeLink', 'typeCode'] as const

function PaletteCard({ preset, selected, modified }: { preset: Preset; selected: boolean; modified: boolean }) {
  const L = palette('light', preset.light)
  const D = palette('dark', preset.dark)
  const c = (k: keyof typeof L) => ld(L[k]!, D[k]!)
  const name = t(`pal_${preset.id}` as const)
  return (
    <Radio.Root value={preset.id} aria-label={modified ? `${name} (${t('palModified')})` : name} {...stylex.props(s.card, s.focus, selected && s.cardOn)}>
      <span aria-hidden style={{ backgroundColor: c('ground'), boxShadow: `inset 0 0 0 ${border.stroke} ${c('frame')}` }} {...stylex.props(s.sw)}>
        <span style={{ backgroundColor: c('card'), borderColor: ld(L.selectionEdge, D.hairline), outlineColor: c('selection') }} {...stylex.props(s.swChip)}>
          <i style={{ backgroundColor: c('typeText') }} />
          <span {...stylex.props(s.swBody)}>
            <i style={{ backgroundColor: c('text'), width: '80%' }} {...stylex.props(s.swLine)} />
            <i style={{ backgroundColor: c('textMuted'), width: '50%' }} {...stylex.props(s.swLine)} />
          </span>
        </span>
        <span {...stylex.props(s.swDots)}>{HUES.map((k) => <i key={k} style={{ backgroundColor: c(k) }} {...stylex.props(s.swDot)} />)}</span>
      </span>
      <span {...stylex.props(s.cardName, selected && s.cardNameOn)}>
        <span>{name}</span>
        {selected && <Check {...stylex.props(icon.sm)} />}
      </span>
      <span aria-hidden={!modified} {...stylex.props(s.hint, s.cardHint, !modified && s.unmodified)}>{t('palModified')}</span>
    </Radio.Root>
  )
}

export function PaletteCards({ value, onChange }: { value: CustomColors; onChange: (c: CustomColors) => void }) {
  const state = presetState(value)
  const choose = (id: PresetId) => onChange(applyPreset(value, id))
  return (
    <RadioGroup aria-label={t('palette')} value={state.id} onValueChange={(v) => choose(v as PresetId)} onClick={(e) => state.modified && (e.target as HTMLElement).closest('[data-checked]') && choose(state.id)} {...stylex.props(s.cards)}>
      {PRESETS.map((p) => <PaletteCard key={p.id} preset={p} selected={p.id === state.id} modified={p.id === state.id && state.modified} />)}
    </RadioGroup>
  )
}

export function ColorEditor({ value, mode, onMode, onChange }: { value: CustomColors; mode: Mode; onMode: (m: Mode) => void; onChange: (c: CustomColors) => void }) {
  const set = value[mode]
  const base = presetOf(value.preset)[mode]
  const full = colorSet(mode, set)
  const warnings = colorWarnings(mode, set)
  const untouched = sameSet(mode, set, base)
  const field = ([k, label]: [ColorKey, I18NKeys]) => (
    <ColorField key={`${mode}-${k}`} label={t(label)} value={full[k] ?? ''} warnings={warnings.filter((w) => w.key === k).map(warningText)} onCommit={(hex) => onChange({ ...value, [mode]: { ...set, [k]: hex } })} />
  )
  return (
    <div {...stylex.props(s.row, s.stack)}>
      <div {...stylex.props(s.editorHead)}>
        <span>{t('colorSetLabel')}</span>
        <div {...stylex.props(s.keys)}>
          <Button size="sm" disabled={untouched} onClick={() => onChange({ ...value, [mode]: { ...base } })}>{t('scReset')}</Button>
          <RadioGroup aria-label={t('colorSetLabel')} value={mode} onValueChange={(v) => onMode(v as Mode)} {...stylex.props(s.segments)}>
            {(['dark', 'light'] as const).map((m) => {
              const k: I18NKeys = m === 'dark' ? 'themeDark' : 'themeLight'
              return <Radio.Root key={m} value={m} data-label={t(k)} {...stylex.props(s.segment, s.focus)}>{t(k)}</Radio.Root>
            })}
          </RadioGroup>
        </div>
      </div>
      <div {...stylex.props(s.fields)}>
        {BASE_ROWS.map(field)}
        <span {...stylex.props(s.hint, s.sub)}>{t('colTypes')}</span>
        {TYPE_ROWS.map(field)}
      </div>
    </div>
  )
}
