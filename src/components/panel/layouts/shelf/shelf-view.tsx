import * as stylex from '@stylexjs/stylex'
import { border, size, space } from '@/lib/constants.stylex'
import type { LayoutViewProps } from '@/types/panel-layout'
import { GroupPills } from '../../group-pills'
import { cardProps } from '../../item-parts'
import { ItemsFrame } from '../../items-frame'
import { GroupMenu } from '../../menus'
import { PanelNotice, PanelTools, dragHandle } from '../../panel-tools'
import { SearchField } from '../../search-field'
import { TypeTabs } from '../../type-tabs'
import { ShelfCard } from './shelf-card'

const s = stylex.create({
  root: { flexGrow: 1, minHeight: 0, display: 'flex', flexDirection: 'column' },
  head: { flexShrink: 0, display: 'flex', alignItems: 'center', gap: space.s12, paddingTop: space.s14, paddingInline: space.s14, paddingBottom: space.s12 },
  body: { flexGrow: 1, minHeight: 0, display: 'flex', flexDirection: 'column', gap: space.s12, paddingInline: space.s14, paddingBottom: space.s14 },
  search: { flexGrow: 0, flexShrink: 0, width: size.shelfSearchW },
  tabs: { flexShrink: 0 },
  pills: { minWidth: 0, display: 'flex', marginInlineEnd: 'auto' },
  row: { display: 'grid', gridAutoFlow: 'column', gridAutoColumns: `calc((${size.shelfW} - 2 * ${border.stroke} - 2 * ${space.s14} - 4 * ${space.s8}) / 5)`, gap: space.s8, overflowX: 'auto', overflowY: 'hidden', scrollbarWidth: 'none' },
  dense: { gridAutoColumns: `calc((${size.shelfW} - 2 * ${border.stroke} - 2 * ${space.s14} - 6 * ${space.s8}) / 7)` },
})

export function ShelfView({ panel: c }: LayoutViewProps) {
  const { query, items } = c.state
  return (
    <div {...stylex.props(s.root)}>
      <div {...dragHandle(c)} {...stylex.props(s.head)}>
        <SearchField text={query.text} selected={!!c.state.textSelected} style={s.search} />
        <TypeTabs kind={query.kind} onPick={c.pickKind} style={s.tabs} />
        <GroupMenu groups={c.groups} shown={c.shown} style={s.pills} onLock={c.lockGroup} onRename={(group) => c.setDialog({ type: 'rename', group })} onDelete={(group) => c.setDialog({ type: 'delete', group })}>
          <GroupPills inline group={query.group} groups={c.groups} dragging={c.dragging} onPick={c.pickGroup} onDropItem={c.move} />
        </GroupMenu>
        <PanelTools panel={c} />
      </div>
      <div {...stylex.props(s.body)}>
        <PanelNotice panel={c} />
        <ItemsFrame panel={c} horizontal style={[s.row, c.compact && s.dense]}>
          {items.map((item, i) => <ShelfCard key={item.id} {...cardProps(c, item, i)} />)}
        </ItemsFrame>
      </div>
    </div>
  )
}
