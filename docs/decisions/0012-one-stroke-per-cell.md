---
status: accepted
date: 2026-09-07
decision-makers: Andrés Moschini
---

# Give a cell one stroke, with none per arm

## Context and Problem Statement

In the model an arm carries a stroke of its own, and the cell's base stroke is what every arm falls
back to. The first slice leaves out degradation, and without it an arm's own stroke has nothing to
fall back from: a cell mixing strokes could be built but never rendered, since the only glyph set
loaded holds no mixed rules.

So the first draft of the spec kept `Arm::Set(Stroke)` and left the base stroke stored but never
read, which is a field with no test that can pin its meaning.

## Decision Drivers

- A field nothing reads has no test that can pin its meaning, so an error in it surfaces specs
  later.
- What this slice can express should be a restriction of the model, not a variation on it.

## Considered Options

- **A** — Keep `Arm::Set(Stroke)` and carry the base stroke inert until degradation arrives.
- **B** — Keep the per-arm stroke and drop the base stroke until degradation needs it.
- **C** — Drop the per-arm stroke. A cell has one stroke, its base, and every arm draws in it.

## Decision Outcome

Chosen option: **C, one stroke per cell**, because it leaves nothing inert: the base stroke is the
only stroke a cell has, it is read on every render, and it is covered by tests from the first commit
rather than carried until the spec that gives it meaning.

Dropping the per-arm stroke is a restriction on what can be expressed, not a different rule. For
every cell this slice can build, the equivalent cell in the full model renders the same character:
if the key exists both find it, and if it does not, the model moves every connected arm to the base
stroke and arrives at the key that just failed. So nothing here has to be unlearned.

### Consequences

- Good, because every piece of a cell is read, and a mistake in any of it fails a test now rather
  than three specs from here.
- Good, because the example for a key with no rule becomes a cell whose base stroke has no rules
  loaded, which stays true once degradation exists, instead of a mixed cell that stops being
  reachable.
- Bad, because `Arm::Set` gains a payload later, and that touches every construction site including
  the tests, so the step cannot be a `refactor` commit under this project's rule.

### Confirmation

By the tests in [spec 0001](../specs/0001-stamp-cells-and-render-them.md): every example renders
through the base stroke, so no test passes while it is wrong.

## Pros and Cons of the Options

### A — Per-arm stroke, base stroke inert

- Good, because no type changes when degradation arrives.
- Bad, because the base stroke is stored, never read and therefore never verified.
- Bad, because cells can be built that this slice can only render as spaces.

### B — Per-arm stroke, no base stroke

- Good, because nothing is inert either.
- Bad, because the model says a defined cell always has a base stroke, so the slice would be missing
  a field the model requires rather than restricting one.

### C — One stroke per cell

- Good, because it is a faithful restriction: every cell it can build renders as the model would.
- Bad, because the type changes later, mechanically but everywhere.

## Reversibility

The change is mechanical and the compiler finds every site, but it is not free: adding the payload
to `Arm::Set` edits construction sites in the library, the CLI and the tests at once.

## Confidence

High (85%).

What would change it: needing to render a mixed cell before degradation exists, which nothing in the
planned order requires.

## More Information

- Extracted from [spec 0001](../specs/0001-stamp-cells-and-render-them.md), which recorded it inline
  before this record existed.
- [ADR-0009](0009-degrade-a-cell-to-its-base-stroke.md), the degradation rule that gives a per-arm
  stroke something to fall back from.
