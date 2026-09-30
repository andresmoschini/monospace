# Contract: what `monospace-diagram` exposes after this slice

The public surface after feature 083. It is feature 082's
[`diagram-api.md`](../../082-a-connector-endpoint-hangs-from-a-box-s/contracts/diagram-api.md) with
one struct gaining a field, one method gaining a step, and one item moving out of _what is still not
here_; that file stays the record of what 082 shipped, and this one is what a caller sees now.
Everything here carries rustdoc as it is written.

Field-by-field meaning is in [data-model.md](../data-model.md); why the arithmetic is where it is
and why the wire grows a union is in [research.md](../research.md), and the three answers in
[decisions.md](../decisions.md) are named beside each below.

## `Reference` — a third field

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// The identity of the shape the position hangs from.
    pub id: ShapeId,
    /// Which of that shape's four sides it hangs from.
    pub anchor: Anchor,
    /// How far from that side, along each screen axis, in cells.
    pub offset: Delta,
}
```

- **Three fields, and the model's row has said three all along** (D1). §1 describes a reference as
  "a `ShapeId`, an `Anchor` on it, and a horizontal and vertical offset" and §4 says a reference "is
  resolved by asking the referenced shape for that anchor and adding the offsets". This is the
  sentence becoming true, and the two `i32` fields that were the alternative are additive in the
  other direction: a caller naming a reference today has to gain one clause rather than two.
- **The offset is a `Delta`, not two `i32`s** (D1). It is a horizontal and a vertical signed amount,
  it derives `Copy`, and it is the type the model's vocabulary already gave a displacement — so the
  crate keeps one place where coordinates are added. §1's `Delta` row is one clause wider in this
  slice's `docs` commit, and that is the whole of the cost, stated in the model rather than here.
- **It is a gap from the side, not a point.** Adding it to whatever the anchor answers _now_ is what
  makes displacing the referenced figure carry the endpoint with it (SC-002). An implementation that
  added it to the anchor's position at construction would be a different type with the same name.
- **Nothing else changes**: not the derives, not the visibility of `id` and `anchor`, and not how a
  reference is built. It is a literal, and it was one before.

## `Position` — changed in behavior, not in surface

`Absolute(Pos)`, `Reference(Reference)`, `From<Pos>` and the derives are all as 082 recorded them.

### `resolve` gains a step

```rust
/// Where this position stands now, in `diagram`, or `None` when it does not resolve.
#[must_use]
pub fn resolve(&self, diagram: &Diagram) -> Option<Pos>;
```

Same signature, same visibility, same `None` for the same two reasons, and **one more step**: a
reference asks the diagram for its figure, asks that figure for its anchor, and adds `offset` to the
point that came back. The order is §4's and it is the whole of research.md Q1's measurement.

- **The offset is added after both `?`s**, so a reference that resolves to nothing is still nothing.
  A large offset on a figure the diagram does not hold adds to nothing, the figure holding it is not
  drawn, every other shape draws exactly what it drew, and no run fails (SC-004).
- **`Delta::apply` saturates rather than wrapping**, and it is the same function a displacement
  uses. A saturated coordinate is past the end of any window a `u32` width can describe and
  therefore draws nothing; a wrapped one can land inside a window a caller could really hold.

### `displaced_by` does not change

```text
NOT CHANGED:  pub(crate) fn displaced_by(&self, by: Delta) -> Self
```

A reference still comes back exactly as it went in, and the offsets do not change that. §4 states
the destination — "a displacement reaches its offsets" — and this slice does not walk there:
[#143](https://github.com/andresmoschini/monospace/issues/143) is the slice that does, and 082's D4
is the answer not to amend §4 to describe the interim. A caller displacing the figure a reference
hangs from sees the connector's far end stand still while the near end travels, which is the
behavior 082 pinned and the one a caller relies on until then (B3, SC-006).

## `Delta` — unchanged in shape, wider in meaning

```rust
pub struct Delta {
    /// How far the figure moves right. Negative moves it left.
    pub dx: i32,
    /// How far the figure moves down. Negative moves it up.
    pub dy: i32,
}
```

**No item is added, removed or retyped.** The field names, the derives, `From<(i32, i32)>` and the
`pub(crate) apply` are all as before, and the doc comment gains the second reading. `apply` stays
crate-private and stays the only arithmetic in the crate.

## `Shape` and `Diagram` — unchanged

`Shape::anchor` is untouched and stays `pub(crate)`: the offset is added to what it returns, never
asked of it, so nothing new reaches past `Position::resolve` (research.md Q4). `Endpoint.at` is
still a `Position`, `leaving` and `terminal` are still the caller's rather than the anchor's
([#89](https://github.com/andresmoschini/monospace/issues/89)), and `Shape::draw` still resolves
both endpoints before writing anything.

`Diagram`'s seven methods keep their signatures, `ShapeId` is unchanged, and no sixth verb appears:
a connector does not travel because the diagram was told to move it, and it still vanishes as a
whole when either endpoint does not resolve.

## What is still not here

- **No displacement that reaches a reference's offsets.** The rule the model states and the code
  does not yet follow, named as a gap rather than left to be found:
  [#143](https://github.com/andresmoschini/monospace/issues/143).
- **No way to say "one cell out from that side" without naming the axis**, which the specification's
  own clarification refused in the screen axes so that #143's displacement keeps its one reading:
  [#146](https://github.com/andresmoschini/monospace/issues/146).
- **No anchor beyond the four sides**: no corner and no center.
  [#90](https://github.com/andresmoschini/monospace/issues/90).
- **No way for a caller to choose an identity**, or to name a figure a file did not put in a list.
  `ShapeId::new` spells an identity a caller wants and the diagram still issues them, so §11's open
  question stands — the trigger it names has fired and been declined (D3).
- **No way to ask which shapes a diagram could not draw.** A reference that does not resolve and a
  diagram that drew nothing look the same from outside. That is the cost
  [ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md) records, and
  [#88](https://github.com/andresmoschini/monospace/issues/88) is where it is answered.
- **No `ids()`, no `len`, no order**, no undo and no history.
- **No serialization here at all.** The format that can name a reference belongs to `monospace-cli`
  and is recorded in [contracts/description-format.md](description-format.md); this crate depends on
  no serializer.
- **Nothing in `monospace-core`**, and nothing in `Position` that reaches for it. The addition is
  over a `Pos` the core already exports and a `Delta` this crate already owns.

One rule this slice accepts with nothing to verify it is named as such per principle IV rather than
described as tested: **an offset that puts the endpoint inside the figure it hangs from draws it
there**, composing in the shared cell by the rule two figures sharing a cell always obey, rather
than the offset being refused. Nothing in the model or the specification promises otherwise, and
there is no code path that could refuse it.

## Using it

```rust
use monospace_core::{Buffer, Direction, GlyphCatalog, Pos, Size, Stroke, Terminal, render};
use monospace_diagram::{Anchor, Delta, Diagram, Endpoint, Position, Reference, Shape};

