---
status: draft
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
unoffered local keys inside sub-modes are consumed to prevent destructive fall-through (such as
accidental figure deletion), while global `Control` shortcuts remain active; `Delete` is removed
from the global `Edit` menu and global shortcut table to become strictly a contextual action of
figure selection; and history walks (`Undo`/`Redo`) deterministically cancel active sub-modes and
clear selection, returning to `IDLE`.

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
  - Unoffered local keys (arrows, `Shift+arrows`, letters, `Delete`, out-of-range digits) return
    `Reply::Consumed`, stopping event propagation and preventing accidental canvas mutations.
  - Global shortcuts carrying `KeyModifiers::CONTROL` (`Ctrl+Z`, `Ctrl+Shift+Z`, `Ctrl+S`, `Ctrl+Q`,
    `Ctrl+N`, `Ctrl+O`) return `Reply::Ignored`, allowing them to fall through to
    `shortcuts::handle_unclaimed`.
- Explicit interaction between global actions, sub-modes, and selection:
  - Executing `Undo` (`Ctrl+Z`) or `Redo` (`Ctrl+Shift+Z`) cancels any active sub-mode
    (`sub_mode = None`), clears selection (`selected = None`), and walks the session history,
    returning to `IDLE`.
  - Opening modal dialogs (`Save`, `Open`, `About`) or transferring keyboard focus to the menu bar
    (`Focus::Bar`) cancels any active sub-mode (`sub_mode = None`).
- Removal of `Delete` from the global `Edit` menu (`menu.rs`) and from the unclaimed global shortcut
  table (`shortcuts.rs`). `Delete` becomes strictly a contextual action provided by the unified
  context table in `SELECTED_BOX`, `SELECTED_LINE`, and `SELECTED_CONNECTOR`.
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
- Guardian tests in `diagram_widget.rs` updated: `'s'` removed from `NOT_THE_REGIONS`, `Delete`
  checked via contextual table, and offer == acceptance verified against context table entries.

### Out

