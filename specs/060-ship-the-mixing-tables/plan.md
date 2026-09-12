# Implementation Plan: Ship the mixing tables

**Branch**: `060-ship-the-mixing-tables-plan` | **Date**: 2026-09-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/060-ship-the-mixing-tables/spec.md`

## Summary

`monospace-glyph-sets` gains four more public functions — `mixing_light_and_double()`,
`mixing_light_and_heavy()`, `mixing_light_round_and_double()` and `mixing_light_round_and_heavy()` —
each built through the existing `build(table, rows)` helper feature 056 already extracted, from a
new private `const` row table copied verbatim from the matching section of `docs/glyph-sets.md`. No
new type, no new crate, no new dependency and no `monospace-core` change: the shape is exactly the
five existing tables' shape, with rows that happen to name two strokes instead of one.

The CLI's shipped demonstration (`description.rs`'s `Description::render`) adds the four new
functions to its existing `GlyphCatalog::union([...])` call, and its fixture (`assets/demo.json`) is
corrected: two of the six overlapping-box pairs feature 056 added pair a stroke combination outside
the four mixing tables (Light with Light Round, and Heavy with Double); those two are changed, by
`stroke` field alone, to Light Round with Double and Light Round with Heavy, so every one of the
four mixing pairs is exercised by a covered combination (FR-009). The existing CLI test asserting
today's degraded crossings is updated in the same commit, since the demonstration's output at those
positions changes by design.

## Technical Context

**Language/Version**: Rust, edition 2024, the toolchain pinned in `rust-toolchain.toml` — unchanged
by this feature.

**Primary Dependencies**: None added. `monospace-glyph-sets` keeps its one existing dependency,
`monospace-core` (a workspace path dependency); nothing from crates.io is introduced, so the
dependency-freshness rule has nothing to check.

**Storage**: N/A.

**Testing**: `cargo test --workspace` — one unit test per new table in `monospace-glyph-sets`
mirroring the existing `*_answers_every_non_empty_combination_of_its_own_stroke` and
`a_box_rendered_with_*_alone_uses_only_*_box_characters` pattern (adapted: a mixing table's coverage
is checked against the rows it publishes rather than "every non-empty combination", since two of the
four are incomplete by design), one test per incomplete table that an uncovered combination still
degrades (SC-005), one test that all nine tables now shipped resolve every key without collision
regardless of union order (FR-007), and an update to `monospace-cli`'s
`crossings_between_the_new_tables_degrade_to_whichever_figure_is_in_front` test so its six
assertions match the corrected demonstration's actual output. No other existing test changes.

**Target Platform**: Unchanged — `monospace-glyph-sets` already compiles for
`wasm32-unknown-unknown` and is already in the gate's `wasm` step; adding data to it needs no change
there.

**Project Type**: Rust cargo workspace — this feature touches one existing library crate
(`monospace-glyph-sets`), one existing binary crate's fixture and source (`monospace-cli`), and two
docs; no new crate.

**Performance Goals**: N/A — not a concern this feature touches.

**Constraints**: A diagram rendered before this feature that touches none of the four mixing tables'
keys renders byte-identically after it (edge case, spec). Each of the four new tables, combined with
the two single-stroke tables its rows draw on, answers exactly the rows it publishes and degrades
everything else exactly as before (FR-006). The demonstration's output differs from before this
feature at exactly the positions where two of its shapes overlap on a combination a mixing table
covers, and nowhere else (SC-006).

**Scale/Scope**: Four `const` row tables (18 + 50 + 18 + 50 = 136 rows total), four public functions
reusing the existing `build` helper unchanged, a two-line fixture correction, a one-line catalog
correction, a handful of new unit tests, one updated CLI test, two corrected documents. No new
crate, no new type, no new CLI argument.

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

- **I. Process over product**: followed — Spec Kit stages, no shortcut taken; the spec's own
  Assumptions already record that no ADR and no new crate are needed, and this plan does not reopen
  either.
- **II. Demonstrable increments**: the four user stories are independently testable per the spec (P1
  Light+Heavy, P2 Light+Double, P3 Light Round+Heavy and Light Round+Double); each lands as its own
  green commit or commit group, and an appended `docs/learning-log.md` entry closes the increment.
- **III. One definition of green**: no gate change — the `wasm` step already names
  `monospace-glyph-sets`, so there is nothing to widen and nothing to prove by disabling first.
- **IV. Claims are measured, not assumed**: SC-001 through SC-007 are run, not asserted, at
  implementation and review time; the demonstration's corrected crossing positions are checked
  against the published tables before the corresponding test is updated, not assumed to land where
  intended.
- **V. Structural vs. behavioral**: no structural change is needed this time — `build` already
  exists and already takes rows of any length, so nothing is extracted or reshaped before the new
  tables are added. Adding the four tables, wiring them into the CLI's catalog, correcting the
  fixture, and updating the CLI test that documents today's degraded behavior are all one behavioral
  change working toward the same visible outcome (the demonstration mixing where it used to
  degrade); they are not split from each other, but they are one `feat` commit or commit group,
  never mixed with a `refactor`.
- **VI. Decisions recorded when taken**: nothing here rises to an ADR — the spec's Assumptions
  already say so, and this plan takes no decision beyond the shape `research.md` records (four
  functions of the existing shape, and which two demo placements move), which is cheap to change
  later and carries no consequence beyond this one crate and one fixture.
- **VII. The core stays portable**: unchanged — no line of `monospace-core` is touched by this
  feature, and the new code adds no CLI or terminal assumption; `monospace-cli`'s own change is
  wiring an existing union, not new domain logic.

**Constraints and Dependencies**: no new dependency, so nothing to date-check. Testing: unit tests
are added in `monospace-glyph-sets` and one existing test is updated in `monospace-cli`, meeting
principle III's and the constitution's Testing paragraph's minimum. Scope: this feature adds no
crate and stays inside the constitution's _In scope for this phase_ (v1.5.0); nothing here proposes
anything from _Out of scope_.

**A scope note beyond the spec's FR list**: `research.md` and `data-model.md` also correct one
sentence each in `docs/glyph-sets.md` and `docs/model.md` that becomes false once the mixing sets
ship as data (they currently say those sets are reference-only, loaded from a file). FR-010 scopes
this feature's functional requirements to the four tables and the demonstration change "and no
other," but says nothing about a document that describes what already exists becoming inaccurate as
a side effect. Principle IV and the precedent features 054 and 056 both set (each corrected the same
two documents for the tables it shipped) call for the same one-sentence correction per document
here; it is called out here rather than folded in silently because the spec, unlike 054's and 056's,
does not name it as an FR.

No violations against the constitution itself. Complexity Tracking is left empty.

## Project Structure

### Documentation (this feature)

```text
specs/060-ship-the-mixing-tables/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
└── contracts/
    └── glyph-tables.md
