---
status: draft
date: 2026-11-10
---

# Duplicate the selected shape with a d key

## Why now

Issue [#308](https://github.com/andresmoschini/monospace/issues/308) added keyboard creation of
default shapes from `IDLE` (`[b] Box [l] Line [c] Connector`), and subsequent slices added direct
contextual mutations for a selected shape: resizing (#284, #301), stroke selection (`[s]`), text
editing (`[t]`), terminal inversion (`[i]`), z-order adjustment (`[z]`), line rotation (`[r]`,
[#346](https://github.com/andresmoschini/monospace/issues/346)), and deletion (`[Del]`, #178).

However, once a user has configured a box, line, or connector—with its size, length, orientation,
stroke, text, or endpoint bindings—there is no way to duplicate that shape without recreating it
from scratch and reapplying every property by hand. Adding a single-key `[d] Duplicate` action
whenever any shape is selected (`SELECTED_BOX`, `SELECTED_LINE`, `SELECTED_CONNECTOR`) completes the
selection manipulation vocabulary across all three shape kinds.

## Scope

### In

- A contextual action `[d] Duplicate` offered in the command row (row 1) of the contextual status
  bar whenever a box, line, or connector is selected (`SELECTED_BOX`, `SELECTED_LINE`,
  `SELECTED_CONNECTOR`), placed immediately before `[Del] Delete · [Esc] Deselect`.
- Duplication of any selected shape (`Shape::Box`, `Shape::Line`, `Shape::Connector`) by cloning the
  selected `Shape` value verbatim (`Shape::clone`) and adding it to the session via `Session::add`
  (`Command::Add { shape }`).
- Exact property preservation on duplication, including position (`at` or endpoints `from`/`to`),
  dimensions (`size` or `len`), `orientation`, `stroke`, `fill`, `text`, and connector endpoint
  configurations (`Terminal` and `EndpointAt::Absolute` or `EndpointAt::Reference`).
- Immediate selection transfer to the newly created duplicate (`ShapeId`), which `Diagram::add`
  places at the front of the drawing order (`to_front`).
- Single-step undo and redo via the existing `Command::Add` history in `Session`.
- Contextual bar and key-dispatch updates in `crates/monospace` (`context_table`, `diagram_state`,
  `diagram_widget`).

### Out

- Automatic spatial displacement (offsetting the duplicate's position on creation).
- Multi-shape selection or duplicating multiple shapes at once.
- System clipboard integration (`Ctrl+c` / `Ctrl+v`) or cross-session copy/paste.
- Automatically duplicating or re-targeting third-party connectors that reference a box or line when
  that box or line is duplicated.

## The decision

**D1 — Contextual keybinding `[d] Duplicate` vs. global `Ctrl+d`.**

**Answer:** Bind shape duplication to unmodified lowercase `d` in the selection context table
(`SELECTION`), rendered as `[d] Duplicate` immediately before `[Del] Delete · [Esc] Deselect` in
`SELECTED_BOX`, `SELECTED_LINE`, and `SELECTED_CONNECTOR`.

**Why not** `Ctrl+d` in the global shortcut bar: every other single-shape action (`[r]` rotate,
`[t]` text, `[i]` invert, `[s]` stroke, `[z]` order, `[Del]` delete) lives in the contextual bar and
uses an unmodified key active only while a shape is selected. Placing `[d] Duplicate` in `SELECTION`
makes the action discoverable on the command row alongside `[Del] Delete` and keeps global `Ctrl+`
bindings reserved for application-wide commands (`Quit`, `Undo`, `Redo`).

**D2 — In-place duplication (`Shape::clone`) vs. automatic displacement offset.**

**Answer:** Duplicate the selected shape at the exact same position with all properties copied as
they stand (`shape.clone()`), placing the new shape at the front of the drawing order.

Duplicating Box 1 at `(1, 1)` produces Box 2 at `(1, 1)` with identical geometry, stroke, and text,
superimposed at the front of the drawing order until the user moves it:

<!-- render: 16x6
{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 1, "y": 1 }, "size": { "width": 6, "height": 3 }, "stroke": "light", "text": "Node" },
    { "kind": "box", "id": 2, "at": { "x": 1, "y": 1 }, "size": { "width": 6, "height": 3 }, "stroke": "light", "text": "Node" }
  ]
}
-->

```text

 ┌────┐
 │Node│
 └────┘


```

<!-- /render -->

By contrast, applying an automatic offset such as `Delta { dx: 2, dy: 1 }` would shift the duplicate
diagonal to `(3, 2)`:

<!-- render: 16x6
{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 1, "y": 1 }, "size": { "width": 6, "height": 3 }, "stroke": "light", "text": "Node" },
    { "kind": "box", "id": 2, "at": { "x": 3, "y": 2 }, "size": { "width": 6, "height": 3 }, "stroke": "light", "text": "Node" }
  ]
}
-->

