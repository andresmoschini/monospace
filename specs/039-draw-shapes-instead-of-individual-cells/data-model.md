# Phase 1 data model: Draw shapes instead of individual cells

**Feature**: 039 | **Date**: 2026-09-10

The entities are the model's, from _Shapes_, _Pieces_, _The initial set_, _An end is an arm; a head
is a glyph_ and _The route of an arrow_ in [`docs/model.md`](../../docs/model.md). This file records
what each becomes in the crate and what each piece writes — it does not redefine any of them.

Every cell rule below is one of ADR-0008's and ADR-0026's, assigned to exactly one fragment by
[ADR-0028](../../docs/decisions/0028-give-each-fragment-its-own-cell-rule.md). No figure in the
lower half of this file names an arm, and that is the point of the split.

## The two abstractions

| Type      | Visibility | Operation                                | Notes                                           |
| --------- | ---------- | ---------------------------------------- | ----------------------------------------------- |
| `Surface` | `pub`      | `stamp(&mut self, at: Pos, cell: Cell)`  | One write, no reader. ADR-0031                  |
| `Shape`   | `pub`      | `draw(&self, surface: &mut dyn Surface)` | One operation, and a shape reports nothing else |

`Layer<'a>` is the only `Surface` the crate ships: a `&'a mut Buffer` and a `StampMode`, bound at
construction so no shape ever names a mode. A test module adds a second implementation that counts
writes per position, which is how FR-020 is observed.

## Supporting types

| Type          | Visibility   | Values                        | Where it lives | Why                                                          |
| ------------- | ------------ | ----------------------------- | -------------- | ------------------------------------------------------------ |
| `Side`        | `pub(crate)` | `Top` `Right` `Bottom` `Left` | `cell.rs`      | A place on a cell. Only fragments name it                    |
| `Direction`   | `pub`        | `Up` `Right` `Down` `Left`    | `geometry.rs`  | A way to move. `Endpoint` names it, and `Endpoint` is public |
| `Orientation` | `pub`        | `Horizontal` `Vertical`       | `geometry.rs`  | `Line` names it                                              |

`Side` and `Direction` stay apart — the model's _Vocabulary_ and ADR-0028. The conversion between
them exists in `Line` and in `Route` and nowhere else: a fragment is told sides, and the figure
above it is the only thing that reasons in directions.

## The fragments

All six are `pub(crate)`, all six implement `Shape`, and all six are **fragments** in the sense of
_Complete and fragment_: each writes only the cells its description names and adds no decoration of
its own. Each derives the cell it writes; none takes one.

| Fragment  | Description                                       | Covers      | The cell it writes                                                 |
| --------- | ------------------------------------------------- | ----------- | ------------------------------------------------------------------ |
| `Corner`  | `at`, the two `Side`s it opens toward, `stroke`   | one cell    | `Set` on those two sides, `Unset` on the other two                 |
| `Segment` | `from`, `len`, `Orientation`, `stroke`            | a run       | `Set` on both sides along the run, `Unset` on the two across it    |
| `End`     | `at`, the `Side` the stroke runs toward, `stroke` | one cell    | `Set` on that side, `Unset` on the other three. ADR-0029           |
| `Border`  | `from`, `len`, the `Side` of the figure, `stroke` | a run       | `Set` along the run, `Closed` facing the interior, `Unset` outward |
| `Fill`    | `at`, `size`, `glyph`                             | a rectangle | the glyph, as `Cell::Literal`                                      |
| `Head`    | `at`, `glyph`                                     | one cell    | the glyph, as `Cell::Literal`. ADR-0029                            |

`Border` derives both its orientation and its closed side from the one `Side` it is told, so a
horizontal border whose interior is to its left cannot be constructed. `Corner` is told the two
sides it opens toward, so a corner opening `Top` and `Bottom` is expressible and is a straight run —
which is the general rule answering, not a case.

The loop over a run and the loop over a rectangle are one private helper each, shared by the
fragments that need them. That helper is the whole of what the rejected single-leaf design would
have saved.

## The figures

Three, all `pub`, all **complete** shapes: what a library user sees, and each draws finished. Each
is also a compositor to its own pieces, which is the both-at-once case FR-014 names.

### `BoxShape`

`at: Pos`, `size: Size`, `stroke: Stroke`, `fill: Option<Glyph>`. Named `BoxShape` because `Box` is
in the prelude — research Q6.

With `x0 = at.x`, `x1 = x0 + width - 1`, `y0 = at.y`, `y1 = y0 + height - 1`:

| Piece                                                          | When                                              |
| -------------------------------------------------------------- | ------------------------------------------------- |
| `Corner` at `(x0, y0)` opening `Right`, `Bottom`               | always                                            |
| `Corner` at `(x1, y0)` opening `Bottom`, `Left`                | always                                            |
| `Corner` at `(x1, y1)` opening `Top`, `Left`                   | always                                            |
| `Corner` at `(x0, y1)` opening `Top`, `Right`                  | always                                            |
| `Border { side: Top }`, `x0 + 1 ..= x1 - 1` at `y0`            | `width > 2`                                       |
| `Border { side: Bottom }`, the same run at `y1`                | `width > 2`                                       |
| `Border { side: Left }`, `y0 + 1 ..= y1 - 1` at `x0`           | `height > 2`                                      |
| `Border { side: Right }`, the same run at `x1`                 | `height > 2`                                      |
| `Fill` over `(x0 + 1, y0 + 1)` sized `(width - 2, height - 2)` | `fill.is_some()` and `width > 2` and `height > 2` |

