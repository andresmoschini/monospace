---
status: "accepted"
date: 2026-09-10
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Give each fragment its own cell rule instead of a cell parameter

## Context and Problem Statement

_Shapes_ in [`docs/model.md`](../model.md) says a shape is made either of cells or of other shapes,
and that one made of other shapes places pieces which do the writing. Feature 039 builds that layer,
so it has to decide what the leaf pieces are — the things that actually call `stamp`.

The first design gave the layer a single geometric leaf: one type holding a position, a size and a
cell, with constructors for the three geometries the model names — a corner is one cell, a border
run is a rectangle one cell thick, an interior is a rectangle. It factors the loop over positions,
which is real duplication, and it is one type instead of several.

It also parametrizes the cell. Every figure above it has to build a `StrokeCell` — a base stroke and
four `Arm`s — before it can place a piece, so `Arm`, `StrokeCell` and the choice between arms and a
literal become part of what a box, a line and an arrow each know. The decomposition table written
for the box under that design has nine rows and a column naming the four arms of each, which is one
rule of _The cell_ written out nine times. The model says the opposite in as many words: a box's
arms "are the ones _The cell_ already fixes", so the arms are not the box's to choose, and a design
where the box states them nine times has put the knowledge in the wrong place.

## Decision Drivers

- _The model owns the design_ in [the constitution](../../.specify/memory/constitution.md): the
  model already assigns a box's arms to _The cell_, and the code should not reassign them to the
  box.
- One rule, one home. Nine restatements of a cell rule is the same defect the constitution's
  _Cross-references_ and _The model owns the design_ exist to prevent, one layer down.
- Make invalid states unrepresentable where it is cheap to do so.
- FR-001 of feature 039: every character inside a figure is the figure's own business rather than
  the caller's. It says nothing about which layer inside the crate holds it.

## Considered Options

- **One geometric leaf parametrized by a cell** — position, size and a `Cell`, with named
  constructors for the three geometries.
- **A vocabulary of leaves named by their role**, each deriving its own cell from a description in
  domain terms.
- **No leaf layer at all**: each figure computes positions and stamps cells directly.

## Decision Outcome

Chosen option: **a vocabulary of leaves named by their role**, because the cell a piece writes
follows from what that piece _is_, and a design that passes the cell in has to state that rule again
at every call site.

The vocabulary, all `pub(crate)`:

| Fragment  | Description                                              | The cell it writes                                            |
| --------- | -------------------------------------------------------- | ------------------------------------------------------------- |
| `Corner`  | the two perpendicular sides a stroke turns between       | `Set` on those two, `Unset` on the other two                  |
| `Segment` | a run that continues past both of its ends               | `Set` along the run in every cell, `Unset` across it          |
| `End`     | the cell where a stroke stops, and the side it runs to   | `Set` on that side, `Unset` on the other three — see ADR-0029 |
| `Border`  | a run bounding a figure's interior, and which side it is | `Set` along, `Closed` facing the interior, `Unset` outward    |
| `Fill`    | an interior that hides what it covers                    | the chosen glyph, as a literal                                |
| `Head`    | an arrow's head                                          | the chosen glyph, as a literal — see ADR-0029                 |

`Route` sits above them and is a compositor rather than a leaf: it turns a path into `Corner`s and
`Segment`s. `Cell`, `StrokeCell` and `Arm` are named by those six files and by no other file in the
shape layer.

Two consequences of the same reasoning, recorded here because they are part of the decision rather
than separate from it:

- **A fragment is described in sides and positions already computed; reasoning about directions
  belongs above it.** `Border` takes the side of the figure it is, and derives its orientation and
  its closed side from that, so "a horizontal border whose interior is to its left" cannot be built.
  `Corner` takes the two sides it opens toward. Turning an arrow's leaving `Direction` into a
  starting position, and a path's steps into a corner's two sides, happens in `Arrow` and `Route`.
- **`Side` and `Direction` stay separate types.** They share four names and are not the same thing:
  a side is a place on a cell, a direction is a way to move. The conversion between them exists at
  one boundary, which is where the shapes above the fragments meet the fragments.

