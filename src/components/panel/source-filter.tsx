import { useEffect, useRef } from 'react'
import * as stylex from '@stylexjs/stylex'
import { Popover } from '@base-ui/react/popover'
import { AppWindow } from 'lucide-react'
import { menuStyles } from '@/components/ui/context-menu'
import { icon, ring } from '@/lib/stylex-utils'
import { fontSize, fontWeight, lineHeight, size, space } from '@/lib/constants.stylex'
import { t } from '@/lib/locale'
import { sourceName } from '@/utils/label'
import { sameSource } from '@/utils/panel-state'
import { headerButton } from './search-field'

const s = stylex.create({
  labeled: { display: 'flex', alignItems: 'center', gap: space.s6, width: 'auto', maxWidth: size.menuMinW, paddingInline: space.s10, fontSize: fontSize.title, lineHeight: lineHeight.title },
  name: { minWidth: 0, overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' },
  current: { fontWeight: fontWeight.titleStrong },
})

type Props = { source: string | null; options: (string | null)[]; highlight: number | undefined; onOpenChange: (open: boolean) => void; onHighlight: (i: number) => void; onPick: (source: string | null) => void }

const label = (o: string | null) => (o === null ? t('allSources') : sourceName(o))

export function SourceFilter({ source, options, highlight, onOpenChange, onHighlight, onPick }: Props) {
  const list = useRef<HTMLDivElement>(null)
  const open = highlight !== undefined

  useEffect(() => {
    list.current?.querySelector('[data-highlighted]')?.scrollIntoView({ block: 'nearest' })
  }, [highlight])

  return (
    <Popover.Root open={open} onOpenChange={onOpenChange}>
      <Popover.Trigger tabIndex={-1} aria-label={t('sourceFilter')} title={t('sourceFilter')} onMouseDown={(e) => e.preventDefault()} {...stylex.props(headerButton.toggle, source !== null && [s.labeled, headerButton.on])}>
        <AppWindow {...stylex.props(icon.md)} />
        {source !== null && <span {...stylex.props(s.name)}>{label(source)}</span>}
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Positioner side="bottom" align="end" sideOffset={4} {...stylex.props(menuStyles.positioner)}>
          <Popover.Popup ref={list} initialFocus={false} finalFocus={false} role="listbox" aria-label={t('sourceFilter')} {...stylex.props(menuStyles.popup, ring())}>
            {options.map((o, i) => (
              <div key={o ?? ''} role="option" aria-selected={sameSource(o, source)} data-highlighted={i === highlight ? '' : undefined} onPointerMove={() => i !== highlight && onHighlight(i)} onClick={() => onPick(o)} {...stylex.props(menuStyles.item, sameSource(o, source) && s.current)}>
                <span {...stylex.props(s.name)}>{label(o)}</span>
              </div>
            ))}
          </Popover.Popup>
        </Popover.Positioner>
      </Popover.Portal>
    </Popover.Root>
  )
}
