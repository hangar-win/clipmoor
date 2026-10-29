import { useEffect, useRef, useState, type KeyboardEvent, type ReactNode } from 'react'
import * as stylex from '@stylexjs/stylex'
import { History, Keyboard, Palette, ShieldCheck, SlidersHorizontal, type LucideIcon } from 'lucide-react'
import { icon } from '@/lib/stylex-utils'
import { Button } from '@/components/ui/button'
import { Notice } from '@/components/ui/notice'
import { Dialog, DialogContent, DialogDescription, DialogFooter, DialogTitle } from '@/components/ui/dialog'
import { colors } from '@/lib/tokens.stylex'
import { t } from '@/lib/locale'
import type { I18NKeys } from '@/locale'
import { SECTIONS, railStep, type Section } from '@/utils/sections'
import { check, formatChord, isDefaultShortcut, isWinV, shortcutText, type Action, type Verdict } from '@/utils/shortcuts'
import type { Mode, Theme } from '@/types/services/settings'
import type { SettingsController } from './use-settings-controller'
import { ink } from '@/tokens.stylex'
import { Choice, NumberField, Row, ShortcutRow, Toggle } from './controls'
import { AppearancePreview } from './appearance-preview'
import { ColorEditor, PaletteCards } from './color-editor'
import { LAYOUTS, layoutById } from '@/components/panel/layouts'
import { s } from './styles'

const settingsInk = stylex.createTheme(ink, {
  ground: colors.settingsSurface,
  card: colors.settingsBg,
  text: colors.settingsText,
  textMuted: colors.settingsTextMuted,
  textControl: colors.settingsText,
  frame: colors.settingsField,
  hairline: colors.settingsBorder,
  strip: colors.settingsMuted,
  active: colors.settingsText,
  activeEdge: colors.settingsText,
  textOnActive: colors.settingsBg,
  overlay: colors.settingsOverlay,
  focusRing: colors.focusRing,
})

