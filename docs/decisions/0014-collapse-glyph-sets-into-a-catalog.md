---
status: accepted
date: 2026-09-07
decision-makers: Andrés Moschini
---

# Collapse glyph sets into a single catalog

## Context and Problem Statement

The model described two things: a glyph set, a group of rules, and a catalog, the loaded sets in
order with the first match winning. The tables in `glyph-sets.md` are separated by headings, which
made the grouping look like part of the design.

The question that undoes it is short: what behavior depends on a rule belonging to one set rather
than another? There is exactly one answer, precedence, and precedence is settled while the catalog
is being built rather than while it is being read.

## Decision Drivers

- Two words for one thing is a vocabulary that has to be kept true in two places.
- Answering a key should be a lookup, not a walk over sets.

## Considered Options

- **A** — Keep an ordered list of sets and consult them in turn.
- **B** — One flat catalog, built by inserting sets in order, where the first rule to claim a key
  keeps it.

## Decision Outcome

Chosen option: **B, one catalog**, because a catalog assembled that way is indistinguishable from an
ordered list consulted in turn, and it is one lookup instead of a walk.

A set is now what a table is when someone writes or loads it, and the catalog is where the rules end
up. Once a catalog is built, nothing can tell which set a rule came from, and no result would change
if it could. Order is a property of how a catalog is built, not of what it holds — which is also how
"render this in ASCII alone" is expressed, by building a catalog from that set and no other.

### Consequences

- Good, because a lookup is a lookup, whatever was loaded and in whatever order.
- Good, because the name of a set stays where it belongs, on the thing someone writes or loads.
- Bad, because nothing can report which set answered a key, so a diagnostic asking why a character
  came out that way cannot name its source.
- Neutral, because an invariant left the model with it: "a glyph set does not repeat a key" cannot
  fail against a catalog, since two rules claiming one key collapse on insertion by definition. An
  invariant the representation makes vacuous is one to delete rather than to test.

### Confirmation

By the model, which now describes one catalog and sets as what goes into it, and by
[spec 0001](../specs/0001-stamp-cells-and-render-them.md), whose catalog answers keys and cannot be
asked anything else.

## Pros and Cons of the Options

### A — An ordered list of sets

- Good, because provenance survives, so a diagnostic could name the set a rule came from.
- Good, because a set could be removed after the fact.
- Bad, because every lookup walks the list, and the walk is the only thing the structure buys.
- Bad, because two entities exist where the behavior of one is the behavior of the other.

### B — One catalog

- Good, because the structure holds nothing that does not change an answer.
- Bad, because provenance is gone, and rebuilding is the only way to change what is loaded.

## Reversibility

Cheap. A catalog that also remembers where each rule came from is additive: the lookup keeps its
shape and gains a second thing to return.

## Confidence

High (85%).

What would change it: wanting to say which set answered a key, either in a diagnostic or in a tool
that explains a rendered character. That is a plausible want, and it is what the rejected option
buys.

## More Information

- Extracted from [spec 0001](../specs/0001-stamp-cells-and-render-them.md) and from the model
  section it summarized, neither of which recorded the alternatives.
