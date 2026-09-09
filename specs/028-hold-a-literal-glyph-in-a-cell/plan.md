# Implementation Plan: Hold a literal glyph in a cell

**Branch**: `028-hold-a-literal-glyph-in-a-cell` | **Date**: 2026-09-09 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `/specs/028-hold-a-literal-glyph-in-a-cell/spec.md`

## Summary

Turn `Cell` from a struct into a sum of two kinds — the stroke cell it is today, and one literal
`Glyph` — so a position can draw a chosen character that nothing connects into and that hides what
is behind it. Then spend it: the front end fills its box interiors, and the overlapping pair on the
terminal shows occlusion next to the junctions it already showed.

Three findings from Phase 0 shape the plan, and all three make the change smaller than the input
draft ([spec 0005](../../docs/specs/0005-hold-a-literal-glyph-in-a-cell.md)) implies:

- **Most of the composition already exists.** A literal is decided on all four sides, so the two
  `is_decided` shortcuts in `Buffer::stamp` —
  [ADR-0017](../../docs/decisions/0017-ask-the-cell-whether-it-is-decided.md) and
  [ADR-0018](../../docs/decisions/0018-mirror-the-decided-skip-in-above.md) — already produce three
  of the model's five rows. The only new work inside the merge is a literal in the _bottom_ role,
  where it contributes four `Closed` sides. See [research.md](research.md), R2 and R3.
- **The renderer needs a lifetime change, not an allocation.** A literal's text is borrowed from the
  buffer while every other glyph is borrowed from the catalog, so the two borrows have to meet in
  one lifetime. `render`'s public signature does not move and nothing is cloned per cell. See
  [research.md](research.md), R4.
- **The rename can leave every test untouched.** A one-commit type alias lets the structural commit
  rename the struct without editing a single test, which is what principle V asks for literally
  rather than by interpretation. See the Constitution Check below and [research.md](research.md),
  R7.

**The one decision this plan surfaced is taken, and its record is the first task.** The sum type is
expensive to undo and someone will ask why it is not a field, so it needs an ADR by principle VI's
own test. The maintainer agreed this plan on 2026-09-09 and chose the sum, on a rule that is wider
than this feature: as far as possible, the model should make invalid states impossible to represent.
ADR-0026 records that before the first commit that depends on it, and [research.md](research.md), R1
holds the material it draws on.

## Technical Context

**Language/Version**: Rust 1.98.1, edition 2024, pinned exactly in `rust-toolchain.toml`
([ADR-0003](../../docs/decisions/0003-pin-the-toolchain-exactly.md)). No nightly features.

**Primary Dependencies**: none added. `unicode-segmentation` is already a dependency of
`monospace-core` from feature 006, and this feature reuses `Glyph` rather than validating anything
itself.

**Storage**: N/A. A buffer is a working surface and nothing is persisted.

**Testing**: `cargo test --workspace`, run by `cargo xtask check`. Unit tests live in
`#[cfg(test)] mod tests` inside the module they cover; the front end's whole output is asserted from
`crates/monospace-cli/tests/cli.rs`, which runs the built binary as a subprocess.

**Target Platform**: platform-independent library. The gate compiles `monospace-core` for
`wasm32-unknown-unknown`, which is what enforces
[The core stays portable](../../.specify/memory/constitution.md).

**Project Type**: Rust library (`monospace-core`) with a thin non-interactive consumer
(`monospace-cli`). Both are touched: the library gains the kind of cell, the consumer spends it.

**Performance Goals**: none for this slice. One point is worth keeping in view rather than
measuring: a literal is decided, so the front-to-back walk stops at it, which is the cheap direction
getting cheaper. Nothing in this feature is timed.

**Constraints**: the core's public API carries no terminal assumption; `Glyph`'s invariant is not
widened or narrowed here; the rendered output of a diagram containing no literal does not move
(FR-004).

**Scale/Scope**: one type renamed, one sum type introduced, two public items added (`Cell::Literal`,
`From<StrokeCell> for Cell`), one public method widened (`Cell::is_decided`), 33 construction sites
across four files — 32 of them in test modules, one in the front end.

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._ Re-checked after Phase 1;
both passes reached the same result.

