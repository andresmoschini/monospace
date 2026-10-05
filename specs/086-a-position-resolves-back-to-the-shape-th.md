---
status: implemented
decided: "#170"
implemented: "#171"
date: 2026-10-04
---

# A position resolves back to the shape that decided it

## Why now

A drawn diagram is a picture with no way back into the model, and an interactive front end is the
first thing that has to answer a click. Issue #86 asks for the half that makes the answer possible:
the buffer records, at every position, which shape decided the cell there, and asking a position
returns a shape rather than a character.

**The rule is already in the model** — `docs/diagram-model.md` §8 _Ownership_, with three properties
in §10 that no test names yet — so this is not a change that needs the model to grow a rule. What is
open is where the record lives and what it holds, and §8 answers that in a sentence with no reason
attached to it: _"What the buffer keeps is a token it never interprets; the mapping from that token
to a `ShapeId` is the diagram's."_ The three reasons §8 does give for keeping the record in the
buffer are all kept by holding the name itself, and its own _"asking what is at a position is asking
the buffer"_ is what a name answers directly. That is why this change is two stages: the decision is
real and it is about the API the interactive front end will sit on. The issue's own decision record,
ADR-0043, was removed with the rest of the artifacts, so this is where the decision is taken now.

## Scope

### In

- **`ShapeId` moves down to `monospace-core`**, the type and nothing else. `monospace-diagram`
  re-exports it, so nothing above the core changes and no call site moves.
- **`Buffer` records the owner beside its cells**, and answers for a position: `Buffer::owner`.
- **`Buffer::stamp` grows an owner argument**, `Layer` carries it, and `Surface::stamp` and the six
  fragments that call it are untouched.
- **`Diagram::draw` binds each shape's identity as it draws.** Its signature does not change, and
  neither does the fact that the window is the caller's.
- **The demonstration**, which writes three identities out by hand today, resolves them by asking
  the picture instead, and gains an eighth step that shows what one answer is worth. How it does
  that is D4.
- **The arithmetic a caller does to turn a click into a position**, stated as a rule with three
  measured windows, because a window whose origin is not `(0, 0)` is the ordinary case and not the
  corner one.

### Out

- **A second owner for a position.** §8 defers the list to "when an editor needs it", and nothing
  needs it: a click resolves to one shape, and the front-most one is the one on the screen. An arm
  inside a cell cannot be pointed at separately, which is why §8 says so.
- **A TUI, a terminal, anything that takes a click.** Nothing here draws one; this slice is what
  makes the click answerable when one exists.
