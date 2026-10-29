The single button for both windows: primary, secondary, icon, small/ghost, and danger.

**Anatomy.** `control-md` (32px; the panel dialog's 30 is retired), padded `space-12`, `radius-control`, `title-strong`. **Primary**: `active` fill, `active-edge` border, `text-on-active`. **Secondary**: no fill, `frame` border, `text-control`, `strip` on hover. **Icon**: 32px square, no border, 16px icon. **Small** (`is-sm`, `control-sm`): no border, `text-muted`, used for "Varsayılan" reset; `is-idle` at `opacity-idle` when already at the default. **Danger** (Settings): `settings-danger` border and text; **danger-fill** for the confirming button inside the Dialog.

**Focus** (Settings, panel dialogs): the 2px `focus-ring` drawn inside the button (`offset-focus`, −2px). On primary and danger-fill it switches to the fill's text colour (`text-on-active` / `settings-on-danger`) at −4px, so it stays visible on the fill.

**Rules.** One primary per view. Destructive actions always confirm in a Dialog.

| TR | EN |
|---|---|
| Yapıştır · Düz yapıştır · Kaydet · Vazgeç · Varsayılan · Geçmişi temizle | Paste · Paste as plain text · Save · Cancel · Default · Clear history |
