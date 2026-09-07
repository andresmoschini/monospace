---
status: draft
date: 2026-09-07
---

# 0001 — Stamp cells into a buffer and render them

## Why now

No domain logic exists. This is the smallest slice that makes the model visible end to end: cells go
into a buffer, the buffer comes out as text, and a diagram appears on the terminal.

It is deliberately smaller than the model. Everything left out — the second stamp mode, degradation,
several glyph sets, loading them from a file — **adds to** what this describes rather than changing
it, so each of those can be its own spec without revisiting this one. That is the property being
tested here as much as the code: whether the model can be delivered in layers.

## Scope

### In

- `Buffer`, a window with an origin and a size, holding an optional cell at each position.
- `Cell`, a base stroke and four arms.
- `Arm`, with its three states and no stroke of its own: every arm of a cell draws in that cell's
  base stroke.
- `stamp`, writing one cell at one absolute position.
- A catalog built from one set: the 15 rules of the Light table, held as data in the library.
- Rendering a rectangle of the buffer to a `String`, resolving each cell by exact lookup only.
- `monospace-cli` printing a box it draws cell by cell through that API.

### Out

Every item names where it is handled instead.

- **The `Below` stamp mode.** A later spec. `stamp` here behaves exactly as `Above` does in the
  model, including respecting the arms a stamp leaves `Unset`. That is what makes the second mode an
  addition rather than a change: if `stamp` overwrote everything instead, adding modes later would
  alter what today's callers get. Two of the three tests
  [ADR-0008](../decisions/0008-compose-overlapping-cells-with-three-state-arms.md) names as carrying
  its decision go with the mode: that front to back with `Below` equals back to front with `Above`,
  and that `Below` on a fully decided cell changes nothing. The third, a segment stamped across a
  border producing a junction with neither figure referring to the other, is the abstaining stamp in
  the examples below, and this spec carries it.
- **Degradation.** [ADR-0009](../decisions/0009-degrade-a-cell-to-its-base-stroke.md), in a later
  spec. Here a cell whose key the catalog does not answer renders as a space, and rule 9 says so on
  purpose rather than by omission.
- **Building a catalog from more than one set, and the order that decides a collision.** A later
  spec. Here a catalog comes from exactly one set, so nothing can collide.
- **Loading sets from a file.** A later spec. The other eight sets already exist as data in
  [`glyph-sets.md`](../glyph-sets.md) and are not built in here.
- **Walking a region of the buffer.** A later spec, when a second consumer asks for it. `render`
  covers its rectangle position by position through `cell`, and what it walks is the requested
  rectangle rather than the buffer — the two need not coincide, and rule 7 is about exactly where
  they do not. An iterator is additive whenever it arrives: a new method, no signature changed and
  no caller affected.
- **Everything above the buffer**: shapes, an input format, placement. The CLI draws its box cell by
  cell, which is the point — it is the first honest test of whether the API is usable.
- **Arms carrying a stroke of their own.** The spec that brings degradation, since the two are one
  idea: an arm can differ from its cell only if something decides what to draw when the combination
  has no character. Until then a cell has exactly one stroke and `Arm::Set` carries no payload.
- **Text, arrows, diagonals, erase.** Out of the model itself, not just out of this slice.

### Why that restriction is faithful

Dropping the per-arm stroke is a restriction on what can be expressed, not a different rule. For
every cell this spec can build, the equivalent cell in the full model renders the same character: if
the key exists both find it, and if it does not, the model moves every connected arm to the base
stroke and arrives at the key that just failed.

So nothing here has to be unlearned. The base stroke is the only stroke a cell has, it is read on
every render, and it is covered by tests from the first commit rather than carried inert until the
spec that gives it meaning.

## Model slice

Implements [`docs/model.md`](../model.md) in full for _The buffer_, _The cell_ and _Stamping_, and
in part for _Strokes, glyph sets and the catalog_ and _Rendering_.

- _Stamping_'s `Below` mode is out; `stamp` implements the `Above` column only.
- _Strokes, glyph sets and the catalog_ is reduced to a catalog built from one set, with nothing
  loaded from a file and no order to settle.
- _Rendering_'s steps 4 and 5 — the degraded lookup — are out. Steps 1, 2 and 3 are in, and so is
  the output shape in the last paragraph.

Nothing here is new. If implementing needs a rule the model does not have, the model changes first.

## Public surface

