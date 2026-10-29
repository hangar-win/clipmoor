import { getCurrentWindow } from '@tauri-apps/api/window'

export const getWindowLabelService = () => getCurrentWindow().label
