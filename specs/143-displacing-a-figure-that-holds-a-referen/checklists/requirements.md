# Specification Quality Checklist: Displacing a figure that holds a reference moves it

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-10-01 **Feature**: [spec.md](../spec.md)

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

Three questions were put to the maintainer on 2026-10-01 rather than left as markers. All three are
recorded under **Clarifications** and all three changed the spec; the markers are gone.

- **P1 — where does the displacement reach? → A reference's offsets**, which is what §4 states. The
  alternatives were refused on the model's own words rather than on taste: reaching the place the
  reference resolves to moves another figure, which §4's "a displacement is a property of one figure
  rather than of a diagram" and §9's "nothing cascades" both forbid, and refusing would leave
  [#62](https://github.com/andresmoschini/monospace/issues/62)'s second-from-last bullet
  unimplementable. §11's question comes out and the ADR is the sheet's; the spec states the rule §4
  already states.
- **P2 — a caller that displaces both figures? → Not this slice's question, and not yet a
  decision.** The maintainer's answer was that there is no selection yet and the question waits for
  one. That is right on the evidence, and the evidence was checked rather than taken: the bare
  demonstration displaces **one** figure per step (`#1` in the third, `#3` in the fifth), no marker
  displaces anything, and `sweep.rs` builds its descriptions from shape lists. So the arrangement is
  unreachable. The arithmetic is still **derived** and stated under **Edge cases** — the connector
  ends up twice as far down with its gap grown — and pinned by a contract test, so the slice that
  brings a selection meets a stated answer instead of discovering one in a picture. What the answer
  bought: the slice amends neither §4 nor §10, which is what naming it early would have cost.
- **P3 — does it reach the description format? → No, and this half was measured before it was
  asked.** The expensive half — a `by` field in the format, ADR-0035, and the twenty-four tracked
  markers that read it — does not exist, because the demonstration's changes are made in
  `monospace-cli`'s own code: `main.rs` carries its deltas deliberately, "rather than a field in the
  description format or an argument on the binary" (B5.8). So the evidence is a **sixth** captioned
  picture, and the format, ADR-0035 and every marker are untouched.

**The sixth picture is better evidence than the arrangement the other two pictures use**, which the
question did not anticipate. The demonstration's fifth step rehangs the arrow from the box it
already pointed at, so at that point **both** of its endpoints are references — a displacement
reaches both offsets at once, and the picture shows the whole figure leaving both boxes while both
stand still. Measured, by adding the step to `main.rs` and running it: **today's answer is the fifth
picture again, byte for byte**, because a displacement reaches neither endpoint. The silent no-op
the issue is about is reproducible in the shipped demonstration, which is the cheapest evidence
available that the bug is real rather than described.

Every picture is measured rather than generated, and each says so on the spot:

- The arrangement, **B1.1** and **B1.2** are drawn by building the values the rule yields — a
  reference to the same side carrying the offset the rule produces, with the `to` the rule produces
  — because no code displaces into them today. Building the post-condition is how the claim was
  checked rather than asserted.
- **B2.1** is 083's own rule, so it is that slice's measured pair again.
- **B2.2** is what `monospace-diagram` draws today for that exact displacement: the endpoint stays
  welded to the border, the free end drops two cells, and the route bends.
- **B3.1** is what the demonstration draws once the sixth step is added, set against the fifth it
  does not change at all today.

`<!-- render: -->` must not be asked to fill any of them, for two separate reasons: a marker reads a
description and a description cannot displace anything
([ADR-0064](../../docs/decisions/0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md)),
and **no marker draws the demonstration's canvas anyway** — the fifty-by-thirteen window is the
largest in the repository and the largest marker is twenty-four by nine. So SC-005 costs nothing to
verify and nothing to regenerate.

Measured on 2026-10-01 by adding `crates/monospace-diagram/tests/scratch_143.rs` and a sixth step to
`crates/monospace-cli/src/main.rs`, running both, and deleting the first and restoring the second
from a copy. The working tree carries no trace: `git diff` on `main.rs` is empty and `git status`
shows this directory and `.vscode/settings.json` as the only changes.

Three things outside this file also declare the behavior this slice replaces, and the slice rewrites
all three rather than contradicting them in passing: `crates/monospace-diagram/src/position.rs`
carries two rustdoc paragraphs calling the no-op "a deliberate no-op rather than an omission" and
naming `#143` as the decision that would change it, and one test named
`a_displacement_moves_a_point_and_leaves_a_reference_alone` pins it by value. The spec names them
under **Testing expectations** rather than restating what they say.

**One item is not satisfiable as written: `spec.md` exceeds the 120-line ceiling in principle
VIII**, at 230 attributable lines against a ceiling of 120 — 286 in the file, less the forty-two
blank lines, the ten fence markers and the four-line comment, counted the way 083's and 148's own
checklists counted their 199 and 187. Twenty of those lines are the five pictures, which are the
measured evidence the slice's claim rests on and which principle VIII does not charge for a record
that replaces prose.

This slice is already as thin as it can be made: one function in one file, one rule, one picture
that moves, and one sixth picture in the demonstration that shows it. There is no split available
inside it, and the same overage is the one
[#83](https://github.com/andresmoschini/monospace/issues/83) and
[#148](https://github.com/andresmoschini/monospace/issues/148) carried, which the maintainer
declined to split for the same reason — the split buys two thinner slices at the price of a second
deciding cycle with its own issue, branch, spec, sheet and pull request. The number is above those
two rather than beside them, and the difference is P3's sixth picture with the edge cases it brought
— which is the part of the slice that was added on the strength of an answer rather than asked for
by the issue. The number and the reason go to `plan.md`'s Complexity Tracking, which is where the
constitution puts a ceiling exceeded without a split.

Deleting prose would bring this under 120 and would be the wrong move: principle VIII says an
artifact MUST NOT restate what another already says, and each block here carries something nowhere
else — the five pictures, the two directions of B2 in one test, the saturation edge case, the
derived pair, and the six non-decisions each with what would force an answer.

This template has no **Assumptions** section and none is missing: the assumptions are the three
model sections named under **What this slice implements**, the two issues this spec builds on, 82
and 83, and the six entries under **What this slice does not decide**, each with what would force an
answer.