```text

 ┌────┐
 │N┌──┼─┐
 └─┼Node│
   └────┘

```

<!-- /render -->

**Why not** applying an automatic displacement offset: first, an arbitrary offset direction (`+x`,
`+y`) does not know whether the user intends to align the copy horizontally, vertically, or to the
left/top, forcing extra corrective keystrokes to undo the unwanted axis offset. Second, on an
attached connector (`EndpointAt::Reference`), `Shape::displaced_by` grows `reference.offset`, which
would silently mutate the connector's anchor offset rather than copying its properties verbatim.
Keeping duplication a pure in-place clone makes it uniform across all three shape types.

**D3 — Selecting the new duplicate vs. keeping the original shape selected.**

**Answer:** Immediately select the newly created duplicate (`Some(new_id)`).

**Why not** keeping selection on the original shape: `Diagram::add` places the duplicate at the
front of the drawing order (`to_front`). Selecting the new shape immediately updates the status row
identity (for example, `Box #1` → `Box #2`) to confirm the duplicate was created, and ensures the
next arrow-key move or property edit operates on the top-most copy rather than moving the hidden
original underneath it. This also matches `DiagramState::add_shape` when creating shapes via `b`,
`l`, or `c` (#308).

**D4 — Connector attachment semantics when duplicating connectors vs. boxes or lines.**

**Answer:** Duplication clones the selected `Shape` value verbatim (`Shape::clone`). When a
connector with `EndpointAt::Reference` endpoints is duplicated, the duplicate connector retains
those exact `Reference` values (pointing to the same target `ShapeId`, `Anchor`, `offset`, and
`out`). When a box or line is duplicated, existing connectors attached to the original box or line
remain attached only to the original.

**Why not** detaching a duplicated connector's endpoints to `EndpointAt::Absolute`, or duplicating
inbound connectors when a box or line is duplicated: detaching would discard binding properties on
the connector, contradicting exact property duplication, while duplicating unselected connectors
when a box or line is duplicated would create extra shapes outside the single-shape selection.

**D5 — Reusing `Command::Add` via `Session::add` vs. adding `Command::Duplicate`.**

**Answer:** Implement duplication in
`DiagramState::duplicate_selected(&mut self) -> Option<ShapeId>` by cloning `self.selected_shape()?`
and delegating to `self.add_shape(shape)`, which calls `Session::add(shape)` and records
`Command::Add { shape }`.

**Why not** introducing a separate `Command::Duplicate { id: ShapeId }` variant in
`monospace-editing`: `Command::Add { shape }` already inserts a `Shape` at the front of the drawing
order, assigns the next `ShapeId`, and records a single step in `Session`. A second command variant
with identical diagram transition semantics would duplicate code across `monospace-editing` without
adding new domain behavior.

## Model slice

- `docs/application-model.md` (_3. The state_ / _4. The action model_ / _7. Properties worth
  testing_): Document `Duplicate` (`[d] Duplicate`) as a contextual selection action that clones the
  selected shape via `Session::add` (`Command::Add`) and selects the new `ShapeId`.
- `docs/contextual-ui.md` (_2. Screen layout and contextual bar grammar_ / _4.2 SELECTED_BOX_ / _4.3
  SELECTED_LINE_ / _4.9 Connector states summary_ / _5. Three-stage implementation plan_): Include
  `[d] Duplicate` immediately before `[Del] Delete · [Esc] Deselect` in `SELECTED_BOX`,
  `SELECTED_LINE`, and `SELECTED_CONNECTOR`.

## Public surface

- `monospace`:
  - `diagram_state`:
    - `DiagramState::duplicate_selected(&mut self) -> Option<ShapeId>`: clones the currently
      selected `Shape`, adds it to the session via `self.add_shape(shape)` (which selects the new
      `ShapeId`), and returns `Some(new_id)`, or `None` if no shape is selected.
  - `context_table`:
    - `ContextMeaning::Duplicate` variant, dispatched in `execute_meaning` by calling
      `state.duplicate_selected()`.
    - `SELECTION` table entry offering `[d] Duplicate` (`KeyPattern::Char('d')`,
      `Condition::Always`) placed immediately before `[Del] Delete`.
  - `diagram_widget`:
    - `(KeyCode::Char('d'), KeyModifiers::NONE)` included in contextual key acceptance tests across
      all selection states.

## Behavior

1. When a box is selected (`SELECTED_BOX`), the command row includes `[d] Duplicate` immediately
   before `[Del] Delete · [Esc] Deselect`.
2. When a horizontal or vertical line is selected (`SELECTED_LINE`), the command row includes
   `[d] Duplicate` immediately before `[Del] Delete · [Esc] Deselect`.
3. When a connector is selected (`SELECTED_CONNECTOR`), the command row includes `[d] Duplicate`
   immediately before `[Del] Delete · [Esc] Deselect`.
4. Pressing `d` when a shape is selected adds an exact clone (`Shape::clone`) of that shape to the
   diagram via `Session::add`, placing the new shape at the front of the drawing order with a fresh
   `ShapeId`.
5. Duplicating a shape preserves all of its properties unchanged (`at`, `size`, `stroke`, `fill`,
   and `text` for a `Box`; `at`, `len`, `orientation`, and `stroke` for a `Line`; `from`, `to`
   including any `EndpointAt::Reference` or `EndpointAt::Absolute` and `Terminal`, and `stroke` for
   a `Connector`).
6. Duplicating a box or line leaves any connectors referencing the original shape's `ShapeId`
   attached to the original shape; the new duplicate has no connectors referencing its new
   `ShapeId`.
7. Immediately after pressing `d`, selection moves from the original `ShapeId` to the newly created
   duplicate's `ShapeId`, and the status row (row 0) updates to show the new identity (`#<new_id>`).
8. When a diagram has one shape selected (so `[Tab] Next [Shift+Tab] Prev` is omitted because
   `shape_count < 2`), pressing `d` increases `shape_count` to `2` and the command row immediately
   includes `[Tab] Next [Shift+Tab] Prev`.
9. Duplicating a shape records exactly one step in `Session`; pressing `Undo` (`Ctrl+z`) removes the
   duplicated shape and reconciles selection in a single step, and `Redo` (`Ctrl+r`) restores the
   duplicated shape.
10. Pressing `d` in `IDLE` or inside any sub-mode (`STROKE_SELECT`, `ORDER_MODE`, `TEXT_EDIT`,
    `ENDPOINT_EDIT`) does not trigger duplication and is ignored by selection dispatch (or typed as
    a character in `TEXT_EDIT`).

## Examples

**1. Duplicating a selected box and moving the duplicate.** Box 1 (`"API"`, double stroke, `6x3` at
`(1, 1)`) is selected in a single-shape diagram. Pressing `d` creates Box 2 with identical
properties at `(1, 1)` and selects Box 2; pressing `→` eight times moves Box 2 to `(9, 1)` while Box
1 remains at `(1, 1)`:

<!-- render: 18x5
{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 1, "y": 1 }, "size": { "width": 6, "height": 3 }, "stroke": "double", "text": "API" },
    { "kind": "box", "id": 2, "at": { "x": 9, "y": 1 }, "size": { "width": 6, "height": 3 }, "stroke": "double", "text": "API" }
  ]
}
-->

