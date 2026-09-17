---
status: "accepted"
date: 2026-09-17
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Let a route cross no cell twice

## Context and Problem Statement

[ADR-0046](0046-rank-a-route-instead-of-bounding-it.md) stops bounding a route and ranks it instead,
which reaches arrangements the old bound never had to answer for. One of them asks a question the
model has never been asked: may a route come back to a cell it has already used?

It arises where both endpoints sit at one position and leave in the same direction. The two starting
positions are then the same cell, so a path that leaves it must return to it, and the route is a
loop. The ranking yields one — four bends, six cells — and the maintainer, shown it, judged it
acceptable:

```text
▲
├┐
└┘
```

That picture costs more than it looks. The cell the loop begins and ends at carries three arms, and
no fragment in the core opens on three sides: `Corner` takes exactly two, `Segment` runs along one
axis, `End` carries one. `Route` would place a segment and a corner on that one cell, writing it
twice — against _Properties worth testing_ in [the model](../model.md), "a shape writes no position
more than once in one drawing", which the sweep asserts on all 1856 renderings through
`CountingSurface`.

So the choice is not whether the loop is a nice picture. It is whether a route is a path or a figure
that may cross itself, and the fragment vocabulary is what makes that expensive.

## Decision Drivers

- The model's own property. A shape writing one position twice is tested, not merely stated, and an
  exemption for one family of four arrangements would be exactly the special case
  [issue 103](https://github.com/andresmoschini/monospace/issues/103) asks not to add.
- [Principle IV](../../.specify/memory/constitution.md#iv-claims-are-measured-not-assumed). Writing
  the loop into the model while the code cannot draw it would put a claim in a durable document that
  nothing backs.
- Cost, measured. Drawing the loop needs a fragment that opens on three sides and a `Route` that
  merges the arms of a cell it reaches twice. Neither is large, and neither moves a single one of
  the 1856 renderings — the loop is the only route in the whole space that revisits a cell.
- [Principle I](../../.specify/memory/constitution.md#i-process-over-product-non-negotiable). A
  fourth fragment is a design decision of its own, and taking it inside a slice about routing is
  taking it where nobody would look for it.

## Considered Options

- **A** — A route is a path: it visits no cell twice. Where that leaves no path, the route is empty,
  by the rule that already says so.
- **B** — A route may cross itself. Add a fragment that opens on three sides, and let `Route` merge
  the arms of a cell it reaches more than once.
- **C** — A route may cross itself, and the cell it revisits is written twice. Exempt the arrow from
  the write-once property.

## Decision Outcome

Chosen option: **A**, because a route that never revisits a cell is one more sentence in a rule that
is already only sentences, whereas drawing a loop is a new fragment, a new decomposition and a new
kind of figure — and none of that belongs to a slice about where a route goes.

Four arrangements are affected, and all four are the same one: both endpoints at one position
leaving in the same direction. They draw two heads on one cell — the `to` endpoint's, per ADR-0044 —
and no route, exactly as they do today. Nothing regresses; what changes is that the model now says
why, instead of saying it happens "where the route rectangle is one cell thick".

### Consequences

- Good, because the write-once property stays true everywhere with no exemption, and the sweep goes
  on testing it as a property rather than as a property-with-a-hole.
- Good, because the model keeps describing a route as a path, which is what every other sentence
  about it already assumes: runs, bends, a first step and a last one.
- Good, because the decision is visible. "Both endpoints at one position leaving the same direction
  draw no route" now has a reason a reader can disagree with, rather than being a leftover of a
  bound that no longer exists.
- Bad, because a picture the maintainer judged acceptable is not drawn. It is the smallest family in
  the space — four arrangements, none of them in the sweep grid, which excludes coincident positions
  — but it is a capability declined rather than deferred by accident.
- Bad, because the reason is partly the fragment vocabulary rather than the domain. "A route does
  not cross itself" reads like design; "no fragment opens on three sides" is why it was affordable
  to say so, and the two are not the same argument. This record is where that is admitted.
- Neutral, because nothing else in the space revisits a cell. Measured over the sweep and over 3000
  random arrangements: the loop is the only one, so the constraint costs nothing anywhere else.

### Confirmation

A test renders both endpoints at one position leaving the same direction and asserts the two heads
and no route. The sweep's `CountingSurface` assertion is the other half: it fails the moment a route
writes a position twice, so the constraint is enforced by a test rather than by review.

## Pros and Cons of the Options

### A — A route visits no cell twice

- Good, for the reasons in the outcome.
- Bad, because it declines a picture rather than deferring it, and the arrow-from-a-point-to-itself
  stays a shrug.

### B — Add a fragment that opens on three sides

- Good, because it draws the loop, and a three-sided fragment is a piece the model will want anyway:
  _Open questions_ already asks "what comes after the first three shapes?" and lists a rounded
  corner and a double line as the same kind of gap.
- Good, because it is honest about what a route is — a figure, not a walk — and the arms of a
  revisited cell compose exactly as two lines meeting already do.
- Bad, because it is a fragment, a `Route` that merges arms, and their tests, inside a slice about
  where a route goes. [ADR-0028](0028-give-each-fragment-its-own-cell-rule.md) gives each fragment
  its own cell rule, so this is a decision with its own record, not a paragraph in someone else's.
- Bad, because it buys four arrangements. The ratio is what rejects it now and what would accept it
  the moment a second caller needs the same fragment.

### C — Exempt the arrow from the write-once property

- Good, because it is the smallest change: the loop is already what the ranking yields.
- Bad, because the property is what proves the decomposition correct. An exemption turns a test of
  the design into a test of one shape, and the next shape that overlaps itself has a precedent
  instead of a decision.
- Bad, because the picture would depend on the stamp mode. Under `Above` the last piece written
  wins, so the cell would show a corner and lose the third arm — which is not the loop, just a
  smaller wrong figure.

## Reversibility

Cheap and additive. Reversing it is the fragment of option B, the arm merge in `Route`, and dropping
one clause from the model; no public type and no other picture depends on it. Nothing has to be
undone first, because the four arrangements draw nothing today and would simply start drawing
something.

What makes the cost grow is prose written elsewhere on the assumption that a route is a walk. There
is one such sentence, in the model, and it is the one this record puts there.

## Confidence

Medium (65%).

What would change it: a second caller for a three-sided fragment. A connector with a branch, a line
that joins another mid-run, or anything the "what comes after the first three shapes?" question
answers — any of them pays for the fragment on its own, and then the loop is free.

What would prove it wrong: a real diagram that wants an arrow from a thing back to itself. That is
an ordinary thing to draw — a state machine's self-transition is exactly it — and the reason this
confidence is not higher. The counter-argument is that such an arrow wants a loop with a shape the
caller controls, not the smallest one the ranking happens to yield, so it is a capability of its own
either way.

What would not change it: the four arrangements being degenerate. _Degenerate arrangements_ in the
model is explicit that what a degenerate arrangement draws is whatever the general rule yields, and
this record changes the general rule rather than carving out an exception for them.

## More Information

- [ADR-0046](0046-rank-a-route-instead-of-bounding-it.md) — the ranking that raised the question.
- [ADR-0028](0028-give-each-fragment-its-own-cell-rule.md) — why a new fragment is a decision.
- [ADR-0029](0029-draw-a-line-end-as-one-arm.md) — the same trade made once already, where the
  fragment vocabulary decided what a figure could be.
- [The model](../model.md), _Properties worth testing_ — "a shape writes no position more than once
  in one drawing", the property this keeps whole.
