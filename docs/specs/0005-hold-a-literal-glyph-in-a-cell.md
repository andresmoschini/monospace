---
status: draft
date: 2026-09-07
---

# 0005 — Hold a literal glyph in a cell

## Why now

A cell draws what its arms imply, and there are two things that cannot be said that way. A letter is
one of them. The other is a fill that hides what it covers: today a box's interior is simply never
stamped, so the figure behind shows straight through it, which is what the two overlapping boxes on
the terminal have been doing since spec 0002.

The nearest thing available is a cell with four `Closed` arms, which renders as a space because the
Light table has no rule for the empty key. That works, and spec 0001 already wrote down why it is
fragile: it is the first thing that breaks the day a set defines the empty key. It also can only
ever be a space — never a shade, a dot, or anything chosen.

Spec 0004 built the type this needs, so this slice spends it rather than inventing it. It comes
before anything to do with text because text is many literals with a figure above them deciding
where they go, and this is one literal with nothing above it.

## Scope

### In

- `Cell` as one of two things: a base stroke with four arms, or one literal `Glyph`.
- `StrokeCell`, which is what today's `Cell` becomes.
- The composition rules for the four combinations of kinds, in both stamp modes.
- `Cell::is_decided` answering for a literal.
- `render` answering a literal with its glyph, without building a key.
- The front end filling both boxes, so the pair shows occlusion as well as junctions.

### Out

Every item names where it is handled instead.

- **Text.** A later spec. A word is many literals and something above the buffer deciding where each
  one goes; this slice has no opinion about what puts a glyph anywhere.
- **A figure in code.** As in spec 0002, the front end still stamps cell by cell, and a filled box
  is two more stamps rather than a new kind of thing in the library.
- **Arms on a literal.** Deliberately not expressible: a literal composes as four `Closed` sides, so
  a letter is something a stroke stops against rather than joins. Wanting to connect into one would
  be a different shape for a cell and a new record, not a change here.
- **Degradation.** [ADR-0009](../decisions/0009-degrade-a-cell-to-its-base-stroke.md), still
  unimplemented, and now with one fewer case to worry about: a literal never reaches a lookup.
- **Column width.** [ADR-0019](../decisions/0019-represent-a-glyph-as-a-grapheme-cluster.md) settled
  it. This is the first slice that can put a wide glyph on the terminal, and it does not: the fill
  it draws is one column.
- **Everything spec 0001 left out and this one does not name** — a catalog from more than one set,
  loading sets from a file, walking a region — stays out, with the destinations that spec gave.

The near miss is that a filled box looks like a new capability in the library. It is not. What the
library gains is a cell that can hold a chosen glyph; "filled" is a word for what the front end does
with two of them.

## Model slice

Implements _The cell_'s new subsection, the three new rows of _Stamping_'s table, and the sentence
after _Rendering_'s numbered list. All three were written into [`docs/model.md`](../model.md) in the
commit before this spec, which is the model changing first as it is supposed to.

_Rendering_'s numbered list is untouched, so spec 0001's reference to its steps 4 and 5 still points
where it did. Nothing else moves: no glyph rule changes, no key is built differently, and the
degraded lookup is still out.

## Public surface

```rust
pub struct StrokeCell {
    pub base: Stroke,
    pub top: Arm,
    pub right: Arm,
    pub bottom: Arm,
    pub left: Arm,
}

pub enum Cell {
    Strokes(StrokeCell),
    Literal(Glyph),
}

impl From<StrokeCell> for Cell { /* ... */ }

impl Cell {
    pub fn is_decided(&self) -> bool;
}
```

- **A sum, not a field.** A cell is one of two things, and a struct carrying both a stroke and an
  optional glyph would make "which one wins" a question every reader has to ask and nobody has to
  answer. The cost is that `Cell` stops being a struct with public fields: every construction site
  changes, and everything that reads one matches on it.
- **`StrokeCell` is a named struct rather than fields on the variant.** It keeps the merge of two
  stroke cells at the signature it already has, and it lets the function that builds a key keep
  taking exactly what it takes today. A variant with inline fields would push both through a
  pattern.
- **`From<StrokeCell> for Cell`.** The front end builds nine stroke cells per box; `.into()` is what
  keeps that from doubling in width. Nothing converts the other way, because it would have to fail.
- **`Cell::is_decided` widens rather than moves.** For a stroke cell it is the four arms, as
  [ADR-0017](../decisions/0017-ask-the-cell-whether-it-is-decided.md) settled; a literal is decided
  on all four sides by definition, so it answers `true`. `stamp` skipping a decided target under
  `Below` then skips a literal target for free, which is exactly the behavior the table asks for.
