---
status: draft
date: 2026-10-29
---

# A shape that draws lines of text

## Why now

The core has a cell that can hold a chosen glyph and no figure that writes a run of them.
[#59](https://github.com/andresmoschini/monospace/issues/59) asked for that figure and named its
limits: no wrapping, no measuring, and a wide glyph occupying one cell. This change adds it, and its
first caller is the box: a description can now label a box, and the label is centred in its
interior.

**This supersedes one line of the wish.** #59 says "no alignment"; a box's text has an interior to
sit in, so the shape gains an **area** and centres in it on both axes. Wrapping, sizing a box to its
text, clipping and display width stay out. The issue body still reads the old scope; this spec is
the agreement that replaces that line. The high-level text shape with an attachment point is a later
change; the area and the centring are what it will reuse.

## Scope

### In

- `monospace-core`: `Text`, a validated block of lines; `TextShape`, the block at a point, centred
  in an `area`; both exported from the crate root.
- `monospace-diagram`: `Shape::Box` gains `text: Option<Text>`, and the box's drawing composes the
  core's `BoxShape` and `TextShape`. A displaced box carries its text unchanged.
- `monospace-description`: a box's optional `text` field, one string with `\n` line breaks, refused
  by name if it holds any other control character.
- Both shipped demonstrations, `crates/monospace/assets/demo.json` and
  `crates/monospace-cli/assets/demo.json`, gain the same labelled box, so the bare run shows the
  shape rather than only a test knowing it:

  ```json
  {
    "kind": "box",
    "id": 27,
    "at": { "x": 0, "y": 13 },
    "size": { "width": 10, "height": 4 },
    "stroke": "light",
    "text": "Hello\nWorld!"
  }
  ```

  `Hello` and `World!` need an interior six cells wide and two tall, and every box the demonstration
  already holds is 4×3 in a window thirteen rows high whose last row is taken. So this is a new row
  and the window grows to hold it, which moves all eight of the demonstration's pictures.

- The tests and the exact pictures in _Examples_; the model amendments below.

### Out

- **`Alignment`**: left, right, top and bottom. The shape centres on both axes; `area` is what a
  later alignment will read.
- **Wrapping, clipping, and a box that sizes itself to its text.** #59 leaves them out; so does this
  change.
- **A text kind in the diagram's closed set** and **a text attached to an anchor**. The box holds
  its own text; a figure that is only text waits for the change that needs it, and with it
  [#89](https://github.com/andresmoschini/monospace/issues/89) and
  [#90](https://github.com/andresmoschini/monospace/issues/90).
- **Display-width measurement.** A grapheme cluster a terminal draws two columns wide occupies one
  cell, as `Glyph` documents; the grid may not line up, and nothing here measures it.

## The decision

**D1 — the core gains a content value and a placed shape, not one value carrying its position.**

**Answer:** `Text` holds the lines and no position; `TextShape` holds `at`, `area` and a `Text`. The
box stores a `Text` and builds the shape where it draws. **Why not one `Text` carrying `at`:** the
box moves the text with its own position and size, so a stored point would be a second copy of where
the block goes and the first thing to go stale. **Why not raw strings:** `Glyph` already carries the
one-cluster, no-control-character invariant, and a shape that split strings while drawing would make
`draw` fallible or silently drop what it could not write. **What it costs:** two names, and a block
copied into the shape at each draw; the building stage measures whether borrowing is worth it.
**Answered by** the maintainer, in the session that wrote this.

**D2 — the placement rule is an area, and the arithmetic is a signed subtraction truncated toward
zero.**

**Answer:** each line starts at `at.x + (area.width - line_width) / 2`, and the first line at
`at.y + (area.height - text.height()) / 2`, both differences computed as signed integers and divided
by two. A zero area is an area of zero, not a sentinel: `0×0` puts the offsets at `-width/2` and
`-height/2`, so the block sits around `at`, and a box with no interior still draws its text. Nothing
is clipped to the area. **Why not saturating subtraction:** it would make `0×0` mean "start at
`at`", a special case, and a zero window should place the text around the point rather than against
its corner. **Why not `div_euclid`:** truncation puts the extra cell on the right and the bottom for
both signs, and a `0×0` area's odd-width block lands on the middle cell; flooring would shift it one
cell left and up. **Why not `Alignment` now:** the only caller asks for the centre, and a closed set
with one member is a field pretending to be a choice. **Why not a required non-zero area:** the
caller that wants the text at a point has no area to give, and zero is the honest spelling of that.
**What it costs:** an area narrower or shorter than the text overflows on both sides, and a box can
write over its own border. **Answered by** the maintainer.

**D3 — the box composes the core text shape, and stamps it after itself.**

**Answer:** the diagram's `Shape::Box` draws its `BoxShape` first and, when it holds a text, a
`TextShape` at the interior's top-left with the interior as its area. `BoxShape` gains no field and
no centring rule. Text and fill may coexist: the fill is drawn first, the text second, and the text
wins where they meet. **Why not widening `BoxShape`:** centring a block is deciding where a figure
goes, which _Shapes_ in [`docs/model.md`](../docs/model.md) leaves above the core, and the box's own
once-per-position property would break as soon as its text could reach its fill or border. **Why not
a separate diagram shape for the text:** the text belongs to the box, moves with it and has no
identity of its own; two shapes would make a caller find and remove both. **Why not partitioning the
fill around the text, or refusing the pair:** partitioning makes the box know the text's footprint
and split its fill into runs, and refusing takes a labelled filled box away from the caller. **What
it costs:** where the text covers the fill or border, those positions are written twice across the
two core figures; no core figure loses its own once-per-position property. **Answered by** the
maintainer.

**D4 — a box with no interior draws its text anyway.**

**Answer:** the area is `(width.saturating_sub(2), height.saturating_sub(2))` and its origin is
`(at.x + 1, at.y + 1)`; a 2×2 box has a 0×0 area, the text centres on the bottom-right corner, and a
1×N box still draws its text. **Why not skipping a box below 3 in either dimension:** the box's own
degenerate rule draws no border below 2, and that rule is the border's; the text is a second figure
and is never silently dropped. **Why not clipping it away when the area is empty:** clipping is out
of this change; the arithmetic already says where the text goes. **What it costs:** text can land on
the border or outside a degenerate box. **Answered by** the maintainer.

**D5 — the description carries the text as one string, validated where it is parsed.**

**Answer:** a box gains an optional `text`; `\n` separates lines and every other control character
refuses the description by field name, using a `deserialize_with` that builds a `Text` exactly as
`fill` builds a `Glyph`. **Why not an array of lines:** the caller types one field and the line
breaks are in it; an array is a second spelling of one text. **Why not accepting any character and
letting the shape drop the ones it cannot write:** a cell holds a glyph and a glyph refuses control
characters, and a parse that succeeds with a picture that is not what was written is the failure
this format already refuses for `fill`. **What it costs:** a CRLF is refused rather than normalized;
the caller writes `\n`. **Answered by** the maintainer.

## Model slice

- [`docs/model.md`](../docs/model.md) §7 _Shapes_, _The initial set_: a fourth figure, the text
  block — a position, an area and lines of literal glyphs; no stroke; one glyph per cell; it centres
  each line in the area and writes no position twice. It may be placed as a piece, which is what the
  box does.
- [`docs/model.md`](../docs/model.md) §9 _Properties worth testing_: a text block writes each
  position once; an empty line writes nothing.
- [`docs/model.md`](../docs/model.md) §10 _Deliberately unresolved_: "Text and diagonals" becomes
  "Diagonals", and the paragraph at the top that says text and diagonals are out is amended to say
  diagonals.
- [`docs/model.md`](../docs/model.md) §3 _A cell can be a literal instead_: "Text needs that" now
  has its consumer; no rule changes.
- [`docs/diagram-model.md`](../docs/diagram-model.md) §7 _Drawing_: a figure may be composed of two
  core figures, the text stamped after the box; where they meet, the later stamp decides, and the
  once-per-position property is the core figure's rather than the composition's.
- `crates/monospace-core/src/shape/text.rs` and `crates/monospace-diagram/src/shape.rs`: the design
  notes these decisions earn, in the modules that depend on them.

## Public surface

```rust
/// A block of text: its lines, each a run of literal glyphs. No position and no stroke.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Text {
    /* private */
}

impl Text {
    /// Splits on '\n'; one `Glyph` per extended grapheme cluster.
    /// `None` when any line holds any other control character.
    pub fn new(content: &str) -> Option<Self>;

    /// The lines, top to bottom.
    pub fn lines(&self) -> &[Vec<Glyph>];

    /// The longest line, in cells.
    pub fn width(&self) -> u16;

    /// The number of lines.
    pub fn height(&self) -> u16;
}

/// The block, its area's top-left at `at`, each line centred in `area`.
#[derive(Debug)]
pub struct TextShape {
    pub at: Pos,
    pub area: Size,
    pub text: Text,
}

impl Shape for TextShape {
    fn draw(&self, surface: &mut dyn Surface);
}
```

`Text` derives `Debug`, `Clone`, `PartialEq` and `Eq` because the diagram's `Shape` derives them;
its fields are private, so a line that holds a control character cannot be built. `TextShape` is a
core shape like `BoxShape`: constructed, drawn and discarded.

The diagram's box gains one field, and the description's box one optional one:

```rust
Box { at: Pos, size: Size, stroke: Stroke, fill: Option<Glyph>, text: Option<Text> }
```

```json
{ "kind": "box", "text": "one\ntwo" }
```

An omitted `text` is no text; the field is `#[serde(default)]` and validated by `deserialize_with`.

## Behavior

1. `Text::new` splits its argument on `'\n'`. Each line is a run of `Glyph`s, one per extended
   grapheme cluster, and a line holding any other control character makes the whole constructor
   answer `None`.
2. `""` is one empty line; `"one\n"` is two lines, the second empty; `"one\ntwo"` is two lines.
   Width is the longest line's glyph count; height is the number of lines.
3. `TextShape` writes one literal cell per glyph. For line `row` and glyph `column`, the position is
   `at.x + dx + column`, `at.y + dy + row`, where `dx = (area.width as i32 - line_width as i32) / 2`
   and `dy = (area.height as i32 - text.height() as i32) / 2`, both divisions truncating toward
   zero. A position that does not fit `i16` is not written, as a line's own positions are not.
4. A zero `area` is a zero area: the offsets are negative and the block is placed around `at`; a
   `0×0` area does not mean "start at `at`". The shape never clips to its area.
5. An empty line writes nothing; a line shorter than the area writes only its glyphs, not spaces to
   fill the area.
6. A box with `text` draws its `BoxShape` first and then a `TextShape` at `(at.x + 1, at.y + 1)`
   with area `(width.saturating_sub(2), height.saturating_sub(2))`. A box below 3 in either
   dimension draws its text too, with a zero area on that axis; an interior origin that does not fit
   `i16` draws no text.
7. A box without `text` draws exactly what it drew before. A box with text and fill draws the fill
   and then the text over it where they overlap; a position the text covers is written twice across
   the two core figures.
8. Displacing a box moves its text with it; the text has no identity, is not a diagram shape of its
   own, and is neither an anchor nor a reference target.
9. A description's `text` is one string; `\n` separates lines; any other control character is
   refused by the field's name. A wide glyph is one cell for the arithmetic, and the row it draws
   may not line up with the rows above and below it.

## Examples

None of these has been observed: this is what the acceptance list checks, not the record of a run.
The pictures are hypothetical until the building stage teaches the description format to carry
`text`; none is a `render` block.

**One line in an 8×5 box at `(0, 0)`, `text: "hi"`.** Interior `6×3`; `dx = 2`, `dy = 1`; the text
is at `(3, 2)`.

```text
┌──────┐
│      │
│  hi  │
│      │
└──────┘
```

**Two lines, `text: "one\ntwo"`.** Interior `6×3`; `dy = 0` (three less two, divided by two),
`dx = 1`; the lines are at `(2, 1)` and `(2, 2)`.

```text
┌──────┐
│ one  │
│ two  │
│      │
└──────┘
```

**A line wider than the interior, a `5×3` box with `text: "hello"`.** Interior `3×1`; `dx = -1`; the
text starts at the box's own left border and covers it, the interior and the right border.

```text
┌───┐
hello
└───┘
```

**A 2×2 box with `text: "x"`.** The area is `0×0` at `(1, 1)`; `dx = 0`, `dy = 0`; the text lands on
the bottom-right corner.

```text
┌┐
└x
```

**A filled `6×3` box with `text: "hi"`.** The fill is drawn first, the text second at `(2, 1)`.

```text
┌────┐
│░hi░│
└────┘
```

**A `0×0` area outside a box, `at: (10, 5)`, `text: "abc"`.** `dx = -1`, `dy = 0`: `a` at `(9, 5)`,
`b` at `(10, 5)`, `c` at `(11, 5)`.

**A three-line block with no area, `at: (5, 5)`, `text: "ABC\nD\nEFG"`.** `dy = -1` for the whole
block, and each line's `dx` is truncated on its own: `-1` for both three-wide lines and `0` for `D`.
The block spans rows 4 to 6 and columns 4 to 6, so `at` is the middle of the cross rather than a
corner. The row and column labels are the grid's, not the shape's.

```text
 0123456789
0
1
2
3
4    ABC
5     D
6    EFG
7
8
9
```

**The same block with a second text shape stamped over it**, `at: (5, 5)`, `area: 6×6`,
`text: "123\n4\n567"`, drawn after the first. The two are separate figures on one surface, so
`(6, 6)` is written twice and the later stamp decides: the `G` is written and then covered by a `1`.
Stamping them the other way round leaves `EFG23` on that row.

```text
 0123456789
0
1
2
3
4    ABC
5     D
6    EF123
7       4
8      567
9
```

**The constructor's edge cases.**

| input       | lines | width | height     |
| ----------- | ----- | ----- | ---------- |
| `""`        | 1     | 0     | 1          |
| `"one"`     | 1     | 3     | 1          |
| `"one\n"`   | 2     | 3     | 2          |
| `"one\n\n"` | 3     | 3     | 3          |
| `"a\tb"`    | —     | —     | — (`None`) |

**A description with a control character.** `"text": "a\tb"` is refused naming `text`;
`"text": "one\ntwo"` parses. No picture: the parser's error is the observable.

## What proves it

The names bind the building stage: `crates/monospace-core/src/shape/text.rs`,
`crates/monospace-diagram/src/shape.rs`, `crates/monospace-description/src/lib.rs`.

| Rule | Test                                                                                                |
| ---- | --------------------------------------------------------------------------------------------------- |
| 1    | `a_text_splits_on_newlines_and_refuses_other_control_characters`                                    |
| 2    | `an_empty_text_is_one_empty_line_and_a_trailing_newline_adds_one`                                   |
| 3    | `a_text_shape_centres_each_line_in_its_area`, `a_text_shape_writes_a_line_at_the_computed_position` |
| 4    | `a_zero_area_places_the_block_around_the_position`, `a_text_shape_does_not_clip_to_its_area`        |
| 5    | `an_empty_line_writes_nothing_and_a_short_line_writes_no_spaces`                                    |
| 6    | `a_box_centres_its_text_in_the_interior`, `a_two_by_two_box_draws_its_text_on_the_corner`           |
| 7    | `a_box_without_text_draws_what_it_drew_before`, `a_filled_box_draws_its_text_over_the_fill`         |
| 8    | `displacing_a_box_moves_its_text`, `a_box_with_text_answers_the_same_anchors`                       |
| 9    | `a_description_refuses_a_control_character_in_text`, `a_description_reads_a_box_with_text`          |

The pictures in _Examples_ are the expected buffers; the tests render and compare them. The
once-per-position property of `TextShape` is held by
`no_text_shape_writes_any_position_more_than_once`.

Two more names, both for the pair _Examples_ draws on one grid:
`a_three_line_block_at_a_zero_area_sits_around_the_point` for the block whose `D` lands on `at`, and
`a_second_text_shape_overwrites_the_first_where_they_meet` for the later stamp deciding at `(6, 6)`
and for the swapped order leaving `EFG23`.

The demonstration is the tenth proof, and the only one that is a run rather than a buffer:

```text
┌────────┐
│ Hello  │
│ World! │
└────────┘
```

`cargo run -p monospace-cli` prints it at the bottom left of every picture it prints, and the window
it draws into is four rows taller to hold it. `the_bare_run_prints_a_labelled_box` holds the rows
the label lands on.

## Open questions

**Whether the text block is copied into the shape at each draw.** `TextShape` owns its `Text`, so
the box clones the block on every draw. A label is small, but a diagram redrawn on every key repeats
the cost. Borrowing the block (`TextShape<'a> { text: &'a Text, .. }`) is compatible with `Shape`;
whether it is worth the lifetime is a question for the building stage's measurement.

**A text that is the whole figure.** A diagram shape that is only text, attached to an anchor, is
the high-level shape the first message named. It needs a position that may be a reference and, for a
top-left attachment, a corner anchor. The core shape is ready for it; this change does not add it.
