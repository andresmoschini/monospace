<!-- cspell:ignore Behaviour -->

# Specification Quality Checklist: An arrow's end is a glyph or an arm

**Purpose**: Validate specification completeness and quality before proceeding to planning
**Created**: 2026-09-26 **Feature**: [spec.md](../spec.md)

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

No [NEEDS CLARIFICATION] marker was written, and none was needed. The spec first routed two
questions to the sheet `/speckit-plan` part one writes, because a spec takes no decision of its own,
and both are now settled ahead of it. Whether ADR-0029 is revised in place rather than recorded
beside, by
[ADR-0063](../../docs/decisions/0063-supersede-adr-0029-and-move-its-reasoning-into-the-arrow-module.md),
which supersedes its second half in part. And what a third terminal is called, which this slice no
longer carries: a terminal that neither points nor joins is recorded as a question the model owns,
without a name, and naming one is a change to the model rather than to any record. Everything else
the spec asserts was either the wish as filed or something measured before the spec was written, so
there is no ambiguity left for `/speckit-clarify` to resolve. Running it is still worth it for the
reading, not for the answers.

Three items were weighed and judged to pass rather than waved through.

- **No implementation details.** The spec names no type, no crate, no file of the implementation and
  no field of the description format. It speaks of a cell, an arm, a glyph, a terminal, a body and a
  stamp order, which is the model's vocabulary. What it does name is the subject: two model sections
  and one diagram-model section it implements, the record behind the head half, and issues #57 and
  #82 as the work that follows. The constitution asks for a section of another document to be cited
  by name, and these are names, not mechanics. The two generated pictures carry today's field name
  inside their descriptions, which is the format this slice changes and which principle VIII exempts
  from the ceiling along with the picture.
- **Success criteria are technology-agnostic.** SC-001 and SC-006 count what the repository's own
  checked artifacts hold — the sweep, the pictures feature 039 pinned, the shipped demonstration and
  the model document's examples. Those are outputs a reader can go and look at, not a stack, and the
  same sweep is what feature 103's criteria counted.
- **Written for non-technical stakeholders.** The reader this wants is someone who reads diagrams,
  and the spec is written at that level: it says what a person sees change and what must hold once
  it has. There is no user-story section because the current template has none, and _Behavior_ holds
  the two flows in their place — name a glyph, name an arm — each as Given / When / Then in the
  model's own words.

Three things to know rather than to judge.

- **The pictures sit after the list that names them, not inside it.** `cargo xtask render` writes a
  generated picture at column 0, so a marker inside a numbered item cannot hold its shape, and
  `markdownlint --fix` renumbers a list that a top-level fence interrupts — measured on this file,
  which lost three of its numbers to it before the pictures were moved out. Each group therefore
  states its scenarios first and shows the two arrangements they describe underneath, named from the
  scenario they belong to. Principle IV asks that a rendering be shown, not that it sit inside the
  sentence, so nothing is lost but the adjacency.
- The template's own heading for that section is `## Behaviour`, and this spec writes `## Behavior`.
  The repository's spell checker is American English only, `specs/` is inside the glob it reads, and
  every tracked document it can see spells it the American way. The template and
  `.github/PULL_REQUEST_TEMPLATE/building.md` both carry the British spelling without the checker
  noticing, because cspell's default glob skips dot-directories. The section and its place in the
  order are the template's; the dialect is the repository's, and the template is the file to change
  if the maintainer wants them to agree.
- `spec.md` runs to 203 lines against principle VIII's ceiling of 120, of which 40 are the two
  generated pictures and their descriptions — which the principle exempts along with the picture —
  and 5 are the one hand-drawn picture, leaving 158 chargeable. The four other specs in the tree run
  from 296 to 399 lines, so the ceiling is not what it was when the template was replaced, and the
  accounting is `plan.md`'s to record rather than this checklist's.

The spec is ready for `/speckit-clarify` if a reading pass is wanted, and for `/speckit-plan` part
one, which is the step that has to take the decisions the sheet ends up holding.
