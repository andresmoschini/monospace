---
status: accepted
scope: tooling
commitment: working
date: 2026-09-21
decision-makers: Andrés Moschini
---

# Record a decision at the boundary it cannot cross

## Context and Problem Statement

Fifty records exist, forty-seven `accepted`, one for every 186 lines of Rust. Five — 0044 and 0046
to 0049 — hold one subject, how an arrow picks its route, and none can be cited without citing
another. Nothing outside `shape::arrow` can observe that subject, yet the rule sits in
`docs/model.md`, which a spec "MUST NOT restate" and which "changes first" when a slice needs a rule
it lacks. Principle VI asked for a record "at the moment it is taken", which sizes one to a moment
rather than to a subject, and it offered a single home, so every decision cost the same whatever
could see it.

## Decision Drivers

- 26,240 lines of Markdown against 9,302 of Rust, which the maintainer reports not reading. An agent
  reads them and treats a sentence nobody reviewed as a rule.
- Promoting a decision later costs one record; demoting one costs undoing the gravity it created.

## Decision Outcome

Chosen option: **record a decision at the altitude of the boundary it cannot cross, one record per
subject, declaring how far it is committed** — because altitude is the only available test a reader
can apply without knowing the history.

Principle VI is rewritten around altitude (domain into a model document plus an ADR, module into
that module's rustdoc), subject (a record that cannot be cited alone is not its own record) and
commitment (`exploratory`, `working`, `load-bearing`, each with how it is changed). Principle VIII
is new: no artifact restates another, and each has a ceiling. The earlier Sync Impact Reports move
to [`constitution-history.md`](constitution-history.md), which nothing imports.

### Consequences

- Good, because a rule nothing outside a module can see now costs a `feat` to change, which is what
  the code already said it was.
- Good, because `Reversibility` and `Confidence` are consumed at last: `commitment` is what they
  argue for.
- Bad, because altitude is a judgement whose error shows late, when a second module wants the rule.
- Bad, because fifty records carry neither field, classified as cited rather than in a sweep.

## Reversibility

Cheap while the fields are declarations nothing reads mechanically. Not cheap: a decision moved down
to rustdoc and later wanted back as a record, whose reasoning has to be rebuilt from the code.

## Confidence

Medium (70%). What would change it: finding that module notes are where decisions go to be
forgotten, because nobody greps rustdoc.

## Revisions

- 2026-09-21 — recorded.
