The panel's search field. It never takes focus: typing goes to it while the panel stays focus-less, and it draws its own caret.

**Anatomy.** `control-lg` (38px), a 1px `frame` outline, `radius-control`, no fill, 16px `search` icon in `text-muted`, `body` text. The placeholder "Ara…" is in `text-muted`. The caret is drawn at 1×18 in `text` and blinks at 1060ms (steady under reduced motion). Selected query text sits on `active` in `text-on-active`.

**Bare variant** (`.is-bare`) for the Palette layout: no outline, 56px tall, `query` (18/24) text, a `hairline` rule below, and the active type filter shown as a pill on the right.

**Don't** add a focus ring or a fill. It is not focusable.

| TR | EN |
|---|---|
| Ara… | Search… |
