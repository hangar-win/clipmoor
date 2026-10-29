import * as stylex from '@stylexjs/stylex'
import { Pin } from 'lucide-react'
import { ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuTrigger } from '@/components/ui/context-menu'
import { t } from '@/lib/locale'
import { icon } from '@/lib/stylex-utils'
import { fontSize, fontWeight, lineHeight, size, space } from '@/lib/constants.stylex'
import type { SectionKey } from '@/utils/date-groups'
import { ink, radius } from '@/tokens.stylex'

const s = stylex.create({
  head: { flexShrink: 0, display: 'flex', alignItems: 'center', gap: space.s6, paddingTop: space.s6, paddingInline: space.s3, fontSize: fontSize.controlStrong, lineHeight: lineHeight.controlStrong, fontWeight: fontWeight.controlStrong, color: ink.textMuted },
  count: { marginInlineStart: 'auto', fontWeight: fontWeight.control, fontVariantNumeric: 'tabular-nums' },
  pinAll: { flexShrink: 0, display: 'grid', placeItems: 'center', width: size.iconMd, height: size.iconMd, borderRadius: radius.item, cursor: 'default', color: { default: ink.textMuted, ':hover': ink.text }, backgroundColor: { default: 'transparent', ':hover': ink.strip } },
})

type Props = { section: SectionKey; count: number; shown?: number; onPinAll?: () => void }

export function SectionHeader({ section, count, shown, onPinAll }: Props) {
  const content = [
    section === 'pinned' && <Pin key="icon" aria-hidden {...stylex.props(icon.sm)} />,
    t(section),
    <span key="count" {...stylex.props(s.count)}>{count}</span>,
  ]
  if (!onPinAll) return <li role="presentation" {...stylex.props(s.head)}>{content}</li>
  return (
    <ContextMenu key={shown}>
      <ContextMenuTrigger render={<li role="presentation" />} style={s.head}>
        {content}
        <button type="button" tabIndex={-1} aria-label={t('pinAll')} title={t('pinAll')} onMouseDown={(e) => e.preventDefault()} onClick={onPinAll} {...stylex.props(s.pinAll)}>
          <Pin aria-hidden {...stylex.props(icon.sm)} />
        </button>
      </ContextMenuTrigger>
      <ContextMenuContent>
        <ContextMenuItem onClick={onPinAll}>{t('pinAll')}</ContextMenuItem>
      </ContextMenuContent>
    </ContextMenu>
  )
}
