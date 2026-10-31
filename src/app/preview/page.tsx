import { ItemPreview } from '@/components/preview/item-preview'
import { useItemPreview } from '@/components/preview/use-item-preview'

export default function PreviewPage() {
  return <ItemPreview preview={useItemPreview()} />
}
