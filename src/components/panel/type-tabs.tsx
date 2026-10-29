import * as stylex from '@stylexjs/stylex'
import { t } from '@/lib/locale'
import type { Kind } from '@/types/services/items'
import { hue, ink, radius } from '@/tokens.stylex'
import { KIND_TABS } from '@/utils/panel-state'
import { border, fontSize, fontWeight, lineHeight, size, space } from '@/lib/constants.stylex'

const s = stylex.create({
  row: { display: 'flex', gap: space.s4 },
  tab: { display: 'flex', alignItems: 'center', gap: space.s6, paddingBlock: space.s4, paddingInline: space.s9, borderWidth: border.stroke, borderStyle: 'solid', borderColor: 'transparent', borderRadius: radius.control, fontSize: fontSize.title, lineHeight: lineHeight.title, color: ink.textMuted, cursor: 'default', backgroundColor: { default: null, ':hover': ink.strip } },
  on: { borderColor: ink.activeEdge, backgroundColor: { default: ink.active, ':hover': ink.active }, color: ink.textOnActive, fontWeight: fontWeight.titleStrong },
  key: { width: size.tabKey, height: size.tabKey, borderRadius: radius.dot },
  text: { backgroundColor: hue.text },
  image: { backgroundColor: hue.image },
  link: { backgroundColor: hue.link },
  color: { backgroundImage: hue.color },
  code: { backgroundColor: hue.code },
})

export const TabKey = ({ kind }: { kind: Kind }) => <i aria-hidden {...stylex.props(s.key, s[kind])} />

export function TypeTabs({ kind, onPick, style }: { kind: Kind | null; onPick: (k: Kind | null) => void; style?: stylex.StyleXStyles }) {
  return (
    <div role="tablist" {...stylex.props(s.row, style)}>
      {KIND_TABS.map((k) => (
        <div key={k ?? 'all'} role="tab" aria-selected={k === kind} onClick={() => onPick(k)} {...stylex.props(s.tab, k === kind && s.on)}>
          {k && <TabKey kind={k} />}
          {t(k ?? 'all')}
        </div>
      ))}
    </div>
  )
}
