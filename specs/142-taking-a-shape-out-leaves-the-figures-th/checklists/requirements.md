# Specification Quality Checklist: Taking a shape out leaves what hangs from it not drawn, and the question is written down

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

Three questions were put to the maintainer on 2026-10-01 rather than left as markers, in Spanish and
with pictures, because the slice takes no decision and the questions were all about how the question
itself should be recorded. All three are under **Clarifications** and all three changed the spec.

- **P1 — what does a slice that decides nothing leave behind? → The question written down plus a
  seventh picture in the shipped demonstration.** The rejected option was a rule recorded only in a
  test. That was refused on this repository's own evidence rather than on taste: a rule's evidence
  has been a picture in the shipped run since 080, and a reader who has to run a test to see what a
  rule does is being asked to take the rule on faith first. What it bought: B2, and with it the only
  observable change in the slice. `assets/demo.json` and the description format are untouched, the
  seventh picture is a change `monospace-cli` makes in its own code, and the count in SC-001 —
  twenty-two cells, the box's twelve and the arrow's ten — was measured by differing the sixth
  against the seventh rather than by reading either.
- **P2 — one question or two? → Two, and the second written step by step.** The maintainer asked for
  the second one to be spelled out rather than summarized, and B3 is that spelling: three routes,
  each a Given/When/Then, arriving at a picture with the arrow's ten cells gone. The step-by-step
  form is not decoration. Written as a sentence, the second question reads as a design preference;
  written as three routes, it reads as the thing it is — B3.2 and B3.3 land on the same picture from
  opposite sides, and B3.3 is the one that closes it, because there the identity is **still in the
  diagram** and `get` finds it. A future answer cannot be "check whether the identity is still
  there", and the three routes are what shows that.
  - One correction this produced, and it is recorded because a spec that asserted the wrong thing
    would have been caught only later: the three routes do **not** draw byte-for-byte equal buffers.
    Route C's replacement draws its own cells, so the test compares the arrow's footprint rather
    than the whole buffer, and says why at the comparison. Measured, not assumed.
- **P3 — does §9 get a note, or does the question live only in §11? → A note, in §9.** The
  maintainer's condition on that answer is worth carrying into the plan: the note is not a claim
  that this one rule is provisional while the rest are settled, because no rule here is final. The
  note says only that this rule has a question standing next to it and names where. That is the
  whole of it, and it is why the spec does not say "temporary" anywhere — the constitution's
  _Understanding changes_ already owns that posture, and repeating it in one rule would be the
  duplication principle VIII refuses.

**On the ceiling.** This spec is over the 120 lines the template names, as its four nearest siblings
are (081 at 314, 082 at 305, 143 at 319, 148 at 227). The overage goes to `plan.md`'s Complexity
Tracking in part one, and the reading is that the template's ceiling and this repository's practice
have parted company rather than that this slice is thick: the four artifacts above are all the same
shape, and none of them was split. What this spec spends its lines on is the measured pictures and
the clarifications, and the pictures do not count against a ceiling. If the maintainer would rather
the practice move than the documents, that is a tooling change and takes an ADR, not a spec edit.

**The one place this spec names a fact that is not in the model.** §9's removal paragraph and one
test's rustdoc between them say a removed identity can never be filled again. `add_under` arrived
with [#148](https://github.com/andresmoschini/monospace/issues/148), is `pub`, and makes that false
— two of that paragraph's three claims. The spec states it under **Testing expectations** as work
this slice does rather than a defect it reports, because a paragraph that contradicts a behavior the
same slice adds a test for is fixed in the same increment.
