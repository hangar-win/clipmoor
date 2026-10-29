---
name: Clipmoor
description: A Win+V clipboard panel where every clip is a printer's swatch chip.
colors:
  ground: "light-dark(#F6F5F2, #1B1B1E)"
  chip-body: "light-dark(#FFFFFF, #1B1B1E)"
  text: "light-dark(#1B1B1E, #EDEDF0)"
  text-soft: "light-dark(#55555C, #B5B5BC)"
  frame: "light-dark(#86827A, #6A6A74)"
  hairline: "light-dark(#8C887F, #6A6A74)"
  rule: "light-dark(#8C887F, #6A6A74)"
  control: "light-dark(#86827A, #6A6A74)"
  control-text: "light-dark(#3A3A40, #D4D4D9)"
  label-strip: "light-dark(#ECE9E2, #2A2A30)"
  label-strip-text: "light-dark(#2A2A2E, #DADADF)"
  label-strip-soft: "light-dark(#55555C, #ABABB3)"
  hover: "light-dark(#ECE9E2, #2A2A30)"
  active: "light-dark(#1B1B1E, #34343C)"
  active-text: "light-dark(#F6F5F2, #FFFFFF)"
  active-edge: "light-dark(#1B1B1E, #EDEDF0)"
  signal-yellow: "#FFD23F"
  signal-edge: "light-dark(#1B1B1E, transparent)"
  band-text: "#2F55B4"
  band-image: "#178A6B"
  band-link: "#D9531E"
  band-code: "#7A45C9"
  band-locked: "#8A8A92"
typography:
  body:
    fontFamily: "Segoe UI Variable Text, Segoe UI Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: "19px"
  code:
    fontFamily: "Cascadia Code, Cascadia Mono, Consolas, monospace"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: "19px"
  title:
    fontFamily: "Segoe UI Variable Text, Segoe UI Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: "18px"
  control:
    fontFamily: "Segoe UI Variable Text, Segoe UI Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: "16px"
  label:
    fontFamily: "Segoe UI Variable Text, Segoe UI Variable, Segoe UI, system-ui, sans-serif"
    fontSize: "11px"
    fontWeight: 400
    letterSpacing: "0.02em"
    fontFeature: "tnum"
rounded:
  dot: "2px"
  key: "5px"
  item: "6px"
  control: "8px"
  chip: "10px"
  panel: "12px"
spacing:
  band: "10px"
  gap-tight: "4px"
  gap-chips: "8px"
  gap-filters: "10px"
  inset: "12px"
  panel: "14px"
components:
  chip:
    backgroundColor: "{colors.chip-body}"
    textColor: "{colors.text}"
    typography: "{typography.body}"
    rounded: "{rounded.chip}"
    height: "76px"
  label-strip:
    backgroundColor: "{colors.label-strip}"
    textColor: "{colors.label-strip-text}"
    typography: "{typography.label}"
    height: "24px"
    padding: "0 12px"
  type-tab:
    textColor: "{colors.text-soft}"
    typography: "{typography.title}"
    rounded: "{rounded.control}"
    padding: "4px 9px"
  type-tab-hover:
    backgroundColor: "{colors.hover}"
  type-tab-active:
    backgroundColor: "{colors.active}"
    textColor: "{colors.active-text}"
  group-pill:
    textColor: "{colors.control-text}"
    typography: "{typography.control}"
    rounded: "{rounded.control}"
    padding: "3px 9px"
  group-pill-active:
    textColor: "{colors.text}"
  search-field:
    textColor: "{colors.text}"
    typography: "{typography.body}"
    rounded: "{rounded.control}"
    height: "38px"
    padding: "0 12px"
  keep-open-toggle:
    textColor: "{colors.text-soft}"
    rounded: "{rounded.control}"
    size: "28px"
  keep-open-toggle-on:
    backgroundColor: "{colors.label-strip}"
    textColor: "{colors.text}"
  image-preview:
    backgroundColor: "{colors.ground}"
    rounded: "{rounded.chip}"
    padding: "6px"
  footer-key:
    textColor: "{colors.control-text}"
    rounded: "{rounded.key}"
    padding: "1px 5px"