```rust
pub struct Pos { pub x: i32, pub y: i32 }

pub struct Size { pub width: u32, pub height: u32 }

pub struct Stroke(pub String);

impl From<&str> for Stroke { /* ... */ }

pub enum Arm { Set, Closed, Unset }

pub struct Cell {
    pub base: Stroke,
    pub top: Arm,
    pub right: Arm,
    pub bottom: Arm,
    pub left: Arm,
}

pub struct Buffer { /* private */ }

impl Buffer {
    pub fn new(origin: Pos, size: Size) -> Self;
    pub fn stamp(&mut self, at: Pos, cell: Cell);
    pub fn cell(&self, at: Pos) -> Option<&Cell>;
}

pub struct GlyphKey {
    pub top: Option<Stroke>,
    pub right: Option<Stroke>,
    pub bottom: Option<Stroke>,
    pub left: Option<Stroke>,
}

pub struct GlyphCatalog { /* private */ }

impl GlyphCatalog {
    pub fn light() -> Self;
    pub fn glyph(&self, key: &GlyphKey) -> Option<char>;
}

pub fn render(buffer: &Buffer, glyphs: &GlyphCatalog, origin: Pos, size: Size) -> String;
```

Each of these shapes is deliberate:

- **`Pos` and `Size` rather than loose integers, and rather than one rectangle.** The model already
  separates them: a position is signed and may be negative, a size never is. Splitting them is what
  lets the compiler refuse a size where a position belongs, which a single rectangle type cannot do.
  There is no `Rect` because nothing here stores or passes a rectangle as one value; when something
  does — clipping, the bounds of a shape — it is `Rect { pos, size }` built from these two.
- **Nothing about a size is rejected, so `Buffer::new` cannot fail.** Zero is allowed everywhere: a
  buffer with no width or no height has no positions, every stamp misses it by rule 3, and any area
  rendered over it is spaces by rule 7. Forbidding zero would need an error path, and the spec would
  then owe an answer about whether it panics or returns a `Result` — a cost with no reader.
- **The buffer does not report its own window.** Nothing in this slice reads it back: rule 7 already
  makes the window observable through `render`, and a caller that built the buffer knows what it
  asked for. Accessors are additive whenever something needs them.
- **`render` is a free function, not a method on `Buffer`.** The model says the buffer knows nothing
  about glyphs. A method would put glyph types in `Buffer`'s own interface and make that sentence
  false.
- **`cell` exists so that stamping can be checked without rendering.** Every rule about stamping is
  observable through `render`, so nothing forces the accessor — but a test that reaches the buffer
  only through the renderer depends on the catalog and on the lookup, and then a wrong glyph fails a
  test whose name claims to be about composition. Reading a cell back keeps those two apart.
- **`stamp` returns nothing.** Stamping outside the window is not an error and not interesting; a
  return value would invite callers to branch on it.
- **`Arm::Set` carries no stroke.** The cell's base stroke is the one a reader has to find, and a
  payload here would let a caller build cells this slice cannot render.
- **A key carries a stroke per side, even though every key this slice builds is uniform.** That is
  the shape the model gives it and the shape the data already has in
  [`glyph-sets.md`](../glyph-sets.md), mixing sets included. It is not the inert generality the base
  stroke would have been: all four sides are read on every lookup, they simply happen to hold the
  same stroke while cells have only one. The restriction lives in the function that builds a key
  from a cell, so loading a mixing set later changes no type.
- **A catalog, not a set.** A set is a group of rules as someone writes or loads it; a catalog is
  where they end up. The only thing a set ever decided was precedence, and that is settled while the
  catalog is built, so nothing downstream has to know which set a rule came from.
- **`light()` is the only constructor, and it is named after what the catalog holds.** It selects
  nothing, because there is nothing to select from yet; a catalog has to come from somewhere and
  there is one set. It is provisional: the spec that loads sets replaces it with construction from
  the sets a caller chooses. It is named `light` rather than `built_in` because `light` stays true
  when a second set ships, while `built_in` would have to either change what existing callers get or
  start lying.
- **A stroke is a `String`.** The model says a stroke is only a name, and this is the least
  committed thing that can be one. It costs an allocation per cell, which is invisible at the sizes
  this handles; `&'static str` would avoid it but stops working the moment a set is loaded from a
  file, and an interned id would put a registry in the public API before anything has shown it is
  needed. The trigger to revisit is a measurement, not a feeling. `From<&str>` is part of the
  surface because without it every construction site says `Stroke(String::from("light"))`.
- **`render` returns a `String`.** `docs/brief.md`, under _Scope_, warns against the consuming
  layer's assumptions reaching the core, and this is arguably one: writing into a `fmt::Write` would
  let a caller avoid holding the whole diagram. It is deferred because the abstraction costs more
  than it buys with a single consumer, and the second consumer — the phase 3 TUI — is the one that
  will say whether it is needed.
