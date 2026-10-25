---
status: implemented
decided: "#218"
implemented: "#224"
date: 2026-10-24
---

# Draw the demo description, or one given by path, in the space available

## Why now

The frame exists and the region is empty. #174 established the three regions, put a box where the
diagram goes, and gave the application a `Session` it has never asked for a diagram. A person can
open the application and see a frame; the first thing a diagram editor owes them is the diagram.
This slice is the smallest one that makes the region hold something: the application's own
demonstration when no path is given, and the description the path names when one is.

It is also the slice that settles how the diagram is drawn, and the answer is what makes the next
two slices cheap. The diagram is drawn into a buffer the size of the region's interior, at the
scroll origin, and the cells are copied one at a time. The buffer's ownership record is what answers
a click, and drawing at the origin is what lets the click's column and row be an offset into the
window with nothing added. `monospace-rust` kept one grid and cut a viewport out of it, which is why
its hit test was a guess its author doubted in a comment.

## Scope

### In

- The application reads one optional path. With none it reads its own demonstration; with one it
  reads that file. Both go through `monospace_description::parse`, so the format has one reader and
  not a second one written for the screen.
- The diagram is drawn into the region's interior on every frame, into a `monospace_core::Buffer`
  whose origin is the scroll origin and whose size is the interior's.
- The cells are copied from that buffer into the frame's, one at a time, with
  `monospace_glyph_sets::all()`.
- The drawn buffer is kept in the state and replaced on every frame (D4).
- The application's own demonstration, the shared catalog and a path that cannot be read are D2, D3
  and D5.
- The application's dependency list, the README's account of the crate, and the snapshots of the
  screen.

### Out

- **Scrolling, and the bound on it.** #176. This slice draws at `(0, 0)` and does not move.
- **Selecting a shape, the inverted drawing and the status bar's report of it.** #177. The owner
  record is drawn here and asked there.
- **Editing, and undo.** #181. No command is performed on the session.
- **The pop-up, the focus and the colors.** #189.
- **A catalog chosen by the configuration.** This slice draws with the one the command line draws
  with; whether the configuration holds a catalog is not decided here.
- **The status bar's content beyond what #174 gave it.** #181 owns what the bar will say about the
  session.
- **Writing a description, and creating a shape.** #180.

## The decision

**D1 — does the diagram fill the region edge to edge, or draw inside the frame's border?**

**Answer:** inside the frame. The window is the region's interior, one cell in from each side, and a
click's column and row are counted from that interior. **Why not** edge to edge: the frame is #174's
and is what says where the diagram's place is; a diagram that painted over it would erase the one
thing on screen that distinguishes the region from the bar and the status bar, and #174's picture
would have to be withdrawn. **What it costs:** two columns and two rows of a small region, and a
click translation that subtracts the border before it asks the owner. _Examples_ shows the same
description at 12×5 and at the 10×3 the frame leaves.

**D2 — where does the shipped demonstration live, now that two binaries read it?**

**Answer:** the application carries its own `crates/monospace/assets/demo.json`, a copy of the
command line's file at the time of this change; no test compares the two, and they may diverge.
**Why not** a shared file at the workspace root or in `monospace-description`: the command line's
demonstration acts on four offsets of its exact diagram, so a shared file would make the
application's sample a hostage of the demonstration's assumptions, and the application only draws.
**Why not** including the command line's path: one crate's build reading a sibling's tree is a
coupling nothing declares, and moving the command line's assets would break the application in
silence. **What it costs:** two files, and a change to one does not move the other.

**D3 — where does the union of the nine glyph tables live?**

**Answer:** `monospace_glyph_sets::all()`, the union of the core's Light and the eight tables the
crate ships, and both binaries call it. **Why not** copying the nine-line function into the
application: two lists that have to be kept in step, and a table added to the crate would be drawn
by one binary and not the other with nothing to say so. **What it costs:** one item added to a
library's public API, and the command line's private `glyph_catalog()` goes.

**D4 — does the drawn buffer survive the frame?**

