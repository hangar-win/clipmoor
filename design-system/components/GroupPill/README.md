The pin-group filter pill (İş, Kod parçaları, Adresler…). It is also the drop target when dragging an item into a group.

**Anatomy.** `control-sm` (24px), padded `space-3` / `space-9`, 1px `frame`, `radius-control`, `control` in `text-control`, a 12px `pin` icon `space-5` before the name, max width 160px with an ellipsis.

**States.** Active: border `active-edge`, `text`, `control-strong`. Drag in progress: dashed `text-control` border on every pill. Item over a pill: solid `text` border plus `strip` fill.

**Consumer provides** the group name and count of items, and the drop handler. Create, rename and delete happen in the Dialog (the panel's only focusable surface).

| | TR | EN |
|---|---|---|
| Concept | Pin grubu | Pin group |
| New | Yeni pin grubu | New pin group |
