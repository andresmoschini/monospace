---
status: draft
date: 2026-11-09
---

# Attach a detached connector endpoint to an existing shape

## Why now

[#325](325-select-and-inspect-connector-endpoints-w.md),
[#328](328-move-connector-endpoints-with-arrows-adj.md),
[#331](331-select-connector-endpoint-terminal-style.md) and
[#334](334-detach-connector-endpoints-with-0-and-se.md) completed the inline endpoint editor: `1` /
`2` inspect an endpoint, arrows move it, `+` / `-` adjust its standoff, `t` picks a terminal, `0`
detaches it, and `d` chooses its leaving direction. Detaching is the one direction that cannot be
reversed: an `EndpointAt::Absolute` has no route back to an `EndpointAt::Reference`, so a connector
created detached with `[c]` ([#308](308-create-a-box-a-line-or-a-connector-from-th.md)) cannot be
connected to the boxes and lines around it.

This is Slice 9 of [`docs/connector-editing.md`](../docs/connector-editing.md) §5.3: reuse the
existing `Tab` / `Shift+Tab` shape selector or a click on the canvas to choose a target shape, and
the digits `1..=4` to choose one of its four attachment points. The focused target is highlighted on
the canvas the way an idle selection is.

## Scope

### In

- Offering `ATTACH: [a]` in `ENDPOINT_EDIT` when the active endpoint is detached
  (`EndpointAt::Absolute`) and the diagram holds at least one attachable shape.
- Entering `SubMode::AttachSelect { end, target }` from that action, carrying the active `End` and
  the focused target `ShapeId`.
- Cycling the focused target with `Tab` (next) and `Shift+Tab` (previous) over every `Shape::Box`
  and `Shape::Line` in `Diagram::shapes()` order, wrapping at both ends. Connectors are not
  candidates because they answer no anchor point
  ([`docs/diagram-model.md`](../docs/diagram-model.md) §5).
- Focusing a target by a left press on a canvas cell it owns. A press on a connector or on a cell
  nobody owns leaves the focus unchanged.
- Committing with `1`, `2`, `3` or `4`: `Top`, `Right`, `Bottom` and `Left` respectively. The
  endpoint becomes
  `EndpointAt::Reference(Reference { id: target, anchor, offset: Delta::ZERO, out: 0 })`, keeping
  its terminal, and the bar returns to `SubMode::Endpoint(end)`.
- Highlighting the focused target on the canvas with the same reverse video an idle selection uses,
  while the selected connector stays inverted.
- `Esc` cancelling the picker and returning to `SubMode::Endpoint(end)` with no history step.
- Extending `Command::Hang` with `end` and `target` (renamed from `from`), and
  `Shape::with_endpoint_attached`, which act only on a detached endpoint and a target that answers
  `anchor`.
- The [#285](285-a-command-that-changes-nothing-records-n.md) silence rule: an attached endpoint, a
  missing target, a target that is a connector, or a no-op attachment records zero steps.

### Out

- Re-attaching an endpoint that is already attached, or changing its parent shape with `[c]`: the
  author detaches first with `[0]`.
- Attaching to a connector: it answers no anchor point.
- Editing `offset` or `out` in the picker; the attachment starts at zero.
- The extended glyph palette and every other connector-editing slice.

## The decision

**D1 — does `[a]` open a picker sub-mode, or does `Tab` cycle targets directly from
`ENDPOINT_EDIT`?**

**Answer:** `[a]` opens `SubMode::AttachSelect { end, target }` with the instructions in row 1.
`Tab` / `Shift+Tab` or a click move the focus, and `1..=4` choose the attachment point. **Why not**
cycling directly from `ENDPOINT_EDIT`: it gives `Tab` a second meaning in a mode where `1` / `2`
already switch endpoints, leaves no place for the target focus to live, and gives `Esc` nothing
distinct to cancel. The picker mirrors `TERMINAL_SELECT` and `LEAVING_SELECT`.

**D2 — in what order does the picker visit the candidates, and which one is focused on entry?**

**Answer:** the order `Diagram::shapes()` yields, which is the drawing order idle `Tab` already
visits, back to front. Entry focuses the first candidate; `Tab` advances and `Shift+Tab` retreats,
wrapping at both ends. **Why not** proximity to the detached endpoint: it needs a distance rule and
an ordering with no precedent in the bar, while drawing order reuses `select_next` /
`select_previous` and is the order the author already knows.

**D3 — which key commits the attachment?**

**Answer:** the picker only focuses a target; the attachment commits when `1`, `2`, `3` or `4`
selects an anchor. There is no `Enter` confirmation. **Why not** `Enter`: it would commit without an
anchor, which forces a default (D4), and it separates the anchor choice from the commit. Attaching
on every `Tab` would make browsing destructive.

**D4 — is there a default anchor for an attachment that does not choose one?**

**Answer:** no. Without `1..=4` nothing attaches, and the picker stays open. **Why not** the nearest
anchor or the side the endpoint faces: either is a second rule the author did not ask for, and it
would make the four keys an optional refinement rather than the one way to commit.

**D5 — does the picker show its focus on the canvas, or only in the bar?**

**Answer:** the focused target is painted with the same reverse video an idle selection uses, and
the selected connector stays inverted too. The record's owner still decides each cell, so a target
behind another figure inverts only the cells it owns. **Why not** the bar alone: the canvas is where
the author sees which shape is focused, and the existing inversion already answers it. **Why not**
inverting only the target: the author would lose sight of the connector being edited.

**D6 — does the attach command grow the existing `Hang`, or add a new `AttachEndpoint`?**

**Answer:** grow `Hang` with `end`, rename its target field `from` to `target`, and restrict it to
detached endpoints. **Why not** a new `AttachEndpoint`: `Hang` already means "one end of a connector
hangs from a figure's side" in [`docs/editing-model.md`](../docs/editing-model.md) §2, so a second
command would keep two names for one action. Its only caller is the CLI demonstration, which hangs
an end that is already attached and so records no step; the detached-only restriction keeps that
call silent and the demonstration's output unchanged. The rename says what the field is when either
end can hang.

**D7 — what does a left press do while the picker is open?**

**Answer:** it focuses the `Box` or `Line` that owns the cell, and nothing else: it neither selects
that shape nor commits. A press on a connector, an empty cell or outside the interior leaves the
focus where it was. **Why not** the idle behavior, where a press selects its owner: it would end the
picker and drop the connector being edited, so the click a picker invites would destroy it. **Why
not** committing on the click: it would need a default anchor, which D4 refuses. **Why not** leaving
the mouse out of this slice: a press inside the picker has to mean something, and focusing is the
cheapest meaning that keeps the picker alive; without it, a diagram of many shapes is only reachable
by repeated `Tab`.

## Model slice

- [`docs/diagram-model.md`](../docs/diagram-model.md) §9 _Changing a diagram_: add
  `Shape::with_endpoint_attached`, state that it acts only on a detached endpoint and a target that
  answers the anchor, and move the figure-mutation count from ten to eleven.
- [`docs/editing-model.md`](../docs/editing-model.md) §2 _The initial set_: change the `Hang` row to
  `Hang { id, end, target, anchor }` and document its #285 silence rules.
- [`docs/application-model.md`](../docs/application-model.md) §3 _The state_ and §4 _The action
  model_: document `SubMode::AttachSelect`, its `Tab` / `Shift+Tab` and mouse dispatch, the status
  and command rows, the canvas highlight, and the return to `SubMode::Endpoint(end)`.
- [`docs/connector-editing.md`](../docs/connector-editing.md) §2.6, §4.4 and §5.3: replace the
  deferred `ATTACH_PICK` description with this slice. `[c]` reassignment and the glyph palette
  remain deferred; the mouse click no longer is.
- [`docs/contextual-ui.md`](../docs/contextual-ui.md) §2, §4.9 and Stage 3: replace `ATTACH_PICK`
  with `ATTACH_SELECT`, its trigger, its keys and the detached-only scope; §4.9 today says it also
  re-attaches an attached endpoint, which this slice leaves out.

## Public surface

- `monospace-diagram`:
  - `impl Shape`:
    - `pub fn with_endpoint_attached(&self, end: End, reference: Reference, diagram: &Diagram) -> Self`
- `monospace-editing`:
  - `pub enum Command`:
    - `Hang { id: ShapeId, end: End, target: ShapeId, anchor: Anchor }` (breaking: `from` renamed
      `target`, `end` added)
- `monospace`:
  - `pub(crate) enum SubMode`:
    - `AttachSelect { end: End, target: ShapeId }`
  - `pub(crate) enum Condition`:
    - `AttachableEndpoint` — the active endpoint is detached and the diagram holds at least one
      `Box` or `Line`. A block carries one condition, so the two facts are one variant, as
      `EndpointReset` already joins "attached" with "offset or out non-zero".

## Behavior

1. In `SubMode::Endpoint(end)` on a detached endpoint, row 1 offers `ATTACH: [a]` after
   `LEAVING: [d]` and before `TERMINAL: [t]` when the diagram holds at least one `Box` or `Line`.
   When it holds none, `[a]` is not offered and pressing `a` returns `Reply::Ignored`.
2. Pressing `a` enters `SubMode::AttachSelect { end, target }`, where `target` is the first
   candidate in `Diagram::shapes()` order, and requests a repaint without recording a history step.
3. In `AttachSelect`, `Tab` advances the focus to the next candidate in drawing order, wrapping from
   the last to the first; `Shift+Tab` retreats, wrapping from the first to the last. The connector
   stays selected and no history step is recorded. A focus that does not move is `Reply::Consumed`
   and owes no frame.
4. In `AttachSelect`, a left press on a cell whose recorded owner is a `Box` or `Line` focuses that
   shape. A press on a connector, on a cell nobody owns, or outside the interior leaves the focus
   unchanged. No history step is recorded; only a moved focus requests a repaint.
5. In `AttachSelect`, row 0 formats:
   `ATTACH for <n>. <From|To> of Connector #<id> · Detached at (<x>, <y>), leaving <direction> · Target: <Box|Line> #<target> · <i> of <count>`.
6. In `AttachSelect`, row 1 formats:
   `TARGET: [Tab] Next [Shift+Tab] Prev · ANCHOR: [1] Top [2] Right [3] Bottom [4] Left · [Esc] Cancel`.
7. In `AttachSelect`, `1`, `2`, `3` and `4` dispatch `Command::Hang { id, end, target, anchor }`
   with `Top`, `Right`, `Bottom` and `Left` respectively, and return to `SubMode::Endpoint(end)`.
8. The attach replaces the targeted endpoint with
   `EndpointAt::Reference(Reference { id: target, anchor, offset: Delta::ZERO, out: 0 })`,
   preserving its terminal. The endpoint's leaving direction becomes `anchor.leaving()`, and its
   resolved position becomes the target's anchor point plus the terminal's effective outward cell (0
   for `Arm`, 1 for `Symbol` or `Glyph`, per
   [#317](317-attached-non-arm-connector-terminals-sta.md)).
9. The attach records one step when the targeted endpoint is detached and the target is a `Box` or
   `Line`, both of which answer all four anchors; it records zero steps when the endpoint is already
   attached, the target is absent, the target is a connector, or the attachment changes nothing
   (#285).
10. In `AttachSelect`, `Esc` cancels and returns to `SubMode::Endpoint(end)` with zero history steps
    (`SubMode::parent` maps `AttachSelect { end, .. }` to `Endpoint(end)`); unoffered keys return
    `Reply::Ignored` and fall through to universal shortcuts.
11. While `AttachSelect` is active, the copy paints reverse video on the cells the record says
    belong to the selected connector and on the cells it says belong to the focused target, and on
    no others. A shape clipped by the window inverts only the part the window holds.
12. Losing or changing the selection, an effective undo or redo among them, ends `AttachSelect` with
    the rest of the sub-mode (`sub_mode = None`). No key the picker leaves to the universal
    shortcuts adds or removes a shape while the selection survives, so the focused target cannot
    vanish under an open picker and the picker carries no reconciliation of its own; a `Hang` that
    names a missing target would still record zero steps (rule 9).

## Examples

_The pictures are generated from the descriptions they carry; the bars and the key sequences have
not been observed: they are what the acceptance list checks, not the record of a run._

**Attaching one detached end to the only box.** Box 1 (`8×3` at `(0, 0)`) and Connector #2, detached
from `(10, 1)` leaving right to `(20, 1)` leaving left, with an arm terminal at `From` and a solid
terminal at `To`:

<!-- render: 22x3
{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "connector", "id": 2,
      "from": { "at": { "kind": "point", "x": 10, "y": 1 }, "leaving": "right", "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "point", "x": 20, "y": 1 }, "leaving": "left", "terminal": { "kind": "symbol", "style": "solid" } },
      "stroke": "light" }
  ]
}
-->

```text
┌──────┐
│      │  ──────────▶
└──────┘
```

<!-- /render -->

Select Connector #2 and press `1` to edit `From`. Row 1 offers the new action:

```text
MOVE: ↑↓←→ · LEAVING: [d] · ATTACH: [a] · TERMINAL: [t] · [2] To · [Esc] Done
```

Press `a`. The picker focuses the only candidate, and the bars read:

```text
ATTACH for 1. From of Connector #2 · Detached at (10, 1), leaving right · Target: Box #1 · 1 of 1
TARGET: [Tab] Next [Shift+Tab] Prev · ANCHOR: [1] Top [2] Right [3] Bottom [4] Left · [Esc] Cancel
```

Press `2` (`Right`). The `From` endpoint attaches to Box 1's right anchor, the arm composes into the
border cell, and the bar returns to `ENDPOINT_EDIT`:

```text
ENDPOINT 1. From of Connector #2 · Attached to Box #1 at right, offset (0, 0), out 0 · Term: arm ─
OFFSET: ↑↓←→ · OUT: [+] [-] · [0] Detach · TERMINAL: [t] · [2] To · [Esc] Done
```

The connector now starts at the box:

<!-- render: 22x3
{
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 3 }, "stroke": "light" },
    { "kind": "connector", "id": 2,
      "from": { "at": { "kind": "reference", "shape": 1, "anchor": "right" }, "terminal": { "kind": "arm" } },
      "to": { "at": { "kind": "point", "x": 20, "y": 1 }, "leaving": "left", "terminal": { "kind": "symbol", "style": "solid" } },
      "stroke": "light" }
  ]
}
-->

```text
┌──────┐
│      ├────────────▶
└──────┘
```

<!-- /render -->

**The mouse focuses, the digit commits.** With Box 1, Line 2 and Box 3, pressing `a` focuses Box 1
and row 0 ends `1 of 3`. Clicking a cell Line 2 owns focuses Line 2, and row 0 ends `2 of 3`;
nothing is attached yet. Pressing `4` then attaches to Line 2's left anchor.

**A connector is not a candidate.** A diagram holding Box 1, Line 2 and Connector 3 offers only Box
1 and Line 2 to `Tab`; Connector 3 is skipped because `Shape::Connector` answers no anchor point
(`docs/diagram-model.md` §5). With two candidates, row 0 reads `1 of 2` and then `2 of 2`. Clicking
a cell Connector 3 owns leaves the focus where it was.

**Cancelling changes nothing.** Pressing `Esc` in `AttachSelect` returns to `ENDPOINT_EDIT` with the
endpoint still `EndpointAt::Absolute`, and `Undo` has no step to give back.

**The focus is visible on the canvas.** The focused target's cells are reverse video, exactly as an
idle selection's are, and the selected connector's cells stay reverse video while the picker is
open.

## What proves it

None yet. The tests do not exist; the building stage names the test that holds each rule in
`## Behavior`, and a rule with no test named against it is unfinished.

## Open questions

- None beyond the decisions above.
