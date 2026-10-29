Keep-open lives in the panel header's ⋮ more menu (`ellipsis-vertical`, the last `control-icon` 28px ghost button, after the source filter). The menu items, in order: keep-open, Kısayollar / Shortcuts (eye, swaps the popup for the shortcut list), a separator, Ayarlar / Settings.

**States.** The keep-open item is a `menuitemcheckbox` with the 12px `pin` icon and a trailing `check` when on; toggling leaves the menu open. While keep-open is on, the ⋮ button shows `strip` fill and a `text` icon, so the state stays visible with the menu closed. When on, Esc collapses the panel to the CollapsedBar instead of closing it.

**Rules.** Its label is "Paneli açık tut / Keep panel open", never the word "pin", which names pinned items.

| | TR | EN |
|---|---|---|
| ⋮ tooltip | Diğer | More |
| Menu item | Paneli açık tut | Keep panel open |
| Footer (on) | Esc Küçült | Esc Collapse |
