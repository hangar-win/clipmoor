import { SettingsView } from '@/components/settings/settings-view'
import { useSettingsController } from '@/components/settings/use-settings-controller'

export default function SettingsPage() {
  return <SettingsView settings={useSettingsController()} />
}
