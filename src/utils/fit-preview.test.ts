import { expect, test } from 'vitest'
import { fitPreview } from './fit-preview'

test('preview fits the box, keeps aspect and never upscales past 2x', () => {
  expect(fitPreview(1280, 720, 400, 384)).toEqual([400, 225])
  expect(fitPreview(300, 1200, 400, 384)).toEqual([96, 384])
  expect(fitPreview(50, 40, 400, 384)).toEqual([100, 80])
})
