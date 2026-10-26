---
status: draft
date: 2026-10-26
---

# Scroll, bounded by what the diagram occupies

## Why now

The arrow keys move the selected figure, and #179 says in its D2 what that costs: a figure displaced
past the edge of the region is not drawn, and the arrow that put it there is the way back only while
the person remembers which one it was. A viewport that moves is the other answer to the same cost,
and it is the one [#175](https://github.com/andresmoschini/monospace/issues/175) left at `(0, 0)`
and #177 and #179 have drawn through since. **Because the drawing is redrawn at the new origin
rather than cut out of a fixed grid, a scroll is a change of origin and nothing else** — the
pictures in _Examples_ are drawn that way today.

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

## Scope

### In

- `monospace-diagram`: `Diagram::bounds` and the `Bounds` it answers, measured by drawing the shapes
  into a surface that records the positions and keeps no cell.
- `monospace`: the origin the state already holds at `(0, 0)` becomes the scroll position, four keys
  move it (D5), and every frame bounds it by the drawing (D4). `Action::Scroll` and
  `DiagramState::scroll` are the two new names.
- The demonstration, its snapshots and every picture already in a tracked document: unchanged,
  because the demonstration's drawing starts at `(0, 0)` and the clamp leaves the origin there.

### Out

- **An indicator of where the view is.** #174 lists the scrollbar as #176's and this slice moves the
  view without drawing anything about it. An indicator decides cells the diagram does not own — a
  bar on the region's border, a mark in the interior — and the issue asks for the stop, not for a
  report of the position (see _Open questions_).
- **The plain arrows.** #179, agreed, and this spec reads its rule 1 and leaves it as it is (D5).
- **Paging, the wheel, a scroll of more than one cell, fitting the whole drawing, and zoom.** Each
  is a second rule about how far a press moves or a change to how a cell is drawn; this is one cell
  per press at a scale of one cell.
- **A bound taken from the figures' declared geometry.** D1: the measurement is the drawing.
- **Keeping a rectangle beside the session between frames.** D1's cost and rule 3: nothing is
  cached, so nothing can be stale.
- **What the status bar says about the view.**
  [#226](https://github.com/andresmoschini/monospace/issues/226).
- **Building before #179's building stage merges.** The four plain arrows have to exist before a
  modifier is added beside them, and the scroll's keys are read in the same handler.

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
has (rule 3). **Answered by** practice: it is the only reading under which a shape still answers
nothing.

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

**D4 — where the origin stops.**

**Answer:** _pending_ — recommended: the window stays inside the drawing's rectangle, and the
drawing sits at the window's corner when it is the smaller of the two. In each axis the origin lies
in `[first, max(first, first + span − window)]`, the rectangle being what D1 answers and the window
being the interior's size; every frame clamps it, and a scroll that cannot move changes nothing and
owes no frame. **Why not** the looser bound that stops when the drawing's far edge reaches the
window's near edge: it allows a view with nothing in it, and the picture at `11,4` in _Examples_ is
the corner of a rectangle whose cells that corner does not hold. **Why not** clamping only when a
scroll moves the origin: a description whose figures are away from `(0, 0)` would open on an empty
screen, and the first scroll after it would jump. **What it costs:** a drawing that does not start
at `(0, 0)` is drawn flush to the corner rather than where its author put it, and a deletion that
shrinks a drawing past the origin moves the picture back inside it.

**D5 — which keys move the view.**

**Answer:** _pending_ — recommended: `Ctrl` with the four arrows, answered by the region while it
holds the keyboard, each press carrying `Action::Scroll(dx, dy)` with the arrow's own delta. The
plain arrows stay what #179 made them, and a modified arrow is a combination no rule answers today,
so the scroll takes keys nothing else wants. **Why not** the plain arrows while nothing is selected:
a key whose meaning depends on the selection cannot scroll to a figure that was just moved out of
sight, and the region's handler would read the selection to decide what to mint. **Why not**
`PageUp`/`PageDown` and `Home`/`End`: a press moves by a window or to an end, which is a second rule
about how far a press moves. **Why not** the wheel: it is a gesture and not a key, it is not
reported for every terminal or on both axes, and the region's mouse branch would answer it beside
the click. **What it costs:** #179's rule 8 says a press carrying any modifier moves nothing, and
its test walks every modifier; with the scroll on `Ctrl` that test narrows to the modifiers that are
not the scroll's, which is a decision taken here rather than a test edited to fit. And a person who
does not know the keys will try the plain arrows, because the menu has no entry to read: #226 is
where the screen could say so.

## Model slice

- `docs/diagram-model.md` §7 gains the measurement: the diagram answers the rectangle its drawing
  occupies, by drawing into a surface that records the positions, and the window stays the caller's;
  a caller that has to choose one now has a question to ask. §10 gains the properties rules 1 to 3
  state. §11's _How big is a diagram?_ is answered and leaves the list.
- `docs/model.md` _Shapes_, in _Pieces_: the paragraph that withdrew a shape's extent is amended.
  "Which positions a figure occupies is a question for the layer that decides where figures go, and
  that layer does not exist yet" stops being true — the layer exists, and it asks the drawing rather
  than the shape, so the withdrawal stands and no shape gains a method. The sentence naming the
  caller that would bring a rectangle back names this one.
- `docs/application-model.md` §3: the origin is bounded by the drawing rather than written nowhere;
  §4: a scroll is an action that changes no session and records no step; §7: the properties rules 4
  to 9 state.
- `README.md`: the paragraph naming what is still missing gains the view moving.
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
- `monospace`: `Action` gains one variant, `Scroll(i16, i16)`, and stays `Copy`, `PartialEq` and
  `Eq` — it carries a delta and no origin, as `Move` carries a delta and no identity.
- `diagram_widget::handle` answers the four `Ctrl` arrows with `Action::Scroll` while the focus it
  is handed is `Focus::Region`; `shortcuts` does not change, because the region's own handler is
  where the arrows already live (D1 of #179).
- `DiagramState` gains `pub(crate) fn scroll(&mut self, dx: i16, dy: i16) -> bool`, which asks the
  session's diagram for its bounds, clamps the origin against the interior of the last frame, and
  answers whether the origin moved. `DiagramState::redraw` clamps the origin before it builds the
  buffer. `callbacks::apply` carries the action and owes the frame only when the answer is `true`.
- No dependency is added, and no crate's public API is widened for the application: the value
  `bounds` answers is used where it is returned, as `draw`'s buffer is.

## Behavior

1. `Diagram::bounds` answers the smallest rectangle holding every position the shapes write, and
   `None` when the drawing writes none.
2. A figure that writes nothing contributes nothing: a box narrower or shorter than two cells, a
   line of length zero, and a connector with an endpoint that does not resolve. A sparse figure
   contributes the smallest rectangle around its cells, the empty corner of a turn included.
3. Asking changes nothing and answers the same rectangle twice. A span wider or taller than a `Size`
   holds saturates at 65 535, which is the plane's own range (`docs/model.md` §2).
4. `Ctrl` plus an arrow with the region holding the keyboard answers `Action::Scroll` with one cell
   along that arrow's axis, and records no step. A press with any other modifier, any other key, a
   release and a repeat answer nothing, and so does every key while the bar holds the keyboard.
5. A scroll the bounds allow moves the origin one cell and owes the frame; a scroll they refuse
   changes nothing and owes none.
6. Every frame draws at the origin the bounds allow: on each axis it lies in
   `[first, max(first, first + span − window)]`, the window being the interior's size. A frame with
   an empty interior draws nothing and does not move the origin.
7. A scroll changes no session and records no step: the walk behind and ahead are what they were,
   and neither an undo nor a redo moves the view.
8. A scroll keeps the selection, and the frame after it inverts the cells the drawing records for
   that identity inside the new window, and no others.
9. A click after a scroll is answered at the offset from the window's corner, as it was before it:
   the origin belongs to the plane and takes no part in the click's arithmetic (rule 1 of #177).
10. The demonstration's frames are the ones they were, because its drawing starts at `(0, 0)` and
    the clamp leaves the origin there. The application names no type below `monospace-editing` — no
    `Bounds`, no `Diagram` — and depends on no crate for either.

## Examples

**One drawing, and the three windows that argue D4.** The description is twelve columns by five: a
light box at `(0, 0)`, a double box at `(10, 1)` and a line at `(2, 4)`. Every picture here is
**generated**: a marker draws a window at an origin, and a frame at a scroll origin is the same
drawing seen through the same window, so `cargo xtask render` re-draws all four. What has not been
observed is the scrolling itself: that is what the tests in _What proves it_ pin.

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

**The near end of the bound.** A window of `5x3`, which is the interior of a small region: with the
drawing larger on both axes, the origin lies in `[0, 7]` across and `[0, 2]` down.

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

**The far end of the bound.** `Ctrl+Right` seven times and `Ctrl+Down` twice from the near end land
here, at `7,2` — the last origin D4 allows, because the window's right edge is the drawing's and its
bottom edge is the drawing's. An eighth `Ctrl+Right` changes nothing and owes no frame (rule 5).

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

**Origins, as a table.** The same drawing and the same window, with the arithmetic of rule 6 written
out. It is arithmetic rather than observation: none of it has been run.

| Drawing                              | Window | Allowed origins, across | down       |
| ------------------------------------ | ------ | ----------------------- | ---------- |
| the `12x5` above                     | `5x3`  | `0` to `7`              | `0` to `2` |
| the same, with nothing written       | `5x3`  | none: the origin stays  | none       |
| a box at `(20, 10)`, `4x3`           | `20x6` | `20` only               | `10` only  |
| a `1x1` box at `(30, 0)` and nothing | `5x3`  | none: the origin stays  | none       |

The third row is the one that decides where a drawing that starts away from `(0, 0)` is drawn: it
sits at the window's corner, because the window is the larger of the two. The second and the fourth
are D2's and rule 2's: a drawing that writes no cell answers no bound, so there is nothing to
scroll, and a `1x1` box writes no cell at all.

## What proves it

The names bind the building stage. The measurement is in `monospace-diagram`; the keys, the origin
and the clamp are in the application, where a test reaches them through the state.

| Rule | Test                                                                                                                                                           |
| ---- | -------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1    | `bounds_hold_every_cell_a_drawing_writes` — over every shape the gallery draws, so the bound is the cells a buffer holds; `an_empty_diagram_answers_no_bounds` |
| 2    | `a_box_too_small_to_draw_contributes_no_bound`, `a_line_of_no_cells_contributes_no_bound`, `a_connector_that_does_not_resolve_contributes_no_bound`            |
| 2    | `a_sparse_figures_bound_holds_the_corner_it_leaves_empty`                                                                                                      |
| 3    | `asking_for_the_bounds_twice_answers_the_same`, `a_span_wider_than_a_size_saturates`                                                                           |
| 4    | `a_ctrl_arrow_scrolls_one_cell_on_its_axis`, `a_scroll_is_a_press_and_no_other_combination`                                                                    |
| 4    | `the_arrows_are_not_the_regions_while_the_bar_holds_the_keyboard` — already, from #179                                                                         |
| 5    | `a_scroll_at_the_end_of_the_bound_moves_nothing_and_owes_no_frame`, `a_scroll_one_cell_in_owes_the_frame`                                                      |
| 6    | `a_frame_draws_at_the_origin_the_bounds_allow`, `a_drawing_smaller_than_the_window_sits_at_its_corner`                                                         |
| 6    | `a_frame_with_no_interior_moves_nothing`                                                                                                                       |
| 7    | `a_scroll_records_no_step`, `neither_an_undo_nor_a_redo_moves_the_view`                                                                                        |
| 8    | `a_scroll_keeps_the_selection`                                                                                                                                 |
| 9    | `a_click_after_a_scroll_selects_the_cell_under_the_pointer`                                                                                                    |
| 10   | `the_demonstrations_frames_are_unchanged` — the snapshots — and _nothing holds the absence of the dependency: a reviewer reads the imports_                    |

The four pictures in _Examples_ are held by their markers: `cargo xtask render` rewrites them and
the gate fails if one is not what its description draws. #179's test
`a_move_is_the_arrow_and_no_modifier` changes with rule 4 — `Ctrl` plus an arrow is the scroll now —
and that is the one existing contract test this change amends (D5).

**Measured in the building stage and reported there.** That the demonstration's frames did not move,
with the count of snapshots that did; and the cost of rule 3, as the time a frame spends on a second
pass over the shapes against the same frame without it. Neither is claimed here: the pictures are
generated, the scrolling is not yet written.

## Open questions

**The scrollbar #174 names.** It lists "the selection highlight for a shape, the scrollbar, and the
keybindings hint" as #176, #177 and later; this slice moves the view and draws no indicator of where
it is. An indicator is a figure of the chrome and not of the diagram — it decides cells the region
does not give the drawing, and it needs the bounds and the window, which this change puts at hand.
It is a change of its own, and the issue is opened when this spec is agreed rather than listed here
as work.

**Holding a key down.** A repeat is not a press, so a held arrow moves one cell and stops; whether a
held key should walk, and whether a walk is one step or many, is `docs/editing-model.md` §8's, left
to the first gesture — and for the view a walk would record no step either way.

**The view following the selection.** A move can still take the selected figure past the window's
edge (D2 of #179), and nothing brings it back. Whether the view should follow a figure that a
command moves, rather than waiting for the person to scroll, is a relation between an action and the
origin that this change does not have: the scroll is the only action that moves the origin. What
would settle it: a person who moves a figure out of sight and asks for it back.
