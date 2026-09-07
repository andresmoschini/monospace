---
status: implemented
date: 2026-09-07
---

# 0002 — Make a box that can be crossed

## Why now

The front end draws one box and nothing ever touches it. Every side that box does not use is
`Closed`, and `Closed` is a decision: nothing may ever connect here. For a box in a diagram that is
the wrong answer, since a diagram is figures meeting. Nothing catches it while the box stands alone,
because a key treats `Closed` and `Unset` alike and the two spellings render the same character, so
the fault only surfaces when a second figure arrives.

That is what makes this the change to do first. The next slice adds the second stamp mode, and its
whole subject is which figure owns a side where two of them meet. Demonstrating it against borders
that refuse every junction would put two figures cutting holes in each other on the terminal, and
the first thing anyone would suspect is the mode.

Nothing in `monospace-core` changes. This is a slice of the layer above it, which today is the front
end and its box.

## Scope

### In

- The convention, recorded in the model: a figure abstains on the sides it does not own and closes
  only the ones that protect its own interior.
- The box `monospace-cli` draws, rebuilt to that convention.
- The front end drawing two overlapping boxes as well as the single one, so a junction between two
  figures that know nothing of each other is visible on the terminal.

### Out

Every item names where it is handled instead.

- **The `Below` stamp mode.** The next spec. Everything here is stamped with `stamp` exactly as spec
  0001 shipped it, which is what lets that spec be a comparison against something already on screen
  rather than a first sighting.
- **Anything in `monospace-core`.** No public item is added, removed or changed, and no rule of spec
  0001 moves. The convention is about what a caller puts in a cell; stamping and rendering already
  do everything it needs.
- **Filled shapes.** The second box's interior is never stamped, so the first box shows through it.
  A shape that hides what it covers has to stamp its interior, which needs a layer that can say what
  an interior is.
- **A figure in code.** The front end still draws cell by cell, as spec 0001 left it. A type that
  knows its own sides is what would make this convention enforced rather than remembered, and it
  belongs to the layer above the buffer.
- **Everything spec 0001 left out and this one does not name** — degradation, a catalog from more
  than one set, loading sets from a file, walking a region — stays out, with the destinations that
  spec gave.

The near miss is a rule that stops anyone getting this wrong again. There is none, and there should
not be: nothing in the library can tell a border that refuses connections from one that welcomes
them, and `Closed` outwards stays exactly right for a figure that means it. What changes here is one
caller, and the model that caller follows.

## Model slice

This slice amends [`docs/model.md`](../model.md) rather than implementing it. _The cell_ asked for
the inner side of a filled shape's border to be `Closed` and said nothing about the outer one; it
now asks for the outer side to abstain, and says what closing a side a figure does not use actually
means. The amendment is written before the code, as the model's own header requires, and the commit
that carries it says so.

Nothing else moves. No section gains an implementation it did not have after spec 0001, and
_Stamping_ and _Rendering_ are untouched — the second stamp mode is still out, and no cell here
renders through a rule spec 0001 did not already exercise.

## Public surface

None. No item in `monospace-core` is added, removed or changed, and no rustdoc moves. A spec with an
empty public surface is unusual enough that saying so out loud is worth more than leaving the
section short.

## Behavior

1. The box the front end draws is the eight cells in the table below: `Set` on the sides the border
   runs along, `Closed` on the sides facing the box's interior, and `Unset` on the sides facing
   outwards.
2. A box drawn on its own renders exactly as it did in spec 0001, character for character.
3. Where two boxes share a position and each has a cell for it, the second stamp writes the base
   stroke and every side it decides, and the first box's stroke survives on every side the second
   leaves `Unset`. That is spec 0001's rule 5, unchanged; what changes is that a box now has sides
   to leave `Unset`.
4. Where two boxes overlap but only one of them has a cell at a position, that cell stands. An
   interior is not a figure covering something, it is the absence of a figure.
5. `monospace-cli` prints one box, then two overlapping boxes, separated by a blank line.

## Examples

As in spec 0001, `S` abbreviates `Set`, `C` is `Closed`, `U` is `Unset`, and a cell is written
`(base stroke; top, right, bottom, left)`.

**The box.** Four wide and three tall, drawn from these eight cells:

