---
status: "accepted"
date: 2026-09-14
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Let each shape answer its own anchor points, and allow it to answer none

## Context and Problem Statement

Issue #62 asks that a shape be able to sit relative to another one: pinned to its right side, to its
center, to a corner. That needs somewhere to pin to, and the issue lists nine candidates — the
center, the four side centers and the four corners — while adding that a shape does not necessarily
provide all of them, and that a position depending on one it does not provide cannot be resolved.

Nine points around a figure is a rectangle described the long way, and that is the tension.
[ADR-0030](0030-drop-extent-until-a-caller-needs-it.md) removed `extent` from the shape abstraction
and named, in its _Confidence_ section, exactly what would bring a bounding rectangle back: "a
caller inside the core that needs to know what another shape occupies — routing around an obstacle,
or sizing a canvas to its content. That is the first slice of the layer above shapes". This is that
slice. So the question is not whether anchor points are wanted, it is whether the way to have them
is to make every shape measurable again.

## Decision Drivers

- ADR-0030's own test, which is the constitution's removal test: an answer has to be shown to be
  needed, or it is not added. It applies to a bounding rectangle as much as it applied to extent.
- The work can arrive one kind at a time. The maintainer's criterion, stated when this was decided:
  do the anchors that are easy now, leave the hard ones — a connector's — for later, and the
  permission not to answer is what makes that possible.
- A rectangle is a poor description of some figures. An arrow routed in an L has a bounding
  rectangle whose center is not on the arrow, and a center that is not on the figure is worse than
  no center at all: it resolves, and it puts a box somewhere nobody pointed.
- [Principle VII](../../.specify/memory/constitution.md#vii-the-core-stays-portable): whatever is
  added here is added to the diagram crate's entity, not to the core's shape, which keeps ADR-0030
  standing where it was taken.

## Considered Options

- **A** — Every shape reports a bounding rectangle, and the nine anchor points are derived from it
  by one rule.
- **B** — Each kind answers each anchor point itself, and may answer none.
- **C** — Derived from a rectangle by default, with a shape allowed to declare exceptions.

## Decision Outcome

Chosen option: **B**, because it is the only one where a kind that cannot say where its center is
simply does not say, instead of being given an answer by a rule that never looked at the figure.

An anchor point is a question a shape answers with a position or with nothing. Nothing is not a
failure: it is the honest answer for a figure that has no such point, and
[ADR-0041](0041-resolve-a-position-through-a-reference.md) says what happens to a position that
depended on it.

Two consequences of the choice are worth stating as rules rather than leaving implied:

**A line answers as a flat box.** A horizontal line has no thickness, so its three left anchors
coincide, its three right anchors coincide, and its top center, center and bottom center coincide:

| Anchors that coincide on a horizontal line | Where         |
| ------------------------------------------ | ------------- |
| top-left, left center, bottom-left         | its left end  |
| top center, center, bottom center          | its middle    |
| top-right, right center, bottom-right      | its right end |

A vertical line is the same seen sideways. This is what makes a line usable as something to pin to
without inventing a width it does not have.

**Arrows answer nothing, for now.** A route with bends has no honest answer to most of the nine, and
none of them is needed to draw one. The permission to answer nothing is what lets that wait for a
slice of its own instead of holding up the ones that are easy.

An anchor point is an absolute position, computed from the shape's own position at the moment it is
asked. A shape whose own position does not resolve offers no anchor points either, which is what
makes a chain of references collapse rather than resolve halfway.

### Consequences

- Good, because anchor points arrive one kind at a time, and a kind with a hard answer costs nothing
  until somebody needs it.
- Good, because `extent` stays dropped and ADR-0030 stays standing. Nothing in the core has to
  answer what it occupies, and the layer that needed measuring turned out to need nine named points
  rather than a rectangle.
- Good, because each kind's answer is testable as itself: a box's nine positions, a line's nine with
  their coincidences, and an arrow's nothing are three small tests rather than one derivation to
  trust.
- Bad, because there is no single rule, so every kind added from here answers the question again,
  and two kinds could disagree about what "center" means without anything noticing.
- Bad, because a shape that answers nothing is silent in the same way a shape that does not exist is
  silent. ADR-0041 accepts that, and it is the cost that record carries.
- Bad, because the diagram cannot size its own canvas. Nine points that may be absent are not a
  bound, so the caller supplies the window — see
  [ADR-0042](0042-draw-a-diagram-front-to-back-into-a-given-window.md), which takes that decision
  and inherits this reason for it.
- Neutral, because a bounding rectangle is not foreclosed. If a later caller needs one — an
  auto-sized canvas, routing around an obstacle — it is added then, for the kinds that can answer
  it, and this record is what it has to argue with.

### Confirmation

By test, per kind. Each kind's anchors are positions that can be asserted directly, and the
coincidences on a line are assertions of equality between two of them. A kind that answers nothing
is confirmed the same way: the question is asked and the answer is nothing.

## Pros and Cons of the Options

### A — One bounding rectangle, nine derived points

- Good, because it is one rule, written once, and no kind can get it wrong.
- Good, because it would give the diagram a way to size its own canvas for free.
- Bad, because it forces every kind to answer at once, including the ones whose honest answer is
  that they have no such point, which is the opposite of what the issue asked for.
- Bad, because it answers where a figure is not. The center of an L-shaped arrow's rectangle is off
  the arrow, and a position pinned there would resolve to empty space with nothing to indicate that
  the answer was fictional.

### B — Each kind answers its own

- Good, because absence is representable, so the work slices and the hard cases wait.
- Good, because each answer is about the figure rather than about a rectangle around it.
- Bad, because the rule is repeated per kind, and consistency between kinds is a matter of review.

### C — Derived, with declared exceptions

- Good, because it has the single rule and the escape hatch at once.
- Bad, because it needs the bounding rectangle anyway, which means every kind still has to be
  measurable — and measuring an arrow was the problem.
- Bad, because an exception has to be justified each time it is taken, which is more process than
  either of the other two for a question that arises per kind regardless.

## Reversibility

Additive in the direction that matters. Adding a bounding rectangle later, for the kinds that can
answer one, takes nothing away: the anchors keep being what a reference resolves through, and the
rectangle serves whatever new caller asked for it.

Removing anchors from a kind that answered them is the expensive direction, because diagrams built
against those answers stop resolving. That cost is real from the first slice that ships an anchor,
and it is why the arrows answer nothing for now rather than answering something provisional.

## Confidence

High (80%).

What would change it: several kinds turning out to have identical nine-point answers derived from a
rectangle they all keep computing. That is option A arriving by evidence rather than by assumption,
and the right response would be to derive the default and keep the permission to answer nothing.

What would prove it wrong: two kinds disagreeing about what an anchor means in a way that surprises
a caller — a center that is the middle of the figure in one kind and the middle of its bounds in
another. Nothing here prevents that, and only review would catch it.

## More Information

- [ADR-0030](0030-drop-extent-until-a-caller-needs-it.md) — the record this one answers, including
  the sentence naming this slice as what would reverse it.
- [ADR-0041](0041-resolve-a-position-through-a-reference.md) — what happens to a position that
  depends on an anchor a shape does not answer.
- [The diagram model](../diagram-model.md), _Anchor points_ — the prose these rules are written
  into.
