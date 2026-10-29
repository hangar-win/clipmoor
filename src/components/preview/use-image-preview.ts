import { useEffect, useLayoutEffect, useRef, useState } from 'react'
import { getItemImageService } from '@/services/items'
import { onPreviewItem, placePreviewService } from '@/services/preview'
import { fitPreview } from '@/utils/fit-preview'

const MIN = 480

type Shown = { id: number; url: string; w: number; h: number; bytes: number }

export type ImagePreviewController = ReturnType<typeof useImagePreview>

export function useImagePreview() {
  const [shown, setShown] = useState<Shown | null>(null)
  const [stage, setStage] = useState<[number, number]>([0, 0])
  const want = useRef<number | null>(null)
  const placed = useRef<[number, number, number] | null>(null)
  const box = useRef<HTMLDivElement>(null)
  const stageEl = useRef<HTMLDivElement>(null)

  useEffect(() => {
    const ro = new ResizeObserver(([e]) => setStage([e.contentRect.width, e.contentRect.height]))
    ro.observe(stageEl.current!)
    const swap = (next: Shown | null) =>
      setShown((cur) => {
        if (cur) URL.revokeObjectURL(cur.url)
        return next
      })
    const off = onPreviewItem((id) => {
      want.current = id
      if (id === null) {
        placed.current = null
        return swap(null)
      }
      if (placed.current?.[0] === id) return void placePreviewService(...placed.current).catch(console.error)
      getItemImageService(id, false).then(async (blob) => {
        if (want.current !== id) return
        const url = URL.createObjectURL(blob)
        const img = new Image()
        img.src = url
        const ok = await img.decode().then(() => true, () => false)
        if (!ok || want.current !== id) return URL.revokeObjectURL(url)
        swap({ id, url, w: img.naturalWidth, h: img.naturalHeight, bytes: blob.size })
      }, console.error)
    })
    return () => {
      ro.disconnect()
      off.then((f) => f())
    }
  }, [])

  useLayoutEffect(() => {
    if (!shown || !box.current || !stageEl.current) return
    const chromeW = box.current.offsetWidth - stageEl.current.clientWidth
    const chromeH = box.current.offsetHeight - stageEl.current.clientHeight
    placed.current = [shown.id, Math.max(MIN, shown.w) + chromeW, Math.max(MIN, shown.h) + chromeH]
    placePreviewService(...placed.current).catch(console.error)
  }, [shown])

  const [width, height] = shown ? fitPreview(shown.w, shown.h, stage[0], stage[1]) : [0, 0]
  return { shown, width, height, box, stage: stageEl }
}
