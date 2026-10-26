---
status: draft
date: 2026-10-26
---

# Scroll, bounded by what the diagram occupies

## Why now

The view is fixed at `(0, 0)`: [#175](https://github.com/andresmoschini/monospace/issues/175) left
the origin there and #177 and #179 have drawn through it since, so a figure moved past the edge is
not drawn and the arrow that put it there is the way back only while the person remembers which one
it was (D2 of #179). A viewport that moves is the other answer to the same cost, and it is what
makes the application usable on a diagram larger than the region.

**Nothing measures a diagram, and that is the part that has to be answered first.** `Diagram`
answers `new`, `numbered_from`, `add`, `add_under`, `forward`, `backward`, `get`, `remove`,
`replace` and `draw`, and none of them says how far the drawing reaches, so a viewport with nothing
to base it on scrolls into blank space and never stops. §7 of
[`diagram-model.md`](../docs/diagram-model.md) holds that the diagram measures nothing and that the
window is the caller's, and §11 asks _How big is a diagram?_ with its own answer already written:
what would settle it is a consumer that has to choose a window with nothing to base it on. A
scrolling viewport is that consumer, and it is the first one: the command line is given its window
by `--size` and a render marker.

**The model also says what a bound means for a sparse figure, and leaves that here.** _Shapes_ in
[`model.md`](../docs/model.md) withdrew a shape's own extent and named the caller that would bring a
rectangle back — one that needs to know what another shape occupies, "routing around an obstacle, or
sizing a canvas to its content" — and says that layer "is also the first layer to have to decide
what a bound means for a figure whose written positions are sparse". D1 and D2 are that decision,
and the withdrawal stands: no shape gains a method.

**The view opens at `(0, 0)`, and the first screen is the other half of the bound.** The plane's
coordinates are signed (`Pos` is an `i16`), so a drawing may lie left of or above the origin as well
as past the screen's right and bottom edges. The content is therefore the smallest rectangle holding
the drawing **and the window at `(0, 0)`**: the view opens on the first screen, goes left to reach
what is negative and right to reach what is beyond it, and a drawing inside the first screen does
not scroll at all. The issue's title names the drawing's half, because that is the half nothing
could answer. The wheel and the scrollbar move the view; the keyboard is
[#237](https://github.com/andresmoschini/monospace/issues/237)'s, because which keys move it and how
far a press moves is a rule this cut does not take.

## Scope

### In

- `monospace-diagram`: `Diagram::bounds` and the `Bounds` it answers, measured by drawing the shapes
  into a surface that records the positions and keeps no cell.
- `monospace`: the origin becomes the content's first cell plus the offset of a `ScrollState` per
  axis, and the wheel and the bar move it. The content is the drawing union the first screen. The
  bar is the rat helper's (`rat_widget::scrolled`), drawn on the region's border, per axis, only
  when the content does not fit (`Collapse`).
- The demonstration, its snapshots and every picture already in a tracked document: unchanged,
  because the demonstration's drawing fits its interior and the bar is hidden.

### Out

- **The keyboard.** [#237](https://github.com/andresmoschini/monospace/issues/237) decides which
  keys move the view and how far a press moves; this slice has no key for it, and the plain arrows
  stay what #179 made them.
- **An indicator of the view anywhere but the border.** A mark in the interior or a number in the
  status bar is not this change; the status line is
  [#226](https://github.com/andresmoschini/monospace/issues/226)'s.
- **A bound taken from the figures' declared geometry.** D1: the measurement is the drawing.
- **Keeping a rectangle beside the session between frames.** D1's cost: nothing is cached, so
  nothing can be stale.
- **Paging, a wheel step of one cell, fitting the whole drawing, and zoom.** Each is a second rule
  about how far an input moves or a change to how a cell is drawn; the wheel moves the page's tenth,
  as the helper suggests.
- **Building before #179's building stage merges.** It edits the same handler and state this change
  does, and the change is cut after it.

## The decision

**D1 — what "occupies" means: the drawing, not the figures.**

**Answer:** the smallest rectangle holding every position the shapes write, found by drawing them
into a surface that records the positions and keeps no cell. A shape still reports nothing about
itself (_Shapes_ in `docs/model.md`), and the diagram asks the drawing it already knows how to make.
**Why not** a rectangle from each kind's declared geometry — a box's `at` and `size`, a line's `at`
and `len`, a connector's two resolved endpoints: it is a second reader of what each kind writes, and
the two drift the first time a kind writes something its declaration does not describe, which three
of them already do — a box narrower than two cells writes nothing, a line of length zero writes
nothing, and a connector whose endpoint does not resolve writes nothing, and a declared rectangle
would claim all three. **Why not** drawing into a buffer and scanning it: the bound is what the
buffer's size would have to be, so the question is circular. **What it costs:** the measurement is a
second pass over the shapes, and caching it would need a freshness rule nothing else in the state
has. **Answered by** practice: it is the only reading under which a shape still answers nothing.

**D2 — what a bound means for a figure whose written positions are sparse.**

**Answer:** the smallest rectangle around the cells, so a connector's elbow contributes the empty
corner inside its rectangle, and a figure that writes nothing contributes nothing. A diagram that
writes nothing answers `None` rather than an empty rectangle at `(0, 0)`. **Why not** a set of
cells, one rectangle per figure, or a convex hull: the caller picks a window, a window is a
rectangle, and turning cells into one would be the caller's work before it could choose anything.
**Why not** an empty rectangle for a drawing that wrote nothing: it is a value that says "here"
about a picture that is not there, and every caller would need a second rule to tell it from a
one-cell drawing. **Answered by** practice: the model left the question here, and the smallest
rectangle is the answer that keeps the caller's arithmetic a clamp.

**D3 — where the answer lives and what it is called.**

**Answer:** `monospace_diagram::Bounds`, `Diagram::bounds(&self) -> Option<Bounds>`, and `at` and
`size` the core's own `Pos` and `Size`. The application names neither the type nor the crate: it
asks the value `session.diagram()` answers for its bounds, as it already asks it to `draw`, and
keeps nothing of the answer between frames. **Why not** a bare `(Pos, Size)`: `Buffer::new` takes
the pair because it is building one, while an answer with no name is a public signature a reader has
to learn, and the model has to name the rectangle in any case. **Why not** the rectangle in
`monospace-core`: `geometry` keeps positions and sizes apart on purpose, and a rectangle the diagram
answers is a word of the diagram's; the core would gain a type for a caller it does not have. **Why
not** handing the clamp to the diagram layer: the window is the caller's (§7), and the bound depends
on the window's size, which the model does not know. **Why not** the name `Extent`: the extent
_Shapes_ withdrew was a shape's own report, and reusing its word would read as though it had come
back.

**D4 — what the view is bounded by.**

**Answer:** the content is the smallest rectangle holding the drawing's written cells and the first
screen — the window at `(0, 0)`, which is where the view opens. On each axis the origin lies in
`[min(0, first), max(0, last − window + 1)]`, where `first` and `last` are the drawing's first and
last written coordinates and `window` is the interior's size; every frame puts it at the nearest
value in that range. **Why not** the drawing alone: a drawing away from `(0, 0)` would open at its
first cell rather than at `(0, 0)`, and the plane's negative positions would be unreachable. **Why
not** stopping when the drawing's far edge reaches the window's near edge (`last`): it allows a view
with nothing in it, and the picture at `(11, 4)` is a corner the drawing does not write. **Why not**
clamping only when a scroll moves the origin: a drawing away from `(0, 0)` would open on an empty
screen and jump on the first scroll, and a resize would do the same. **What it costs:** a drawing
that fits the first screen but starts before `(0, 0)` opens with part of it off-screen, and the
person scrolls left to bring it in. **Answered by** the maintainer, with the three-line example.

**D5 — what moves the view.**

**Answer:** the wheel and the bar, both the rat helper's: a wheel turn over the region moves the
view by a tenth of the window (at least one cell) on the axis the wheel reports, and `Alt` with the
wheel moves it across; a press on the bar takes the view to that position and a drag follows the
pointer. **Why not** the keyboard here: which keys, how far a press moves, and whether a held key
walks are a rule of its own, and [#237](https://github.com/andresmoschini/monospace/issues/237)
decides it; this slice has no key for the view. **Why not** one cell per wheel turn: the helper's
step is the page's tenth, a gesture's natural step, and overriding it would be a second rule about
how far a turn moves. **Why not** a bar that only marks: the helper answers a press and a drag with
a position, and a bar that looked live and was not would be worse than no bar. **What it costs:**
the wheel moves a tenth of the window and the bar jumps, so a person who wants to move exactly one
cell has no input for it until #237. **Answered by** the maintainer.

**D6 — where the bar is drawn, and when it shows.**

**Answer:** on the region's border: the helper lays the vertical bar on the right border column and
the horizontal one on the bottom border row, over the border's glyphs, taking no cell from the
window. Each axis shows its bar only when the content does not fit it (`ScrollbarPolicy::Collapse`),
so content that fits leaves the border as it was. **Why not** inside the interior: it would cost a
column or a row of the window, and the window's size would change when the bar appears, moving the
bound with it. **Why not** always visible: the demonstration's drawing fits, and its frames would
change for a bar that says nothing. **Why not** the helper's default `Minimize`: it keeps the bar's
cells when there is nothing to scroll, which is the same cost with less to read. **What it costs:**
when the content does not fit, the border the bar is on stops being a plain line; the corners stay.
**Answered by** the maintainer.

## Model slice

- `docs/diagram-model.md` §7 gains the measurement: the diagram answers the rectangle its drawing
  occupies, by drawing into a surface that records the positions, and the window stays the caller's;
  §10 gains the properties rules 1 to 3 state; §11's _How big is a diagram?_ is answered and leaves
  the list.
- `docs/model.md` _Shapes_, in _Pieces_: the paragraph that withdrew a shape's extent is amended.
  The layer that decides where figures go now exists, and it asks the drawing rather than the shape,
  so the withdrawal stands and no shape gains a method. The sentence naming the caller that would
  bring a rectangle back names this one.
- `docs/application-model.md` §3: the state holds the view's origin and the two scroll offsets
  beside the drawing; §4: the wheel and the bar change the view and not the session, and answer
  `ScreenChanged`; §5: the bar is the framework's widget, drawn on the region's border; §7: the
  properties rules 4 to 11 state.
- `README.md`: the paragraph naming what is still missing gains the view moving, and leaves the
  keyboard to #237.
- `docs/editing-model.md`: none. A scroll is not a command, so §5's one-step-per-command is not
  touched and the walk never holds a view.

## Public surface

```rust
// monospace-diagram
pub struct Bounds { /* at: Pos, size: Size */ }

impl Bounds {
    pub fn at(&self) -> Pos;
    pub fn size(&self) -> Size;
}

impl Diagram {
    pub fn bounds(&self) -> Option<Bounds>;
}
```

- `Bounds` is `Copy`, `Clone`, `Debug`, `PartialEq` and `Eq`, and its fields are private: the two
  accessors are the whole of it, and a rectangle is a value rather than a place to write into.
- `Diagram::bounds` takes `&self` and answers nothing when the drawing writes no cell. It is the
  only addition below the application.
- `monospace`: `DiagramState` keeps the origin it already holds and gains a `ScrollState` per axis
  and the two `Scroll` descriptors. `redraw` sets each state's `page_len` and `max_offset` from the
  content, clamps the origin to the range of rule 5, writes the clamped origin into the state as its
  offset from the content's first cell, and draws at the origin. `handle` applies the step or
  position the helper reports to that state and takes the origin back from the offset it leaves.
  **The origin is what survives a frame, so a content change does not carry the view with it**; the
  first frame's origin is `(0, 0)`.
- `diagram_widget::draw` renders the bars with `ScrollArea` after the diagram, over the region's own
  block, so the border keeps its style; `handle` answers the wheel and the bar with a
  `ScrollAreaState` built over the region (`.area(region)`), applies what it reports to that axis's
  `ScrollState` (`scroll_up`, `scroll_down`, `set_offset`), and answers `Action::ScreenChanged` only
  when the apply said the view moved. `Action` gains no variant; #237 owns the key one.
- The app names
  `rat_widget::scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollbarPolicy}` and
  nothing below `monospace-editing`: no `Bounds`, no `Diagram`. `rat_widget` re-exports the helper,
  so no dependency is added.

## Behavior

1. `Diagram::bounds` answers the smallest rectangle holding every position the shapes write, and
   `None` when the drawing writes none.
2. A figure that writes nothing contributes nothing: a box narrower or shorter than two cells, a
   line of length zero, and a connector with an endpoint that does not resolve. A sparse figure
   contributes the smallest rectangle around its cells, the empty corner of a turn included.
3. Asking changes nothing and answers the same rectangle twice. A span wider or taller than a `Size`
   holds saturates at 65 535, which is the plane's own range (`docs/model.md` §2).
4. The view opens at `(0, 0)`, and the content is the drawing union the first screen: a drawing away
   from the origin still draws at `(0, 0)`, and a drawing that writes nothing leaves the content as
   the first screen.
5. On each axis the origin lies in `[min(0, first), max(0, last − window + 1)]`, where `first` and
   `last` are the drawing's first and last written coordinates; every frame puts it at the nearest
   value. **The origin is carried, so a change to the content does not move the view unless the
   content no longer holds it.** A drawing inside the first screen does not scroll.
6. A wheel turn over the region moves the view by a tenth of the window (at least one cell) on the
   axis it reports; `Alt` with the wheel moves it across. A turn the content refuses changes nothing
   and owes no frame.
7. The bar shows on the border, per axis, only when the content does not fit that axis, and takes no
   cell from the window. A press on it takes the view to that position and a drag follows the
   pointer; both owe a frame only when the origin moved.
8. A scroll changes no session and records no step: the walk behind and ahead are what they were,
   and neither an undo nor a redo restores a view. The origin is where rule 5 leaves it.
9. A scroll keeps the selection, and the frame after it inverts the cells the drawing records for
   that identity inside the new window, and no others.
10. A click after a scroll is answered at the offset from the window's corner, as it was before it:
    the origin belongs to the plane and takes no part in the click's arithmetic (rule 1 of #177). A
    press on the bar is the bar's, not the diagram's.
11. The demonstration's frames are the ones they were, because its drawing fits the interior and the
    bar is hidden. The application names no `Bounds` and no `Diagram`, and depends on no crate
    beyond the six.

## Examples

**One drawing, and the windows that argue D4.** The description is twelve columns by five: a light
box at `(0, 0)`, a double box at `(10, 1)` and a line at `(2, 4)`. Every picture here is
**generated**: a marker draws a window at an origin, and a frame at a scroll origin is the same
drawing seen through the same window, so `cargo xtask render` re-draws all of them. What has not
been observed is the scrolling itself: that is what the tests in _What proves it_ pin.

<!-- render: 12x5
{ "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 10, "y": 1 }, "size": { "width": 2, "height": 2 },
      "stroke": "double" },
    { "kind": "line", "id": 3, "at": { "x": 2, "y": 4 }, "len": 7, "orientation": "horizontal",
      "stroke": "light" }
  ] }
-->

```text
┌──┐
│  │      ╔╗
└──┘      ╚╝

  ───────
```

<!-- /render -->

The whole drawing. Its rectangle is `12x5` at `(0, 0)`, which is what D1 has to answer from the
cells these three figures write rather than from the positions they name.

**The initial view, and the near end of the bound.** A window of `5x3`, which is the interior of a
small region: the drawing is larger on both axes, so the origin lies in `[0, 7]` across and `[0, 2]`
down.

<!-- render: 5x3
{ "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 10, "y": 1 }, "size": { "width": 2, "height": 2 },
      "stroke": "double" },
    { "kind": "line", "id": 3, "at": { "x": 2, "y": 4 }, "len": 7, "orientation": "horizontal",
      "stroke": "light" }
  ] }
-->

```text
┌──┐
│  │
└──┘
```

<!-- /render -->

**The far end of the bound.** The bar dragged to its corner lands here, at `7,2` — the last origin
D4 allows, because the window's right edge is the drawing's and its bottom edge is the drawing's. A
step past it changes nothing and owes no frame (rule 6).

<!-- render: 5x3 at 7,2
{ "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 10, "y": 1 }, "size": { "width": 2, "height": 2 },
      "stroke": "double" },
    { "kind": "line", "id": 3, "at": { "x": 2, "y": 4 }, "len": 7, "orientation": "horizontal",
      "stroke": "light" }
  ] }