| Principle                                       | Verdict | Why                                                                                                                                                                                                                                                                                                               |
| ----------------------------------------------- | ------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| I. Process over product                         | PASS    | The route taken is the slower one twice: an ADR before any code, and a structural commit that could have been folded into the behavioral one. Both are there so the second commit's diff is what a literal actually costs.                                                                                        |
| II. Demonstrable increments                     | PASS    | Each commit leaves the gate green and `cargo run -p monospace-cli` producing output. The increment ends with the fill on the terminal, and then with one appended entry in `docs/learning-log.md`.                                                                                                                |
| III. One definition of green                    | PASS    | No check is added, moved or configured. `cargo xtask check` is untouched by this feature.                                                                                                                                                                                                                         |
| IV. Claims are measured                         | PASS    | Every picture in the input draft is derived rather than observed, and the draft says so. The plan treats them as predictions: the CLI assertion is written from a real run, and the two `is_decided` shortcuts are shown to be optimizations by deleting each and re-running. See [quickstart.md](quickstart.md). |
| V. Structural and behavioral never share commit | PASS    | Two commits, and the structural one edits no test at all. See the note below.                                                                                                                                                                                                                                     |
| VI. Decisions recorded when taken               | PASS    | The maintainer took the decision on the plan rather than on the code; ADR-0026 records it as the first task, before anything depends on it. The plan gathered the options and did not choose among them.                                                                                                          |
| VII. The core stays portable                    | PASS    | The kind of cell lands in `monospace-core`; the front end gains stamps, not logic. Nothing new names a terminal, and the gate's `wasm` step compiles the crate as before.                                                                                                                                         |
| Constraints — language of the artifacts         | PASS    | Every artifact of this feature is in English; the conversation that produced it was not.                                                                                                                                                                                                                          |
| Constraints — dependencies                      | PASS    | None is added, so the seven-day rule has nothing to apply to. `Glyph` and its dependency arrived with feature 006.                                                                                                                                                                                                |
| Constraints — testing                           | PASS    | Every functional requirement has a success criterion naming a test: FR-001 to FR-003 and FR-005 to FR-007 through SC-003, FR-008 through SC-004, FR-004 and FR-012 through SC-005, FR-010 through SC-001 and SC-002.                                                                                              |
| The model owns the design                       | PASS    | The five rows of _Stamping_ and the sentence in _Rendering_ are implemented as they stand. [data-model.md](data-model.md) traces each row to what the code does; it does not restate the rule.                                                                                                                    |

**On principle V, and how the rename avoids interpreting it.** Renaming `Cell` to `StrokeCell`
touches 33 construction sites, 32 of them inside test modules. Done directly, the structural commit
would have to edit tests to compile, and the principle says a structural commit modifies no test —
so the plan would be asking for an exception on its first commit.

It does not need one. The structural commit renames the struct and leaves
`pub type Cell = StrokeCell;` behind it, so every call site, tests included, compiles unchanged: a
type alias to a struct works in struct-literal expressions and in patterns. The behavioral commit
then takes the name `Cell` for the sum, deletes the alias, and carries the mechanical
`StrokeCell { .. }.into()` edits along with the new behavior — which a `feat` is allowed to do,
since the call sites its own signature change forces are part of that change rather than a refactor
riding along. That reading is the one feature 006's plan already established for the same situation.

The cost is a public type alias that exists for exactly one commit. It is worth the commit boundary
staying literal rather than argued.

**There is no foundational work.** The ADR is not foundational work in the plan's sense — it is a
document, not a step of the build — and everything else depends on the sum type existing, so there
is nothing that could usefully run before it.

## Project Structure

### Documentation (this feature)

```text
specs/028-hold-a-literal-glyph-in-a-cell/
├── spec.md              # /speckit-specify output
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── contracts/
│   └── public-api.md    # Phase 1 output
├── checklists/
│   └── requirements.md  # Spec quality checklist
└── tasks.md             # /speckit-tasks output — not created here

docs/decisions/
└── 0026-...md           # Written before the first commit that depends on it
```

### Source Code (repository root)

```text
crates/monospace-core/
└── src/
    ├── cell.rs          # StrokeCell, the Cell sum, From<StrokeCell>, is_decided widened
    ├── buffer.rs        # merge gains the literal-as-bottom case; stamp itself is untouched
    ├── render.rs        # One branch for a literal; key_of narrows to a stroke cell
    ├── lib.rs           # Re-exports StrokeCell alongside Cell
    ├── glyph.rs         # Untouched
    ├── geometry.rs      # Untouched
    └── stroke.rs        # Untouched

crates/monospace-cli/
├── src/main.rs          # The box gains two interior stamps
└── tests/cli.rs         # The asserted output grows the fill; written from a real run

docs/
├── model.md             # Untouched: this feature implements it as it stands
└── learning-log.md      # One appended entry at the end of the increment
```

**Structure Decision**: the composition rules stay in `buffer.rs`, where `merge` already lives, and
the kinds of cell stay in `cell.rs`. Moving the merge next to the type it merges would be a
structural commit this feature does not need, and it stays available afterwards at no cost to any
caller, since both functions are private.

`StrokeCell` keeps the name the input draft gave it, rather than following the row labels in
_Stamping_ ("Arms", "A literal"). Confirmed by the maintainer, and for a better reason than the
draft being first: the arms are the low-level device this model uses to represent strokes meeting at
a position, and what the layer above cares about is the strokes. See [research.md](research.md), R5.

No test file is added. The new cases go into the `#[cfg(test)] mod tests` of `buffer.rs`, `cell.rs`
and `render.rs`, next to what they cover, and the front end keeps its single end-to-end assertion.

## Complexity Tracking

Empty, deliberately. The Constitution Check has no violations to justify: the one place this feature
looked like it needed an exception — a structural commit editing tests to compile — is solved by a
one-commit type alias instead of by a justification. Recording a departure here that the plan does
not take would leave a reader looking for a problem that was designed out.
