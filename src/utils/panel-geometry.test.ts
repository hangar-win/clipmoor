import { expect, test } from 'vitest'
import { fitWidth } from './panel-geometry'

test('fitted width follows the content between the minimum and the maximum', () => {
  expect(fitWidth(663, 448, 920)).toBe(663)
  expect(fitWidth(767.2, 448, 920)).toBe(768)
  expect(fitWidth(1400, 448, 920)).toBe(920)
  expect(fitWidth(120, 448, 920)).toBe(448)
})