**Below 2 in either dimension the box has no pieces at all.** FR-021, and the one guard this feature
has: user story 1's fourth acceptance scenario reaches it, so SC-004 is discharged by removing it
and watching that test fail.

The decomposition depends on the size rather than on the kind, which is FR-008: a 2×2 box is four
corners and nothing else, a 6×3 box is four corners and four border runs, and an unfilled box places
no interior piece at all — ADR-0030 removed the one that used to exist only to carry positions.

`monospace-cli`'s twelve hand-written stamps are this decomposition for a 4×3 filled box, position
for position. That correspondence is the whole content of the final `refactor` commit.

### `Line`

`at: Pos`, `len: u32`, `orientation: Orientation`, `stroke: Stroke`. It names no glyph — ADR-0029.

Let `forward` be `Right` for `Horizontal` and `Bottom` for `Vertical`, and `backward` its opposite.
The positions are `p[0] ..= p[len - 1]` along the orientation.

| Piece                                   | When       |
| --------------------------------------- | ---------- |
| `End` at `p[0]` toward `forward`        | `len >= 1` |
| `Segment` from `p[1]`, length `len - 2` | `len >= 3` |
| `End` at `p[len - 1]` toward `backward` | `len >= 2` |

Read as a single pass that assigns each position exactly once, in that order, rather than as three
guards: a length of 1 gives one `End` and a length of 0 gives nothing, so FR-020 holds at every
length **by the decomposition** and no guard produces it — user story 2's eighth scenario. FR-022's
"no minimum length is declared" is this table having no lower bound in it.

What the one cell of a length-1 line carries is `End` toward `forward`. It is the rule's answer and
the spec pins nothing about it.

### `Arrow`

`from: Endpoint`, `to: Endpoint`, `stroke: Stroke`, where `Endpoint` is `at: Pos`,
`leaving: Direction`, `head: Glyph` — all `pub`.

| Piece                                | When                       |
| ------------------------------------ | -------------------------- |
| `Head` at `from.at` with `from.head` | always                     |
| `Head` at `to.at` with `to.head`     | always                     |
| `Route` over the derived path        | when the path is non-empty |

A head points opposite to the direction its endpoint leaves in; which glyph does that is the
caller's, FR-027 and FR-028, and `Arrow` neither checks nor derives it.

`Arrow` computes the whole path before constructing `Route`, so FR-016's "whatever mechanism
supplies where it starts" is: it is told. Nothing reports anything back.

## `Route`

`pub(crate)`, a compositor rather than a leaf: it places `Corner`s and `Segment`s and writes no
position itself. It receives the route positions — the derived path without its two endpoint
positions, which carry the heads — and a stroke.

For each route position, the cell has a stroke toward its predecessor and toward its successor in
the **full** path, the two endpoint positions included. So the outermost route cell carries an arm
pointing at a head, which is what makes scenario 1's `(0, 1)` render `│` rather than a half-line.

| Route position                                | Piece                          |
| --------------------------------------------- | ------------------------------ |
| predecessor and successor on different axes   | a `Corner` opening toward both |
| a maximal run with both neighbors on one axis | one `Segment` covering the run |

A bend belongs to exactly one `Corner` and no `Segment` contains one, which is how FR-020 survives a
route with four bends — the failure SC-006 requires be produced on purpose is a route drawn as
overlapping runs that share their bends.

### Deriving the path

The algorithm and the hand-worked evidence are in [`research.md`](research.md), Q5. In brief:
`start_a` and `start_b` are one step from each endpoint in that endpoint's own leaving direction;
the route rectangle is their bounding box; candidates are simple alternating orthogonal paths whose
turns fall on the lattice of the two starting coordinates and the rectangle's middle on each axis;
the path is the candidate with fewest bends, then the one turning nearest the middle, then the
lexicographically first. **No candidate means an empty route and an arrow that is its two heads** —
step 4 of a total rule finding nothing, not a case written for a degenerate arrangement, which is
_Degenerate arrangements_ and FR-017.

## What this feature does not add

`Buffer`, `stamp`, `Cell`, `StrokeCell`, `Arm`, `Stroke`, `Glyph`, `GlyphCatalog`, `GlyphKey`,
`render`, `Pos` and `Size` are unchanged in what they do — FR-024. `cell.rs` and `geometry.rs` gain
`Side`, `Direction` and `Orientation` and lose nothing. `docs/glyph-sets.md` gains no rule — FR-027,
SC-010.

No shape reports what it covers. There is no extent, no bounding rectangle and no method that could
carry one — ADR-0030.
