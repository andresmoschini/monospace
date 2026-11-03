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

Adding the `s` sub-mode (`STROKE_SELECT`) provides the first interactive sub-mode in Monospace,
establishes the Option D unified context table architecture (Phase 5 of the contextual functions
plan), and allows users to change shape stroke styles with direct numeric keys `1..=5`.

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
  - In `STROKE_SELECT`: command row displays
    `STROKE: [1] light ┌─┐ [2] double ╔═╗ [3] light-round ╭─╮ · [4] heavy ┏━┓ [5] ascii +-+ · [Esc] Cancel`,
    and status row displays `STROKE SELECTION for <Kind> #<id> · Current: <current_stroke>`.
- Unified context table (Option D): single source of truth for contextual key bindings, visual
  labels, and actions shared by status bar formatting and widget key dispatching.
- Guardian tests in `diagram_widget.rs` updated to verify offer == acceptance across all contexts.

### Out

- Multi-line wrap and dynamic dock expansion when bar width exceeds terminal columns (deferred to
  [#280](https://github.com/andresmoschini/monospace/issues/280) / Phase 4).
- Custom stroke names beyond the five catalog presets.
- Fill sub-mode (`f`), layer order sub-mode (`z`), connector endpoint editing (`1`/`2`), and text
  editing (`Enter`), which have their own future issues in Phase 6.
- Multi-figure stroke batch modification (selection is strictly single-figure).

## The decision

**D1 — ephemeral sub-mode modeling and transitions.**

**Answer:** `DiagramState` holds an optional sub-mode state `sub_mode: Option<SubMode>` where
`SubMode::StrokeSelect` represents active stroke selection. Pressing `s` while a shape is selected
transitions `sub_mode` to `Some(SubMode::StrokeSelect)` and returns `Reply::Repaint`. While in
`StrokeSelect`, keys `1..=5` dispatch `Action::SetStroke(stroke)`, reset `sub_mode` to `None`
(returning to the figure's selection state), and return `Reply::Act(...)`; `Esc` resets `sub_mode`
to `None` without modifying the diagram, leaving the figure selected, and returns `Reply::Repaint`;
and all other keys (arrows, `Shift+arrows`, `Delete`, other characters) return `Reply::Ignored`.
Deselecting or clicking outside resets `sub_mode` to `None`. **Why not** holding the modal state in
`AppState` or a modal overlay: `STROKE_SELECT` is an inline interaction mode of the diagram region
where the canvas remains visible and interactive; storing it in `AppState` would decouple it from
`DiagramState::selected` and require redundant synchronization across modules. **Why not**
deselecting on `Esc`: `Esc` in a sub-mode cancels the sub-mode back to the enclosing selection,
requiring a second `Esc` to drop selection, ensuring consistent non-destructive escape semantics.
**Answered by** the maintainer.

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

**D3 — contextual bar presentation and unified context table (Option D).**

**Answer:** The context bar and the widget event dispatcher derive their offered keys, labels, and
actions from a unified context table (Option D). In `SELECTED_BOX`, `SELECTED_LINE`, and
`SELECTED_CONNECTOR`, the command row displays `STROKE: [s]`. In `STROKE_SELECT`, the command row
displays
`STROKE: [1] light ┌─┐ [2] double ╔═╗ [3] light-round ╭─╮ · [4] heavy ┏━┓ [5] ascii +-+ · [Esc] Cancel`,
and the status row reports `STROKE SELECTION for <Kind> #<id> · Current: <current_stroke>`.
`status::command_line` formats the labels from the active context's table entries, and
`diagram_widget::key` dispatches matching keys from the same entries. **Why not** continuing
separate `match` expressions in `status.rs` and `diagram_widget.rs`: duplicates context logic and
causes divergence between what is offered and what is accepted (the smell identified in #288). **Why
not** a modal overlay dialog: direct key selection in the bottom bar is faster, maintains visual
context of the drawing, and avoids occlusion. **Why not** cycling strokes directly with `s`: cycling
prevents direct choice, conceals available options, and clutters undo history. **Answered by** the
maintainer.

## Model slice

- `docs/diagram-model.md` §4: document `Shape::with_stroke`, which replaces the stroke of `Box`,
  `Line`, or `Connector` while preserving all other properties.
- `docs/editing-model.md` §2 and §5: document `Command::SetStroke`, which changes the stroke of a
  figure by its `ShapeId`, and note that setting the same stroke produces `None` and records no step
  in history (#285).
- `docs/application-model.md` §4 and §7: document `STROKE_SELECT` sub-mode, `s` key activation,
  direct number keys `1..=5`, `Esc` cancellation, and the Option D context table binding offered
  commands to accepted keys.

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
7. While in `STROKE_SELECT`, all other keys (arrows, `Shift+arrows`, letters, `Delete`) are ignored
   (`Reply::Ignored`).
8. In `SELECTED_BOX`, `SELECTED_LINE`, and `SELECTED_CONNECTOR`, the contextual command row displays
   `STROKE: [s]`.
9. In `STROKE_SELECT`, the contextual command row displays
   `STROKE: [1] light ┌─┐ [2] double ╔═╗ [3] light-round ╭─╮ · [4] heavy ┏━┓ [5] ascii +-+ · [Esc] Cancel`,
   and the status row reports `STROKE SELECTION for <Kind> #<id> · Current: <current_stroke>`.
10. The set of contextual keys accepted by the region and the set of keys offered in the contextual
    command row are derived from a unified context table, satisfying the Representation Rule across
    all contexts (`IDLE`, `SELECTED_BOX`, `SELECTED_LINE`, `SELECTED_CONNECTOR`, `STROKE_SELECT`).

## Examples

Hypothetical: none of it has been run; it is what the acceptance list checks, not the record of a
run.

| Initial Context         | Key Pressed    | Sub-mode After  | Action Dispatched     | Diagram Changed? | Step Recorded?     |
| ----------------------- | -------------- | --------------- | --------------------- | ---------------- | ------------------ |
| `SELECTED_BOX` (light)  | `s`            | `StrokeSelect`  | none (`Repaint`)      | no               | no                 |
| `STROKE_SELECT` (light) | `2` (`double`) | `None` (box)    | `SetStroke("double")` | yes              | yes (1 step)       |
| `STROKE_SELECT` (light) | `1` (`light`)  | `None` (box)    | `SetStroke("light")`  | no               | no (0 steps, #285) |
| `STROKE_SELECT` (light) | `Esc`          | `None` (box)    | none (`Repaint`)      | no               | no                 |
| `STROKE_SELECT` (light) | `↑`            | `StrokeSelect`  | none (`Ignored`)      | no               | no                 |
| `IDLE`                  | `s`            | `None` (`IDLE`) | none (`Ignored`)      | no               | no                 |

| Context                  | Status Row (Row 0)                                                            | Command Row (Row 1)                                                                                     |
| ------------------------ | ----------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------- |
| `SELECTED_BOX` #1        | `Box #1 · Pos: (2, 2) · Size: 10×4 · Stroke: light · Fill: None`              | `MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · STROKE: [s] · [Del] Delete · [Esc] Deselect`                         |
| `SELECTED_LINE` #2       | `Line #2 · Pos: (0, 0) · Length: 6 · Orientation: Horizontal · Stroke: light` | `MOVE: ↑↓←→ · STROKE: [s] · [Del] Delete · [Esc] Deselect`                                              |
| `SELECTED_CONNECTOR` #3  | `Connector #3 · Stroke: light · 1. From: ... · 2. To: ...`                    | `MOVE: ↑↓←→ · STROKE: [s] · [Del] Delete · [Esc] Deselect`                                              |
| `STROKE_SELECT` (Box #1) | `STROKE SELECTION for Box #1 · Current: light`                                | `STROKE: [1] light ┌─┐ [2] double ╔═╗ [3] light-round ╭─╮ · [4] heavy ┏━┓ [5] ascii +-+ · [Esc] Cancel` |

## What proves it

| Rule | Test                                                                    |
| ---- | ----------------------------------------------------------------------- |
| 1    | `pressing_s_on_selected_figure_enters_stroke_select_sub_mode`           |
| 2    | `pressing_s_in_idle_is_ignored`                                         |
| 3    | `pressing_1_to_5_in_stroke_select_applies_stroke_and_exits_sub_mode`    |
| 4    | `setting_identical_stroke_leaves_diagram_untouched_and_records_no_step` |
| 5    | `setting_different_stroke_records_one_step_in_session_history`          |
| 6    | `pressing_escape_in_stroke_select_cancels_sub_mode_without_deselecting` |
| 7    | `unoffered_keys_in_stroke_select_are_ignored`                           |
| 8    | `command_row_offers_stroke_s_for_selected_box_line_and_connector`       |
| 9    | `stroke_select_context_formats_prompt_and_status_row`                   |
| 10   | `context_table_guarantees_offer_equals_acceptance_across_all_contexts`  |

## Open questions

None.
