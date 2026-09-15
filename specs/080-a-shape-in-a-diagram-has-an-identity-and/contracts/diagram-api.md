# Contract: what `monospace-diagram` exposes after this slice

The public surface after feature 080. It is spec 079's
[`diagram-api.md`](../../079-a-diagram-holds-shapes-and-draws-itself/contracts/diagram-api.md) with
one type added and one signature changed; that file stays the record of what 079 shipped, and this
one is what a caller sees now. Everything here carries rustdoc as it is written (FR-006).

Field-by-field meaning is in [data-model.md](../data-model.md); why the types are shaped this way is
in [research.md](../research.md).

## `ShapeId` — new

```rust
pub struct ShapeId(/* private */);

impl std::fmt::Display for ShapeId {
    /// Writes the identity as `#1`, `#2`, and so on.
}
```

Derives `Clone`, `Copy`, `Debug`, `PartialEq`, `Eq`.

- The diagram generates one per addition and hands it back. There is no other source (FR-002).
- There is no constructor, no `From<&str>`, no `FromStr`, and no accessor for what is inside:
  reading one gives no way to build one (FR-003).
- Two identities compare equal when they name the same addition to the same diagram. An identity
  from another diagram is a valid value that matches nothing.

## `Diagram` — changed

```rust
impl Diagram {
    /// An empty diagram, holding no shapes.
    #[must_use]
    pub fn new() -> Self;

    /// Puts `shape` at the front of the order and returns the identity the diagram gave it.
    pub fn add(&mut self, shape: Shape) -> ShapeId;

    /// Moves the shape named by `id` one place toward the front of the order.
    pub fn forward(&mut self, id: ShapeId);

    /// Moves the shape named by `id` one place toward the back of the order.
    pub fn backward(&mut self, id: ShapeId);

    /// Draws every shape into `buffer`, front to back, stamping every cell with `Below`.
    pub fn draw(&self, buffer: &mut Buffer);
}
```

- `add` gains a return value and changes nothing else: the shape still goes to the front (FR-005).
  It is not `#[must_use]`, so a caller with no use for the identity writes the call as it wrote it
  before.
- `forward` and `backward` return nothing, and neither can fail (FR-009). An identity the diagram
  does not hold, and a shape already at the end it is moving toward, both change nothing and report
  nothing.
- Neither changes anything but the order: the shapes, their parameters and their identities survive
  (FR-011), and an identity keeps naming the same shape across any number of moves (FR-004).
- `draw` is unchanged, and stays the only way the order can be observed (FR-012).

## `Shape` and `Endpoint` — unchanged

Both are exactly what spec 079 shipped. A shape carries no identity: the caller builds it as a
literal, and the diagram is what names it.

## What is still not here

- No `remove` and no `replace` (issue #81).
- No way for a caller to choose an identity or to edit one — an open question in the model.
- No reader: a diagram still answers no question about the shapes it holds, their identities or
  their order.
- No move to the very front or the very back in one step. Not in the model.
- No anchor, reference, offset, measurement, rendering or glyph catalog.

Each of these is a line in the spec's **Out of scope** table.

## Using it

```rust
use monospace_core::{Buffer, GlyphCatalog, Orientation, Pos, Size, Stroke, render};
use monospace_diagram::{Diagram, Shape};

let origin = Pos { x: 0, y: 0 };
let size = Size { width: 20, height: 6 };

let mut diagram = Diagram::new();
let back = diagram.add(Shape::Box { at: origin, size, stroke: Stroke::from("light"), fill: None });
diagram.add(Shape::Line { at: origin, len: 4, orientation: Orientation::Horizontal, stroke: Stroke::from("light") });

assert_eq!(back.to_string(), "#1");

// The box was added first, so it is behind the line. This puts it in front.
diagram.forward(back);

let mut buffer = Buffer::new(origin, size);
diagram.draw(&mut buffer);
let text = render(&buffer, &GlyphCatalog::light(), origin, size);
```

Drawing again into a fresh buffer is how the move is seen. The diagram is asked nothing, and answers
nothing.
