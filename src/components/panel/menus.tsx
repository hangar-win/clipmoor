import { useEffect, useRef, useState, type ReactNode } from 'react'
import * as stylex from '@stylexjs/stylex'
import { ContextMenu, ContextMenuContent, ContextMenuItem, ContextMenuSeparator, ContextMenuSub, ContextMenuSubContent, ContextMenuSubTrigger, ContextMenuTrigger } from '@/components/ui/context-menu'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogTitle } from '@/components/ui/dialog'
import { Button } from '@/components/ui/button'
import { border, fontSize, fontWeight, lineHeight, size, space } from '@/lib/constants.stylex'
import { t } from '@/lib/locale'
import type { ItemView } from '@/types/services/items'
import type { GroupView } from '@/types/services/groups'
import type { GroupDialog } from './use-panel-controller'
import { ink, radius } from '@/tokens.stylex'

const m = stylex.create({
  fill: { display: 'flex', flexDirection: 'column', flexGrow: 1, minHeight: 0 },
  name: { overflow: 'hidden', textOverflow: 'ellipsis', whiteSpace: 'nowrap' },
  current: { fontWeight: fontWeight.titleStrong },
  form: { display: 'flex', flexDirection: 'column', gap: space.s12 },
  input: {
    height: size.controlLg, paddingInline: space.s12, borderWidth: border.stroke, borderStyle: 'solid', borderColor: ink.frame, borderRadius: radius.control, backgroundColor: 'transparent', color: ink.text, fontSize: fontSize.body, lineHeight: lineHeight.body,
    outlineStyle: { default: 'none', ':focus-visible': 'solid' }, outlineWidth: border.strokeRing, outlineColor: ink.focusRing, outlineOffset: border.offsetFocus,
  },
})

function useTarget<T>(find: (el: HTMLElement) => T | null, shown: number) {
  const [target, setTarget] = useState<T | null>(null)
  const pending = useRef<T | null>(null)
  useEffect(() => setTarget(null), [shown])
  const capture = (e: React.MouseEvent) => {
    pending.current = find(e.target as HTMLElement)
    if (target !== null) setTarget(pending.current)
  }
  return { target, capture, change: (open: boolean) => setTarget(open ? pending.current : null) }
}

type ItemMenuProps = { items: ItemView[]; groups: GroupView[]; shown: number; children: ReactNode; onLock: (i: ItemView) => void; onPin: (i: ItemView) => void; onMove: (i: ItemView, g: number | null) => void; onNewGroup: (i: ItemView) => void; onDelete: (i: ItemView) => void }

export function ItemMenu({ items, groups, shown, children, onLock, onPin, onMove, onNewGroup, onDelete }: ItemMenuProps) {
  const { target: item, capture, change } = useTarget((el) => {
    const id = Number(el.closest('[data-item]')?.getAttribute('data-item'))
    return items.find((i) => i.id === id) ?? null
  }, shown)
  return (
    <ContextMenu open={item !== null} onOpenChange={change}>
      <ContextMenuTrigger style={m.fill} onContextMenuCapture={capture}>{children}</ContextMenuTrigger>
      <ContextMenuContent>
        {item && (
          <>
            <ContextMenuItem onClick={() => onPin(item)}>{t(item.pinned ? 'unpinItem' : 'pinItem')}</ContextMenuItem>
            <ContextMenuSub>
              <ContextMenuSubTrigger disabled={!item.kind}>{t('moveToGroup')}</ContextMenuSubTrigger>
              <ContextMenuSubContent>
                {groups.map((g) => (
                  <ContextMenuItem key={g.id} style={g.id === item.group && m.current} disabled={g.id === item.group} onClick={() => onMove(item, g.id)}>
                    <span {...stylex.props(m.name)}>{g.name}</span>
                  </ContextMenuItem>
                ))}
                {item.group !== null && <ContextMenuItem onClick={() => onMove(item, null)}>{t('removeFromGroup')}</ContextMenuItem>}
                {(groups.length > 0 || item.group !== null) && <ContextMenuSeparator />}
                <ContextMenuItem onClick={() => onNewGroup(item)}>{t('newGroup')}</ContextMenuItem>
              </ContextMenuSubContent>
            </ContextMenuSub>
            <ContextMenuSeparator />
            <ContextMenuItem disabled={item.lockedBy !== null} title={item.lockedBy === 'all' ? t('lockedByAll') : item.lockedBy === 'group' ? t('lockedByGroup') : undefined} onClick={() => onLock(item)}>{t(item.locked ? 'unlockItem' : 'lockItem')}</ContextMenuItem>
            <ContextMenuSeparator />
            <ContextMenuItem onClick={() => onDelete(item)}>{t('delete')}</ContextMenuItem>
          </>
        )}
      </ContextMenuContent>
    </ContextMenu>
  )
}

