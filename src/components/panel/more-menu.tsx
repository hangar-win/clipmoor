import * as stylex from '@stylexjs/stylex'
import { Popover } from '@base-ui/react/popover'
import { Check, EllipsisVertical, Eye, Pin, Settings } from 'lucide-react'
import { menuStyles } from '@/components/ui/context-menu'
import { icon, ring } from '@/lib/stylex-utils'
import { t } from '@/lib/locale'
import { ink } from '@/tokens.stylex'
import type { PanelController } from './use-panel-controller'
import { headerButton } from './search-field'
import { ShortcutHints } from './shortcut-hints'

const s = stylex.create({
  item: { backgroundColor: { default: 'transparent', ':hover': ink.strip } },
})

const item = [menuStyles.item, s.item]

export function MoreMenu({ panel: c }: { panel: PanelController }) {
  const { menu } = c
  const hints = menu === 'hints'
  return (
    <Popover.Root open={menu !== null} onOpenChange={(open) => c.setMenu(open ? 'more' : null)}>
      <Popover.Trigger tabIndex={-1} aria-label={t('more')} title={t('more')} aria-haspopup="menu" onMouseDown={(e) => e.preventDefault()} {...stylex.props(headerButton.toggle, (menu !== null || c.keepOpen) && headerButton.on)}>
        <EllipsisVertical {...stylex.props(icon.md)} />
      </Popover.Trigger>
      <Popover.Portal>
        <Popover.Positioner side="bottom" align="end" sideOffset={4} {...stylex.props(menuStyles.positioner)}>
          <Popover.Popup initialFocus={false} finalFocus={false} role={hints ? undefined : 'menu'} aria-label={t(hints ? 'secShortcuts' : 'more')} {...stylex.props(menuStyles.popup, ring())}>
            {hints ? (
              <ShortcutHints shortcuts={c.shortcuts} nav={c.layout.nav} />
            ) : (
              <>
                <div role="menuitemcheckbox" aria-checked={c.keepOpen} onClick={c.toggleKeepOpen} {...stylex.props(item)}>
                  <Pin {...stylex.props(icon.sm)} />
                  {t('keepOpen')}
                  {c.keepOpen && <Check {...stylex.props(icon.sm, menuStyles.chevron)} />}
                </div>
                <div role="menuitem" onClick={() => c.setMenu('hints')} {...stylex.props(item)}>
                  <Eye {...stylex.props(icon.sm)} />
                  {t('secShortcuts')}
                </div>
                <div role="separator" {...stylex.props(menuStyles.separator)} />
                <div role="menuitem" onClick={c.openSettings} {...stylex.props(item)}>
                  <Settings {...stylex.props(icon.sm)} />
                  {t('settingsTitle')}
                </div>
              </>
            )}
          </Popover.Popup>
        </Popover.Positioner>
      </Popover.Portal>
    </Popover.Root>
  )
}