- **Reporting the shapes a diagram could not draw** ([#88]), and **editing an identity after the
  fact**, which §11 already holds open.
- **A hit test against a shape's extent rather than its cells.** See `## Open questions`: an
  unfilled box owns nothing inside itself, and that is a real answer rather than an accident, but it
  is not the only answer a front end could want.

## The decision

**D1 — the buffer holds the shape's identity itself, and `ShapeId` moves down to the core to let
it.**

**Answer:** `monospace_core::ShapeId`, re-exported by `monospace-diagram`, and a `Buffer` that
records it beside each cell. **Why not** the token §8 describes, with the mapping held by the
diagram: the three reasons §8 gives for putting the record in the buffer are kept, and the sentence
asking for a token carries none of its own — with a token the caller asks the buffer and then asks
the diagram what the answer means, and `Diagram::draw` hands back a value it does not hand back
today. **Why not** the type staying in `monospace-diagram` and the buffer holding a `String`:
nothing would say what the string is, and the one type that says it would be two crates away. The
cost is a `ShapeId` clone per stamped position, and **it is not claimed as a cost until it is
measured** — the fix if it ever is one is a borrowed `&ShapeId` per position rather than a smaller
token. **Answered by** the maintainer, in the session that wrote this.

**D2 — the identity travels on the layer, not through every stamp.**

**Answer:** `Layer` carries `Option<&ShapeId>`; `Diagram::draw` binds one layer per shape. A shape
is never told which identity it draws under, and `Surface::stamp` keeps its one argument. **Why
not** `Surface::stamp(at, cell, owner)`: ownership is per shape — §8 asks which _shape_ decided the
cell — and a figure with an identity parameter is a figure that can be drawn under two, or under
none. **Why not** a fourth argument on `Layer::new`: it is 55 mechanical edits in the core's own
tests, for no behavior, and a second constructor beside it says the same thing.

**D3 — the query takes an offset into the buffer's own window, and the caller does no arithmetic.**

**Answer:** `Buffer::owner(at: Offset)`, where an `Offset` is a pair of `u32` counted from the
window's top-left corner, and `None` for anything the window does not hold. The offsets a caller
holds are the ones it was given: a click arrives as a column and a row, and an interactive front end
that draws what fits at the scroll position it is at asks about that without reading the window's
`origin` first. **Why not** an absolute `Pos`, which is what `cell` takes: a window at `(2, 2)` four
by four answers `None` for `(0, 0)`, because it holds no such cell — so the absolute spelling can
only ever ask the second question, and a buffer that answers about a cell it does not hold is
answering about another one. **Why not** `owner(dx, dy, origin)`, unsigned offsets with the absolute
origin beside them: the origin is already a field of the buffer the caller built, and passing it
twice is a chance to pass two different ones. **Answered by** the maintainer, in the session that
wrote this.

**D3a — the rendered rectangle is the window, and a caller drawing a viewport out of a bigger buffer
has to know it is doing that.**

**Answer:** the offsets are counted from the **window's** corner, which is also the corner the
rectangle rendered from has in every caller in this repository: `render` is handed the window's own
origin and size in all four of them. **Why not** leave it unsaid: a caller that keeps one buffer for
a whole diagram and renders a viewport out of it holds click offsets relative to the _viewport_, and
the relative API then answers about a different and possibly off-screen cell without reporting
anything. That is the one silent wrong answer this API can give, and a paragraph is what it costs to
rule out. **Why not** forbid it by making `render` take the window and nothing else: that changes
`render`'s own signature and its tests, which is wider than this slice, and this paragraph plus a
named `Offset` is what a caller needs to be right either way.

**D4 — the demonstration finds its three shapes by asking the picture, and then shows one answer.**

**Answer:** both halves. The three identities the demonstration writes out by hand are resolved
through `Buffer::owner` on its first picture, and the captions name the offsets they were found at;
and an eighth step asks the picture for one offset and takes out the shape the record names, so a
reader sees what the record decided rather than a claim that it decided something. **Why not** the
substitution alone: it trades "the first entry" for "the offset `(0, 0)`", which is the same class
of assumption about this demonstration, and **no picture moves** — the reader would be told the new
behavior exists and shown nothing. **Why not** a caption naming an owner and stopping there: a
caption is a claim, and the eighth step is one a reader can check with their eyes. **Why not**
drawing the found shape alone in a window sized to it: nothing measures a shape's extent —
`docs/diagram-model.md` §11 holds that question open — so a window for one would be a new rule
rather than a demonstration. The eighth step asks for **`(20, 2)`**, which two shapes wrote, and
taking out the one the record names leaves the other's arm standing in that cell: `┼` becomes `│`.
**Answered by** the maintainer, in the session that wrote this — the demonstration belongs to this
change, since it is what lets the new behavior be seen. Which offset it asks about, and that asking
is followed by taking out rather than by a line of text, is practice settled as written.

**D5 — `stamp` grows one argument rather than gaining a second door.**

**Answer:** `Buffer::stamp(at, cell, mode, owner: Option<&ShapeId>)`, where `None` is the answer for
a stamp by nobody — the core's own gallery draws figures with no identity, and a stamp that could
silently record nothing is a bug nobody would notice. **Why not** keeping the three-argument `stamp`
and adding `stamp_owned`: two doors is a place where "changed nothing, took nothing" and "records
nothing" get confused, and the second is silent. The cost is 55 call sites, every one of them a test
inside `monospace-core`.

## Model slice

- `docs/diagram-model.md` §8 _Ownership_: the sentence about a token and a mapping held by the
  diagram is replaced by the record holding the identity itself, and the paragraph gains the
  question in D3 — a position asked for as an offset into the window, which is what a click carries.
- `docs/diagram-model.md` §3 _Identity_: one sentence saying the type is the core's and the rules
  stay here, since a type that has moved crates and a rule that has not are two different facts.
- `docs/diagram-model.md` §10: the third property is amended rather than a fourth added. "And so
  does one outside the window" already covers what D3 changes — an offset the window does not hold —
  and what the amendment adds is the viewport sentence of D3a, which is the one caller that has to
  know it is drawing a rectangle other than the window.
- `docs/model.md` §1 _Vocabulary_ and §Stamping: a row for `ShapeId` and one for `Offset`, and one
  paragraph saying that a stamp carries who is stamping, that the buffer keeps that beside the cell
  rather than inside it, and that `Offset` is counted from the window's corner where `Pos` is a
  point in the plane.

All four land in the **building** stage, beside the code they describe. The deciding stage changes
no model, and a spec that contradicts the model for a whole stage would be a rule with one reader.

## Public surface

```rust
// monospace-core — moved here from monospace-diagram, unchanged otherwise
pub struct ShapeId(String);
impl ShapeId {
    pub fn new(text: impl Into<String>) -> Self;
}
impl std::fmt::Display for ShapeId {
    /* writes the identity as it was written: "#1" */
}

// monospace-core — a position in the window, counted from its top-left corner
pub struct Offset {
    pub x: u32,
    pub y: u32,
}

// monospace-core — the record and the query
impl Buffer {
    pub fn stamp(&mut self, at: Pos, cell: Cell, mode: StampMode, owner: Option<&ShapeId>);

    #[must_use]
    pub fn owner(&self, at: Offset) -> Option<&ShapeId>;
}

pub struct Layer<'a, 'b> { /* … */ }
impl<'a, 'b> Layer<'a, 'b> {
    // Unchanged: a layer bound to nobody owns nothing.
    #[must_use]
    pub fn new(buffer: &'a mut Buffer, mode: StampMode) -> Self;

    #[must_use]
    pub fn stamped_by(buffer: &'a mut Buffer, mode: StampMode, owner: &'b ShapeId) -> Self;
}

// monospace-diagram
pub use monospace_core::ShapeId; // re-exported, so no call site above the core moves

impl Diagram {
    pub fn draw(&self, buffer: &mut Buffer); // unchanged, and it binds each identity itself
}
```

And what a caller does with it, which is the whole point of D3 — the click's column and row go
straight in, and nothing is added to anything:

```rust
let mut buffer = Buffer::new(window.origin, window.size);
diagram.draw(&mut buffer);
let text = render(&buffer, &catalog, window.origin, window.size);

// A click arrives as a column and a row in what was rendered, which is the window.
let clicked: Option<&ShapeId> = buffer.owner(Offset { x: column, y: row });
```

## Behavior

1. The record is kept beside the cell, never inside it: two cells that look the same are the same
   cell whoever wrote them.
2. A cell belongs to the front-most shape that decided it.
3. A stamp that changes nothing at a position takes nothing.
4. A stamp that decides no side — four `Unset` arms — takes nothing either, and the position stays
   with whoever holds it or with nobody.
5. `Below` never takes a position that already has an owner, so a shape behind cannot take a cell
   from one in front of it.
6. `Above` takes a position whenever it decides something, which with 2 and 5 is what makes the
   record the same whether a diagram is drawn front to back or back to front.
7. `owner(at)` takes an `Offset` counted from the window's top-left corner, and answers `None` for
   an offset the window does not hold and for one inside it that no shape wrote. It does **not**
   take the `Pos` that `cell` takes: `owner` answers about a cell of _this window_, `cell` about a
   point in the plane. The two conventions sit side by side in one type, named apart, because the
   first is what a click carries and the second is what a figure is placed at.
8. The window's `origin` takes no part in any answer beyond which cells the window holds. A caller
   that reads it, adds it to a click and asks with the sum has asked about a different cell, and
   gets a truthful answer about that one.
9. An offset the window's own origin cannot carry — a window at `-2147483648` and any column above
   zero — is owned by nobody, and so is every cell it stands on. The arithmetic that decides this is
   the buffer's, and no caller performs it.
10. The record belongs to the drawing that produced it: changing the diagram does not update it, and
    drawing again into a buffer that already holds cells records nothing where it changed nothing.
11. A shape is drawn under whatever identity its layer was bound to, and a shape drawn by nobody
    owns nothing.
12. Two shapes carrying one identity both record that identity. The record cannot tell them apart,
    exactly as `Diagram::get` cannot.

## Examples

The pictures below are **measured**, not composed: each was rendered through the shipped description
format and read back, and `cargo xtask render --check` holds them. The `owner` answers are **not** —
there is no code to observe them with, and they are the arithmetic of rules 2 to 7 over those
pictures. That is what the acceptance list in `## What proves it` is for.

One thing is worth knowing before counting columns, because it decided the shape of every example
here: **`cargo xtask render` strips the trailing blanks of each row**, so a window's last column
survives in a picture only where something was drawn in it. Each window below is sized so that every
cell the tables cite is one a reader can see.

**A window whose origin is not `(0, 0)`.** A light box at `(0, 0)`, a filled double box at
`(-2, -1)` — in front of it, since it is written last — and a heavy box at `(6, 3)`, entirely
outside the window. The window is seven by five and starts at `(-3, -2)`, so it holds `x -3..3` and
`y -2..2`:

<!-- render:
{ "canvas": { "origin": { "x": -3, "y": -2 }, "size": { "width": 7, "height": 5 } },
  "next_id": 4,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": -2, "y": -1 }, "size": { "width": 4, "height": 3 },
      "stroke": "double", "fill": "░" },
    { "kind": "box", "id": 3, "at": { "x": 6, "y": 3 }, "size": { "width": 4, "height": 3 },
      "stroke": "heavy" } ] }
