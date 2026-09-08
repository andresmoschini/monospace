# Specification Quality Checklist: Give a glyph a type of its own

**Purpose**: Validate specification completeness and quality before proceeding to planning

**Created**: 2026-09-08

**Feature**: [spec.md](../spec.md)

## Content Quality

- [x] No implementation details (languages, frameworks, APIs) — deliberate deviation, see Notes
- [x] Focused on user value and business needs
- [x] Written for non-technical stakeholders
- [x] All mandatory sections completed

## Requirement Completeness

- [x] No [NEEDS CLARIFICATION] markers remain
- [x] Requirements are testable and unambiguous
- [x] Success criteria are measurable
- [x] Success criteria are technology-agnostic (no implementation details) — see Notes
- [x] All acceptance scenarios are defined
- [x] Edge cases are identified
- [x] Scope is clearly bounded
- [x] Dependencies and assumptions identified

## Feature Readiness

- [x] All functional requirements have clear acceptance criteria
- [x] User scenarios cover primary flows
- [x] Feature meets measurable outcomes defined in Success Criteria
- [x] No implementation details leak into specification — deliberate deviation, see Notes

## Story independence

Not part of the standard checklist, added because the first draft of this spec failed it and the
failure was not visible from any item above.

- [x] Each story, shipped alone and stopped after, leaves something demonstrable
- [x] No story is a partition by technical layer of another story
- [x] No story is really an acceptance criterion or an invariant
- [x] The story list matches the increment list: two stories, two increments, same order

## Notes

- **The two "no implementation details" items are deliberate deviations, not oversights.** The
  deliverable of this slice is a library's public surface, so the surface _is_ what a user sees;
  there is no screen behind which to hide it. This repository's own specs carry a public-surface
  section for that reason — spec 0001 does, and the input document does — and the surface here was
  agreed before this spec existed, in ADR-0019 and in the input. What the spec still does not name
  is anything internal: no storage layout, no algorithm, and no crate. Which crate and which version
  belong to the plan's research, and the spec says so under _Assumptions_.
- **Success criteria name commands and files** — `cargo run -p monospace-cli`,
  `crates/monospace-cli/tests/cli.rs`, `cargo xtask check`. For a slice whose whole acceptance is
  "the output does not change", those are the measurement, and _One definition of green_ in the
  constitution makes the gate command the only definition of passing. A criterion phrased without
  them would not be verifiable.
- **The first draft had three stories and was rewritten to two.** The type, its consumers and "the
  output does not move" were three slices of one change: the second was not independently
  deliverable, and the third was an acceptance criterion wearing a story's clothes. The two stories
  now match the two increments the input document agreed, so the priority order and the increment
  order are the same list rather than two competing ones.
- **The commit plan was removed from the spec.** It said "two commits, in this order", which the
  story priorities now say by themselves; the justification for P1 editing existing assertions moved
  to _Handoff to the plan_, where the constitution puts departures from itself.
