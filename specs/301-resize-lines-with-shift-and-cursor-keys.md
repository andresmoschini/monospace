---
status: implemented
decided: "#302"
implemented: "#303"
date: 2026-11-04
---

# Resize lines with Shift and cursor keys

## Why now

Issue [#284](https://github.com/andresmoschini/monospace/issues/284) introduced box resizing via
`Shift` and arrow keys (`Shift+↑↓←→`) and defined `Command::Resize { id, by: Delta }`. Its scope
explicitly left line resizing for its own issue:

> "Resizing a line. Its own issue: the length, not a width and a height, and a minimum of its own.
> Until then Shift+arrow with a line or a connector selected draws nothing new."

With [#285](https://github.com/andresmoschini/monospace/issues/285) ensuring that commands leaving
the diagram unchanged record no history step, and
[#288](https://github.com/andresmoschini/monospace/issues/288) establishing the Representation Rule
("offer == acceptance"), the application is ready to support interactive resizing of lines.

Currently, selecting a line hides the `RESIZE` hint entirely and discards `Shift+arrow` presses.
Lines have a single degree of freedom (their length `len`), oriented either horizontally or
vertically. Enabling line resizing completes the interactive editing vocabulary for complete
geometric shapes before connectors are addressed.

## Scope

### In

- Resizing the selected `Line` with `Shift` and arrow keys along the line's orientation:
  - **Horizontal line**: `Shift+→` extends length by 1 cell, `Shift+←` shrinks length by 1 cell.
    `Shift+↑` and `Shift+↓` are inactive (no effect, not offered, no undo entry).
  - **Vertical line**: `Shift+↓` extends length by 1 cell, `Shift+↑` shrinks length by 1 cell.
    `Shift+←` and `Shift+→` are inactive (no effect, not offered, no undo entry).
- The origin `at` of the line remains fixed; resizing adjusts the line length from its trailing end.
- Line length clamping:
  - Minimum interactive length is **1**: a line cannot shrink below 1 cell (drawing its single point
    as an end), preventing figures from disappearing.
  - A line already at length 1 that receives a shrink command remains unchanged. Per #285, no
    history step is recorded.
  - Clamping follows `min(current, 1)..=u16::MAX`: if a diagram is loaded with a pre-existing
    zero-length line (`len == 0`), shrinking leaves it at 0 (producing no change and no history
    step), while extending it grows its length to 1.
- Contextual status and command bar updates:
  - Shows `RESIZE: Shift+←→` when a horizontal line is selected.
  - Shows `RESIZE: Shift+↑↓` when a vertical line is selected.
  - Preserves `RESIZE: Shift+↑↓←→` when a box is selected.
- Reuses the existing `Command::Resize { id, by: Delta }` in `monospace-editing` and updates
  `Shape::resized_by` in `monospace-diagram` to handle `Shape::Line`.

### Out

- **Resizing connectors (`Shape::Connector`).** Connectors have endpoints, routing, and reference
  attachments; their interactive modification belongs to connector editing issues.
- **Resizing from the start point (`at`).** Consistent with box resizing (anchored at `at`), the
  line resizes from its end. Moving the start point is achieved via `Move` (`↑↓←→`).
- **Changing line orientation via resize.** `Shift` arrows orthogonal to the line orientation are
  ignored.

## The decision

**D1 — how line resize is expressed in `Shape::resized_by`.**

**Answer:** `Shape::resized_by` handles `Shape::Line`:

- For `Orientation::Horizontal`, delta is taken from `by.dx`.
- For `Orientation::Vertical`, delta is taken from `by.dy`.
- Length `len` is adjusted by the delta, clamped to `min(current, 1)..=u16::MAX`.
- Origin `at`, `orientation`, and `stroke` remain untouched.
- Orthogonal deltas (e.g. `dy` on horizontal, `dx` on vertical) are ignored when computing `len`.

**Why not** a separate `Command::ResizeLine { id, delta: i16 }`: `Command::Resize { id, by: Delta }`
already exists in `monospace-editing` and represents resizing a figure along screen dimensions.
Reusing it keeps the editing command vocabulary compact and unified across shapes.

**Why not** moving `at` when shrinking or extending with opposite arrows: Boxes do not resize from
`at` (D1 & D2 of #284); keeping `at` anchored matches user expectation and preserves consistency
across the entire application.

**D2 — minimum line length and clamping behavior.**

**Answer:** Clamped via `clamp_dimension(current, delta)` with minimum bound `min(current, 1)`:

- For positive-length lines, the line does not shrink below **1** (`min(current, 1) == 1`).
- At `len == 1`, shrinking produces `len == 1` (no change). By #285, no history step is recorded.
- If a line loaded from an external description already has `len == 0`, shrinking produces 0 (no
  change, no history step), while extending by 1 produces `len == 1`.

**Why not** minimum length 2: In `monospace-core`, a line of length 1 is a valid shape that renders
a single terminal glyph end. Restricting to 2 would unnecessarily prevent single-cell lines.

**Why not** minimum length 0: At `len == 0`, a line renders nothing on the canvas and cannot be
clicked or selected interactively, leaving the user with an invisible shape that cannot be selected.

**D3 — contextual status bar offer and key acceptance.**

**Answer:** Extend the contextual table conditions or definitions to distinguish:

- Box: displays `RESIZE: Shift+↑↓←→` and accepts all 4 `Shift+arrow` keys.
- Horizontal Line: displays `RESIZE: Shift+←→` and accepts only `Shift+Left` and `Shift+Right`.
- Vertical Line: displays `RESIZE: Shift+↑↓` and accepts only `Shift+Up` and `Shift+Down`.

The contextual bar renders only the offered directions, and `dispatch_key` matches exactly the
corresponding subset of keys per the Representation Rule (#288).

**Why not** offering `Shift+↑↓←→` for lines and silently doing nothing on orthogonal keys: The
Representation Rule (#288) explicitly mandates "offer == acceptance". Offering keys that perform no
action violates the honest interface principle of Monospace.

## Model slice

- `docs/diagram-model.md` (_Changing a diagram_): Update the paragraph stating that resizing a line
  leaves the shape unchanged to describe line resizing along its orientation, clamping length to
  `min(current, 1)..=u16::MAX` and preserving `at`, orientation, and stroke.
- `docs/editing-model.md` (_The initial set_): Note that `Command::Resize` applies to both `Box`
  (width and height) and `Line` (length along orientation), while connectors remain unchanged.
- `docs/application-model.md` (_Movement and resize_): Describe line resizing via `Shift` plus
  cursor keys matching the line's orientation, and update contextual command line description to
  reflect orientation-specific arrows.

## Public surface

- `monospace-diagram`:
  - `Shape::resized_by` updates `Shape::Line` length.
  - Helper / predicate updates if needed (e.g. `Shape::can_resize` or line orientation queries).
- `monospace`:
  - `context_table`: updates to `Condition` / `SELECTION` table to distinguish box, horizontal line,
    and vertical line resize blocks and labels (`Shift+←→`, `Shift+↑↓`).

## Behavior

1. **Horizontal Line Selected:**
   - Command row displays `MOVE: ↑↓←→`, `RESIZE: Shift+←→`, `STROKE: [s]`, `[Del] Delete`.
   - Pressing `Shift+Right`: line length increases by 1 (`by = (1, 0)`).
   - Pressing `Shift+Left`: line length decreases by 1 (`by = (-1, 0)`).
   - Pressing `Shift+Up` or `Shift+Down`: ignored, no command dispatched, no history recorded.
2. **Vertical Line Selected:**
   - Command row displays `MOVE: ↑↓←→`, `RESIZE: Shift+↑↓`, `STROKE: [s]`, `[Del] Delete`.
   - Pressing `Shift+Down`: line length increases by 1 (`by = (0, 1)`).
   - Pressing `Shift+Up`: line length decreases by 1 (`by = (0, -1)`).
   - Pressing `Shift+Left` or `Shift+Right`: ignored, no command dispatched, no history recorded.
3. **Boundary Clamping:**
   - If a line has length 1 and is shrunk, the resulting length is 1; no history step is recorded.
   - If a line has length 0 and is shrunk, the resulting length is 0; no history step is recorded.
   - If a line has length 0 and is extended, the resulting length is 1.
4. **Preserved Attributes:**
   - Resizing preserves the line's origin `at`, its `orientation`, and its `stroke`.
5. **Undo / Redo:**
   - A valid resize step can be undone (`Ctrl+z`) and redone (`Ctrl+r`), restoring previous line
     lengths.

## Examples

**A horizontal line widened with `Shift+Right`.** A horizontal line of length 4, before and after
`Shift+Right`. One press, one `Command::Resize { id: 1, by: (1, 0) }`, one step.

<!-- render: 8x3
{
  "next_id": 2,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": 1, "y": 1 }, "len": 4, "orientation": "horizontal", "stroke": "light" }
  ]
}
-->

```text

 ────

```

<!-- /render -->

<!-- render: 8x3
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

**A vertical line heightened with `Shift+Down`.** A vertical line of length 3, before and after
`Shift+Down`. One press, one `Command::Resize { id: 1, by: (0, 1) }`, one step.

<!-- render: 4x5
{
  "next_id": 2,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": 1, "y": 0 }, "len": 3, "orientation": "vertical", "stroke": "light" }
  ]
}
-->

```text
 │
 │
 │


```

<!-- /render -->

<!-- render: 4x5
{
  "next_id": 2,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": 1, "y": 0 }, "len": 4, "orientation": "vertical", "stroke": "light" }
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

The bar's row has no render marker; `## What proves it` names its tests.

## What proves it

| Rule | Test                                                                                                                                                                                                                                                                                                                                     |
| ---- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `a_horizontal_line_is_resized_along_its_row`, `shift_arrows_resize_a_horizontal_line_along_its_row`, `shift_arrows_resize_the_selected_horizontal_line_along_its_orientation`, `key_acceptance_matches_contextual_bar_for_selected_horizontal_line`, `selected_line_shows_identity_pos_length_orientation_stroke_and_selection_commands` |
| 2    | `a_vertical_line_is_resized_along_its_column`, `shift_arrows_resize_a_vertical_line_along_its_column`, `shift_arrows_resize_the_selected_vertical_line_along_its_orientation`, `key_acceptance_matches_contextual_bar_for_selected_vertical_line`, `selected_vertical_line_offers_the_vertical_resize`                                   |
| 3    | `a_line_does_not_shrink_below_one`, `a_zero_length_line_is_not_shrunk_and_grows_to_one`, `a_zero_length_line_is_not_shrunk_and_grows_to_one_in_the_application`, `a_line_grows_no_further_than_u16_max`, `a_line_at_its_minimum_accepts_the_resize_and_records_no_step`                                                                  |
| 4    | `a_line_keeps_its_position_orientation_and_stroke_when_resized`                                                                                                                                                                                                                                                                          |
| 5    | `two_line_resizes_are_two_steps`, `a_line_resize_keeps_the_selection`                                                                                                                                                                                                                                                                    |

The contextual bar's three `RESIZE` labels are checked across figures by
`the_command_row_offers_resize_for_a_box_and_a_line`.

The two pictures are held by their render markers: `cargo xtask render` rewrites them and the gate
fails if one is not what its description draws.

## Open questions

_None._
