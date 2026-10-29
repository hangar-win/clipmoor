import * as stylex from '@stylexjs/stylex'
import { fontSize, lineHeight, size, space } from '@/lib/constants.stylex'
import { chipLabel } from '@/utils/label'
import { ink } from '@/tokens.stylex'
import { Flags, ItemShell, ItemText, QuickHint, type CardProps } from './item-parts'

const s = stylex.create({
  row: { height: size.compactRowH },
  body: { display: 'flex', alignItems: 'center', gap: space.s12, minWidth: 0, paddingInline: space.s12 },
  text: { flexGrow: 1, minWidth: 0 },
  time: { flexShrink: 0, display: 'inline-flex', alignItems: 'center', gap: space.s6, fontSize: fontSize.control, lineHeight: lineHeight.control, fontVariantNumeric: 'tabular-nums', color: ink.textMuted },
})

export function CompactRow({ item, selected, thumb, now, quick, onActivate, onHover, onPreview }: CardProps) {
  return (
    <ItemShell item={item} selected={selected} onActivate={onActivate} onHover={onHover} style={s.row}>
      <div {...stylex.props(s.body)}>
        <span {...stylex.props(s.text)}>
          <ItemText item={item} thumb={thumb} onPreview={onPreview} />
        </span>
        <span {...stylex.props(s.time)}>
          <Flags item={item} />
          {chipLabel(item, now).time}
          <QuickHint text={quick} />
        </span>
      </div>
    </ItemShell>
  )
}
