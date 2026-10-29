import * as stylex from '@stylexjs/stylex'
import { t } from '@/lib/locale'
import { ink, radius } from '@/tokens.stylex'
import type { ComponentPropsWithoutRef } from 'react'
import { Search } from 'lucide-react'
import { icon } from '@/lib/stylex-utils'
import { border, duration, fontSize, lineHeight, size, space } from '@/lib/constants.stylex'

const blink = stylex.keyframes({
  '0%, 55%': { opacity: 1 },
  '56%, 100%': { opacity: 0 },
})

const s = stylex.create({
  field: { flexGrow: 1, minWidth: 0, display: 'flex', alignItems: 'center', height: size.controlLg, paddingInline: space.s12, borderWidth: border.stroke, borderStyle: 'solid', borderColor: ink.frame, borderRadius: radius.control, fontSize: fontSize.body, lineHeight: lineHeight.body, color: ink.text, whiteSpace: 'pre', overflow: 'hidden' },
  caret: {
    flexShrink: 0,
    width: border.stroke,
    height: 18,
    marginInline: space.s1,
    backgroundColor: ink.text,
    animationName: { default: blink, '@media (prefers-reduced-motion: reduce)': 'none' },
    animationDuration: duration.caretBlink,
    animationIterationCount: 'infinite',
  },
  icon: { marginInlineEnd: space.s8, color: ink.textMuted },
  placeholder: { color: ink.textMuted },
  text: { display: 'flex', justifyContent: 'flex-end', minWidth: 0, overflow: 'hidden' },
  selected: { backgroundColor: ink.active, color: ink.textOnActive },
  query: { height: 'auto', alignSelf: 'stretch', paddingInline: 0, borderWidth: 0, borderRadius: 0, fontSize: fontSize.query, lineHeight: lineHeight.query },
  queryCaret: { height: lineHeight.query },
})

type Props = Omit<ComponentPropsWithoutRef<'div'>, 'className' | 'style'> & { text: string; selected?: boolean; variant?: 'field' | 'query'; style?: stylex.StyleXStyles }

export function SearchField({ text, selected = false, variant = 'field', style, ...props }: Props) {
  const query = variant === 'query'
  return (
    <div role="searchbox" aria-label={t('search')} {...props} {...stylex.props(s.field, query && s.query, style)}>
      <Search aria-hidden {...stylex.props(icon.md, s.icon)} />
      {text && <span {...stylex.props(s.text, selected && s.selected)}>{text}</span>}
      <span {...stylex.props(s.caret, query && s.queryCaret)} />
      {!text && <span {...stylex.props(s.placeholder)}>{t('search')}</span>}
    </div>
  )
}

export const headerButton = stylex.create({
  toggle: {
    flexShrink: 0, display: 'grid', placeItems: 'center', width: size.controlIcon, height: size.controlIcon, borderRadius: radius.control, cursor: 'default',
    backgroundColor: { default: 'transparent', ':hover': ink.strip }, color: ink.textMuted,
  },
  on: { backgroundColor: ink.strip, color: ink.text },
})