**Answer:** the state keeps the buffer the last frame produced, replaced on every frame, and a click
asks it. **Why not** rebuilding it on a click: the owner record belongs to the drawing that produced
the picture, and a click is on the picture the person saw; a redraw at click time answers about a
drawing the person did not click when the diagram has changed since the frame. **What it costs:** a
third thing in the state beside the session and the chrome's own, and `docs/application-model.md` §3
says so.

**D5 — what does the application do with a path it cannot read or parse?**

**Answer:** the application reads and parses before it takes the screen; a failure prints
`monospace: <error>` and exits non-zero, as `monospace-cli` does. **Why not** starting with an empty
diagram and saying so in the status bar: the status bar is not built to carry a failure, and a
person who mistyped a path would be looking at a screen that says nothing about why the diagram is
empty. **Why not** the framework's error handler: it ends the loop and prints the failure with the
terminal back, which is right for a failure inside the loop and pointless for one that can be
reported before the loop starts. **What it costs:** the failure is not a frame, so no snapshot can
show it; the test is the function that reads and parses.

## Model slice

- `docs/application-model.md` §2: the region is the diagram's, the frame stays, and the diagram is
  drawn inside it; the drawing order puts the diagram after the bar and before the status bar, with
  the drop-down after the region.
- `docs/application-model.md` §3: the state gains the drawing of the last frame, beside the session
  and the chrome's own, because a click asks it what the picture said; the glyph catalog is not part
  of the state either, and whether the configuration holds it is not settled here.
- `docs/application-model.md` §4: a handler answers with an action or with nothing, and the first
  answer ends the asking; a change of the chrome's own mints a repaint, and a repaint is the one
  action about the screen rather than about what a person means to do.
- `docs/application-model.md` §5: the drawing is the framework's and the core's; the application
  copies the cells and owns the window.
- `docs/diagram-model.md`: none. §7 already makes the window the caller's, and §8 already says the
  owner is asked with an offset into it.
- `docs/editing-model.md`: none. The session holds the diagram and this slice reads it.
- `README.md`: the `monospace` row and the "Still missing" paragraph, which says the diagram the
  application holds is not here.

## Public surface

The binary's usage is `monospace [path]`. It names `monospace-editing`, `monospace-description`,
`monospace-core` and `monospace-glyph-sets`, and **it names no `monospace-diagram` type**: `parse`
returns the diagram, `Session::new` takes it, and `Session::diagram` hands it back, so the type is
inferred at every step and that crate is not an edge. `monospace-glyph-sets` gains one item:
`pub fn all() -> GlyphCatalog`, the union of the core's Light and the eight tables the crate ships.
Nothing else in any library changes, and no command is performed on the session.

## Behavior

1. The application takes one optional path. With none it reads its own demonstration; with one it
   reads that file. The text is parsed by `monospace_description::parse`, and the diagram it returns
   is the session's starting diagram. **No window is read out of the file**, because the format has
   none (spec 203).
2. The region's interior is the window. On every frame the application creates a
   `monospace_core::Buffer` whose origin is the scroll origin and whose size is the interior's,
   draws the session's diagram into it, and writes every cell of the interior from it. The frame is
   painted around it. The initial scroll origin is `(0, 0)`; #176 changes it.
3. A cell the buffer does not hold renders as a space, and so does a cell whose glyph the catalog
   does not know. The region is therefore painted whole on every frame, which is #174's rule 3.
4. The cells are copied one at a time. **`monospace_core::render` is not used**, because the
   `String` it returns drops the ownership record, and the record is what #177 asks.
5. The buffer's origin is the scroll origin and its size is the interior's, so a click's column and
   row, counted from the interior's corner, are the offset `Buffer::owner` takes: nothing is added
   to them. **The buffer the last frame produced is kept in the state**, replaced on every frame,
   and a click asks it.
6. The diagram is drawn with `monospace_glyph_sets::all()`, the union of the core's Light and the
   eight tables the crate ships.
7. The region is painted after the bar and before the status bar; the drop-down is painted after the
   region, so it covers the diagram. **The drop-down leaves the region byte-identical when it
   closes**, which is #174's rule 4 and now means the diagram's cells too.
