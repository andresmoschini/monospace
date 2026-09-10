# Implementation Plan: Draw shapes instead of individual cells

**Branch**: `039-draw-shapes-instead-of-individual-cells` | **Date**: 2026-09-10 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `/specs/039-draw-shapes-instead-of-individual-cells/spec.md`

This is the second plan for feature 039. The first was reviewed rather than agreed, and the review
produced [ADR-0028](../../docs/decisions/0028-give-each-fragment-its-own-cell-rule.md),
[ADR-0029](../../docs/decisions/0029-draw-a-line-end-as-one-arm.md) and
[ADR-0030](../../docs/decisions/0030-drop-extent-until-a-caller-needs-it.md), an amended model and
an amended spec. Those are inputs here, not decisions to retake.

## Summary

Add the layer directly above the buffer: a caller describes a figure and the figure draws itself.
Three complete shapes — a box, a straight line, an arrow between two directed endpoints — built out
of six crate-private fragments that each own one cell rule, plus a compositor that turns a derived
route into corners and segments. A shape draws into a `Surface`, a trait with one write and no
reader, so a fragment structurally cannot inspect the buffer and the write counter FR-020 needs is a
second implementation in a test module rather than a change to `Buffer`.

Nothing below the buffer changes. `Buffer`, `stamp`, `Cell`, the renderer and `docs/glyph-sets.md`
are untouched, and `monospace-cli` renders byte for byte what it renders today.

## Technical Context

**Language/Version**: Rust, edition 2024, exact version pinned in `rust-toolchain.toml`. No
nightly-only features.

**Primary Dependencies**: none. This feature adds no dependency, so no publication date needs
verifying. Domain logic prefers the standard library, and everything here is domain logic.

**Storage**: N/A.

**Testing**: `cargo test --workspace`. Unit tests in `crates/monospace-core/src/shape/`, next to
what they cover; `crates/monospace-cli/tests/cli.rs` unchanged, as the confirmation that the final
refactor changed no output.

**Target Platform**: `monospace-core` compiles for the host and for `wasm32-unknown-unknown`,
checked by the gate's `wasm` step.

**Project Type**: Rust workspace — one library crate and one CLI consumer. All of this feature lands
in the library.

**Performance Goals**: none declared, and none measured. Nothing in this phase has a performance
requirement, and the route search is bounded by a lattice of at most nine points rather than by the
distance between the endpoints — research Q5.

**Constraints**: `monospace-core` assumes no terminal, command line or user interface (FR-025).
`clippy::pedantic` runs at `-D warnings`. Every public item carries rustdoc as it is introduced.

**Scale/Scope**: three public figures, six crate-private fragments, one compositor, two traits, one
adapter, three small enums. Roughly a dozen new files under `crates/monospace-core/src/shape/`.

## Preparatory work

The plan was invoked with the observation that the plan and the model had moved, and that
preparatory changes might be needed before the figures. They are, and they are documents rather than
code. Research Q1 and the _Preparatory work_ section of [`research.md`](research.md) hold the
detail.

**No structural change to existing code is needed, and none is written.** Everything above the
buffer is new. `cell.rs` and `geometry.rs` each gain an enum and lose nothing; the rest of the core
is untouched. The existing core already matches the amended model — verified by reproducing
`monospace-cli`'s twelve hand-written stamps from the box's decomposition, cell for cell. _One
definition of green_'s removal test applies to a preparatory refactor as much as to a lint entry:
one that cannot be shown to be needed is not written, and saying so is the honest finding rather
than an omission.

**Two documents are behind decisions already taken, and both land before the first line of shape
code.**

1. **ADR-0031, what a shape draws into.** "A shape draws into a surface, not into a buffer" is
   recorded today only in the spec's _Handoff to the plan_. _Decisions recorded when taken_ says a
   decision is an ADR at the moment it is taken and that a spec MUST NOT take one, so this is a
   record owed rather than a new decision. It carries the trait's shape from research Q1 — one
   write, no reader, the stamp mode bound by a `Layer` rather than passed per stamp — and the
   dispatch choice from Q3.
2. **The model amendment.** `docs/model.md` still says a shape draws into a buffer, in three places:
   the _Vocabulary_ row for `Shape`, the second paragraph of _Shapes_, and the last sentence of the
   fragment paragraph in _Complete and fragment_. `Surface` has no row at all. _The model owns the
   design_ requires the model change first.

Order: the record, then the model, then the code. Neither widens scope; both are the artifacts the
decisions already taken were supposed to land in.

## Constitution Check

_GATE: passed before Phase 0 research, re-evaluated after Phase 1 design. No violation to justify;
Complexity Tracking is empty._

