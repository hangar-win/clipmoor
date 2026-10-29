import type { LocaleType } from '@/locale'

export type Mods = { ctrl: boolean; shift: boolean; alt: boolean; win: boolean }
export type Chord = Mods & { key: number }
export type Shortcuts = { open: Chord; paste: Chord; copy: Chord; plainPaste: Chord; pin: Chord; preview: Chord; close: Chord; source: Chord; quick: Mods }

export type ColorKey = 'ground' | 'card' | 'text' | 'selection' | 'typeText' | 'typeImage' | 'typeLink' | 'typeCode' | 'typeLocked'
export type Mode = 'dark' | 'light'
export type ColorSet = Partial<Record<ColorKey, string>>
export type CustomColors = { enabled: boolean; dark: ColorSet; light: ColorSet; preset?: string | null }

export type Theme = 'system' | 'light' | 'dark'
export type UnlockDuration = 'everyTime' | 'fiveMinutes' | 'whileRunning'
export type PanelPosition = 'caret' | 'mouse' | 'center'
export type Settings = { historyMaxItems: number; historyMaxDays: number; takeover: boolean; language: 'system' | LocaleType; closeOnPaste: boolean; unlockDuration: UnlockDuration; relockOnSessionChange: boolean; lockAllHistory: boolean; panelPosition: PanelPosition; theme: Theme; excludedApps: string[]; keepPanelOpen: boolean; shortcuts: Shortcuts; customColors: CustomColors; layout: string; compact: boolean }
export type AppInfo = { keyMethod: 'tpm' | 'dpapi'; windowsHistory: boolean; autostart: boolean; lockKeyMissing: boolean; lockPending: boolean; settingsReset: boolean; unreadable: number }
