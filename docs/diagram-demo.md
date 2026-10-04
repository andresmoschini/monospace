# What a diagram can draw

Every picture below is **generated**: a `<!-- render: … -->` marker carries a description in JSON,
`cargo xtask render` draws it, and a step of `cargo xtask check` re-draws it and fails if the block
in this file is not what the description produces. None of them is written by hand.

**This is a demonstration, not an inventory.** What a description can say is unbounded in two
directions — a `fill` and a terminal's `glyph` are any single grapheme cluster — so no document can
enumerate the space. What this file does is give **one instance of each decision the format makes**,
so that a reader who has never opened [`description.rs`](../crates/monospace-cli/src/description.rs)
has seen every shape of thing the JSON can say.

That leaves a finite remainder, and it is deliberately not enumerated here either. A document that
tried would either grow without end or claim a coverage it does not have. The remainder is the
subject of a **characterization** instead, where a change is reported rather than reviewed and a
range is covered whole rather than by sample.

It is not the model's document. [`diagram-model.md`](diagram-model.md) states the rules and
[`model.md`](model.md) owns the vocabulary; this one only shows what those rules look like on a
canvas.

## Where this sits against the crate's own gallery

Two carriers exist and they are not interchangeable. They are split by **reach rather than by
preference**: a marker owns a picture in a document, and the gallery inside `monospace-diagram` owns
a picture of a figure. The gallery draws values the crate keeps private — what a `Border` or a
`Head` stamps, and a surface table of every cell's arms — and no JSON reaches any of that. What is
in this file is exactly what a description _can_ reach, which is the other half of the split and the
reason it exists at all.

## The three kinds

A `box` takes a position, a size, a stroke and an optional fill. A `line` takes a position, a length
and an orientation. A `connector` takes two endpoints, and each endpoint is a position, a way of
leaving and a terminal.

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 17, "height": 3 } },
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light", "fill": "░" },
    { "kind": "line", "id": "#2", "at": { "x": 6, "y": 1 }, "len": 4, "orientation": "horizontal",
      "stroke": "light" },
    { "kind": "connector", "id": "#3",
      "from": { "at": { "kind": "point", "x": 12, "y": 1 }, "leaving": "right",
                "terminal": { "kind": "glyph", "glyph": "◀" } },
      "to":   { "at": { "kind": "point", "x": 16, "y": 1 }, "leaving": "left",
                "terminal": { "kind": "arm" } },
      "stroke": "light" } ] }
-->

```text
┌──┐
│░░│  ────  ◀────
└──┘
```

<!-- /render -->

## Strokes

A `stroke` names a glyph table rather than a set of characters, so the same box is drawn in several
different ways. **Five of the nine tables draw a figure on their own:**

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 24, "height": 3 } },
  "next_id": 6,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x":  0, "y": 0 }, "size": { "width": 3, "height": 3 }, "stroke": "ascii" },
    { "kind": "box", "id": "#2", "at": { "x":  5, "y": 0 }, "size": { "width": 3, "height": 3 }, "stroke": "light" },
    { "kind": "box", "id": "#3", "at": { "x": 10, "y": 0 }, "size": { "width": 3, "height": 3 }, "stroke": "light-round" },
    { "kind": "box", "id": "#4", "at": { "x": 15, "y": 0 }, "size": { "width": 3, "height": 3 }, "stroke": "heavy" },
    { "kind": "box", "id": "#5", "at": { "x": 20, "y": 0 }, "size": { "width": 3, "height": 3 }, "stroke": "double" } ] }
-->

```text
+-+  ┌─┐  ╭─╮  ┏━┓  ╔═╗
| |  │ │  │ │  ┃ ┃  ║ ║
+-+  └─┘  ╰─╯  ┗━┛  ╚═╝
```

<!-- /render -->

**The other four are mixing tables, and no figure reaches one on its own.** A shape carries one
stroke and writes all four of its arms in it, so every cell a lone box produces is a single-stroke
key — and `light-double`, `light-heavy`, `light-round-double` and `light-round-heavy` hold only the
_mixtures between_ two tables. A description naming one of them draws nothing at all, at any size,
which is a fact about the model rather than about the format.

