import { useEffect, useState } from 'react'
import { setRecordingService } from '@/services/panel'
import type { Chord, Settings } from '@/types/services/settings'
import { DEFAULT_SHORTCUTS, check, shortcutChord, withShortcut, type Action, type Verdict } from '@/utils/shortcuts'

export function useShortcutRecording(cfg: Settings | null, update: (patch: Partial<Settings>) => void) {
  const [recording, setRec] = useState<Action | null>(null)
  const [refused, setRefused] = useState<{ action: Action; verdict: Verdict } | null>(null)

  useEffect(() => {
    if (!recording) return
    setRecordingService(true).catch(console.error)
    return () => void setRecordingService(false).catch(console.error)
  }, [recording])

  const assign = (action: Action, c: Chord) => {
    if (!cfg) return
    setRec(null)
    const verdict = check(cfg.shortcuts, action, c)
    if (verdict.error) return setRefused({ action, verdict })
    setRefused(null)
    update(withShortcut(cfg, action, c))
  }

  return {
    recording,
    refused,
    start: (action: Action) => {
      setRefused(null)
      setRec(action)
    },
    stop: () => setRec(null),
    assign,
    reset: (action: Action) => assign(action, shortcutChord(DEFAULT_SHORTCUTS, action)),
  }
}
