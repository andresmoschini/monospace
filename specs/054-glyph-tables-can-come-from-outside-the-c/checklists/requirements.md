# Specification Quality Checklist: Glyph tables can come from outside the core

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-09-11 **Feature**: [spec.md](../spec.md)

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

- **The spec names crates, and that is deliberate rather than a leak.** Where a glyph table lives is
  the feature's entire subject, and the constitution's _In scope for this phase_ and principle VII
  both name `monospace-core` and `monospace-cli` as the boundary the design is stated in. Naming
  them is using the project's vocabulary, not describing an implementation; what the spec does not
  name is the shape of the interface a table is handed through, which the Assumptions section
  records as a plan decision.
- **The WebAssembly target and `cargo xtask check` appear in FR-010 for the same reason.** Principle
  VII says the portability boundary is enforced by the gate rather than by prose, so a requirement
  that the new library stays portable has nowhere else to point.
- **Three questions were opened for review and all three are now answered**, in the Clarifications
  section: the library is `monospace-glyph-sets`, the constitution's scope section was amended to
  1.5.0 to admit it, and the crossing of a Light figure with an ASCII one needs nothing new because
  one stroke per cell already decides it. The third answer replaced a requirement that was wrong:
  FR-017 forbade the crossing on the belief that it would render as a hole, and it now requires the
  demonstration to show it both ways round.