-->

```text
   ╚╝

──
```

<!-- /render -->

**The corner the looser bound allows.** At `11,4` the window's top-left corner is the drawing's
bottom-right cell — which no figure wrote, because the double box ends at `(11, 2)` and the line at
`(8, 4)`. This is the picture that argues against stopping when the far edge reaches the near edge:
a bound built from the rectangle's corners admits a view of nothing, and a bound built from the
cells does not (D2).

<!-- render: 5x3 at 11,4
{ "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 10, "y": 1 }, "size": { "width": 2, "height": 2 },
      "stroke": "double" },
    { "kind": "line", "id": 3, "at": { "x": 2, "y": 4 }, "len": 7, "orientation": "horizontal",
      "stroke": "light" }
  ] }
-->

```text



```

<!-- /render -->

**The first screen is the other half of the content.** Three lines of three cells, one at `x = -4`,
one at `x = 0` and one at `x = 7`, with a window of `6x2`. The content is the drawing union the
first screen, so it runs from `-4` to `9` across, and the origin lies in `[-4, 4]`; the view opens
at `(0, 0)`, which is the middle of that range because the content extends to the left of the
origin.

<!-- render: 14x2 at -4,0
{ "next_id": 4,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": -4, "y": 1 }, "len": 3, "orientation": "horizontal",
      "stroke": "light" },
    { "kind": "line", "id": 2, "at": { "x": 0, "y": 0 }, "len": 3, "orientation": "horizontal",
      "stroke": "light" },
    { "kind": "line", "id": 3, "at": { "x": 7, "y": 1 }, "len": 3, "orientation": "horizontal",
      "stroke": "light" }
  ] }
