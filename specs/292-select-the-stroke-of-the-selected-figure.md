---
status: implemented
decided: "#293"
implemented: "#298"
date: 2026-11-03
---

# Select the stroke of the selected figure with an s sub-mode

## Why now

Monospace supports rendering boxes, lines, and connectors in five distinct stroke styles: `"light"`,
`"double"`, `"light-round"`, `"heavy"`, and `"ascii"`. However, in the interactive TUI, once a shape
exists or is loaded, there is no mechanism to alter its stroke.

Issue [#279](https://github.com/andresmoschini/monospace/issues/279) introduced the contextual
status and command bar, which reports the current stroke in Row 0 (`Stroke: light`), but left style
manipulation commands for later issues. Issue
[#285](https://github.com/andresmoschini/monospace/issues/285) established that commands changing
nothing record no step in session history, and issue
[#288](https://github.com/andresmoschini/monospace/issues/288) enforced the Representation Rule
("offer == acceptance") between the status bar and the widget dispatcher.

Adding the `s` sub-mode (`STROKE_SELECT`) provides the first interactive sub-mode in Monospace and
establishes the Option D unified context table architecture (Phase 5 of the contextual functions
plan). Concurrently, it settles the boundary between global actions and contextual actions:
unoffered local keys inside sub-modes are consumed, while the keys the unclaimed global table
answers remain active; `Delete` is removed from the global `Edit` menu and global shortcut table to
become strictly a contextual action of figure selection; and history walks (`Undo`/`Redo`) cancel
active sub-modes. Whether a walk should also clear the selection is a separate question, owned by
[#297](https://github.com/andresmoschini/monospace/issues/297).

## Scope

### In

- An ephemeral `STROKE_SELECT` sub-mode entered via `s` when any figure (`Box`, `Line`, or
  `Connector`) is selected (`SELECTED_BOX`, `SELECTED_LINE`, `SELECTED_CONNECTOR`).
- Single-key direct stroke selection:
  - `1`: `Stroke("light")` (`┌─┐`)
  - `2`: `Stroke("double")` (`╔═╗`)
  - `3`: `Stroke("light-round")` (`╭─╮`)
  - `4`: `Stroke("heavy")` (`┏━┓`)
  - `5`: `Stroke("ascii")` (`+-+`) Pressing `1..=5` applies the chosen stroke, clears
    `STROKE_SELECT`, returns to the previous selection context, and requests a repaint.
- Pressing `Esc` while in `STROKE_SELECT` cancels the sub-mode and returns to the previous selection
  context without applying any change and without deselecting the figure.
- Two-layer key dispatching in sub-modes:
  - Every key the region is asked about that the table does not offer (arrows, `Shift+arrows`,
    letters, `Delete`, `Tab`, `Enter`, `Home`, `End`, function keys, out-of-range digits) returns
    `Reply::Consumed`, stopping event propagation and preventing canvas mutations. The chrome's own
    keys (`F10`, `Alt` plus a navigation character) are claimed by the menu before the region and
    never reach it.
  - A key the unclaimed global shortcut table answers (`Ctrl+N`, `Ctrl+O`, `Ctrl+S`, `Ctrl+Q`,
    `Ctrl+Z`, `Ctrl+Shift+Z`) returns `Reply::Ignored`, allowing it to fall through to
    `shortcuts::handle_unclaimed`. The forced exit `Ctrl+C` is `Reach::Always` and is answered
    before the region, so it is not part of this rule.
- Explicit interaction between global actions, sub-modes, and selection:
  - Executing `Undo` (`Ctrl+Z`) or `Redo` (`Ctrl+Shift+Z`) cancels any active sub-mode
    (`sub_mode = None`) and walks the session history. The selection is left as
    [#181](https://github.com/andresmoschini/monospace/issues/181) says; clearing it is
    [#297](https://github.com/andresmoschini/monospace/issues/297)'s question.
  - Opening an application-level modal dialog (`Save`, `Open`, `About`, `ShowError`) or transferring
    keyboard focus to the menu bar (`Focus::Bar`) cancels any active sub-mode (`sub_mode = None`). A
    dialog that belongs to a sub-mode itself is not one of these.
  - A click that changes the selection — another figure, or no figure at all — cancels the active
    sub-mode.
- Removal of `Delete` from the global `Edit` menu (`menu.rs`) and from the unclaimed global shortcut
  table (`shortcuts.rs`). `Delete` becomes strictly a contextual action provided by the unified
  context table in `SELECTED_BOX`, `SELECTED_LINE`, and `SELECTED_CONNECTOR`; `Backspace` is not a
  second delete key.
- A new command in `monospace-editing`: `Command::SetStroke { id: ShapeId, stroke: Stroke }`.
- A new method in `monospace-diagram`: `Shape::with_stroke(&self, stroke: Stroke) -> Self` for
  `Box`, `Line`, and `Connector`, preserving position, dimensions, fill, text, and connector
  endpoints.
- Compliance with #285: choosing the stroke the figure already has produces `None` in
  `Command::apply_to` and records no step in session history.
- Contextual bar updates:
  - In `SELECTED_BOX`:
    `MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · STROKE: [s] · [Del] Delete · [Esc] Deselect`.
  - In `SELECTED_LINE` and `SELECTED_CONNECTOR`:
    `MOVE: ↑↓←→ · STROKE: [s] · [Del] Delete · [Esc] Deselect`.
  - In `STROKE_SELECT`: command row displays canonical single-block prompt:
    `STROKE: [1] light ┌─┐ [2] double ╔═╗ [3] light-round ╭─╮ [4] heavy ┏━┓ [5] ascii +-+ · [Esc] Cancel`,
    and status row displays `STROKE SELECTION for <Kind> #<id> · Current: <current_stroke>`.
- Unified context table (Option D) implemented in `crates/monospace/src/context_table.rs`: single
  source of truth for contextual key bindings, visual labels, and actions shared by status bar
  formatting and widget key dispatching.
- Guardian tests in `diagram_widget.rs` updated: `'s'` and `Delete` removed from `NOT_THE_REGIONS`,
  `Delete` checked via the contextual table, and offer == acceptance verified against context table
  entries.

### Out

- Multi-line wrap and dynamic dock expansion when bar width exceeds terminal columns (deferred to
  [#280](https://github.com/andresmoschini/monospace/issues/280) / Phase 4).
- Clearing the selection when the history is walked (amending #181):
  [#297](https://github.com/andresmoschini/monospace/issues/297).
- `Backspace` as a delete key; the contextual action is `Delete` only.
- Nested sub-modes (connector endpoint editing, fill → glyph palette). `Option<SubMode>` models flat
  sub-modes; the first nested one decides its parent or stack in its own spec.
- Custom stroke names beyond the five catalog presets.
- Fill sub-mode (`f`), layer order sub-mode (`z`), connector endpoint editing (`1`/`2`), and text
  editing (`Enter`), which have their own future issues in Phase 6.
- Multi-figure stroke batch modification (selection is strictly single-figure).

## The decision

**D1 — ephemeral sub-mode modeling, dispatching, and transitions.**

**Answer:** `DiagramState` holds an optional sub-mode state `sub_mode: Option<SubMode>` where
`SubMode::StrokeSelect` represents active stroke selection. Pressing `s` while a shape is selected
transitions `sub_mode` to `Some(SubMode::StrokeSelect)` and returns `Reply::Repaint`. While in
`StrokeSelect`:

- Keys `1..=5` dispatch `Action::SetStroke(stroke)`, reset `sub_mode` to `None` (returning to the
  figure's selection state), and return `Reply::Act(...)`.
- `Esc` resets `sub_mode` to `None` without modifying the diagram, leaving the figure selected, and
  returns `Reply::Repaint`.
- Every key the region is asked about that the table does not offer returns `Reply::Consumed`. That
  includes arrows, `Shift+arrows`, letters, `Delete`, `Tab`, `Enter`, `Home`, `End`, function keys
  and out-of-range digits. The chrome's own keys (`F10`, `Alt` plus a navigation character) are
  claimed by the menu before the region, so they never reach this rule.
- A key the unclaimed global shortcut table answers (`Ctrl+N`, `Ctrl+O`, `Ctrl+S`, `Ctrl+Q`,
  `Ctrl+Z`, `Ctrl+Shift+Z`) returns `Reply::Ignored`, so it falls through to
  `shortcuts::handle_unclaimed`. The forced exit `Ctrl+C` is `Reach::Always` and is answered before
  the region, so it is not part of this rule.
- History walks (`Undo` / `Redo`) reset `sub_mode = None` and walk the session history. The
  selection is left alone, as [#181](https://github.com/andresmoschini/monospace/issues/181) says: a
  selection the restored drawing does not record is kept and inverts nothing. Clearing it is
  [#297](https://github.com/andresmoschini/monospace/issues/297)'s question.
- A click that changes the selection (another figure, or no figure), opening an application-level
  overlay dialog (`Save`, `Open`, `About`, `ShowError`), or moving focus to the menu bar
  (`Focus::Bar`) resets `sub_mode` to `None`. A dialog that belongs to a sub-mode itself is not one
  of these.

**Why not** holding the modal state in `AppState` or a modal overlay: `STROKE_SELECT` is an inline
interaction mode of the diagram region where the canvas remains visible and interactive; storing it
in `AppState` would decouple it from `DiagramState::selected` and require redundant synchronization
across modules. **Why not** deselecting on `Esc`: `Esc` in a sub-mode cancels the sub-mode back to
the enclosing selection, requiring a second `Esc` to drop selection, ensuring consistent
non-destructive escape semantics. **Why not** returning `Reply::Ignored` for all unoffered keys:
`callbacks::route` falls through to `shortcuts::handle_unclaimed`, so a key the bar does not offer
could reach a global row or a row added later, and the sub-mode would not own the keyboard it says
it owns. `Consumed` is the same answer a dialog gives an event it traps: "mine, and nothing
changed". `Delete` is contextual now, so it would not delete today; the rule does not rest on that
accident. **Why not** returning `Reply::Consumed` for all keys: silences `Ctrl+Z`, `Ctrl+S`, and
`Ctrl+Q`, trapping users in the sub-mode without access to universal global operations. **Why not**
clearing the selection on a walk: that amends the rule of #181 and is a separate question, owned by
[#297](https://github.com/andresmoschini/monospace/issues/297); this slice only cancels the
sub-mode. **Answered by** the maintainer.

**D2 — domain command and editing operation.**

**Answer:** `monospace-editing` adds `Command::SetStroke { id: ShapeId, stroke: Stroke }`, and
`monospace-diagram` adds `Shape::with_stroke(&self, stroke: Stroke) -> Self`. In `apply_to`,
`SetStroke` executes `changed(diagram, id, |shape| shape.with_stroke(stroke.clone()))`. For `Box`,
`Line`, and `Connector`, `with_stroke` produces a new `Shape` clone with `stroke` replaced while
preserving position, size, fill, text, and connector endpoints. If the figure already uses the
selected stroke, `changed` evaluates to `after == *before`, yielding `None`, which under #285
records no step in session history and preserves redo steps. **Why not**
`Command::Replace { id, shape }`: rejected by `docs/editing-model.md` §2 because caller would have
to reconstruct geometry and internal state rather than expressing intent. **Why not** in-place
mutation without `Command`: bypassing `Session::perform` breaks undo/redo tracking. **Why not**
recording a step when stroke is unchanged: violates #285 contract and creates ghost undo steps.
**Answered by** the maintainer.

**D3 — contextual bar presentation, unified context table (Option D), and contextual Delete.**

**Answer:** The context bar and the widget event dispatcher derive their offered keys, labels, and
actions from a unified context table implemented in `crates/monospace/src/context_table.rs` (Option
D). The module defines `ContextBlock`, `ContextAction`, `KeyPattern`, `ContextMeaning`, and
`SubMode`.

- Visual grammar: inside a `ContextBlock`, actions are separated by a single space (`" "`).
  Categories are prefixed with uppercase names (`"STROKE: "` or `"MOVE: "`) or omitted for trailing
  actions. Distinct blocks are delimited by `" · "` with a space on each side.
- Canonical prompt: in `STROKE_SELECT`, the single-block prompt formats as
  `STROKE: [1] light ┌─┐ [2] double ╔═╗ [3] light-round ╭─╮ [4] heavy ┏━┓ [5] ascii +-+ · [Esc] Cancel`.
  No spurious `·` is placed between options 3 and 4; the options form a single `STROKE:` category.
- Contextual `Delete`: `Delete` is removed from the global `_Edit` menu (`menu.rs`) and from the
  unclaimed table (`shortcuts.rs`). `_Edit` retains only universal session actions (`_Undo|Ctrl+Z`
  and `_Redo|Ctrl+Shift+Z`). `Delete` is registered exclusively as a contextual action in the
  unified context table for `SELECTED_BOX`, `SELECTED_LINE`, and `SELECTED_CONNECTOR`. With nothing
  selected (`IDLE`) or in a sub-mode (`STROKE_SELECT`), `Delete` is not offered and performs no
  action.
- Dispatching: `diagram_widget::key` delegates key matching to `context_table::dispatch_key`. In
  selection contexts, matching keys dispatch corresponding actions (`Move`, `Resize`, `SetStroke`,
  `Delete`). In `STROKE_SELECT`, every key the table does not offer returns `Reply::Consumed`, and a
  key the unclaimed global shortcut table answers returns `Reply::Ignored`.
- Static table: the stroke options carry a borrowed name (`ContextMeaning::SetStroke(&'static str)`)
  and `dispatch_key` builds the `Stroke` when the action is dispatched. A `Stroke` owns a `String`,
  so it cannot be a field of a `&'static [ContextBlock]`; the table stays static and the owned value
  is made at dispatch time.

**Why not** continuing separate `match` expressions in `status.rs` and `diagram_widget.rs`:
duplicates context logic and causes divergence between what is offered and what is accepted (the
smell identified in #288). **Why not** keeping `Delete` in the Edit menu: menus are for global,
state-independent commands. Deleting a figure requires a selected shape; offering `Delete` with
nothing selected violates contextual coherence and confuses users. **Why not** removing `Delete`
from the bar and leaving it global: the bar would stop offering the action that the contextual
design exists to offer, and the same key would be answered by a different layer than every other key
of the selected figure, so the table would no longer be the one definition of the context. **Why
not** a modal overlay dialog: direct key selection in the bottom bar is faster, maintains visual
context of the drawing, and avoids occlusion. **Why not** cycling strokes directly with `s`: cycling
prevents direct choice, conceals available options, and clutters undo history. **Answered by** the
maintainer.

## Model slice

- `docs/diagram-model.md` §9: document `Shape::with_stroke`, which replaces the stroke of `Box`,
  `Line`, or `Connector` while preserving all other properties.
- `docs/editing-model.md` §2 and §5: document `Command::SetStroke`, which changes the stroke of a
  figure by its `ShapeId`, and note that setting the same stroke produces `None` and records no step
  in history (#285).
- `docs/application-model.md`:
  - §1: document `Sub-mode`, `Global action`, and `Contextual action` in vocabulary.
  - §2: document that `Edit` menu offers `Undo` and `Redo`, with `Delete` removed as a contextual
    action.
  - §3: document `sub_mode: Option<SubMode>` in `DiagramState` and establish that history walks
    (`undo`/`redo`) cancel active sub-modes while leaving the selection as #181 says; clearing it is
    [#297](https://github.com/andresmoschini/monospace/issues/297)'s.
  - §4: document two-layer dispatching in sub-modes (keys the table does not offer are consumed,
    keys the unclaimed global table answers are ignored, the chrome's keys are asked before the
    region) and document that `Delete` is a contextual action of selection contexts.
  - §7: document acceptance properties for sub-modes, two-layer dispatching, the context table, and
    `Delete` as a contextual action; and that a walk cancels the sub-mode without touching the
    selection.
  - §8: record that clearing the selection on a walk is
    [#297](https://github.com/andresmoschini/monospace/issues/297)'s, not this slice's.

## Public surface

- `monospace-diagram`:

  ```rust
  impl Shape {
      #[must_use]
      pub fn with_stroke(&self, stroke: Stroke) -> Self;
  }
  ```

- `monospace-editing`:

  ```rust
  pub enum Command {
      // ... existing variants
      /// The figure's stroke is replaced with `stroke`.
      SetStroke {
          /// The figure whose stroke is being set.
          id: ShapeId,
          /// The stroke to apply.
          stroke: Stroke,
      },
  }
  ```

- `monospace`:

  ```rust
  pub(crate) enum Action {
      // ... existing variants
      /// The selected shape's stroke is set to `stroke`.
      SetStroke(Stroke),
  }

  // crates/monospace/src/context_table.rs

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub(crate) enum SubMode {
      StrokeSelect,
  }

  #[derive(Debug, Clone, Copy, PartialEq, Eq)]
  pub(crate) enum KeyPattern {
      Char(char),
      Code(KeyCode),
      Arrows,
      ShiftArrows,
  }

  impl KeyPattern {
      pub(crate) fn matches(&self, event: &KeyEvent) -> bool;
  }

  pub(crate) enum ContextMeaning {
      Act(Action),
      SetStroke(&'static str),
      Directional(fn(i16, i16) -> Action),
      EnterSubMode(SubMode),
      LeaveSubMode,
      Deselect,
  }

  // `SetStroke` carries the catalog name rather than a `Stroke`: a `Stroke` owns a `String` and
  // cannot live in the `&'static` table; `dispatch_key` builds it at dispatch time.

  pub(crate) struct ContextAction {
      pub pattern: KeyPattern,
      pub display: &'static str,
      pub meaning: ContextMeaning,
  }

  pub(crate) struct ContextBlock {
      pub category: Option<&'static str>,
      pub actions: &'static [ContextAction],
  }

  pub(crate) fn context_blocks(state: &DiagramState) -> &'static [ContextBlock];
  pub(crate) fn format_command_line(state: &DiagramState) -> String;
  pub(crate) fn dispatch_key(state: &mut DiagramState, event: &KeyEvent) -> Reply;

  // crates/monospace/src/diagram_state.rs

  impl DiagramState {
      pub(crate) fn sub_mode(&self) -> Option<SubMode>;
      pub(crate) fn set_sub_mode(&mut self, sub_mode: Option<SubMode>);
      pub(crate) fn undo(&mut self);
      pub(crate) fn redo(&mut self);
      pub(crate) fn select(&mut self, id: Option<ShapeId>);
  }

  // `select` clears `sub_mode` when the selection changes; `undo` and `redo` clear `sub_mode` and
  // leave `selected` as #181 says.

  // crates/monospace/src/status.rs
  // `status_line` gains the STROKE_SELECT arm:
  // `STROKE SELECTION for <Kind> #<id> · Current: <stroke>`.
  // `command_line` delegates to `context_table::format_command_line`.

  // crates/monospace/src/callbacks.rs
  // `apply(Save | Open | About | ShowError)` clears `DiagramState::sub_mode` before opening the
  // overlay, and `route` clears it when the keyboard has moved to `Focus::Bar`.

  // crates/monospace/src/menu.rs
  // Edit menu entries: &["_Undo|Ctrl+Z", "_Redo|Ctrl+Shift+Z"] (Delete removed).

  // crates/monospace/src/shortcuts.rs
  // SHORTCUTS table: Delete removed from unclaimed global shortcuts.
  ```

## Behavior

1. When a `Box`, `Line`, or `Connector` is selected and the region holds focus, pressing `s`
   transitions `DiagramState` to `SubMode::StrokeSelect` and requests a repaint (`Reply::Repaint`).
2. When nothing is selected (`IDLE`), pressing `s` is ignored (`Reply::Ignored`).
3. While in `STROKE_SELECT`, pressing `1`, `2`, `3`, `4`, or `5` dispatches `Action::SetStroke` with
   `Stroke::from("light")`, `Stroke::from("double")`, `Stroke::from("light-round")`,
   `Stroke::from("heavy")`, or `Stroke::from("ascii")` respectively, clears `SubMode::StrokeSelect`,
   and requests a repaint.
4. While in `STROKE_SELECT`, if the chosen stroke matches the figure's current stroke,
   `Command::SetStroke` leaves the diagram unchanged, records no step in `Session` history, and
   leaves redo steps ahead intact (#285).
5. While in `STROKE_SELECT`, if the chosen stroke differs from the figure's current stroke,
   `Command::SetStroke` updates the figure's stroke and records exactly one step in `Session`
   history.
6. While in `STROKE_SELECT`, pressing `Esc` clears `SubMode::StrokeSelect` back to normal selection
   without modifying the diagram or recording any step, leaving the figure selected, and requests a
   repaint (`Reply::Repaint`).
7. While in `STROKE_SELECT`, every key the region is asked about that the table does not offer is
   consumed (`Reply::Consumed`): arrows, `Shift+arrows`, letters, `Delete`, `Tab`, `Enter`, `Home`,
   `End`, function keys, and out-of-range digits. The chrome's keys (`F10`, `Alt` plus a navigation
   character) are claimed by the menu before the region and are not part of this rule.
8. While in `STROKE_SELECT`, a key the unclaimed global shortcut table answers (`Ctrl+Z`,
   `Ctrl+Shift+Z`, `Ctrl+S`, `Ctrl+Q`, `Ctrl+N`, `Ctrl+O`) is ignored (`Reply::Ignored`), allowing
   it to fall through to `shortcuts::handle_unclaimed`. `Ctrl+C` is `Reach::Always` and is answered
   before the region.
9. Invoking `Undo` (`Ctrl+Z`) or `Redo` (`Ctrl+Shift+Z`) cancels any active sub-mode
   (`sub_mode = None`) and walks session history. The selection is left alone, as #181 says; a
   selection the restored drawing does not record is kept and inverts nothing. Clearing it is
   [#297](https://github.com/andresmoschini/monospace/issues/297)'s.
10. Opening an application-level modal overlay (`Save`, `Open`, `About`, `ShowError`) or moving
    focus to the menu bar (`Focus::Bar`) cancels any active sub-mode (`sub_mode = None`). A dialog
    that belongs to a sub-mode itself is not one of these.
11. A click that changes the selection — another figure, or no figure at all — cancels the active
    sub-mode (`sub_mode = None`).
12. In `SELECTED_BOX`, `SELECTED_LINE`, and `SELECTED_CONNECTOR`, the contextual command row
    displays `STROKE: [s]`, and pressing `Delete` dispatches `Action::Delete`.
13. In `IDLE`, pressing `Delete` is ignored (`Reply::Ignored`).
14. In `STROKE_SELECT`, the contextual command row displays
    `STROKE: [1] light ┌─┐ [2] double ╔═╗ [3] light-round ╭─╮ [4] heavy ┏━┓ [5] ascii +-+ · [Esc] Cancel`,
    and the status row reports `STROKE SELECTION for <Kind> #<id> · Current: <current_stroke>`.
15. The set of contextual keys accepted by the region and the set of keys offered in the contextual
    command row are derived from a unified context table, satisfying the Representation Rule across
    all contexts (`IDLE`, `SELECTED_BOX`, `SELECTED_LINE`, `SELECTED_CONNECTOR`, `STROKE_SELECT`).

## Examples

Hypothetical: none of it has been run; it is what the acceptance list checks, not the record of a
run.

| Initial Context         | Key Pressed     | Sub-mode After  | Action Dispatched     | Diagram Changed? | Step Recorded?     |
| ----------------------- | --------------- | --------------- | --------------------- | ---------------- | ------------------ |
| `SELECTED_BOX` (light)  | `s`             | `StrokeSelect`  | none (`Repaint`)      | no               | no                 |
| `STROKE_SELECT` (light) | `2` (`double`)  | `None` (box)    | `SetStroke("double")` | yes              | yes (1 step)       |
| `STROKE_SELECT` (light) | `1` (`light`)   | `None` (box)    | `SetStroke("light")`  | no               | no (0 steps, #285) |
| `STROKE_SELECT` (light) | `Esc`           | `None` (box)    | none (`Repaint`)      | no               | no                 |
| `STROKE_SELECT` (light) | `Delete`        | `StrokeSelect`  | none (`Consumed`)     | no               | no                 |
| `STROKE_SELECT` (light) | `↑`             | `StrokeSelect`  | none (`Consumed`)     | no               | no                 |
| `STROKE_SELECT` (light) | `Ctrl+Z` (Undo) | `None` (box)    | `Action::Undo`        | yes              | walks back         |
| `STROKE_SELECT` (light) | `Ctrl+S` (Save) | `None` (box)    | `Action::Save`        | no               | opens save dialog  |
| `IDLE`                  | `s`             | `None` (`IDLE`) | none (`Ignored`)      | no               | no                 |
| `IDLE`                  | `Delete`        | `None` (`IDLE`) | none (`Ignored`)      | no               | no                 |
| `SELECTED_BOX` (light)  | `Delete`        | `None` (`IDLE`) | `Action::Delete`      | yes              | yes (1 step)       |

| Context                  | Status Row (Row 0)                                                            | Command Row (Row 1)                                                                                   |
| ------------------------ | ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `IDLE`                   | `monospace 0.1.0 · Footprint: nothing drawn · 0 shapes · Viewport: (0, 0)`    | `SELECT: [Click] Pick`                                                                                |
| `SELECTED_BOX` #1        | `Box #1 · Pos: (2, 2) · Size: 10×4 · Stroke: light · Fill: None`              | `MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · STROKE: [s] · [Del] Delete · [Esc] Deselect`                       |
| `SELECTED_LINE` #2       | `Line #2 · Pos: (0, 0) · Length: 6 · Orientation: Horizontal · Stroke: light` | `MOVE: ↑↓←→ · STROKE: [s] · [Del] Delete · [Esc] Deselect`                                            |
| `SELECTED_CONNECTOR` #3  | `Connector #3 · Stroke: light · 1. From: ... · 2. To: ...`                    | `MOVE: ↑↓←→ · STROKE: [s] · [Del] Delete · [Esc] Deselect`                                            |
| `STROKE_SELECT` (Box #1) | `STROKE SELECTION for Box #1 · Current: light`                                | `STROKE: [1] light ┌─┐ [2] double ╔═╗ [3] light-round ╭─╮ [4] heavy ┏━┓ [5] ascii +-+ · [Esc] Cancel` |

At 80 columns the `STROKE_SELECT` command row is truncated until
[#280](https://github.com/andresmoschini/monospace/issues/280); the canonical string above is the
whole prompt, and Phase 4 is the slice that makes it fit.

## What proves it

| Rule | Test                                                                    |
| ---- | ----------------------------------------------------------------------- |
| 1    | `pressing_s_on_selected_figure_enters_stroke_select_sub_mode`           |
| 2    | `pressing_s_in_idle_is_ignored`                                         |
| 3    | `pressing_1_to_5_in_stroke_select_applies_stroke_and_exits_sub_mode`    |
| 4    | `setting_identical_stroke_leaves_diagram_untouched_and_records_no_step` |
| 5    | `setting_different_stroke_records_one_step_in_session_history`          |
| 6    | `pressing_escape_in_stroke_select_cancels_sub_mode_without_deselecting` |
| 7    | `unoffered_local_keys_in_stroke_select_are_consumed`                    |
| 8    | `global_unclaimed_shortcuts_pass_through_from_stroke_select`            |
| 9    | `undo_and_redo_in_stroke_select_cancel_sub_mode_and_walk_history`       |
| 10   | `opening_overlay_or_focusing_menu_cancels_sub_mode`                     |
| 11   | `clicking_another_figure_or_outside_cancels_sub_mode`                   |
| 12   | `command_row_offers_stroke_s_and_delete_for_selected_figures`           |
| 13   | `delete_in_idle_is_ignored`                                             |
| 14   | `stroke_select_context_formats_canonical_prompt_and_status_row`         |
| 15   | `context_table_guarantees_offer_equals_acceptance_across_all_contexts`  |

Existing guardian tests updated during building:

- `crates/monospace/src/diagram_widget.rs`: `'s'` and `Delete` removed from `NOT_THE_REGIONS`; the
  `STROKE_SELECT` fixture added; `assert_offer_matches_acceptance` reads `context_blocks` instead of
  its own list.
- `crates/monospace/src/callbacks.rs`: the delete-entry tests removed or rewritten for the
  contextual key; `the_undo_and_redo_keys_leave_the_selection_alone` stays as #181 wrote it, and the
  sub-mode cancellation is asserted separately.
- `crates/monospace/src/menu.rs`: `_Edit` menu tested for `_Undo` and `_Redo` only; the `DELETE`
  entry and its meaning arm removed.
- `crates/monospace/src/shortcuts.rs`: the `Delete` row removed, and the doc comment that used it as
  the no-modifier row rewritten.
- `crates/monospace/src/status.rs`: the `STROKE: [s]` block and the `STROKE_SELECT` prompt/status
  row asserted.
- `crates/monospace/src/diagram_state.rs`: `undo`/`redo` cancel `sub_mode` and leave the selection;
  `select` clears `sub_mode` when the selection changes.
- `docs/application-model.md`: the stale `Delete` sentence in §4 corrected.

## Open questions

- **Nested sub-modes.** `Option<SubMode>` models the flat sub-modes this slice has. Connector
  endpoint editing (`1`/`2` with `a`/`t`/`d`/`0`) and the fill palette (`f` then `m`) are nested;
  their own specs decide whether the parent is encoded in the variant or held in a stack.
- **The selection when the history is walked.** #181 leaves it alone; this slice only cancels the
  active sub-mode. Whether a walk should clear it is
  [#297](https://github.com/andresmoschini/monospace/issues/297).
