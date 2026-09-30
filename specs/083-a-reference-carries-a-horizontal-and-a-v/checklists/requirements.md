# Specification Quality Checklist: A reference carries a horizontal and a vertical offset

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-09-29 **Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs)
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Success criteria are technology-agnostic (no implementation details)
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [x] No implementation details leak into specification

## Notes

Two questions were put to the maintainer on 2026-09-29 rather than left as [NEEDS CLARIFICATION]
markers. Both are recorded under **Clarifications**, and both changed the spec.

- **The frame the two offsets are measured in.** The screen axes — a horizontal and a vertical
  amount, both signed, added to the point the anchor resolves to. That is what §1 and §4 of the
  model say, and the reason that settles it rather than the anchor's own frame is
  [#143](https://github.com/andresmoschini/monospace/issues/143): a displacement is in the screen
  axes too, so it reaches a reference's offsets by adding a delta to the two fields rather than by
  converting it first, which is what the cross-note on issue #83 called a second meaning for the
  same method. The convenience the anchor's frame buys is not refused, it is written down in
  [#146](https://github.com/andresmoschini/monospace/issues/146), and that issue carries the
  constraint that keeps it from becoming a second meaning: it is a spelling taken at the boundary
  and never a second pair of fields. Issue 146 carries `capability` and `wish` like every other not
  yet built, because it is a capability and not a decision.
- **What the demonstration does.** The connector's far endpoint becomes a reference to the bottom
  side of the box above it with an offset of one in each axis, and the picture does not change. The
  arithmetic was measured rather than assumed, and it is the reason the answer is possible: that box
  is the fifth entry of the shipped description, at `{20, 1}` and four by three, so its bottom side
  middle is `{21, 3}`; the shipped description already spells the terminal at `{22, 4}`; and
  `{21, 3} + (1, 1) = {22, 4}`. The fifth picture the demonstration prints today therefore has to
  come out byte for byte identical, which is what SC-005 claims and what the contract test in
  **Testing expectations** pins.

**One of the two reverses an answer 082 gave**, and the spec says so rather than quietly dropping
it. Spec 082 recorded, under **What this slice does not decide**: "Whether the description format
grows a field for a reference. Nothing here needs one: the demonstration can build the picture in
its own code. Settled by: the first consumer that reads a diagram from a file rather than from
code." This slice is that consumer, and the maintainer's answer is that the evidence of the offset
belongs in the description rather than in the picture. B4 is therefore a new behavior group, and
`contracts/description-format.md` and `data-model.md` are artifacts this increment revises rather
than ones it inherits. The port is a consumer of the model and not a change to it, which is what
[ADR-0035](../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md) requires, and the
spec says so in the same place it names the format.

The reversal has a consequence the spec names rather than leaves for review: **a description has to
name a shape**, and the format names its shapes by the place they are listed. Whether the name
written down is the identity the diagram issues or the position in the list is a decision, and it is
routed to the sheet together with the question 082's own checklist flagged — whether a caller can
name an identity at all. The second of those stops being hypothetical here: the model's own trigger
for answering it is the first slice reading a diagram from a file, and B4 is that slice.

Three things sit close to the line between requirement and design, and each is named in the spec:

- **What type carries the two amounts.** The spec says a reference carries a horizontal and a
  vertical amount and that both are signed, which is the requirement a caller can be held to.
  Whether that is a second use of the crate's existing `Delta` or a type of its own is the sheet's,
  and the reuse is not free: `Delta` is documented as how far a figure moves, and a reference's
  offset is how far from a side — the same two numbers with a different meaning, which is the same
  hazard #146's constraint exists to avoid.
- **How a caller reaches the arithmetic.** The spec says a reference resolves to its anchor plus the
  two offsets, which is §4's own sentence, and does not say whether that is one addition or two
  axis-at-a-time additions.
- **Whether a caller can ask where a position resolves to.** B1 and B2 require the number, but only
  through drawing and through the crate's own tests. A public query stays out, under the removal
  test: no consumer needs it, and the anchors exist to be resolved _through_.

Every picture in `spec.md` is hypothetical and measured, not generated: each block is what the
current binary prints for the _absolute_ equivalent of the requirement, run through
`cargo run -p monospace-cli`. B1.2's second block is the picture issue 082 already produces, which
is what makes the pair a claim about the offset rather than about the arrow. `cargo xtask render`
must not be asked to fill any of them — a reference is not expressible in the description format the
marker reads, which is what
[ADR-0064](../../docs/decisions/0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md)
records, and the gate would fail on a shape the format cannot build.

This template has no **Assumptions** section, and none is missing: the assumptions are the four
model sections named under **What this slice implements** — none amended, because the model already
describes this end to end — and the five open questions under **What this slice does not decide**,
each with what would force an answer. The dependencies are the six issues this spec names, 62, 82,
89, 143 and 146 with the ADR it cites.

One item is not satisfiable as written: **`spec.md` exceeds the 120-line ceiling in principle
VIII**, at 199 attributable lines against a ceiling of 120 — 254 in the file, less the ten fences,
the forty-four blank lines and the one comment, measured the way feature 055's plan measured its own
166 and the way specs 081 and 082 measured theirs. This is a fact about the slice's size rather than
a formatting slip, and the constitution asks for a split first.

**The split available is the crate boundary, and it is clean.** B1, B2 and B3 are
`monospace-diagram`: the two offsets exist, they are added at resolution, and a displacement still
does not reach them. B4 and B5 are `monospace-cli`: a description can name a reference, and the
demonstration's far end hangs from a box. B4 needs B1 — a reference with an offset has to exist
before a file can name one — and B5 needs B4, so the cut has an order and no cycle. Taken at that
line, the first slice carries no contract revision and no demonstration change, and the second
carries nothing that the first does not already have behind it.

It does not reach the ceiling either way, so it is offered rather than recommended: the arithmetic
above leaves the offsets slice at roughly 165 attributable lines, still 45 over, which is the same
kind of overage 081 and 082 both carried and which the maintainer declined to split on 2026-09-28
for the same reason — the split buys a thinner slice rather than one that fits, and it costs a
second deciding cycle with its own issue, branch, spec, decision sheet and pull request. The number
and the reason go to `plan.md`'s Complexity Tracking, which is where the constitution puts a ceiling
exceeded without a split, and the answer is the maintainer's to change at plan time.
