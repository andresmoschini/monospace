---
status: implemented
decided: "#239"
implemented: "#244"
date: 2026-10-27
---

# Scroll, bounded by what the diagram occupies

## Why now

The view is fixed at `(0, 0)`: [#175](https://github.com/andresmoschini/monospace/issues/175) left
the origin there and #177 and #179 have drawn through it since, so a figure moved past the edge is
not drawn and the arrow that put it there is the way back only while the person remembers which one
it was (D2 of #179). A viewport that moves is the other answer to the same cost, and it is what
makes the application usable on a diagram larger than the region.

**How far a drawing reaches is answered, and it landed.**
[#238](https://github.com/andresmoschini/monospace/issues/238) measures the stamps on their way into
the buffer — `FootprintSink` in `monospace-core`, `Footprint` its answer — and
`DiagramState::footprint` already keeps the last frame's answer for the status bar. This spec reads
that answer and does not restate it: what a footprint is, and that a stamp the window discarded
counts, are §2 of [`model.md`](../docs/model.md) and §7 of
[`diagram-model.md`](../docs/diagram-model.md).

**The measurement named this consumer and left it unnamed.** D3 of #238 says a consumer that chooses
its window _before_ drawing measures through the sink into a window of no size and "owns that second
traversal", and its open question asks whether that should be one call on the diagram. This spec is
that consumer, and D2 answers the question rather than deferring it.

**The view opens at `(0, 0)`, and the first screen is the other half of the bound.** The plane's
coordinates are signed (`Pos` is an `i16`), so a drawing may lie left of or above the origin as well
as past the screen's right and bottom edges. The content is therefore the smallest rectangle holding
the drawing the footprint answers **and the window at `(0, 0)`**: the view opens on the first
screen, goes left to reach what is negative and right to reach what is beyond it, and a drawing
inside the first screen does not scroll at all. The issue's title names the drawing's half, because
that is the half that needed a measurement. The wheel and the scrollbar move the view; the keyboard
is [#237](https://github.com/andresmoschini/monospace/issues/237)'s, because which keys move it and
how far a press moves is a rule this cut does not take.

## Scope

### In

- `monospace-core` and `monospace-diagram`: nothing. `Footprint`, `FootprintSink` and the
  `DiagramState::footprint` reader landed with #238; this spec consumes them and adds no second
  reader of the shapes, no `Bounds`, and no method on a diagram.
- `monospace`: the origin stays what survives a frame, and a `ScrollState` per axis is derived from
  it; `redraw` measures the drawing into a window of no size before choosing the origin (D2), clamps
  the origin to the content (D1), and draws at it; the wheel and the bar move the view (D3); the bar
  is drawn on the region's border and shows only when the content does not fit (D4).
- The five contract snapshots: one of them carries a drawing. The demonstration's footprint is
  `46×13` and its interior `38×6`, so the region's border gains both bars; the other four screens
  say `nothing drawn` and keep a plain border. Every generated picture in a tracked document is
  unchanged, because the command line draws no chrome.

### Out

- **The keyboard.** [#237](https://github.com/andresmoschini/monospace/issues/237) decides which
  keys move the view and how far a press moves; this slice has no key for it, and the plain arrows
  stay what #179 made them.
- **Where the view stands, in the status bar.** The bar keeps saying how far the drawing reaches,
  which #238 landed. Where the origin is belongs to the rest of the row, and
  [#226](https://github.com/andresmoschini/monospace/issues/226) owns it.
- **A cached rectangle, or one kept beside the session between frames.** Nothing is cached, so
  nothing can be stale; D2 says what the measurement costs instead.
- **A `footprint` call on the session or on the diagram.** The sink is the measurement's home, and
  such a call would draw the diagram anyway (D2).
- **Paging, a wheel step of one cell, fitting the whole drawing, and zoom.** Each is a second rule
  about how far an input moves or a change to how a cell is drawn; the wheel moves the page's tenth,
  as the helper suggests.

**This spec runs past the 250-line ceiling, and lands that way.** The format's first answer is to
split the change rather than compress the prose; the maintainer read the length and accepted it, so
the cut stands as one change and no paragraph was shortened to fit. What it costs: a reviewer reads
further than the ceiling asks, and the eight pictures are the part that has to be read twice.

## The decision

**D1 — what the view is bounded by, and what survives a frame.**

**Answer:** the content is the smallest rectangle holding the drawing's footprint and the first
screen, the window at `(0, 0)`, which is where the view opens. On each axis the origin lies in
`[min(0, first), max(0, last − window + 1)]`, where `first` and `last` are the content's first and
last coordinates and `window` is the interior's size; every frame puts it at the nearest value in
that range. **The origin is what survives a frame**, and each axis's scroll offset is derived from
it — `origin − content's first` — rather than kept. **Why not** the drawing's footprint alone: a
drawing away from `(0, 0)` would open at its first cell rather than at `(0, 0)`, and the plane's
negative positions would be unreachable. **Why not** stopping when the drawing's far edge reaches
the window's near edge (`last`): the footprint is two axis maxima and not a corner, so `(11, 4)` of
_Examples_ is a corner no stamp reached, and a bound that stopped there would admit a view of
nothing. **Why not** the offset as what survives: a content change that moves the drawing's first
cell would carry the view with it, and a person's place is a place in the plane rather than a
distance from a figure. **Why not** clamping only when a scroll moves the origin: a drawing away
from `(0, 0)` would open on an empty screen and jump on the first scroll, and a resize would do the
same. **What it costs:** a drawing that fits the first screen but starts before `(0, 0)` opens with
part of it off-screen, and the person scrolls left to bring it in. **Answered by** the maintainer,
with the three-line example.

**D2 — how the rectangle is obtained before the window is chosen.**

**Answer:** `redraw` draws the session's diagram once more before it draws the frame: into a
`Buffer` whose window is empty, wrapped in `FootprintSink`, and reads the answer. It clamps the
origin with it, then draws again into the real window — wrapped in a sink too, so the status bar
keeps reading the footprint of the drawing that produced the screen (rule 6 of #238). **Why not**
the last frame's footprint: it is one frame stale, and the frame after a command may be the only
one, so the view would never reach the clamp that command needed. **Why not** a `footprint` method
on the session or on the diagram: it would draw the diagram to answer, exactly as this does, and it
would put the view's convenience on a surface below the application, which names no `Diagram`. **Why
not** a rectangle kept beside the session: the state has no cheap way to tell that the session
changed, and a second copy of a fact the drawing answers is a second thing to keep true. **What it
costs:** the diagram is traversed twice per frame, and the first traversal writes nothing. **This is
the call #238's open question left unnamed**, answered here as the practice it points at. **Answered
by** the maintainer.

**D3 — what moves the view.**

**Answer:** the wheel and the bar, both the rat helper's: a wheel turn over the region moves the
view by the helper's step — a tenth of the window, at least one cell — on the axis the wheel
reports, and `Alt` with the wheel moves it across; a press on the bar takes the view to the position
it names and a drag follows the pointer. A terminal's `ScrollLeft` and `ScrollRight` are not
answered, which is the helper's own reading of the wheel and is left as it is. **Why not** the
keyboard here: which keys, how far a press moves, and whether a held key walks are a rule of its
own, and [#237](https://github.com/andresmoschini/monospace/issues/237) decides it; this slice has
no key for the view. **Why not** one cell per wheel turn: the helper's step is the page's tenth, a
gesture's natural step, and overriding it would be a second rule about how far a turn moves. **Why
not** a bar that only marks: the helper answers a press and a drag with a position, and a bar that
looked live and was not would be worse than no bar. **Why not** answering the horizontal wheel's own
events: it would be a second reading of one gesture, and the helper's `Alt` is the one the bar and
the wheel already share. **What it costs:** the wheel moves a tenth of the window and the bar jumps,
so a person who wants to move exactly one cell has no input for it until #237. **Answered by** the
maintainer.

**D4 — where the bar is drawn, and when it shows.**

**Answer:** on the region's border: the helper lays the vertical bar on the right border column and
the horizontal one on the bottom border row, over the border's glyphs and between the block's four
corners, taking no cell from the window. Each axis shows its bar only when the content does not fit
it (`ScrollbarPolicy::Collapse`), so content that fits leaves the border as it was. **Why not**
inside the interior: it would cost a column or a row of the window, and the window's size would
change when the bar appears, moving the bound with it. **Why not** always visible: a drawing inside
the first screen — the empty screen at rest among them — would gain two bars that say nothing. **Why
not** the helper's default `Minimize`: it keeps the bar's cells when there is nothing to scroll,
which is the same cost with less to read. **Why not** a style of its own: the default is what the
helper's other scrollable widgets show, and nothing has yet been seen to mistake the track for a
border. **What it costs:** a drawing larger than the interior stops having a plain border. The
corners and the style stay the block's, and the bar is the helper's default — a `║` track with `▲`
and `▼` at the ends and a solid thumb, `═`, `◄`, `►` across — so a scrolled drawing's border reads
as a border drawn twice. **Answered by** the maintainer.

## Model slice

- `docs/application-model.md` §2 _The frame_: the region's interior is the window at the scroll
  origin, and the bar is drawn on the region's border over the block's glyphs. §3 _The state_: the
  origin is kept beside the drawing, and each axis's scroll state is derived from it and the
  content. §4 _The action model_: the wheel and the bar change the view and not the session, and
  mint `ScreenChanged` only when the origin moved. §5 _What the framework owns_: the bar is the
  framework's scrollbar, drawn by the helper. §7 _Properties worth testing_: the rules this slice
  states. §8 _Open questions_: what else the status bar says keeps
  [#226](https://github.com/andresmoschini/monospace/issues/226) and gains nothing here.
- `README.md`: the paragraph naming what is still missing gains the view moving, and leaves the
  keyboard to #237.
- `docs/diagram-model.md` §7 _Drawing_ and §11: none. The window is already the caller's and the
  measurement landed with #238; a scroll chooses a window and draws nothing new.
- `docs/model.md`: none. `Footprint` is what it was.
- `docs/editing-model.md`: none. A scroll is not a command, so §5's one-step-per-command is not
  touched and the walk never holds a view.

## Public surface

The measurement's surface is #238's and is already public:
`monospace_core::{Footprint, FootprintSink}`, and the application's private
`DiagramState::footprint`. This spec adds nothing there. What it adds is the application's, and all
of it is private:

- `DiagramState` keeps `origin: Pos` and gains `vscroll: ScrollState` and `hscroll: ScrollState`.
  `redraw(region, interior)` measures through `FootprintSink` into a `Buffer` over an empty
  `Window`, clamps the origin to D1's range, sets each scroll state's `page_len` to the window and
  its `max_offset` to the content's span less the window, writes the origin's offset from the
  content's first cell, and draws into the window at the origin through a second `FootprintSink`.
  **The origin is what survives a frame, so a content change does not carry the view with it**; the
  first frame's origin is `(0, 0)`. The range is computed in `i32`, because `last − window + 1`
  leaves an `i16` at the plane's own corner.
- `diagram_widget::draw` renders the bars after the block and the diagram, with
  `ScrollArea::new().v_scroll(Some(&v)).h_scroll(Some(&h))`, where `v` and `h` are
  `Scroll::vertical()` and `Scroll::horizontal()` carrying `.policy(ScrollbarPolicy::Collapse)` and
  a margin of one cell at each end.
- `diagram_widget::handle` asks the bars first, with a `ScrollAreaState` over the region
  (`.area(region)`), and applies what they report to that axis's `ScrollState` (`scroll_up`,
  `scroll_down`, `set_offset`), answering `Action::ScreenChanged` only when the origin moved. The
  event is the helper's `HandleEvent<Event, MouseOnly, ScrollOutcome>`, so a wheel, a press and a
  drag are one call. `Action` gains no variant; #237 owns the key one.
- The app names `rat_widget::event::{HandleEvent, MouseOnly, ScrollOutcome}` and
  `rat_widget::scrolled::{Scroll, ScrollArea, ScrollAreaState, ScrollState, ScrollbarPolicy}`. Both
  are re-exports of crates `rat-widget` already depends on, so no dependency is added, and the app
  names no measurement type of its own and no `Diagram`.

## Behavior

1. The view opens at `(0, 0)`, and the content is the footprint union the first screen: a drawing
   away from the origin still draws at `(0, 0)`, and a drawing that stamped nothing leaves the
   content as the first screen, which does not scroll.
2. On each axis the origin lies in `[min(0, first), max(0, last − window + 1)]`, where `first` and
   `last` are the content's first and last coordinates; every frame puts it at the nearest value.
   **The origin is carried, so a change to the content does not move the view unless the content no
   longer holds it.** A drawing inside the first screen does not scroll.
3. A wheel turn over the region moves the view by a tenth of the window (at least one cell) on the
   axis it reports; `Alt` with the wheel moves it across. A turn the content refuses changes nothing
   and owes no frame.
4. The bar shows on the border, per axis, only when the content does not fit that axis, between the
   block's corners and taking no cell from the window. A press on it takes the view to that position
   and a drag follows the pointer; both owe a frame only when the origin moved.
5. A scroll changes no session and records no step: the walk behind and ahead are what they were,
   and neither an undo nor a redo restores a view. The origin is where rule 2 leaves it.
6. A scroll keeps the selection, and the frame after it inverts the cells the drawing records for
   that identity inside the new window, and no others.
7. A click after a scroll is answered at the offset from the window's corner, as it was before it:
   the origin belongs to the plane and takes no part in the click's arithmetic (rule 1 of #177). A
   press on the bar is the bar's and not the diagram's, and it still gives the region the keyboard,
   because a press on the border is the region's (rule 2 of #189).
8. The demonstration's cells inside the region are the ones they were — its drawing is larger than
   the interior on both axes and was already clipped — and its border gains a bar on each axis, so
   the one snapshot that carries it moves on the border and nowhere else. The application names no
   `Diagram` and depends on no crate beyond the six.

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

The whole drawing. Its footprint is `min (0,0)`, `max (11,4)` — the stamps these three figures
reached, rather than the cells that survived a window — so its rectangle is `12×5` at `(0, 0)`.

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
top-left corner is the footprint's corner — the double box's last column and the line's last row —
and no figure wrote that cell, because the double box ends at `(11, 2)` and the line at `(8, 4)`.
This is the picture that argues against stopping when the far edge reaches the near edge: such a
bound admits a view of nothing, and the range D1 states does not reach it. The footprint is two axis
maxima rather than a corner, which is what makes the corner empty rather than drawn.

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

**Origins, as a table.** The content's range on each axis, written out. It is arithmetic on the
answer #238 landed rather than observation: none of it has been run.

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
The last two are #238's answer: a drawing that stamps nothing has no footprint, and a `1x1` box
draws nothing, so both leave the content as the first screen.

## What proves it

The names bind the building stage. The wheel, the bar and the content are in the application, where
a test reaches them through the state and the frame; the measurement's own tests are #238's.

| Rule | Test                                                                                                                                                                                                                |
| ---- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `a_frame_opens_at_the_origin`, `a_drawing_inside_the_first_screen_does_not_scroll`                                                                                                                                  |
| 2    | `a_frame_draws_at_the_origin_the_content_allows`, `a_frame_at_the_end_of_the_content_owes_no_frame`, `a_content_change_keeps_the_origin`, `a_drag_past_the_track_is_clamped_to_the_content`                         |
| 3    | `the_wheel_moves_the_view_by_the_helpers_step`, `a_wheel_at_the_end_moves_nothing_and_owes_no_frame`                                                                                                                |
| 4    | `the_bar_is_on_the_border_and_takes_no_cell_from_the_window`, `the_bar_is_hidden_when_the_content_fits`, `the_corners_are_the_blocks`, `a_press_on_the_bar_jumps_the_view`, `a_drag_on_the_bar_follows_the_pointer` |
| 5    | `a_scroll_records_no_step`, `neither_an_undo_nor_a_redo_restores_a_view`                                                                                                                                            |
| 6    | `a_scroll_keeps_the_selection`                                                                                                                                                                                      |
| 7    | `a_click_after_a_scroll_selects_the_cell_under_the_pointer`, `a_press_on_the_bar_gives_the_region_the_keyboard`                                                                                                     |
| 8    | `the_screen_with_the_demo` — the snapshot — and `the_demonstrations_picture_keeps_its_cells`                                                                                                                        |

The eight pictures in _Examples_ are held by their markers: `cargo xtask render` rewrites them and
the gate fails if one is not what its description draws. The bar is not in any of them, because the
command line draws the diagram and no chrome; the bar's frame is the application's test. #179's
tests are untouched: the plain arrows still move the figure, and no key moves the view.

**Measured in the building stage and reported there.** Which snapshot moved and on which cells; that
the demonstration's cells inside the region did not; and that the interior kept its size when the
bar appeared. None of it is claimed here: the pictures are generated, the scrolling is not yet
written. The measurement's own cost was #238's to report.

## Open questions

**The view following the selection.** A move can still take the selected figure past the window's
edge (D2 of #179), and nothing brings it back. Whether the view should follow a figure that a
command moves, rather than waiting for the person to scroll, is a relation between an action and the
origin that this change does not have: the wheel and the bar are the only things that move it.
**Left open deliberately:** nothing here depends on it, and what would settle it is a person who
moves a figure out of sight and asks for it back.
