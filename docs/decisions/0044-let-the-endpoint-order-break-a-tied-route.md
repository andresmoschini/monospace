---
status: "accepted"
date: 2026-09-16
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Let the endpoint order break a tied route

## Context and Problem Statement

An arrow is two endpoints and a stroke, and _The route of an arrow_ in [the model](../model.md)
derives its route "from the two endpoint positions and the two directions, and from nothing else".
Which endpoint is written first is therefore an accident of how the caller was thinking, and
[issue 99](https://github.com/andresmoschini/monospace/issues/99) reports that exchanging the two
changes the picture. Measured: `(0, 0)` leaving `right` with head `◄` and `(6, 0)` leaving `left`
with head `►` renders `◄─────►`, and the same arrow written the other way round renders `◄    ─►─`.

Repairing that is feature 099 and needs no record: the second picture is a defect against a rule
that already exists. Reviewing the specification surfaced the decision this record is about.

The rule chooses "the one that turns at the middle of the route rectangle on whichever coordinate
the bends leave free". Where that coordinate spans an even number of cells the middle falls between
two of them and the rule names no winner. Two routes are left, both with the fewest bends, both the
same length, and both correct. For `(0, 1)` leaving `down` and `(2, 6)` leaving `up`, whose free
coordinate spans rows 2 to 5:

```text
▲        ▲
│        │
└─┐      │
  │      └─┐
  │        │
  ▼        ▼
```

The maintainer's judgement is that both are right, which makes "the same picture from either end" a
reading of the bug report that is too literal. That relaxation is what exposes the gap: the model
presents the turn as determined, and where the span is even it is not. Two implementations
conforming to the model would draw different pictures.

There is a second place the order decides something. Where both endpoints occupy one position the
route is empty and the two heads land on one cell; only one of them can be seen, and nothing in the
model says which.

## Decision Drivers

- The model's own sentence. "From nothing else" is a promise the even span cannot keep, whichever
  way this is resolved, so the section changes either way.
- [Principle VI](../../.specify/memory/constitution.md#vi-decisions-recorded-when-taken): a route is
  about to be implemented against this, and someone reading a diagram will ask why exchanging two
  endpoints moved a corner. That question has no answer in the code.
- The tie-break has already been decided once, in
  [feature 039's research](../../specs/039-draw-shapes-instead-of-individual-cells/research.md), Q5
  — "fewest bends, then distance from the middle, then the lexicographic order of the turn
  sequence". The constitution is explicit that such a file carries no status and cannot serve as the
  history. Whatever is chosen here belongs in a record that can be superseded.
- The single-cell head collision has to be decided by something. If the order decides nothing else,
  that case stays an exception with no principle behind it — which is what made the draft
  specification need one.

## Considered Options

- **A** — The route turns at the smaller coordinate of the two. The order decides nothing.
- **B** — The route turns at the one nearer the endpoint the arrow leaves from. The order decides
  the tie, and the head collision follows the same principle.
- **C** — Leave it undecided in the model and let a test pin whatever ships.

## Decision Outcome

Chosen option: **B**, because an arrow is drawn running from its `from` to its `to`, and once that
is the principle both open questions close with one sentence instead of one sentence and one
exception.

_The route of an arrow_ is amended in the same increment: the derivation names the leaving endpoint
as an input where the middle falls between two cells, the tie-break names which of the two is taken,
and the head collision is stated where the empty route already is.

### Consequences

- Good, because the two things the order decides are now one idea rather than a rule and an
  exception. The head seen is the `to` one for the same reason the turn falls nearer the `from`: the
  arrow runs from one to the other.
- Good, because the picture stays predictable. A reader who knows which endpoint was written first
  can say where the corner is, which option C gives up.
- Good, because the model stops claiming a determinism it does not have. That claim was false before
  this record, not because of it.
- Bad, because the implementation has to be changed on purpose. Measured on `(0, 1)` leaving `down`
  and `(2, 6)` leaving `up`, what ships today turns at the smaller coordinate, so option A was
  reachable by repairing the defect alone and this is not.
- Bad, because "an arrow's picture does not depend on which endpoint is written first" stops being
  true as stated, and every document that says it has to say the longer thing instead.
- Neutral, because nothing here widens what bounds a route. The arrangements that draw no route
  still draw none, and that remains a slice of its own.

### Confirmation

Two renderings of one arrangement, taken from both ends, pinned as a test: the even-span pair above
is the case, and the two pictures it draws differ in exactly the position of the turn. A sweep over
the whole grid of arrangements ships as a reviewed snapshot alongside it.

Mechanically, nothing enforces this: it is prose in the model with a test under it.

## Pros and Cons of the Options

### A — The smaller coordinate wins

- Good, because the picture never depends on the order, so the rule is one sentence shorter and
  needs no record to explain a surprise.
- Good, because it is free: what ships already turns at the smaller coordinate in the arrangement
  measured, so repairing the reported defect would have landed it.
- Bad, because the head collision keeps no principle to follow and stays an exception.
- Bad, because it throws away a picture the maintainer judged correct, and picks between two equal
  routes on a ground — "smaller" — that means nothing to anyone drawing a diagram.

### B — Nearer the leaving endpoint wins

- Good, for the reasons in the outcome.
- Bad, because it costs an implementation change and this record.

### C — Undecided, pinned by a test

- Good, because it is the lightest: no amendment and no record.
- Bad, because the model would keep a sentence that is wrong, and the answer would live only in a
  snapshot. "Why here?" would be answered by "because that is what it does".
- Bad, because it was already tried. Q5 of feature 039's research is this option, and the result is
  a load-bearing tie-break in a document that cannot be superseded.

## Reversibility

Cheap. The tie-break is one comparison inside the route derivation and one sentence in the model; no
public type, no signature and no file layout depends on it. Reversing it is a new record, the
sentence rewritten, and a snapshot regenerated.

What is not cheap is changing it repeatedly. Every reversal moves corners in pictures people have
already drawn, and the snapshot diff is the whole grid each time.

## Confidence

High (80%).

What would change it: diagrams in which the turn moving with the order reads as noise rather than as
direction. The interactive phase is where that shows, because a person exchanging two endpoints
there sees the corner jump.

What would prove it wrong: a second place where the order has to decide something and the "travels
towards the `to`" principle gives the wrong answer. Two instances agreeing is what this rests on.

What would not change it: the implementation cost. It was weighed and it is small.

## More Information

- [The model](../model.md), _The route of an arrow_ — the rule this amends.
- [Feature 099](../../specs/099-bug-with-arrows/spec.md) — the specification this unblocks.
- [Feature 039's research](../../specs/039-draw-shapes-instead-of-individual-cells/research.md), Q5
  — where the tie-break was decided before, and why that was the wrong home for it.
- [ADR-0029](0029-draw-a-line-end-as-one-arm.md) — why a head is a chosen glyph, which is what makes
  two heads on one cell a question about which is drawn on top.
