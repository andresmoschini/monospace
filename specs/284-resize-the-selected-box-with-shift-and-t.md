---
status: draft
date: 2026-11-03
---

# Resize the selected box with Shift and the arrow keys

## Why now

Issue #279 gave the application the contextual bar, and its row of commands is the roadmap's own:
`MOVE` exists since #179, and `RESIZE: Shift+↑↓←→` is the next block the roadmap puts beside it.
Resizing is also the one action of that row the editing layer cannot express:
`docs/editing-model.md` §2 already lists "a figure resized" as one of the meanings `replace` serves,
and says that a command that resizes "has to know how to rebuild a figure of each kind at another
size, and to answer what resizing a figure that has no size means". Nobody has answered either yet,
and this spec does.

## Scope

### In

- `Shift` plus an arrow, with the region holding the keyboard and a **box** selected, resizes it by
  one cell: `Shift+→` widens, `Shift+←` narrows, `Shift+↓` heightens and `Shift+↑` lowers it. The
  box stays anchored at its top-left corner `at`.
- A new `Command::Resize { id, by: Delta }`, one press and one step, and the `Shape` operation it
  becomes.
- The command row of the contextual bar offers `RESIZE: Shift+↑↓←→` beside `MOVE` **only while a box
  is selected**, in the same change (the roadmap's honesty rule).

### Out

- **Resizing a line.** Its own issue: the length, not a width and a height, and a minimum of its
  own. Until then `Shift+arrow` with a line or a connector selected draws nothing new.
- **A replacement for `Shift+arrow`** on terminals that do not report it (the roadmap's rule 9). The
  key stays as the roadmap names it and compatibility is a later issue.
- **Auto-fit (`[a]`), the text dialog, and any growth of a box to what its text needs.** A box
  narrowed under its text clips the text exactly as one described that small does today.
- **A command that changes nothing recording a step.** It is the session's rule today, it already\n
  holds for `Move` with an identity the diagram does not hold, and it is\n
  [#285](https://github.com/andresmoschini/monospace/issues/285)'s to settle for every command at\n
  once. This spec adds no application-side check and no exception (see Open questions).\n- **A step
  of more than one cell, a drag, and holding the key.** One press, one cell.
- **Resizing from the top or the left side.** The box is anchored at `at` (rule 5 of the roadmap).

## The decision

**D1 — how a resize is written in the editing layer.**

**Answer:** `Command::Resize { id, by: Delta }`, where `dx` is the change of width and `dy` the
change of height, and the command becomes `replace` with `Shape::resized_by(by)`, a new public
method of `monospace-diagram` beside `displaced_by`. **Why not** `Resize { id, to: Size }`: the
caller would have to read the box to compute the size, and the application names no `Size` or
`Shape` to decide anything (rule 9 of #178) — the same reason `Move` carries a delta. **Why not** a
new type for the change of size: a width and a height are the two screen axes a `Delta` already
signs, and a second type would be one more public item that is the first one renamed. **Answered
by** the pattern of `Move`, §2 of the editing model.

**D2 — the smallest box, and what shrinking does at it.**

**Answer:** a box does not shrink below **2×2**, the size under which `BoxShape` draws nothing, and
growth stops at `u16::MAX`. A box that is already below 2 in a dimension (a description may say
`1×5`) is not touched by shrinking in that dimension, and growing it by one is allowed. In short the
new dimension is `current + by` clamped to `min(current, 2)..=u16::MAX`. **Why not** the roadmap's
minimum of 4×5, the auto-fit's: that is the size a text _needs_, and a box without text is useful as
a 2×2 corner mark. **Why not** letting it reach 0: the box vanishes with nothing selected to pull it
back by, and the roadmap's own table says `mín 2`. **What it costs:** a box with text can be
narrowed until its text is clipped; auto-fit is the way back and is not this slice's.

**D3 — what happens to the connectors that hang from a resized box.**

**Answer:** nothing is rewritten. A reference stores a side, an `offset` and an `out`, and the side
is resolved from the box's `at` and `size` at every draw, so the right and bottom sides move with
the new size, the others keep their place, and each endpoint keeps its gap. **Why not** freezing
them into absolute points: it detaches every connector from a box the person only widened. **Why
not** rewriting the `offset` so the endpoint stays still: a side that moved is what hanging means,
and `displaced_by` already settled that a figure displaced carries its references along. **Answered
by** §4 and §5 of the diagram model; this decision only says no rule is added.

## Model slice

- `docs/editing-model.md` §2: the initial set gains `Resize { id, by }` — _the figure becomes so
  much wider and taller_, becoming `replace` with the figure resized — and the sentence about "what
  resizing a figure that has no size means" is answered: it is the figure unchanged.
- `docs/diagram-model.md` §9: `Shape::resized_by` beside `displaced_by`, with D2's clamp and the
  rule that a `Line` and a `Connector` come back equal. §4 and §5: none.
- `docs/application-model.md` §4: the movement paragraph gains the resize — `Shift` with an arrow is
  one action carrying a delta, one press is one command and one step. §7: the properties of this
  change, below.

## Public surface

- `monospace-diagram`: `Shape::resized_by(&self, by: Delta) -> Self`.
- `monospace-editing`: `Command::Resize { id: ShapeId, by: Delta }`.
- `monospace` (no public items): `Action::Resize(i16, i16)`,
  `DiagramState::resize_shape(&mut self, id, dx, dy)` and the region's key answering the four
  `Shift`+arrow presses.
- `status::command_line` offers the `RESIZE` block for a selected box. No dependency is added.

## Behavior

1. `Shift+Right`, with a box selected and the region holding the keyboard, performs one
   `Command::Resize { id, by: (1, 0) }` and records one step; `Shift+Left` is `(-1, 0)`,
   `Shift+Down` `(0, 1)` and `Shift+Up` `(0, -1)`.
2. The box keeps its `at`, its stroke, its fill and its text; only `size` changes.
3. A press that would take a dimension under 2 leaves it where it was, per D2; at `u16::MAX` growth
   leaves it too.
4. With nothing selected a press performs no command and records no step; `Control` is `Continue`.
   With a line or a connector selected, or a box at its minimum, the command is performed, the
   figure comes back equal and the session records its step like any other command (#285 owns that).
5. The selection survives, and the next frame draws the box at its new size with the new cells
   inverted.
6. A connector hanging from the box follows (D3): the right side after `Shift+→`, the bottom after
   `Shift+↓`, and one hanging from the left or the top does not move.
7. One press is one step: `Undo` gives back one cell per press.
8. A plain arrow still moves (rule 1 of #179). `Shift` with an arrow is the resize and **no other
   modifier combination** acts, `Ctrl+Shift+arrow` included. A release and a repeat do nothing.
9. While the bar holds the keyboard `Shift+arrow` resizes nothing (D1 of #179 applies unchanged).
10. The command row reads `MOVE: ↑↓←→ · RESIZE: Shift+↑↓←→ · [Del] Delete · [Esc] Deselect` while a
    box is selected, and keeps what it said for a line, a connector and an empty selection.
11. The application names no `Size` and no `Shape` constructor, and calls neither `replace` nor
    `resized_by`: it hands a delta and nothing else.

## Examples

**The key and what it leaves behind.** Hypothetical: nothing has been run; the acceptance list
checks it. One `4×3` box selected by a click, with the region holding the keyboard:

| What arrives                       | Commands | Steps | Size afterwards           |
| ---------------------------------- | -------- | ----- | ------------------------- |
| `Shift+Right`                      | 1        | 1     | 5×3                       |
| `Shift+Down` twice                 | 2        | 2     | 4×5                       |
| `Shift+Left` twice from 4×3        | 2        | 2     | 2×3                       |
| `Shift+Left` a third time          | 1        | 1     | 2×3 (D2, #285)            |
| `Shift+Right` with a line selected | 1        | 1     | the line as it was (#285) |
| `Shift+Right`, nothing selected    | 0        | 0     | —                         |
| `Right` (no `Shift`)               | 1 `Move` | 1     | 4×3, one cell on          |

**A widened box and its connector.** A box with a connector hanging from its right side at a gap of
one cell and reaching an absolute point, before and after one `Shift+Right`. The side moved one cell
and the connector with it, so it is one cell shorter. One press, one
`Command::Resize { id: 1, by: (1, 0) }`, one step.

<!-- render: 20x4
{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": 2,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right",
                        "offset": { "dx": 2, "dy": 0 } },
                "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "point", "x": 15, "y": 2 },
                "leaving": "left", "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ] }
-->

```text

  ┌──┐
  │  │ ─────────
  └──┘
```

<!-- /render -->

<!-- render: 20x4
{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 2, "y": 1 }, "size": { "width": 5, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": 2,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right",
                        "offset": { "dx": 2, "dy": 0 } },
                "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "point", "x": 15, "y": 2 },
                "leaving": "left", "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ] }
-->

```text

  ┌───┐
  │   │ ────────
  └───┘
```

<!-- /render -->

The bar's row has no render marker; `## What proves it` names its tests.

## What proves it

| Rule | Test                                                                                    |
| ---- | --------------------------------------------------------------------------------------- |
| 1    | `shift_arrows_resize_the_selected_box_by_one_cell`, `a_resize_command_changes_the_size` |
| 2    | `a_resized_box_keeps_its_position_stroke_fill_and_text`                                 |
| 3    | `a_box_does_not_shrink_below_two`, `a_box_below_two_is_not_shrunk_further`              |
| 3    | `growth_stops_at_the_largest_size`                                                      |
| 4    | `a_resize_with_nothing_selected_performs_no_command`, `a_line_is_not_resized`           |
| 5    | `a_resize_keeps_the_selection`                                                          |
| 6    | `a_resize_carries_the_connector_hanging_from_the_right_side` and the two pictures above |
| 7    | `two_resizes_are_two_steps`                                                             |
| 8    | `a_plain_arrow_still_moves`, `no_other_modifier_resizes`                                |
| 9    | `shift_arrow_with_the_bar_focused_resizes_nothing`                                      |
| 10   | `the_command_row_offers_resize_for_a_box_only`                                          |
| 11   | _nothing holds this: a reviewer reads the imports_                                      |

The two pictures are held by their render markers: `cargo xtask render` rewrites them and the gate
fails if one is not what its description draws.

## Open questions

**A box narrowed under its text.** The text is clipped until auto-fit exists; whether a resize
should refuse to cut the text is the auto-fit issue's, together with the minimum it reads.

**A command that changes nothing still records a step.** Rows 4 and 5 of the table are the cost, and
[#285](https://github.com/andresmoschini/monospace/issues/285) is where it goes away for every
command at once. A test of this spec must not assert the step count in those cases as a rule, only
that the figure is unchanged, so #285 can change it without touching this spec.

**Resizing from the other corner.** Anchoring the top-left is the roadmap's rule; a way to grow
upward or leftward is a move plus a resize, and nothing here prevents it.
