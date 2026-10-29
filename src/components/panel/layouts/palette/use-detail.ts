import { useEffect, useState } from 'react'
import type { ItemView } from '@/types/services/items'
import type { PanelController } from '../../use-panel-controller'

type Detail = { id: number; text?: string; image?: string }

export function useDetail(c: PanelController, item: ItemView): Detail | null {
  const [detail, setDetail] = useState<Detail | null>(null)
  useEffect(() => {
    let live = true
    let url: string | undefined
    if (item.kind === 'image' && item.hasImage) {
      c.loadImage(item.id).then((blob) => {
        if (!live) return
        url = URL.createObjectURL(blob)
        setDetail({ id: item.id, image: url })
      }, console.error)
    } else if (item.kind && item.kind !== 'image') {
      c.loadText(item.id).then((text) => live && setDetail({ id: item.id, text }), console.error)
    }
    return () => {
      live = false
      if (url) URL.revokeObjectURL(url)
    }
  }, [item.id, item.kind, item.hasImage, item.lastUsed])
  return detail?.id === item.id ? detail : null
}
