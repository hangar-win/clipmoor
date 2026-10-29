import type { PointerEvent } from 'react'
import * as stylex from '@stylexjs/stylex'
import { space } from '@/lib/constants.stylex'
import { Notice } from '@/components/ui/notice'
import { t } from '@/lib/locale'
import type { PanelController } from './use-panel-controller'
import { MoreMenu } from './more-menu'
import { SourceFilter } from './source-filter'

const s = stylex.create({
  tools: { flexShrink: 0, display: 'flex', gap: space.s8 },
  notice: { flexShrink: 0 },
})

export const dragArea = { 'data-drag-area': '' }

export const startsDrag = (e: { button: number; target: EventTarget }) => e.button === 0 && (e.target as HTMLElement).dataset?.dragArea !== undefined

export const dragHandle = (c: PanelController) => ({
  ...dragArea,
  onPointerDown: (e: PointerEvent<HTMLElement>) => {
    if (!startsDrag(e)) return
    e.currentTarget.setPointerCapture(e.pointerId)
    c.drag('grab')
  },
  onPointerMove: (e: PointerEvent<HTMLElement>) => e.currentTarget.hasPointerCapture(e.pointerId) && c.drag('move'),
  onLostPointerCapture: () => c.drag('release'),
})

export function PanelTools({ panel: c }: { panel: PanelController }) {
  const { query } = c.state
  return (
    <div {...dragArea} {...stylex.props(s.tools)}>
      <SourceFilter source={query.source} options={c.sourceOptions} highlight={c.state.sourceMenu} onOpenChange={c.openSources} onHighlight={c.highlightSource} onPick={c.pickSource} />
      <MoreMenu panel={c} />
    </div>
  )
}

export function PanelNotice({ panel: c, style }: { panel: PanelController; style?: stylex.StyleXStyles }) {
  return c.notice ? <Notice role="alert" variant="warning" style={[s.notice, style]}>{t(c.notice)}</Notice> : null
}
