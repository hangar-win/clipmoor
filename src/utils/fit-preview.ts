export function fitPreview(w: number, h: number, maxW: number, maxH: number): [number, number] {
  const scale = Math.min(maxW / w, maxH / h, 2)
  return [Math.round(w * scale), Math.round(h * scale)]
}
