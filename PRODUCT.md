# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Stack

Tauri 2 (Rust) desktop shell + React + Vite + StyleX. Windows is the primary target. React + Vite chosen by the user over Next.js static export.

## Users

Windows users who copy and paste all day (developers first: SQL, links, snippets, color codes, screenshots) and are frustrated by the built-in Win+V clipboard history. Released publicly as open source.

## Product Purpose

A clipboard history for Windows that replaces Win+V. Opens instantly on Win+V, never steals focus from the app you are working in, and pastes the chosen item straight at the cursor. Success: the user forgets Win+V's built-in panel exists.

## Positioning

Built-in Win+V is slow, visually poor, and takes focus, which closes any open dropdown, context menu or popup in the target app. This product opens as a non-activating overlay: the target app keeps focus, its popups stay open, and a click pastes into it directly.

Inspired by Tendedero (macOS, MIT), which only handles screenshots and images. This product handles every clipboard type and adds categorization and named pin groups. It must use its own name and icon; Tendedero's name and icon are excluded from its license.

## Operating Context

- Invoked mid-task, many times an hour, usually while typing in another app (editor, browser, Notepad, pgAdmin, chat).
- Keyboard-first: Win+V to open, arrows + Enter to paste, Esc to dismiss. Mouse click pastes too.
- Click = paste at cursor. Shift+click = copy to clipboard only.
- Win+V is intercepted with a low-level keyboard hook; a setting turns the takeover off and restores Windows' own panel.

## Capabilities and Constraints

- Automatic categorization by type: text, image, link, color code (more types may follow).
- Pinning: pinned items never expire and can be organized into user-named pin groups.
- Retention: history is limited (last N items or X days, configurable); pins are kept forever.
- Must not steal focus or close popups in the target app.
- Must open fast; perceived latency is the main complaint against Win+V.
- Local only: no account, no network, no analytics.
- Public release: needs TR + EN localization, an installer, own name and icon.
- Undecided: product name, icon, exact retention defaults, screenshot capture (Tendedero-style) in scope or not.

## Brand Commitments

None yet. Name and icon are undecided and must not reuse Tendedero's.

## Evidence on Hand

None. No users, testimonials, benchmarks or assets exist yet; do not fabricate them.

## Product Principles

1. Never take focus from the user's work.
2. Speed over features: anything that slows opening does not ship.
3. Keyboard and mouse are equal citizens.
4. The user's data stays on the machine.