| Principle                                                | How this plan satisfies it                                                                                                                                                                                                                                                            |
| -------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| I. Process over product                                  | The slower route is taken twice on purpose: six fragments rather than one parametrized leaf (ADR-0028), and two documents written before any code rather than after it. Neither is the fastest way to a box on screen                                                                 |
| II. Demonstrable increments                              | Three slices, one per user story, each ending in a runnable state; one task per commit; `cargo run -p monospace-cli` is the acceptance command; the increment ends with a `docs/learning-log.md` entry                                                                                |
| III. One definition of green                             | `cargo xtask check` at every commit, through the hook. No check is added by this feature, so the two-commit rule for adding one does not arise. SC-009 also asks for a fresh-clone run                                                                                                |
| IV. Claims are measured, not assumed                     | Every pinned picture is produced by running the code before it is pasted into a test ([`quickstart.md`](quickstart.md)); the box's one guard and the write counter are each made to fail on purpose and restored (SC-004, SC-006); research Q5 is labelled a prediction until it runs |
| V. Structural and behavioral change never share a commit | The figures are `feat`; the CLI's redraw through `BoxShape` is a `refactor` at the end that changes no output and adds no test. The two never share a commit, and no preparatory refactor is manufactured to look like one                                                            |
| VI. Decisions recorded when taken                        | ADR-0031 is written before the code that depends on it — the one thing this plan found outstanding. ADR-0028, 0029 and 0030 already cover the fragments, the ends and the extent. Findings local to this feature stay in `research.md`                                                |
| VII. The core stays portable                             | Everything lands in `monospace-core`; the CLI gains no logic and loses some. No public item names a terminal. `Side` stays `pub(crate)` because no public item reads it. The `wasm` step enforces the boundary                                                                        |

**Testing**, per the constitution's own constraint that each spec sets its expectations: unit tests
for core logic are the minimum, and this spec asks for more — every acceptance picture asserted
exactly, every direction family covered, and two checks verified by being made to fail.

**Dependencies**: none added, so the seven-day publication rule has nothing to check.

**Language**: every artifact and identifier is English.

## Project Structure

### Documentation (this feature)

```text
specs/039-draw-shapes-instead-of-individual-cells/
├── plan.md                    # this file
├── spec.md                    # amended for ADR-0028, 0029 and 0030
├── research.md                # Phase 0: eight questions, and the preparatory work
├── data-model.md              # Phase 1: the fragments, the figures, the decompositions
├── quickstart.md              # Phase 1: how to run and validate each slice
├── contracts/
│   └── public-api.md          # Phase 1: the public API monospace-core gains
├── checklists/
│   └── requirements.md
└── tasks.md                   # /speckit-tasks, not created here
```

### Source code (repository root)

```text
crates/monospace-core/src/
├── lib.rs                     # gains `mod shape;` and the re-exports
├── buffer.rs                  # unchanged
├── cell.rs                    # gains `Side` (pub(crate))
├── geometry.rs                # gains `Direction` and `Orientation` (pub)
├── glyph.rs                   # unchanged
├── render.rs                  # unchanged
├── stroke.rs                  # unchanged
├── shape.rs                   # the `Shape` and `Surface` traits, and `Layer`
└── shape/
    ├── box_shape.rs           # BoxShape
    ├── line.rs                # Line
    ├── arrow.rs               # Arrow, Endpoint, and deriving the path
    ├── route.rs               # Route, the compositor
    └── fragment.rs + fragment/{corner,segment,end,border,fill,head}.rs

crates/monospace-cli/
├── src/main.rs                # stamp_box replaced by BoxShape, in a refactor commit
└── tests/cli.rs               # unchanged, and not edited in that commit

docs/
├── model.md                   # amended: a shape draws into a surface
├── decisions/0031-*.md        # new: what a shape draws into
└── learning-log.md            # appended once, at the end of the increment
```

**Structure Decision**: the existing single-library-plus-CLI workspace, unchanged. The shape layer
is one new module tree inside `monospace-core`, using `shape.rs` beside a `shape/` directory rather
than `shape/mod.rs`, which is the current Rust idiom and matches the flat modules already in the
crate. The fragments sit one level down because their visibility is the thing that makes the
complete/fragment distinction real (research Q2), and a directory of their own makes the
`pub(crate)` boundary visible in the tree rather than only in the source.

## Increments and commits

Four increments. One task per commit; every commit green through the hook.

**0 — the record and the model.** `docs` commits. ADR-0031, then the model amendment for it. No
code.

**1 — the layer, and the box (user story 1, P1).** `Surface`, `Layer`, `Shape`; `Side`, `Direction`,
`Orientation`; the six fragments; `BoxShape`; the counting surface in tests. Ends with the
`refactor` commit that has `monospace-cli` redraw its box through `BoxShape`, with
`crates/monospace-cli/tests/cli.rs` unedited and passing. Demonstrable: `cargo run -p monospace-cli`
prints exactly what it prints today.

**2 — the line (user story 2, P2).** `Line`, on the `End` and `Segment` fragments increment 1
already built. The join test is the one that matters. No existing shape is touched, which is the
first evidence for SC-007.

**3 — the arrow (user story 3, P3).** Deriving the path, `Route`, `Arrow`, `Endpoint`. The largest
increment and the one whose pictures are produced by running rather than by reading. The write
counter is made to fail on purpose here (SC-006), the box's guard in increment 1 (SC-004).

The increment ends with one appended entry in `docs/learning-log.md`.

## Risks

- **The route derivation is the whole of the risk.** Research Q5 is worked by hand against ten
  pinned pictures and no further. If the implementation disagrees with one of them the algorithm is
  wrong, not the spec — the pictures are what the feature owes. The two unpinned rows are where a
  wrong rule would hide, which is why SC-003 requires a test for them anyway.
- **The even-span tie-break is deliberately unpinned**, so an arrow and its reverse may differ by a
  column. That is _Clarifications_, not a defect, and the test records whichever the rule yields.
- **`Surface::stamp` takes a `Cell`**, so the six cell rules guard nothing for a shape written
  outside the crate. ADR-0028 records that hole and the reason it stays open; the contract repeats
  it so a caller meets it.

## Complexity Tracking

No Constitution Check violation. Nothing to justify.
