import * as stylex from '@stylexjs/stylex'
import { CornerDownLeft } from 'lucide-react'
import { t } from '@/lib/locale'
import type { I18NKeys } from '@/locale'
import { icon } from '@/lib/stylex-utils'
import { border, fontSize, lineHeight, space } from '@/lib/constants.stylex'
import { formatChord } from '@/utils/shortcuts'
import { ink, radius } from '@/tokens.stylex'
import type { PanelController } from './use-panel-controller'

export const keycap = stylex.create({
  kbd: { flexShrink: 0, justifySelf: 'start', display: 'inline-flex', alignItems: 'center', gap: space.s2, fontFamily: 'inherit', fontSize: fontSize.keycap, lineHeight: lineHeight.keycap, paddingBlock: space.s1, paddingInline: space.s5, borderWidth: border.stroke, borderStyle: 'solid', borderColor: ink.frame, borderRadius: radius.key, color: ink.textControl, whiteSpace: 'nowrap' },
})

const s = stylex.create({
  row: { flexShrink: 0, display: 'flex', flexWrap: 'wrap', alignItems: 'center', rowGap: space.s6, columnGap: space.s14, paddingTop: space.s10, borderTopWidth: border.stroke, borderTopStyle: 'solid', borderTopColor: ink.hairline, fontSize: fontSize.control, lineHeight: lineHeight.control, color: ink.textMuted },
  hint: { display: 'inline-flex', alignItems: 'center', gap: space.s5 },
  end: { marginInlineStart: 'auto' },
})

export type Hint = { keys: string; label: I18NKeys; end?: boolean }

export function Keycap({ text }: { text: string }) {
  const enter = text.endsWith('Enter')
  return (
    <kbd {...stylex.props(keycap.kbd)}>
      {enter ? text.slice(0, -'Enter'.length).replace(/\+$/, '') : text}
      {enter && <CornerDownLeft aria-label="Enter" {...stylex.props(icon.sm)} />}
    </kbd>
  )
}

export const closeHint = (c: PanelController, end = false): Hint => ({ keys: formatChord(c.shortcuts.close), label: c.keepOpen ? 'collapse' : 'scClose', end })

export function FooterHints({ hints, style }: { hints: Hint[]; style?: stylex.StyleXStyles }) {
  return (
    <div {...stylex.props(s.row, style)}>
      {hints.map((h) => (
        <span key={h.label} {...stylex.props(s.hint, h.end && s.end)}>
          <Keycap text={h.keys} />
          {t(h.label)}
        </span>
      ))}
    </div>
  )
}
