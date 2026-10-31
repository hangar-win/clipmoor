Clipmoor is a clipboard manager made only for Windows 11. It replaces Win+V. It has two windows: a **panel** that opens under the cursor without stealing focus, and a separate **Settings** window. This system is the single source for both. Code reads `tokens.dtcg.json` (W3C DTCG: base palette → semantic roles → component tokens). This page reads `tokens.json`, which holds the same values in list form. Component CSS lives in `components/bundle.css`.

## Principles

1. **Dark means dark.** Dark is the primary theme. In dark, every surface is dark. Light colour appears only in text, icons and 1px lines, never as an inverted light block. Light is derived from dark: a stock `ground`, a white `card`, a deeper stock `strip`, and a graphite inversion for `active`.
2. **One ring.** Selection is a 2px `selection` outline (Signal Yellow `#FFD23F`) with a 1px offset, on the selected item only. Yellow is used nowhere else: not for focus, buttons, links or badges. In light, the selected item's border becomes `selection-edge` (graphite), because yellow on stock is 1.32:1.
3. **The band speaks first.** Type hues (`type-text`, `type-image`, `type-link`, `type-code`, `type-locked`) paint only an item's left band and the 8px tab key. They never colour text, fills or controls. Type names ("Kod · 2 dk") are always `text-muted`.
4. **Scale only.** Build hierarchy from size (11–14px; 18px only for the Palette query) and weight (400/500/600). No display face, no coloured text.
5. **Hairline and flat.** No shadow or acrylic in either window. Separate things with a 1px `frame` or `hairline`, or a tone step (`strip`, `active`). Menus and dialogs carry a `0 0 0 1px frame` ring, not elevation.
6. **Feels like Windows 11.** Use system fonts only (Segoe UI Variable, Cascadia Code), Lucide line icons, and no network resources.
7. **Keep-open is a pushpin.** Show "Paneli açık tut / Keep panel open" with the `pin` icon, as a checkable item in the header's ⋮ (`ellipsis-vertical`) menu; the ⋮ button carries the `strip` fill while it is on. The word "pin" never labels it; "Pin" is reserved for pinned items. The same menu's `eye` item opens the shortcut hints, and its `settings` item opens Settings.
8. **Never take focus.** Panel elements do not take keyboard focus. The only exception is the pin-group name and delete dialogs. Settings is a normal focusable window.
9. **Motion only on request.** Every animation runs only under `prefers-reduced-motion: no-preference`.
10. **Contrast floor, both themes.** Text must reach at least 4.5:1. Lines that identify a component (item edges, control outlines, bands, switch track, field edges) and the selection and focus rings must reach at least 3:1. Each colour token's usage note gives the measured pairs.

## Voice

The UI ships in Turkish and English. Turkish is the primary language. Keep labels short, as verbs or nouns, in sentence case. No exclamation marks, no emoji. Hints describe what happens ("Kapalıysa panel ekranın ortasında açılır."). Errors name the conflict and the way out ("“Paneli aç” ile çakışıyor").

| Concept | TR | EN | Never |
|---|---|---|---|
| Clipboard entry | Öğe | Item | kayıt, entry |
| Pinned item / section | Pin, Pinler | Pin, Pinned | favori, sabitle, sabitlenen |
| Pin group (pill) | Pin grubu | Pin group | etiket, klasör |
| Locked item | Kilitli öğe | Locked item | gizli |
| Paste | Yapıştır | Paste | |
| Paste as plain text | Düz yapıştır | Paste as plain text | Düz metin olarak |
| Copy | Kopyala | Copy | |
| Keep-open panel | Açık tutulan panel; toggle "Paneli açık tut" | Kept-open panel; "Keep panel open" | pin, sabitle |
| Collapse (Esc while kept open) | Küçült | Collapse | gizle |
| Search placeholder | Ara… | Search… | |
| Date groups | Pinler · Bugün · Dün · Bu hafta · Daha eski | Pinned · Today · Yesterday · This week · Older | |
| Relative time | şimdi · 2 dk · 1 sa · dün · 2 gün · geçen hafta | now · 2 min · 1 h · yesterday · 2 days · last week | |
| Types | Metin · Resim · Link · Renk · Kod · Kilitli | Text · Image · Link · Color · Code · Locked | |
| Settings sections | Genel · Kısayollar · Görünüm · Geçmiş · Güvenlik | General · Shortcuts · Appearance · History · Security | |

