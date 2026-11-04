---
status: draft
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
- Minimum line length is **1**: a line cannot shrink below 1 cell (drawing its single point as an
  end), preventing figures from disappearing.
- A line already at length 1 that receives a shrink command remains unchanged. Per #285, no history
  step is recorded.
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
- **Zero-length lines.** `monospace-core` technically permits `len: 0`, but interactive resizing
  clamps to a minimum of 1 to keep the shape selectable and visible.

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

**Answer:** Minimum length is **1**, clamped via `clamp_line_len(current: u16, delta: i16) -> u16`
with range `min(current, 1)..=u16::MAX`.

- At `len == 1`, shrinking produces `len == 1` (no change). By #285, no history step is recorded.
- If a line loaded from a description already has `len == 0`, shrinking leaves it at 0, while
  growing by 1 produces `len == 1`.

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

- `docs/diagram-model.md` §9: `Shape::resized_by` specifies line resizing behavior, clamping `len`
  to minimum 1 along the line's orientation and preserving `at`.
- `docs/editing-model.md` §2: Note that `Command::Resize` applies to `Box` and `Line`.
- `docs/application-model.md` §4: `Shift` plus cursor keys adjusts line length along its active
  axis, and the contextual command line displays directional arrows matching the active orientation.

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
4. **Undo / Redo:**
   - A valid resize step can be undone (`Ctrl+z`) and redone (`Ctrl+r`), restoring previous line
     lengths.

## Examples

```text
Horizontal Line (len: 4):
────

After Shift+→ (len: 5):
─────

After Shift+← (len: 4):
────
```

```text
Vertical Line (len: 3):
│
│
│

After Shift+↓ (len: 4):
│
│
│
│
```

## What proves it

- Unit tests in `monospace-diagram`:
  - Resizing horizontal line with `(1, 0)` increases length.
  - Resizing horizontal line with `(-1, 0)` decreases length.
  - Resizing horizontal line with `(0, 1)` or `(0, -1)` leaves line unchanged.
  - Resizing vertical line with `(0, 1)` increases length.
  - Resizing vertical line with `(0, -1)` decreases length.
  - Resizing vertical line with `(1, 0)` or `(-1, 0)` leaves line unchanged.
  - Clamping preserves minimum length of 1.
- Unit tests in `monospace` (`context_table` / widget):
  - Command line string formatting matches `Shift+←→` for horizontal line, `Shift+↑↓` for vertical
    line, `Shift+↑↓←→` for box.
  - Key dispatching accepts only the offered directional Shift+arrow keys and ignores orthogonal
    ones.
  - History tests verify no undo step is recorded when pressing orthogonal arrows or shrinking at
    minimum length.

## Open questions

_None._