-->

```text

 ╔══╗
 ║░░╟─┐
 ╚═╤╝ │
   └──┘
```

<!-- /render -->

Each row is a position asked for, and beside it the offset that reaches it — the column and row a
click would carry, counted from the window's top-left corner and handed to `owner` unchanged.

| Asked for  | As an offset | Cell  | `owner`                                                       |
| ---------- | ------------ | ----- | ------------------------------------------------------------- |
| `(-3, -2)` | `(0, 0)`     | space | `None` — the window holds it and no shape wrote it            |
| `(-3, 0)`  | `(0, 2)`     | space | `None`                                                        |
| `(-2, -1)` | `(1, 1)`     | `╔`   | `#2`                                                          |
| `(0, 0)`   | `(3, 2)`     | `░`   | `#2` — a literal, and the front-most shape at that cell       |
| `(1, 0)`   | `(4, 2)`     | `╟`   | `#2`, although the right arm of that cell was written by `#1` |
| `(2, 2)`   | `(5, 4)`     | `─`   | `#1`                                                          |
| `(3, 1)`   | `(6, 3)`     | `│`   | `#1`                                                          |
| `(4, -2)`  | `(7, 0)`     | —     | `None` — outside the window, which is what rules 7 and 8 say  |

Two of those rows are rules rather than arithmetic. **`╟` at `(1, 0)` is a crossing whose right arm
was written by `#1` and whose owner is `#2`**, because `#2` is in front and a cell is owned whole —
which is the one answer a reader cannot check with their eyes, and the reason the record cannot be
derived from the character. And **`#3` owns nothing anywhere**: it is outside the window, so it
stamped nothing, and no click reaches it either.

