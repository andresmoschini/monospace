# Implementation Plan: Ship the Double, Heavy and Light Round tables

**Branch**: `056-ship-the-double-heavy-and-light-round-ta-plan` | **Date**: 2026-09-11 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `/specs/056-ship-the-double-heavy-and-light-round-ta/spec.md`

## Summary

`monospace-glyph-sets` gains three more public functions — `double()`, `heavy()` and `light_round()`
— each returning a `GlyphCatalog` built from its own fifteen-row table via the extension point
feature 054 already built (`GlyphCatalog::from_rules`). No new type, no new crate, no new dependency
and no core change: the shape is exactly `ascii()`'s, copied three times with different data.
Because `double()`, `heavy()` and `light_round()` would otherwise repeat `ascii()`'s row-to-catalog
conversion and its panic message verbatim, that conversion is pulled into one private helper first,
in a structural commit that changes nothing about `ascii()`'s behavior, before the three new tables
are added as a behavioral commit that uses it. `docs/model.md` and `docs/glyph-sets.md` are
corrected to say all four non-Light tables — ASCII, Double, Heavy, Light Round — are carried as
data, closing the gap FR-011 and FR-012 name.

## Technical Context

**Language/Version**: Rust, edition 2024, the toolchain pinned in `rust-toolchain.toml` — unchanged
by this feature.

**Primary Dependencies**: None added. `monospace-glyph-sets` keeps its one existing dependency,
`monospace-core` (a workspace path dependency); nothing from crates.io is introduced, so the
dependency-freshness rule has nothing to check.

**Storage**: N/A.

**Testing**: `cargo test --workspace` — one unit test per new table in `monospace-glyph-sets`,
mirroring `ascii_answers_every_non_empty_combination_of_its_own_stroke` and
`a_box_rendered_with_ascii_alone_uses_only_ascii_box_characters`, plus one test that a catalog built
from two or more of the four single-stroke tables answers a key from each regardless of order
(FR-008, SC-006). No existing test changes.

**Target Platform**: Unchanged — `monospace-glyph-sets` already compiles for
`wasm32-unknown-unknown` and is already in the gate's `wasm` step (`xtask/src/main.rs:132`); adding
data to it needs no change there.

**Project Type**: Rust cargo workspace — this feature touches one existing library crate and two
docs; no new crate, no CLI change.

**Performance Goals**: N/A — not a concern this feature touches.

**Constraints**: Every diagram rendered before this feature renders byte-identically after it
(SC-007) — the CLI's shipped demo and default catalog are untouched (FR-010). Each of the three new
tables, used alone, produces no character outside its own fifteen glyphs and space (SC-005).

**Scale/Scope**: Three `const` row tables of fifteen rows each, three public functions, one private
helper extracted from the existing `ascii()`, a handful of new unit tests, two corrected documents.
No new crate, no new type, no CLI wiring.

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

- **I. Process over product**: followed — Spec Kit stages, no shortcut taken; the spec's own
  Assumptions already record that no ADR and no new crate are needed, and this plan does not reopen
  either.
- **II. Demonstrable increments**: the three user stories are already independently testable per the
  spec (P1 Double, P2 Heavy, P3 Light Round); each lands as its own green commit or commit group,
  and an appended `docs/learning-log.md` entry closes the increment.
- **III. One definition of green**: no gate change at all — the `wasm` step already names
  `monospace-glyph-sets`, so there is nothing to widen and nothing to prove by disabling first.
- **IV. Claims are measured, not assumed**: SC-001 through SC-007 are run, not asserted, at
  implementation and review time.
- **V. Structural vs. behavioral**: extracting the shared row-to-catalog helper out of `ascii()`
  changes no test and no output, so it is a `refactor` commit on its own, strictly before the `feat`
  commit(s) that add `double()`, `heavy()` and `light_round()` using that helper. The two kinds of
  change do not share a commit.
- **VI. Decisions recorded when taken**: nothing here rises to an ADR — the spec's Assumptions
  already say so, and this plan takes no decision beyond the shape `research.md` records (the shared
  helper), which is cheap to change later and carries no consequence beyond this one crate.
- **VII. The core stays portable**: unchanged — no line of `monospace-core` is touched by this
  feature, and the new code adds no CLI or terminal assumption.

**Constraints and Dependencies**: no new dependency, so nothing to date-check. Testing: unit tests
are added in `monospace-glyph-sets`, meeting principle III's and the constitution's Testing
paragraph's minimum. Scope: this feature adds no crate and stays inside the constitution's _In scope
for this phase_ (v1.5.0); nothing here proposes anything from _Out of scope_.

No violations. Complexity Tracking is left empty.

## Project Structure

### Documentation (this feature)

```text
specs/056-ship-the-double-heavy-and-light-round-ta/
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
└── monospace-glyph-sets/
    └── src/lib.rs        # gains a shared row-to-catalog helper (refactor), then DOUBLE, HEAVY
                           # and LIGHT_ROUND row tables and double(), heavy(), light_round()
                           # (FR-001..FR-008)

docs/
├── model.md              # _Strokes, glyph sets and the catalog_ corrected (FR-011)
└── glyph-sets.md          # opening note corrected (FR-012)
```

**Structure Decision**: No new crate and no new top-level directory. Everything lands in the one
file `monospace-glyph-sets/src/lib.rs` already holds, beside `ascii()`, following the same pattern
ADR-0036 and feature 054 already established; the two documents corrected are the same two feature
054 corrected for ASCII and Light.

## Complexity Tracking

_No violations to justify._
