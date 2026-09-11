---
status: "accepted"
date: 2026-09-10
decision-makers: "Andrés Moschini, with Claude Sonnet 5"
---

# A shape draws into a surface, not into a buffer

## Context and Problem Statement

Feature 039 gives `monospace-core` a `Shape` abstraction: a value that draws itself by calling
`stamp`. Every existing call to `stamp` is a method on `Buffer`, which also reads — `Buffer::cell` —
and knows about `StampMode` at the call site. Handing a `Shape` a `&mut Buffer` directly is the
obvious way to let it draw, and the maintainer rejected it before this plan was written, which is
what makes this a decision to record rather than one still open.

_Shapes_ in [`docs/model.md`](../model.md) says a fragment "never inspects the buffer and never
inspects its siblings" (FR-015 restates the same rule as an obligation). That has to hold for every
fragment feature 039 writes, not just the ones this review happens to notice, and a `&mut Buffer`
argument makes it a convention to audit rather than a fact the compiler can check: nothing stops a
future fragment from calling `Buffer::cell`. `StampMode` is a second, smaller problem: nothing says
which of the two modes a shape draws under, and a shape that took one would be forming an opinion
about how it composes with figures behind it — the thing _Shapes_ says a shape has none of.

## Decision Drivers

- FR-015: a fragment must not inspect the buffer or its siblings. A structural guarantee holds for
  every fragment written after this decision, not only the ones reviewed against it.
- FR-020: no shape writes a position more than once in one drawing, and this has to be verified
  rather than asserted — a test needs a second implementation of whatever a shape draws into.
- FR-024: `Buffer`, `stamp`, `Cell` and the renderer are unchanged in what they do. Whatever a shape
  draws into is additive to `Buffer`, not a change to it.
- A shape has no opinion about another shape (_Shapes_), which extends to the stamp mode: two shapes
  drawn into the same buffer must compose the same way regardless of which one happens to name a
  mode, so nothing below the caller should be able to name one at all.

## Considered Options

- **Pass `&mut Buffer` and a `StampMode` to every `draw` call.** The obvious option, and the one
  refused before this record.
- **A `Surface` trait with one write operation and no reader, plus a `Layer` adapter binding a
  buffer to a mode.** `Surface::stamp(&mut self, at: Pos, cell: Cell)`; `Layer::new(buffer, mode)`
  is the one `impl Surface` the crate ships.
- **`impl Surface` directly on `Buffer`, with the mode fixed to `Above`.** Removes `Layer`, at the
  cost of the caller's choice between the two stamp modes.

## Decision Outcome

Chosen option: **a `Surface` trait with one write operation and no reader, and a `Layer` adapter**,
because it is the only option that makes FR-015 a fact the compiler enforces rather than a rule a
reviewer checks, while still leaving the caller's choice of stamp mode in place.

```rust
pub trait Surface {
    fn stamp(&mut self, at: Pos, cell: Cell);
}

pub struct Layer<'a> { /* &'a mut Buffer, StampMode */ }
impl<'a> Layer<'a> {
    pub fn new(buffer: &'a mut Buffer, mode: StampMode) -> Self;
}
impl Surface for Layer<'_> { /* forwards to Buffer::stamp with the bound mode */ }
```

`Shape::draw` takes `&mut dyn Surface`, never `&mut Buffer`. `Layer::new(&mut buffer, mode)` binds
the mode once, at construction, so no shape or fragment ever names `StampMode` — the caller's choice
is taken exactly once, before any drawing happens, and every piece a figure places draws under it
without being told.

### Consequences

- Good, because `Surface` exposes no `cell` or equivalent read, so "a fragment never inspects the
  buffer" holds structurally: there is nothing to inspect. A future fragment that tried would fail
  to compile rather than fail review.
- Good, because the write counter FR-020 needs is a second `impl Surface` in a test module, not a
  change to `Buffer`. FR-024 stays true of an untouched file rather than a judgment call about
  whether a new constructor counts as a change.
- Good, because no shape or fragment ever names `StampMode`. A shape that took one would be deciding
  how it composes with figures it has no opinion about, which _Shapes_ forbids in as many words.
- Bad, because `Surface::stamp` takes a `Cell`, so a shape defined outside this crate builds its
  cells by hand and passes through none of the six cell rules
  [ADR-0028](0028-give-each-fragment-its-own-cell-rule.md) assigns. The vocabulary's guarantee is
  internal to this crate; widening `Surface` to expose more is additive, not a reason to treat the
  hole as closed today.
- Neutral, because `Layer` is one more public type with one constructor and one trait impl — the
  entire cost of the adapter.

### Confirmation

Structural rather than a gate check: `Surface` has no method that reads a cell, so a fragment that
tried to inspect the buffer would not compile. FR-020's write counter is the test-only
`impl Surface` that data-model.md's shape layer adds, and it is made to fail on purpose and
restored, per SC-006.

## Pros and Cons of the Options

### Pass `&mut Buffer` and a `StampMode` to every `draw` call

- Good, because it needs no new type at all.
- Bad, because FR-015 becomes a convention every fragment has to be reviewed against, forever,
  rather than a fact the compiler checks once.
- Bad, because every shape and fragment would name `StampMode`, which is an opinion about
  composition _Shapes_ says a shape does not have.
- Refused by the maintainer before this plan.

### A `Surface` trait and a `Layer` adapter

- Good, because it closes the FR-015 hole structurally and keeps the mode out of every shape.
- Good, because the write counter is a second, test-only implementation rather than a change to
  `Buffer`.
- Bad, because it is a new public trait and a new public type, for what a direct `&mut Buffer`
  argument would have done in fewer lines.

### `impl Surface` directly on `Buffer`, mode fixed to `Above`

- Good, because it removes `Layer` entirely.
- Bad, because it deletes the caller's choice between the two stamp modes, which user story 2's
  fourth acceptance scenario in feature 039's spec asserts under both.

## Reversibility

Cheap today: `Surface` and `Layer` are new types with nothing else built against them yet. The cost
grows with the number of shapes and fragments written against `Surface::stamp(at, cell)` — three
figures and six fragments by the end of feature 039 — and grows again, discontinuously, if `Surface`
is ever widened to expose a reader, since that would need every existing fragment's guarantee
re-examined rather than merely added to.

## Confidence

High (85%).

What would change it: a shape outside this crate that needs the same cell-rule guarantees the six
fragments get, which would require widening `Surface` past one write operation. The observation that
would prove this wrong is a caller building malformed `Cell`s by hand often enough that the internal
guarantee stops being worth the split — nothing in feature 039 does that, since every cell this
feature writes goes through one of the six fragments.

## More Information

- research.md, Q1 and Q3, in `specs/039-draw-shapes-instead-of-individual-cells/`: the trait's shape
  and the dispatch choice this record carries.
- [ADR-0028](0028-give-each-fragment-its-own-cell-rule.md), the vocabulary that writes through
  `Surface::stamp` and the hole this decision's "Bad" consequence repeats.
- [ADR-0030](0030-drop-extent-until-a-caller-needs-it.md), which leaves `Shape` with `draw` and
  nothing else — the trait this record's `Surface` sits opposite.
- _Shapes_ in [`docs/model.md`](../model.md), amended alongside this record so it says a shape draws
  into a surface rather than a buffer.
