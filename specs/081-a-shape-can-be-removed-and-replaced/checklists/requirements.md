# Specification Quality Checklist: A shape can be removed and replaced

**Purpose**: Validate specification completeness and quality before proceeding to planning

**Created**: 2026-09-27

**Feature**: [spec.md](../spec.md)

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

Three questions were put to the maintainer on 2026-09-27 rather than left as [NEEDS CLARIFICATION]
markers, and all three are recorded under **Clarifications**. The first is the one that sets the
slice's size, and it is worth stating plainly because issue #81's body reads the other way:

- **Scope.** The issue defers movement to "the slice that implements movement". The spec brings
  `remove`, `replace`, the displacement, and the reader into #81 together. That was the maintainer's
  answer, given the alternative of `remove` and `replace` alone.
- **Model amendments.** Four sections of `docs/diagram-model.md` are amended on this branch ahead of
  this spec, so the model's displacement paragraph is written before the code that implements it.
- **A stale claim in spec 080.** `contracts/diagram-api.md` says an identity has no constructor
  while `ShapeId::new` is public and the shipped demonstration uses it. Corrected on this branch in
  a commit of its own.

Two of the items above are ticked against the wording of the generic checklist rather than against
this repository's own template, and the reason is the same in both: **this is a library, so naming
what the library offers is the specification, and its reader is a caller rather than an end user.**
`remove`, `replace`, the displacement and the reader are named in the spec because those are the
four things a caller is being promised, and `cargo xtask check` and `wasm32-unknown-unknown` are
named in SC-008 because "the gate is green, including the core still compiling for WebAssembly" is
how the constitution defines green. Specs 055 and 080 tick the same two items the same way. The line
the spec does hold is the one that matters: it takes no decision, and it says nothing about how any
of it is typed.

Two further things sit close to the line between requirement and design, and both are named in the
spec rather than left to be discovered at review:

- **B4's reader is justified by read-before-replace, not by the demonstration.** The risk this slice
  carries is that the first public query of the crate is also the first surface that cannot be
  removed without breaking a caller. An earlier draft of this note said the spec justified it
  because the shipped demonstration needs it; the spec never said that, and the demonstration does
  not — B5 scenario 9 keeps it naming `#1` by hand, as it does today. What justifies the reader is
  the general argument in the spec's **Input**: a caller holding only an identity cannot displace or
  replace a figure without reading it back first. B4 scenario 3 rules out the listing that would
  make it a general reader instead, and the direction that is genuinely missing — getting a
  diagram's identities back, by listing them or by asking a position which shape decided it — is
  what the spec's **What this slice does not decide** names, with issue #86 settling it.
- **Two rules have no test behind them.** Removing a shape leaves references unresolved, and
  displacing a reference reaches its offsets. No figure can hold a reference yet, so neither is
  reachable from a test. Both are named as accepted with nothing to verify them, per principle IV,
  rather than described as tested.

This template has no **Assumptions** section, and none is missing: the assumptions are the four
amended model sections and the decision sheet under **What this slice implements**, and the five
open questions under **What this slice does not decide**, each with what would force an answer. The
dependencies are issue #62, issue #80, and the `docs/diagram-model.md` sections named there.

One item is not satisfiable as written: **`spec.md` exceeds the 120-line ceiling in principle
VIII**, at 288 attributable lines against a ceiling of 120 — 298 lines in the file, less the four
fences and the two labels, measured the way feature 055's plan measured its own 166. That is a fact
about the slice's size, not a formatting slip, and the answer the constitution asks for first is to
split the feature. Splitting was offered and refused in the same session that fixed the scope, so
the ceiling is recorded here and `plan.md`'s Complexity Tracking carries it at plan time, the way
feature 055's does. What the extra lines buy: B5's four-picture demonstration, and B3's two
connectors shown rather than described — which is _Show the rendering_ doing what it asks, on the
one item where a silent no-op is indistinguishable from a bug.

Four further questions were put to the maintainer on 2026-09-28 and are recorded under
**Clarifications**; three of the four changed this file as well as the spec:

- **The demonstration grows a fourth picture.** The slice is named for removing and replacing a
  shape, and the shipped run showed neither — it showed a reorder and a displacement. It now shows
  the removal too, appended after the pair that contrasts the other two, so all four pictures are
  about one figure. B5, SC-006 and the demonstration's contract test all changed with it. The
  ceiling above grew by the same change, which is the constitution's own argument for splitting a
  slice that keeps needing more: it was refused, and it is recorded rather than absorbed.
- **The displacement's amount is a constant in the demonstration's code.** Not a field in the
  description format and not a new argument, so a file's picture is unchanged and `assets/demo.json`
  needs no edit. This is what keeps spec 080's FR-019 and ADR-0035 intact.
- **The reader's justification was misstated here, not in the spec.** See the note above; the
  correction is to this file.
- **A figure carries no identity.** The model's _Vocabulary_ row said a diagram's `Shape` holds one;
  the code has held it beside the shape since spec 079, _Changing a diagram_ already said so, and
  B4's reader now states it. The row is corrected on this branch beside the `Delta` row the same
  branch adds, so the model and the code agree before the reader that makes it observable lands.