let origin = Pos { x: 0, y: 0 };
let size = Size { width: 20, height: 6 };

let mut diagram = Diagram::new();

// The box the arrow hangs from, named by the identity `add` handed back rather than written out.
let box_id = diagram.add(Shape::Box { at: origin,
                                      size: Size { width: 4, height: 3 },
                                      stroke: Stroke::from("light"),
                                      fill: None });

// The arrow, with one endpoint standing two cells right of that box's bottom side centre. Neither
// number above says where the bottom side centre is, and the offset is a gap from it rather than a
// point: displacing the box carries the endpoint with it and the gap does not change.
diagram.add(Shape::Connector {
    from: Endpoint { at: Pos { x: 1, y: 3 }.into(),
                     leaving: Direction::Down,
                     terminal: Terminal::Arm },
    to: Endpoint { at: Position::Reference(Reference { id: box_id,
                                                       anchor: Anchor::Bottom,
                                                       offset: Delta { dx: 2, dy: 0 } }),
                   leaving: Direction::Right,
                   terminal: Terminal::Arm },
    stroke: Stroke::from("light"),
});

let mut buffer = Buffer::new(origin, size);
diagram.draw(&mut buffer);
let text = render(&buffer, &GlyphCatalog::light(), origin, size);
```

A caller that wants to know where an endpoint stands without drawing asks the same way the drawing
does, and `None` is still an ordinary answer rather than a failure:

```rust
let at = endpoint_position.resolve(&diagram);   // Option<Pos>
```