- **A catalog cannot be counted or enumerated.** `glyph` is the whole interface. What is worth
  asserting is that every key a light cell can produce is answered, which `glyph` alone proves; a
  `len` would only invite a test that counts rules without checking any of them.

Every public item carries rustdoc as it is introduced, as `docs/brief.md` asks under _Non-Functional
Constraints_. `Arm::Unset` documents what it means, since "not mine to decide" is the part a reader
will get wrong.

## Behavior

1. A buffer is created from an origin and a size. A width or height of zero is allowed and gives a
   buffer with no positions; construction cannot fail.
2. Every position in a new buffer is undefined.
3. `stamp` takes an absolute position. A position outside the window leaves the buffer unchanged,
   and reports nothing.
4. Stamping an undefined position defines it: the base stroke and all four arms are taken from the
   stamp, `Unset` arms included.
5. Stamping a defined position writes the stamp's base stroke and every arm it decides. An arm the
   stamp leaves `Unset` keeps whatever the target had.
6. `render` takes an absolute origin and a size, and returns a `String` of exactly `height` lines,
   each of exactly `width` characters, each ending in `\n` — the last line included. Trailing spaces
   are not trimmed. The first line is the row at the origin's `y` and each line after it is one
   greater; within a line the first character is the column at the origin's `x` and each character
   after it is one greater.
7. A position inside the rendered area but outside the buffer's window renders as a space.
8. A position with no cell renders as a space.
9. A cell renders through the exact key of its four arms: the cell's base stroke where the arm is
   `Set`, and nothing where it is `Closed` or `Unset`. If the catalog does not answer that key, it
   renders as a space.
10. `GlyphCatalog::light()` answers every key the Light table in [`glyph-sets.md`](../glyph-sets.md)
    gives a character for, with that character, and answers no other key.
11. `glyph` returns the character for a key the catalog answers, and `None` for one it does not. Two
    keys are the same key when all four sides match, `None` included.
12. The built-in rules are data in the library, not parsed at run time.

## Examples

Throughout, `S` abbreviates `Set`, `C` is `Closed`, `U` is `Unset`, and a cell is written
`(base stroke; top, right, bottom, left)`.

**From a cell to a character.** The cell `(light; S, C, S, C)` — a stroke running up and down, and
nothing sideways — builds this key:

```rust
GlyphKey {
    top:    Some(Stroke("light")),
    right:  None,
    bottom: Some(Stroke("light")),
    left:   None,
}
```

`GlyphCatalog::light().glyph(&key)` returns `Some('│')`, which is the row of the Light table in
[`glyph-sets.md`](../glyph-sets.md) reading `light`, empty, `light`, empty. Every step is a lookup:
nothing inspects the strokes, compares them or falls back, which is what makes degradation a later
addition rather than a rewrite.

**A single junction.** A buffer at origin `(0, 0)` of size 1 x 1, one stamp at `(0, 0)` of
`(light; S, S, S, S)`, rendered from `(0, 0)` at size 1 x 1:

```text
┼
```

**A box.** A buffer at origin `(0, 0)` of size 4 x 3, with eight stamps, rendered over the whole of
it:

| Position         | Cell                  | Glyph |
| ---------------- | --------------------- | ----- |
| `(0,0)`          | `(light; C, S, S, C)` | `┌`   |
| `(1,0)`, `(2,0)` | `(light; C, S, C, S)` | `─`   |
| `(3,0)`          | `(light; C, C, S, S)` | `┐`   |
| `(0,1)`, `(3,1)` | `(light; S, C, S, C)` | `│`   |
| `(0,2)`          | `(light; S, S, C, C)` | `└`   |
| `(1,2)`, `(2,2)` | `(light; C, S, C, S)` | `─`   |
| `(3,2)`          | `(light; S, C, C, S)` | `┘`   |

```text
┌──┐
│  │
└──┘
```

The two interior positions are never stamped, so rule 8 fills them.

This is also the example that pins the direction of both axes, which rule 6 states. An
implementation with `y` growing upward would put `└──┘` on the first line, and one with `x` growing
leftward would emit each row reversed, starting `┐──┌`.

**An abstaining stamp.** A buffer of size 1 x 1 at the origin. Stamp `(light; S, C, C, C)` at
`(0, 0)`, which renders `│`. Then stamp `(light; U, S, C, S)` at the same position. The second stamp
decides three sides and abstains on the top, so the cell becomes `(light; S, S, C, S)`:

