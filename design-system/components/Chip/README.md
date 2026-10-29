The classic 76px item card: a 10px type band, the content, and a 24px label strip.

**Use** in the panel's classic item list. It shows one clipboard item in full detail. For the default List layout use ListCard. For dense lists use CompactRow.

**Anatomy.** `.cw-item.cw-chip.t-<type>` is a grid of `band-w` and the body. The body is the content (`body` text, 2-line clamp; `code` for code and colour values; a 40px `cw-thumb` or `cw-swatch` before the value, `space-10` apart) over `.cw-strip` (`strip` fill, `label` style, `text-strip`). The strip's right cluster holds pin/lock icons (12px) and the time in `text-strip-muted`.

**The strip carries no type or number label.** It holds only source, size, time and the pin/lock icons. The band alone says the type.

**States.** Default: `card` body with a 1px `hairline` edge. Selected: a 2px `selection` outline at 1px offset, border `selection-edge`. There is no separate hover style. Locked: `t-locked` band, content replaced by "Kilitli öğe", lock icon on the strip.

**Consumer provides** type (`text|image|link|code|locked|colour`), content, source label, size (images), relative time, pinned/locked flags, and the colour value for colour items (`--swatch`, which also paints the band).

**Don't** colour the content or the strip by type. Don't add a hover fill. Don't drop the ring room (`.cw-list`).

| | TR | EN |
|---|---|---|
| Locked content | Kilitli öğe | Locked item |
| Time | şimdi · 2 dk · dün | now · 2 min · yesterday |