| Cell of the box     | Cell                  | Glyph |
| ------------------- | --------------------- | ----- |
| Top-left corner     | `(light; U, S, S, U)` | `┌`   |
| Top edge            | `(light; U, S, C, S)` | `─`   |
| Top-right corner    | `(light; U, U, S, S)` | `┐`   |
| Left edge           | `(light; S, C, S, U)` | `│`   |
| Right edge          | `(light; S, U, S, C)` | `│`   |
| Bottom-left corner  | `(light; S, S, U, U)` | `└`   |
| Bottom edge         | `(light; C, S, U, S)` | `─`   |
| Bottom-right corner | `(light; S, U, U, S)` | `┘`   |

Spec 0001's box closed each of those outward sides instead of abstaining on them, and its two
interior positions were never stamped. Drawn alone at `(0, 0)` in a buffer of size 4 x 3, both boxes
render the same thing, which is rule 2:

```text
┌──┐
│  │
└──┘
```

They render the same because a key carries a stroke or nothing, and `Closed` and `Unset` both give
nothing. Every one of the eight cells above builds the same key its spec 0001 counterpart did.

**Two boxes.** The same box at `(0, 0)` and again at `(2, 1)`, into a buffer at origin `(0, 0)` of
size 6 x 4, rendered whole. They share four positions. At `(2, 1)` the first box has no cell, so the
second defines it whole; at `(3, 2)` the second has none, so the first one's `┘` stands, which is
rule 4 — the first box showing through an interior that was never stamped. The two positions where
each box has a cell are:

| Position | First box             | Second box            | Result                      |
| -------- | --------------------- | --------------------- | --------------------------- |
| `(3, 1)` | `(light; S, U, S, C)` | `(light; U, S, C, S)` | `(light; S, S, C, S)` → `┴` |
| `(2, 2)` | `(light; C, S, U, S)` | `(light; S, C, S, U)` | `(light; S, C, S, S)` → `┤` |

The second box decides every side it names and abstains outwards, so the first box's stroke survives
on the side facing away from the second — rule 3. Neither box refers to the other, which is what
ADR-0008's third test asks for; spec 0001 carries that test with a segment crossing a border, and
this is the same thing between two borders.

The output is:

```text
┌──┐
│ ┌┴─┐
└─┤┘ │
  └──┘
```

**What the old box produced.** The same two boxes built the spec 0001 way, for contrast. With every
outward side `Closed`, the second box decides all four sides at both shared positions, so `(3, 1)`
becomes `─` and `(2, 2)` becomes `│`:

```text
┌──┐
│ ┌──┐
└─│┘ │
  └──┘
```

Two borders passing through each other and neither joining: the first box's right edge is cut where
the second box's top edge crosses it, and the first box's bottom edge is cut where the second's left
edge does. This is what the slice buys, and it is the picture to compare against when reviewing the
one above.

**The whole output.** The front end prints the single box, a blank line, then the pair. The first
line of the pair is four characters and two spaces, which this document cannot show because the
repository trims trailing whitespace, so exactly:

```text
"┌──┐\n│  │\n└──┘\n\n┌──┐  \n│ ┌┴─┐\n└─┤┘ │\n  └──┘\n"
```

Every cell table and every output above is derived from the rules and from the Light table in
[`glyph-sets.md`](../glyph-sets.md). None of it has been observed: it is what the acceptance list
checks, not the record of a run.

## Acceptance

- [x] `cargo xtask check` passes.
- [x] `cargo run -p monospace-cli` prints the single box, a blank line, and the two overlapping
      boxes.
- [x] `crates/monospace-cli/tests/cli.rs` asserts that whole output, trailing spaces included. Its
      first three lines are byte-identical to what the test asserted before this spec, which is how
      rule 2 gets checked rather than believed.
- [x] No file under `crates/monospace-core/` changes. `git diff --stat` over the branch is the
      check, and it is the one that would catch this slice quietly growing into the next one.
- [x] The box is built once and stamped twice, at two origins, rather than written out cell by cell
      a second time. Two copies of the eight cells would let the pair drift from the single box, and
      rule 2 would then pass for a box nobody draws.
- [x] A comment in the test says which box each junction faces: `┴` and `┤` open towards the first
      box because the second one abstains outwards. It goes there rather than in this spec, where it
      would be explaining rather than specifying.
- [x] `docs/model.md` carries the amendment described under _Model slice_.

## Open questions

**Whether the front end should label what it prints.** Two blocks are told apart by their shape, so
a label buys nothing yet. The next spec puts a second pair on screen, drawn with the other stamp
mode, and two pairs that differ in two characters are not told apart by shape. That is when a label
starts earning its place, and deciding it now would be deciding it without the case that needs it.
