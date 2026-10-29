import { useEffect } from 'react'
import * as stylex from '@stylexjs/stylex'
import { colors } from '@/lib/tokens.stylex'
import { border, fontSize, letterSpacing, lineHeight, space } from '@/lib/constants.stylex'
import { bytes } from '@/utils/label'
import type { ImagePreviewController } from './use-image-preview'
import { ink, radius } from '@/tokens.stylex'

const transparentPage = stylex.createTheme(colors, { settingsBg: 'transparent' })

const s = stylex.create({
  window: { clipPath: `inset(0 round ${radius.card})` },
  box: { display: 'flex', flexDirection: 'column', alignItems: 'center', gap: space.s4, height: '100vh', padding: space.s6, paddingBottom: space.s4, borderWidth: border.stroke, borderStyle: 'solid', borderColor: ink.frame, borderRadius: radius.card, backgroundColor: ink.ground, userSelect: 'none', overflow: 'hidden' },
  stage: { flexGrow: 1, minHeight: 0, alignSelf: 'stretch', display: 'flex', alignItems: 'center', justifyContent: 'center' },
  img: { display: 'block', maxWidth: 'none', borderRadius: radius.key, boxShadow: `inset 0 0 0 ${border.stroke} ${ink.swatchEdge}` },
  caption: { flexShrink: 0, fontSize: fontSize.label, lineHeight: lineHeight.label, letterSpacing: letterSpacing.label, fontVariantNumeric: 'tabular-nums', color: ink.textMuted },
})

const place = stylex.create({
  size: (width: number, height: number) => ({ width, height }),
})

export function ImagePreview({ preview: { shown, width, height, box, stage } }: { preview: ImagePreviewController }) {
  useEffect(() => {
    document.body.classList.add(...(stylex.props(transparentPage, s.window).className?.split(' ') ?? []))
  }, [])

  return (
    <div ref={box} aria-hidden {...stylex.props(s.box)}>
      <div ref={stage} {...stylex.props(s.stage)}>
        {shown && <img src={shown.url} alt="" {...stylex.props(s.img, place.size(width, height))} />}
      </div>
      {shown && <span {...stylex.props(s.caption)}>{shown.w}×{shown.h} · {bytes(shown.bytes)}</span>}
    </div>
  )
}
