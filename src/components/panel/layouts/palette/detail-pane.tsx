import * as stylex from '@stylexjs/stylex'
import { Button } from '@/components/ui/button'
import { t } from '@/lib/locale'
import { border, fontSize, lineHeight, space } from '@/lib/constants.stylex'
import type { ItemView } from '@/types/services/items'
import { itemMeta } from '@/utils/label'
import { font, ink, radius } from '@/tokens.stylex'
import type { PanelController } from '../../use-panel-controller'
import { KindKey } from '../../item-parts'
import { useDetail } from './use-detail'

const s = stylex.create({
  pane: { display: 'grid', gridTemplateRows: 'auto minmax(0, 1fr) auto', gap: space.s12, padding: space.s16, minWidth: 0, minHeight: 0 },
  meta: { display: 'flex', alignItems: 'center', gap: space.s6, minWidth: 0, fontSize: fontSize.title, lineHeight: lineHeight.title, color: ink.textMuted, whiteSpace: 'nowrap', overflow: 'hidden', textOverflow: 'ellipsis' },
  body: { alignSelf: 'start', maxHeight: '100%', overflow: 'auto', padding: space.s12, borderWidth: border.stroke, borderStyle: 'solid', borderColor: ink.hairline, borderRadius: radius.card, backgroundColor: ink.card, fontSize: fontSize.body, lineHeight: lineHeight.body, whiteSpace: 'pre-wrap', overflowWrap: 'anywhere', scrollbarWidth: 'thin', scrollbarColor: `${ink.frame} transparent` },
  code: { fontFamily: font.mono, fontSize: fontSize.code, lineHeight: lineHeight.code },
  muted: { color: ink.textMuted },
  media: { minHeight: 0, display: 'flex', alignItems: 'flex-start' },
  image: { maxWidth: '100%', maxHeight: '100%', objectFit: 'contain', borderRadius: radius.card, boxShadow: `inset 0 0 0 ${border.stroke} ${ink.swatchEdge}` },
  actions: { display: 'flex', gap: space.s8 },
})

const passive = { tabIndex: -1, onMouseDown: (e: React.MouseEvent) => e.preventDefault() }

export function DetailPane({ panel: c, item }: { panel: PanelController; item: ItemView }) {
  const detail = useDetail(c, item)
  const image = detail?.image ?? c.thumbs(item)
  return (
    <div {...stylex.props(s.pane)}>
      <div {...stylex.props(s.meta)}>
        <KindKey item={item} />
        {itemMeta(item, c.now, true)}
      </div>
      {item.kind === 'image' ? (
        <div {...stylex.props(s.media)}>{image && <img src={image} alt="" draggable={false} {...stylex.props(s.image)} />}</div>
      ) : (
        <div {...stylex.props(s.body, (item.kind === 'code' || item.kind === 'color') && s.code, !item.kind && s.muted)}>{item.kind ? (detail?.text ?? item.preview) : t('lockedItem')}</div>
      )}
      <div {...stylex.props(s.actions)}>
        <Button variant="primary" {...passive} onClick={() => c.activateWith(item, 'paste')}>{t('paste')}</Button>
        <Button {...passive} onClick={() => c.activateWith(item, 'plainPaste')}>{t('plainPaste')}</Button>
      </div>
    </div>
  )
}