### Consequences

- Good, because each cell rule is stated once, in the rustdoc and the body of one small type, and a
  figure's decomposition says which pieces and where — not which arms.
- Good, because the six names are the model's own nouns. _The initial set_ already says a box is
  corners, border runs and an interior; _An end is an arm, a head is a glyph_ names the other two;
  _The route of an arrow_ names the route. Feature 039's _Key Entities_ claim that it introduces no
  entity of its own becomes literally true.
- Good, because `Border { side }` and `Corner { opens }` make a family of nonsense pieces
  unrepresentable rather than merely undocumented.
- Bad, because it is six types where one would have compiled, each of them a handful of lines. The
  duplication that buys is the loop over a rectangle, which survives as one private helper.
- Bad, because the vocabulary is `pub(crate)`, so **the guarantee is internal**. `Surface` is public
  and takes a `Cell`, so a shape defined outside this crate — which feature 039 requires to be
  possible — builds cells by hand and passes through none of these rules. Widening the vocabulary
  later is additive; that is the reason to start narrow, not a reason to pretend the hole is closed.
- Neutral, because a compositor allocates a `Vec<Box<dyn Shape>>` each time it decomposes. That cost
  belongs to composition itself rather than to this decision, and nothing in this phase measures it.

### Confirmation

By review, and by one thing anyone can count: no file in `crates/monospace-core/src/shape/` outside
the six fragment modules names `Cell`, `StrokeCell` or `Arm`. No gate step is added for it — this
feature adds no check — so the count is a grep a reviewer runs, not a guarantee the build makes.

## Pros and Cons of the Options

### One geometric leaf parametrized by a cell

- Good, because it is one type, and the three geometries the model names are three constructors on
  it rather than three types.
- Good, because the loop over a rectangle exists once, in the only place that needs it.
- Bad, because it moves `Arm` and `StrokeCell` into every figure, which is where the model says
  those decisions are not taken.
- Bad, because it names a geometry rather than a role, so the type cannot carry the rule and every
  call site carries it instead.
- Bad, because "an area with this cell" permits every wrong cell as readily as the right one.

### A vocabulary of leaves named by their role

- Good, because the type is the rule: there is one way to build a border and it is the correct one.
- Good, because the vocabulary reads as the model's prose reads.
- Bad, because more types, and because a new figure needing an unforeseen cell rule needs a new
  fragment rather than a new argument.

### No leaf layer at all

- Good, because it is the least code that could work.
- Bad, because it puts the cell rules in the figures, which is the first option's cost without the
  first option's saving, and because the model's "one made of other shapes writes no position
  itself" would have nothing to place.

## Reversibility

Cheap today: three figures, six fragments, all crate-private, and the public API does not mention
any of them. The cost grows with the number of figures, because each one is written against the
vocabulary — and it grows discontinuously if the vocabulary is ever made public, which would freeze
the names and the cell rules as a contract.

## Confidence

High (90%).

What would change it: a figure whose cells follow no reusable rule, which would need either a
fragment used once or a way to pass a cell in after all. The observation that would prove it wrong
is a fragment count that grows as fast as the figure count — six fragments for three figures is
tolerable because four of them are already shared; twelve for six figures would mean the vocabulary
is not a vocabulary.

## More Information

- [ADR-0029](0029-draw-a-line-end-as-one-arm.md) settles what `End` and `Head` write, and why they
  are not the same kind of thing.
- [ADR-0030](0030-drop-extent-until-a-caller-needs-it.md) removes `extent` from the trait these
  fragments implement, in the same increment.
- [ADR-0008](0008-compose-overlapping-cells-with-three-state-arms.md) and
  [ADR-0026](0026-represent-a-cell-as-a-sum-of-strokes-and-a-literal.md) are the cell rules these
  fragments each hold one of.
- Feature 039's spec, `specs/039-draw-shapes-instead-of-individual-cells/spec.md`, and the review of
  its plan on 2026-09-10 that produced this record.
