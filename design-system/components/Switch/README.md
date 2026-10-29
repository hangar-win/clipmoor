The on/off switch in Settings.

**Anatomy.** 36×20, `radius-full`, a 16px thumb. Off: track `settings-field` (3.45:1 dark, 3.36:1 light), thumb `settings-bg`. On: track `settings-text`, thumb `settings-bg`. The track's lightness, not hue, tells the states apart. The thumb slides 150ms ease-out and jumps under reduce. Focus: the 2px `focus-ring` **outside** the track (`offset-focus-switch`, 2px). This is the one exception to the inset rule, because the 20px pill has no room inside and the ring would merge with the on-state track.

Use `role="switch"` with `aria-checked`. The row label is the switch's name.