---

# Design System: Clipmoor

## Overview

**Creative North Star: "Renk Kartelası" (the printer's swatch deck)**

Every clipboard item is a swatch chip from a printer's fan deck: a type-coloured band down the left edge, the content in the body, and a printed label strip underneath reading "source · size · time". The panel is a dense, fixed-module stack of these chips under a search line, type tabs and pin-group pills, closed by a shortcut footer. It reads as a Windows tool, not a toy, and refuses the grey acrylic list of identical rows that the built-in Win+V shows.

The world was set dark first (graphite ground, graphite label strips one tone up). Light is derived from it, not designed separately: the stock becomes the ground, the chip body turns white, the label strip goes one step deeper into stock, and the active tab inverts to graphite. Both modes follow the OS; a `data-theme` attribute on `<html>` forces one. The resolved `color-scheme` lives on `<body>`, not `<html>`: a dark root makes the browser paint an opaque canvas, which would fill the transparent window corners.

The panel is a non-activating overlay. Keys arrive from a Rust keyboard hook, so no panel element ever takes focus; the search field is a display with a drawn caret, not an input. The only motion is that caret blinking.

**Key Characteristics:**
- Type is read by band colour before words.
- Hierarchy by scale and weight only; no display type.
- Flat: 1px hairlines and tonal steps, no shadows.
- One selection style: a signal-yellow ring.
- Windows system fonts only; no web fonts, no network.

## Colors

A neutral graphite-and-stock pair carrying five saturated type hues and one signal yellow.

### Primary
- **Signal Yellow** (signal-yellow): the selection ring and nothing else.

### Secondary (type hues, used only as chip bands and tab keys)
- **Press Blue** (band-text): text items.
- **Proof Green** (band-image): images.
- **Vermilion** (band-link): links.
- **Plate Violet** (band-code): code.
- **Locked Grey** (band-locked): locked or untyped items.
- Colour items use their own parsed colour as the band, with a 1px inner edge (`#0000001f` light, `#ffffff29` dark) so near-ground colours stay visible. The colour tab key is a conic gradient of the spectrum.

### Neutral
- **Graphite / Stock** (ground): panel ground; graphite in dark, stock `#F6F5F2` in light.
- **Chip Body** (chip-body): equals the ground in dark (chips are drawn by hairline); white in light.
- **Label Strip** (label-strip): the printed strip; graphite one tone up (`#2A2A30`) in dark with light text, deeper stock `#ECE9E2` in light with dark text.
- **Frame, Hairline, Rule, Control**: 1px structure: window edge, chip edge, section dividers, control outlines. All of them hold 3:1 against the ground and chip body.
- **Active** (active, active-text, active-edge): the selected tab, and the primary dialog button. Graphite block with stock text in light; in dark a raised graphite block (`#34343C`) with white text and a light 1px edge, never a light fill.

### Named Rules
**The Dark Means Dark Rule.** In dark mode every surface is dark: ground, chip body, label strip, active blocks, menus, dialogs, buttons, the image preview. Light values appear only as text, icons and 1px lines. No inverted light block, ever.

**The Contrast Floor Rule.** Text holds at least 4.5:1 against the surface it sits on, in both modes and on the label strip. Outlines that identify a component (chip edge, control outline, frame, dividers) and the selection ring hold at least 3:1. Key pairs: dark text-soft on ground 8.43, label-strip-soft on strip 6.25 (dark) and 6.10 (light), control on ground 3.21 (dark) and 3.51 (light), hairline on white chip 3.53, signal yellow on dark ground 11.90. Disabled menu items are exempt.

**The One Ring Rule.** Selection is a 2px signal-yellow outline at 1px offset, never a fill, glow or second colour. On light grounds yellow alone fails 3:1, so the selected chip's own border also turns graphite (signal-edge); in dark the border goes transparent.

**The Band Speaks First Rule.** Type hues appear only as the 10px chip band and the 8px tab key. They never colour text, fills or controls.

### Custom colours
The user may replace ground, chip body, text, the selection accent and the five type hues, with a separate set for dark and light (`customColors` in `settings.json`: an `enabled` flag plus sparse `dark`/`light` maps of `#RRGGBB`; a missing key keeps the stock value). `src/utils/colors.ts` resolves both sets into one `light-dark(light, dark)` value per `ink`/`hue` token and `applyColors` writes them as inline custom properties on `<html>`, so the active scheme picks the set with no extra switch. A set whose ground, chip body and text are untouched keeps the stock structure tones verbatim; otherwise soft text, frame, hairline, rule, control, label strip, hover and active are mixed from the chosen text into the chosen ground (strip into the chip body) in oklab (`mixOklab` in `src/utils/colors.ts`, written out as plain hex so the contrast tests read the real values), and light keeps the graphite signal-edge as the chosen text. A custom colour hue turns the rainbow tab key solid. The Contrast Floor and Dark Means Dark rules become warnings for custom sets (text 4.5:1 on ground and chip body, accent and type hues 3:1 on ground, a light ground in the dark set); they never block.

### Preset palettes
Five prefilled sets in the same shape, each with a dark and a light variant (`PRESETS` in `src/utils/colors.ts`; the chosen id is `customColors.preset` in `settings.json`, absent in older files). Choosing one fills both sets and turns custom colours on; editing any colour afterwards marks it modified. Custom colours off, or Grafit untouched, render exactly the stock tokens. Every preset except Grafit holds text 4.5:1 on ground and chip body (also soft text, control text, strip text and the active block), lines 3:1 on ground and chip body, and accent and all four type bands 3:1 on ground; every dark variant keeps ground, chip, strip, hover and active dark. Unit tests assert all of it.

| Preset | Dark ground / chip / text / accent | Light ground / chip / text / accent |
|---|---|---|
| Grafit (stock, default) | `#1B1B1E` / `#1B1B1E` / `#EDEDF0` / `#FFD23F` | `#F6F5F2` / `#FFFFFF` / `#1B1B1E` / `#FFD23F` + graphite edge |
| Gri | `#202020` / `#2B2B2B` / `#F3F3F3` / `#FFD23F` | `#F3F3F3` / `#FFFFFF` / `#1C1C1C` / `#8A6100` |
| Haki gri | `#1E201A` / `#262920` / `#ECEDE4` / `#E3C75A` | `#F0EFE6` / `#FBFAF4` / `#1A1C15` / `#6E6A1E` |
| Gece | `#0E0E10` / `#151518` / `#F2F2F5` / `#FFD23F` | `#E8E8EC` / `#F7F7F9` / `#0E0E10` / `#7A5A00` |
| Kum | `#221D18` / `#2B2520` / `#F1E9DF` / `#F2B544` | `#F4EEE4` / `#FCF9F3` / `#22190F` / `#9A5B13` |

Dark variants lift the type bands so they clear 3:1 on their ground (for example Gri: text `#7C9CE8`, image `#3DB38F`, link `#F07A45`, code `#B08CF0`); light variants keep the stock-depth bands. On light grounds yellow cannot reach 3:1, so the light accents are deep ochres and olives. Grafit keeps the stock bands, including the two dark bands under 3:1.

## Typography

**Body Font:** Segoe UI Variable Text (system-ui fallback)
**Label/Mono Font:** Cascadia Code (Consolas fallback), for code and colour values

**Character:** The Windows voice itself, set tight and small; mono appears only where the content is code.

### Hierarchy
- **Body** (400, 14px, 19px): chip content (text clamped to 2 lines, links 1 line), search text, empty-state title at 600.
- **Code** (400, 13px Cascadia): code chips (first two non-blank lines, de-indented) and colour values.
- **Title** (400, 13px, 18px): type tabs; 600 when active.
- **Control** (400, 12px): pin-group pills, footer captions, empty-state hint.
- **Label** (400, 11px, 0.02em, tabular numerals): label strip. Footer keycaps also use 11px.

### Named Rules
**The Scale-Only Rule.** Hierarchy comes from size and weight steps between 11px and 14px. No display face, no headline sizes, no coloured text.

## Layout

Fixed 448x640 window, 14px inset (12px at the bottom), 12px gaps between search, filters, list and footer. The search row is the field plus two 28px ghost icon buttons (source filter, then the ⋮ more menu), 8px apart. Filters stack tabs over pills with 10px between, closed by a 1px rule with 12px below. Chips are a fixed 76px module (body + 24px label strip) stacked with 8px gaps in a thin-scrollbar list; the selected chip scrolls into view at the nearest edge. Content insets are 12px throughout. The footer wraps its seven shortcuts (14px column, 6px row gaps) onto two lines above a 1px rule.

Empty states centre a dashed ghost chip (168x44, 6px band, 12px strip) over a 14px/600 title and 12px soft hint.

## Elevation & Depth

Flat. No box shadows anywhere; depth is 1px lines and tonal steps (ground, chip body, label strip). The window is undecorated, transparent and has no OS shadow; the panel draws its own 12px rounded ground and 1px frame, and `<body>` clips everything (dialog backdrop included) to the same 12px corner. The window appears directly at its final position. The user can drag it by the header's empty space (the 14px top strip, the side gutters and the gap around the search field); the search field, header buttons, tabs, pills and chips never start a drag. The drag moves the window itself without the OS move loop, so the panel never activates and the target app keeps focus; on release the panel is clamped into the work area of the nearest monitor. Every other move while visible (snap, maximize, Win+arrow, the system menu) stays blocked, and the window cannot be resized. A dragged position lasts only while the panel stays open: the next show places it by the position mode again, while a keep-open panel stays where it was dragged.

### Named Rules
**The Hairline Rule.** Separation is a 1px line or a tone step, never a shadow.

## Shapes

A rounded scale that steps down from the window inward: 12px panel window and dialogs, 10px chips, menus and the image preview, 8px controls (search field, header buttons, tabs, pills, buttons, inputs, empty-state ghost), 6px menu items, 5px keycaps, thumbnails and swatches, 2px tab keys. The chip clips its band and strip to its radius. Icons are 12px lucide line glyphs (pin, lock) in currentColor.

## Components

### Chip (signature)
- **Structure:** 10px band column + body; body = content row + 24px label strip. 76px tall, 1px hairline border, 5px radius.
- **Content by type:** text (2-line clamp), link (1 line, ellipsis), code (2 mono lines), image (40px thumbnail), colour (40px swatch + mono value), locked (lock icon + soft text).
- **Label strip:** source app, size, then lock/pin icons and relative time pushed right in the soft strip tone. No type word or number: the band shows the type and the footer's Ctrl+1–9 hint covers quick numbers. A visually hidden "Type N" label keeps kind and position for screen readers.
- **States:** selected = One Ring. Pointer movement moves the selection; there is no separate hover style.

### Type tabs
- **Style:** text tabs with an 8px colour key; soft text; hover takes the hover tone.
- **Active:** the Active block (raised graphite with white text and a light edge in dark, graphite with stock text in light), 600 weight.

### Group pills
- **Style:** 1px control outline, 4px radius, 12px text, optional pin icon.
- **Active:** outline goes to active-edge and text to full text colour, 600 weight.

### Search field
- **Style:** 38px tall, 1px control outline, 8px radius, no fill.
- **Behaviour:** not focusable; renders typed text right-aligned against a drawn 1px x 18px caret that blinks (1.06s, hard on/off) and stops under reduced motion. Placeholder in soft text.

### More menu and keep-open
- **Style:** the last header button: a 28px square ghost icon button with the vertical ellipsis, no outline, muted icon, strip fill on hover, while its menu is open and while keep-open is on (with text-coloured icon), 8px radius. "Diğer" / "More" is its tooltip and accessible name.
- **Menu:** shared menu popup, not focusable; items in order: "Paneli açık tut" / "Keep panel open" (pushpin, checkable, shows a check when on, stays open on toggle; never the word "pin"), "Kısayollar" / "Shortcuts" (eye, swaps the popup for the shortcut list), a separator, "Ayarlar" / "Settings" (opens the Settings window; the panel closes unless kept open). Esc, an outside click or hiding the panel closes it.
- **Keep-open on:** the panel ignores outside clicks and focus changes and stays open after paste or copy; the close and open shortcuts (Esc and Win+V by default) still close it.

### Image preview
- **Trigger:** the pointer moves onto an image chip's thumbnail and rests 300ms (pointer movement drives it, so a thumbnail that appeared under a resting cursor opens on the first move). Ctrl+Space opens it at once for the selected image chip. Closes on leave, list scroll, drag, selection change and the next panel show; the Ctrl+Space preview closes on Esc (before the panel), selection change or Ctrl+Space again.
- **Style:** ground surface, 1px frame, 10px radius, 6px inset; the full image fit inside the panel width and 60% of its height, never past 2x its natural size; natural pixel size underneath in 11px soft tabular numerals. Pointer-transparent, so clicks still reach the chip.

### Footer
- **Style:** keycaps (11px, 1px control outline, 5px radius) followed by a 12px soft caption.
- **Content:** the configured shortcuts, not fixed text: paste, copy, plain paste, pin, quick number (modifier + "1–9") and preview, named as on the keycaps ("Ctrl+Shift+Enter", "Win+V"; Space is "Boşluk" in Turkish). Last comes the fixed "←/→" tab hint ("Sekme" / "Tab"): Left/Right step through the type tabs like Shift+Tab/Tab.

### Notice
- **Style:** a 12px line in a 1px control outline, 8px radius, above the list. Only for the lock key missing and unreadable settings messages; cleared when the panel opens again.

### Settings window
- A normal, focusable, resizable 900x640 window (minimum 760x520) on madeui's own tokens (`src/lib/tokens.stylex.ts`), not the panel ink. Same system fonts and `data-theme` switch as the panel.
- **Navigation:** a Windows 11 style left rail (240px, 200px under 880px wide) of five sections: Genel, Kısayollar, Görünüm, Geçmiş, Güvenlik. Each item is a 36px row with a 16px lucide icon and a 14px label, 8px radius; hover takes `navHover`, the selected item takes the `segmentHover` fill, 500 weight and a 3x16px foreground pill on its left edge. It is a vertical `tablist`: ↑/↓ (wrapping), Home and End move and select, Enter or Tab moves into the section's first control. The last section is remembered in `localStorage`. Opening for a reason picks the section: unreadable rows or a missing lock key open Güvenlik, an unreadable settings file opens Genel with its notice.
- **Pages:** every section is its own scroller, all mounted and stacked in one grid cell, the inactive ones `visibility: hidden` and `inert`, so each keeps its scroll position and a language or theme change re-renders in place. Page padding 28/32, content up to 720px, an 18px/600 page title, groups 32px apart; a group may open with a 12px/600 muted caption with no rule under it. Save errors and "Tamam" appear in a 12px status line under the pages, with a top rule only while it has text.
- **Contents:** Genel holds the Win+V toggle, start with Windows, language, then "Panel" (position, close after paste, keep open). Kısayollar holds the shortcut rows. Görünüm holds the theme, then "Renkler" (palette cards, the custom-colours switch and editor); a future layout row goes after the theme. Geçmiş holds limits and excluded apps, then "Veri" (delete history, delete all). Güvenlik shows the lock-key, unreadable-rows and Windows clipboard history notices first, then lock all, unlock duration, relock, key storage.
- Rows: label (14px) with a 12px muted hint 4px below on the left, control on the right, 1px border between rows, every row at least 64px with 12px vertical padding.
- **Contrast fixes:** `border` and `input` are `oklch(0.66)` light / `oklch(0.49)` dark, 3.1:1 against the background in both modes, so row lines, field edges, the segmented track edge and the off switch track all clear 3:1. The light focus `ring` is `oklch(0.58)` (4.3:1 on white, 3.9:1 on `muted`), and light `mutedForeground` is `oklch(0.5)` (6.0:1 on white, 5.5:1 on `muted`).
- Controls: Base UI switch (36x20), 32px inputs and buttons. Three-way choices are a Base UI radio group drawn as a segmented control: 32px `muted` track with a 1px `input` inset and 8px radius, 2px inset, content-width 28px segments with 6px radius; the selected thumb (`segmentSelected`) is white with a hairline in light, one step lighter than the track (`oklch(0.371 0 0)`) in dark, with `shadow.segment`, 14px/500 text that keeps its width; unselected text (`segmentText`) is `oklch(0.439)` / `oklch(0.75)` for AA on the track, with a `segmentHover` fill and an inset 2px ring on focus-visible. Destructive action is outlined in `destructive` and confirmed in a madeui dialog.
- **Kısayollar / Shortcuts section:** one row per configurable action (open the panel, paste, copy, plain paste, pin, preview, close, quick-number modifier). The control is a 152px-wide 32px field showing the combination, then a ghost "Varsayılana dön" / "Reset" button that dims when the row is already at its default. Click the field, it reads "Kısayola bas…" with a ring, and the next non-modifier key with its modifiers is recorded and saved at once; clicking again or leaving the field cancels. While a field records, the global hook passes every key so the window sees the combination. Arrows, Home/End, Tab, Backspace, Ctrl+A, Ctrl+V and typing are fixed and not listed.
- **Refusals and warnings:** a refused combination is not saved; a 12px `destructive` line under the label says why (Windows-reserved such as Win+L, Win+letter, Alt+Tab, Alt+F4, Ctrl+Shift+Esc; a clash with another action, named; a clash with typing or a fixed key; Win outside the open shortcut). A common shortcut (Ctrl+C, Ctrl+V, Ctrl+Z…) as the open shortcut is saved with a 12px foreground line: "Bu kısayol diğer uygulamalarda çalışmayacak".
- **Renk paleti / Color palette:** a stacked row above the custom-colours switch with five equal cards in one line. Each card is a 76px swatch drawn in the live `light-dark()` pair of that preset (so it shows the active scheme): the ground with its frame, a 30px chip in the chip colour with a type band, a text and a soft line, the accent as a 1px ring (graphite edge in light), and four type-hue keys; the name sits below. The cards are a radio group; the selected one has a 2px foreground outline around its swatch and a check after the name. A tweaked preset shows a 12px muted "değiştirildi" line, reserved on every card so the row never changes height; clicking the selected, modified card applies the preset again.
- **Özel renkler / Custom colours:** a switch under the palette; when on, a row with "Düzenlenen set" (Koyu/Açık segmented, opening on the active scheme) and a ghost "Varsayılana dön" that restores the active preset's colours for the edited set (stock for Grafit). Below, a two-column editor: per colour a 32px native colour input and a 92px mono hex field (3 or 6 digits, # optional; six digits save as you type, three on blur or Enter; an invalid value shows a 12px `destructive` line and never saves), and a sticky 248px live preview of the panel (tabs and three chips, the first ringed) drawn in the edited set's scheme. Contrast warnings are 12px amber lines with a triangle icon under the colour's label and only appear for pairs the user changed.
- **Win+V toggle:** first row of Genel (it moved out of Kısayollar). While the open shortcut is Win+V it reads "Win+V’yi ele geçir" (off = Windows keeps Win+V). With another open shortcut it reads "Win+V’yi de yakala" and starts off, so Win+V goes back to Windows; switching back to Win+V turns it on.

## Do's and Don'ts

### Do:
- **Do** derive any light value from the dark contract (stock ground, white body, deeper-stock strip, graphite inversion) rather than inventing a new light palette.
- **Do** keep every panel element unfocusable and drive it from the hook's keys. The only exception is the pin-group name and delete dialogs: they take real focus while open (mouse flow).
- **Do** use system fonts only (Segoe UI Variable, Cascadia Code).
- **Do** gate any animation behind `prefers-reduced-motion`.

### Don't:
- **Don't** use a second highlight style for selection, or yellow for anything but selection.
- **Don't** add shadows or acrylic, let the panel snap, or let anything but the header's empty space drag it.
- **Don't** put a light surface in dark mode.
- **Don't** use madeui components in the panel, except the context menus and pin-group dialogs; those are re-inked through `madeuiInk` (panel ink mapped onto madeui tokens, set on `<body>`) and get a 1px frame ring instead of a shadow.
- **Don't** colour text with type hues.
- **Don't** load web fonts or any network resource.
