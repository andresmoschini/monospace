---
status: accepted
scope: tooling
commitment: exploratory
date: 2026-09-27
decision-makers: Andrés Moschini
---

# Give each generated picture the carrier that can reach its subject

## Context and Problem Statement

[ADR-0052](0052-show-the-rendering.md) asks that where the subject of an artifact is something this
project renders, the artifact shows the rendering. It defines one way of generating a picture: a
`<!-- render: … -->` marker beside it, carrying a description in JSON, rewritten by
`cargo xtask render` and checked by a step of `cargo xtask check`.

That mechanism reaches whatever a description can name, which today means the three diagram shapes
at any position in any composition. It cannot reach a fragment: `Border`, `Corner`, `End`, `Fill`,
`Head` and `Segment` are `pub(crate)`, and no JSON gets there. So the figures those fragments
compose have no generated picture at all, and what a border run writes is stated in the model's
prose and asserted in `border.rs` one arm at a time — neither of which a reader holds in their head,
and the first of which cannot show the difference that matters, because a border with
`closes_interior` true and with it false render the same three characters. The requirement has two
carriers now and nothing records which owns what.

## Decision Outcome

Chosen option: **a marker owns a picture in a document, a gallery owns a picture of a figure, and
neither substitutes for the other** — because the split is reach rather than preference. A marker is
a description a subprocess reads, and whatever the crate keeps private is outside its reach by
construction; a gallery draws the values themselves, which is the only way to see what a fragment
stamps.

What this deliberately leaves open: the gallery's form. Where its files live, what a block's label
and table look like, whether a figure appears once or once per variant, and whether anything sits
below the figures are all revisable here without asking, which is what `exploratory` is for.

## Reversibility

Cheap, and the commitment is chosen for that. Nothing consumes a gallery: a figure's behavior is
held by the tests that assert it, and this adds a place to look rather than a guarantee replacing
one. Dropping the second carrier leaves ADR-0052 and every marker doing what they already do.

The one change that does not come back is a `Debug` derive on a figure type, which is public API on
four of the ten that need it. That is the cost of labels that cannot disagree with the value they
name, and it is the only reason this record is worth writing down.

## Revisions

- 2026-09-27 — recorded, after a spike on `spike-the-shape-gallery` measured a block per figure at
  roughly 400 lines and confirmed that the two `Border` variants are indistinguishable in a picture
  while differing in the table beside it.

## More Information

- [ADR-0052](0052-show-the-rendering.md), whose status is unchanged: this adds a carrier rather than
  contradicting it.
- [ADR-0028](0028-give-each-fragment-its-own-cell-rule.md), for why a fragment is `pub(crate)` and
  for the six rules a gallery makes readable.
