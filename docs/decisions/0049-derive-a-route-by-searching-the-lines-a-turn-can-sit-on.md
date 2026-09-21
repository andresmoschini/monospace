---
status: "absorbed into crates/monospace-core/src/shape/arrow.rs"
date: 2026-09-17
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Derive a route by searching the lines a turn can sit on

## Context and Problem Statement

Feature 099 rewrote `derive_path` from a search to a direct construction, and its `research.md` Q2
gave the reason: "the rule it implements is a sentence long — fewest bends, and every turn the bends
leave free sits at the middle of the route rectangle — and a search with a scoring function cannot
be read against that sentence."

[ADR-0046](0046-rank-a-route-instead-of-bounding-it.md) changes what that sentence is. The rule is
now a ranking of four terms in order — fewest bends, then shortest, then nearest the middle, then to
the right of the arrow's travel — which is a scoring function written in prose. The argument that
sent the implementation to a construction has been inverted by the rule it was implementing.

The construction has also now been wrong twice, both times by omission rather than by a mistake in
any one shape. Feature 099's own hand-checked table missed the arrangements that needed a seventh
run, and a brute-force sweep found them;
[issue 104](https://github.com/andresmoschini/monospace/issues/104) was missed by the table and by
the sweep both, because the shape that should have won was silently discarded for naming one point
twice. A list of shapes has to be argued complete, and that argument has failed each time it was
made.

## Decision Drivers

- The rule is a ranking, so the implementation that reads against it is the one that minimizes it.
  This is the same driver as 099's, pointing the other way because the rule changed.
- Completeness must be derived rather than argued. Both defects above are cases where every
  individual shape was right and the set was short.
- Cost must not grow with the distance between the endpoints. `Arrow` is a core shape and the core
  compiles to WebAssembly; a derivation that walks a 1000-by-1000 field because two endpoints are
  far apart is not one this crate can ship.
- [Principle VII](../../.specify/memory/constitution.md#vii-the-core-stays-portable) and the
  standard library. A search needs a priority queue and a map from state to cost; `BinaryHeap` and a
  flat `Vec` are both in `std`, and no dependency is asked for.

## Considered Options

- **A** — Keep the construction and add the shapes the new rule needs: the two wrap-around shapes
  and whatever else the sweep turns up.
- **B** — Search every cell of a bounded field, with the ranking as the cost.
- **C** — Search only the lines a turn can sit on: per axis, the line each starting position pins,
  the line beside each of those, the middle, and one line outside the rectangle the two starting
  positions span. Nine coordinates per axis at most.

## Decision Outcome

Chosen option: **C**, because it is the only one whose cost is fixed while the space it searches is
complete.

The ranking is carried as one value — bends, length, distance from the middle, hand — compared
lexicographically, and every term is charged at the turn that starts a run, so a single Dijkstra
over the states `(node, the heading it was reached on)` yields the route the model names with no
second pass over candidates. The lattice is at most 9 by 9, so 324 states, whatever the arrangement.

### Consequences

- Good, because the code and the rule are the same sentence. `Cost` has one field per term of the
  model's tie-break, in the model's order, and the derivation is "the cheapest path" rather than
  "the best of these seven shapes".
- Good, because completeness stops being an argument. Nothing has to be shown to be the only shape
  that fits; the search either finds a path or there is none.
- Good, because `zigzag`, `three_waypoint`, `path_bends` and `opposite_orientation` are deleted. The
  double escape does not survive as code any more than it survives as a rule.
- Good, because the cost does not grow with distance. Measured: the same result for two endpoints
  one cell apart and fifty, in the same 324 states.
- Bad, because a lattice has to be justified where a construction did not. "A route turns only on
  these lines" is a claim about the rule, and this record's confirmation is a comparison against a
  search that assumes nothing, not a proof.
- Bad, because a Dijkstra is more machinery than a list of shapes: a priority queue, a state
  encoding, a predecessor walk. It is larger to read at once, even though each part is ordinary.
- Neutral, because the derivation stays private to `crates/monospace-core/src/shape/arrow.rs` for
  now. [Issue 105](https://github.com/andresmoschini/monospace/issues/105) moves the whole
  connection into a shape of its own, and this record takes no position on where it lands — only
  that whatever holds it holds a ranking and a search, not a catalogue of shapes.

### Confirmation

Two comparisons, both over the reviewed sweep and over random arrangements, run before the code was
written and again after:

- The lattice search against a search over every cell of a generously bounded field, which assumes
  nothing about where a route turns: **identical on 5872 arrangements** — the 1856 sweep renderings,
  4000 random ones up to fifty cells apart, and the 16 where both endpoints coincide. The only
  divergence is the loop of [ADR-0047](0047-let-a-route-cross-no-cell-twice.md), which the lattice
  does not reach and which that record rules out anyway.
- The single-pass ranking against a two-pass reference that enumerates every fewest-bend-then-
  shortest path and then scores it: identical everywhere but the same loop.

The sweep snapshot is what holds this afterwards. A shape the search stops finding is eight files of
diff.

## Pros and Cons of the Options

### A — Keep the construction and add shapes

- Good, because each shape can be read against a drawing, and a reviewer can check one without
  holding the rest in mind.
- Good, because it is the smallest diff: the existing scoring stays and two or three `Vec<Pos>`
  builders are added.
- Bad, because the set has to be proven complete and has twice not been. The failure mode is silent
  — a missing shape does not error, it draws the second-best picture, or nothing.
- Bad, because the list only grows. The wrap-around has two mirror forms, and every future term in
  the rule multiplies the shapes rather than adding a field.

### B — Search every cell of a bounded field

- Good, because it assumes nothing at all about where a route turns, which makes it the right
  reference implementation — and it is the one this decision is confirmed against.
- Good, because it is the least code: a grid, a queue, and the cost.
- Bad, because its cost is the area of the field. Two endpoints a thousand cells apart is a million
  cells times four headings, for a route with four bends in it.
- Bad, because the field needs a bound anyway, and the bound is the same claim the lattice makes,
  stated less precisely.

### C — Search the lines a turn can sit on

- Good, for the reasons in the outcome.
- Bad, because the lattice is nine coordinates per axis chosen for reasons — a pinned line, the cell
  beside a blocked one, the middle, one line outside — and getting that list short by one is a
  silent defect of exactly the kind option A suffers from. It was found once during this work: a
  first lattice omitted the lines beside the starting positions, and 26 arrangements routed worse
  than the reference until they were added.

## Reversibility

Cheap in one direction and not the other. Falling back to option B is deleting the lattice and
walking cells, which is fewer lines and the same pictures; the cost is performance nobody has
measured a need for yet. Returning to option A means reconstructing a set of shapes for a rule that
now has four terms, which is more work than it was when the rule had two.

Nothing public depends on this. `derive_path` is private, `Route` is `pub(crate)`, and the only
observable thing is the picture — which is ADR-0046's business, not this one's.

## Confidence

High (80%).

What would change it: a profile showing the search costs anything that matters. 324 states per arrow
is small, but it is allocation and a heap where the construction was a handful of `Vec`s, and no
diagram large enough to notice has been drawn yet.

What would prove it wrong: an arrangement where the lattice's nine lines per axis are not enough.
The comparison against the unrestricted search is what would catch it, and it is worth re-running
against any new term added to the ranking — a term the lattice does not know about is exactly how
this fails.

What would not change it: the readability objection. A Dijkstra is more machinery than a list of
shapes and that was weighed; what settles it is that the machinery is generic and the list was not.

## Revisions

- 2026-09-21 — absorbed into `crates/monospace-core/src/shape/arrow.rs`. Nothing outside that module
  observes how the route is found beyond the picture it produces, so the reasoning is now the
  module's `Design notes` and changing it is an ordinary `feat` or `fix`. The contract the route
  satisfies is [ADR-0055](0055-an-arrows-route-is-a-path-and-nothing-bounds-it.md).

## More Information

- [ADR-0046](0046-rank-a-route-instead-of-bounding-it.md) — the rule this implements, and why it is
  a ranking.
- [ADR-0047](0047-let-a-route-cross-no-cell-twice.md) — the one place the lattice and the
  unrestricted search disagree, and why that is the answer rather than a gap.
- [ADR-0048](0048-let-the-travel-pick-the-side-of-a-mirrored-route.md) — the last term of the cost.
- Feature 099's [`research.md`](../../specs/099-bug-with-arrows/research.md), Q2 — the decision this
  reverses, and the reason it was right when it was taken.
- [Issue 104](https://github.com/andresmoschini/monospace/issues/104) — the second time a
  constructed set of shapes was short.
- [Issue 105](https://github.com/andresmoschini/monospace/issues/105) — where the derivation is
  expected to move next.
