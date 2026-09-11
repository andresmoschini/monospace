# Phase 0 research: Draw shapes instead of individual cells

**Feature**: 039 | **Date**: 2026-09-10

This file holds investigation local to feature 039. Where a finding turned out to be durable it is
an ADR and this file links to it instead of standing in for it — _Where a rationale goes_ in the
constitution.

Three ADRs were written before this plan and are inputs rather than findings:
[ADR-0028](../../docs/decisions/0028-give-each-fragment-its-own-cell-rule.md) fixes what a fragment
is described by, [ADR-0029](../../docs/decisions/0029-draw-a-line-end-as-one-arm.md) fixes what an
end and a head write, and
[ADR-0030](../../docs/decisions/0030-drop-extent-until-a-caller-needs-it.md) leaves a shape with
drawing and nothing else. Nothing below reopens any of them.

## What the code already is

Read rather than assumed, at `cfa0d7f`:

- `crates/monospace-core/src/` is six modules: `buffer`, `cell`, `geometry`, `glyph`, `render`,
  `stroke`. 1,841 lines in the workspace, tests included.
- `Buffer::stamp(&mut self, at: Pos, cell: Cell, mode: StampMode)` is the only write, and
  `Buffer::cell(&self, at: Pos) -> Option<&Cell>` the only read.
- There is no `Side`, no `Direction`, no `Surface` and no `Shape` anywhere in `crates/`. Every type
  this feature needs above the buffer is new.
- `monospace-cli` has one call site, `stamp_box`, which writes a 4×3 filled box as twelve
  hand-computed stamps.

**The twelve stamps agree cell for cell with the decomposition in
[`data-model.md`](data-model.md).** Checked position by position against
`crates/monospace-cli/src/main.rs`: `(0, 0)` is `(Unset, Set, Set, Unset)`, which is `Corner`
opening right and bottom; `(1, 0)` is `(Unset, Set, Closed, Set)`, which is `Border { side: Top }`;
`(0, 1)` is `(Set, Closed, Set, Unset)`, which is `Border { side: Left }`; `(1, 1)` and `(2, 1)` are
the literal `░`, which is `Fill`. So the refactor the spec's _Handoff to the plan_ already settled
is byte-identical by construction, and `crates/monospace-cli/tests/cli.rs` passing unchanged is what
confirms it.

## Q1. What does a shape draw into?

**Decision**: a `Surface` trait with one write operation and no reader, and a `Layer` adapter that
binds a `&mut Buffer` to a `StampMode`.

```rust
pub trait Surface {
    fn stamp(&mut self, at: Pos, cell: Cell);
}

pub struct Layer<'a> { /* &'a mut Buffer, StampMode */ }
impl Layer<'_> { pub fn new(buffer: &mut Buffer, mode: StampMode) -> Layer<'_>; }
impl Surface for Layer<'_> { /* forwards to Buffer::stamp with the bound mode */ }
```

The maintainer took "a shape draws into a surface, not into a buffer" on the first plan, and the
spec's _Handoff to the plan_ records that the name and the shape of the trait are still the plan's.
Three things follow from the trait having no reader and no mode:

- FR-015's "a fragment MUST NOT inspect the buffer" holds structurally. There is nothing to inspect:
  `Surface` exposes no `cell`.
- The write counter FR-020 needs is a second `impl Surface` in a test module, not a change to
  `Buffer`. FR-024 stays satisfied without an argument.
- The stamp mode is the caller's, taken once when the `Layer` is made, and no shape ever names
  `StampMode`. A shape that took a mode would be deciding how it composes with figures it has no
  opinion about, which _Shapes_ forbids in as many words.

`Layer::new(&mut buffer, mode)` rather than a `Buffer::layer` constructor, so that `Buffer` gains no
item at all and FR-024 is a fact about an untouched file rather than a judgement about whether a new
constructor changes what the buffer does.

**Alternatives considered.** `Surface::stamp(at, cell, mode)` — rejected: it puts the mode in every
shape and every fragment, which is the same defect ADR-0028 removed for cells one layer down.
`impl Surface for Buffer` with `Above` implied — rejected: it deletes the caller's choice between
the two modes, which user story 2's scenario 4 asserts under both. Passing `&mut Buffer` directly —
refused by the maintainer before this plan, and the two consequences above are why it stays refused.

**This needs a record and a model amendment before any code depends on it.** See _Preparatory work_
below.

## Q2. Is the complete/fragment distinction a type?

**Decision**: a convention, carried in rustdoc, with no type and no marker.

The spec settled it while this was still open. FR-026 requires that "the documentation of a shape
MUST say whether it is complete or a fragment, **because no type carries that distinction**", which
is the answer stated as an obligation. ADR-0030 is why it stopped wanting to be a type: with the
extent gone, a shape has exactly one operation, so there is nothing a second type could promise that
the first does not.