-->

```text
    ───
───        ───
```

<!-- /render -->

**At the origin, the middle line.** Nothing negative and nothing past the screen is in the window;
the person scrolls left or right to find it.

<!-- render: 6x2 at 0,0
{ "next_id": 4,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": -4, "y": 1 }, "len": 3, "orientation": "horizontal",
      "stroke": "light" },
    { "kind": "line", "id": 2, "at": { "x": 0, "y": 0 }, "len": 3, "orientation": "horizontal",
      "stroke": "light" },
    { "kind": "line", "id": 3, "at": { "x": 7, "y": 1 }, "len": 3, "orientation": "horizontal",
      "stroke": "light" }
  ] }
-->

```text
───

```

<!-- /render -->

**The left end.** At `-4` the window's left edge is the negative line's first cell; a step further
left admits only empty plane.

<!-- render: 6x2 at -4,0
{ "next_id": 4,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": -4, "y": 1 }, "len": 3, "orientation": "horizontal",
      "stroke": "light" },
    { "kind": "line", "id": 2, "at": { "x": 0, "y": 0 }, "len": 3, "orientation": "horizontal",
      "stroke": "light" },
    { "kind": "line", "id": 3, "at": { "x": 7, "y": 1 }, "len": 3, "orientation": "horizontal",
      "stroke": "light" }
  ] }
