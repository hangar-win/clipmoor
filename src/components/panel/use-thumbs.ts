import { useEffect, useReducer, useRef } from 'react'
import { getItemImageService } from '@/services/items'
import type { ItemView } from '@/types/services/items'

const thumbKey = (i: ItemView) => `${i.id}:${i.lastUsed}`

export function useThumbs(items: ItemView[]) {
  const urls = useRef(new Map<string, string>())
  const live = useRef(new Set<string>())
  const [, bump] = useReducer((n: number) => n + 1, 0)
  useEffect(() => {
    const want = items.filter((i) => i.hasImage)
    live.current = new Set(want.map(thumbKey))
    for (const [key, url] of urls.current) {
      if (!live.current.has(key)) {
        URL.revokeObjectURL(url)
        urls.current.delete(key)
      }
    }
    for (const item of want) {
      const key = thumbKey(item)
      if (urls.current.has(key)) continue
      getItemImageService(item.id, true).then((blob) => {
        if (urls.current.has(key) || !live.current.has(key)) return
        urls.current.set(key, URL.createObjectURL(blob))
        bump()
      }, console.error)
    }
  }, [items])
  return (item: ItemView) => urls.current.get(thumbKey(item))
}