- Multi-line wrap and dynamic dock expansion when bar width exceeds terminal columns (deferred to
  [#280](https://github.com/andresmoschini/monospace/issues/280) / Phase 4).
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
- Unoffered local keys (arrows, `Shift+arrows`, letters, `Delete`, out-of-range digits) return
  `Reply::Consumed`. This stops event propagation in `callbacks::route` and prevents accidental
  canvas displacements or destructive deletion.
- Global application shortcuts carrying `KeyModifiers::CONTROL` (`Ctrl+Z`, `Ctrl+Shift+Z`, `Ctrl+S`,
  `Ctrl+Q`, `Ctrl+N`, `Ctrl+O`) return `Reply::Ignored`. This allows them to pass through the region
  handler to `shortcuts::handle_unclaimed`, keeping essential productivity shortcuts alive.
- History walks (`Undo` / `Redo`) reset `sub_mode = None` and `selected = None`, returning to
  `IDLE`. Walking history shifts the diagram to a different temporal state; resetting sub-mode and
  selection prevents orphan references to non-existent shapes, eliminates visual clutter of inverted
  cells, and ensures deterministic return to `IDLE`.
- Deselecting, clicking outside, opening an overlay dialog (`Save`, `Open`, `About`), or moving
  focus to the menu bar (`Focus::Bar`) resets `sub_mode` to `None`.

**Why not** holding the modal state in `AppState` or a modal overlay: `STROKE_SELECT` is an inline
interaction mode of the diagram region where the canvas remains visible and interactive; storing it
in `AppState` would decouple it from `DiagramState::selected` and require redundant synchronization
across modules. **Why not** deselecting on `Esc`: `Esc` in a sub-mode cancels the sub-mode back to
the enclosing selection, requiring a second `Esc` to drop selection, ensuring consistent
non-destructive escape semantics. **Why not** returning `Reply::Ignored` for all unoffered keys:
`callbacks::route` falls through to `shortcuts::handle_unclaimed`, where `Delete` would dispatch
`Action::Delete` and delete the figure while the user was selecting a stroke. **Why not** returning
`Reply::Consumed` for all keys: silences `Ctrl+Z`, `Ctrl+S`, and `Ctrl+Q`, trapping users in the
sub-mode without access to universal global operations. **Why not** keeping selection across `Undo`:
amends the historical rule of #181 because restoring an earlier state could invalidate the selected
`ShapeId` and leaves misleading visual cues when inspecting history. **Answered by** the maintainer.

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
  `Delete`). In `STROKE_SELECT`, unoffered local keys return `Reply::Consumed`, and Control
  shortcuts return `Reply::Ignored`.

**Why not** continuing separate `match` expressions in `status.rs` and `diagram_widget.rs`:
duplicates context logic and causes divergence between what is offered and what is accepted (the
smell identified in #288). **Why not** keeping `Delete` in the Edit menu: menus are for global,
state-independent commands. Deleting a figure requires a selected shape; offering `Delete` with
nothing selected violates contextual coherence and confuses users. **Why not** a modal overlay
dialog: direct key selection in the bottom bar is faster, maintains visual context of the drawing,
and avoids occlusion. **Why not** cycling strokes directly with `s`: cycling prevents direct choice,
conceals available options, and clutters undo history. **Answered by** the maintainer.

## Model slice

- `docs/diagram-model.md` §4: document `Shape::with_stroke`, which replaces the stroke of `Box`,
  `Line`, or `Connector` while preserving all other properties.
- `docs/editing-model.md` §2 and §5: document `Command::SetStroke`, which changes the stroke of a
  figure by its `ShapeId`, and note that setting the same stroke produces `None` and records no step
  in history (#285).
- `docs/application-model.md`:
  - §1: document `Sub-mode`, `Global action`, and `Contextual action` in vocabulary.
  - §2: document that `Edit` menu offers `Undo` and `Redo`, with `Delete` removed as a contextual
    action.
  - §3: document `sub_mode: Option<SubMode>` in `DiagramState` and establish that history walks
    (`undo`/`redo`) cancel active sub-modes and clear selection to `IDLE`.
  - §4: document two-layer dispatching in sub-modes (local keys consumed, Control shortcuts ignored)
    and document that `Delete` is a contextual action of selection contexts.
  - §7: document acceptance properties for sub-modes, two-layer dispatching, and context table.

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
      Directional(fn(i16, i16) -> Action),
      EnterSubMode(SubMode),
      LeaveSubMode,
      Deselect,
  }

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
7. While in `STROKE_SELECT`, unoffered local keys (arrows, `Shift+arrows`, letters, `Delete`,
   out-of-range digits) are consumed (`Reply::Consumed`), preventing canvas displacements or
   accidental figure deletion.
8. While in `STROKE_SELECT`, global shortcuts carrying `KeyModifiers::CONTROL` (`Ctrl+Z`,
   `Ctrl+Shift+Z`, `Ctrl+S`, `Ctrl+Q`, `Ctrl+N`, `Ctrl+O`) are ignored (`Reply::Ignored`), allowing
   them to fall through to `shortcuts::handle_unclaimed`.
9. Invoking `Undo` (`Ctrl+Z`) or `Redo` (`Ctrl+Shift+Z`) cancels any active sub-mode
   (`sub_mode = None`), clears selection (`selected = None`), and walks session history, returning
   to `IDLE`.
10. Opening a modal overlay (`Save`, `Open`, `About`) or moving focus to the menu bar (`Focus::Bar`)
    cancels any active sub-mode (`sub_mode = None`).
11. In `SELECTED_BOX`, `SELECTED_LINE`, and `SELECTED_CONNECTOR`, the contextual command row
    displays `STROKE: [s]`, and pressing `Delete` dispatches `Action::Delete`.
12. In `IDLE`, pressing `Delete` is ignored (`Reply::Ignored`).
13. In `STROKE_SELECT`, the contextual command row displays
    `STROKE: [1] light ┌─┐ [2] double ╔═╗ [3] light-round ╭─╮ [4] heavy ┏━┓ [5] ascii +-+ · [Esc] Cancel`,
    and the status row reports `STROKE SELECTION for <Kind> #<id> · Current: <current_stroke>`.
14. The set of contextual keys accepted by the region and the set of keys offered in the contextual
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
| `STROKE_SELECT` (light) | `Ctrl+Z` (Undo) | `None` (`IDLE`) | `Action::Undo`        | yes              | walks back         |
| `STROKE_SELECT` (light) | `Ctrl+S` (Save) | `None` (box)    | `Action::Save`        | no               | opens save dialog  |
| `IDLE`                  | `s`             | `None` (`IDLE`) | none (`Ignored`)      | no               | no                 |
| `IDLE`                  | `Delete`        | `None` (`IDLE`) | none (`Ignored`)      | no               | no                 |
| `SELECTED_BOX` (light)  | `Delete`        | `None` (`IDLE`) | `Action::Delete`      | yes              | yes (1 step)       |

| Context                  | Status Row (Row 0)                                                            | Command Row (Row 1)                                                                                   |
| ------------------------ | ----------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| `IDLE`                   | `Diagram · Origin: (0, 0) · Size: 0×0 · Shapes: 0`                            | `SELECT: [Click] Pick`                                                                                |
| `SELECTED_BOX` #1        | `Box #1 · Pos: (2, 2) · Size: 10×4 · Stroke: light · Fill: None`              | `MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · STROKE: [s] · [Del] Delete · [Esc] Deselect`                       |
| `SELECTED_LINE` #2       | `Line #2 · Pos: (0, 0) · Length: 6 · Orientation: Horizontal · Stroke: light` | `MOVE: ↑↓←→ · STROKE: [s] · [Del] Delete · [Esc] Deselect`                                            |
| `SELECTED_CONNECTOR` #3  | `Connector #3 · Stroke: light · 1. From: ... · 2. To: ...`                    | `MOVE: ↑↓←→ · STROKE: [s] · [Del] Delete · [Esc] Deselect`                                            |
| `STROKE_SELECT` (Box #1) | `STROKE SELECTION for Box #1 · Current: light`                                | `STROKE: [1] light ┌─┐ [2] double ╔═╗ [3] light-round ╭─╮ [4] heavy ┏━┓ [5] ascii +-+ · [Esc] Cancel` |

## What proves it

| Rule | Test                                                                              |
| ---- | --------------------------------------------------------------------------------- |
| 1    | `pressing_s_on_selected_figure_enters_stroke_select_sub_mode`                     |
| 2    | `pressing_s_in_idle_is_ignored`                                                   |
| 3    | `pressing_1_to_5_in_stroke_select_applies_stroke_and_exits_sub_mode`              |
| 4    | `setting_identical_stroke_leaves_diagram_untouched_and_records_no_step`           |
| 5    | `setting_different_stroke_records_one_step_in_session_history`                    |
| 6    | `pressing_escape_in_stroke_select_cancels_sub_mode_without_deselecting`           |
| 7    | `unoffered_local_keys_in_stroke_select_are_consumed`                              |
| 8    | `global_control_shortcuts_pass_through_from_stroke_select`                        |
| 9    | `undo_and_redo_in_stroke_select_cancel_sub_mode_clear_selection_and_walk_history` |
| 10   | `opening_overlay_or_focusing_menu_cancels_sub_mode`                               |
| 11   | `command_row_offers_stroke_s_and_delete_for_selected_figures`                     |
| 12   | `delete_in_idle_is_ignored`                                                       |
| 13   | `stroke_select_context_formats_canonical_prompt_and_status_row`                   |
| 14   | `context_table_guarantees_offer_equals_acceptance_across_all_contexts`            |

Existing guardian tests updated during building:

- `crates/monospace/src/diagram_widget.rs`: `'s'` removed from `NOT_THE_REGIONS`; `Delete` asserted
  via contextual table; `assert_offer_matches_acceptance` verifies against `context_blocks`.
- `crates/monospace/src/callbacks.rs`: `the_undo_and_redo_keys_leave_the_selection_alone` updated to
  verify that `Undo` and `Redo` return to `IDLE` (`selected = None`, `sub_mode = None`).
- `crates/monospace/src/menu.rs`: `_Edit` menu tested for `_Undo` and `_Redo` only.

## Open questions

None.