```text
┴
```

This is rule 5, and it is the one to get wrong: the top arm survives a stamp that overwrote
everything else.

**A key with no rule.** A buffer of size 1 x 1 at the origin, one stamp of `(double; S, C, C, C)`.
The catalog was built from the light set alone and none of its rules mention `double`, so by rule 9
the whole output is `" \n"`: one space and a newline.

It is written as a literal rather than shown in a block because this repository trims trailing
whitespace, and a block could not hold it honestly.

This example survives the rest of the model. `docs/model.md`, under _Rendering_, says the degraded
lookup also fails when the set for the cell's base stroke was never loaded, so the answer stays a
space once degradation exists — the test is not recording temporary behavior.

**An undefined cell and a closed one.** A buffer at origin `(0, 0)` of size 2 x 1. Stamp
`(light; C, C, C, C)` at `(0, 0)` and leave `(1, 0)` untouched. Rendered over the whole buffer, the
output is `"   "`: two spaces.

They get there by different rules. The first position holds a cell, and its key, with all four sides
empty, is one the light set has no rule for, so rule 9 gives a space. The second holds no cell at
all, so rule 8 does. `docs/model.md` insists the two are not the same thing, and in this slice
nothing can tell them apart; the test records that rather than settling it. It is the first thing
that fails the day a set defines the empty key or degradation arrives, which is when whether they
_should_ differ has to be answered.

**A buffer with no positions.** A buffer at origin `(0, 0)` of size 0 x 0. Stamp
`(light; S, S, S, S)` at `(0, 0)`, which lands outside the window by rule 3 and changes nothing.
Rendered from `(0, 0)` at size 2 x 1, the output is `"   "`: every position is outside the window,
so rule 7 fills both.

**Rendering outside the window.** A buffer at origin `(0, 0)` of size 2 x 1, with
`(light; S, C, S, C)` stamped at `(0, 0)`, rendered from `(-1, 0)` at size 4 x 1. The output is
`" │  \n"`.

Four characters: a space for `x = -1`, outside the window; `│` at `x = 0`; a space at `x = 1` for
the undefined cell; and a space for `x = 2`, outside the window again. The two trailing spaces are
part of the output, and rule 6 keeps them.

## Acceptance

- [ ] `cargo xtask check` passes.
- [ ] `cargo run -p monospace-cli` prints a box, drawn cell by cell through the public API. The
      existing greeting is replaced, so `crates/monospace-cli/tests/cli.rs` changes with it.
- [ ] One test per behavior rule, named after what it asserts, and one per example above. The tests
      for rules 3, 4 and 5 read the buffer back through `cell` rather than through `render`, so a
      fault in the lookup cannot fail a test whose name is about composition.
- [ ] A test renders an undefined position and a position holding a cell with four `Closed` arms,
      and asserts both are spaces. `docs/model.md` asks under _Properties worth testing_ for the
      behavior to be recorded, not decided; this is the test that will fail when it stops being
      true.
- [ ] A test walks the 15 non-empty combinations of four sides carrying `light` and asserts that
      `GlyphCatalog::light()` answers every one. It is the invariant `docs/model.md` names under
      _Properties worth testing_, it is a property rather than a second copy of the table, and it
      catches the failure that would otherwise be silent: a missing rule draws a space and raises
      nothing. The characters themselves are covered by the examples above, through `render`.
- [ ] Every public item has rustdoc. The gate's `doc` step already fails on broken intra-doc links.

## Open questions

**Whether `GlyphKey` should be built positionally.** Four named fields of `Option<Stroke>` are
unambiguous to read and long to write, and a previous attempt at this domain ended up positional for
exactly that reason.

It is left open because the verbosity lands on construction sites rather than on the type, and there
are only three of them: the one function that builds a key from a cell, the built-in table of rules,
and the tests. The table can be a compact `&[(&str, &str, &str, &str, char)]` converted when the
catalog is built, and the tests can have a local helper — neither needs the public type to change.
Most of the length is the `Some(Stroke(...))` around each side rather than the field names, and a
helper removes that either way.

What would settle it: writing those tests and that table. If the named form is unpleasant there, it
is unpleasant with evidence, and the likely answer is not bare positional fields but
`[Option<Stroke>; 4]` indexed by a `Side` — compact to build, named to read, and `Side` is already
in the model's vocabulary waiting to exist in code. A swap of left and right in a positional literal
compiles and produces a mirrored diagram, which is the failure that makes this worth a moment's
thought rather than a coin toss.