type GroupMenuProps = { groups: GroupView[]; shown: number; children: ReactNode; style?: stylex.StyleXStyles; onLock: (g: GroupView) => void; onRename: (g: GroupView) => void; onDelete: (g: GroupView) => void }

export function GroupMenu({ groups, shown, children, style, onLock, onRename, onDelete }: GroupMenuProps) {
  const { target: group, capture, change } = useTarget((el) => {
    const id = Number(el.closest('[data-group]')?.getAttribute('data-group'))
    return groups.find((g) => g.id === id) ?? null
  }, shown)
  return (
    <ContextMenu open={group !== null} onOpenChange={change}>
      <ContextMenuTrigger style={style} onContextMenuCapture={capture}>{children}</ContextMenuTrigger>
      <ContextMenuContent>
        {group && (
          <>
            <ContextMenuItem onClick={() => onRename(group)}>{t('rename')}</ContextMenuItem>
            <ContextMenuItem onClick={() => onLock(group)}>{t(group.locked ? 'unlockItem' : 'lockItem')}</ContextMenuItem>
            <ContextMenuSeparator />
            <ContextMenuItem onClick={() => onDelete(group)}>{t('delete')}</ContextMenuItem>
          </>
        )}
      </ContextMenuContent>
    </ContextMenu>
  )
}

type DialogProps = { dialog: GroupDialog | null; onClose: () => void; onCreate: (item: ItemView, name: string) => void; onRename: (g: GroupView, name: string) => void; onDelete: (g: GroupView) => void; onClear: () => void }

export function GroupDialogs({ dialog, onClose, onCreate, onRename, onDelete, onClear }: DialogProps) {
  const [name, setName] = useState('')
  useEffect(() => setName(dialog?.type === 'rename' ? dialog.group.name : ''), [dialog])
  const confirm = dialog?.type === 'delete' || dialog?.type === 'clear'
  const submit = (e: React.FormEvent) => {
    e.preventDefault()
    const n = name.trim()
    if (!dialog || (!confirm && !n)) return
    if (dialog.type === 'new') onCreate(dialog.item, n)
    else if (dialog.type === 'rename') onRename(dialog.group, n)
    else if (dialog.type === 'delete') onDelete(dialog.group)
    else onClear()
    onClose()
  }
  const title = !dialog ? '' : dialog.type === 'new' ? t('newGroupTitle') : dialog.type === 'rename' ? t('renameTitle') : dialog.type === 'delete' ? t('deleteTitle').replace('{name}', dialog.group.name) : t('clearTitle').replace('{n}', String(dialog.count))
  const hint = dialog?.type === 'new' ? t('newGroupHint') : dialog?.type === 'delete' ? t('deleteHint') : dialog?.type === 'clear' ? t('clearHint') : null
  return (
    <Dialog open={dialog !== null} onOpenChange={(open) => !open && onClose()}>
      <DialogContent>
        <DialogTitle>{title}</DialogTitle>
        {hint && <DialogDescription>{hint}</DialogDescription>}
        <form onSubmit={submit} {...stylex.props(m.form)}>
          {!confirm && <input autoFocus aria-label={t('groupName')} placeholder={t('groupName')} maxLength={60} value={name} onChange={(e) => setName(e.target.value)} {...stylex.props(m.input)} />}
          <DialogFooter>
            <Button onClick={onClose}>{t('cancel')}</Button>
            <Button type="submit" variant="primary" autoFocus={confirm}>{t(dialog?.type === 'new' ? 'create' : dialog?.type === 'rename' ? 'save' : 'delete')}</Button>
          </DialogFooter>
        </form>
      </DialogContent>
    </Dialog>
  )
}
