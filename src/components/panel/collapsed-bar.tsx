import * as stylex from '@stylexjs/stylex'
import { ChevronUp, Clipboard } from 'lucide-react'
import { Button } from '@/components/ui/button'
import { t } from '@/lib/locale'
import { icon } from '@/lib/stylex-utils'
import { border, fontSize, letterSpacing, lineHeight, size, space } from '@/lib/constants.stylex'
import { ink, radius } from '@/tokens.stylex'
import type { PanelController } from './use-panel-controller'
import { ItemText, KindKey } from './item-parts'
import { dragHandle } from './panel-tools'

const s = stylex.create({
  bar: { flexGrow: 1, display: 'flex', alignItems: 'center', gap: space.s10, paddingInlineStart: space.s14, paddingInlineEnd: space.s8 },
  passive: { pointerEvents: 'none' },
  muted: { color: ink.textMuted },
  text: { flexGrow: 1, minWidth: 0 },
  count: { flexShrink: 0, minWidth: size.controlSm, paddingBlock: space.s1, paddingInline: space.s6, textAlign: 'center', borderWidth: border.stroke, borderStyle: 'solid', borderColor: ink.frame, borderRadius: radius.key, fontSize: fontSize.label, lineHeight: lineHeight.label, letterSpacing: letterSpacing.label, fontVariantNumeric: 'tabular-nums', color: ink.textControl },
  expand: { width: size.controlMd, paddingInline: 0, borderColor: 'transparent' },
})

export function CollapsedBar({ panel: c }: { panel: PanelController }) {
  const item = c.latest
  return (
    <div {...dragHandle(c)} {...stylex.props(s.bar)}>
      <Clipboard aria-hidden {...stylex.props(icon.md, s.muted, s.passive)} />
      {item && <KindKey item={item} />}
      <span {...stylex.props(s.text, s.passive)}>{item ? <ItemText item={item} /> : t('emptyTitle')}</span>
      <span aria-label={t('itemCount').replace('{n}', String(c.total))} {...stylex.props(s.count, s.passive)}>{c.total}</span>
      <Button tabIndex={-1} aria-label={t('expand')} title={t('expand')} onMouseDown={(e) => e.preventDefault()} onClick={c.expand} style={s.expand}>
        <ChevronUp aria-hidden {...stylex.props(icon.md)} />
      </Button>
    </div>
  )
}