- **A literal carries no arms in the type.** The four `Closed` sides exist in the merge rules, not
  in memory. Storing them would let a caller write a literal with an open side, which is a state the
  model does not have.

Every public item carries rustdoc. `Cell`'s says which of the two kinds wins where they meet, since
that is the sentence a reader will otherwise reconstruct from the merge code.

## Behavior

1. A literal composes as a cell whose four sides are `Closed`. It has no arms of its own and nothing
   connects into it.
2. Stamping any cell onto an undefined position defines it entirely, in either mode, as spec 0001's
   rule 4 already says.
3. Stamping a literal `Above` a stroke cell replaces it: the position holds the literal.
4. Stamping a literal `Below` a stroke cell leaves it a stroke cell, and closes every side the
   target had `Unset`.
5. Stamping a stroke cell `Above` a literal replaces it: the position holds the stroke cell, and
   every side the stamp leaves `Unset` comes out `Closed`, inherited from the literal.
6. Stamping a stroke cell `Below` a literal changes nothing.
7. Stamping a literal `Above` a literal replaces it. Stamping one `Below` a literal changes nothing.
8. `Cell::is_decided` is `true` for every literal, and unchanged for a stroke cell.
9. A literal renders as its glyph. No key is built for it and the catalog is not consulted.
10. Stamping a list of figures front to back with `Below` gives the same buffer as stamping the same
    list back to front with `Above`, with literals in the list as well as stroke cells.

Rule 10 is spec 0003's rule 8 restated over a wider domain, and rules 4 and 5 are what keep it true.
Without the `Closed` sides a literal would be opaque in one order and transparent in the other; the
example below is the case that shows it.

## Examples

As in spec 0001, `S` abbreviates `Set`, `C` is `Closed`, `U` is `Unset`, and a stroke cell is
written `(base stroke; top, right, bottom, left)`. A literal is written `'x'`.

**The four combinations.** One position, a target and a stamp, in both modes:

| Target                | Stamp                 | `Above`               | `Below`               |
| --------------------- | --------------------- | --------------------- | --------------------- |
| `(light; U, S, S, S)` | `'A'`                 | `'A'`                 | `(light; C, S, S, S)` |
| `'A'`                 | `(light; U, S, S, S)` | `(light; C, S, S, S)` | `'A'`                 |
| `'A'`                 | `'B'`                 | `'B'`                 | `'A'`                 |
| `(light; U, S, S, S)` | `(light; S, C, C, C)` | `(light; S, C, C, C)` | `(light; S, S, S, S)` |

The last row is spec 0003 unchanged, and it is there so the first three are read against something
familiar. The two `(light; C, S, S, S)` results are rules 4 and 5: the same cell reached from either
side, which is the whole point of them.

**Why the closed sides matter.** Three figures at one position, front to back: `S1` is
`(light; U, S, S, S)`, `L` is `'A'`, `S2` is `(light; S, S, S, S)`.

| Front to back with `Below` | Cell after            | Back to front with `Above` | Cell after            |
| -------------------------- | --------------------- | -------------------------- | --------------------- |
| `S1`                       | `(light; U, S, S, S)` | `S2`                       | `(light; S, S, S, S)` |
| `L`                        | `(light; C, S, S, S)` | `L`                        | `'A'`                 |
| `S2`                       | `(light; C, S, S, S)` | `S1`                       | `(light; C, S, S, S)` |

Both end at `(light; C, S, S, S)`, which renders `┬`. Take rule 4 away — let a literal that loses
under `Below` do nothing at all — and the left column ends at `(light; S, S, S, S)`, which renders
`┼`, because `S2` reaches through the literal to fill a side that the other order had already
sealed. Two different characters from the same three figures. That is the failure rules 4 and 5
exist to prevent, and it is what rule 10's test would catch.

**A filled box.** The box of spec 0002, with its two interior positions stamped `'░'`:

| Cell of the box        | Cell         | Renders  |
| ---------------------- | ------------ | -------- |
| The eight border cells | as spec 0002 | `┌─┐│└┘` |
| Interior               | `'░'`        | `░`      |

Drawn alone at `(0, 0)` in a buffer of size 4 x 3 it renders:

```text
┌──┐
│░░│
└──┘
```

The two interior positions were undefined before this slice and rendered as spaces. They are the
only difference, and by rule 2 the mode does not affect them.

**Two filled boxes.** The same box at `(0, 0)` and again at `(2, 1)`, into a buffer at origin
`(0, 0)` of size 6 x 4, rendered whole. Four positions are shared, and now all four differ between
the modes:

