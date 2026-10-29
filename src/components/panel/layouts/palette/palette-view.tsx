import * as stylex from '@stylexjs/stylex'
import { X } from 'lucide-react'
import { t } from '@/lib/locale'
import { icon } from '@/lib/stylex-utils'
import { border, fontSize, lineHeight, size, space } from '@/lib/constants.stylex'
import type { LayoutViewProps } from '@/types/panel-layout'
import { formatChord, navHint } from '@/utils/shortcuts'
import { ink, radius } from '@/tokens.stylex'
import { CompactRow } from '../../compact-row'
import { FooterHints, closeHint, type Hint } from '../../footer-hints'
import { cardProps } from '../../item-parts'
import { ItemsFrame } from '../../items-frame'
import { ListCard } from '../../list-card'
import { PanelNotice, PanelTools, dragHandle } from '../../panel-tools'
import { SearchField } from '../../search-field'
import { TabKey } from '../../type-tabs'
import { DetailPane } from './detail-pane'

const s = stylex.create({
  root: { flexGrow: 1, minHeight: 0, display: 'flex', flexDirection: 'column' },
  head: { flexShrink: 0, display: 'flex', alignItems: 'center', gap: space.s8, height: size.queryH, paddingInline: space.s16, borderBottomWidth: border.stroke, borderBottomStyle: 'solid', borderBottomColor: ink.hairline },
  query: { flexGrow: 0, marginInlineEnd: 'auto' },
  pill: { flexShrink: 0, display: 'flex', alignItems: 'center', gap: space.s5, maxWidth: size.menuMinW, height: size.controlSm, paddingInline: space.s9, borderWidth: border.stroke, borderStyle: 'solid', borderColor: ink.frame, borderRadius: radius.control, fontSize: fontSize.control, lineHeight: lineHeight.control, color: ink.textControl, cursor: 'default', backgroundColor: { default: 'transparent', ':hover': ink.strip } },
  pillName: { minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' },
  notice: { marginInline: space.s16, marginTop: space.s12 },
  main: { flexGrow: 1, minHeight: 0, display: 'grid', gridTemplateColumns: `${size.paletteListW} minmax(0, 1fr)` },
  list: { minHeight: 0, display: 'flex', flexDirection: 'column', padding: space.s12, borderRightWidth: border.stroke, borderRightStyle: 'solid', borderRightColor: ink.hairline },
  foot: { paddingInline: space.s16, paddingBottom: space.s10 },
})

export function PaletteView({ panel: c }: LayoutViewProps) {
  const { query, items, selected } = c.state
  const Row = c.compact ? CompactRow : ListCard
  const item = items[selected]
  const group = query.group.type === 'pinned' ? t('pinned') : query.group.type === 'group' ? c.groups.find((g) => query.group.type === 'group' && g.id === query.group.id)?.name : null
  const hints: Hint[] = [
    { keys: navHint(c.layout.nav, 'prev', 'next'), label: 'select' },
    { keys: formatChord(c.shortcuts.paste), label: 'paste' },
    { keys: formatChord(c.shortcuts.plainPaste), label: 'plainPaste' },
    { keys: 'Tab', label: 'typeFilter' },
    closeHint(c, true),
  ]

  return (
    <div {...stylex.props(s.root)}>
      <div {...dragHandle(c)} {...stylex.props(s.head)}>
        <SearchField variant="query" text={query.text} selected={!!c.state.textSelected} style={s.query} />
        <button type="button" tabIndex={-1} aria-label={t('typeFilter')} title={t('typeFilter')} onMouseDown={(e) => e.preventDefault()} onClick={c.nextKind} {...stylex.props(s.pill)}>
          {query.kind && <TabKey kind={query.kind} />}
          {t(query.kind ?? 'all')}
        </button>
        {group && (
          <button type="button" tabIndex={-1} title={group} onMouseDown={(e) => e.preventDefault()} onClick={() => c.pickGroup({ type: 'all' })} {...stylex.props(s.pill)}>
            <span {...stylex.props(s.pillName)}>{group}</span>
            <X aria-hidden {...stylex.props(icon.sm)} />
          </button>
        )}
        <PanelTools panel={c} />
      </div>
      <PanelNotice panel={c} style={s.notice} />
      <div {...stylex.props(s.main)}>
        <div {...stylex.props(s.list)}>
          <ItemsFrame panel={c}>
            {items.map((it, i) => <Row key={it.id} {...cardProps(c, it, i)} />)}
          </ItemsFrame>
        </div>
        {item ? <DetailPane panel={c} item={item} /> : <div />}
      </div>
      <FooterHints hints={hints} style={s.foot} />
    </div>
  )
}
