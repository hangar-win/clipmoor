import * as stylex from '@stylexjs/stylex'
import { fontSize, lineHeight, space } from '@/lib/constants.stylex'
import { t } from '@/lib/locale'
import type { Shortcuts } from '@/types/services/settings'
import type { NavMap } from '@/types/panel-layout'
import { formatChord, formatQuick, navHint } from '@/utils/shortcuts'
import { ink } from '@/tokens.stylex'
import { keycap } from './footer-hints'

const s = stylex.create({
  list: { display: 'grid', gridTemplateColumns: 'auto 1fr', alignItems: 'center', columnGap: space.s10, rowGap: space.s6, paddingBlock: space.s6, paddingInline: space.s10, fontSize: fontSize.control, lineHeight: lineHeight.control, color: ink.textMuted },
})

export function ShortcutHints({ shortcuts: sc, nav }: { shortcuts: Shortcuts; nav: NavMap }) {
  const keys = [
    [formatChord(sc.paste), 'paste'],
    [formatChord(sc.copy), 'copy'],
    [formatChord(sc.plainPaste), 'plainPaste'],
    [formatChord(sc.pin), 'pin'],
    [formatQuick(sc.quick), 'quickNumber'],
    [formatChord(sc.preview), 'preview'],
    [navHint(nav, 'prev', 'next'), 'move'],
    [`Tab ${navHint(nav, 'prevKind', 'nextKind')}`, 'switchTab'],
    [formatChord(sc.source), 'sourceFilter'],
  ] as const
  return (
    <div {...stylex.props(s.list)}>
      {keys.map(([key, label]) => [<kbd key={`${label}-key`} {...stylex.props(keycap.kbd)}>{key}</kbd>, <span key={label}>{t(label)}</span>])}
    </div>
  )
}
