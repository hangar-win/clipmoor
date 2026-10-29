The two-line item card of the default List layout and of the Palette list: content on line 1, the grey type name and time on line 2.

**Anatomy.** A `band-slim-w` (4px) band beside a body padded `space-6` / `space-12`. Line 1 is `body` (or `code`), one line with an ellipsis. Line 2 is `control` in `text-muted`: `<type name> · <time>`, with an optional 12px lock or pin icon in front.

**States.** Default `card` with a `hairline` edge. Hover fills `strip`. Selected shows the `selection` ring and `selection-edge`, with no hover fill.

**Consumer provides** type, line-1 text, type name (TR/EN), relative time, and flags.

**Rules.** The type name on line 2 is **never** in the type hue (the screenshots' coloured names are superseded): the band already says it. Use `code` on line 1 for code and colour items.

| | TR | EN |
|---|---|---|
| Meta | Kod · 2 dk | Code · 2 min |
| Locked | Kilitli öğe · Kilitli · dün | Locked item · Locked · yesterday |
