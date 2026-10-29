import { PanelView } from '@/components/panel/panel-view'
import { usePanelController } from '@/components/panel/use-panel-controller'

export default function PanelPage() {
  return <PanelView panel={usePanelController()} />
}
