The shortcut recorder for Settings › Kısayollar. It shows the current keys, records new ones, and reports conflicts and reserved keys.

**Anatomy.** Min width `field-shortcut-min` 152, `control-md`, 1px `settings-field`, `radius-control`, padded `space-10`. Keys show as Keycaps, `space-4` apart. A reset Button follows.

**States.**
- **Idle:** the keys.
- **Recording:** click to start. The field shows "Tuşlara basın…" in `settings-text-muted` with the 2px `focus-ring` inside the field. Esc cancels. Backspace clears.
- **Conflict:** the border turns `settings-danger`. Below it, a `settings-hint` error with a `triangle-alert` icon names the other action. Nothing is saved.
- **Reserved:** the same error style for shortcuts Windows owns (Win+L, Ctrl+Alt+Del…).
- **Warning:** a Notice under the section when taking over Win+V.
- **Reset:** "Varsayılana dön" restores the default.

| | TR | EN |
|---|---|---|
| Recording | Tuşlara basın… | Press keys… |
| Conflict | “Paneli aç” ile çakışıyor | Conflicts with “Open panel” |
| Reserved | Windows’a ayrılmış kısayol | Reserved by Windows |
| Warning | Win+V, Windows pano geçmişinin yerini alır. | Win+V replaces Windows clipboard history. |
| Reset | Varsayılana dön | Reset to default |
