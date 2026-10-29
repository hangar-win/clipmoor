The right pane of the Palette layout. It shows the selected item's meta, its full content, and the paste actions.

**Anatomy.** Three rows `space-12` apart, padded `space-16`. Meta is the tab key (8px, type hue) plus `title` in `text-muted`: `Kod · 2 dk · Windows Terminal`. The body is a `card` box with a `hairline` edge, `radius-card`, padding `space-12`, `code` or `body`, wrapping, scrolling when long. Images show the image instead. Actions are a primary **Yapıştır** and a secondary **Düz yapıştır**, both `control-md`.

**Consumer provides** the item, the source app name, and handlers.

| | TR | EN |
|---|---|---|
| Primary | Yapıştır | Paste |
| Secondary | Düz yapıştır | Paste as plain text |
