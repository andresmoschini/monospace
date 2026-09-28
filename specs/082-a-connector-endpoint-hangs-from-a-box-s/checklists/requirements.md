# Specification Quality Checklist: A connector endpoint hangs from a box's side anchor

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-09-28 **Feature**: [spec.md](../spec.md)

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

Two questions were put to the maintainer on 2026-09-28 rather than left as [NEEDS CLARIFICATION]
markers, and both are recorded under **Clarifications**. They are the two subjects the description
named as temporary, and each one now has an issue carrying the answer we actually want:

- **Taking a referenced shape out.** The general answer, unchanged: the reference is left as it was,
  it does not resolve, and the shape that held it is not drawn. B3 makes that observable for the
  first time; it is not a decision this slice takes.
- **Displacing a figure that hangs from another.** A no-op, and a silent one. There is nothing for
  the displacement to reach until a reference has offsets, which is #83.

Two issues were created for those, and the spec cites them by number:

- [#142](https://github.com/andresmoschini/monospace/issues/142), _Taking a shape out leaves the
  figures that hung from it where they were_ — the removal answer.
- [#143](https://github.com/andresmoschini/monospace/issues/143), _Displacing a figure that holds a
  reference moves it_ — the displacement answer, and it names #83 as the thing it depends on.

Both carry the `capability` and `wish` labels, the way #83, #84, #88, #89 and #90 do, because
neither is a decision and neither has a branch. The branch, the `deciding` label and the pointer to
this directory were created by `cargo xtask spec`, not by hand.

Three of the items above are ticked against the wording of the generic checklist rather than against
this repository's own template, and the reason is the same in all three: **this is a library, so
naming what the library offers is the specification, and its reader is a caller rather than an end
user.** `Position`, `Reference` and `Anchor` are named because those are the three things a caller
is being promised, and `cargo xtask check` and `wasm32-unknown-unknown` are named in SC-008 because
"the gate is green, including the core still compiling for WebAssembly" is how the constitution
defines green. Specs 055, 080 and 081 tick the same items the same way. The line the spec does hold
is the one that matters: it takes no decision, and it says nothing about how any of it is typed.

**Every picture in `spec.md` is hypothetical**, and each is measured rather than guessed. The four
blocks are what the current code already draws for the _absolute_ equivalent of each requirement — a
connector whose `from` sits at the point a reference would resolve to, run through
`cargo run -p monospace-cli` — which is what makes the requirement checkable: the referenced diagram
has to draw exactly that. B3's second block is the exception worth naming: it is the window with the
connector simply gone, and it is empty where the arrow used to be, because a connector whose
position does not resolve writes nothing at all rather than drawing most of itself. The four blocks
are labelled as hypothetical in the sense the constitution defines — no code produces them yet, and
`cargo xtask render` must not be asked to fill them, or the gate will fail on a shape the model
cannot yet build.

Two things sit close to the line between requirement and design, and both are named in the spec
rather than left to be discovered at review:

- **The reference in this slice is two fields, not three.** §1 and §4 of the model both describe a
  `Reference` as an identity, an anchor and two offsets, and the offsets arrive with #83. So the
  type this slice builds is the first two of the three the model describes, which is what makes B4's
  no-op the only displacement answer available: a displacement reaches offsets, and there are none
  yet. The spec names the gap under **What this slice does not decide** and asks the decision sheet,
  not this file, whether the model needs a line saying which fields have landed.
- **Whether a caller can ask a shape where one of its anchors is.** B1 states the rule the model
  states — each kind answers its four side centers or nothing — and does not commit to a public
  query for it, because drawing is the only consumer so far and the constitution's removal test is
  about exactly this. The same reasoning is why spec 081's B4 states that no listing of a diagram's
  identities exists and leaves the question to issue 86.

The one thing that is an assumption rather than a decision, and that the maintainer should confirm
at `/speckit.clarify` if it is wrong: **the shipped demonstration grows a fifth picture.** The body
of issue 82 does not ask for one, and a reference cannot be expressed in the description format, so
the picture has to be built by the demonstration's own code — the route 081's clarification already
took for the displacement's amount. The alternative, leaving `monospace-cli` untouched, keeps this
spec about twenty lines shorter and makes the slice's one claim invisible to anyone who runs the
application, which is what principle II asks an increment to be.

This template has no **Assumptions** section, and none is missing: the assumptions are the five
model sections named under **What this slice implements** — none amended, because the model already
describes this end to end — and the six open questions under **What this slice does not decide**,
each with what would force an answer. The dependencies are the nine issues this spec names, 62, 81,
83, 84, 88, 89, 90, 142 and 143, and the two ADRs it cites.

One item is not satisfiable as written: **`spec.md` exceeds the 120-line ceiling in principle
VIII**, at 209 attributable lines against a ceiling of 120 — 272 lines in the file, less the
fourteen fences, the forty-eight blank lines and the one comment, measured the way feature 055's
plan measured its own 166 and the way spec 081 measured its own 288. That is a fact about the
slice's size rather than a formatting slip, and the answer the constitution asks for first is to
split the feature. The split available here is the one issue 62 already made: the anchors in B1 and
the reference in B2, with B3 and B4 riding along on B2 because neither is reachable without one. It
is offered here rather than taken, because the maintainer set the scope of this slice in the same
message that set the two temporary answers, and splitting it would put B3 and B4 in a slice with no
references in it. The ceiling is recorded here and `plan.md`'s Complexity Tracking carries it at
plan time, the way feature 081's does.
