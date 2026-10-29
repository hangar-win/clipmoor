import { useEffect, useRef } from 'react'
import { previewItemService } from '@/services/preview'

export function usePreview(keyPreview: boolean, selectedId: number | undefined) {
  const timer = useRef(0)
  const previewing = useRef<number | null>(null)
  const keyed = useRef(false)

  const show = (id: number | null, delay = 0) => {
    if (id === previewing.current) return
    clearTimeout(timer.current)
    previewing.current = id
    if (id === null) previewItemService(null).catch(console.error)
    else timer.current = window.setTimeout(() => previewItemService(id).catch(console.error), delay)
  }

  useEffect(() => {
    if (keyPreview) show(selectedId ?? null)
    else if (keyed.current || (previewing.current !== null && previewing.current !== selectedId)) show(null)
    keyed.current = keyPreview
  }, [keyPreview, selectedId])

  useEffect(() => () => show(null), [])

  return show
}
