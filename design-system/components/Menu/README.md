The context menu, the same in both windows.

**Anatomy.** `ground` (`settings-surface` in Settings), `radius-card` (10px in both windows; Settings was 8), a `0 0 0 1px frame` ring and no shadow, `space-4` block padding, width `menu-min-w` 184 to `menu-max-w` 260. Items are `title`, inset `space-4`, padded `space-5` / `space-10`, `radius-item`, with an optional 12px icon and a trailing keycap. Separator: 1px `hairline`.

**States.** Highlighted: `strip`. Disabled: `text-muted` at `opacity-disabled`. Enter/exit runs 150ms ease-out (opacity + scale 0.97), with no motion under reduce.

**Item menu (TR / EN).** Yapıştır / Paste · Düz yapıştır / Paste as plain text · Kopyala / Copy · Pin grubuna ekle / Add to pin group · Kilitle / Lock · Sil / Delete.
