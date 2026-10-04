---
status: agreed
decided: "#167"
date: 2026-10-04
---

# A reference can be given a gap in the side's own words, as "one cell out"

## Why now

A reference's gap is two screen amounts today, so "one cell out of the bottom" is
`{"dx": 0, "dy": 1}` and "one cell out of the top" is `{"dx": 0, "dy": -1}`: one idea, four numbers,
and a caller holding a gap of one has to know which side it named before it can write anything down.
Issue #143 settled the axes so that a displacement reaches a reference's offsets, and an `out` is
the second amount that says the same gap in the side's own words — added to the offset rather than
put in its place, so no file that exists today moves a cell.

It is this cut before the corners and the centre of issue #90 because "away from the shape" has no
direction at a centre and two at a corner, and the rule written now is the one that has to answer
for all nine.

The caller is [`docs/diagram-demo.md`](../docs/diagram-demo.md): its bottom head and its left arm
stand a cell clear of the border today because the file translates the gap into a screen offset by
hand. That translation is the whole cost.

## Scope

### In

- An `out` written beside a reference's `offset`, in the side's own words, added to the offset.
- The four signs, owned by the diagram crate rather than by whoever reads a file.
- The field on the wire, absent meaning zero.
- An instance of it in [`docs/diagram-demo.md`](../docs/diagram-demo.md) and in the demonstration
  that ships beside it, because that document owes one instance of every decision the format makes
  (`crates/monospace-cli/src/sweep.rs`). The section it belongs in already exists and already
  carries the translation: _An endpoint that hangs from a side_ spells its bottom and left
  connectors with a one-cell `offset`, and those two become an `out` of one. The four signs get the
  treatment that section gives a range — one box, four connectors, one description.

### Out

- **A second spelling for the amount along the side.** `along` is what the offset already says: of a
  reference's two screen amounts, one runs away from the shape and one runs along it, so for a
  bottom anchor `dy` is the first and `dx` is the second, and for a right anchor they are the other
  way round. `along` is therefore the `dx` of a bottom anchor and the `dy` of a right one, and a
  field for it would give one pair of numbers two readings — the conversion issue #83 declined.
  `out` is worth adding because nothing in the offset says which of the two axes is meant; `along`
  is not worth adding because the offset already says it, whatever the side.
- The corners and the centre of issue #90, which are named in _Anchor points_ as not existing yet
  and are that issue's question to settle.
- A reference held by anything but a connector's endpoint, which is issue #89.

## The decision

**D1 — `out` is added to the offset; it never replaces the amount already written there.**

**Answer:** a reference resolves to its anchor's point, plus its `offset`, plus the outward amount
of its `out`, and a file may write either one or both. **Why not** the other way round, with the
`out` taking the axis it is measured along: a caller that wants a head slid along a side, or a point
inside a shape, would be pushed back into naming the screen axes — the one thing `out` exists to
stop it doing — and `out: 0` would stop meaning the reference written today, so the change would
move every file rather than none. **Answered by** the maintainer, in the session that wrote this.

**D2 — the outward amount is turned into the offset when the reference is built, and the reference
stores nothing else.**

**Answer:** the four signs are the diagram crate's rule, reached through a `Reference` constructor
that takes the `out` beside the `offset` and adds it there. What the reference holds afterwards is
the three fields it holds today, so `displaced_by` grows the offset it already grows and nothing
else has to learn that a second spelling exists. **Why not** storing the `out` on the reference:
then `displaced_by` has two readings of one reference and has to know which spelling it was written
in, which is the conversion issue #83 declined — and a displacement of one cell right and one cell
down is not one `out` at all, it is an `out` and an amount along the side, and this change does not
store that second one. **Why not** the arithmetic in `monospace-cli` instead: the four signs are a
rule about a diagram, and a rule the reader holds is a rule the next caller repeats. **Answered by**
the maintainer, in the session that wrote this.

**D3 — `out` is a signed count of cells.**

**Answer:** an `i32`, positive away from the shape and negative into it, so `out: -1` on a bottom
anchor is one cell into the box — a point the `offset` already reaches, spelled the way every other
amount in the format is spelled. **Why not** an unsigned gap: a negative would be the one value in a
format that describes pictures and reports nothing that is an error rather than a point, and the
`offset`'s own amounts are signed and unchecked against the side they are measured from. **Why not**
signed and clamped at zero: the same file would then read two ways. **Answered by** the maintainer,
in the session that wrote this.

## Model slice

- `docs/diagram-model.md` §4 _Positions_, beside the paragraph that defines a reference's offset: a
  paragraph recording that the gap may be written in the side's own words, the four signs, and that
  the reference stores the sum rather than the spelling. It lands there because that section is what
  defines what a reference is, and a rule kept out of it is a rule with one reader.
- `docs/model.md`: none. The cell, a literal glyph terminal and the displacement rules already say
  what this needs, and `Delta` already carries "how far from it a reference stands"
  (`crates/monospace-diagram/src/delta.rs`). Nothing here changes what a terminal replaces.

## Public surface

```rust
// monospace-diagram
impl Reference {
    #[must_use]
    pub fn new(id: ShapeId, anchor: Anchor, offset: Delta, out: i32) -> Self;
}
```

Nothing else appears. `Reference` keeps its three public fields, `Position::displaced_by` keeps its
signature, and `monospace-core` is not touched. On the wire a reference gains one optional key
beside its `offset`, and a file that omits it is a file that wrote `out: 0`:

```json
{
  "kind": "reference",
  "shape": "#1",
  "anchor": "bottom",
  "offset": { "dx": 1, "dy": 3 },
  "out": 1
}
```

## Behavior

1. A reference resolves to its anchor's point, plus its `offset`, plus the outward amount of its
   `out`.
