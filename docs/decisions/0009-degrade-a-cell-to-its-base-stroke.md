---
status: accepted
date: 2026-09-07
decision-makers: Andrés Moschini
---

# Degrade a cell to its base stroke when no character matches

## Context and Problem Statement

A cell is rendered by looking up its four arms in the loaded character tables. When every arm
carries the same stroke the lookup is a formality: the pure tables for `ascii`, `light`, `double`,
`heavy` and `light-round` each hold all 15 combinations. The problem is the mixed cell, where a
light line crosses a double border and the key names two strokes at once.

Unicode does not have those characters. Of the 50 possible combinations mixing `light` with
`double`, the table holds 18 — **32 mixed cells have no character at all**. Light with heavy is
complete at 50, but that is luck, not a rule, and it will not hold for whatever a user declares
next. Degrading is therefore not an edge case in this model; for two of the shipped strokes it is
the common case.

So the render needs a second answer for "no character matches", and it needs one that a person can
predict without reading the implementation, because it fires constantly.

## Decision Drivers

- Predictable beats faithful. Someone looking at a diagram has to be able to say why a character
  came out the way it did.
- Loading a character table should not change cells it does not describe. A user adding a table to
  improve one junction must not find other junctions redrawn.
- The rule has to survive attributes that are not strokes. Color is the obvious next one, and two
  colors cannot be blended in one character either — whatever answers this question will be asked
  again.
- Cheap to state, because every later spec that renders anything has to restate it.

## Considered Options

- **A** — Every connected arm takes the cell's base stroke, and the lookup is tried once more.
- **B** — Search for the smallest set of arms that, moved to the base stroke, yields a key that
  exists.
- **C** — Give each stroke a declared fallback stroke, and substitute along that chain before
  falling back to the base stroke.
- **D** — Score every table row against the wanted key and take the cheapest.

## Decision Outcome

Chosen option: **A, degrade the whole cell to its base stroke**, because it makes the answer to "why
that character?" a single sentence: the figure that owns the cell imposes its stroke on all of it.

The base stroke is written by the figure that last claimed the cell, so the rule reads as a rule
about figures rather than about tables: when a combination does not exist, the topmost figure wins
outright instead of winning a side. Rendering a cell is then exactly two lookups — the exact key,
then the uniform one — and if neither matches there is no character, which prints as a space.

That also removes the need for any convention about degrading to `ascii`. With the pure tables
complete, the second lookup only fails when the cell has no connected arm at all, or when the table
for its base stroke was never loaded.

### Consequences

- Good, because the rule is two lookups and one sentence, with no ordering, no search and no scoring
  to explain.
- Good, because adding a character table can only add exact matches. It can never silently change a
  cell that was already resolving.
- Good, because it generalizes to attributes that cannot be blended. Color, if it arrives, follows
  the same shape: one owner imposes, the rest yield.
- Bad, because detail is lost where a partial answer existed. Measured over the 32 unmatched
  light/double combinations, the smallest-subset search of option B would have kept at least one
  original stroke in 12 of them; in the other 20 the two rules agree. A mixed T becomes a pure T.
- Bad, because a stroke that is a variant of another pays for the whole table. `light-round` differs
  from `light` in four corners but needs all 15 of its own rows, and its two mixing tables — 68 rows
  copied from `light` — become optional only in the sense that dropping them means those mixes
  degrade.
- Neutral, because the escape hatch is the existing one. A user who needs a specific mixed junction
  loads a table containing it, the first lookup finds it, and degradation never runs.

### Confirmation

By test, once a spec implements the render:

- The exact key wins when it exists.
- A key with no exact match resolves to the uniform key built from the base stroke.
- A cell whose base stroke has no loaded table renders as a space rather than failing.

Until then, only by review.

## Pros and Cons of the Options

### A — Degrade every connected arm to the base stroke

- Good, because it is the only option whose result can be predicted without knowing which tables are
  loaded.
- Bad, because it discards strokes that a partial substitution could have kept.

### B — Smallest subset of arms

Try degrading one arm, then two, then three, keeping the first key that exists; break ties by a
fixed order of sides.

- Good, because it keeps more of the original drawing — measured at 12 of 32 for light and double —
  and it is the only option that keeps anything at all in a cell mixing three strokes, since no
  three-stroke table exists.
- Bad, because the outcome depends on which tables happen to be loaded, so adding one can redraw
  cells that had nothing to do with it.
- Bad, because "why that character?" becomes "because the search found this combination first".
- Neutral on cost: sixteen lookups in the worst case, which is nothing.

### C — Declared fallback chains per stroke

- Good, because a variant stroke becomes cheap: `light-round` would declare four rows and a fallback
  to `light`, and inherit every mixed combination `light` has.
- Bad, because it introduces an ordering question that does not otherwise exist — chain first or
  base stroke first — and the two answers give different characters for the same cell.
- Bad, because the discipline it needs, declaring a chain only for genuine variants, cannot be
  checked by anything.

### D — Score the rows

- Good, because it gives the most faithful character available.
- Bad, because the scoring function is a second thing to specify, to test and to keep stable across
  versions, and it is the hardest of the four to explain to someone holding a diagram that came out
  wrong.

## Reversibility

Cheap, and it stays cheap. The rule is one function behind the lookup, and every alternative is
additive: B is a search wrapped around the same two lookups, C is a map consulted before them.
Nothing in the buffer, the cell or the tables has to change to adopt any of them later.

The one thing that would harden it is diagram files in the wild whose appearance depends on the
current rule. That is far away.

## Confidence

Medium-high (75%).

What would change it: a diagram that mixes three strokes in one cell and comes out visibly poorer
than it needs to — that is exactly the case option B rescues and this rule cannot. Or a second
variant stroke being declared and copying tables again, which is the case for option C.

What would prove it wrong: users routinely loading mixing tables by hand to work around the
degradation. That would mean the rule is answering a question they wanted answered differently.

## More Information

- The model this rule belongs to: [`docs/model.md`](../model.md).
- The composition decision that puts a base stroke on every cell:
  [ADR-0008](0008-compose-overlapping-cells-with-three-state-arms.md).