They are reached where **two** figures of different strokes share a cell, because a crossing leaves
arms in both. The same crossing twice, the second time with both figures in one stroke:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 5 } },
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 5 },
      "stroke": "light" },
    { "kind": "line", "id": "#2", "at": { "x": 2, "y": 2 }, "len": 6, "orientation": "horizontal",
      "stroke": "double" } ] }
-->

```text
┌──┐
│  │
│ ═╪════
│  │
└──┘
```

<!-- /render -->

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 5 } },
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 5 },
      "stroke": "light" },
    { "kind": "line", "id": "#2", "at": { "x": 2, "y": 2 }, "len": 6, "orientation": "horizontal",
      "stroke": "light" } ] }
-->

```text
┌──┐
│  │
│ ─┼────
│  │
└──┘
```

<!-- /render -->

That is the whole of what a mixing table is for, and it is why nothing in this repository could have
shown one before now: reaching it takes two figures, one stroke each, and one shared cell.

## `fill`

Any single grapheme cluster, written once per interior cell, or omitted for no fill. `░` and `▓` are
conventions the shipped demonstration uses rather than anything the format requires.

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 9, "height": 3 } },
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light", "fill": "░" },
    { "kind": "box", "id": "#2", "at": { "x": 5, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" } ] }
-->

```text
┌──┐ ┌──┐
│░░│ │  │
└──┘ └──┘
```

<!-- /render -->

## A figure too small to draw still answers

A box with either extent below two draws **nothing at all** — there is no interior to close and no
run to make, and there is a rule in the core that says so. What it still does is answer an anchor,
because asking a figure where its side is and asking it to draw are two different questions.

The picture below is two references with the same offset, one hanging from a box one cell wide and
one from an ordinary box. The left arrow is on a border that is not there.

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 16, "height": 3 } },
  "next_id": 5,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 1, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": "#2",
      "from": { "at": { "kind": "reference", "shape": "#1", "anchor": "right",
                        "offset": { "dx": 3, "dy": -1 } },
                "leaving": "right", "terminal": { "kind": "glyph", "glyph": "◀" } },
      "to":   { "at": { "kind": "point", "x": 7, "y": 0 }, "leaving": "left",
                "terminal": { "kind": "arm" } },
      "stroke": "light" },
    { "kind": "box", "id": "#3", "at": { "x": 6, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": "#4",
      "from": { "at": { "kind": "reference", "shape": "#3", "anchor": "right",
                        "offset": { "dx": 3, "dy": 1 } },
                "leaving": "right", "terminal": { "kind": "glyph", "glyph": "◀" } },
      "to":   { "at": { "kind": "point", "x": 15, "y": 2 }, "leaving": "left",
                "terminal": { "kind": "arm" } },
      "stroke": "light" } ] }
-->

```text
   ◀──┬──┐
      │  │
      └──┘  ◀───
```

<!-- /render -->

## An endpoint that hangs from a side

An endpoint's `at` may be a `reference` rather than a `point`: the identity of another shape, one of
its four sides, and a gap from that side. One box carries four connectors, one per side, and
**nothing in the description says where any of those sides is.** Two of them **arrive** at the box
and two **leave** it — the arrowhead is on the box's own border for the first pair and out at the
far end for the second, and which is which is the caller's choice rather than something the anchor
decides. A point would have to be written down for all four, and kept true by hand whenever the box
moved.

