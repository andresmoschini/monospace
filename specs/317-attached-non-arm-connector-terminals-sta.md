---
status: implemented
decided: "#318"
implemented: "#319"
date: 2026-11-07
---

# Attached non-arm connector terminals stand one cell outward by default

## Why now

In Monospace, connectors can anchor their endpoints to shapes via references
([`docs/diagram-model.md`](../docs/diagram-model.md) §6 _Attachment_).

When an endpoint is attached to a shape's side anchor, an `arm` terminal is designed to join and
compose with the border cell, turning the border into a junction (`├`, `┤`, `┬`, `┴`). However,
non-arm terminals (`Symbol` like `▶` or `◀`, and `Glyph` literals) do not compose with borders: they
overwrite the cell they land on.

Currently, an attached endpoint with default `out: 0` resolves directly onto the border cell of the
referenced shape. For a non-arm terminal (such as an arrowhead), this places the symbol directly on
the border cell, overwriting the border line and pointing inward into the interior of the shape. To
avoid destroying the border, a author must manually supply `out: 1` on every attached arrow
endpoint.

The initial spike exploration demonstrated this friction: Case 1 with default `out: 0` overwrites
the destination box border, whereas Case 2 with `out: 1` renders cleanly with the arrow touching the
border from the outside.

Human intuition expects an attached arrow to terminate against the shape's boundary from the outside
by default. By introducing an automatic one-cell outward adjustment for attached non-arm terminals,
connectors naturally point at their target shapes without manual offset adjustments.

## Scope

### In

