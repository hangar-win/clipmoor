The vertical card of the Shelf layout. It shows the type name, a Ctrl-number hint, up to four lines of content, and the time.

**Anatomy.** A `band-slim-w` band and a body padded `space-12`, with three rows `space-10` apart: the head (type name in `control-strong` **`text-muted`**, and `Ctrl N` in `label` `text-muted`), the content (`body`/`code`, 4-line clamp; colour items show a 48px swatch over the value, images a 48px thumbnail), and the time in `label`. Five cards fill the 920px Shelf, `space-8` apart. Height is 180px, so the 1px window frame, `space-14` padding, the 38px header row, `space-12` and the card fill the 260px window exactly.

**States.** Selected takes the ring. ←/→ move the selection. Ctrl+1…9 paste the card directly.

**Consumer provides** type, type name, index 1–9 (omit the hint from 10 on), content, time, and the swatch or thumbnail.

**Don't** colour the type name (the screenshot did; the band carries the hue).

| | TR | EN |
|---|---|---|
| Hint | Ctrl 1 | Ctrl 1 |