2. `out` is a signed count of cells: positive away from the shape, negative into it.
3. The outward amount is `(0, -out)` from `top`, `(+out, 0)` from `right`, `(0, +out)` from `bottom`
   and `(-out, 0)` from `left`. It moves a point along one axis, and never along the axis the side
   runs in.
4. An `out` of zero is the reference written without the field: the same three fields, the same
   resolution, the same picture.
5. A reference holds the sum and not the `out`, so nothing in it records which of the two spellings
   it was written in.
6. A displacement grows the offset of a reference written with an `out` exactly as it grows any
   other: the point slides and the gap from the border is what it was.
7. An `out` and an `offset` pushing the same axis in opposite directions add, and may cancel.
   Nothing reports it — an unresolved reference is a normal state of a diagram being built, and this
   format reports nothing at all.
8. A reference whose shape the diagram does not hold, or whose anchor point that shape does not
   answer, resolves to nothing and draws nothing whatever its `out` is.
9. A line answers the four centres of a flat box, so on a horizontal line the top centre and the
   bottom centre are one point — and an `out` from either of them moves it one cell clear of that
   line, one above it and one below it, because the two sides face away from each other and a
   one-cell-tall figure has no interior for them to share.
10. An unknown key inside a reference is dropped in silence today, which is why the field has to
    exist before any file spells it: `"out": 1` read by a build without the field draws the point on
    the border and says nothing at all.

## Examples

**The case the issue names.** A box three cells wide and one cell tall at (1, 1) — the width is what
puts its bottom centre on (2, 1), and a box one cell tall has the same centre on its top and its
bottom — with a reference to that side, an `offset` of `(1, 3)` and an `out` of `1`:

```text
(2, 1) + (1, 3) + (0, 1) = (3, 5)
```

With `x` the point of the side and `y` the point the reference resolves to, and the box itself not
drawn, the way the issue leaves it:

```text
......
..x...
......
......
......
...y..
```

None of this has been observed. It is the arithmetic of rules 1 to 3, not the record of a run.

**The four signs**, for a box five cells wide and three tall at (0, 0) with no `offset` and an `out`
of `1`:

| Anchor   | The side's point | `out: 1` adds | The point |
| -------- | ---------------- | ------------- | --------- |
| `top`    | (2, 0)           | (0, -1)       | (2, -1)   |
| `right`  | (4, 1)           | (+1, 0)       | (5, 1)    |
| `bottom` | (2, 2)           | (0, +1)       | (2, 3)    |
| `left`   | (0, 1)           | (-1, 0)       | (-1, 1)   |

**Both answers to D1, drawn.** The same box and the same head, once standing on the border and once
a cell clear of it. A `glyph` terminal is a literal, so the first one is written over the border's
own `─` at (2, 2), which is what a caller reaching for a gap is reaching past:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 5 } },
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 5, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": "#2",
      "from": { "at": { "kind": "reference", "shape": "#1", "anchor": "bottom" },
                "leaving": "down", "terminal": { "kind": "glyph", "glyph": "▲" } },
      "to":   { "at": { "kind": "point", "x": 2, "y": 4 }, "leaving": "up",
                "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ] }
-->

```text
┌───┐
│   │
└─▲─┘
  │
  │
```

<!-- /render -->

And with the gap one cell out of the same side, at (2, 3), where the border is a border again:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 5 } },
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 5, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": "#2",
      "from": { "at": { "kind": "reference", "shape": "#1", "anchor": "bottom",
                        "offset": { "dx": 0, "dy": 1 } },
                "leaving": "down", "terminal": { "kind": "glyph", "glyph": "▲" } },
      "to":   { "at": { "kind": "point", "x": 2, "y": 4 }, "leaving": "up",
                "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ] }
-->

```text
┌───┐
│   │
└───┘
  ▲
  │
```

<!-- /render -->

Both markers spell the gap with the `offset` it stands for, because the deciding stage is the one
before the field exists. The building stage re-spells them as `out: 0` and `out: 1`, and neither
picture moves — which is rule 4 drawn rather than asserted.

## What proves it

None of these tests exists yet; the deciding stage adds no code. The names are what the building
stage is held to.

| Rule | Test                                                                                          |
| ---- | --------------------------------------------------------------------------------------------- |
| 1    | `a_reference_with_an_out_resolves_to_its_side_moved_by_the_offset_and_by_the_out`             |
| 2    | `a_negative_out_is_a_point_into_the_shape_rather_than_an_error`                               |
| 3    | `the_four_signs_are_the_outward_normal_of_the_side_they_are_named_from`                       |
| 4    | `an_out_of_zero_is_the_reference_written_without_it`                                          |
| 5    | `a_reference_cannot_be_told_which_of_the_two_spellings_it_was_written_in`                     |
| 6    | `a_displacement_grows_the_offset_of_a_reference_written_with_an_out_as_it_grows_any_other`    |
| 7    | `an_out_and_an_offset_against_each_other_add_and_nothing_reports_the_cancellation`            |
| 8    | `an_out_on_a_reference_that_resolves_to_nothing_still_draws_nothing`                          |
| 9    | `an_out_from_the_top_or_the_bottom_of_a_horizontal_line_moves_the_same_one_cell`              |
| 10   | the description reader, whose silence about an unknown key is what makes rule 10 worth a test |

The last two pictures in `## Examples` are held by the render markers themselves:
`cargo xtask render` rewrites them, and a picture that moved without a change here is the failure.

## Open questions

- Whether a corner or the centre takes an `out` at all is issue #90's question and is written down
  there: "away from the shape" has no direction at a centre and two at a corner, so an `out` there
  is either meaningless or a second meaning of the field, and #146 does not settle which.