-->

```text
    ──
───
```

<!-- /render -->

**The right end.** At `4` the window's right edge is the positive line's last cell.

<!-- render: 6x2 at 4,0
{ "next_id": 4,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": -4, "y": 1 }, "len": 3, "orientation": "horizontal",
      "stroke": "light" },
    { "kind": "line", "id": 2, "at": { "x": 0, "y": 0 }, "len": 3, "orientation": "horizontal",
      "stroke": "light" },
    { "kind": "line", "id": 3, "at": { "x": 7, "y": 1 }, "len": 3, "orientation": "horizontal",
      "stroke": "light" }
  ] }
-->

```text

   ───
```

<!-- /render -->

**Origins, as a table.** The content's range on each axis, written out. It is arithmetic rather than
observation: none of it has been run.

| Drawing                   | Window | Opens at                           | Across  | Down   |
| ------------------------- | ------ | ---------------------------------- | ------- | ------ |
| the `12x5` above          | `5x3`  | `(0, 0)`                           | 0 to 7  | 0 to 2 |
| the three lines above     | `6x2`  | `(0, 0)`                           | −4 to 4 | 0 only |
| a `4x3` box at `(20, 10)` | `20x6` | `(0, 0)`, showing nothing          | 0 to 4  | 0 to 7 |
| a `3x3` box at `(2, 2)`   | `20x6` | `(0, 0)`, with the box at `(2, 2)` | 0 only  | 0 only |
| nothing written           | `5x3`  | `(0, 0)`                           | 0 only  | 0 only |
| a `1x1` box at `(30, 0)`  | `5x3`  | `(0, 0)`                           | 0 only  | 0 only |

