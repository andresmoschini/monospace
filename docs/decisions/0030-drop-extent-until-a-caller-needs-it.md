---
status: "accepted"
date: 2026-09-10
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Drop extent from the shape abstraction until a caller needs it

## Context and Problem Statement

_Shapes_ in [`docs/model.md`](../model.md) declared that every shape reports an **extent**: the set
of positions it may write, derivable from its description alone and without drawing. `Extent` had a
row in the model's _Vocabulary_ and a subsection of its own, and feature 039 turned it into two
functional requirements, an acceptance scenario and a line in the shape trait.

Reviewing that plan asked what reads it. Three things did, and all three are bookkeeping rather than
drawing:

1. **Comparing a filled box with an unfilled one.** The requirement was that the two have the same
   extent, which forced the unfilled box to place an interior piece that writes nothing purely so
   that the position range would still belong to it.
2. **Checking that a shape writes nothing outside its extent.** A property the tests assert, with
   the extent as the thing they assert against.
3. **Stating that a compositor partitions its extent among its pieces.** The model's way of saying
   that no position is written twice.

None of the three is a caller drawing a diagram. Each has a replacement that costs less: the first
scenario is deleted, because a filled and an unfilled box occupying the same rectangle is a fact
about the rectangle and not a question anyone asks the shape; the second is what the expected buffer
of each test already asserts, since a stray write shows up as an extra character in the picture; and
the third is the property "no position is written more than once in one drawing", which the model
already lists under _Properties worth testing_ and which stands on its own without an extent to
partition.

## Decision Drivers

- The maintainer's criterion, stated when this was reviewed: unless it has an effective use for
  drawing diagrams, it goes.
- The constitution's removal test, from _One definition of green_: an entry must be shown to break
  something when taken out, or it is removed. Feature 039 applies the same standard to guards in
  FR-018 — a guard no test reaches is deleted rather than documented.
- _The core stays portable_ keeps the public API to what is used, since every public item is a
  promise the WebAssembly boundary and every later phase inherit.

## Considered Options

- **Keep it**, as a set of positions on the shape trait.
- **Drop it**, and add something back when a caller asks.
- **Replace it with a bounding rectangle**, which is what a placement layer would actually want.

## Decision Outcome

Chosen option: **drop it**, because everything that read it was a test or a restatement of "no
position is written twice", and both survive without it.

A shape therefore has one operation — draw itself into a surface — and answers no question about
itself.

The bounding rectangle was rejected for now on the same test that removed the extent: nothing asks
for it either. It is the shape the eventual answer will most likely take, because a layer that
decides where a figure goes, or fits a diagram into a canvas of the right size, wants bounds rather
than a set of positions — and adding a method is additive, so waiting costs nothing.

### Consequences

- Good, because the shape trait has a single method, which removes the question of what a shape
  promises besides drawing, and with it most of the reason the complete/fragment distinction looked
  like it wanted to be a type.
- Good, because the unfilled box loses a piece that existed only to carry positions nobody writes. A
  fill piece now exists exactly when there is a fill, and the `Option` that carried "an interior
  that writes nothing" is gone with it.
- Good, because `Pos` does not need `Hash`, and the set type, its equality and its `FromIterator`
  never enter the public API.
- Bad, because the model loses a documented concept: `Extent` leaves its _Vocabulary_ row and its
  subsection, and _Complete and fragment_ is reworded, since its definition of a complete shape
  leaned on the extent being readable from outside.
- Bad, because "a shape writes nothing outside its extent" stops being a guarantee anyone can state
  and becomes whatever each test's expected buffer happens to cover. That is weaker, and it is the
  price of not carrying an abstraction for the sake of the sentence it enables.
- Bad, because the first layer that places figures automatically will have to add bounds, and will
  have to decide then what a bound means for a figure whose written positions are sparse — an arrow
  above all. Deferring that is deliberate; forgetting it would not be, which is why it is written
  here.

### Confirmation

Nothing in the gate. The confirmation is structural: there is no method, so a test that wanted an
extent would not compile. If one is ever added back, this record is the thing to supersede.

## Pros and Cons of the Options

### Keep it

- Good, because the model already described it, so nothing had to be amended.
- Good, because it makes two properties expressible as one-line assertions.
- Bad, because those two properties were its only readers, and one of them existed to justify a
  piece that writes nothing.
- Bad, because it is a public collection type, an equality relation and a derive on `Pos`, all in
  service of tests.

### Drop it

- Good, because the layer is smaller and every remaining item has a reader.
- Bad, because a documented model concept is withdrawn, and a reader of the model's history will ask
  why.

### Replace it with a bounding rectangle

- Good, because it is what the eventual caller wants, and it is cheap to compute and to compare.
- Bad, because no caller wants it yet, so it would be the same unread abstraction in a smaller box.

## Reversibility

Additive to restore: a method on the trait and an implementation per shape. Nothing that exists
would have to change to accommodate it. What makes the cost grow is only the number of shapes
implementing the trait by then, and each one's answer is a few lines it already knows.

## Confidence

High (85%).

What would change it: a caller inside the core that needs to know what another shape occupies —
routing around an obstacle, or sizing a canvas to its content. That is the first slice of the layer
above shapes, and it is not in this phase. The observation that would prove this wrong is a figure
whose decomposition cannot be tested without asking it what it covers; three figures were worked
through and none is.

## More Information

- _Pieces_ and _Complete and fragment_ in [`docs/model.md`](../model.md), amended alongside this
  record.
- [ADR-0028](0028-give-each-fragment-its-own-cell-rule.md), the other half of the same review: what
  the pieces are, now that they no longer report what they cover.
- [ADR-0010](0010-separate-position-and-size.md) is where a bounding rectangle would have to start,
  since this repository deliberately has no rectangle type.
- Feature 039's spec, `specs/039-draw-shapes-instead-of-individual-cells/spec.md`, which loses
  FR-010, FR-011 and one acceptance scenario to this record.
