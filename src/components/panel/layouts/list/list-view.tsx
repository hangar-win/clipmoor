import * as stylex from '@stylexjs/stylex'
import { ChevronRight, ChevronUp } from 'lucide-react'
import { t } from '@/lib/locale'
import { icon } from '@/lib/stylex-utils'
import { border, fontSize, fontWeight, lineHeight, size, space } from '@/lib/constants.stylex'
import type { LayoutViewProps } from '@/types/panel-layout'
import { groupItems, pinAllIds } from '@/utils/date-groups'
import { formatChord, navHint } from '@/utils/shortcuts'
import { ink, radius } from '@/tokens.stylex'
import { CompactRow } from '../../compact-row'
import { FooterHints, closeHint, type Hint } from '../../footer-hints'
import { GroupPills } from '../../group-pills'
import { cardProps } from '../../item-parts'
import { ItemsFrame } from '../../items-frame'
import { ListCard } from '../../list-card'
import { GroupMenu } from '../../menus'
import { PanelNotice, PanelTools, dragArea, dragHandle } from '../../panel-tools'
import { SearchField } from '../../search-field'
import { SectionHeader } from '../../section-header'
import { TypeTabs } from '../../type-tabs'

const s = stylex.create({
  root: { flexGrow: 1, minHeight: 0, display: 'flex', flexDirection: 'column' },
  head: { flexShrink: 0, display: 'flex', flexDirection: 'column', gap: space.s12, paddingTop: space.s14, paddingInline: space.s14, paddingBottom: space.s12 },
  body: { flexGrow: 1, minHeight: 0, display: 'flex', flexDirection: 'column', gap: space.s12, paddingInline: space.s14, paddingBottom: space.s12 },
  title: { flexShrink: 0, display: 'flex', alignItems: 'center', justifyContent: 'space-between', gap: space.s8 },
  heading: { pointerEvents: 'none', fontSize: fontSize.heading, lineHeight: lineHeight.heading, fontWeight: fontWeight.heading },
  chevron: { display: 'grid', placeItems: 'center', width: size.controlSm, height: size.controlSm, borderWidth: border.stroke, borderStyle: 'solid', borderColor: 'transparent', borderRadius: radius.control, color: ink.textMuted, cursor: 'default', backgroundColor: { default: 'transparent', ':hover': ink.strip } },
  search: { flexShrink: 0, display: 'flex', alignItems: 'center', gap: space.s8 },
  filters: { flexShrink: 0, display: 'flex', flexDirection: 'column', gap: space.s10 },
})

export function ListView({ panel: c }: LayoutViewProps) {
  const { query, items } = c.state
  const filtersShown = !!c.state.filters || c.dragging || query.kind !== null || query.group.type !== 'all'
  const Row = c.compact ? CompactRow : ListCard
  const paste = formatChord(c.shortcuts.paste)
  const hints: Hint[] = c.compact && c.keepOpen
    ? [{ keys: paste, label: 'paste' }, { keys: formatChord(c.shortcuts.pin), label: 'pin' }, closeHint(c)]
    : [{ keys: navHint(c.layout.nav, 'prev', 'next'), label: 'move' }, { keys: paste, label: 'paste' }, { keys: 'Tab', label: 'filters' }, closeHint(c)]
  const Chevron = filtersShown ? ChevronUp : ChevronRight

  return (
    <div {...stylex.props(s.root)}>
      <div {...dragHandle(c)} {...stylex.props(s.head)}>
        <div {...dragArea} {...stylex.props(s.title)}>
          <h1 {...stylex.props(s.heading)}>{t('clipboard')}</h1>
          <button type="button" tabIndex={-1} aria-expanded={filtersShown} aria-label={t('filters')} title={t('filters')} onMouseDown={(e) => e.preventDefault()} onClick={c.toggleFilters} {...stylex.props(s.chevron)}>
            <Chevron aria-hidden {...stylex.props(icon.md)} />
          </button>
        </div>
        <div {...dragArea} {...stylex.props(s.search)}>
          <SearchField text={query.text} selected={!!c.state.textSelected} />
          <PanelTools panel={c} />
        </div>
        {filtersShown && (
          <div {...dragArea} {...stylex.props(s.filters)}>
            <TypeTabs kind={query.kind} onPick={c.pickKind} />
            <GroupMenu groups={c.groups} shown={c.shown} onLock={c.lockGroup} onRename={(group) => c.setDialog({ type: 'rename', group })} onDelete={(group) => c.setDialog({ type: 'delete', group })}>
              <GroupPills group={query.group} groups={c.groups} dragging={c.dragging} unpinned={c.unpinned} onPick={c.pickGroup} onDropItem={c.move} onClear={() => c.setDialog({ type: 'clear', count: c.unpinned })} />
            </GroupMenu>
          </div>
        )}
      </div>
      <div {...stylex.props(s.body)}>
        <PanelNotice panel={c} />
        <ItemsFrame panel={c}>
          {groupItems(items, new Date(c.now * 1000)).map((section) => [
            <SectionHeader key={section.key} section={section.key} count={section.items.length} shown={c.shown} onPinAll={pinAllIds(section).length ? () => c.pinAll(pinAllIds(section)) : undefined} />,
            ...section.items.map((item, j) => <Row key={item.id} {...cardProps(c, item, section.start + j)} />),
          ])}
        </ItemsFrame>
        <FooterHints hints={hints} />
      </div>
    </div>
  )
}