What enforces it instead is visibility. The six fragments are `pub(crate)`; `BoxShape`, `Line` and
`Arrow` are public. A library user cannot name a fragment, so the distinction a type would have made
unrepresentable is unreachable anyway, one layer out.

## Q3. Static or dynamic dispatch, and does a compositor allocate?

**Decision**: `fn draw(&self, surface: &mut dyn Surface)`, and a compositor calls its pieces
directly. No `Box<dyn Shape>`, no `Vec` of pieces, no allocation in the shape layer.

```rust
pub trait Shape {
    fn draw(&self, surface: &mut dyn Surface);
}
```

`&mut dyn Surface` rather than a generic `S: Surface` because a generic `draw` is not object-safe,
and FR-003 requires composition to arbitrary depth with a shape placed as a piece drawable the same
way at the top level. `&self` is FR-006: a shape is a value and drawing it twice writes twice the
same.

A compositor's `draw` builds each piece as a concrete value and calls `piece.draw(surface)` on it,
so nothing is boxed:

```rust
Corner { at, opens: (Side::Right, Side::Bottom), stroke }.draw(surface);
```

ADR-0028 anticipated `Vec<Box<dyn Shape>>` and marked the allocation neutral. That was a consequence
note about the design it recorded, not part of what it decided, and the design it recorded is
unchanged by not allocating: the pieces are the same six fragments with the same cell rules. Direct
calls are recorded here rather than there because nothing outside this feature depends on it.

**Alternative considered**: a `fn pieces(&self) -> Vec<Box<dyn Shape>>` on compositors, with `draw`
defaulting to drawing each. It reads well and it makes a decomposition testable without a surface —
but a decomposition is already observable through a counting surface, which FR-020 needs anyway, so
the `Vec` buys a second way to see the same thing at the cost of an allocation per figure per draw.

## Q4. How does a fragment learn about its surroundings, and a route where it starts?

**Decision**: one mechanism, not two. Everything a piece needs is in the value the figure above it
constructs, and the figure computes all of it before placing anything.

The spec's _Handoff to the plan_ asks for this to be answered explicitly rather than by accident.
FR-015 and FR-016 look like two questions and are one: a fragment is told sides and positions
already computed (ADR-0028), and an arrow's route is derived by `Arrow` — from the two endpoints and
the two directions and nothing else — before `Route` is constructed. `Route` receives a path that is
already a list of positions. It never asks where it starts, because it is told.

So the direction-to-side translation happens at exactly two places, which is what ADR-0028 means by
"the boundary where the shapes above the fragments meet the fragments": `Line` turns its orientation
into the `Side` its two `End`s run toward, and `Route` turns each pair of consecutive path steps
into the two `Side`s a `Corner` opens toward. `Arrow` reasons in `Direction`s and constructs no
cell.

## Q5. How is an arrow's route derived?

**Decision**: enumerate the orthogonal paths that turn only on a three-by-three lattice of
coordinates, and select by fewest bends, then by distance from the middle of the route rectangle.

_The route of an arrow_ in [`docs/model.md`](../../docs/model.md) states the rule and this is how it
is computed. Given endpoints `a`, `b` and leaving directions `da`, `db`:

1. `start_a = a + da`, `start_b = b + db`. The route rectangle `R` is the bounding box of the two
   starting positions.
2. The lattice is `{start_a.x, start_b.x, mid_x(R)} × {start_a.y, start_b.y, mid_y(R)}` — at most
   nine points, fewer when coordinates coincide.
3. A **candidate** is a simple orthogonal path `a → start_a → … → start_b → b` whose interior turns
   are lattice points, whose consecutive runs alternate axis — which forbids a reversal, and so
   forbids a path that doubles back into either head — and which stays inside `R` between `start_a`
   and `start_b`. A candidate whose interior passes through `a` or `b` is rejected: a head is a
   literal and a route position landing on one would be the double write FR-020 forbids.
4. The route is the candidate with the fewest bends over the whole path; ties are broken by the
   smallest total distance from each turn's free coordinate to the corresponding middle of `R`, and
   any remaining tie by the lexicographic order of the turn sequence, so the result is total and
   deterministic.
5. The route writes the path without its two ends. `a` and `b` carry the heads.
6. **Where no candidate exists the route is empty and the arrow is its two heads.** That is step 4
   finding nothing, not a branch written for it.

**Why a lattice rather than a search over cells.** A breadth-first search over the positions of `R`
is O(area) and would have to carry the alternation and the bend count as state; the lattice is at
most nine points regardless of how far apart the endpoints are, and every turn in every picture the
spec pins falls on one. Worked by hand against all ten pinned arrows before this was written:
scenario 5 turns at `x = 3`, the middle of `R`'s `x ∈ [1, 5]`; scenario 7's long run sits at
`y = 1`, the middle of `y ∈ [0, 2]`; scenario 8's three turns are all at forced coordinates and no
middle is free.

