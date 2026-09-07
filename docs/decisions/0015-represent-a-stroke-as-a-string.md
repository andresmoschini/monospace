---
status: accepted
date: 2026-09-07
decision-makers: Andrés Moschini
---

# Represent a stroke as an owned `String`

## Context and Problem Statement

The model says a stroke is only a name: no attributes, no declaration of its own, existing because
cells and rules mention it. A name still has to be represented, and the choice reaches every cell,
every key and every rule.

## Decision Drivers

- Strokes are compared on every lookup and stored in every cell, so the representation is on the hot
  path if there ever is one.
- Sets will be loaded from a file, so a stroke's name is not known at compile time.

## Considered Options

- **A** — `String`.
- **B** — `&'static str`.
- **C** — An interned id with a registry.

## Decision Outcome

Chosen option: **A, `String`**, because it is the least committed thing that can be a name.

It costs an allocation per cell, which is invisible at the sizes this handles. `From<&str>` is part
of the public surface, because without it every construction site says
`Stroke(String::from("light"))`.

### Consequences

- Good, because nothing about it has to be decided before there is something to measure.
- Bad, because a cell allocates, and a large buffer allocates once per cell.
- Neutral, because the trigger to revisit is a measurement rather than a feeling.

### Confirmation

None yet, and deliberately so: there is nothing to measure until a buffer is filled at a size that
matters. The record names the trigger instead of asserting a cost.

## Pros and Cons of the Options

### A — `String`

- Good, because it works for a name from any source, including a file.
- Bad, because it allocates, per cell and per key.

### B — `&'static str`

- Good, because there is no allocation at all.
- Bad, because it stops working the moment a set is loaded from a file, which is a planned spec.

### C — An interned id

- Good, because comparison becomes an integer and a stroke costs a word.
- Bad, because it puts a registry in the public API before anything has shown it is needed, and
  every construction site then needs access to it.

## Reversibility

Not free. Changing the representation touches every construction site: the library, the built-in
rules, the CLI and the tests. It is mechanical, and the compiler finds all of it.

## Confidence

Medium-high (75%).

What would change it: a measurement on a buffer of a realistic size showing the allocations matter.
That is the only thing that should, and option C is what it would lead to.

## More Information

- Extracted from [spec 0001](../specs/0001-stamp-cells-and-render-them.md), which recorded it as an
  open question and then as a decision, inline, before this record existed.