The **bottom** and **left** connectors carry an `offset` of one cell, which is what separates them
from the border: the bottom head stands a cell below the box and the left arm a cell to its left, so
neither touches it. The **top** and **right** ones stand on the border itself, where an arriving
head replaces the border's own glyph and a leaving arm composes with it into a junction.

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 22, "height": 9 } },
  "next_id": 6,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 9, "y": 3 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": "#2",
      "from": { "at": { "kind": "reference", "shape": "#1", "anchor": "top" },
                "leaving": "up", "terminal": { "kind": "glyph", "glyph": "▼" } },
      "to":   { "at": { "kind": "point", "x": 11, "y": 0 }, "leaving": "down",
                "terminal": { "kind": "arm" } },
      "stroke": "light" },
    { "kind": "connector", "id": "#3",
      "from": { "at": { "kind": "reference", "shape": "#1", "anchor": "right" },
                "leaving": "right", "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "point", "x": 21, "y": 4 }, "leaving": "left",
                "terminal": { "kind": "glyph", "glyph": "▶" } },
      "stroke": "light" },
    { "kind": "connector", "id": "#4",
      "from": { "at": { "kind": "reference", "shape": "#1", "anchor": "bottom",
                        "offset": { "dx": 0, "dy": 1 } },
                "leaving": "down", "terminal": { "kind": "glyph", "glyph": "▲" } },
      "to":   { "at": { "kind": "point", "x": 11, "y": 8 }, "leaving": "up",
                "terminal": { "kind": "arm" } },
      "stroke": "light" },
    { "kind": "connector", "id": "#5",
      "from": { "at": { "kind": "reference", "shape": "#1", "anchor": "left",
                        "offset": { "dx": -1, "dy": 0 } },
                "leaving": "left", "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "point", "x": 0, "y": 4 }, "leaving": "right",
                "terminal": { "kind": "glyph", "glyph": "◀" } },
      "stroke": "light" } ] }
-->

```text
           │
           │
          ┌┘
         ┌▼─┐
◀────────│  ├────────▶
         └──┘
          ▲
          └┐
           │
```

<!-- /render -->

## The offset is a gap from the side, in both axes

One box, four connectors, four offsets, and **one description** — so nothing here can drift out of
step with anything beside it. Every connector leaves the same right side; two of them stand on the
border and two stand three cells clear of it, and the rows are separated by the vertical amount
rather than by saying where to put anything.

The border column is the tell. In rows one and three the head stands **on** the side, so the
border's own glyph is gone at that cell and the head is all that is there. In row two the head
stands three cells clear and the border runs through untouched. Row four is the same three cells
with the terminal on the **other** end: the head has left the box entirely and the offset now
positions a bare `arm` out in the open, which is why that row ends in a plain `─` where the others
end in a head. Nothing in the description said where the side is; the offset is a gap from wherever
it turns out to be, and it positions the endpoint rather than the arrow.

That is a `glyph` terminal, which is a literal and therefore **replaces** the cell it lands on. An
`arm` terminal instead composes with what is already there and turns the border into a junction —
which is the rule [§6 _Attachment_](diagram-model.md#6-attachment) states, and the two are worth
telling apart.

The two axes are independent and neither is checked against the side it is measured from, so a
negative amount is simply a point on the far side of the anchor.

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 14, "height": 9 } },
  "next_id": 6,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 9 },
      "stroke": "light" },
    { "kind": "connector", "id": "#2",
      "from": { "at": { "kind": "reference", "shape": "#1", "anchor": "right",
                        "offset": { "dx": 0, "dy": -3 } },
                "leaving": "right", "terminal": { "kind": "glyph", "glyph": "◀" } },
      "to":   { "at": { "kind": "point", "x": 10, "y": 1 }, "leaving": "left",
                "terminal": { "kind": "arm" } },
      "stroke": "light" },
    { "kind": "connector", "id": "#3",
      "from": { "at": { "kind": "reference", "shape": "#1", "anchor": "right",
                        "offset": { "dx": 3, "dy": -1 } },
                "leaving": "right", "terminal": { "kind": "glyph", "glyph": "◀" } },
      "to":   { "at": { "kind": "point", "x": 11, "y": 3 }, "leaving": "left",
                "terminal": { "kind": "arm" } },
      "stroke": "light" },
    { "kind": "connector", "id": "#4",
      "from": { "at": { "kind": "reference", "shape": "#1", "anchor": "right",
                        "offset": { "dx": 0, "dy": 1 } },
                "leaving": "right", "terminal": { "kind": "glyph", "glyph": "◀" } },
      "to":   { "at": { "kind": "point", "x": 10, "y": 5 }, "leaving": "left",
                "terminal": { "kind": "arm" } },
      "stroke": "light" },
    { "kind": "connector", "id": "#5",
      "from": { "at": { "kind": "reference", "shape": "#1", "anchor": "right",
                        "offset": { "dx": 3, "dy": 3 } },
                "leaving": "right", "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "point", "x": 11, "y": 7 }, "leaving": "left",
                "terminal": { "kind": "glyph", "glyph": "▶" } },
      "stroke": "light" } ] }
