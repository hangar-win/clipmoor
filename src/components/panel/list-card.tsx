import * as stylex from '@stylexjs/stylex'
import { fontSize, lineHeight, space } from '@/lib/constants.stylex'
import { itemMeta } from '@/utils/label'
import { ink } from '@/tokens.stylex'
import { Flags, ItemShell, ItemText, QuickHint, type CardProps } from './item-parts'

const s = stylex.create({
  body: { display: 'grid', minWidth: 0, paddingBlock: space.s6, paddingInline: space.s12 },
  meta: { display: 'flex', alignItems: 'center', gap: space.s5, minWidth: 0, fontSize: fontSize.control, lineHeight: lineHeight.control, color: ink.textMuted },
  text: { minWidth: 0, flexGrow: 1, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' },
})

export function ListCard({ item, selected, thumb, now, quick, onActivate, onHover, onPreview }: CardProps) {
  return (
    <ItemShell item={item} selected={selected} onActivate={onActivate} onHover={onHover}>
      <div {...stylex.props(s.body)}>
        <ItemText item={item} thumb={thumb} onPreview={onPreview} />
        <div {...stylex.props(s.meta)}>
          <Flags item={item} />
          <span {...stylex.props(s.text)}>{itemMeta(item, now)}</span>
          <QuickHint text={quick} />
        </div>
      </div>
    </ItemShell>
  )
}