**A shape half out of the window.** A light box at `(0, 0)` drawn into a three-by-three window whose
origin is `(1, 1)`, so only its right side and bottom-right corner are inside. The blank cells that
matter here sit at the **front** of their rows, which is what keeps them in the picture:

<!-- render:
{ "canvas": { "origin": { "x": 1, "y": 1 }, "size": { "width": 3, "height": 3 } },
  "next_id": 2,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" } ] }
-->

```text
  │
──┘

```

<!-- /render -->

`(3, 1)` and `(2, 2)` are `#1`. **`(2, 1)` is `None` even though the box covers it**: the box's cell
there is its top border at `(2, 0)`, which is outside the window, and a stamp outside the window
changes nothing — `stamping_outside_the_window_changes_nothing` in
`crates/monospace-core/src/buffer.rs` already holds that half. `(0, 0)`, the box's own top-left
corner, answers `None` as well: the window does not hold it, so nothing was ever stamped there. An
offset inside the window cannot reach either of those two, which is the point — the offsets are the
window's own, so nothing outside it can be asked about at all.

**An origin no offset can be added to.** Measured: a four-by-three window whose origin is
`(-2147483648, 0)`, holding a box at `(0, 0)`, renders as four blank columns and three blank rows,
because the window holds no cell and no offset in it is a position anything can carry. Under D3 that
is the buffer's own arithmetic and not the caller's, which is why it is one sentence here rather
than an example with an API beside it — and why **it has no marker and cannot have one**:
`checked_add_unsigned` fails monotonically along an axis, so the columns a window like that drops
are always a trailing run of blanks, and `cargo xtask render` strips exactly those.

**The demonstration's own crossing, and why the answer is not in the character.** Three cells of the
shipped demonstration's first picture, at `(9, 2)`, `(10, 3)` and `(11, 3)`: `░`, `┘`, `░`. The
first two are `#4` — its fill, and its bottom-right corner over `#3`'s fill — and the third is
`#3`'s fill. **The first and the third print the same character and belong to different shapes**,
which is rule 1 drawn on the picture the binary already prints.

**Why there is no `render` marker for D4.** The answers to D4 differ in captions and in one picture
that `cargo xtask render` cannot produce: it renders a description into a single picture and nothing
else, and an ownership record is not a picture the format can express — it is invisible in every
picture above. The argument for D4 is therefore prose, and what verifies the examples is the marker
`cargo xtask render` rewrites.

## What proves it

None of these tests exists yet; the deciding stage adds no code. The names are what the building
stage is held to.

| Rule | Test                                                                                   |
| ---- | -------------------------------------------------------------------------------------- |
| 1    | `two_cells_that_render_the_same_are_the_same_cell_and_the_record_does_not_change_that` |
| 2    | `in_an_overlap_every_position_resolves_to_the_front_most_shape_that_decided_it`        |
| 3    | `a_shape_that_stamps_a_position_without_changing_anything_there_does_not_take_it`      |
| 4    | `a_stamp_that_decides_no_side_takes_nothing_and_leaves_the_owner_it_found`             |
| 5    | `a_shape_behind_cannot_take_a_cell_from_one_in_front_of_it_however_much_it_decides`    |
| 6    | `the_record_is_the_same_whether_a_diagram_is_drawn_front_to_back_or_back_to_front`     |
| 7    | `owner_answers_for_an_offset_into_the_window_and_none_for_one_it_does_not_hold`        |
| 8    | `a_caller_that_adds_the_windows_origin_to_a_click_asks_about_another_cell`             |
| 9    | `an_offset_a_windows_own_origin_cannot_carry_is_owned_by_nobody`                       |
| 10   | `a_diagram_changed_and_not_drawn_again_leaves_the_record_exactly_as_it_was`            |
| 11   | `a_shape_drawn_by_nobody_owns_nothing_even_where_it_writes`                            |
| 12   | `two_shapes_carrying_one_identity_both_record_it_as_get_cannot_tell_them_apart`        |

The two pictures are held by the render markers themselves: `cargo xtask render --check` redraws
them, and a picture that moved without a change here is the failure. The demonstration's three cells
are held by `the_shape_a_position_resolves_to_is_the_front_most_of_the_two_that_wrote_it` in
`crates/monospace-cli/src/main.rs`, which fails until the demonstration resolves its identities by
asking rather than by hand, and by
`the_shape_the_demonstration_finds_is_the_one_the_crossing_cell_names` for the eighth step — a
crossing cell that reads `┼` while the shape behind it stands, and `│` once the shape the record
named is gone.

## Open questions

- **An unfilled box owns nothing inside itself.** The demonstration's `#11` is a box four by three
  with no fill, and `(27, 1)` is inside it, blank in the picture the binary prints and therefore
  owned by nobody under rule 4. A person clicking there expects the box. What would settle it: the
  first front end that gets the click, since a hit test against a shape's **extent** rather than its
  cells is a different question from this one and the model has no rule for it. §8's deferred list
  of owners is not the answer either — the interior of an unfilled box has no owner to list.
- **Whether a second owner is ever wanted**, which §8 defers to an editor that needs it and this
  slice has no consumer for.
- **Editing an identity after the fact**, which §11 already holds open and D1 does not touch: the
  type moved crates, the question did not move with it.
- **Whether `cell` should follow `owner` into window-relative coordinates.** D3 leaves one type
  holding two conventions, named apart: `owner` takes an `Offset` because a click carries one,
  `cell` takes a `Pos` because a figure is placed at one. What would settle it: the first caller
  that asks `cell` a question about the window rather than about the plane, at which point the
  asymmetry is costing something rather than naming something. This is not a decision the slice has
  to take, and `Buffer::cell` is public API that a change would break.
