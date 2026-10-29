import { useLayoutEffect, useMemo, useRef, useState, type CSSProperties } from 'react'
import * as stylex from '@stylexjs/stylex'
import { panelFrame } from '@/components/panel/panel-view'
import { colorVars } from '@/lib/apply-theme'
import { border, size, space, z } from '@/lib/constants.stylex'
import { t } from '@/lib/locale'
import { colors } from '@/lib/tokens.stylex'
import type { PanelLayout } from '@/types/panel-layout'
import type { CustomColors, Mode, Shortcuts } from '@/types/services/settings'
import { applyPreset } from '@/utils/colors'
import { samplePanel } from './sample-panel'

const MAX_H = parseFloat(size.settingsPreviewH)

const p = stylex.create({
  stage: { position: 'sticky', top: 0, zIndex: z.raised, display: 'flex', justifyContent: 'center', margin: 0, paddingBlock: space.s16, backgroundColor: colors.settingsBg, borderBottomWidth: border.stroke, borderBottomStyle: 'solid', borderBottomColor: colors.settingsBorder },
  dark: { colorScheme: 'dark' },
  light: { colorScheme: 'light' },
  box: (w: number, h: number) => ({ flexShrink: 0, width: `${w}px`, height: `${h}px`, overflow: 'clip' }),
  panel: (w: number, h: number, scale: number) => ({ width: `${w}px`, height: `${h}px`, transform: `scale(${scale})`, transformOrigin: '0 0', pointerEvents: 'none' }),
})

type Props = { layout: PanelLayout; compact: boolean; shortcuts: Shortcuts; colors: CustomColors; mode: Mode }

export function AppearancePreview({ layout, compact, shortcuts, colors: custom, mode }: Props) {
  const stage = useRef<HTMLElement>(null)
  const [room, setRoom] = useState(0)
  const panel = useMemo(() => samplePanel(layout, compact, shortcuts), [layout, compact, shortcuts])

  useLayoutEffect(() => {
    const ro = new ResizeObserver(([e]) => setRoom(e.contentRect.width))
    ro.observe(stage.current!)
    return () => ro.disconnect()
  }, [])

  const { width: w, height: h } = layout.window
  const scale = Math.min(room / w, MAX_H / h, 1)
  const box = stylex.props(p.box(w * scale, h * scale), mode === 'dark' ? p.dark : p.light)
  const vars = colorVars(custom.enabled ? custom : applyPreset(custom, 'grafit'))
  const View = layout.View

  return (
    <figure ref={stage} role="img" aria-label={t('colPreview')} {...stylex.props(p.stage)}>
      <div inert className={box.className} style={{ ...box.style, ...vars } as CSSProperties}>
        <div {...stylex.props(panelFrame, p.panel(w, h, scale))}>
          <View panel={panel} />
        </div>
      </div>
    </figure>
  )
}
