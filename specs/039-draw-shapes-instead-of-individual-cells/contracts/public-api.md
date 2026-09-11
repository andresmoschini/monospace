# Contract: the public API `monospace-core` gains

**Feature**: 039 | **Date**: 2026-09-10

The interface this project exposes is the public API of the `monospace-core` library. Everything
below is added by this feature; nothing already public changes, which is FR-024 and SC-008.

Every item here carries rustdoc as it is introduced — FR-026 and _The core stays portable_ — and the
documentation of each shape says whether it is complete or a fragment, because no type carries that
distinction (research Q2).

## Added to `monospace_core`

```rust
// --- what a shape draws into -------------------------------------------------

/// One write operation and no reader.
pub trait Surface {
    fn stamp(&mut self, at: Pos, cell: Cell);
}

/// A buffer and a stamp mode, bound together. The caller's choice of mode is taken here, once.
pub struct Layer<'a>;

impl<'a> Layer<'a> {
    pub fn new(buffer: &'a mut Buffer, mode: StampMode) -> Self;
}

impl Surface for Layer<'_> { /* ... */ }

// --- what a shape is ---------------------------------------------------------

/// A value describing a figure. It draws, and answers nothing about itself.
pub trait Shape {
    fn draw(&self, surface: &mut dyn Surface);
}

// --- supporting vocabulary ---------------------------------------------------

pub enum Direction { Up, Right, Down, Left }
pub enum Orientation { Horizontal, Vertical }

// --- the three figures, all complete shapes ----------------------------------

pub struct BoxShape {
    pub at: Pos,
    pub size: Size,
    pub stroke: Stroke,
    pub fill: Option<Glyph>,
}

pub struct Line {
    pub at: Pos,
    pub len: u32,
    pub orientation: Orientation,
    pub stroke: Stroke,
}

pub struct Endpoint {
    pub at: Pos,
    pub leaving: Direction,
    pub head: Glyph,
}

pub struct Arrow {
    pub from: Endpoint,
    pub to: Endpoint,
    pub stroke: Stroke,
}

impl Shape for BoxShape { /* ... */ }
impl Shape for Line { /* ... */ }
impl Shape for Arrow { /* ... */ }
```

## Not public

`Side`, `Corner`, `Segment`, `End`, `Border`, `Fill`, `Head` and `Route` are `pub(crate)`. A caller
cannot name a fragment, which is what makes "a library user sees complete shapes" true without a
type to say so.

The consequence ADR-0028 already records stands and is repeated here so a reader of the contract
meets it: `Surface::stamp` takes a `Cell`, so a shape defined **outside** this crate builds its
cells by hand and passes through none of the six cell rules. The vocabulary's guarantee is internal.
Widening it later is additive.

## What the contract promises

- **Uniform drawing** (FR-002, FR-003). A caller cannot tell from `Shape::draw` whether the value
  writes cells or places pieces, and a shape placed as a piece is drawable the same way at the top
  level. There is no depth parameter and no context argument.
- **No central list** (FR-009). Adding a kind of shape is a new type and an `impl Shape`. There is
  no enum, no registry and no factory to extend, so SC-007 is countable: the diff of the commit that
  adds one touches no file defining another shape.
- **A shape is a value** (FR-006). `draw` takes `&self`. Drawing the same shape twice produces the
  same writes.
- **Single pass** (FR-005). `draw` stamps against the surface it is handed. There is no intermediate
  buffer and no layout stage that runs first.
- **Never fails** (FR-017, FR-019). No constructor returns a `Result`, no `draw` returns anything,
  and no combination of otherwise valid parameters panics. A figure whose parameters are degenerate
  draws whatever its general rule yields, which may be nothing.
- **Nothing is asked of a shape** (ADR-0030). The trait has one method. There is no extent and no
  bounding rectangle.

## What a caller writes

```rust
let mut buffer = Buffer::new(origin, size);
let mut layer = Layer::new(&mut buffer, StampMode::Above);

BoxShape {
    at: Pos { x: 0, y: 0 },
    size: Size { width: 6, height: 3 },
    stroke: Stroke::from("light"),
    fill: None,
}
.draw(&mut layer);

let text = render(&buffer, &GlyphCatalog::light(), origin, size);
```

No position inside the figure is named, no cell is constructed, and no corner character appears —
FR-001, and SC-005 for the tests.