**The tie-break is load-bearing and was worked by hand.** Scenario 5 has three two-bend candidates —
turning at `x = 1`, `x = 3` and `x = 5` — and only the middle one is the pinned picture. Fewest
bends alone does not choose.

**The empty route is reachable and was worked by hand.** `(2, 0) Right → (2, 0) Left` gives
`start_a = (3, 0)`, `start_b = (1, 0)` and a rectangle one cell thick, where the only move out of
`start_a` toward `start_b` is a reversal: no candidate, empty route, two heads on one position. Two
identical directions with the endpoints in line on that axis collapse the same way. These are the
two arrangements the learning log records the first plan's rule having no answer for; here they are
an outcome of step 4 rather than a case.

**The even-span rounding is not pinned**, per _Clarifications_. `mid` is integer division of the two
bounds, and whatever picture that produces becomes the expected text of the test that covers an even
span. An arrow and its reverse are permitted to differ by one column.

**Everything above is derived on paper and is a prediction until it runs.** _Claims are measured,
not assumed_: the ten pinned pictures are the measurement, and any that the implementation
contradicts is this section being wrong rather than the spec.

## Q6. What is the box shape called?

**Decision**: `BoxShape`, in `crates/monospace-core/src/shape/box_shape.rs`.

The maintainer settled "not `Box`" on the first plan: `Box` is in the prelude and would shadow it
for anyone importing this crate, and `box` is a reserved keyword so the module cannot carry the word
either. The same ruling records the measurement that decides the rest — on the pinned toolchain,
with `clippy::pedantic` at `-D warnings`, `module_name_repetitions` does not fire on a suffixed type
inside a module of the same stem, and the probe was made to fail on purpose before the clean run was
believed. `BoxShape` in `box_shape` is that arrangement.

`box` stays the word in `docs/model.md`, in the spec and in every doc comment. `Line` and `Arrow`
need no suffix and get none; the asymmetry is the prelude's, not a preference.

## Q7. Where do `Side` and `Direction` live, and what is public?

**Decision**: `Direction` is public, in `geometry.rs`. `Side` is `pub(crate)`, in `cell.rs`.

They stay two types — ADR-0028, and the model's _Vocabulary_ says why. Placement follows the same
sentence: a side is a place on a cell, a direction is a way to move across the plane, so `Side` sits
with `Cell` and `Direction` with `Pos`.

`Direction` is public because `Endpoint` names it and `Arrow` is public. `Side` is `pub(crate)`
because only the six fragments name it and every one of them is `pub(crate)`: ADR-0030's driver —
every public item is a promise the WebAssembly boundary and every later phase inherit — says a
public item with no public reader is not carried. Making it public later is additive.

## Q8. How is "no position written twice" observed?

**Decision**: a test-only `impl Surface` that counts writes per position, in the shape layer's test
module.

Trivial now that `Surface` exists, and the reason the maintainer's ruling in Q1 pays for itself
twice: FR-020 is verified rather than asserted, and `Buffer` gains nothing. SC-006 requires it be
made to fail on purpose — a route drawn as two runs sharing their bend — and then restored, which is
_Claims are measured, not assumed_ applied to the counter itself.

## Preparatory work this research found

The question this plan was invoked with. The answer is in two halves.

**No structural change to existing code is needed, and none is manufactured.** Everything above the
buffer is new: a `shape` module tree, `Surface`, `Layer`, `Side`, `Direction`. `cell.rs` and
`geometry.rs` each gain a type and lose nothing. The existing core already matches the amended model
— checked by reproducing the box's arms from `stamp_box` above — so there is nothing to make easy
before making the easy change. _One definition of green_'s standard applies here too: a refactor
that cannot be shown to be needed is not written. The one `refactor` commit this feature has is at
the far end, where `monospace-cli` redraws its box through `BoxShape`.

**Two documents are behind the decisions already taken, and both are the plan's to fix first.**

1. **`docs/model.md` still says a shape draws into a buffer.** Its _Vocabulary_ row reads "A value
   describing a figure, which draws itself into a buffer"; _Shapes_ says "Shapes are the layer
   directly above the buffer"; _Pieces_ says "A fragment never inspects the buffer". After Q1 all
   three are wrong in the same way, and `Surface` has no row at all. _The model owns the design_ is
   explicit that a slice needing a rule the model does not have amends the model first.
2. **"A shape draws into a surface" is recorded only in a spec's _Handoff to the plan_.** The
   constitution says a spec MUST NOT take a decision, and that a decision is recorded as an ADR at
   the moment it is taken — before code depends on it. This one is expensive to undo once three
   figures, six fragments and a test surface are written against it, and it is exactly the kind of
   thing a reader asks "why is this like this?" about. It needs ADR-0031, which also carries the
   trait's shape from Q1 and the dispatch choice from Q3.

Both land before the first line of shape code, in that order: the record, then the model, then the
code. Neither is a scope widening — they are the two artifacts the decisions already taken were
supposed to land in.
