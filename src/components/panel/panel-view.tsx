import { useEffect, useLayoutEffect, useRef } from 'react'
import * as stylex from '@stylexjs/stylex'
import { colors } from '@/lib/tokens.stylex'
import { border } from '@/lib/constants.stylex'
import { ink, radius } from '@/tokens.stylex'
import { fitWidth } from '@/utils/panel-geometry'
import type { PanelController } from './use-panel-controller'
import { CollapsedBar } from './collapsed-bar'
import { GroupDialogs } from './menus'

const transparentPage = stylex.createTheme(colors, { settingsBg: 'transparent' })

const styles = stylex.create({
  window: { clipPath: `inset(0 round ${radius.window})` },
  fit: (min: number, max: number) => ({ width: 'max-content', minWidth: `${min}px`, maxWidth: `${max}px` }),
  frame: { display: 'flex', flexDirection: 'column', height: '100vh', borderWidth: border.stroke, borderStyle: 'solid', borderColor: ink.frame, borderRadius: radius.window, backgroundColor: ink.ground, color: ink.text, userSelect: 'none', overflow: 'hidden' },
})

export const panelFrame = styles.frame

export function PanelView({ panel: c }: { panel: PanelController }) {
  const View = c.layout.View
  const frame = useRef<HTMLDivElement>(null)
  const { fitTo } = c
  const max = c.layout.window.width
  const min = c.state.collapsed ? undefined : c.layout.window.fit?.minWidth

  useLayoutEffect(() => {
    if (min === undefined) return
    const el = frame.current!
    const ro = new ResizeObserver(() => fitTo(fitWidth(el.getBoundingClientRect().width, min, max)))
    ro.observe(el)
    return () => ro.disconnect()
  }, [min, max])

  useEffect(() => {
    document.body.classList.add(...(stylex.props(transparentPage, styles.window).className?.split(' ') ?? []))
  }, [])

  return (
    <div ref={frame} {...stylex.props(styles.frame, min !== undefined && styles.fit(min, max))}>
      {c.state.collapsed ? <CollapsedBar panel={c} /> : <View panel={c} />}
      <GroupDialogs dialog={c.dialog} onClose={() => c.setDialog(null)} onCreate={c.createGroup} onRename={c.renameGroup} onDelete={c.deleteGroup} onClear={c.clearUnpinned} />
    </div>
  )
}
