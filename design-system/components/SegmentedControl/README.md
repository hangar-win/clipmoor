A segmented choice (Sistem/Koyu/Açık, Liste/Raf/Palet, Normal/Kompakt) in Settings.

**Anatomy.** The track is `control-md`, `settings-muted`, `radius-control`, with `space-2` inset and gap. Segments are `control-segment` (28), padded `space-12`, `radius-item`, `settings-body` in `settings-segment-text`. Selected: a `settings-segment-thumb` fill, a 1px `settings-segment-thumb-edge` ring, and `settings-body-strong` in `settings-text` (keep the width stable). Hover: `settings-segment-hover`. Focus: the 2px `focus-ring`, inside the segment.

Use `role="radiogroup"` / `role="radio"`.
