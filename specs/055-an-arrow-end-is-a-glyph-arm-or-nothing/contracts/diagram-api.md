# Contract: what `monospace-diagram` exposes after this slice

The public surface of the diagram layer. It is feature 079's
[`diagram-api.md`](../../079-a-diagram-holds-shapes-and-draws-itself/contracts/diagram-api.md) with
one field renamed and one type named; that file stays the record of what 079 shipped, and this one
is what a caller sees now. Everything here carries rustdoc as it is written.

Why the layer keeps its own `Endpoint` and does not gain its own `Terminal` is in
[data-model.md](../data-model.md).

## `Endpoint` — changed

```rust
pub struct Endpoint {
    /// The cell the terminal hangs from. The terminal occupies this position itself.
    pub at: Pos,
    /// The direction the arrow leaves this endpoint in.
    pub leaving: Direction,
    /// What this endpoint contributes to the cell at `at`.
    pub terminal: monospace_core::Terminal,
}
```

- `head` is gone and `terminal` stands in its place, as it does in the core.
- The field holds the **core's** `Terminal`, and this crate re-exports nothing: a caller takes
  `Terminal` from `monospace_core`, exactly as it already takes the `Pos`, `Direction` and `Glyph`
  the other two fields hold.
- The mirror is kept, and the reason its own rustdoc gives is unchanged — a later change to how an
  endpoint is anchored stays inside this crate. A terminal is not an anchor, and this slice changes
  what a terminal writes rather than where an endpoint hangs.

## Unchanged

- `Diagram` is exactly what feature 080's
  [`diagram-api.md`](../../080-a-shape-in-a-diagram-has-an-identity-and/contracts/diagram-api.md)
  describes — `new`, `add` returning a `ShapeId`, `forward`, `backward` and `draw`. Nothing here
  touches it.
- `Shape`: the same three variants with the same fields. `Arrow { from, to, stroke }` holds two
  `Endpoint`s of this crate's own type.
- The conversion into `monospace_core::Endpoint` drops no parameter, as before. It is one line
  shorter, because the terminal moves across whole.

## What is not here

Everything 079's file lists, unchanged: no reader, no anchor point, reference or offset, no
measuring, and no rendering. This slice adds none of them and removes none.

## Using it

```rust
use monospace_core::{Buffer, Direction, Glyph, GlyphCatalog, Pos, Size, Stroke, Terminal, render};
use monospace_diagram::{Diagram, Endpoint, Shape};

let origin = Pos { x: 0, y: 0 };
let size = Size { width: 11, height: 3 };

let mut diagram = Diagram::new();
diagram.add(Shape::Arrow {
    from: Endpoint {
        at: Pos { x: 2, y: 1 },
        leaving: Direction::Right,
        terminal: Terminal::Arm,
    },
    to: Endpoint {
        at: Pos { x: 8, y: 1 },
        leaving: Direction::Left,
        terminal: Terminal::Glyph { glyph: Glyph::new("►").expect("one glyph") },
    },
    stroke: Stroke::from("light"),
});

let mut buffer = Buffer::new(origin, size);
diagram.draw(&mut buffer);
let text = render(&buffer, &GlyphCatalog::light(), origin, size);
```

The core's own equivalent is in [`core-api.md`](core-api.md), and what the arrangements draw is in
[spec.md](../spec.md).
