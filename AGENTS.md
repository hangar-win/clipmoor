## Agent skills

### Issue tracker

Issues live as local markdown files under `.scratch/<feature>/` (no remote yet). See `docs/agents/issue-tracker.md`.

### Triage labels

Default five-role vocabulary (needs-triage, needs-info, ready-for-agent, ready-for-human, wontfix). See `docs/agents/triage-labels.md`.

### Domain docs

Single-context: root `GLOSSARY.md` + `docs/adr/`. See `docs/agents/domain.md`.

## Project exceptions to the madeui rules

- The resolved `color-scheme` lives on `<body>` (`src/styles/globals.css`, `@layer base`), not on `<html>` via `colorScheme`: a dark root paints an opaque canvas over the transparent panel corners. `<html>` keeps `color-scheme: normal`.

<!-- BEGIN:madeui-agent-rules -->

## UI components (madeui)

The components under the configured `ui` path (see madeui.json) are owned by
this project (installed via `npx @madeui/cli add <name>`, edit freely). They wrap
Base UI primitives and are styled with StyleX (compile-time CSS).

### Rules

- Tokens over literals: colors/radius/shadows come from
  `tokens.stylex.ts` (themable, `defineVars`); spacing/type/z/duration
  scales from `constants.stylex.ts` (`defineConsts`). Never hardcode
  colors, spacing, font sizes, z-indices, or durations in component styles.
- No `fontFamily` in component styles: components inherit the page's font
  (set on `<html>`, so portaled popups get it too). Kbd's `'inherit'` is
  the one exception: it undoes the reset's monospace `<kbd>`.
- Responsive styles are mobile-first: the phone layout is `default`, wider
  layouts use `breakpoint` keys from `constants.stylex.ts`
  (`[breakpoint.sm]` 640px, `md` 768, `lg` 1024, `xl` 1280, `xxl` 1536).
  Keys may overlap; the widest match wins. No literal width queries, no
  `max-width`, and a `null` condition is not a reset (name the value).
- Every component accepts `variant`, `size` (where meaningful), and a
  `style?: StyleXStyles` prop merged last via `stylex.props(...)` — caller
  styles always win. Extend by adding variants, not inline escapes.
- Base UI state (checked/open/highlighted) is styled with attribute-selector
  condition keys inside `stylex.create` — Base UI mirrors state as data
  attributes: `backgroundColor: { default: '...', '[data-checked]': ... }`.
  Conditional custom properties: use `default: null` + `var(--x, fallback)`
  where consumed (a non-null default is emitted unlayered and always wins).
- Popup edges use the `ring()` recipe (box-shadow), not borders — Base UI
  positioning math ignores borders.
- Icons are `lucide-react`, sized with the `icon` styles from
  `stylex-utils.ts` (`{...stylex.props(icon.md)}`) — never hand-drawn SVG,
  never the `size` prop.
- Dark mode lives in the tokens: every color is a `light-dark(light, dark)`
  pair. `colorScheme` from `themes.ts` goes on `<html>` (follows the OS;
  `data-theme="light"|"dark"` on `<html>` forces a mode). Never add a dark
  theme or per-component dark styles; never set `color-scheme` inline.
- Page background and text color come from `page` in `themes.ts`, on
  `<body>` (init applied both styles in the root layout or entry). Change
  them through the `background`/`foreground` tokens, not global CSS.
- Brand themes (`stylex.createTheme`) go on `<html>`, not a wrapper —
  dialogs/popovers portal to `<body>`. One theme per element: two themes
  of the same token group do not merge.
- Global CSS: keep resets inside `@layer base` (declared before `@stylex`);
  unlayered CSS overrides all StyleX rules.

### Adding a variant

1. Add the variant name to the component's type union.
2. Add a named style object in its `stylex.create` variants map using tokens.
3. Never fork a component for a one-off — pass `style` for layout-level tweaks.

### Updating components

- `npx @madeui/cli diff --json` compares every installed file with the
  registry: `same`, `differs` (with a patch toward the registry), or
  `missing`. It cannot tell a local edit from a registry update — a
  `differs` file may hold this project's own changes.
- Before running an item's `fix` command (`add <name> --overwrite`),
  read the patch: lines it removes that this project wrote are lost. When
  both sides changed, apply the registry's hunks by hand instead.

<!-- END:madeui-agent-rules -->
