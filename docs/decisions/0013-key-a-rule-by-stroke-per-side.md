---
status: accepted
date: 2026-09-07
decision-makers: Andrés Moschini
---

# Key a glyph rule by a stroke per side

## Context and Problem Statement

A cell in the first slice has one stroke, so every key it can build carries that same stroke on each
side that has one. The key type could be narrowed to match — one stroke plus four flags — and it
would then be impossible to express a key the slice cannot produce.

That is the shape the argument for dropping the per-arm stroke had, so the same question has to be
asked here and answered separately: is a key with four independent strokes generality nobody uses?

## Decision Drivers

- Generality that nothing reads is what the base stroke would have been, and it was rejected for
  being unverifiable.
- The tables the rules come from already have four independent columns, mixing sets included.

## Considered Options

- **A** — A key of four optional strokes, as the model describes a rule.
- **B** — A key of one stroke and four flags, matching exactly what this slice can build.

## Decision Outcome

Chosen option: **A, a stroke per side**, because it is not the inert generality the base stroke
would have been: all four sides are read on every lookup, they simply happen to hold the same stroke
while cells have only one.

The restriction lives in the function that builds a key from a cell, so loading a mixing set later
changes no type.

### Consequences

- Good, because a mixing set can be loaded without changing the key, the catalog or anything that
  reads them.
- Good, because the key has the same shape as the data it is built from, so a loader has nothing to
  translate.
- Bad, because the type can express keys this slice's cells cannot produce, and only a mixing set
  would ever answer them.

### Confirmation

By the worked example in [spec 0001](../specs/0001-stamp-cells-and-render-them.md) that shows a cell
becoming a key and the key becoming a character, with every side of the key read.

## Pros and Cons of the Options

### A — A stroke per side

- Good, because nothing changes when strokes stop being uniform.
- Bad, because it admits keys nothing in this slice builds.

### B — One stroke and four flags

- Good, because it can express exactly what the slice can produce and no more.
- Bad, because loading a mixing set later would change the key type, and with it the catalog and
  every rule in the built-in data.

## Reversibility

Asymmetric, which is what decides it. Widening B into A later touches the key, the catalog and the
data; narrowing A into B is a change nobody would have reason to make.

## Confidence

High (85%).

What would prove it wrong: mixing sets never arriving, which would leave three quarters of every key
holding a value that is always the same.

## More Information

- Extracted from [spec 0001](../specs/0001-stamp-cells-and-render-them.md), which recorded it inline
  before this record existed.
- [ADR-0012](0012-one-stroke-per-cell.md), where the same question about unused generality was
  answered the other way, and why.
