---
status: accepted
scope: domain
commitment: exploratory
date: 2026-09-27
decision-makers: "Andrés Moschini, with Space Bunny Free"
---

# Supersede ADR-0029's second half, and move its reasoning into the arrow module

## Context and Problem Statement

[ADR-0029](0029-draw-a-line-end-as-one-arm.md) chose an answer for the cell where a stroke stops and
an answer for the cell where a connector ends, and recorded both as facts about two shapes. One of
them no longer holds: the cell an arrow endpoint writes is a choice, and the choice is settled by
measurement rather than by taste — a literal composes as four `Closed` arms and interrupts whatever
it is placed on, while a partial cell decision composes in either order. The measurement and the
pictures are in the module's `Design notes`, which is where the reasoning now lives.

ADR-0029 is `accepted` and predates the fields principle VI requires, so `docs/decisions/README.md`
classifies a record on the next time it is touched. This is that time.

## Decision Outcome

Chosen option: **the reasoning moves down to the module and this record supersedes the half that no
longer holds** — because nothing outside `shape::connector` can observe what an endpoint writes in
its own cell beyond the picture it produces, which is the test
[ADR-0050](0050-record-a-decision-at-the-boundary-it-cannot-cross.md) put in the template.

[ADR-0029](0029-draw-a-line-end-as-one-arm.md) becomes `accepted; superseded in part by ADR-0063`,
with `scope: domain` and `commitment: load-bearing`. Its prose is untouched, including a
`Reversibility` section that called reversing it cheap until a description format outside the
process existed. That format does now exist, which is what the `load-bearing` classification says.

[`docs/model.md`](../model.md) states the rule and owns the vocabulary of what an endpoint may
write. This record does not enumerate that vocabulary, and adding to it costs the model and the
figure nothing here.

## Reversibility

Cheap. Reopening a record whose prose is history is a status line, and the reasoning that moved is
prose in a module that a reader of the code is already looking at.

It reopens only if the model's account of what an endpoint may write is replaced rather than
extended. Naming one more thing an endpoint can write is not that.

## Confidence

High (85%). What would change it: a second module that has to know what an endpoint wrote, which
would put the subject back above the boundary it now sits under.

## Revisions

- 2026-09-27 — recorded. Supersedes the second half of
  [ADR-0029](0029-draw-a-line-end-as-one-arm.md); the reasoning moves to
  `crates/monospace-core/src/shape/connector.rs`.

## More Information

- The `Design notes` of `crates/monospace-core/src/shape/connector.rs`, where the reasoning lives
  and which already holds how a route is ranked.
- [ADR-0050](0050-record-a-decision-at-the-boundary-it-cannot-cross.md) for the altitude test, and
  [ADR-0026](0026-represent-a-cell-as-a-sum-of-strokes-and-a-literal.md) for why a literal composes
  as four `Closed` arms.
- [Issue #55](https://github.com/andreschini/monospace/issues/55) and the `spec.md` under
  `specs/055-an-arrow-end-is-a-glyph-arm-or-nothing/`.
