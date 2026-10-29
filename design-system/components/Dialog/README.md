The modal dialog for both windows (pin-group name, delete confirmations, Settings confirmations).

**Anatomy.** It sits on the `overlay` backdrop (`settings-overlay` in Settings), on `ground` / `settings-surface`, with `radius-window` (12px in both windows; Settings was 10), a `frame` ring and no shadow, padding `space-14`, and gap `space-12`. The title uses `heading`. The input is `control-lg` (38px) in both windows. Buttons are `control-md`, right-aligned, `space-8` apart, primary last.

**Focus.** The panel's only focusable surface. The input takes focus on open and the ring is the 2px `focus-ring`, inside the field. Enter confirms and Esc cancels. Opening runs 200ms ease-out, with no motion under reduce.

| | TR | EN |
|---|---|---|
| New group | Yeni pin grubu · Oluştur · Vazgeç | New pin group · Create · Cancel |
| Delete group | “İş” grubu silinsin mi? Öğeler silinmez. · Sil | Delete the “İş” group? Items are kept. · Delete |