```

### Source Code (repository root)

```text
crates/
├── monospace-glyph-sets/
│   └── src/lib.rs        # gains MIXING_LIGHT_AND_DOUBLE, MIXING_LIGHT_AND_HEAVY,
│                          # MIXING_LIGHT_ROUND_AND_DOUBLE, MIXING_LIGHT_ROUND_AND_HEAVY row
│                          # tables and their four functions (FR-001..FR-007), via the
│                          # existing `build` helper — unchanged
└── monospace-cli/
    ├── assets/demo.json  # two box pairs corrected to Light Round+Double and
    │                      # Light Round+Heavy (FR-009)
    ├── src/description.rs # catalog union gains the four new functions (FR-008)
    └── tests/cli.rs       # crossings test updated for the six now-mixed positions

docs/
├── model.md              # _Strokes, glyph sets and the catalog_ corrected
└── glyph-sets.md          # opening note corrected
```

**Structure Decision**: No new crate and no new top-level directory. The four tables land in the one
file `monospace-glyph-sets/src/lib.rs` already holds, beside the five existing tables, following the
same pattern ADR-0036, feature 054 and feature 056 already established; the CLI's fixture, its one
source file and its test file are the only other files touched, and the two documents corrected are
the same two features 054 and 056 corrected for their own tables.

## Complexity Tracking

_No violations to justify._
