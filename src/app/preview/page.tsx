import { ImagePreview } from '@/components/preview/image-preview'
import { useImagePreview } from '@/components/preview/use-image-preview'

export default function PreviewPage() {
  return <ImagePreview preview={useImagePreview()} />
}
