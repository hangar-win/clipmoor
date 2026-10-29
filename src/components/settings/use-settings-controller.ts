import { useEffect, useState } from 'react'
import { t } from '@/lib/locale'
import { deleteAllService, deleteHistoryService, getRecentSourcesService } from '@/services/items'
import { getAppInfoService, getSettingsService, onLockError, onSettingsChanged, openWindowsClipboardSettingsService, saveSettingsService, setAutostartService } from '@/services/settings'
import type { AppInfo, Settings } from '@/types/services/settings'
import { loadSection, reasonSection, saveSection, type Section } from '@/utils/sections'
import { useShortcutRecording } from './use-shortcut-recording'

export type SettingsController = ReturnType<typeof useSettingsController>

const errorText = (e: unknown) => (e === 'lockCanceled' || e === 'lockKeyMissing' ? t(e) : t('saveFailed').replace('{e}', String(e)))

export function useSettingsController() {
  const [cfg, setCfg] = useState<Settings | null>(null)
  const [info, setInfo] = useState<AppInfo | null>(null)
  const [recent, setRecent] = useState<string[]>([])
  const [status, setStatus] = useState('')
  const [confirm, setConfirm] = useState(false)
  const [section, setSection] = useState<Section>(loadSection)

  const go = (to: Section) => {
    setSection(to)
    saveSection(to)
  }
  const refreshInfo = () => getAppInfoService().then(setInfo, console.error)

  useEffect(() => {
    getSettingsService().then(setCfg, console.error)
    const offSettings = onSettingsChanged(setCfg)
    getRecentSourcesService().then(setRecent, console.error)
    getAppInfoService().then((i) => {
      setInfo(i)
      const why = reasonSection(i)
      if (why) go(why)
    }, console.error)
    addEventListener('focus', refreshInfo)
    const off = onLockError((code) => {
      refreshInfo()
      if (code === 'lockKeyMissing') go('security')
    })
    return () => {
      removeEventListener('focus', refreshInfo)
      off.then((f) => f())
      offSettings.then((f) => f())
    }
  }, [])

  const update = (patch: Partial<Settings>) => {
    if (!cfg) return
    const next = { ...cfg, ...patch }
    setCfg(next)
    setStatus('')
    saveSettingsService(next).catch((e) => setStatus(errorText(e))).finally(refreshInfo)
  }
  const run = (p: Promise<unknown>) => p.then(() => setStatus(t('done')), (e) => setStatus(errorText(e))).finally(refreshInfo)
  const shortcuts = useShortcutRecording(cfg, update)
  const excluded = (name: string) => !!cfg?.excludedApps.some((a) => a.toLowerCase() === name.toLowerCase())

  return {
    cfg,
    info,
    status,
    confirm,
    section,
    shortcuts,
    suggestions: recent.filter((r) => !excluded(r)),
    go,
    update,
    setConfirm,
    exclude: (name: string) => {
      const n = name.trim()
      if (cfg && n && !excluded(n)) update({ excludedApps: [...cfg.excludedApps, n] })
    },
    include: (name: string) => cfg && update({ excludedApps: cfg.excludedApps.filter((x) => x !== name) }),
    setAutostart: (on: boolean) => setAutostartService(on).catch((e) => setStatus(errorText(e))).finally(refreshInfo),
    deleteHistory: () => run(deleteHistoryService()),
    deleteAll: () => {
      setConfirm(false)
      run(deleteAllService())
    },
    openWindowsSettings: () => openWindowsClipboardSettingsService().catch(console.error),
  }
}