Footer hints pair a keycap with a verb: `↑↓ Gez/Move`, `↵ Yapıştır/Paste`, `Shift ↵ Düz yapıştır`, `Ctrl P Pin`, `Tab Filtre/Filter`, `Esc Kapat/Close` (or `Esc Küçült/Collapse` when the panel is kept open).

## Color

Two themes, **Dark** (primary) and **Light**. Each role is one token with a value per theme. Roles marked *customizable* are the ones a user can change (see Theming). Every other role is derived from them.

- **Surfaces.** `ground` is the window background (panel, menus, dialogs, collapsed bar, image preview). `card` is an item's body. In dark it equals `ground`, and the item is drawn by its `hairline` edge. In light it is white. `strip` is the chip label strip and every hover fill. `active` is the pressed or active block (active tab, keep-open on, primary button), with `text-on-active` and an `active-edge` border. `overlay` is the dialog backdrop.
- **Text.** Use `text` for content and titles. Use `text-muted` for tabs, hints, times, type names and section headers. Use `text-control` for pill, keycap and secondary-button labels. On the strip, use `text-strip` and `text-strip-muted`.
- **Lines.** Use `frame` for the window edge and every control outline. Use `hairline` for item edges and rules. Use `swatch-edge` for the inner edge of colour bands, swatches and thumbnails.
- **Signal.** `selection` is the ring. `selection-edge` is the selected item's border. `focus-ring` is the keyboard focus ring in both windows.
- **Type hues.** `type-text` Press Blue, `type-image` Proof Green, `type-link` Vermilion, `type-code` Plate Violet, `type-locked` Locked Grey. The Colour tab key is a conic gradient of `spectrum-red → spectrum-yellow → spectrum-green → spectrum-blue → spectrum-red`. A colour item's band is the copied colour itself, edged by `swatch-edge`. In dark, Press Blue and Plate Violet are lifted to `#3F67C8` (3.26:1) and `#834FD3` (3.29:1). They were 2.52 and 2.88, and the band is the item's only type cue.
- **Syntax.** The preview window colours a code item by language: `syntax-keyword` violet, `syntax-string` green, `syntax-comment` grey, `syntax-number` amber, `syntax-function` blue, `syntax-type` teal. This is the one place text takes a hue, and only in the code preview, never in the panel. Each reaches at least 5:1 on `ground` in both themes (usage notes give the pairs). No syntax colour is yellow, so the selection ring stays unique. Punctuation and operators stay `text`.
- **Settings.** The Settings window keeps its own neutral set: `settings-bg`, `settings-surface`, `settings-text`, `settings-text-muted`, `settings-muted`, `settings-border`, `settings-field`, the `settings-segment-*` roles, `settings-danger`, `settings-on-danger` and `settings-overlay`. Shared components (Button, Menu, Dialog, Notice) read the panel roles. Inside `.cw-settings` they are re-pointed at the settings set, so one component serves both windows. `settings-field` is the edge of every Settings control: 3.45:1 dark, 3.36:1 light (it was 1.26–1.59). `settings-border` is for row dividers only.

Merged and renamed (code migration):

| Old | New |
|---|---|
| `ink.frame`, `ink.control` | `frame` |
| `ink.hairline`, `ink.rule` | `hairline` |
| `ink.hover`, `ink.strip` | `strip` |
| `ink.ground` / `ink.chip` / `ink.soft` / `ink.controlText` / `ink.stripText` / `ink.stripSoft` / `ink.activeText` / `ink.signal` / `ink.signalEdge` | `ground` / `card` / `text-muted` / `text-control` / `text-strip` / `text-strip-muted` / `text-on-active` / `selection` / `selection-edge` |
| light `ink.stripText` `#2A2A2E` | `graphite-800` `#2A2A30` |
| madeui `muted`, `accent`, `secondary` | `settings-muted` |
| madeui `input` | `settings-field` (darkened/lightened to ≥3:1) |
| madeui `ring` (2.58:1 light) | `focus-ring` |
| madeui `card`, `primary`, `chart1–6`; hard-coded segment colours in `SettingsApp.tsx` | removed; `settings-segment-*` tokens |

## Typography

System fonts only, with no webfont and no files: `--font-sans` is Segoe UI Variable Text → Segoe UI Variable → Segoe UI → system-ui. `--font-mono` is Cascadia Code → Cascadia Mono → Consolas, and it is declared once (index.css reads the token, not a literal). Every style has an explicit line height. Nothing inherits 1.5.

