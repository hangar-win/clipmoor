import { useEffect, useMemo } from 'react'
import * as stylex from '@stylexjs/stylex'
import { colors } from '@/lib/tokens.stylex'
import { border, fontSize, fontWeight, letterSpacing, lineHeight, size, space } from '@/lib/constants.stylex'
import { colorFill } from '@/components/panel/item-parts'
import { highlight } from '@/utils/highlight'
import { bytes, chipLabel, kindName, urlParts } from '@/utils/label'
import type { ItemPreviewController, Shown } from './use-item-preview'
import { font, ink, radius, syntax } from '@/tokens.stylex'

const transparentPage = stylex.createTheme(colors, { settingsBg: 'transparent' })

const s = stylex.create({
  window: { clipPath: `inset(0 round ${radius.card})` },
  box: { display: 'flex', flexDirection: 'column', alignItems: 'center', gap: space.s4, height: '100vh', padding: space.s6, paddingBottom: space.s4, borderWidth: border.stroke, borderStyle: 'solid', borderColor: ink.frame, borderRadius: radius.card, backgroundColor: ink.ground, userSelect: 'none', overflow: 'hidden' },
  fit: { width: 'max-content', height: 'auto', alignItems: 'flex-start', gap: space.s6, padding: space.s12, paddingBottom: space.s6 },
  stage: { flexGrow: 1, minHeight: 0, alignSelf: 'stretch', display: 'flex', alignItems: 'center', justifyContent: 'center' },
  stageFit: { justifyContent: 'flex-start' },
  img: { display: 'block', maxWidth: 'none', borderRadius: radius.key, boxShadow: `inset 0 0 0 ${border.stroke} ${ink.swatchEdge}` },
  caption: { flexShrink: 0, fontSize: fontSize.label, lineHeight: lineHeight.label, letterSpacing: letterSpacing.label, fontVariantNumeric: 'tabular-nums', color: ink.textMuted },
  content: { maxWidth: size.previewTextMaxW, maxHeight: size.previewTextMaxH, overflow: 'hidden', fontSize: fontSize.body, lineHeight: lineHeight.body, color: ink.text, whiteSpace: 'pre-wrap', overflowWrap: 'anywhere' },
  code: { fontFamily: font.mono, fontSize: fontSize.code, lineHeight: lineHeight.code, whiteSpace: 'pre', overflowWrap: 'normal' },
  host: { fontWeight: fontWeight.titleStrong },
  color: { display: 'flex', flexDirection: 'column', gap: space.s8 },
  swatch: { width: size.previewSwatch, height: size.previewSwatch, borderRadius: radius.key, boxShadow: `inset 0 0 0 ${border.stroke} ${ink.swatchEdge}` },
})

const tone = stylex.create({
  keyword: { color: syntax.keyword },
  string: { color: syntax.string },
  comment: { color: syntax.comment },
  number: { color: syntax.number },
  function: { color: syntax.function },
  type: { color: syntax.type },
})

const place = stylex.create({
  size: (width: number, height: number) => ({ width, height }),
})

function Body({ shown: { item, text = '' }, code }: { shown: Shown; code?: ReturnType<typeof highlight> }) {
  switch (item.kind) {
    case 'code': return <div {...stylex.props(s.content, s.code)}>{code?.segments.map((g, i) => (g.tone ? <span key={i} {...stylex.props(tone[g.tone])}>{g.text}</span> : g.text))}</div>
    case 'color': return <div {...stylex.props(s.color)}><i {...stylex.props(s.swatch, colorFill(item))} /><span {...stylex.props(s.content, s.code)}>{text.trim()}</span></div>
    case 'link': {
      const [head, host, tail] = urlParts(text.trim())
      return <div {...stylex.props(s.content)}>{head}<span {...stylex.props(s.host)}>{host}</span>{tail}</div>
    }
    default: return <div {...stylex.props(s.content)}>{text}</div>
  }
}

export function ItemPreview({ preview: { shown, width, height, box, stage } }: { preview: ItemPreviewController }) {
  useEffect(() => {
    document.body.classList.add(...(stylex.props(transparentPage, s.window).className?.split(' ') ?? []))
  }, [])

  const code = useMemo(() => (shown?.item.kind === 'code' ? highlight(shown.text ?? '') : undefined), [shown])
  const fit = !!shown && !shown.image
  const caption = shown && (shown.image ? `${shown.image.w}×${shown.image.h} · ${bytes(shown.image.bytes)}` : [kindName(shown.item), code?.language, chipLabel(shown.item, shown.item.lastUsed).size].filter(Boolean).join(' · '))
  return (
    <div ref={box} aria-hidden {...stylex.props(s.box, fit && s.fit)}>
      <div ref={stage} {...stylex.props(s.stage, fit && s.stageFit)}>
        {shown?.image && <img src={shown.image.url} alt="" {...stylex.props(s.img, place.size(width, height))} />}
        {fit && <Body shown={shown} code={code} />}
      </div>
      {shown && <span {...stylex.props(s.caption)}>{caption}</span>}
    </div>
  )
}
