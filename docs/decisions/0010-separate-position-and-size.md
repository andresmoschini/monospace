---
status: accepted
date: 2026-09-07
decision-makers: Andrés Moschini
---

# Separate position and size instead of one rectangle

## Context and Problem Statement

A buffer is created from a window and a render is given an area, and each of those is a place plus
an extent. Something has to carry the four numbers, and four positional numbers in a row is an easy
place to swap two of them without the compiler noticing.

## Decision Drivers

- The model already separates them: a position is signed and may be negative, a size never is.
- Nothing in the first slice stores or passes a rectangle as one value.

## Considered Options

- **A** — Four loose integers at each call site.
- **B** — One rectangle type carrying `x`, `y`, `width` and `height`.
- **C** — `Pos` and `Size` as separate types.

## Decision Outcome

Chosen option: **C, `Pos` and `Size`**, because splitting them is what lets the compiler refuse a
size where a position belongs, which a single rectangle type cannot do.

There is no `Rect` because nothing here stores or passes a rectangle as one value; when something
does — clipping, the bounds of a shape — it is `Rect { pos, size }` built from these two.

### Consequences

- Good, because the distinction the model draws is the one the compiler enforces.
- Bad, because `render` takes four parameters where three would do, and adding `Rect` later would
  change that signature.
- Neutral, because a rectangle stays expressible the moment something needs one.

### Confirmation

By the signatures in [spec 0001](../specs/0001-stamp-cells-and-render-them.md) and by the compiler.
Nothing has to be remembered: the two types do not interchange.

## Pros and Cons of the Options

### A — Four loose integers

- Good, because it adds no types.
- Bad, because two of the four can be swapped silently at every call site.

### B — One rectangle

- Good, because a window and an area are each one value.
- Bad, because both pairs are the same type inside it, so it prevents none of the swaps that matter.

### C — Position and size

- Good, because a size cannot be passed where a position belongs.
- Bad, because it is two types where one would read more briefly, until a rectangle earns its place.

## Reversibility

Cheap. `Rect { pos, size }` is additive whenever something needs it, and the only signatures that
would change are the two this slice introduces.

## Confidence

High (85%).

What would change it: something needing to store or pass a rectangle as one value. That would add
`Rect` alongside these rather than replace them.

## More Information

- Extracted from [spec 0001](../specs/0001-stamp-cells-and-render-them.md), which recorded it inline
  before this record existed.
