import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import { getItemImageService, getItemTextService } from '@/services/items'
import { onPreviewItem, placePreviewService } from '@/services/preview'
import type { ItemView } from '@/types/services/items'
import { fitPreview } from '@/utils/fit-preview'

const MIN = 480
const WINDOW_MIN = 640

type Image = { url: string; w: number; h: number; bytes: number }
export type Shown = { item: ItemView; text?: string; image?: Image }

export type ItemPreviewController = ReturnType<typeof useItemPreview>

export function useItemPreview() {
  const [shown, setShown] = useState<Shown | null>(null)
  const [stage, setStage] = useState<[number, number]>([0, 0])
  const want = useRef<number | null>(null)
  const placed = useRef<[number, number, number, number] | null>(null)
  const box = useRef<HTMLDivElement>(null)
  const stageEl = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const ro = new ResizeObserver(([e]) => setStage([e.contentRect.width, e.contentRect.height]))
    ro.observe(stageEl.current!)
    const swap = (next: Shown | null) =>
      setShown((cur) => {
        if (cur?.image) URL.revokeObjectURL(cur.image.url)
        return next
      })
    const off = onPreviewItem((item) => {
      want.current = item?.id ?? null
      if (!item) {
        placed.current = null
        return swap(null)
      }
      const id = item.id
      if (placed.current?.[0] === id) return void placePreviewService(...placed.current).catch(console.error)
      if (item.kind !== 'image') return void getItemTextService(id).then((text) => want.current === id && swap({ item, text: text.trimEnd() }), console.error)
      getItemImageService(id, false).then(async (blob) => {
        if (want.current !== id) return
        const url = URL.createObjectURL(blob)
        const img = new Image()
        img.src = url
        const ok = await img.decode().then(() => true, () => false)
        if (!ok || want.current !== id) return URL.revokeObjectURL(url)
        swap({ item, image: { url, w: img.naturalWidth, h: img.naturalHeight, bytes: blob.size } })
      }, console.error)
    })
    return () => {
      ro.disconnect()
      off.then((f) => f())
    }
  }, [])

  useLayoutEffect(() => {
    if (!shown || !box.current || !stageEl.current) return
    const { image, item } = shown
    if (image) {
      const chromeW = box.current.offsetWidth - stageEl.current.clientWidth
      const chromeH = box.current.offsetHeight - stageEl.current.clientHeight
      placed.current = [item.id, Math.max(MIN, image.w) + chromeW, Math.max(MIN, image.h) + chromeH, WINDOW_MIN]
    } else {
      const r = box.current.getBoundingClientRect()
      placed.current = [item.id, Math.ceil(r.width), Math.ceil(r.height), 0]
    }
    placePreviewService(...placed.current).catch(console.error)
  }, [shown])

  const [width, height] = shown?.image ? fitPreview(shown.image.w, shown.image.h, stage[0], stage[1]) : [0, 0]
  return { shown, width, height, box, stage: stageEl }
}
