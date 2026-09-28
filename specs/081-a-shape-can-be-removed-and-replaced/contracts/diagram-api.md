# Contract: what `monospace-diagram` exposes after this slice

The public surface after feature 081. It is feature 080's
[`diagram-api.md`](../../080-a-shape-in-a-diagram-has-an-identity-and/contracts/diagram-api.md) with
one type added, one type's derives widened and three methods added; that file stays the record of
what 080 shipped, and this one is what a caller sees now. Everything here carries rustdoc as it is
written.

Field-by-field meaning is in [data-model.md](../data-model.md); why the types are shaped this way is
in [research.md](../research.md).

## `Delta` — new

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Delta {
    /// How far the figure moves right.
    pub dx: i32,
    /// How far the figure moves down.
    pub dy: i32,
}

impl From<(i32, i32)> for Delta {
    /// Builds a delta from a pair of amounts, right and down.
}
```

A caller adds a delta to a position by giving it to `displaced_by`, and by nothing else: the one
function that does the arithmetic is crate-private and is not part of this surface.

- The model's _Vocabulary_ gives a delta this row — a horizontal and a vertical amount — and nothing
  else. It is not `Pos` reused: that is one type meaning two things without saying so. It is not
  `Size`, which is `u32` and means an extent.
- Both fields are public and the type is `Copy`, so a caller names a displacement as a value and
  applies it to as many figures as it likes. `From<(i32, i32)>` is the second way in, for a caller
  holding a pair of amounts rather than a delta.
- It cannot fail. Saturating is what makes that true at the ends of `i32`, and a saturated
  coordinate is past the end of any window a `u32` width can describe, so the figure draws nothing
  rather than wrapping into a window the caller could really hold.

## `Shape` — changed

```rust
impl Shape {
    /// Builds a new figure, this one moved by `by`, changing nothing.
    pub fn displaced_by(&self, by: Delta) -> Self;
}
```

The derives widen to `Debug, Clone, PartialEq, Eq`. `Clone` and `PartialEq` are what let a caller
compare what `get` returned with what it added, and what let a displaced-by-nothing figure come back
equal to itself; every field already supported them.

- `displaced_by` takes `&self` and gives back a `Self`, so it composes with `get`'s borrow and
  nothing is cloned at the call site.
- It builds a value and changes nothing. No diagram, no buffer, no counter: a displacement is a
  property of one figure, and putting the value back under an identity is what changes anything.
- One arm per variant. `Box` and `Line` move their own `at`; a `Connector` moves `from.at` and
  `to.at` together, because its route is derived from its two endpoints and never described by the
  caller. A connector is not the exception to displacement, and displacing one endpoint is not a
  silent no-op.
- What comes back is the bare figure the caller added. The identity and the place in the order are
  the diagram's, held beside the figure rather than inside it, and nothing about a figure names
  where it sits.
- It has no answer for a figure holding a **reference**, because none can. The model states the
  first answer under _Positions_; issue #82 is where there is a reference to displace.

## `Diagram` — changed

```rust
impl Diagram {
    /// The figure named by `id`, or `None` when the diagram holds no such shape.
    pub fn get(&self, id: &ShapeId) -> Option<&Shape>;

    /// Takes the shape named by `id` out of the diagram.
    pub fn remove(&mut self, id: &ShapeId);

    /// Puts `shape` where the shape named by `id` stands, in the same place in the order.
    pub fn replace(&mut self, id: &ShapeId, shape: Shape);
}
```

`new`, `add`, `forward`, `backward` and `draw` are exactly what feature 080 shipped, and this slice
changes none of them. `ShapeId`, `Placed` and `Endpoint` are unchanged.

- **`get`** is the crate's first reader, and the only one. It borrows, and there is no `cloned()`
  beside it. One query by an identity the caller already holds is the whole of it: there is no
  listing of a diagram's identities, no count, no order, and no way to ask which shape decided a
  position. Those are §11's open question and issue #86, not gaps in this.
- **`remove`** returns nothing, and neither can fail. An identity the diagram does not hold changes
  nothing, with no error, no report and no panic — the same answer the model's _Positions_ gives a
  reference to a shape that is not there. What remains of the order keeps the order it had. The
  counter is not touched, so an identity is never reissued: take `#1` out, add a figure, and the
  identity handed back is the next one rather than `#1`.
- **`replace`** takes the shape by value and returns nothing, for the same reason `remove` does. An
  identity the diagram does not hold changes nothing and the shape handed in is not added either:
  there is no way to name a shape into existence. A caller that needs to keep the value anyway
  clones it first — `Shape` is `Clone` — and this slice's own caller does not.
- **What a replacement keeps** is what the diagram owns: the identity and the place in the order.
  Everything the figure owned is the new figure's — its kind, its parameters and its position. The
  diagram never reaches into a box and widens it, and a replacement may change the kind outright, a
  box becoming a line, with nothing of the previous figure surviving.
- **No sixth verb.** Moving a figure is `get`, `displaced_by` and `replace`, and the diagram itself
  has no `move`. §9 still counts five changes.

## What is still not here

- No `ids()`, no `len`, no order, and no way to ask which shape decided a position.
- No undo, and no history: `remove` hands back nothing, and putting a figure back means building it
  again.
- Nothing about **references**: no `Position`, no `Anchor`, no offset, so removing a shape leaves
  every reference to it unresolved only in the model, and displacing a reference only in the model.
  Issue #82.
- No way for a caller to choose an identity or to edit one — an open question in the model.
- No move to the very front or the very back in one step. Not in the model.
- No measuring, no rendering and no glyph catalog; `draw` remains the only observation a caller
  makes of an order, beside the one query `get` is.

Each of these is a line in the specification's **What this slice does not decide**, and the two that
are not in the model are the ones the specification accepts with nothing to verify them, per
principle IV: no figure can hold a reference yet, so neither is reachable from a test.

## Using it

```rust
use monospace_core::{Buffer, GlyphCatalog, Orientation, Pos, Size, Stroke, render};
use monospace_diagram::{Delta, Diagram, Shape};

let origin = Pos { x: 0, y: 0 };
let size = Size { width: 20, height: 6 };

let mut diagram = Diagram::new();
let back = diagram.add(Shape::Box { at: origin, size: Size { width: 4, height: 3 },
                                    stroke: Stroke::from("light"), fill: None });
let front = diagram.add(Shape::Line { at: origin, len: 4, orientation: Orientation::Horizontal,
                                      stroke: Stroke::from("light") });

// Moving the back one. Reading the figure back is what makes the move expressible: the identity
// says which entry, and the entry holds the value to displace. Nothing is cloned at the call site,
// and the diagram is untouched until `replace`.
if let Some(moved) = diagram.get(&back).map(|shape| shape.displaced_by(Delta { dx: 2, dy: 0 })) {
    diagram.replace(&back, moved);
}

// Taking the line out. It hands back nothing and cannot fail, so there is no result to inspect.
diagram.remove(&front);

let mut buffer = Buffer::new(origin, size);
diagram.draw(&mut buffer);
let text = render(&buffer, &GlyphCatalog::light(), origin, size);
```

The diagram is asked one question and answers one thing, and everything else is a value the caller
built. Drawing again into a fresh buffer is how any of it is seen, and a shape removed draws nothing
rather than leaving a hole: each cell is whatever the figure behind it decides, or empty.
