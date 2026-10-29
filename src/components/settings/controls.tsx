import { useEffect, useState, type ReactNode } from 'react'
import * as stylex from '@stylexjs/stylex'
import { Switch as BaseSwitch } from '@base-ui/react/switch'
import { RadioGroup } from '@base-ui/react/radio-group'
import { Radio } from '@base-ui/react/radio'
import { TriangleAlert } from 'lucide-react'
import { icon } from '@/lib/stylex-utils'
import { Button } from '@/components/ui/button'
import { t } from '@/lib/locale'
import type { I18NKeys } from '@/locale'
import type { Chord } from '@/types/services/settings'
import { chordFromKey } from '@/utils/shortcuts'
import { s } from './styles'

export function Row({ label, hint, hintText, children }: { label: I18NKeys; hint?: I18NKeys; hintText?: string; children: ReactNode }) {
  return (
    <div {...stylex.props(s.row)}>
      <div {...stylex.props(s.text)}>
        <span>{t(label)}</span>
        {(hintText || hint) && <span {...stylex.props(s.hint)}>{hintText ?? t(hint!)}</span>}
      </div>
      {children}
    </div>
  )
}

export function Toggle({ label, hint, hintText, checked, onChange }: { label: I18NKeys; hint?: I18NKeys; hintText?: string; checked: boolean; onChange: (on: boolean) => void }) {
  return (
    <Row label={label} hint={hint} hintText={hintText}>
      <BaseSwitch.Root aria-label={t(label)} checked={checked} onCheckedChange={(on) => onChange(on)} {...stylex.props(s.focus, s.switchRoot)}>
        <BaseSwitch.Thumb {...stylex.props(s.thumb)} />
      </BaseSwitch.Root>
    </Row>
  )
}

export function Choice<T extends string>({ label, hint, value, options, onChange }: { label: I18NKeys; hint?: I18NKeys; value: T; options: [T, I18NKeys][]; onChange: (v: T) => void }) {
  return (
    <Row label={label} hint={hint}>
      <RadioGroup aria-label={t(label)} value={value} onValueChange={(v) => onChange(v as T)} {...stylex.props(s.segments)}>
        {options.map(([v, k]) => (
          <Radio.Root key={v} value={v} data-label={t(k)} {...stylex.props(s.segment, s.focus)}>{t(k)}</Radio.Root>
        ))}
      </RadioGroup>
    </Row>
  )
}

export function NumberField({ label, value, max, onCommit }: { label: string; value: number; max: number; onCommit: (n: number) => void }) {
  const [draft, setDraft] = useState(String(value))
  useEffect(() => setDraft(String(value)), [value])
  const commit = () => {
    const n = Math.round(Number(draft))
    if (Number.isFinite(n) && n >= 1) onCommit(Math.min(n, max))
    else setDraft(String(value))
  }
  return (
    <input type="number" inputMode="numeric" min={1} max={max} aria-label={label} value={draft} onChange={(e) => setDraft(e.target.value)} onBlur={commit} onKeyDown={(e) => e.key === 'Enter' && commit()} {...stylex.props(s.input, s.focus, s.number)} />
  )
}

export function ShortcutRow({ label, hint, text, recording, message, isDefault, onStart, onStop, onKey, onReset }: { label: I18NKeys; hint?: I18NKeys; text: string; recording: boolean; message?: { text: string; error: boolean }; isDefault: boolean; onStart: () => void; onStop: () => void; onKey: (c: Chord) => void; onReset: () => void }) {
  return (
    <div {...stylex.props(s.row)}>
      <div {...stylex.props(s.text)}>
        <span>{t(label)}</span>
        {hint && <span {...stylex.props(s.hint)}>{t(hint)}</span>}
        {message && (
          <span role={message.error ? 'alert' : undefined} {...stylex.props(s.hint, message.error ? s.error : s.caution)}>
            {message.error && <TriangleAlert aria-hidden {...stylex.props(icon.sm, s.warnIcon)} />}
            {message.text}
          </span>
        )}
      </div>
      <div {...stylex.props(s.keys)}>
        <button
          type="button"
          aria-label={t(label)}
          aria-pressed={recording}
          onClick={() => (recording ? onStop() : onStart())}
          onBlur={() => recording && onStop()}
          onKeyDown={(e) => {
            if (!recording) return
            e.preventDefault()
            const c = chordFromKey(e)
            if (c) onKey(c)
          }}
          {...stylex.props(s.field, s.focus, recording && s.recording, message?.error && s.fieldError)}
        >
          {recording ? t('scRecording') : text}
        </button>
        <Button size="sm" disabled={isDefault} onClick={onReset}>{t('scReset')}</Button>
      </div>
    </div>
  )
}
