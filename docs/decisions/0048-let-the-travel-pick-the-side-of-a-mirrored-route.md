---
status: "accepted"
date: 2026-09-17
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Let the arrow's own travel pick the side of a mirrored route

## Context and Problem Statement

[ADR-0044](0044-let-the-endpoint-order-break-a-tied-route.md) closed the one tie the rule had at the
time: where the middle of a span falls between two cells, the route turns at the one nearer the
endpoint the arrow leaves from. Its confidence section named what would prove it wrong — "a second
place where the order has to decide something and the 'travels towards the `to`' principle gives the
wrong answer" — and said that two instances agreeing is what it rests on.

[ADR-0046](0046-rank-a-route-instead-of-bounding-it.md) produces the second instance. Once a route
may leave the rectangle the two starting positions span, an arrangement whose two starting positions
share a row or a column has two routes that are exact mirror images about that line: one detours to
one side, one to the other, with the same bends, the same length, and the same distance from the
middle. Measured over the reviewed sweep, **1700 of the 1856 renderings are decided with no new rule
at all** — fewest bends, then shortest, then ADR-0044's middle — and **84 are not**. All 84 are this
one shape.

ADR-0044's principle cannot reach them. "Nearer the endpoint the arrow leaves from" needs the two
candidates to be at different distances from it, and mirror images about the line through both
starting positions are equidistant by construction. Something else has to decide, or the model
claims a determinism it does not have — the same defect ADR-0044 was written to remove.

## Decision Drivers

- The model must not name two pictures. That is the whole of ADR-0044's reasoning and it applies
  unchanged: two implementations conforming to the model would otherwise draw different diagrams.
- [Principle VI](../../.specify/memory/constitution.md#vi-decisions-recorded-at-the-altitude-they-belong-to).
  Someone will ask why a detour goes left here and right there; without a record the answer is
  "because that is what it does".
- The maintainer's stated preference, given the four candidates as generated pictures: the tie
  should depend on the order of the endpoints wherever it can, extending ADR-0044 rather than
  sitting beside it, and the simplest algorithm wins where that leaves a choice.
- Cost, measured. All four candidates are one comparison inside the ranking. None is simpler than
  another, so simplicity does not decide and taste does.

## Considered Options

- **A** — The smaller coordinate wins: the detour goes left, or up. Order-independent.
- **B** — The larger coordinate wins: right, or down. Order-independent.
- **C** — The detour passes to the **right** of the direction the arrow leaves its `from` endpoint
  in. Order-dependent.
- **D** — The same, to the left.

Coordinates grow rightward and downward, so under C leaving `Up` puts the detour at the larger `x`
and leaving `Right` at the larger `y`.

## Decision Outcome

Chosen option: **C**, because an arrow runs from its `from` to its `to`, and a rule phrased in terms
of that travel is the same idea ADR-0044 already chose rather than a second, unrelated one.

Two endpoints facing away from each other and aligned — issue 103's own example — then draw:

```text
(0, 0) leaving Up  ->  (0, 3) leaving Down      the same arrow, written the other way round

┌┐                                              ┌┐
▼│                                              │▼
 │                                              │
 │                                              │
▲│                                              │▲
└┘                                              └┘
```

_The route of an arrow_ is amended in the same increment: the tie-break gains its last term, and the
sentence naming ADR-0044's rounding as "the only thing the order of the endpoints decides about a
route" becomes the second of two.

### Consequences

- Good, because the rule is complete. With this term the ranking decides all 1856 renderings with no
  tie left over — measured, by generating every route that ties on the first three terms and
  checking that exactly one survives the fourth.
- Good, because ADR-0044's principle now has the two instances it said it rested on, and they agree.
  Both are "the arrow travels from its `from`", applied to a different kind of tie.
- Good, because the picture stays predictable in the way ADR-0044 made it predictable: a reader who
  knows which endpoint was written first can say which side the detour is on.
- Bad, because it widens what the order of the endpoints decides. ADR-0044 already accepted that
  cost for a corner moving by one cell; here the whole detour moves to the other side of the line,
  which is a louder change to the same arrow described from its other end.
- Bad, because "to the right of travel" is a handedness, and handedness in a coordinate system whose
  `y` grows downward is exactly the kind of thing a reader gets backwards. The model has to say
  which way, and the code has to say it in the same words.
- Neutral, because the choice between C and D is arbitrary and this record does not pretend
  otherwise. They are mirror images of one another; C is written down so that the model names one.

### Confirmation

Two tests, both on the arrangement above: one asserts the picture from each end, and together they
assert that the two are mirrors rather than equal. The sweep pins the other 82 renderings.

Mechanically, nothing enforces it. It is prose in the model with tests under it, the same as
ADR-0044.

## Pros and Cons of the Options

### A — The smaller coordinate wins

- Good, because the picture never depends on the order, so the model's older and simpler claim —
  that an arrow draws the same whichever endpoint is named first — is true again for this family.
- Good, because "smaller" needs no diagram to explain and cannot be got backwards.
- Bad, because it contradicts ADR-0044 in spirit while sitting next to it. One tie would be decided
  by the travel and the other by the coordinate system, and the model would have to say both.
- Bad, because "smaller" means nothing to anyone drawing a diagram, which is the same objection that
  rejected it in ADR-0044.

### B — The larger coordinate wins

- Good and bad exactly as A. It is A's mirror and was weighed only to confirm that neither
  order-independent option was preferred.

### C — To the right of the arrow's travel

- Good, for the reasons in the outcome.
- Bad, because it is the option that makes the two orders differ the most visibly.

### D — To the left of the arrow's travel

- Good, identically: it is order-dependent and extends the same principle.
- Bad, because nothing distinguishes it from C. It was on the table so that the arbitrariness of the
  choice is on the record rather than hidden in it.

## Reversibility

Cheap. It is one comparison in the ranking and one clause in the model, and reversing to D is a sign
change. Reversing to A or B additionally drops a sentence from the model and returns this family to
order-independence.

What is not cheap is the snapshot. Any of these reversals moves 84 renderings and regenerates all
eight reviewed files, so the cost is a reading of the sweep each time — and, as ADR-0044 already
said, moving corners in pictures people have drawn is what makes repeated changes expensive rather
than any one of them.

## Confidence

Medium (70%).

What would change it: seeing the two mirror pictures side by side in a real diagram. The interactive
phase is where a person exchanging two endpoints watches the whole detour jump to the other side,
and that is a louder effect than the corner ADR-0044 moves.

What would prove it wrong: a third place where the order has to decide something and "travels
towards the `to`" gives an answer the maintainer rejects. Two instances now agree; a third
disagreeing is what would turn the principle into a coincidence.

What would not change it: which of C and D was picked. They are mirrors, the choice was made by
looking at generated pictures of both, and no argument distinguishes them.

## More Information

- [ADR-0044](0044-let-the-endpoint-order-break-a-tied-route.md) — the first tie, the principle this
  extends, and the confirmation it asked for.
- [ADR-0046](0046-rank-a-route-instead-of-bounding-it.md) — the ranking that created this tie.
- [The model](../model.md), _The route of an arrow_ — the rule this amends.
- [Issue 103](https://github.com/andresmoschini/monospace/issues/103) — the arrangement in the
  outcome is its first example.
