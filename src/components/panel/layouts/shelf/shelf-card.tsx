import * as stylex from '@stylexjs/stylex'
import { Lock } from 'lucide-react'
import { t } from '@/lib/locale'
import { icon } from '@/lib/stylex-utils'
import { border, fontSize, fontWeight, letterSpacing, lineHeight, size, space } from '@/lib/constants.stylex'
import { chipLabel, codeLines, kindName } from '@/utils/label'
import { ink, radius } from '@/tokens.stylex'
import { Flags, ItemShell, QuickHint, colorFill, itemStyles, previewHover, type CardProps } from '../../item-parts'

const s = stylex.create({
  card: { height: size.shelfCardH },
  body: { display: 'grid', gridTemplateRows: 'auto minmax(0, 1fr) auto', gap: space.s10, padding: space.s12, minWidth: 0 },
  head: { display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: space.s8, minWidth: 0, fontSize: fontSize.controlStrong, lineHeight: lineHeight.controlStrong, fontWeight: fontWeight.controlStrong, color: ink.textMuted },
  name: { minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' },
  content: { minHeight: 0, overflow: 'hidden', fontSize: fontSize.body, lineHeight: lineHeight.body, overflowWrap: 'anywhere' },
  clamp: { display: '-webkit-box', WebkitBoxOrient: 'vertical', WebkitLineClamp: 4 },
  media: { display: 'flex', flexDirection: 'column', gap: space.s6 },
  block: { display: 'block', width: '100%', height: size.shelfMediaH, flexShrink: 0, borderRadius: radius.key, boxShadow: `inset 0 0 0 ${border.stroke} ${ink.swatchEdge}`, objectFit: 'cover' },
  locked: { display: 'flex', alignItems: 'center', gap: space.s8, color: ink.textMuted },
  time: { display: 'flex', alignItems: 'center', gap: space.s6, fontSize: fontSize.label, lineHeight: lineHeight.label, letterSpacing: letterSpacing.label, fontVariantNumeric: 'tabular-nums', color: ink.textMuted },
})

function Content({ item, thumb, onPreview }: Pick<CardProps, 'item' | 'thumb' | 'onPreview'>) {
  const hover = previewHover(onPreview)
  switch (item.kind) {
    case null: return <div {...stylex.props(s.content, s.locked)}><Lock aria-hidden {...stylex.props(icon.sm)} />{t('lockedItem')}</div>
    case 'image':
      return <div {...stylex.props(s.content)}>{thumb ? <img src={thumb} alt="" draggable={false} {...hover} {...stylex.props(s.block)} /> : <div {...stylex.props(s.block)} />}</div>
    case 'color':
      return (
        <div {...hover} {...stylex.props(s.content, s.media)}>
          <div {...stylex.props(s.block, colorFill(item))} />
          <span {...stylex.props(itemStyles.line, itemStyles.code)}>{item.preview?.trim()}</span>
        </div>
      )
    case 'code': return <div {...hover} {...stylex.props(s.content, itemStyles.code)}>{codeLines(item.preview ?? '', 4).map((l, i) => <span key={i} {...stylex.props(itemStyles.line, itemStyles.code)}>{l}</span>)}</div>
    default: return <div {...hover} {...stylex.props(s.content, s.clamp)}>{item.preview}</div>
  }
}

export function ShelfCard({ item, selected, thumb, now, quick, onActivate, onHover, onPreview }: CardProps) {
  return (
    <ItemShell item={item} selected={selected} onActivate={onActivate} onHover={onHover} style={s.card}>
      <div {...stylex.props(s.body)}>
        <div {...stylex.props(s.head)}>
          <span {...stylex.props(s.name)}>{kindName(item)}</span>
          <QuickHint text={quick} />
        </div>
        <Content item={item} thumb={thumb} onPreview={onPreview} />
        <div {...stylex.props(s.time)}>
          <Flags item={item} />
          {chipLabel(item, now).time}
        </div>
      </div>
    </ItemShell>
  )
}