| Style | Size / line | Weight | Use |
|---|---|---|---|
| `label` | 11 / 16, +0.02em, tabular nums | 400 | Label strip, preview caption, Shelf "Ctrl 1" |
| `keycap` | 11 / 14 | 400 | Keycaps (sans, never mono) |
| `control` · `control-strong` | 12 / 16 | 400 · 600 | Pills, notice, footer caption, ListCard meta · active pill, SectionHeader, ShelfCard type name |
| `title` · `title-strong` | 13 / 18 | 400 · 600 | Tabs, menu items, DetailPane meta · active tab, buttons |
| `body` | 14 / 19 | 400 | Item content, search, inputs, collapsed bar |
| `heading` | 14 / 19 | 600 | Panel title, empty title, every dialog title |
| `query` | 18 / 24 | 400 | Palette search only |
| `code` | 13 / 19 mono | 400 | Code items, colour values, DetailPane code |
| `settings-hint` · `settings-caption` | 12 / 16 | 400 · 600 | Row hints and errors · section captions (in `settings-text-muted`) |
| `settings-body` · `settings-body-strong` | 14 / 20 | 400 · 500 | Row labels, rail, segments, inputs · selected segment and rail item |
| `settings-title` | 18 / 28 | 600 | Settings page title |

Clamp item content to 2 lines (Chip), 1 line (ListCard, CompactRow, collapsed bar) or 4 lines (ShelfCard). DetailPane shows it in full.

## Spacing & layout

One spacing scale serves both windows: `space-1` `2` `3` `4` `5` `6` `8` `9` `10` `12` `14` `16` `24` `28` `32`. The off-grid steps 3, 5, 9 and 14 are deliberate and stay exact. Do not snap them to the 4px grid.

- **Panel.** Inset is `space-14` on the sides and top and `space-12` at the bottom. Sections are `space-12` apart. Search to toggle is `space-8`. Tabs sit `space-10` over the pills. The rule below the filters has `space-12` above it. Items are `space-8` apart. Item content inset is `space-12`. The list keeps `space-3` of ring room (margin −3, padding 3) so the selection ring is never clipped.
- **Settings.** Page padding is `space-28` / `space-32`. Sections are `space-28` apart. Rows are at least 57px tall with `space-12` block padding and a `space-24` gap from label to control.
- **Controls.** There is one height scale for both windows: `control-sm` 24 (pills, small buttons, count), `control-segment` 28, `control-icon` 28 (ghost icon buttons beside the panel search), `control-md` 32 (every button, Settings inputs, ShortcutField, segmented track) and `control-lg` 38 (search, the dialog input). The panel dialog button was 30 and is now 32.
- **Fixed modules.** Chip is `item-h` 76 with a `strip-h` 24 strip and a `band-w` 10 band. ListCard, CompactRow and ShelfCard use `band-slim-w` 4. CompactRow is `compact-row-h` 36. Thumbnails and swatches are `thumb-h` 40. Menus run from `menu-min-w` 184 (both windows) to `menu-max-w` 260.

## Shape

There is one radius scale for both windows: `radius-dot` 2 (tab key), `radius-key` 5 (keycap, thumbnail, swatch, count), `radius-item` 6 (menu item, segment, rail item), `radius-control` 8 (every control and the notice), `radius-card` 10 (items, **menus in both windows**, preview, DetailPane body, PaletteCard), `radius-window` 12 (every window and **dialogs in both windows**), and `radius-full` (switch only). Borders are always `stroke` 1px. Rings are `stroke-ring` 2px.

## Motion

| Motion | Duration | Curve |
|---|---|---|
| Search caret blink (on 0–55%, off 56–100%, hard step) | 1060ms | steps |
| Menu enter/exit (opacity + scale 0.97) | 150ms | ease-out `cubic-bezier(0.23, 1, 0.32, 1)` |
| Dialog and backdrop (opacity + scale 0.97) | 200ms | ease-out |
| Switch thumb, segment thumb, collapse/expand | 150ms | ease-out |
| Preview delay (Ctrl+Space opens at once) | 300ms rest | — |

There is one curve, ease-out. Switches and segments no longer fall back to the browser's `ease`. Under `prefers-reduced-motion: reduce` nothing moves or fades: the caret is steady and popups appear in place. Nothing else animates. Hover and active states change instantly.

## Focus

