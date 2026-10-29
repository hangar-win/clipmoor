The kept-open panel collapsed to one 48px row: the last item, the total count, and an expand button.

**When.** Keep-open is on and the user presses Esc ("Küçült"). Click the chevron or press the panel shortcut to expand back to the same layout.

**Anatomy.** A window of `panel-w` × `collapsed-bar-h`, `ground`, `frame` ring, `radius-window`, padded `space-14` left and `space-8` right, with `space-10` gaps. From left to right: a 16px `clipboard` icon (`text-muted`), the 8px type key of the last item, the last item's content (`body`/`code`, one line), the count in a `radius-key` badge (`label`, `text-control`, `frame`), and an icon Button with `chevron-up`.

**Don't** show the selection ring here. Nothing is selected while collapsed.

| | TR | EN |
|---|---|---|
| Expand | Aç | Expand |
| Count label (a11y) | 24 öğe | 24 items |