The third row is where the first screen matters: the drawing is past the screen, so the view opens
on an empty first screen and scrolls right and down until the box reaches the bottom-right corner.
The fourth is a drawing inside the first screen: it opens where it is and the view does not move.
The last two are D2's and rule 2's: a drawing that writes no cell answers no bound, and a `1x1` box
writes no cell at all.

## What proves it

The names bind the building stage. The measurement is in `monospace-diagram`; the wheel, the bar and
the content are in the application, where a test reaches them through the state and the frame.

| Rule | Test                                                                                                                                                           |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `bounds_hold_every_cell_a_drawing_writes` — over every shape the gallery draws, so the bound is the cells a buffer holds; `an_empty_diagram_answers_no_bounds` |
| 2    | `a_box_too_small_to_draw_contributes_no_bound`, `a_line_of_no_cells_contributes_no_bound`, `a_connector_that_does_not_resolve_contributes_no_bound`            |
| 2    | `a_sparse_figures_bound_holds_the_corner_it_leaves_empty`                                                                                                      |
| 3    | `asking_for_the_bounds_twice_answers_the_same`, `a_span_wider_than_a_size_saturates`                                                                           |
| 4    | `a_frame_opens_at_the_origin`, `a_drawing_inside_the_first_screen_does_not_scroll`                                                                             |
| 5    | `a_frame_draws_at_the_origin_the_content_allows`, `a_frame_at_the_end_of_the_content_owes_no_frame`, `a_content_change_keeps_the_origin`                       |
| 6    | `the_wheel_moves_the_view_by_the_helpers_step`, `a_wheel_at_the_end_moves_nothing_and_owes_no_frame`                                                           |
| 7    | `the_bar_is_on_the_border_and_takes_no_cell_from_the_window`, `the_bar_is_hidden_when_the_content_fits`                                                        |
| 7    | `a_press_on_the_bar_jumps_the_view`, `a_drag_on_the_bar_follows_the_pointer`                                                                                   |
| 8    | `a_scroll_records_no_step`, `neither_an_undo_nor_a_redo_restores_a_view`                                                                                       |
| 9    | `a_scroll_keeps_the_selection`                                                                                                                                 |
| 10   | `a_click_after_a_scroll_selects_the_cell_under_the_pointer`                                                                                                    |
| 11   | `the_demonstrations_frames_are_unchanged` — the snapshots — and _nothing holds the absence of the dependency: a reviewer reads the imports_                    |

The eight pictures in _Examples_ are held by their markers: `cargo xtask render` rewrites them and
the gate fails if one is not what its description draws. The bar is not in any of them, because the
command line draws the diagram and no chrome; the bar's frame is the application's test. #179's
tests are untouched: the plain arrows still move the figure, and no key moves the view.

**Measured in the building stage and reported there.** That the demonstration's frames did not move,
with the count of snapshots that did; the cells the bar takes on the border, and that the interior
kept its size; and the cost of the measurement (rule 1), as the time a frame spends on a second pass
over the shapes against the same frame without it. None of it is claimed here: the pictures are
generated, the scrolling is not yet written.

## Open questions

**The horizontal wheel.** The helper answers `Alt` with the wheel as a scroll across, and does not
answer the terminal's `ScrollLeft` and `ScrollRight` events. Whether those should be answered, and
whether a terminal sends them at all, is a question for the first one seen to do it.

**The view following the selection.** A move can still take the selected figure past the window's
edge (D2 of #179), and nothing brings it back. Whether the view should follow a figure that a
command moves, rather than waiting for the person to scroll, is a relation between an action and the
origin that this change does not have: the wheel and the bar are the only things that move it. What
would settle it: a person who moves a figure out of sight and asks for it back.
