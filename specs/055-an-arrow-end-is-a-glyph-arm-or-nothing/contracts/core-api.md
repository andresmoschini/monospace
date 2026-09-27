# Contract: what `monospace-core` exposes after this slice

The public surface of the core. It is feature 039's
[`public-api.md`](../../039-draw-shapes-instead-of-individual-cells/contracts/public-api.md) with
one field renamed, one type added and no signature changed; that file stays the record of what 039
shipped, and this one is what a caller sees now. Everything here carries rustdoc as it is written.

Field-by-field meaning is in [data-model.md](../data-model.md); why the type is shaped this way is
in [decisions.md](../decisions.md) D2.

## `Endpoint` — changed

```rust
pub struct Endpoint {
    /// The cell the terminal hangs from. The terminal occupies this position itself.
    pub at: Pos,
    /// The direction the arrow leaves this endpoint in. The route's starting position is one
    /// step from `at` in this direction.
    pub leaving: Direction,
    /// What this endpoint contributes to the cell at `at`.
    pub terminal: Terminal,
}
```

`head` is gone and `terminal` stands in its place. This is the slice's one breaking change to a
published surface, and every caller inside this repository moves in the same increment. The field
keeps its position in the struct and its documentation's subject changes from a head to a terminal.

## `Terminal` — new

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Terminal {
    /// One chosen glyph, written as a literal: decided on every side, so nothing composes into it.
    Glyph { glyph: Glyph },
    /// One arm, in the arrow's own stroke, on the side `leaving` names, leaving the other three
    /// sides undecided.
    Arm,
}
```

- Both variants are built by the caller as literals. There is no constructor, no `Default`, and no
  accessor: a caller reads a terminal by pattern matching the public field.
- `Arm` carries nothing, because the side is `leaving`'s own and the stroke is the arrow's. A caller
  cannot name either here, and cannot draw an arm on a side other than the one the arrow leaves in.
- `Clone` because `Shape::draw` takes `&self`; no `Copy`, because `Glyph` owns its text.
- The set is closed at two variants and is not `#[non_exhaustive]`. A third terminal is a new
  variant in a later slice, which is a change to this enum and to the model rather than an invisible
  one.

## Unchanged

- `Arrow { from, to, stroke }`, field for field. `from` and `to` are still `Endpoint`.
- The route is still derived from the two positions and the two leaving directions alone. Nothing
  about a terminal reaches it, so no signature moves and no arrangement draws a different route.
- `Shape`, `Surface`, `Layer`, `Buffer`, `Cell`, `Arm`, `StrokeCell`, `Glyph`, `GlyphCatalog`,
  `GlyphKey`, `Pos`, `Size`, `Direction`, `Orientation`, `Stroke` and `render`.
- `Terminal` is re-exported from the crate root beside `Endpoint`, and a caller takes it from there.

## What is not here

- No third terminal, and nothing said about the cell one would leave.
- No way to ask a terminal what it writes or where. It is a value a shape draws, and no caller in
  this repository needs to interrogate one.
- No `serde`, and no new dependency. The crate that deserializes is `monospace-cli`
  ([ADR-0035](../../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md)).
- No anchor, reference or offset on an endpoint — issue #82's work, and this slice is what makes
  such a cell survivable rather than what adds one.

## Using it

```rust
use monospace_core::{
    Buffer, Direction, Endpoint, Glyph, GlyphCatalog, Layer, Pos, Shape, Size, StampMode, Stroke,
    Terminal, Arrow, render,
};

let origin = Pos { x: 0, y: 0 };
let size = Size { width: 11, height: 3 };

let arrow = Arrow {
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
};

let mut buffer = Buffer::new(origin, size);
arrow.draw(&mut Layer::new(&mut buffer, StampMode::Above));
let text = render(&buffer, &GlyphCatalog::light(), origin, size);
```

One terminal of each kind, which is the shortest thing that shows both constructions. What that
particular mix draws is not one of the spec's pictures, and this file asserts nothing about it: the
arrangements the spec measures and the pictures they draw are in [spec.md](../spec.md) _B1_ and
_B2_.
