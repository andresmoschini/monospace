# Contract: what `monospace-diagram` exposes

The public surface of the new crate — the interface a library user sees, and the only part of it
another crate can depend on. Everything here carries rustdoc as it is written (FR-004).

Field-by-field meaning and the mapping onto `monospace_core` are in
[data-model.md](../data-model.md); why the types are shaped this way is in
[research.md](../research.md).

## The crate

```toml
[package]
name = "monospace-diagram"
description = "The model that holds a diagram after it is drawn: shapes in an order, drawable"
```

It depends on `monospace-core` and on nothing else (FR-001), and it compiles for
`wasm32-unknown-unknown` (FR-003).

## `Diagram`

```rust
pub struct Diagram { /* private */ }

impl Diagram {
    /// An empty diagram, holding no shapes.
    #[must_use]
    pub fn new() -> Self;

    /// Puts `shape` at the front of the order, in front of everything already there.
    pub fn add(&mut self, shape: Shape);

    /// Draws every shape into `buffer`, front to back, stamping every cell with `Below`.
    pub fn draw(&self, buffer: &mut Buffer);
}

impl Default for Diagram { /* delegates to new */ }
```

- `add` cannot fail and returns nothing. It gives no identity in this slice.
- `draw` takes `&self`: it changes nothing about the diagram, and drawing twice into equal windows
  produces equal buffers (FR-015).
- `draw` takes a `Buffer`, not a `Surface`: the window is the buffer's, and the stamp mode is the
  diagram's to bind (FR-010, FR-012). There is no parameter through which a caller can choose one.
- `draw` produces cells and stops. It takes no glyph catalog and returns no text (FR-013).
- A shape that falls outside the buffer's window is clipped by the buffer, silently. `draw` has no
  return value and no error (FR-014).

## `Shape`

```rust
pub enum Shape {
    Box {
        at: Pos,
        size: Size,
        stroke: Stroke,
        fill: Option<Glyph>,
    },
    Line {
        at: Pos,
        len: u32,
        orientation: Orientation,
        stroke: Stroke,
    },
    Arrow {
        from: Endpoint,
        to: Endpoint,
        stroke: Stroke,
    },
}
```

The set is closed and holds exactly these three (FR-008). Fields are public and a shape is built as
a literal; there is no constructor and no builder.

## `Endpoint`

```rust
pub struct Endpoint {
    pub at: Pos,
    pub leaving: Direction,
    pub head: Glyph,
}
```

## What is not here

- No identity, and nothing that takes one: no `remove`, `replace`, `forward` or `backward`.
- No reader: a diagram answers no question about the shapes it holds.
- No anchor point, no reference, no offset.
- No measuring: nothing asks a diagram how big it is or what window would fit it.
- No rendering, and no glyph catalog anywhere in the surface.
- No re-export of the `monospace_core` types the shapes hold; a caller takes those from the core.

Each of these is a named line in the spec's **Out of scope** table, with the issue it belongs to.

## Using it

```rust
use monospace_core::{Buffer, GlyphCatalog, Pos, Size, Stroke, render};
use monospace_diagram::{Diagram, Shape};

let origin = Pos { x: 0, y: 0 };
let size = Size { width: 20, height: 6 };

let mut diagram = Diagram::new();
diagram.add(Shape::Box { at: origin, size, stroke: Stroke::from("light"), fill: None });

let mut buffer = Buffer::new(origin, size);
diagram.draw(&mut buffer);
let text = render(&buffer, &GlyphCatalog::light(), origin, size);
```

The last two lines are the caller's half of the split: the diagram wrote cells, and turning them
into text needed a catalog the diagram never saw.
