import type { PointerEvent, ReactNode } from 'react'
import * as stylex from '@stylexjs/stylex'
import { Lock, Pin } from 'lucide-react'
import { t } from '@/lib/locale'
import { icon } from '@/lib/stylex-utils'
import { border, fontSize, letterSpacing, lineHeight, size, space } from '@/lib/constants.stylex'
import type { ItemView } from '@/types/services/items'
import { bytes, codeLines, oneLine } from '@/utils/label'
import { quickHint } from '@/utils/shortcuts'
import { font, hue, ink, radius } from '@/tokens.stylex'
import type { PanelController } from './use-panel-controller'
import { DRAG_TYPE } from './group-pills'

const s = stylex.create({
  shell: { display: 'grid', gridTemplateColumns: `${size.bandSlimW} minmax(0, 1fr)`, flexShrink: 0, scrollMarginBlock: space.s4, scrollMarginInline: space.s4, borderWidth: border.stroke, borderStyle: 'solid', borderColor: ink.hairline, borderRadius: radius.card, overflow: 'hidden', backgroundColor: ink.card, color: ink.text, cursor: 'default' },
  selected: { borderColor: ink.selectionEdge, outlineWidth: border.strokeRing, outlineStyle: 'solid', outlineColor: ink.selection, outlineOffset: border.offsetSelection },
  line: { display: 'block', minWidth: 0, fontSize: fontSize.body, lineHeight: lineHeight.body, whiteSpace: 'pre', overflow: 'hidden', textOverflow: 'ellipsis' },
  code: { fontFamily: font.mono, fontSize: fontSize.code, lineHeight: lineHeight.code },
  thumb: { display: 'inline-block', verticalAlign: 'top', height: lineHeight.body, width: 'auto', maxWidth: size.thumbH, marginInlineEnd: space.s6, borderRadius: radius.dot, objectFit: 'cover' },
  swatch: { display: 'inline-block', verticalAlign: 'middle', width: size.iconSm, height: size.iconSm, marginInlineEnd: space.s6, borderRadius: radius.dot, boxShadow: `inset 0 0 0 ${border.stroke} ${ink.swatchEdge}` },
  key: { flexShrink: 0, width: size.tabKey, height: size.tabKey, borderRadius: radius.dot },
  flags: { display: 'inline-flex', alignItems: 'center', gap: space.s5 },
  quick: { flexShrink: 0, fontSize: fontSize.label, lineHeight: lineHeight.label, letterSpacing: letterSpacing.label, fontVariantNumeric: 'tabular-nums', color: ink.textMuted, whiteSpace: 'nowrap' },
})

const band = stylex.create({
  text: { backgroundColor: hue.text },
  image: { backgroundColor: hue.image },
  link: { backgroundColor: hue.link },
  code: { backgroundColor: hue.code },
  none: { backgroundColor: hue.locked },
  edge: { boxShadow: `inset calc(-1 * ${border.stroke}) 0 0 ${ink.swatchEdge}` },
})

const own = stylex.create({
  color: (c: string) => ({ backgroundColor: c }),
})

export const colorFill = (item: ItemView) => own.color(item.preview?.trim() ?? '')
export const bandStyle = (item: ItemView) => (item.kind === 'color' ? [colorFill(item), band.edge] : band[item.kind ?? 'none'])

export const itemStyles = s

export function KindKey({ item }: { item: ItemView }) {
  return <i aria-hidden {...stylex.props(s.key, bandStyle(item))} />
}

export function Flags({ item }: { item: ItemView }) {
  if (!item.locked && !item.pinned) return null
  return (
    <span {...stylex.props(s.flags)}>
      {item.locked && <Lock aria-hidden {...stylex.props(icon.sm)} />}
      {item.pinned && <Pin aria-hidden {...stylex.props(icon.sm)} />}
    </span>
  )
}

export function QuickHint({ text }: { text?: string }) {
  return text ? <span {...stylex.props(s.quick)}>{text}</span> : null
}

type TextProps = { item: ItemView; thumb?: string; onPreview?: (el: HTMLElement | null) => void }

export const previewHover = (onPreview?: (el: HTMLElement | null) => void) => ({
  onPointerMove: (e: PointerEvent<HTMLElement>) => (e.movementX || e.movementY) && onPreview?.(e.currentTarget),
  onPointerLeave: () => onPreview?.(null),
})

export function ItemText({ item, thumb, onPreview }: TextProps) {
  const hover = previewHover(onPreview)
  switch (item.kind) {
    case null: return <span {...stylex.props(s.line)}>{t('lockedItem')}</span>
    case 'image':
      return (
        <span {...stylex.props(s.line)}>
          {thumb && <img src={thumb} alt="" draggable={false} {...hover} {...stylex.props(s.thumb)} />}
          {item.size === null ? t('image') : bytes(item.size)}
        </span>
      )
    case 'color': return <span {...hover} {...stylex.props(s.line, s.code)}><i {...stylex.props(s.swatch, colorFill(item))} />{item.preview?.trim()}</span>
    case 'code': return <span {...hover} {...stylex.props(s.line, s.code)}>{codeLines(item.preview ?? '', 1)[0]}</span>
    default: return <span {...hover} {...stylex.props(s.line)}>{oneLine(item.preview)}</span>
  }
}

export type CardProps = { item: ItemView; selected: boolean; thumb?: string; now: number; quick?: string; onActivate: () => void; onHover: () => void; onPreview: (el: HTMLElement | null) => void }

export const cardProps = (c: PanelController, item: ItemView, i: number): CardProps => ({
  item,
  selected: i === c.state.selected,
  thumb: c.thumbs(item),
  now: c.now,
  quick: i < 9 ? quickHint(c.shortcuts.quick, i + 1) : undefined,
  onActivate: () => c.activate(item),
  onHover: () => c.hover(i),
  onPreview: (el) => c.hoverPreview(el ? item.id : null),
})

type ShellProps = Pick<CardProps, 'item' | 'selected' | 'onActivate' | 'onHover'> & { style?: stylex.StyleXStyles; children: ReactNode }

export function ItemShell({ item, selected, onActivate, onHover, style, children }: ShellProps) {
  return (
    <li
      role="option"
      aria-selected={selected}
      data-selected={selected || undefined}
      data-item={item.id}
      draggable={!!item.kind}
      onDragStart={(e) => {
        e.dataTransfer.setData(DRAG_TYPE, String(item.id))
        e.dataTransfer.effectAllowed = 'move'
      }}
      onClick={onActivate}
      onPointerMove={(e) => (e.movementX || e.movementY) && onHover()}
      {...stylex.props(s.shell, selected && s.selected, style)}
    >
      <i {...stylex.props(bandStyle(item))} />
      {children}
    </li>
  )
}
