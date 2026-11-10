---
status: draft
date: 2026-11-10
---

# Rotate a selected line with an r key

## Why now

In Monospace, a `Line` is defined by a position `at`, a length `len`, an `orientation`
(`Orientation::Horizontal` or `Orientation::Vertical`), and a `stroke`
([`docs/model.md`](../docs/model.md) §7).

Issue [#308](https://github.com/andresmoschini/monospace/issues/308) introduced keyboard creation of
shapes from `IDLE`, where pressing `l` creates a vertical line of length 5 at the viewport center.
Issue [#301](https://github.com/andresmoschini/monospace/issues/301) introduced interactive line
resizing via `Shift+←→` (for horizontal lines) and `Shift+↑↓` (for vertical lines), explicitly
excluding orientation changes from resize operations.

As a result, once a line is created from the keyboard—or when an existing line in a diagram needs to
change between horizontal and vertical—the interactive TUI currently provides no mechanism to change
its orientation. Adding a direct `ROTATE: [r]` action when a line is selected (`SELECTED_LINE`)
completes the interactive manipulation vocabulary for lines in a single atomic keystroke, without
introducing a sub-mode.

## Scope

### In

- A contextual action `ROTATE: [r]` offered in the command row (row 1) of the contextual status bar
  when a line is selected (`SELECTED_LINE`), placed between `RESIZE` and `STROKE: [s]`.
- Domain mutation method on `Shape`: `pub fn with_rotated_orientation(&self) -> Self` in
  `monospace-diagram`.
- Atomic command in `monospace-editing`: `Command::RotateLine { id: ShapeId }`, mapping to
  `Diagram::replace` with `Shape::with_rotated_orientation()`.
- Rotation semantics:
  - Toggles `orientation` between `Orientation::Horizontal` and `Orientation::Vertical`.
  - Preserves `at`, `len`, and `stroke` unchanged (pivoting around the line's origin `at`).
  - Leaves non-line shapes (`Shape::Box`, `Shape::Connector`) unchanged, returning `None` in
    `Command::apply_to` and recording no step (#285).
  - Preserves any connector `Reference` attached to the line, which dynamically resolves against the
    line's updated anchor coordinates.
- Single-step undo history: rotating a line records one step in `Session`; `Undo` restores the prior
  orientation in a single step.
- Contextual bar and dispatch updates in `crates/monospace`:
  - `Condition::Line` added to `crates/monospace/src/context_table.rs` (active when the selected
    shape is either a horizontal or vertical line).
  - `SELECTION` table includes `ContextBlock` for `ROTATE: [r]` under `Condition::Line`.
  - Key `'r'` (`KeyCode::Char('r'), KeyModifiers::NONE`) added to `ALL_POSSIBLE_CONTEXT_KEYS` and
    dispatched as `Action::Command(Command::RotateLine { id })` when a line is selected.
  - Status row (row 0) immediately reflects the updated `Orientation: Horizontal` or
    `Orientation: Vertical`, and command row (row 1) updates the orientation-gated `RESIZE` hint
    (`Shift+←→` vs. `Shift+↑↓`).

### Out

- Rotating `Shape::Box` (boxes already support independent width and height resizing via
  `Shift+↑↓←→`) or `Shape::Connector` (connectors route between endpoints and support terminal
  inversion via `i`).
- Rotating a line around its midpoint or trailing endpoint.
- Changing line orientation via `Shift` + arrow keys (`Command::Resize`).

## The decision

**D1 — pivoting at the line's origin `at` vs. rotating around its midpoint.**

**Answer:** Rotating a line toggles `orientation` between `Orientation::Horizontal` and
`Orientation::Vertical` while keeping `at`, `len`, and `stroke` unchanged.

A vertical line of length 4 at `(2, 1)` rotated around `at` becomes a horizontal line of length 4 at
`(2, 1)`:

<!-- render: 8x6
{
  "next_id": 2,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": 2, "y": 1 }, "len": 4, "orientation": "vertical", "stroke": "light" }
  ]
}
-->

```text

  │
  │
  │
  │

```

<!-- /render -->

<!-- render: 8x6
{
  "next_id": 2,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": 2, "y": 1 }, "len": 4, "orientation": "horizontal", "stroke": "light" }
  ]
}
-->

```text

  ────




```

<!-- /render -->

By contrast, rotating the same line around its midpoint would shift `at` to `(1, 2)`:

<!-- render: 8x6
{
  "next_id": 2,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": 1, "y": 2 }, "len": 4, "orientation": "horizontal", "stroke": "light" }
  ]
}
-->

```text


 ────



```

<!-- /render -->

**Why not** rotating around the line's midpoint: first, for lines of even length (`len = 4`), a grid
of discrete terminal cells has no exact center cell, so integer half-length division either drifts
across repeated rotations or requires asymmetric rounding rules, whereas pivoting at `at` is a
strict involution for every `len`
(`shape.with_rotated_orientation().with_rotated_orientation() == shape`). Second, rotating a
horizontal line near the top border (for example, `at = (5, 0)`, `len = 5`) around its midpoint
would move `at.y` to `-2`, pushing part of the line into negative coordinates outside the initial
viewport. Third, keeping `at` anchored during non-move transformations is consistent with
`Shape::resized_by` (#284, #301), where `Pos: (x, y)` in the status row remains invariant unless the
user explicitly moves the shape with `↑↓←→`.

**D2 — keybinding and contextual label (`ROTATE: [r]` vs. `ORIENTATION: [o]`).**

**Answer:** Offer `ROTATE: [r]` bound to unmodified `r` in `SELECTED_LINE`, placed immediately after
`RESIZE` and before `STROKE: [s]`.

**Why not** `ORIENTATION: [o]`: `ROTATE` is shorter in the 80-column command row and uses a verb
matching `MOVE`, `RESIZE`, and `INVERT`. Unmodified `r` is the direct mnemonic for rotation and is
unused in `SELECTED_LINE` (`Redo` uses `Ctrl+r`).

**D3 — dedicated atomic command `Command::RotateLine` vs. overloading `Command::Resize`.**

**Answer:** `monospace-editing` adds `Command::RotateLine { id: ShapeId }`, which applies
`Diagram::replace(id, shape.with_rotated_orientation())`.

**Why not** overloading `Command::Resize` with orthogonal deltas: #301 explicitly excluded
orientation flips from `Shift+arrow` resizing so that pressing an orthogonal arrow during resize
cannot accidentally flip the line. Rotation is a discrete toggle of `Orientation` rather than a
delta along an axis, warranting its own command variant analogous to `Command::InvertTerminals`.

**D4 — preserving connector attachments when a line rotates.**

**Answer:** `Diagram::replace` leaves any connector endpoint referencing the rotated line
(`EndpointAt::Reference`) unchanged; the connector dynamically resolves the target `Anchor` (`Top`,
`Right`, `Bottom`, `Left`) against the rotated line's new geometry.

**Why not** detaching connectors or mutating their `Anchor` on rotation: `Shape::Line` answers all
four side anchors in both `Horizontal` and `Vertical` orientations via
`flat_size(len, orientation)`, and keeping `Reference` untouched is how `Move` and `Resize` already
operate in `monospace-diagram`.

## Model slice

- `docs/diagram-model.md` (_Changing a diagram_): Document `with_rotated_orientation` on `Shape`,
  which toggles `orientation` on a `Line` while preserving `at`, `len`, and `stroke`, and leaves a
  `Box` or `Connector` unchanged.
- `docs/editing-model.md` (_The initial set_): Add `Command::RotateLine { id }` to the command
  table, mapping to `replace` with the line rotated.
- `docs/application-model.md` (_Commands_ / _Contextual table and dispatch_): Document `ROTATE: [r]`
  in `SELECTED_LINE`.
- `docs/contextual-ui.md` (_4.3 SELECTED_LINE_): Include `ROTATE: [r]` between `RESIZE` and
  `STROKE: [s]` in the horizontal and vertical line command rows.

## Public surface

- `monospace-diagram`:
  - `impl Shape`:
    - `pub fn with_rotated_orientation(&self) -> Self`: toggles `orientation` between
      `Orientation::Horizontal` and `Orientation::Vertical` if `self` is a `Shape::Line`; otherwise
      returns `self.clone()`.
- `monospace-editing`:
  - `pub enum Command`:
    - `RotateLine { id: ShapeId }`
- `monospace`:
  - `context_table`:
    - `Condition::Line` variant (true when the selected shape is a horizontal or vertical line).
    - `ContextMeaning::RotateLine` variant (or `Action::Command` dispatch) and `SELECTION` block
      offering `ROTATE: [r]` under `Condition::Line`.
  - `diagram_widget`:
    - `(KeyCode::Char('r'), KeyModifiers::NONE)` added to `ALL_POSSIBLE_CONTEXT_KEYS`.

## Behavior

1. When a horizontal line is selected (`SELECTED_LINE`), the command row displays:
   `MOVE: ↑↓←→ · RESIZE: Shift+←→ · ROTATE: [r] · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect`
   (omitting `[Tab] Next [Shift+Tab] Prev` when the diagram contains fewer than two shapes).
2. When a vertical line is selected (`SELECTED_LINE`), the command row displays:
   `MOVE: ↑↓←→ · RESIZE: Shift+↑↓ · ROTATE: [r] · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [Del] Delete · [Esc] Deselect`.
3. Pressing `r` when a line is selected dispatches `Command::RotateLine { id }` for the selected
   line's `ShapeId`.
4. Rotating a horizontal line changes its `orientation` to `Orientation::Vertical`; rotating a
   vertical line changes its `orientation` to `Orientation::Horizontal`.
5. Rotating a line preserves its `at`, `len`, and `stroke`, keeps the line selected, and records
   exactly one step in `Session`.
6. Pressing `r` a second time returns the line to its original orientation and records a second
   step; pressing `Undo` (`Ctrl+z`) reverts a rotation in a single step.
7. After rotating a line, the status row (row 0) updates `Orientation: Horizontal` ↔
   `Orientation: Vertical`, and the command row (row 1) updates `RESIZE: Shift+←→` ↔
   `RESIZE: Shift+↑↓` to match the new orientation.
8. `Command::RotateLine { id }` applied to a `Box`, a `Connector`, or a nonexistent `ShapeId` leaves
   the diagram unchanged, returns `None` from `Command::apply_to`, and records no step (#285).
9. Pressing `r` when a box or connector is selected, when no shape is selected (`IDLE`), or inside a
   sub-mode (`STROKE_SELECT`, `ORDER_MODE`, `TEXT_EDIT`, `ENDPOINT_EDIT`) is not accepted as line
   rotation.

## Examples

**1. Rotating a newly created vertical line to horizontal.** Pressing `l` in `IDLE` creates a
vertical line of length 5 at `(1, 1)`. Pressing `r` dispatches `Command::RotateLine { id: 1 }`,
turning it into a horizontal line of length 5 at `(1, 1)`.

<!-- render: 8x7
{
  "next_id": 2,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": 1, "y": 1 }, "len": 5, "orientation": "vertical", "stroke": "light" }
  ]
}
-->

```text

 │
 │
 │
 │
 │

```

<!-- /render -->

<!-- render: 8x7
{
  "next_id": 2,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": 1, "y": 1 }, "len": 5, "orientation": "horizontal", "stroke": "light" }
  ]
}
-->

```text

 ─────





```

<!-- /render -->

- Before `r`:
  - Status row: `Line #1 · Pos: (1, 1) · Length: 5 · Orientation: Vertical · Stroke: light`
  - Command row:
    `MOVE: ↑↓←→ · RESIZE: Shift+↑↓ · ROTATE: [r] · STROKE: [s] · ORDER: [z] · [Del] Delete · [Esc] Deselect`
- After `r`:
  - Status row: `Line #1 · Pos: (1, 1) · Length: 5 · Orientation: Horizontal · Stroke: light`
  - Command row:
    `MOVE: ↑↓←→ · RESIZE: Shift+←→ · ROTATE: [r] · STROKE: [s] · ORDER: [z] · [Del] Delete · [Esc] Deselect`

**2. Rotating a horizontal line with an attached connector.** Line 1 at `(1, 1)` of length 5 is
connected from its `Right` anchor to Box 2 at `(9, 0)`. Rotating Line 1 to `Vertical` keeps the
connector attached to Line 1's `Right` anchor (`(1, 3)`):

<!-- render: 16x7
{
  "next_id": 3,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": 1, "y": 1 }, "len": 5, "orientation": "horizontal", "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 9, "y": 0 }, "size": { "width": 6, "height": 3 }, "stroke": "light", "text": "Node" },
    {
      "kind": "connector",
      "id": 3,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" }, "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "reference", "shape": 2, "anchor": "left" }, "terminal": { "kind": "symbol", "style": "solid" } },
      "stroke": "light"
    }
  ]
}
-->

