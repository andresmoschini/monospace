---
status: accepted
scope: domain
commitment: working
date: 2026-09-21
decision-makers: Andrés Moschini
---

# An arrow's route is a path, and nothing bounds it

## Context and Problem Statement

Five records held one subject — the example
[ADR-0050](0050-record-a-decision-at-the-boundary-it-cannot-cross.md) was written from. Its altitude
test splits them: what a route _is_, anyone who draws one observes; _which_ of several gets drawn,
nobody observes beyond the picture.

The observable half had a defect. The model bounded a route by the rectangle its two starting
positions span, and a rectangle one cell thick holds no alternating path: 170 of 928 arrangements
drew no route at all, and 17 more needed a shape of their own, the double escape, because their
escape run had no room inside the bound. Dropping the bound then raised a question the model had
never been asked: may a route come back to a cell it has already used?

## Decision Drivers

- [Issue 103](https://github.com/andresmoschini/monospace/issues/103) asks for a shape for the rule,
  not a picture: "one simple rule rather than a pile of special cases for each geometry".
- _Properties worth testing_ already holds that a shape writes no position more than once. An
  exemption for one family turns a test of the decomposition into a test of one shape.

## Decision Outcome

Chosen option: **a route is a path — alternating runs, no position twice, through neither endpoint
position — and nothing bounds where it may go**, because the shortest path is what the bound was
trying to name, and saying so directly costs the model a construct instead of earning it a second
one.

Which path is drawn is `shape::arrow`'s business, in that module's `Design notes`, where ADR-0044,
ADR-0048 and ADR-0049 are absorbed. ADR-0046 and ADR-0047 are superseded here.

### Consequences

- Good, because 134 arrangements stop being two disconnected heads, and the 36 still drawing no
  route are one family — an endpoint on the cell the route would arrive at, its two heads adjacent.
- Good, because the write-once property stays whole, with no exemption for one family.
- Bad, because an arrow from a point back to itself is declined rather than deferred, and partly for
  a reason outside the domain: no fragment opens on three sides, which
  [ADR-0028](0028-give-each-fragment-its-own-cell-rule.md) owns. A diagram wanting a self-transition
  would reopen it — against the argument that such an arrow wants a loop the caller shapes.

## Reversibility

Cheap into a rectangle widened by one cell on every side — measured, identical pictures on all 1856
renderings and on 4000 random arrangements. Expensive back into a bound with a wrap-around beside
it: the double escape would have to be rebuilt from here and from 099's `research.md`.

## Revisions

- 2026-09-17 — recorded as ADR-0046, dropping the bound, and ADR-0047, making a route a path.
- 2026-09-21 — consolidated here, the choice among paths absorbed into `shape::arrow`.
