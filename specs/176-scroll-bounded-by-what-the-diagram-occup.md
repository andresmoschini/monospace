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

**Nothing measures a diagram, and [#238](https://github.com/andresmoschini/monospace/issues/238)
answers that before this spec can.** `Diagram` answers `new`, `numbered_from`, `add`, `add_under`,
`forward`, `backward`, `get`, `remove`, `replace` and `draw`, and none of them says how far the
drawing reaches, so a viewport with nothing to base it on scrolls into blank space and never stops.
§7 of [`diagram-model.md`](../docs/diagram-model.md) holds that the diagram measures nothing and
that the window is the caller's, and §11 asks _How big is a diagram?_ with its own answer already
written: what would settle it is a consumer that has to choose a window with nothing to base it on.
[#238](https://github.com/andresmoschini/monospace/issues/238) is that consumer — it shows the
diagram's origin and size in the status bar and settles how the size is calculated. This spec reads
that answer and does not restate it.

**The model also says what a bound means for a sparse figure, and leaves that to #238.** _Shapes_ in
[`model.md`](../docs/model.md) withdrew a shape's own extent and named the caller that would bring a
rectangle back — one that needs to know what another shape occupies, "routing around an obstacle, or
sizing a canvas to its content" — and says that layer "is also the first layer to have to decide
what a bound means for a figure whose written positions are sparse". The work in #238 is that
decision, and the withdrawal stands: no shape gains a method, and this spec adds no reader of the
shapes.

**The view opens at `(0, 0)`, and the first screen is the other half of the bound.** The plane's
coordinates are signed (`Pos` is an `i16`), so a drawing may lie left of or above the origin as well
as past the screen's right and bottom edges. The content is therefore the smallest rectangle holding
the drawing **as #238 measures it and the window at `(0, 0)`**: the view opens on the first screen,
goes left to reach what is negative and right to reach what is beyond it, and a drawing inside the
first screen does not scroll at all. The issue's title names the drawing's half, because that is the
half that needed a measurement. The wheel and the scrollbar move the view; the keyboard is
[#237](https://github.com/andresmoschini/monospace/issues/237)'s, because which keys move it and how
far a press moves is a rule this cut does not take. **This spec is unfinished until #238 lands.**

## Scope

### In

- `monospace-diagram`: nothing. The measurement, its type and its name are
  [#238](https://github.com/andresmoschini/monospace/issues/238)'s; this spec consumes the rectangle
  it lands and adds no second reader of the shapes.
- `monospace`: the origin becomes the content's first cell plus the offset of a `ScrollState` per
  axis, bounded by the rectangle #238 answers and the first screen, and the wheel and the bar move
  it. The content is the drawing union the first screen. The bar is the rat helper's
  (`rat_widget::scrolled`), drawn on the region's border, per axis, only when the content does not
  fit (`Collapse`).
- The demonstration, its snapshots and every picture already in a tracked document: unchanged,
  because the demonstration's drawing fits its interior and the bar is hidden.

### Out

- **The keyboard.** [#237](https://github.com/andresmoschini/monospace/issues/237) decides which
  keys move the view and how far a press moves; this slice has no key for it, and the plain arrows
  stay what #179 made them.
- **The measurement, and the number in the status bar.** How the diagram's origin and size are
  calculated, and showing them in the status line, are
  [#238](https://github.com/andresmoschini/monospace/issues/238)'s. This spec adds no `Bounds`, no
  `Diagram::bounds`, and no second reader of the shapes; where the view stands in the status line
  remains [#226](https://github.com/andresmoschini/monospace/issues/226)'s.
- **A measurement beside #238's.** One rectangle, answered once. This spec does not read the shapes
  itself and does not name the type or the crate that answers.
- **Keeping a rectangle beside the session between frames.** Nothing is cached, so nothing can be
  stale; the rectangle is asked again on each frame.
- **Paging, a wheel step of one cell, fitting the whole drawing, and zoom.** Each is a second rule
  about how far an input moves or a change to how a cell is drawn; the wheel moves the page's tenth,
  as the helper suggests.

## The decision

**The measurement is [#238](https://github.com/andresmoschini/monospace/issues/238)'s and is not
decided here.** It shows the drawing's origin and size in the status bar and lands the accessor this
spec consumes; how it calculates them is its own decision. The three decisions below are the view's,
and every one of them reads the rectangle #238 answers. **This spec is finished when #238 is.**

**D1 — what the view is bounded by.**

**Answer:** the content is the smallest rectangle holding the drawing's bound — the rectangle #238
answers — and the first screen, the window at `(0, 0)`, which is where the view opens. On each axis
the origin lies in `[min(0, first), max(0, last − window + 1)]`, where `first` and `last` are the
content's first and last coordinates and `window` is the interior's size; every frame puts it at the
nearest value in that range. **Why not** the drawing's bound alone: a drawing away from `(0, 0)`
would open at its first cell rather than at `(0, 0)`, and the plane's negative positions would be
unreachable. **Why not** stopping when the drawing's far edge reaches the window's near edge
(`last`): it allows a view with nothing in it, and the picture at `(11, 4)` is a corner the drawing
does not write. **Why not** clamping only when a scroll moves the origin: a drawing away from
`(0, 0)` would open on an empty screen and jump on the first scroll, and a resize would do the same.
**What it costs:** a drawing that fits the first screen but starts before `(0, 0)` opens with part
of it off-screen, and the person scrolls left to bring it in. **Answered by** the maintainer, with
the three-line example.

**D2 — what moves the view.**

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

**D3 — where the bar is drawn, and when it shows.**

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

- `docs/diagram-model.md` and `docs/model.md` _Shapes_: none here.
  [#238](https://github.com/andresmoschini/monospace/issues/238) amends §7, §11 and the _Shapes_
  paragraph for the measurement; this spec adds no rectangle of its own.
- `docs/application-model.md` §3: the state holds the view's origin and the two scroll offsets
  beside the drawing; §4: the wheel and the bar change the view and not the session, and answer
  `ScreenChanged`; §5: the bar is the framework's widget, drawn on the region's border; §7: the
  properties rules 1 to 8 state.
- `README.md`: the paragraph naming what is still missing gains the view moving, and leaves the
  keyboard to #237.
- `docs/editing-model.md`: none. A scroll is not a command, so §5's one-step-per-command is not
  touched and the walk never holds a view.

## Public surface

The measurement's public surface is [#238](https://github.com/andresmoschini/monospace/issues/238)'s
and is not named here. What this spec adds is the application's:

- `monospace`: `DiagramState` keeps the origin it already holds and gains a `ScrollState` per axis
  and the two `Scroll` descriptors. `redraw` asks the session for the diagram's rectangle, sets each
  state's `page_len` and `max_offset` from the content, clamps the origin to the range of rule 2,
  writes the clamped origin into the state as its offset from the content's first cell, and draws at
  the origin. `handle` applies the step or position the helper reports to that state and takes the
  origin back from the offset it leaves. **The origin is what survives a frame, so a content change
  does not carry the view with it**; the first frame's origin is `(0, 0)`.
- `diagram_widget::draw` renders the bars with `ScrollArea` after the diagram, over the region's own
  block, so the border keeps its style; `handle` answers the wheel and the bar with a
  `ScrollAreaState` built over the region (`.area(region)`), applies what it reports to that axis's
  `ScrollState` (`scroll_up`, `scroll_down`, `set_offset`), and answers `Action::ScreenChanged` only
  when the apply said the view moved. `Action` gains no variant; #237 owns the key one.
- The app names
  `rat_widget::scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollbarPolicy}` and
  nothing below `monospace-editing`: no measurement type, no `Diagram`. `rat_widget` re-exports the
  helper, so no dependency is added.

## Behavior

The measurement's rules are [#238](https://github.com/andresmoschini/monospace/issues/238)'s; these
are the view's.

1. The view opens at `(0, 0)`, and the content is the drawing union the first screen: a drawing away
   from the origin still draws at `(0, 0)`, and a drawing that writes nothing leaves the content as
   the first screen.
2. On each axis the origin lies in `[min(0, first), max(0, last − window + 1)]`, where `first` and
   `last` are the content's first and last coordinates; every frame puts it at the nearest value.
   **The origin is carried, so a change to the content does not move the view unless the content no
   longer holds it.** A drawing inside the first screen does not scroll.
3. A wheel turn over the region moves the view by a tenth of the window (at least one cell) on the
   axis it reports; `Alt` with the wheel moves it across. A turn the content refuses changes nothing
   and owes no frame.
4. The bar shows on the border, per axis, only when the content does not fit that axis, and takes no
   cell from the window. A press on it takes the view to that position and a drag follows the
   pointer; both owe a frame only when the origin moved.
5. A scroll changes no session and records no step: the walk behind and ahead are what they were,
   and neither an undo nor a redo restores a view. The origin is where rule 2 leaves it.
6. A scroll keeps the selection, and the frame after it inverts the cells the drawing records for
   that identity inside the new window, and no others.
7. A click after a scroll is answered at the offset from the window's corner, as it was before it:
   the origin belongs to the plane and takes no part in the click's arithmetic (rule 1 of #177). A
   press on the bar is the bar's, not the diagram's.
8. The demonstration's frames are the ones they were, because its drawing fits the interior and the
   bar is hidden. The application names no measurement type and no `Diagram`, and depends on no
   crate beyond the six.

## Examples

**One drawing, and the windows that argue D1.** The description is twelve columns by five: a light
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

The whole drawing. Its rectangle is `12x5` at `(0, 0)`, which is what #238's measurement answers
from the cells these three figures write rather than from the positions they name.

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
D1 allows, because the window's right edge is the drawing's and its bottom edge is the drawing's. A
step past it changes nothing and owes no frame (rule 3).

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

**The corner a bound built from the rectangle's corners would allow.** At `11,4` the window's
top-left corner is the drawing's bottom-right cell — which no figure wrote, because the double box
ends at `(11, 2)` and the line at `(8, 4)`. This is the picture that argues against stopping when
the far edge reaches the near edge: such a bound admits a view of nothing, and the range this spec's
D1 states does not reach it. Whether the drawing's rectangle is taken from written cells or from
named corners is [#238](https://github.com/andresmoschini/monospace/issues/238)'s to settle; the
picture is here for the argument, not as a frame this spec allows.

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
The last two are [#238](https://github.com/andresmoschini/monospace/issues/238)'s: a drawing that
writes no cell answers no bound, and a `1x1` box writes no cell at all.

## What proves it

The names bind the building stage. The wheel, the bar and the content are in the application, where
a test reaches them through the state and the frame; the measurement's own tests are
[#238](https://github.com/andresmoschini/monospace/issues/238)'s.

| Rule | Test                                                                                                                                        |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `a_frame_opens_at_the_origin`, `a_drawing_inside_the_first_screen_does_not_scroll`                                                          |
| 2    | `a_frame_draws_at_the_origin_the_content_allows`, `a_frame_at_the_end_of_the_content_owes_no_frame`, `a_content_change_keeps_the_origin`    |
| 3    | `the_wheel_moves_the_view_by_the_helpers_step`, `a_wheel_at_the_end_moves_nothing_and_owes_no_frame`                                        |
| 4    | `the_bar_is_on_the_border_and_takes_no_cell_from_the_window`, `the_bar_is_hidden_when_the_content_fits`                                     |
| 4    | `a_press_on_the_bar_jumps_the_view`, `a_drag_on_the_bar_follows_the_pointer`                                                                |
| 5    | `a_scroll_records_no_step`, `neither_an_undo_nor_a_redo_restores_a_view`                                                                    |
| 6    | `a_scroll_keeps_the_selection`                                                                                                              |
| 7    | `a_click_after_a_scroll_selects_the_cell_under_the_pointer`                                                                                 |
| 8    | `the_demonstrations_frames_are_unchanged` — the snapshots — and _nothing holds the absence of the dependency: a reviewer reads the imports_ |

The eight pictures in _Examples_ are held by their markers: `cargo xtask render` rewrites them and
the gate fails if one is not what its description draws. The bar is not in any of them, because the
command line draws the diagram and no chrome; the bar's frame is the application's test. #179's
tests are untouched: the plain arrows still move the figure, and no key moves the view.

**Measured in the building stage and reported there.** That the demonstration's frames did not move,
with the count of snapshots that did; and the cells the bar takes on the border, and that the
interior kept its size. None of it is claimed here: the pictures are generated, the scrolling is not
yet written. The measurement's own cost is
[#238](https://github.com/andresmoschini/monospace/issues/238)'s to report.

## Open questions

**The horizontal wheel.** The helper answers `Alt` with the wheel as a scroll across, and does not
answer the terminal's `ScrollLeft` and `ScrollRight` events. Whether those should be answered, and
whether a terminal sends them at all, is a question for the first one seen to do it.

**The view following the selection.** A move can still take the selected figure past the window's
edge (D2 of #179), and nothing brings it back. Whether the view should follow a figure that a
command moves, rather than waiting for the person to scroll, is a relation between an action and the
origin that this change does not have: the wheel and the bar are the only things that move it. What
would settle it: a person who moves a figure out of sight and asks for it back.