-->

```text
┌──┐
│  ◀───────
│  │
│  │  ◀─────
│  │
│  ◀───────
│  │
│  │  ─────▶
└──┘
```

<!-- /render -->

## Two terminals, and the one difference between them

A terminal is an `arm` or a `glyph`, and the choice decides what happens at the cell the endpoint
stands on. An **arm** composes into whatever is already there: the first row's left end sits on the
box's border and the two figures make a junction, `├`. A **glyph** is one grapheme cluster written
as a literal, and a literal **replaces** the cell it lands on: the second and third rows lose the
border at that cell and show the head instead.

The right-hand ends differ for a second reason worth keeping apart. A connector's `leaving` is its
own and not the anchor's, so an `arm` arriving from the right writes its arm on the side it is
leaving and leaves the other three unset — which is why the first row ends in a bare `─` and the
third in one too, while the second row's `▶` is a literal and closes both ends.

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 16, "height": 12 } },
  "next_id": 5,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": "#2",
      "from": { "at": { "kind": "point", "x": 3, "y": 1 }, "leaving": "right",
                "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "point", "x": 13, "y": 1 }, "leaving": "left",
                "terminal": { "kind": "arm" } },
      "stroke": "light" },
    { "kind": "connector", "id": "#3",
      "from": { "at": { "kind": "point", "x": 3, "y": 5 }, "leaving": "right",
                "terminal": { "kind": "glyph", "glyph": "◀" } },
      "to":   { "at": { "kind": "point", "x": 13, "y": 5 }, "leaving": "left",
                "terminal": { "kind": "glyph", "glyph": "▶" } },
      "stroke": "light" },
    { "kind": "connector", "id": "#4",
      "from": { "at": { "kind": "point", "x": 3, "y": 9 }, "leaving": "right",
                "terminal": { "kind": "glyph", "glyph": "◀" } },
      "to":   { "at": { "kind": "point", "x": 13, "y": 9 }, "leaving": "left",
                "terminal": { "kind": "arm" } },
      "stroke": "light" } ] }
-->

```text
┌──┐
│  ├──────────
└──┘


   ◀─────────▶



   ◀──────────


```

<!-- /render -->

## Overlap and the order that decides it

The last shape in the array is the front-most and decides a shared cell first. The same two boxes
are drawn twice, in opposite order, and the crossing is a different character each time.

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 4 } },
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light", "fill": "░" },
    { "kind": "box", "id": "#2", "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "light", "fill": "░" } ] }
-->

```text
┌──┐
│░┌┴─┐
└─┤░░│
  └──┘
```

<!-- /render -->

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 4 } },
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "light", "fill": "░" },
    { "kind": "box", "id": "#2", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light", "fill": "░" } ] }
-->

```text
┌──┐
│░░├─┐
└─┬┘░│
  └──┘
```

<!-- /render -->

## A reference to a shape the file does not hold

This is the cost of a reference the format accepts silently, and it is worth drawing because it
looks like nothing happened: the connector is simply not drawn, every other shape is untouched, and
the run succeeds. `#7` names nothing in a two-shape file. Asking which shapes could not be drawn is
[#88](https://github.com/andresmoschini/monospace/issues/88)'s, and nothing here reports it.

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 11, "height": 3 } },
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": "#1", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": "#2",
      "from": { "at": { "kind": "reference", "shape": "#7", "anchor": "right",
                        "offset": { "dx": 2, "dy": 0 } },
                "leaving": "right", "terminal": { "kind": "glyph", "glyph": "◀" } },
      "to":   { "at": { "kind": "point", "x": 10, "y": 1 }, "leaving": "left",
                "terminal": { "kind": "arm" } },
      "stroke": "light" } ] }
-->

```text
┌──┐
│  │
└──┘
```

<!-- /render -->