8. An empty diagram draws nothing: the interior is blank and the frame stays. A region with no
   interior draws nothing and does not fail.
9. A path that cannot be read or parsed is reported before the screen is taken: `monospace: <error>`
   on stderr and a non-zero exit, as `monospace-cli` does.
10. The application performs no command on the session and holds no domain logic: it reads, parses,
    draws and copies.

## Examples

**The difference D1 makes.** The same description, drawn at 12×5 and at the 10×3 the frame leaves.
The first is the option D1 rejected; the second is what the application draws, inside the frame.
Both are the command line's own rendering; the screen adds the bar above and the status bar below,
and neither is in these pictures.

<!-- render: 12x5
{ "next_id": 3, "shapes": [
  { "kind": "line", "id": 1, "at": { "x": 0, "y": 0 }, "len": 12, "orientation": "horizontal", "stroke": "light" },
  { "kind": "line", "id": 2, "at": { "x": 0, "y": 0 }, "len": 5, "orientation": "vertical", "stroke": "light" }
] }
-->

```text
┌───────────
│
│
│
│
```

<!-- /render -->

<!-- render: 10x3
{ "next_id": 3, "shapes": [
  { "kind": "line", "id": 1, "at": { "x": 0, "y": 0 }, "len": 12, "orientation": "horizontal", "stroke": "light" },
  { "kind": "line", "id": 2, "at": { "x": 0, "y": 0 }, "len": 5, "orientation": "vertical", "stroke": "light" }
] }
-->

```text
┌─────────
│
│
```

<!-- /render -->

**The screen with the demonstration has not been run.** The application does not draw the diagram
yet, so the snapshots `the_screen_with_the_demo`, `the_diagram_inside_the_frame` and
`the_demo_scrolled` (#176) are what the building stage pins, not a record of a run. The
demonstration itself is 50×13, so at 40×10 the interior is 38×6 and holds its first 38×6 cells; the
rest is clipped by the window, exactly as `docs/diagram-model.md` §7 says a stamp outside a window
is.

## What proves it

The tests do not exist yet. The names are what the building stage is held to.

| Rule | Test                                                          |
| ---- | ------------------------------------------------------------- |
| 1    | `the_demonstration_is_drawn_when_no_path_is_given`            |
| 1    | `the_description_at_the_path_is_drawn`                        |
| 2    | `the_interior_is_the_window_the_diagram_is_drawn_into`        |
| 2    | `the_diagram_is_drawn_again_on_every_frame`                   |
| 3    | `a_cell_no_shape_wrote_is_a_space`                            |
| 3    | `a_glyph_the_catalog_does_not_know_is_a_space`                |
| 4    | `the_owner_survives_the_copy`                                 |
| 5    | `the_owner_is_asked_with_the_click_offset_and_nothing_added`  |
| 5    | `the_buffer_of_the_last_frame_answers_a_click`                |
| 6    | `an_ascii_box_is_drawn_with_the_ascii_table`                  |
| 7    | `a_drop_down_leaves_the_region_byte_identical_when_it_closes` |
| 8    | `an_empty_diagram_draws_nothing`                              |
| 8    | `a_region_with_no_interior_draws_nothing`                     |
| 9    | `a_path_that_cannot_be_read_is_reported`                      |
| 9    | `a_path_that_is_not_a_description_is_reported`                |
| 10   | _nothing holds this: the dependency list is read, not tested_ |
| —    | `the_screen_with_the_demo` (snapshot)                         |

## Open questions

**The status bar's content.** It still names the application and its version. What it says about the
diagram, the selected shape or the file is #177's and #181's.

**A catalog chosen by the configuration.** The application draws with the one the command line draws
with. Whether the configuration holds a catalog, and whether a description may name one, is not this
slice's.

**The scroll origin's bounds.** #176 asks how far the viewport may move, and the model holds that
nothing measures a diagram today. This slice draws at `(0, 0)` and does not answer it.

**The application's demonstration.** Whether it should ship a sample chosen for a small region
rather than the demonstration's 50×13 is not decided here.

**The light terminal and `NO_COLOR`.** #189's, unchanged.
