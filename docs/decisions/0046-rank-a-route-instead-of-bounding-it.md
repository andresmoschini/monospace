---
status: "accepted"
date: 2026-09-17
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Rank a route instead of bounding it

## Context and Problem Statement

_The route of an arrow_ in [the model](../model.md) bounds a route: it "stays inside the **route
rectangle**, the smallest rectangle containing both" starting positions, and within that bound takes
the fewest bends. Where no alternating path fits inside the rectangle the route is empty and the
arrow is its two heads — stated, deliberately, as the rule's own answer rather than an exception to
it.

[Issue 103](https://github.com/andresmoschini/monospace/issues/103) reports what that costs. Two
endpoints facing away from each other and aligned on the axis they face along draw two heads with
nothing between them, however far apart they are, because the rectangle is one cell thick and no
path can alternate inside it. Measured over the reviewed sweep of
[ADR-0045](0045-pin-every-arrow-arrangement-as-a-reviewed-snapshot.md): **340 of the 1856 renderings
— 170 of the 928 arrangements — draw no route at all**, and they are not one family but five, from
two endpoints in line facing away to two whose starting positions are not even collinear.

The bound is also what forced the **double escape** into the rule. Feature 099's `research.md` Q2
found 17 arrangements whose route needs seven runs rather than five, reached "only when the ordinary
escape degenerates", and gave them a shape of their own — because the escape run it needed had no
room _inside the rectangle_. So the rectangle produces both defects at once: a family that draws
nothing, and a family that draws something no other family draws.

## Decision Drivers

- The issue's own requirement, which is a shape for the rule rather than a picture: "one simple rule
  rather than a pile of special cases for each geometry".
- [Principle I](../../.specify/memory/constitution.md#i-process-over-product-non-negotiable). The
  route rectangle is a construct the model has to explain — a paragraph on how it exceeds the
  endpoint rectangle "by exactly one cell on each side a direction points away from" — and a
  construct that earns its paragraph by causing two defects is a design to revisit, not to patch.
- [Principle IV](../../.specify/memory/constitution.md#iv-claims-are-measured-not-assumed). Whatever
  is chosen here moves pictures that are already pinned, and the count has to be measured before it
  is chosen, not after.
- The tie-break the model already has must survive. ADR-0044's rounding toward the `from` endpoint
  decides 138 renderings; anything that makes it unreachable is not a candidate.

## Considered Options

- **A** — Keep the route rectangle as the bound and widen it by one cell on every side.
- **B** — Drop the bound. Rank candidates by the fewest bends, then the shortest, and let the
  rectangle survive only as the span the middle is taken from.
- **C** — Keep the rectangle exactly as it is and add a rule that wraps around outside it, applying
  only where no path fits inside.

A second question rides along with all three, because the ranking answers it for the first time: a
path from a position back to itself. Two endpoints at one position leaving in the same direction
have one starting position, so the route would have to arrive where it began — a loop, which writes
one cell with three arms. Whether a route may cross itself is decided in
[ADR-0047](0047-let-a-route-cross-no-cell-twice.md) rather than here.

## Decision Outcome

Chosen option: **B**, because the shortest route is already the one the bound was trying to name,
and saying so directly costs the model a construct instead of earning it a second one.

The rule becomes: among the paths the two leaving directions admit, the fewest bends, then the
shortest, then the free runs nearest the middle rounding toward the `from` endpoint (ADR-0044), then
the side the arrow's own travel puts to its right
([ADR-0048](0048-let-the-travel-pick-the-side-of-a-mirrored-route.md)). _The route of an arrow_ is
amended in the same increment.

The route rectangle stops bounding anything. It is a consequence instead: a path that leaves the
rectangle the two starting positions span is longer than one that stays inside it, so it wins only
where no path inside it exists at all — which is exactly the family the issue reports. Measured, no
chosen route ever reaches **more than one cell** outside that rectangle, over the sweep and over
4000 random arrangements up to fifty cells apart.

### Consequences

- Good, because the five families that drew nothing become one rule and no special case. **268
  renderings stop being two disconnected heads** — 134 arrangements taken from both ends — and what
  still draws no route is 36 arrangements rather than 170.
- Good, because the double escape is deleted rather than kept. Its 17 arrangements — 34 renderings —
  take four bends around the outside where they took six inside, so the shape that built them, and
  the paragraph of `research.md` that justified it, describe a route the rule can no longer choose.
  The run-count ceiling drops from seven to five.
- Good, because what is left has one shape and one sentence behind it: an endpoint standing on the
  cell the route would have to arrive at. In all 36 the two heads are orthogonally adjacent, so
  nothing looks disconnected.
- Bad, because 34 renderings that drew a compact six-bend route now draw a wider four-bend one.
  Fewest bends is the rule's first term and always was; what changes is that the wider picture is
  now reachable. A reader who liked the compact one has no argument left except the bound this
  record removes.
- Bad, because "the shortest" is a new term to state, to test and to keep true. It does nothing on
  any monotone path — every such path is the same length — so it is invisible in most of the sweep
  and easy to break without noticing.
- Neutral, because the sentence "where no such path exists the route is empty" does not change. Only
  the set it applies to shrinks.
- Neutral, because the route rectangle keeps a job: the middle of its span on each axis is still
  what ADR-0044's tie-break rounds within. It stops being a bound, not a definition.

### Confirmation

Measured on the branch `103-spike-route-rule`, against `main` at `727bd3b`, with the reviewed sweep
regenerated: **290 of the 1856 renderings change and 1566 do not**, and every one of the eleven
pictures the named scenario tests pin is among the 1566, including ADR-0044's own pair from both
ends. A further 16 change for the defect of
[issue 104](https://github.com/andresmoschini/monospace/issues/104), which is not this decision.

Twelve of the newly routed renderings gain a route that falls outside the sweep window and is
clipped, so the sweep understates the change by that much: 268 renderings gain a route, 256 of them
visibly.

Nothing enforces this mechanically. It is prose in the model, the snapshot under it, and the named
tests beside it.

## Pros and Cons of the Options

### A — Widen the route rectangle by one cell on every side

- Good, because it is the smallest edit to the model: one clause, and the rest of the paragraph
  stands.
- Good, because the middle does not move. Growing a span by one cell at each end leaves its halfway
  cell where it was, so ADR-0044 is untouched.
- Good, because it draws **the same pictures as B** — measured, by searching with a one-cell margin
  and with a four-cell one and comparing all 1856 renderings plus 4000 random arrangements: no
  difference anywhere.
- Bad, because the bound then decides nothing. Inside the widened band several routes share the
  fewest bends at different lengths, so "the shortest" has to be added anyway — and once it is
  there, the rectangle is a second rule that never disagrees with the first.
- Bad, because the widening has no reason a reader can check. "One cell" is the number that happens
  to work; nothing in the model says why not two.

### B — Rank by the fewest bends, then the shortest

- Good, for the reasons in the outcome.
- Bad, because it is the largest change to the model's prose of the three, and to the
  implementation: the derivation stops being a list of shapes
  ([ADR-0049](0049-derive-a-route-by-searching-the-lines-a-turn-can-sit-on.md)).

### C — Keep the rectangle and add a wrap-around rule beside it

- Good, because it moves the fewest pictures: 268 renderings gain a route and nothing that already
  drew one changes, so the double escape and its 34 renderings survive untouched.
- Good, because each rule can be read against a drawing, which is how the shapes are written today.
- Bad, because it is the pile of special cases the issue asks not to have. The model would carry the
  rectangle, the middle, the double escape and the wrap-around, and "why does this one go around and
  that one not?" has no short answer.
- Bad, because completeness has to be argued rather than derived, and it has already failed twice
  under that burden: the double escape was missed by feature 099's hand-checked table and found by a
  brute-force sweep, and issue 104 was missed by both.

## Reversibility

Cheap to reverse into A, which is one clause and no picture changes at all. Reversing into C is
expensive: the double escape and its justification would have to be reconstructed from this record
and from 099's `research.md`, and the snapshot diff is the whole grid again.

What is not cheap, in any direction, is changing it repeatedly. Every reversal regenerates eight
reviewed snapshot files and asks for the whole sweep to be read again — which is the cost ADR-0045
accepted knowingly, and it is the same cost each time.

## Confidence

High (85%).

What would change it: diagrams where the wide four-bend detour reads worse than the compact double
escape it replaces. The 34 renderings are all one narrow family, and none of them appears in a real
diagram yet.

What would prove it wrong: an arrangement where the fewest-bend-then-shortest route is one a person
would not draw, and where the route rectangle would have prevented it. The sweep is 1856 renderings
and was read once; a second reading is what would find it.

What would not change it: the implementation cost. It was measured, and the derivation gets smaller
rather than larger.

## More Information

- [The model](../model.md), _The route of an arrow_ — the rule this amends.
- [Issue 103](https://github.com/andresmoschini/monospace/issues/103) — the report, and the sketches
  that turned out to be what the ranking already yields.
- [ADR-0044](0044-let-the-endpoint-order-break-a-tied-route.md) — the tie-break this keeps, and
  whose last consequence said that widening what bounds a route "remains a slice of its own". This
  is it.
- [ADR-0045](0045-pin-every-arrow-arrangement-as-a-reviewed-snapshot.md) — the sweep every number
  here is measured over.
- [Issue 104](https://github.com/andresmoschini/monospace/issues/104) — the defect separated out of
  this change so the two are reviewable apart.
- Feature 099's [`research.md`](../../specs/099-bug-with-arrows/research.md), Q2 — where the double
  escape was found, and the sentence this record retires: "widening the search does not risk
  widening the _bound_".