| Position | First box             | Second box            | `Above` | `Below` |
| -------- | --------------------- | --------------------- | ------- | ------- |
| `(2, 1)` | `'░'`                 | `(light; U, S, S, U)` | `┌`     | `░`     |
| `(3, 1)` | `(light; S, U, S, C)` | `(light; U, S, C, S)` | `┴`     | `├`     |
| `(2, 2)` | `(light; C, S, U, S)` | `(light; S, C, S, U)` | `┤`     | `┬`     |
| `(3, 2)` | `(light; S, U, U, S)` | `'░'`                 | `░`     | `┘`     |

`(2, 1)` and `(3, 2)` are the new ones and they are the two halves of the same story: with `Above`
the second box's corner sits on the first box's fill and its own fill covers the first box's corner;
with `Below` the first box's fill hides the second box's corner and the first box's corner survives
the second box's fill. `(3, 1)` and `(2, 2)` are spec 0002 and spec 0003 unchanged — filling an
interior does not touch where two borders cross.

With `StampMode::Above`, the second box is in front:

```text
┌──┐
│░┌┴─┐
└─┤░░│
  └──┘
```

With `StampMode::Below`, the first one is:

```text
┌──┐
│░░├─┐
└─┬┘░│
  └──┘
```

The first line of each is four characters and two spaces, which this document cannot show because
the repository trims trailing whitespace, so the whole output is exactly:

```text
"┌──┐\n│░░│\n└──┘\n\nAbove:\n┌──┐  \n│░┌┴─┐\n└─┤░░│\n  └──┘\n\nBelow:\n┌──┐  \n│░░├─┐\n└─┬┘░│\n  └──┘\n"
```

Every cell and every output above is derived from the rules and from the Light table in
[`glyph-sets.md`](../glyph-sets.md). None of it has been observed: it is what the acceptance list
checks, not the record of a run.

Rules 4 and 5 are invisible in these pictures — at `(2, 1)` under `Above` and `(3, 2)` under
`Below`, a side coming out `Closed` rather than `Unset` renders the same either way, because a key
carries a stroke or nothing. The picture cannot show the difference; the three-figure example above
is what does, and rule 10's test is what keeps it.

## Acceptance

- [ ] `cargo xtask check` passes.
- [ ] `cargo run -p monospace-cli` prints the three blocks above: a filled box, the filled pair
      stamped `Above`, the filled pair stamped `Below`. `crates/monospace-cli/tests/cli.rs` asserts
      the whole output, trailing spaces included.
- [ ] The two unfilled pairs are gone rather than kept alongside. That answers the question spec
      0003 left open about the front end growing a block per slice: a block came out because the new
      one shows everything it showed — both junctions are still there — and one thing more.
- [ ] A test per behavior rule 1 to 9, named after what it asserts, reading the buffer back through
      `cell` per [ADR-0011](../decisions/0011-expose-cell-for-testing-stamping.md).
- [ ] A test for rule 10 stamps the three figures above front to back with `Below` and back to front
      with `Above`, into two buffers, and asserts the cell at the shared position is equal in both.
      It is spec 0003's equivalence test with a literal added to the list, and it is the one that
      fails if a literal stops closing the sides it loses.
- [ ] The box is still built once and stamped twice, as spec 0002's acceptance list required, with
      the fill part of the same function.
- [ ] Every changed public item has rustdoc, and `Cell`'s says which kind wins where they meet.

**Two commits, in this order.** The first renames today's `Cell` to `StrokeCell` and introduces
`Cell` as a sum with one variant used, threading both through the buffer, the renderer and the front
end. Behavior does not move and no test is added or removed, though existing tests are rewritten to
name the new type. The second adds the `Literal` variant with its rules, the render branch and the
fill. Splitting it this way keeps the second commit's diff equal to what a literal actually costs,
instead of burying it under a rename that touches every construction site in the workspace.

## Open questions

**What a fill character should be when nobody chooses one.** `'░'` is a choice this slice makes for
a demonstration, and it is the first character the project draws that is not a box-drawing line.
Text will want a space, a filled shape in a real diagram will probably want one too, and a shade is
the one that shows on a terminal what a space cannot. What would settle it: the slice that gives
figures a way to say what they are filled with, which is where the default belongs rather than here.

**Whether `Cell` should be constructible only through functions.** The sum is public and so are its
variants, so `Cell::Literal(glyph)` is available to anyone — which is fine, because `Glyph` already
validated what matters. It stops being fine if a later cell kind has an invariant across variants.
What would settle it: the third kind of cell, if there is ever one.