There is one focus ring, used everywhere a focus ring appears: `stroke-ring` 2px solid `focus-ring`, drawn **inside** the control at `offset-focus` (−2px), over its 1px border. Nothing pokes out of a control, so focus never collides with neighbours or the yellow selection ring. On filled controls (primary, danger-fill) the ring takes the fill's text colour (`text-on-active`, `settings-on-danger`) at −4px, so it stays visible on the fill. The one exception is the Switch: its 20px pill has no room inside, so its ring sits outside at `offset-focus-switch` (2px). `focus-ring` is the theme's `text`, which gives at least 12:1 on every surface it lands on (`ground`, `strip`, `settings-bg`, `settings-surface`, `settings-muted`). The panel takes no focus except in its dialogs. Keyboard moves the *selection*, which is the yellow ring. Never use yellow for focus, and never use the focus ring for selection. ShortcutField shows the same inset ring while it is recording.

## Iconography

Icons are [Lucide](https://lucide.dev) line icons at the default 2px stroke, in `currentColor`. They are 12px (`icon-sm`) inside items, strips, pills, keycaps and errors, and 16px (`icon-md`) in 38px controls, the rail, the collapsed bar and the panel header. The panel uses `search`, `ellipsis-vertical`, `eye`, `pin`, `settings`, `lock`, `clipboard`, `chevron-up/right`, `corner-down-left` (↵ in keycaps), `arrow-up-down`, `copy`, `trash-2` and `triangle-alert`. Settings uses `settings`, `keyboard`, `palette`, `history`, `shield`, `rotate-ccw`, `check` and `pencil`. Icons take the colour of the text beside them and never a type hue. Use no emoji and no Fluent glyphs. `components/bundle.js` carries the paths the previews draw.

## Layouts

The panel has four arrangements, chosen under Görünüm › Düzen, plus a collapsed state. Every layout uses the same items, ring and footer.

1. **List** (default, 448×640). Header "Pano" with a layout chevron, search plus the source filter and ⋮ menu, then **Pinler**, **Bugün**, **Dün** groups (SectionHeader) of ListCards: line 1 is the content, line 2 is the grey type name · time. Tab reveals the filter row (TypeTabs over GroupPills). The footer shows hints. → `ListLayout`
2. **Compact**. Density setting, mostly for List. Single-line CompactRows of 36px with time on the right. → `CompactLayout`
3. **Shelf** (Raf, 920×260). Search, type tabs and pin-group pills share one row. Horizontal ShelfCards carry a `Ctrl 1…9` hint. ←/→ move between cards. → `ShelfLayout`
4. **Palette** (680×420). A large `query` search with the active type filter as a pill. On the left, a ListCard list. On the right, a DetailPane (meta, full content, Yapıştır / Düz yapıştır). The footer has hints, with Esc on the right. → `PaletteLayout`
5. **Collapsed bar**. With the panel kept open, Esc collapses it to one 48px row: the last item, the total count and an expand button. → `CollapsedBar`

Also, the preview opens on a 300ms hover over an item's content or on Ctrl+Space. An image shows at no more than 60% of the window height and 2× natural size. Text, links and code size the window to their content up to `preview-text-max-w` × `preview-text-max-h`, then text wraps and the rest is clipped (the preview takes no pointer input, so it cannot scroll). A colour shows a `preview-swatch` square with its value. A locked item never opens the preview. The panel drags from the header's empty area only. The window has 12px corners, no border chrome and no snap.

Settings is about 900×640 and resizable, with a Windows 11 style left `SettingsRail`. The sections are Genel, Kısayollar, Görünüm, Geçmiş and Güvenlik. → `SettingsLayout`

## Theming

**Palettes.** Five ready-made palettes, each with a dark and a light set: **Grafit** (default, the values on this page), **Gri**, **Haki gri**, **Gece** and **Kum**. A palette is an override set of the nine customizable roles, found in `tokens.dtcg.json` › `palette.<id>.dark|light`. Gri, Haki gri, Gece and Kum are declared with `status: awaiting-values` until their colours are supplied. Do not ship them before that. Pick a palette with PaletteCard.

**Custom colours.** The user edits nine roles per mode in ColorEditor: `ground`, `card`, `text`, `selection`, and the five hues `type-text`, `type-image`, `type-link`, `type-code` and `type-locked`. Every other role is computed from those with `color-mix(in srgb, …)` (`tokens.dtcg.json` › `derive`), then clamped. If a derived text role misses 4.5:1, or a line misses 3:1, against its grounds, its share of `text` rises in 2% steps until it passes. A hue that misses 3:1 on `ground` or `card` is lifted (dark) or deepened (light) in OKLCH lightness. ColorEditor shows each role's ratio and flags a failure before the value is applied. In light mode `selection-edge` is always `text`, so any selection colour still reads on a light ground. A preset is just a pre-filled custom set with exact values for every role, so the clamp never touches it.