- Automatic one-cell outward placement for non-arm connector terminals (`Terminal::Symbol` and
  `Terminal::Glyph`) when attached via a [`Reference`](../docs/diagram-model.md#4-positions).
- Effective outward distance calculation:
  `effective_out = reference.out + (if terminal == Terminal::Arm { 0 } else { 1 })`.
- Symmetric behavior for both `from` and `to` endpoints.
- Additive behavior for explicit `out`: `out: 0` (or omitted) yields effective outward distance 1;
  `out: 1` yields 2; `out: -1` allows intentionally placing the symbol on the border cell.
- Freezing consistency: when a referenced shape is removed, an attached endpoint freezes to an
  absolute position matching its effective rendered position (`effective_out`).
- Updating [`docs/diagram-demo.md`](../docs/diagram-demo.md) to document and reflect the new default
  rule.

### Out

- Changes to absolute endpoints ([`EndpointAt::Absolute`](../docs/diagram-model.md#6-attachment)):
  absolute coordinates have no anchor border to offset from and remain exact.
- Changes to `Terminal::Arm` attachment: arms continue to land on the border cell (`out: 0`) to
  compose into junctions.
- New terminal types or modifications to routing algorithms.

## The decision

**D1 — non-arm attached connector terminals stand one cell outward by default.**

**Answer:** When an attached endpoint's terminal is not `Terminal::Arm` (i.e. `Terminal::Symbol` or
`Terminal::Glyph`), the endpoint resolves with an automatic `+1` outward cell offset along the
anchor's normal (`effective_out = reference.out + 1`). `Terminal::Arm` remains at
`effective_out = reference.out`.

**Why not** requiring manual `out: 1` in the description or UI: An attached arrow intuitively points
_at_ the target shape from the outside. Defaulting to `out: 0` for symbols lands the arrowhead
directly on top of the border cell, erasing the border character and pointing into the box. Users
should not have to manually specify `out: 1` on every connector to avoid border destruction.

**Why not** shifting the box or clipping the arrow: Monospace uses a discrete cell grid. Shifting
the endpoint by one cell outward along the anchor normal is the natural discrete representation of
"pointing at the border from outside".

**Answered by** the maintainer, in #317.

**D2 — explicit `out` is additive to the base offset.**

**Answer:** Any explicit `out` provided in the reference is added to the terminal's default offset:
`effective_out = reference.out + (if terminal == Terminal::Arm { 0 } else { 1 })`. An explicit
`out: 0` (or omitted) yields `effective_out = 1` for symbols, while `out: 1` yields
`effective_out = 2`, and `out: -1` yields `effective_out = 0` (placing the symbol on the border cell
if explicitly desired).

**Why not** treating explicit `out` as an absolute override (where `out: 0` would force placing on
the border): That would break the intuition that `out` is an outward adjustment from the natural
resting position of that terminal kind. `out` represents outward distance beyond the attachment
boundary. Having `out` be additive ensures that omitting or providing `0` means "default attachment
distance", positive `out` pushes it further clear, and negative pulls it inward.

**Answered by** the maintainer, in #317.

**D3 — symmetric behavior for `from` and `to` endpoints.**

**Answer:** The default outward offset rule applies identically to both `from` and `to` endpoints of
a connector. An attached `from` endpoint with a non-arm terminal (e.g. an arrow pointing back toward
the source box) resolves one cell outward from the source shape's anchor border.

**Why not** applying only to `to`: A connector can have heads at either end or both ends
(bidirectional arrows, reverse arrows, glyphs). The physical justification (preventing non-arm
terminals from erasing the border cell they terminate against) is identical for both endpoints.

**Answered by** the maintainer, in #317.

**D4 — frozen endpoint positions reflect the effective resolved position.**

**Answer:** When a shape is removed from the diagram and an attached endpoint freezes to an absolute
position (`frozen_position`), it freezes at the position it was actually rendered at (the effective
position including the extra outward cell).

**Why not** freezing at the anchor border cell (`effective_out - 1`): If it froze to the border
cell, deleting the attached box would cause the connector's arrow to visually jump one cell forward
into the empty space where the box border used to be. Freezing must preserve the exact visual
geometry of the connector.

**Answered by** the maintainer, in #317.

**D5 — layer ownership in `monospace-diagram`.**

**Answer:** The outward adjustment is owned by `monospace-diagram` during endpoint resolution
(`Endpoint::resolve`). `monospace-core` remains unaware of references, anchors, and shapes,
operating purely on absolute `Endpoint { at: Pos, leaving: Direction, terminal: Terminal }`.

**Why not** handling it in `monospace-cli` or parser: The reader only parses JSON into diagram
structures. The diagram model owns references and anchors, and any consumer (TUI, CLI, renderer,
tests) must observe consistent diagram rendering without duplicating placement math.

**Answered by** the maintainer, in #317.

## Model slice

- `docs/diagram-model.md` §6 _Attachment_: Document that attached non-arm terminals (`Symbol`,
  `Glyph`) stand one cell outward from the anchor border by default
  (`effective_out = reference.out + 1`), whereas `Arm` stands on the border
  (`effective_out = reference.out + 0`) to compose into a junction.
- `docs/diagram-demo.md`: Update row 1 and row 3 of the offset demonstration or its explanatory text
  where the symbol terminal was historically placed on the border, noting the new default behavior.

## Public surface

- `monospace-diagram`:
  - `impl Endpoint`:
    - `pub fn resolve(&self, diagram: &Diagram) -> Option<Pos>`: resolves the endpoint's position
      taking its `terminal` into account (adding 1 outward cell for non-arm terminals attached to a
      reference).
    - `pub(crate) fn effective_out(&self) -> i16`: helper answering the effective outward cell
      count.

## Behavior

1. When an `Endpoint` with `terminal == Terminal::Arm` is attached to a `Reference`, its position
   resolves to `anchor_point + offset + anchor.outward(reference.out)`.
2. When an `Endpoint` with `terminal != Terminal::Arm` (`Symbol` or `Glyph`) is attached to a
   `Reference`, its position resolves to
   `anchor_point + offset + anchor.outward(reference.out + 1)`.
3. An explicit `out` on a `Reference` adds to this base offset (`reference.out + 1` for non-arm,
   `reference.out` for arm).
4. Both `from` and `to` endpoints follow rules 1-3 symmetrically.
5. When a referenced shape is removed, an attached endpoint freezes (`EndpointAt::Absolute`) at its
   effective resolved position (`Pos`) with the anchor's leaving direction.
6. Absolute endpoints (`EndpointAt::Absolute`) are unaffected and remain at their specified canvas
   `Pos`.

## Examples

### Example 1: Connector between two boxes with default `out: 0`

Given Box 1 at `{x: 0, y: 1, w: 6, h: 3}` and Box 2 at `{x: 16, y: 1, w: 6, h: 3}`, connected by a
connector with `from` (anchor `right`, terminal `arm`, `out: 0`) and `to` (anchor `left`, terminal
`symbol: solid`, `out: 0`):

```text
┌────┐          ┌────┐
│    ├─────────▶│    │
└────┘          └────┘
```

The arm joins Box 1's right border at `x=5`. The arrow stands at `x=15` (1 cell clear of Box 2's
left border at `x=16`), leaving Box 2's border intact.

### Example 2: Connector with explicit `out: 1` on symbol terminal

With `to: { anchor: "left", out: 1 }` on the symbol terminal, the arrow stands at `x=14` (2 cells
clear of Box 2's border at `x=16`).

### Example 3: Bidirectional arrows attached to boxes

A connector with `from` (anchor `right`, terminal `symbol: solid`, `out: 0`) and `to` (anchor
`left`, terminal `symbol: solid`, `out: 0`):

```text
┌────┐          ┌────┐
│    │◀────────▶│    │
└────┘          └────┘
```

Both arrowheads stand 1 cell outward from their respective box borders, leaving both boxes intact.

### Example 4: Freezing on shape removal

Removing Box 2 freezes the connector's `to` endpoint to `EndpointAt::Absolute` at
`Pos { x: 15, y: 2 }` with `leaving: Direction::Left`. The rendered visual representation does not
change upon removal of the box.

## What proves it

- Unit tests in `monospace-diagram`:
  - `Endpoint::resolve` for `Terminal::Arm` resolves on the border cell.
  - `Endpoint::resolve` for `Terminal::Symbol` and `Terminal::Glyph` resolves one cell outward.
  - `Endpoint::resolve` with explicit positive and negative `out`.
  - `Endpoint::frozen_position` preserves the effective resolved position.
- Gallery snapshot tests in `monospace-diagram`:
  - `an_attached_symbol_terminal_on_a_side_stands_one_cell_outward_by_default.snap` covering all
    four anchors, explicit `out`, negative `out`, and both `from`/`to` endpoints.
- Regression tests ensuring absolute endpoints and arm attachments remain unchanged.

## Open questions

- None.
