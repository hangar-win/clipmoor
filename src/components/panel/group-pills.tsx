import { useState } from 'react'
import * as stylex from '@stylexjs/stylex'
import { t } from '@/lib/locale'
import type { GroupFilter } from '@/types/services/items'
import type { GroupView } from '@/types/services/groups'
import { ink, radius } from '@/tokens.stylex'
import { ListX, Lock, Pin } from 'lucide-react'
import { icon } from '@/lib/stylex-utils'
import { border, fontSize, fontWeight, lineHeight, opacity, size, space } from '@/lib/constants.stylex'

export const DRAG_TYPE = 'application/x-clipmoor-item'

const s = stylex.create({
  row: { display: 'flex', flexWrap: 'wrap', gap: space.s6, paddingBottom: space.s12, borderBottomWidth: border.stroke, borderBottomStyle: 'solid', borderBottomColor: ink.hairline },
  inline: { flexWrap: 'nowrap', paddingBottom: 0, borderBottomWidth: 0, overflowX: 'auto', scrollbarWidth: 'none' },
  fixed: { flexShrink: 0 },
  pill: { display: 'flex', alignItems: 'center', gap: space.s5, maxWidth: 160, height: size.controlSm, paddingBlock: space.s3, paddingInline: space.s9, borderWidth: border.stroke, borderStyle: 'solid', borderColor: ink.frame, borderRadius: radius.control, fontSize: fontSize.control, lineHeight: lineHeight.control, color: ink.textControl, cursor: 'default', backgroundColor: { default: null, ':hover': ink.strip } },
  name: { overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' },
  on: { borderColor: ink.activeEdge, color: ink.text, fontWeight: fontWeight.controlStrong },
  target: { borderStyle: 'dashed', borderColor: ink.textControl },
  over: { borderStyle: 'solid', borderColor: ink.text, color: ink.text, backgroundColor: ink.strip },
  clear: { marginInlineStart: 'auto', borderColor: 'transparent', color: { default: ink.textMuted, ':hover': { default: ink.text, ':disabled': ink.textMuted } }, backgroundColor: { default: 'transparent', ':hover': { default: ink.strip, ':disabled': 'transparent' } }, opacity: { default: null, ':disabled': opacity.disabled }, outline: 'none' },
})

type Props = { group: GroupFilter; groups: GroupView[]; dragging: boolean; unpinned: number; inline?: boolean; onPick: (g: GroupFilter) => void; onDropItem: (item: number, group: number) => void; onClear: () => void }

const FIXED: GroupFilter[] = [{ type: 'all' }, { type: 'pinned' }]
const same = (a: GroupFilter, b: GroupFilter) => a.type === b.type && (a.type !== 'group' || (b.type === 'group' && a.id === b.id))

export function GroupPills({ group, groups, dragging, unpinned, inline = false, onPick, onDropItem, onClear }: Props) {
  const [over, setOver] = useState<number | null>(null)
  return (
    <div role="tablist" {...stylex.props(s.row, inline && s.inline)}>
      {FIXED.map((g) => (
        <div key={g.type} role="tab" aria-selected={same(g, group)} onClick={() => onPick(g)} {...stylex.props(s.pill, s.fixed, same(g, group) && s.on)}>
          {g.type === 'pinned' && <Pin {...stylex.props(icon.sm)} />}
          {t(g.type === 'all' ? 'all' : 'pinned')}
        </div>
      ))}
      {groups.map((g) => {
        const f: GroupFilter = { type: 'group', id: g.id }
        return (
          <div
            key={g.id}
            role="tab"
            data-group={g.id}
            title={g.name}
            aria-selected={same(f, group)}
            onClick={() => onPick(f)}
            onDragOver={(e) => {
              if (!e.dataTransfer.types.includes(DRAG_TYPE)) return
              e.preventDefault()
              e.dataTransfer.dropEffect = 'move'
              setOver(g.id)
            }}
            onDragLeave={() => setOver(null)}
            onDrop={(e) => {
              e.preventDefault()
              setOver(null)
              const id = Number(e.dataTransfer.getData(DRAG_TYPE))
              if (id) onDropItem(id, g.id)
            }}
            {...stylex.props(s.pill, inline && s.fixed, same(f, group) && s.on, dragging && s.target, over === g.id && s.over)}
          >
            {g.locked && <Lock {...stylex.props(icon.sm)} />}
            <span {...stylex.props(s.name)}>{g.name}</span>
          </div>
        )
      })}
      {!inline && (
        <button type="button" disabled={!unpinned} onClick={onClear} {...stylex.props(s.pill, s.clear)}>
          <ListX {...stylex.props(icon.sm)} />
          {t('clearUnpinned')}
        </button>
      )}
    </div>
  )
}
