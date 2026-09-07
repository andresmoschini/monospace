---
status: accepted
date: 2026-09-07
decision-makers: Andrés Moschini
---

# Expose `cell` so stamping can be checked without rendering

## Context and Problem Statement

Every rule about stamping is observable through `render`, so nothing forces a buffer to let a caller
read a cell back. The question is whether the accessor should exist anyway.

## Decision Drivers

- A test should fail for the reason its name gives.
- Public surface that no rule requires is a promise with no reader.

## Considered Options

- **A** — No accessor. Tests reach the buffer through `render`.
- **B** — Expose `cell`.

## Decision Outcome

Chosen option: **B, expose `cell`**, because a test that reaches the buffer only through the
renderer depends on the catalog and on the lookup, and then a wrong glyph fails a test whose name
claims to be about composition. Reading a cell back keeps those two apart.

### Consequences

- Good, because the tests for stamping do not depend on any glyph rule being right.
- Bad, because it is public surface that no behavior rule requires, and the only thing that cashes
  it is a test.
- Neutral, because a later consumer that reads cells — the interactive application, an output by
  coordinates — finds it already there.

### Confirmation

By the acceptance item in [spec 0001](../specs/0001-stamp-cells-and-render-them.md) that requires
the tests for the stamping rules to read the buffer back through `cell` rather than through
`render`.

## Pros and Cons of the Options

### A — No accessor

- Good, because the surface stays as small as the rules demand.
- Bad, because every test about composition also exercises the catalog, so a fault in either fails
  tests named after the other.

### B — Expose `cell`

- Good, because the two concerns are testable apart.
- Bad, because the surface grows for a reason no rule states.

## Reversibility

Cheap while nothing outside the tests uses it. Removing it later is a breaking change like any other
in a library with no external consumers, which is to say not much of one.

## Confidence

Medium-high (75%).

What would change it: finding that the stamping tests read as well through `render`, which would
leave the accessor as surface without a purpose.

## More Information

- Extracted from [spec 0001](../specs/0001-stamp-cells-and-render-them.md), which recorded it inline
  before this record existed.
