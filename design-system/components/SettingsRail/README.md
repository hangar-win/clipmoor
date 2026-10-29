The Windows 11 style navigation rail of the Settings window.

**Anatomy.** `rail-w` 280, `settings-bg`, padded `space-16` / `space-8`. The app row has the 16px clipboard icon and "Clipmoor" in `heading`. Items are 36px tall, padded `space-12`, `radius-item`, with a 16px icon and `settings-body`. Hover and selected use a `settings-muted` fill. Selected also gets `settings-body-strong` and a 3×16 `settings-text` indicator on the left edge. It is not yellow and not a hue.

**A11y.** `aria-current="page"` on the selected item. The focus ring is the 2px `focus-ring`, inside the item.

| TR | EN |
|---|---|
| Genel · Kısayollar · Görünüm · Geçmiş · Güvenlik | General · Shortcuts · Appearance · History · Security |