```text

 ╔════╗  ╔════╗
 ║API ║  ║API ║
 ╚════╝  ╚════╝

```

<!-- /render -->

- Before `d`:
  - Status row: `Box #1 · Pos: (1, 1) · Size: 6×3 · Stroke: double · Text: "API"`
  - Command row:
    `MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · TEXT: [t] Edit · STROKE: [s] · ORDER: [z] · [d] Duplicate · [Del] Delete · [Esc] Deselect`
- Immediately after `d`:
  - Status row: `Box #2 · Pos: (1, 1) · Size: 6×3 · Stroke: double · Text: "API"`
  - Command row:
    `MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · TEXT: [t] Edit · STROKE: [s] · ORDER: [z] · [Tab] Next [Shift+Tab] Prev · [d] Duplicate · [Del] Delete · [Esc] Deselect`

## What proves it

| Rule        | Test                                                                                                                                                                                                                                                                                                                                                           |
| ----------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1, 2, 3     | `status::tests::selected_box_shows_identity_pos_size_stroke_text_and_selection_commands`, `status::tests::selected_line_shows_identity_pos_length_orientation_stroke_and_selection_commands`, `status::tests::selected_connector_shows_identity_endpoints_stroke_and_selection_commands`                                                                       |
| 4, 5, 6     | `diagram_state::tests::duplicate_selected_clones_box_line_and_connector_properties_in_place`                                                                                                                                                                                                                                                                   |
| 7, 8, 9     | `callbacks::tests::pressing_d_duplicates_selected_shape_selects_new_id_and_undoes_in_one_step`                                                                                                                                                                                                                                                                 |
| 1, 2, 3, 10 | `diagram_widget::tests::key_acceptance_matches_contextual_bar_for_selected_box`, `diagram_widget::tests::key_acceptance_matches_contextual_bar_for_selected_horizontal_line`, `diagram_widget::tests::key_acceptance_matches_contextual_bar_for_selected_vertical_line`, `diagram_widget::tests::key_acceptance_matches_contextual_bar_for_selected_connector` |

## Open questions

_None._
