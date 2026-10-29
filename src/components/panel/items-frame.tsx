import { useEffect, useRef, type ReactNode } from 'react'
import * as stylex from '@stylexjs/stylex'
import { border, fontSize, fontWeight, lineHeight, space } from '@/lib/constants.stylex'
import { t } from '@/lib/locale'
import { formatChord } from '@/utils/shortcuts'
import { ink, radius } from '@/tokens.stylex'
import type { PanelController } from './use-panel-controller'
import { ItemMenu } from './menus'

const s = stylex.create({
  list: { flexGrow: 1, minHeight: 0, display: 'flex', flexDirection: 'column', gap: space.s8, margin: `calc(-1 * ${space.s3})`, padding: space.s3, overflowY: 'auto', scrollbarWidth: 'thin', scrollbarColor: `${ink.frame} transparent` },
  empty: { flexGrow: 1, display: 'flex', flexDirection: 'column', alignItems: 'center', justifyContent: 'center', gap: space.s4, paddingBottom: space.s24, textAlign: 'center' },
  ghost: { width: 168, height: 44, marginBottom: space.s14, display: 'grid', gridTemplateColumns: `${space.s6} 1fr`, gridTemplateRows: `1fr ${space.s12}`, borderWidth: border.stroke, borderStyle: 'dashed', borderColor: ink.frame, borderRadius: radius.control, overflow: 'hidden' },
  ghostBand: { gridRow: '1 / span 2', backgroundColor: ink.hairline },
  ghostStrip: { gridColumn: 2, gridRow: 2, borderTopWidth: border.stroke, borderTopStyle: 'dashed', borderTopColor: ink.frame },
  emptyTitle: { fontSize: fontSize.heading, fontWeight: fontWeight.heading, lineHeight: lineHeight.heading },
  emptyHint: { fontSize: fontSize.control, lineHeight: lineHeight.control, color: ink.textMuted, maxWidth: 260 },
})

function EmptyState({ panel: c }: { panel: PanelController }) {
  const [title, hint] = c.empty
  return (
    <div {...stylex.props(s.empty)}>
      <div aria-hidden {...stylex.props(s.ghost)}>
        <i {...stylex.props(s.ghostBand)} />
        <i {...stylex.props(s.ghostStrip)} />
      </div>
      <div {...stylex.props(s.emptyTitle)}>{t(title)}</div>
      <div {...stylex.props(s.emptyHint)}>{t(hint).replace('{key}', formatChord(c.shortcuts.pin))}</div>
    </div>
  )
}

type Props = { panel: PanelController; horizontal?: boolean; style?: stylex.StyleXStyles; children: ReactNode }

export function ItemsFrame({ panel: c, horizontal = false, style, children }: Props) {
  const list = useRef<HTMLUListElement>(null)
  const { items, selected } = c.state

  useEffect(() => {
    list.current?.querySelector('[data-selected]')?.scrollIntoView({ block: 'nearest', inline: 'nearest' })
  }, [selected, items])

  return (
    <ItemMenu items={items} groups={c.groups} shown={c.shown} onLock={c.lockItem} onPin={c.pin} onMove={(i, g) => c.move(i.id, g)} onNewGroup={(item) => c.setDialog({ type: 'new', item })} onDelete={c.deleteItem}>
      {items.length ? (
        <ul
          ref={list}
          role="listbox"
          aria-orientation={horizontal ? 'horizontal' : 'vertical'}
          onScroll={() => c.hoverPreview(null)}
          onWheel={horizontal ? (e) => void (e.currentTarget.scrollLeft += e.deltaY) : undefined}
          onDragStart={c.startDrag}
          onDragEnd={c.endDrag}
          {...stylex.props(s.list, style)}
        >
          {children}
        </ul>
      ) : (
        <EmptyState panel={c} />
      )}
    </ItemMenu>
  )
}