const ACTIONS: [Action, I18NKeys, I18NKeys?][] = [['open', 'scOpen', 'scOpenHint'], ['paste', 'paste'], ['copy', 'copy'], ['plainPaste', 'plainPaste'], ['pin', 'pin'], ['preview', 'preview'], ['close', 'scClose'], ['source', 'sourceFilter'], ['quick', 'quickNumber', 'scQuickHint']]
const LABEL = Object.fromEntries(ACTIONS.map(([a, k]) => [a, k])) as Record<Action, I18NKeys>
const themeMode = (theme: Theme): Mode => (theme === 'system' ? (matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light') : theme)
const verdictText = (v: Verdict) => (v.error === 'scConflict' ? t('scConflict').replace('{name}', t(LABEL[v.clash!])) : t((v.error ?? v.warn)!))

const NAV: Record<Section, [I18NKeys, LucideIcon]> = {
  general: ['navGeneral', SlidersHorizontal],
  shortcuts: ['secShortcuts', Keyboard],
  look: ['secLook', Palette],
  history: ['secHistory', History],
  security: ['navSecurity', ShieldCheck],
}

export function SettingsView({ settings: c }: { settings: SettingsController }) {
  const [app, setApp] = useState('')
  const [editSet, setEditSet] = useState<Mode | null>(null)
  const picker = useRef<HTMLInputElement>(null)
  const tabs =useRef<Partial<Record<Section, HTMLButtonElement | null>>>({})

  useEffect(() => {
    document.body.classList.add(...(stylex.props(settingsInk).className?.split(' ') ?? []))
  }, [])

  const { cfg, info, section, shortcuts: rec } = c
  if (!cfg) return null

  const update = c.update
  const layout = layoutById(cfg.layout)
  const mode = editSet ?? themeMode(cfg.theme)
  const winV = isWinV(cfg.shortcuts.open)
  const openWarn = check(cfg.shortcuts, 'open', cfg.shortcuts.open).warn
  const shortcutRow = ([action, label, hint]: [Action, I18NKeys, I18NKeys?]) => {
    const message = rec.refused?.action === action ? { text: verdictText(rec.refused.verdict), error: true } : action === 'open' && openWarn ? { text: t(openWarn), error: false } : undefined
    return (
      <ShortcutRow
        key={action}
        label={label}
        hint={action === 'open' && winV ? undefined : hint}
        text={shortcutText(cfg.shortcuts, action)}
        recording={rec.recording === action}
        message={message}
        isDefault={isDefaultShortcut(cfg.shortcuts, action)}
        onStart={() => rec.start(action)}
        onStop={rec.stop}
        onKey={(chord) => rec.assign(action, chord)}
        onReset={() => rec.reset(action)}
      />
    )
  }
  const exclude = (name: string) => {
    c.exclude(name)
    setApp('')
  }
  const railKey = (e: KeyboardEvent) => {
    if (e.key === 'Enter') {
      e.preventDefault()
      const panel = document.getElementById(`panel-${section}`)
      ;(panel?.querySelector<HTMLElement>('[tabindex="0"], button:not([disabled]):not([tabindex="-1"]), input:not([tabindex="-1"])') ?? panel)?.focus()
      return
    }
    const to = railStep(section, e.key)
    if (!to) return
    e.preventDefault()
    c.go(to)
    tabs.current[to]?.focus()
  }

  const pages: Record<Section, ReactNode> = {
    general: (
      <>
        {info?.settingsReset && <Notice role="alert" variant="warning">{t('settingsReset')}</Notice>}
        <section {...stylex.props(s.section)}>
          <Toggle label={winV ? 'takeover' : 'takeoverAlso'} hint={winV ? 'takeoverHint' : 'takeoverAlsoHint'} checked={cfg.takeover} onChange={(takeover) => update({ takeover })} />
          <Toggle label="autostart" checked={info?.autostart ?? false} onChange={c.setAutostart} />
          <Choice label="language" value={cfg.language} options={[['system', 'langSystem'], ['tr', 'langTr'], ['en', 'langEn']]} onChange={(language) => update({ language })} />
        </section>
        <section {...stylex.props(s.section)}>
          <h2 {...stylex.props(s.h2)}>{t('secPanel')}</h2>
          <Choice label="panelPosition" value={cfg.panelPosition} options={[['caret', 'posCaret'], ['mouse', 'posMouse'], ['center', 'posCenter']]} onChange={(panelPosition) => update({ panelPosition })} />
          <Toggle label="closeOnPaste" hint="closeOnPasteHint" checked={cfg.closeOnPaste} onChange={(closeOnPaste) => update({ closeOnPaste })} />
          <Toggle label="keepOpen" hintText={t('keepOpenHint').replace('{close}', formatChord(cfg.shortcuts.close)).replace('{open}', formatChord(cfg.shortcuts.open))} checked={cfg.keepPanelOpen} onChange={(keepPanelOpen) => update({ keepPanelOpen })} />
        </section>
      </>
    ),
    shortcuts: <section {...stylex.props(s.section)}>{ACTIONS.map(shortcutRow)}</section>,
    look: (
      <>
        <AppearancePreview layout={layout} compact={cfg.compact} shortcuts={cfg.shortcuts} colors={cfg.customColors} mode={cfg.customColors.enabled ? mode : themeMode(cfg.theme)} />
        <section {...stylex.props(s.section)}>
          <Choice label="theme" value={cfg.theme} options={[['system', 'themeSystem'], ['light', 'themeLight'], ['dark', 'themeDark']]} onChange={(theme) => { update({ theme }); setEditSet(null) }} />
          <Choice label="layout" hint="layoutHint" value={layout.id} options={LAYOUTS.map((l) => [l.id, l.label])} onChange={(id) => update({ layout: id })} />
          <Toggle label="compactMode" hint="compactModeHint" checked={cfg.compact} onChange={(compact) => update({ compact })} />
        </section>
        <section {...stylex.props(s.section)}>
          <h2 {...stylex.props(s.h2)}>{t('secColors')}</h2>
          <div {...stylex.props(s.row, s.stack)}>
            <div {...stylex.props(s.text)}>
              <span>{t('palette')}</span>
              <span {...stylex.props(s.hint)}>{t('paletteHint')}</span>
            </div>
            <PaletteCards value={cfg.customColors} onChange={(customColors) => update({ customColors })} />
          </div>
          <Toggle label="customColors" hint="customColorsHint" checked={cfg.customColors.enabled} onChange={(enabled) => update({ customColors: { ...cfg.customColors, enabled } })} />
          {cfg.customColors.enabled && <ColorEditor value={cfg.customColors} mode={mode} onMode={setEditSet} onChange={(customColors) => update({ customColors })} />}
        </section>
      </>
    ),
    history: (
      <>
        <section {...stylex.props(s.section)}>
          <Row label="historyLimit" hint="historyLimitHint">
            <span {...stylex.props(s.numbers)}>
              <NumberField label={t('itemsUnit')} value={cfg.historyMaxItems} max={100_000} onCommit={(historyMaxItems) => historyMaxItems !== cfg.historyMaxItems && update({ historyMaxItems })} />
              {t('itemsUnit')}
              <NumberField label={t('daysUnit')} value={cfg.historyMaxDays} max={3650} onCommit={(historyMaxDays) => historyMaxDays !== cfg.historyMaxDays && update({ historyMaxDays })} />
              {t('daysUnit')}
            </span>
          </Row>
          <div {...stylex.props(s.row, s.stack)}>
            <div {...stylex.props(s.text)}>
              <span>{t('secExcluded')}</span>
              <span {...stylex.props(s.hint)}>{t('excludedHint')}</span>
            </div>
            <div {...stylex.props(s.list)}>
              {cfg.excludedApps.length === 0 && <span {...stylex.props(s.hint)}>{t('noExcluded')}</span>}
              {cfg.excludedApps.map((a) => (
                <div key={a} {...stylex.props(s.app)}>
                  <code>{a}</code>
                  <Button size="sm" onClick={() => c.include(a)}>{t('remove')}</Button>
                </div>
              ))}
            </div>
            <form onSubmit={(e) => { e.preventDefault(); exclude(app) }} {...stylex.props(s.inline)}>
              <input aria-label={t('secExcluded')} placeholder={t('excludedPlaceholder')} spellCheck={false} value={app} onChange={(e) => setApp(e.target.value)} {...stylex.props(s.input, s.focus, s.grow)} />
              <Button type="submit" disabled={!app.trim()}>{t('add')}</Button>
              <Button onClick={() => picker.current?.click()}>{t('pickApp')}</Button>
              <input ref={picker} type="file" accept=".exe" hidden onChange={(e) => { const f = e.target.files?.[0]; if (f) exclude(f.name); e.target.value = '' }} />
            </form>
            {c.suggestions.length > 0 && (
              <div {...stylex.props(s.list)}>
                <span {...stylex.props(s.hint)}>{t('recentApps')}</span>
                <div {...stylex.props(s.chips)}>
                  {c.suggestions.map((r) => (
                    <button key={r} type="button" onClick={() => exclude(r)} {...stylex.props(s.chip, s.focus)}><code>+ {r}</code></button>
                  ))}
                </div>
              </div>
            )}
          </div>
        </section>
        <section {...stylex.props(s.section)}>
          <h2 {...stylex.props(s.h2)}>{t('secData')}</h2>
          <Row label="deleteHistory" hint="deleteHistoryHint">
            <Button onClick={c.deleteHistory}>{t('deleteHistory')}</Button>
          </Row>
          <Row label="deleteAll" hint="deleteAllHint">
            <Button variant="danger" onClick={() => c.setConfirm(true)}>{t('deleteAll')}</Button>
          </Row>
        </section>
      </>
    ),
    security: (
      <>
        {(info?.lockKeyMissing || !!info?.unreadable || info?.windowsHistory) && (
          <div {...stylex.props(s.notices)}>
            {info.lockKeyMissing && <Notice role="alert" variant="warning">{t('lockKeyMissing')}</Notice>}
            {!!info.unreadable && <Notice role="alert" variant="warning">{t('unreadableItems').replace('{n}', String(info.unreadable))}</Notice>}
            {info.windowsHistory && (
              <Notice role="status" action={<Button onClick={c.openWindowsSettings}>{t('openWindowsSettings')}</Button>}>
                {t('windowsHistory')}
              </Notice>
            )}
          </div>
        )}
        <section {...stylex.props(s.section)}>
          <Toggle label="lockAll" hint={cfg.lockAllHistory && info?.lockPending ? 'lockPending' : 'lockAllHint'} checked={cfg.lockAllHistory} onChange={(lockAllHistory) => update({ lockAllHistory })} />
          <Choice label="unlockDuration" value={cfg.unlockDuration} options={[['everyTime', 'durEveryTime'], ['fiveMinutes', 'durFiveMinutes'], ['whileRunning', 'durWhileRunning']]} onChange={(unlockDuration) => update({ unlockDuration })} />
          <Toggle label="relock" checked={cfg.relockOnSessionChange} onChange={(relockOnSessionChange) => update({ relockOnSessionChange })} />
          {info && (
            <Row label="keyStorage">
              <span {...stylex.props(s.hint)}>{t(info.keyMethod === 'tpm' ? 'keyTpm' : 'keyDpapi')}</span>
            </Row>
          )}
        </section>
      </>
    ),
  }

  return (
    <div {...stylex.props(s.shell)}>
      <div role="tablist" aria-orientation="vertical" aria-label={t('settingsNav')} onKeyDown={railKey} {...stylex.props(s.rail)}>
        {SECTIONS.map((id) => {
          const [label, Icon] = NAV[id]
          const on = id === section
          return (
            <button key={id} ref={(el) => void (tabs.current[id] = el)} type="button" role="tab" id={`tab-${id}`} aria-selected={on} aria-controls={`panel-${id}`} tabIndex={on ? 0 : -1} onClick={() => c.go(id)} {...stylex.props(s.tab, s.focus, on && s.tabOn)}>
              {on && <i {...stylex.props(s.pill)} />}
              <Icon aria-hidden {...stylex.props(icon.md)} />
              {t(label)}
            </button>
          )
        })}
      </div>

      <div {...stylex.props(s.main)}>
        <div {...stylex.props(s.stage)}>
          {SECTIONS.map((id) => (
            <section key={id} role="tabpanel" id={`panel-${id}`} aria-labelledby={`tab-${id}`} tabIndex={-1} inert={id !== section} {...stylex.props(s.page, id !== section && s.away)}>
              <div {...stylex.props(s.inner)}>
                <h1 {...stylex.props(s.h1)}>{t(NAV[id][0])}</h1>
                {pages[id]}
              </div>
            </section>
          ))}
        </div>
        <div role="status" aria-live="polite" {...stylex.props(s.status)}>{c.status}</div>
      </div>

      <Dialog open={c.confirm} onOpenChange={c.setConfirm}>
        <DialogContent>
          <DialogTitle>{t('deleteAllTitle')}</DialogTitle>
          <DialogDescription>{t('deleteAllHint')}</DialogDescription>
          <DialogFooter>
            <Button autoFocus onClick={() => c.setConfirm(false)}>{t('cancel')}</Button>
            <Button variant="dangerFill" onClick={c.deleteAll}>{t('deleteAll')}</Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  )
}