```text
         ┌────┐
 ───────▶│Node│
         └────┘




```

<!-- /render -->

<!-- render: 16x7
{
  "next_id": 3,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": 1, "y": 1 }, "len": 5, "orientation": "vertical", "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 9, "y": 0 }, "size": { "width": 6, "height": 3 }, "stroke": "light", "text": "Node" },
    {
      "kind": "connector",
      "id": 3,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" }, "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "reference", "shape": 2, "anchor": "left" }, "terminal": { "kind": "symbol", "style": "solid" } },
      "stroke": "light"
    }
  ]
}
-->

```text
         ┌────┐
 │  ┌───▶│Node│
 │  │    └────┘
 ├──┘
 │
 │

```

<!-- /render -->

## What proves it

| Rule | Test                                                                                                                                                                                                                                                                       |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1, 2 | `status::tests::the_command_row_offers_resize_for_a_box_and_a_line`, `status::tests::selected_line_shows_identity_pos_length_orientation_stroke_and_selection_commands`, `status::tests::selected_vertical_line_offers_the_vertical_resize`                                |
| 3, 9 | `diagram_widget::tests::key_acceptance_matches_contextual_bar_for_selected_horizontal_line`, `diagram_widget::tests::key_acceptance_matches_contextual_bar_for_selected_vertical_line`, `diagram_widget::tests::pressing_r_rotates_selected_line_and_is_ignored_otherwise` |
| 4, 5 | `shape::tests::with_rotated_orientation_toggles_a_line_and_keeps_position_length_and_stroke`, `contract::rotating_a_line_toggles_its_orientation_and_records_one_step`                                                                                                     |
| 6, 7 | `callbacks::tests::pressing_r_on_selected_line_toggles_orientation_and_updates_status`                                                                                                                                                                                     |
| 8    | `shape::tests::with_rotated_orientation_on_a_box_or_connector_changes_nothing`, `contract::rotating_a_box_connector_or_nonexistent_shape_records_no_step`                                                                                                                  |

## Open questions

_None._
