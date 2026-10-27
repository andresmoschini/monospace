---
status: draft
date: 2026-10-27
---

# Show the origin and size of the diagram in the status bar

## Why now

The status bar names the application and its version and says nothing about the drawing. A person
who moves a figure past the edge of the region, or opens a description larger than the screen, has
no way to read where the drawing starts or how far it reaches.
[#176](https://github.com/andresmoschini/monospace/issues/176) cannot bound a scrolling viewport
without that answer; it reads the rectangle this change lands and does not restate it.

**The boundary this needs was put in place one merge ago.**
[#240](https://github.com/andresmoschini/monospace/pull/240) made one write operation a `StampSink`
between a drawing and the store, and wrote in `crates/monospace-core/src/buffer.rs` that an adapter
may stand there "measuring what passes, or sending it somewhere else — without the drawing knowing".
This change is that adapter. No shape reports what it covers and the diagram does not measure
itself: the traffic is measured on its way into the buffer, where every figure's reach already
exists — including when the window clips it or another figure decides the cell first.

## Scope

### In

- `monospace-core`: `Footprint`, the smallest and largest coordinate a drawing stamped, and
  `FootprintSink`, the `StampSink` adapter that accumulates each stamp and then hands it on. Both
  are exported from the crate root.
- `monospace`: the frame's buffer is wrapped while the session's diagram is drawn; `DiagramState`
  keeps the last frame's footprint beside the buffer; `status` gains one section and takes the
  footprint as a parameter; `frame` reads it after the region is drawn.
- The contract snapshots of the five frames that carry a status bar.

### Out

- **The scroll, the viewport and the rectangle as a bound.**
  [#176](https://github.com/andresmoschini/monospace/issues/176) decides what bounds scrolling; this
  change answers how far a drawing reaches and shows it.
- **What the session state says** — what has changed and what can be undone.
  [#226](https://github.com/andresmoschini/monospace/issues/226) owns that half of the bar.
- **A shape's own rectangle, a `Shape` method, or a reader on `Diagram`'s shapes.** _Shapes_ in
  [`model.md`](../docs/model.md) withdrew the shape-level extent and says why; the decision below
  keeps it withdrawn and adds no second reader.
- **The command line's pictures and the demonstration's drawing.** The bar is chrome and the command
  line draws no chrome; the demonstration's picture does not move, only the row below it.
- **A section whose width follows its text (D4), and a `Size` as the answer (D2).**

## The decision

**D1 — the footprint is the positions the drawing stamped, not the cells that survived.**

**Answer:** the smallest and largest coordinate any stamp reached, accumulated as the stamp passes
into the buffer. A stamp the window does not hold and a stamp the composition finds already decided
both count. **Why not the cells the buffer holds**: that answer is the intersection of the drawing
with the window, so it changes with the region's size and with the order of the figures; a viewport
bounded by it would move the bound it is bounded by as it scrolled, and a figure wholly behind
another would vanish from the measure. **Why not the ownership record**: _Ownership_ in
[`diagram-model.md`](../docs/diagram-model.md) says a position whose every arm is still `Unset`
belongs to nobody, and that position was still written by a figure that reaches it. **Why not each
shape's own rectangle, taken as a union by the diagram**: the model withdrew that extent because a
shape may be a compositor of pieces or may write positions sparsely, an anchor may be absent, and a
connector's route is computed while drawing — the stamps are where every figure's reach already
exists. **What it costs:** a stamp that renders nothing still widens the answer, so the bar can say
a drawing reaches a cell the picture shows as a space; it asks how far the drawing reached, not what
the screen shows. **Answered by** the maintainer, in the session that wrote this.

**D2 — the footprint is two positions, and not an origin and a `Size`.**

**Answer:** `Footprint { min, max }`, both inclusive, both `Pos`. A consumer that wants a width
derives `max.x - min.x + 1` in an `i32`. **Why not an origin and a `Size`**: a drawing reaching from
`-32 768` to `32 767` spans 65 536 cells and no `u16` holds it, so the one span that does not fit a
`Size` would have to be rejected or saturated by the measurement rather than by the caller that
wants a width. **Why not a `Window`**: a window is chosen before anything is drawn and may hold no
position at all; a footprint exists only once a stamp has reached somewhere, and `None` says so.
**What it costs:** every consumer that wants a size does the arithmetic; this change does it once,
in the bar, and the tests name the `+ 1` that makes one stamped cell `1×1` rather than `0×0`.
**Answered by** the maintainer.

**D3 — the measurement is a sink adapter, and the bar takes the answer as a parameter.**

**Answer:** `FootprintSink` wraps the frame's buffer, accumulates each stamp and then delegates; the
state keeps the answer beside the buffer the frame drew, and `status::draw` gains a footprint
parameter. **Why not a `Diagram::footprint()` asked by the bar**: the diagram has to be drawn
anyway, so asking for the footprint separately draws it a second time on every frame; and the
adapter's doc in the core already names measuring on the way past as the reason the sink exists.
**Why not measuring the buffer after the draw**: D1 — what survived is the picture, not the reach.
**Why not letting the status module hold the diagram or the frame's buffer**: the bar would know the
region and the drawing, and a caller could draw it without the drawing it speaks for. **What it
costs:** the answer is produced during the draw, so a consumer that has to choose its window
_before_ drawing — [#176](https://github.com/andresmoschini/monospace/issues/176) — measures through
the sink into a window of no size first, and owns that second traversal; this change does not name
that call. **Answered by** the maintainer.

**D4 — the section has a fixed width, and a drawing that stamped nothing says `nothing drawn`.**

**Answer:** one section between the name and the version, a fixed number of cells wide, written
`W×H at X,Y`, or `nothing drawn` when the footprint is `None`. **Why not `0×0 at 0,0`**: it is
indistinguishable from a drawing that put one cell at the origin, and `None` and a one-cell
rectangle are different states. **Why not a width that follows the text**: the name's fill absorbs
the leftover, so a width that grew at the fourth digit would move the name's boundary whenever the
drawing did, and the bar would move under the person reading it. **What it costs:** a drawing whose
size and corner are longer than the section is clipped — the plane's own corner,
`65536×65536 at -32768,-32768`, is. **Answered by** the maintainer.

## Model slice

- `docs/model.md` §2 _The buffer_: the paragraph the sink's doc points at. A buffer holds what it
  was given and a drawing can reach past it; a stamp outside the window and a stamp already decided
  both count; the answer is two positions, and `None` is a drawing that stamped nothing.
- `docs/model.md` §7 _Shapes_: the caller the withdrawal anticipated — one that needs to know what
  figures occupy — has arrived, and what it reads is the stamps rather than a method on a shape. The
  withdrawal stands.
- `docs/diagram-model.md` §7 _Drawing_: drawing still measures nothing of its shapes and sizes
  nothing; a sink at the boundary measures what passed, and the window does not decide the answer.
- `docs/diagram-model.md` §8 _Ownership_: the footprint is not the ownership record. A position
  another figure owns, and one whose arms are all `Unset`, still count.
- `docs/diagram-model.md` §11 _Open questions_: _How big is a diagram?_ is answered and leaves the
  list; the answer is prose in §7 and §8.
- `docs/application-model.md` §2 _The frame_: the bar's third section and where its value comes
  from; §3 _The state_: the footprint is kept beside the drawing; §8 _Open questions_: the status
  bar's question keeps [#226](https://github.com/andresmoschini/monospace/issues/226) and gains the
  extent here.
- `crates/monospace-diagram/src/gallery.rs`: the module doc names the withdrawn shape-level
  measurement. It is no longer an `Extent` — nothing in the tree carries that name — and it is not
  this footprint either, which is the drawing's.

## Public surface

```rust
/// The rectangle a drawing reaches: the smallest and largest coordinate stamped into it.
pub struct Footprint {
    pub min: Pos,
    pub max: Pos,
}

/// A `StampSink` that measures the stamps passing through it and hands each one on.
pub struct FootprintSink<'a> { /* private */ }

impl<'a> FootprintSink<'a> {
    pub fn new(sink: &'a mut dyn StampSink) -> Self;
    pub fn footprint(&self) -> Option<Footprint>;
}

impl StampSink for FootprintSink<'_> {
    fn stamp(&mut self, at: Pos, cell: Cell, mode: StampMode, owner: Option<ShapeId>);
}
```

`Footprint` derives `Debug`, `Clone`, `Copy`, `PartialEq` and `Eq`. The fold that grows a footprint
is `pub(crate)`; no arithmetic on the span is public, because D2 leaves that decision to the caller
that wants it. The application's additions are private: `DiagramState::footprint()` and the extra
parameter of `status::draw`.

## Behavior

1. A drawing that stamped nothing has no footprint: the answer is `None`, not a rectangle at the
   origin. One stamp is a one-cell footprint at its own position.
2. The footprint is the union of every stamp: its minimum grows left and up, its maximum right and
   down, on both axes independently.
3. A stamp outside the window, a stamp whose cell another figure already decided, and a stamp whose
   cell decides no side all count. The window, the order of the figures and the picture do not
   change the answer.
4. The bar says `W×H at X,Y`, where `W = max.x - min.x + 1` and `H = max.y - min.y + 1`, with the
   corners as signed decimals and no space around the comma: `46×13 at 0,0`, `4×3 at -4,-3`.
5. A drawing that stamped nothing says `nothing drawn`. The section keeps its width in both cases;
   text longer than that width is clipped.
6. The bar is drawn with the footprint of the frame being drawn. The value is a parameter of
   `status::draw`, so no caller can draw the bar without saying what the drawing reached, and the
   frame reads it after the region, from the draw that produced the screen.
7. The frame's answer is the drawing's and not the interior's: a figure clipped by the window, or
   wholly behind another, still sizes the section as it would with no window at all. The state
   answers `None` until its first `redraw`.
8. The demonstration's picture is unchanged on every row above the status bar, and the five frames
   that carry the bar gain the section. The command line's output does not change.

## Examples

The bar is chrome, not a diagram: `cargo xtask render` draws a description and never a status bar,
so these are strings the section produces rather than generated pictures. The demonstration's
numbers are observed in the building stage's snapshot; the rest is arithmetic, not run until the
tests below.

| Drawing                                                 | Window it was drawn into            | Footprint                    | The bar says    |
| ------------------------------------------------------- | ----------------------------------- | ---------------------------- | --------------- |
| a `4×3` box at `(2, 1)`                                 | `20×10`                             | `min (2,1)`, `max (5,3)`     | `4×3 at 2,1`    |
| the same box                                            | `1×1` at `(2, 1)`                   | the same                     | `4×3 at 2,1`    |
| the same box, drawn into a window of no size            | `0×0`                               | the same                     | `4×3 at 2,1`    |
| a `4×3` box at `(-4, -3)`                               | any                                 | `min (-4,-3)`, `max (-1,-1)` | `4×3 at -4,-3`  |
| a filled `6×4` at `(0, 0)` covering a `2×2` at `(1, 1)` | a window holding only the front box | `min (0,0)`, `max (5,3)`     | `6×4 at 0,0`    |
| a `1×1` box at `(7, 2)` — a box draws nothing below 2   | any                                 | `None`                       | `nothing drawn` |
| nothing                                                 | any                                 | `None`                       | `nothing drawn` |
| the demonstration                                       | the shipped `38×6` interior         | `min (0,0)`, `max (45,12)`   | `46×13 at 0,0`  |

**The second and third rows are the point of the measurement.** The same drawing answers the same
rectangle in a window that holds a corner of it and in one that holds nothing, which is the answer
[#176](https://github.com/andresmoschini/monospace/issues/176) needs: a bound that does not move
when the window does. **The fifth row is why the answer cannot come from the owners**: the box
behind owns no cell at all, and the drawing still reaches where it stands.

**The bar, as text.** Three sections at their fixed widths; the name takes what is left:

```text
monospace        46×13 at 0,0      0.1.0
monospace        nothing drawn     0.1.0
monospace        4×3 at -4,-3      0.1.0
```

**A span that does not fit.** The plane's full corner is `65536×65536 at -32768,-32768`; the section
clips it. A section that grew is what D4 refuses, and that one drawing pays for every frame's
layout.

## What proves it

The names bind the building stage: the core's in `crates/monospace-core/src/footprint.rs`, the
diagram's in `crates/monospace-diagram/src/diagram.rs`, the application's in `state.rs`, `status.rs`
and `frame.rs`.

| Rule | Test                                                                                                                                                                                                                                                                                                                                                              |
| ---- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `an_empty_drawing_has_no_footprint_and_the_first_stamp_is_one_cell`, `an_empty_diagram_has_no_footprint`, `nothing_drawn_is_said_in_words`, `a_state_that_has_not_drawn_a_frame_has_no_footprint`                                                                                                                                                                 |
| 2    | `the_footprint_is_the_union_of_every_stamp`                                                                                                                                                                                                                                                                                                                       |
| 3    | `a_stamp_outside_the_window_writes_no_cell_and_still_counts`, `a_stamp_that_decides_no_side_still_counts`, `a_stamp_below_a_decided_cell_changes_nothing_and_still_counts`, `the_footprint_is_the_same_whatever_window_the_diagram_is_drawn_into`, `moving_a_figure_through_the_order_does_not_move_the_footprint`, `a_figure_wholly_behind_another_still_counts` |
| 2, 3 | `the_footprint_starts_at_the_top_left_of_a_figure_at_negative_coordinates`                                                                                                                                                                                                                                                                                        |
| 3    | `every_stamp_reaches_the_sink_it_wraps`                                                                                                                                                                                                                                                                                                                           |
| 4    | `the_bar_says_how_far_the_drawing_reaches`, `the_bar_says_a_corner_before_the_origin_as_it_is`, `a_single_stamped_cell_is_one_by_one`, `the_status_bar_says_how_far_the_drawing_reaches`                                                                                                                                                                          |
| 5    | `nothing_drawn_is_said_in_words`, `the_state_has_one_section_per_row_of_the_table`                                                                                                                                                                                                                                                                                |
| 6    | `the_status_bar_says_how_far_the_drawing_reaches` — the bar is read from the frame the region was drawn in                                                                                                                                                                                                                                                        |
| 7    | `the_last_frames_footprint_is_the_diagrams`                                                                                                                                                                                                                                                                                                                       |
| 8    | the five snapshots that carry the bar, and `the_demonstration_is_drawn_when_no_path_is_given` unchanged                                                                                                                                                                                                                                                           |

**Measured in the building stage and reported there.** Which snapshots moved and by how many cells,
the bar the shipped demonstration reads, and the gate's result. None of it is claimed here: the
tables are arithmetic, and the string rows are what the tests are written to produce.

## Open questions

**A name for the consumer that chooses its window first.**
[#176](https://github.com/andresmoschini/monospace/issues/176) needs the rectangle before it can
clamp the origin it draws at, and this change measures during a draw, where the window is already
chosen. It can draw through `FootprintSink` into a window of no size — the third row of _Examples_ —
but whether that should be one call on `Diagram`, and where it lives, is not answered here. What
would settle it: #176's first frame after a content change, if a stale answer is visible for one
frame.

**A drawing too large for the section.** The fixed width holds a three-digit size at a three-digit
negative corner and clips the plane's own. Whether the bar should spend more columns, abbreviate, or
say nothing when the text does not fit is a question about a person with a drawing that large. What
would settle it: one.
