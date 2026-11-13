---
status: draft
date: 2026-11-13
---

# Undo and Redo say when there is nothing to give back

## Why now

`Session::can_undo` and `Session::can_redo` are already part of `monospace-editing`'s contract
(#197), and #189 settled how the chrome draws focused and unfocused controls and how the drop-down
behaves. Until now, `menu::MENU` has been a `static StaticMenu`, so `Edit > Undo` and `Edit > Redo`
look active at every position of the history. A user who opens `Edit` at the start or end of the
history learns that there is nothing to give back only by taking the entry and watching the diagram
stay still.

Since #181's D2 and issue #231 were written, #292 removed `Delete` from the `Edit` menu and made it
strictly contextual on the bottom command row (`docs/application-model.md` §2 and §3). `Edit` now
contains only `Undo` and `Redo`. This slice therefore gives those two entries their disabled state
from the session and establishes the rule for any future menu entry whose precondition can be known
before it is taken.

## Scope

### In

- Replacing `static MENU: StaticMenu` in `crates/monospace/src/menu.rs` with a per-frame
  `MenuStructure` built from `MenuFlags { can_undo: bool, can_redo: bool }`, disabling `Edit > Undo`
  when `!can_undo` and `Edit > Redo` when `!can_redo`.
- Lifting `#[cfg(test)]` from `DiagramState::can_undo` and `DiagramState::can_redo` in
  `crates/monospace/src/diagram_state.rs` so `frame::draw` reads them each frame and passes
  `MenuFlags` to `menu::draw_bar` and `menu::draw_drop_down`.
- Drawing disabled menu entries with `Modifier::DIM` over the menu's base style (`MENU_BASE`) across
  the entire entry row — label text, navigation character, and shortcut text — with no
  `Modifier::REVERSED` and without `Style::DISABLED`'s light-gray background or `KEY_BINDING`'s
  orange shortcut background.
- Enforcing disabled-entry selection and activation invariants in `menu`: opening a drop-down lands
  initial selection on the first enabled entry (or `None` when all entries are disabled), and
  `Enter`, left mouse click, `Home`, `End`, and a disabled entry's navigation character in the open
  drop-down never activate any entry and keep the drop-down open (`Reply::Consumed`).
- Keeping global shortcuts (`Ctrl+Z` and `Ctrl+Shift+Z` in `shortcuts::SHORTCUTS`) and
  `callbacks::apply` unchanged.
- Updating `docs/application-model.md` (§3, §4, §7), updating
  `frame::tests::the_edit_menu_offers_undo_and_redo_at_every_position`, and adding style and
  interaction tests in `crates/monospace/src/menu.rs` and `crates/monospace/src/frame.rs`.

### Out

- **Re-adding `Delete` to the `Edit` menu.** `Delete` stays strictly contextual on the bottom
  command row and in `context_table.rs` per #292 D3.
- **Disabling `File` or `Help` entries.** `File > New`, `Open`, `Save`, `Export as Text`,
  `Open Demo`, `Quit`, and `Help > About` stay always enabled; dirty/unsaved-state tracking is
  tracked separately in #380.
- **Changing `Ctrl+Z` or `Ctrl+Shift+Z` behavior at history ends.** Shortcut dispatch remains
  `Reach::Unclaimed` and returns `Control::Changed` with an unchanged session.
- **Unifying the shortcut label drawn in the menu with `shortcuts::SHORTCUTS`.** Tracked in #180.
- **Modifying upstream `rat-menu` or `rat-theme4` crates.** All styling and event guards live in
  `crates/monospace/src/menu.rs`; `Cargo.toml` and `Cargo.lock` do not change.

## The decision

**D1 — which entries carry a disabled state, and where does `Delete` stand?**

**Answer:** Only `Edit > Undo` (`!can_undo`) and `Edit > Redo` (`!can_redo`) carry a disabled state
in this slice; `Delete` remains out of `Edit` and strictly contextual in the bottom command row
(#292 D3), and every `File` and `Help` entry stays enabled. The rule established here is the
entry's, not the command's: a menu entry whose precondition is statically answerable from the
session is disabled when that precondition is false, while global shortcuts and `callbacks::apply`
keep their existing behavior. **Why not** restoring `Delete` to `Edit` as #181 D2 and issue #231's
older prose described: #292 deliberately moved `Delete` to the contextual command row so global
menus hold only session-wide actions and contextual commands are not duplicated across two surfaces.
**Why not** rejecting `Action::Undo` or `Action::Redo` in `callbacks::apply` or disabling the
keyboard shortcuts: `docs/editing-model.md` §6 already makes `undo`/`redo` at history ends a no-op,
and disabling at the menu entry gives visual feedback before pressing without breaking existing
shortcut contracts. **Answered by** the maintainer, in #231.

**D2 — how does the menu receive session state each frame?**

**Answer:** `menu` replaces `static MENU: StaticMenu` with a per-frame `Menu` value implementing
`rat_widget::menu::MenuStructure<'_>` constructed from
`MenuFlags { can_undo: bool, can_redo: bool }` (defaulting to both `false`), passing
`.disabled(!flags.can_undo)` on `Undo` and `.disabled(!flags.can_redo)` on `Redo`. `frame::draw`
reads `state.diagram().can_undo()` and `state.diagram().can_redo()` each frame and passes them to
`menu::draw_bar` and `menu::draw_drop_down`. **Why not** storing `can_undo` and `can_redo` inside
`Chrome`: `Chrome` owns only the menu bar and the keyboard leaf (`crates/monospace/src/chrome.rs`),
and copying session predicates into `Chrome` creates a second state that can fall out of sync
between commands and frames. **Why not** hiding disabled entries or changing their labels (for
example, omitting `Redo`): hiding entries shifts menu geometry under the pointer and violates #181
D2 and `docs/tui-conventions.md` §7.4. **Answered by** the maintainer, in #231.

**D3 — how is a disabled menu entry drawn?**

**Answer:** A disabled entry is drawn with `Modifier::DIM` over the menu's base style (`MENU_BASE`)
across the entire entry row — label text, navigation character, and right-hand shortcut — with no
`Modifier::REVERSED`, no underlined highlight color mismatch, and no `KEY_BINDING` background on the
shortcut. Because `rat-theme4`'s `BASE16` dark theme maps `Style::DISABLED` to a high-contrast
light-gray background (`#aaaaaa`) with black foreground (`#000000`), and `rat-menu 3.1.1`'s
`PopupMenu::render_items` applies `disabled_style` only to plain label spans while leaving the
navigation character in `highlight_style` and the shortcut in `right_style`, `menu::widgets` sets
`style.popup_disabled = Some(Style::new().dim())` and `menu::draw_drop_down` normalizes every cell
of each disabled item's `item_areas[i]` to `base_style.add_modifier(Modifier::DIM)` (preserving each
cell's character symbol and dropping `REVERSED` and shortcut/highlight color overrides). **Why not**
using `rat-theme4`'s `Style::DISABLED`: a solid `#aaaaaa` block with `#000000` text makes a disabled
entry visually louder than enabled entries and contradicts `docs/tui-conventions.md` §7.4 ("disabled
is marked with a dimmed attribute"). **Why not** setting `popup_disabled` alone without normalizing
the row: `rat-menu` would still draw `Ctrl+Z` / `Ctrl+Shift+Z` on an orange `KEY_BINDING` background
and `_U`/`_R` undimmed, making a disabled entry look half-active. **Answered by** the maintainer in
[#231](https://github.com/andresmoschini/monospace/issues/231).

**D4 — where does keyboard selection land when a drop-down opens?**

**Answer:** When a drop-down opens (or switches from another top-level label), initial selection
lands on the first enabled entry, or `None` when every entry in that drop-down is disabled. In
`Edit`: on a fresh session (`!can_undo && !can_redo`), neither entry is selected (`None`); when
`can_undo && !can_redo`, `Undo` (index `0`) is selected; when `!can_undo && can_redo`, `Redo` (index
`1`) is selected. Because `rat-menu 3.1.1`'s `render_popup_menu` forces `state.selected = Some(0)`
whenever `state.selected.is_none()` even if entry `0` is disabled, `menu::draw_drop_down` normalizes
selection before and after rendering so a disabled entry never holds `selected` and never paints in
`Modifier::REVERSED`. **Why not** leaving `selected` as `None` whenever entry `0` is disabled even
if `Redo` is enabled: opening `Edit` when only `Redo` is possible should let `Enter` act on the
single available entry without requiring an extra `Down` press first. **Why not** accepting
`rat-menu`'s `Some(0)` default: `Alt+E` followed by `Enter` on an empty session would target a
disabled `Undo`, and item `0` would render in reverse video while disabled. **Answered by** the
maintainer, in #231.

**D5 — how do keyboard and mouse events behave on disabled entries?**

**Answer:** Disabled entries cannot be selected or activated by keyboard or mouse; attempting to
take or navigate onto one never emits an `Action`, never activates another entry, and keeps the
drop-down open (`Reply::Consumed`). Specifically, `menu::handle` guards against `rat-menu 3.1.1`
defects where `Enter`, left click, `Home`, and `End` ignore `disabled`:

- `Enter` while the open drop-down has `selected == None` or a disabled `selected` index keeps the
  drop-down open and returns `Reply::Consumed`.
- A left mouse click inside the active pop-up on a disabled item's `item_areas[idx]` keeps the
  drop-down open and returns `Reply::Consumed` (preventing `rat-menu` from closing the menu or
  activating whatever `selected` previously pointed to, such as activating `Redo` when clicking
  disabled `Undo`).
- `Home` moves selection to the first enabled entry, and `End` moves selection to the last enabled
  entry; if no entry is enabled, selection remains `None` and `menu::handle` returns
  `Reply::Consumed`.
- Pressing a bare navigation character matching a disabled entry in the active drop-down (such as
  `u` or `r` in `Edit` when that entry is disabled) keeps the drop-down open and returns
  `Reply::Consumed` instead of leaking to `diagram_widget` (for instance, preventing `r` on a
  disabled `Redo` from rotating a selected line behind the open menu).

**Why not** closing the drop-down without acting when a disabled entry is clicked or pressed: a user
expecting no action on a disabled entry expects the menu to stay in place rather than dismissing
silently as if an action succeeded. **Answered by** the maintainer, in #231.

## Model slice

- `docs/application-model.md` §3 (_The state_): replace the statement that nothing on the screen
  reads from where the walk stands with the rule that `Edit > Undo` and `Edit > Redo` reflect
  `can_undo` and `can_redo` each frame while keeping the same menu labels and geometry.
- `docs/application-model.md` §4 (_The action model_, Undo and Redo): update the Undo/Redo
  paragraphs to say that `Edit > Undo` is disabled when `!can_undo` and `Edit > Redo` is disabled
  when `!can_redo`, drawn with `Modifier::DIM` across the whole row, never selected or reversed,
  ignoring activation with the drop-down kept open, while `Ctrl+Z` / `Ctrl+Shift+Z` stay
  `Reach::Unclaimed` and keep their existing behavior; remove the forward reference saying #231 is
  still pending.
- `docs/application-model.md` §7 (_Properties worth testing_): replace the property claiming no
  entry is ever drawn differently at history ends with the properties that `Edit > Undo` and
  `Edit > Redo` keep their text/positions while their enabled/disabled styling, initial selection,
  and click/`Enter`/`Home`/`End`/navigation-key behavior follow `can_undo` and `can_redo`.
- `docs/tui-conventions.md` §7.4: no change (implemented directly by D3).
- `docs/editing-model.md` §6 and `docs/diagram-model.md`: no change (`can_undo` and `can_redo` are
  already specified in `docs/editing-model.md` §6).

## Public surface

No library crate (`monospace-core`, `monospace-diagram`, `monospace-editing`,
`monospace-glyph-sets`, `monospace-description`) gains or changes any public item, and `Cargo.toml`
/ `Cargo.lock` gain no dependencies. Within `crates/monospace`:

- `crates/monospace/src/diagram_state.rs`:
  - `pub(crate) fn can_undo(&self) -> bool` (remove `#[cfg(test)]`)
  - `pub(crate) fn can_redo(&self) -> bool` (remove `#[cfg(test)]`)
- `crates/monospace/src/menu.rs`:
  - `#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)] pub(crate) struct MenuFlags { pub(crate) can_undo: bool, pub(crate) can_redo: bool }`
  - `pub(crate) fn draw_bar(area: Rect, buf: &mut Buffer, state: &mut MenubarState, theme: &SalsaTheme, focused: bool, flags: MenuFlags)`
  - `pub(crate) fn draw_drop_down(area: Rect, buf: &mut Buffer, state: &mut MenubarState, theme: &SalsaTheme, flags: MenuFlags)`
  - `menu::handle(raw: &Event, chrome: &mut Chrome) -> Reply` retains its signature and enforces D5
    using `chrome.menu().popup` state (`disabled`, `item_areas`, and navigation characters) (with
    `alt_navigation` reading the top-level label definitions directly rather than a
    `static StaticMenu`).
- `crates/monospace/src/frame.rs`:
  - `frame::draw` reads
    `MenuFlags { can_undo: state.diagram().can_undo(), can_redo: state.diagram().can_redo() }` and
    passes it to `menu::draw_bar` and `menu::draw_drop_down`.

## Behavior

1. **Per-frame disabled state:** `Edit > Undo` is disabled iff `!diagram.can_undo()`, and
   `Edit > Redo` is disabled iff `!diagram.can_redo()`. All `File` entries and `Help > About` are
   always enabled.
2. **Entry text and geometry never move:** `Edit` always lists `_Undo|Ctrl+Z` followed by
   `_Redo|Ctrl+Shift+Z`, and `File` / `Edit` / `Help` sizes and cell text are identical regardless
   of `can_undo` and `can_redo`.
3. **Disabled visual styling:** Every cell in a disabled entry's row (label text, navigation
   character, and right-hand shortcut text) is painted with `MENU_BASE`'s foreground and background
   plus `Modifier::DIM`, without `Modifier::REVERSED` and without `KEY_BINDING` or `Style::DISABLED`
   backgrounds.
4. **Enabled visual styling:** Every enabled entry continues to paint with `MENU_BASE` (and
   `KEY_BINDING` on its shortcut, underline on its navigation character), adding
   `Modifier::REVERSED` only on the currently selected entry.
5. **Initial selection on opening a drop-down:** When a drop-down is rendered for the first time
   after opening (or switching labels), if its selection is `None` or points to a disabled item,
   selection becomes the first enabled item (`Some(idx)`), or `None` if every item in the drop-down
   is disabled.
6. **Arrow, `Home`, and `End` navigation:** `Up` and `Down` skip disabled items. `Home` selects the
   first enabled item and `End` selects the last enabled item; if none are enabled, selection stays
   `None` and the event returns `Reply::Consumed`.
7. **Keyboard activation (`Enter` and navigation keys):** Pressing `Enter` when `selected` is `None`
   or disabled leaves the drop-down open, emits no `Action`, and returns `Reply::Consumed`. Pressing
   a bare navigation character of a disabled entry in the active drop-down (`u` when `Undo` is
   disabled, `r` when `Redo` is disabled) leaves the drop-down open, emits no `Action`, and returns
   `Reply::Consumed`.
8. **Mouse hover and click on disabled entries:** Moving the pointer over a disabled entry does not
   select it and leaves the previous selection intact. Left-clicking a disabled entry's row keeps
   the drop-down open, emits no `Action`, never activates another selected entry, and returns
   `Reply::Consumed`.
9. **Active entry activation:** Selecting and activating an enabled `Undo` or `Redo` entry closes
   the drop-down, returns `Reply::Act(Action::Undo)` or `Reply::Act(Action::Redo)`, and returns the
   keyboard to the region as before.
10. **Global shortcuts unchanged:** Pressing `Ctrl+Z` or `Ctrl+Shift+Z` (even while a drop-down is
    open) continues to bypass `menu::handle`, reach `shortcuts::handle_unclaimed`, and call
    `DiagramState::undo` / `DiagramState::redo` directly, leaving the session unchanged at history
    ends.

## Examples

None of the examples below has been observed: this is the acceptance list the building stage checks,
not the record of a run.

**Example 1 — Fresh session (`can_undo = false`, `can_redo = false`), `Alt+E`:**

```text
File Edit Help
┌──┌────────────────────┐──────────────┐
│  │ Undo        Ctrl+Z │              │  <- all cells: MENU_BASE + DIM, not REVERSED
│  │ Redo  Ctrl+Shift+Z │              │  <- all cells: MENU_BASE + DIM, not REVERSED
│  └────────────────────┘              │
```

- `chrome.menu().popup.selected` is `None`.
- Pressing `Enter`, `Home`, `End`, `u`, `r`, or left-clicking `"Undo"` or `"Redo"` returns
  `Reply::Consumed` and leaves `chrome.popup_active() == true`.
- Pressing `Ctrl+Z` still reaches `Action::Undo` via `shortcuts::handle_unclaimed` and leaves the
  session at position `0`.

**Example 2 — After deleting a box (`can_undo = true`, `can_redo = false`), `Alt+E`:**

- `Undo` is enabled, initially selected (`selected == Some(0)`), and drawn with `Modifier::REVERSED`
  and no `Modifier::DIM`.
- `Redo` is disabled and drawn with `MENU_BASE + Modifier::DIM` across `"Redo"` and `"Ctrl+Shift+Z"`
  (no orange `KEY_BINDING` background).
- Pressing `Down` keeps selection on `Some(0)` (`Undo`); pressing `End` stays on `Some(0)`.
- Left-clicking `"Redo"` returns `Reply::Consumed`, keeps the drop-down open, and does **not**
  activate `Undo`.

**Example 3 — After undoing that deletion (`can_undo = false`, `can_redo = true`), `Alt+E`:**

- `Undo` is disabled and drawn with `MENU_BASE + Modifier::DIM` across `"Undo"` and `"Ctrl+Z"`, with
  no `Modifier::REVERSED`.
- `Redo` is enabled, initially selected (`selected == Some(1)`), and drawn with
  `Modifier::REVERSED`.
- Left-clicking `"Undo"` returns `Reply::Consumed`, keeps the drop-down open, and does **not**
  activate `Redo`.
- Pressing `Enter` activates `Redo` (`Reply::Act(Action::Redo)`) and closes the drop-down.

## What proves it

- `crates/monospace/src/frame.rs`:
  - `the_edit_menu_offers_undo_and_redo_at_every_position` (updated): proves entry text/positions
    remain identical across fresh, edited, and undone states while `Undo` and `Redo` cell modifiers
    (`DIM`, `REVERSED`, and absence of `KEY_BINDING` background on disabled shortcuts) flip with
    `can_undo` and `can_redo`.
  - `the_edit_drop_down` contract snapshot: text remains unchanged (all 18 lines match).
- `crates/monospace/src/menu.rs`:
  - Existing `taking_the_undo_entry_becomes_the_action_of_the_key` (with
    `MenuFlags { can_undo: true, can_redo: false }`) and a matching test for `Redo`.
  - New tests covering:
    - disabled styling (`Modifier::DIM` on label, navigation char, and shortcut; no
      `Modifier::REVERSED` and no `KEY_BINDING` bg) when `!can_undo` / `!can_redo`;
    - initial selection skipping disabled `Undo` to `Some(1)` (`Redo`), and landing on `None` when
      both are disabled;
    - `Enter`, `Home`, `End`, and `u`/`r` on disabled entries returning `Reply::Consumed` and
      keeping the drop-down open;
    - left-clicking disabled `Undo` while `Redo` is selected returning `Reply::Consumed` without
      closing the pop-up or activating `Redo`;
    - `File` and `Help` entries remaining enabled regardless of `MenuFlags`.
- `crates/monospace/src/callbacks.rs`:
  - Existing `the_undo_key_with_nothing_behind_changes_nothing` and
    `the_redo_key_with_nothing_ahead_changes_nothing` remain passing without modification.

## Open questions

- **Dirty tracking for `File > Save`:** Whether `File > Save` should also use the entry rule once
  unsaved-change tracking exists is left to #380.
- **Unifying shortcut labels with `shortcuts::SHORTCUTS`:** Keeping the menu's drawn shortcut text
  in sync with the global shortcut table stays in #180.
