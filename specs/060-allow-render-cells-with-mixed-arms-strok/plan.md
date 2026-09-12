<!-- The branch and directory names are truncated by the tooling that creates them; "strok" is
     that truncation, not a word. cspell:ignore strok -->

# Implementation Plan: Allow render cells with mixed arms' strokes

**Branch**: `060-allow-render-cells-with-mixed-arms-strok-plan` | **Date**: 2026-09-12 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `/specs/060-allow-render-cells-with-mixed-arms-strok/spec.md`

## Summary

[ADR-0037](../../docs/decisions/0037-give-each-arm-its-own-stroke.md) already chose the shape: `Arm`
gains `Set(Stroke)`, ending the one-stroke-per-cell restriction
[ADR-0012](../../docs/decisions/0012-one-stroke-per-cell.md) recorded as temporary.
`StrokeCell::key()` stops reading `base` for every `Set` side and reads each arm's own stroke
instead. That alone changes nothing visible, because
[ADR-0009](../../docs/decisions/0009-degrade-a-cell-to-its-base-stroke.md)'s second lookup — degrade
every connected arm to the base stroke and try again — was never actually written: under ADR-0012
the one lookup already produced the degraded key, so `Cell::glyph_str` has only ever done one. This
plan adds that missing second lookup next to `key()`, then ships the four mixing tables
`docs/glyph-sets.md` already records as data in `monospace-glyph-sets`, then wires them into the
command-line demonstration's catalog — the only one of the three steps with a visible effect, since
a mixed key finds nothing to match until a mixing table is loaded. Three behavioral commits, each
green on its own, no new type, no new crate, no new dependency, and one correction each to
`docs/glyph-sets.md` and `docs/model.md` for a sentence the second step makes stale.

## Technical Context

**Language/Version**: Rust, edition 2024, the toolchain pinned in `rust-toolchain.toml` — unchanged
by this feature.

**Primary Dependencies**: None added. `monospace-core`, `monospace-glyph-sets` and `monospace-cli`
keep the dependencies they have; nothing from crates.io is introduced, so the dependency-freshness
rule has nothing to check.

**Storage**: N/A.

**Testing**: `cargo test --workspace`. `monospace-core` gains tests for the two-lookup render path
ADR-0009 described but never pinned (exact key wins; falls back to the base-stroke key; falls back
to a space when neither matches) and for per-arm-stroke key construction. `monospace-glyph-sets`
gains a row-count and a spot-check test per new table, mirroring the existing per-table pairs for
ASCII/Double/Heavy/Light Round, plus the existing union-order-independence test extended from four
tables to all nine. No test is added for the CLI's demonstration output — FR-014/SC-003 are accepted
on observation, not by test, exactly as the spec names them.

**Target Platform**: Unchanged. `monospace-core` and `monospace-glyph-sets` already compile for
`wasm32-unknown-unknown` and are already in the gate's `wasm` step; an owned `Stroke` per arm and
four more `const` tables need nothing new there.

**Project Type**: Rust cargo workspace — this feature touches two library crates (`monospace-core`,
`monospace-glyph-sets`) and one non-interactive CLI consumer (`monospace-cli`); no new crate.

**Performance Goals**: N/A. ADR-0037 already accepts the cost of four more owned-`String`
allocations per connected cell as a consequence, worth measuring only if rendering is ever reported
slow.

**Constraints**: The shipped demo file (`crates/monospace-cli/assets/demo.json`) MUST NOT change
(FR-015); the visible difference comes only from rendering and catalog logic. Every diagram that
mixes no strokes MUST render byte-identically before and after the feature (SC-006). No two of the
nine tables `monospace-glyph-sets` ships MUST define the same key (FR-011).

**Scale/Scope**: One enum variant gains a payload (~112 construction sites across `monospace-core`,
all mechanical); one new lookup step in `cell.rs`; four `const` row tables (18, 50, 18, 50 rows) and
four public functions in `monospace-glyph-sets`; one `GlyphCatalog::union` call in `monospace-cli`
grows by four entries; two documents corrected. No CLI flag, no new single-stroke table, no change
to any demo figure beyond the characters at its existing crossings (FR-016).

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

- **I. Process over product**: followed. The spec's own Assumptions already settle that no new ADR
  and no new crate are needed and that the interface shape is this plan's job; this plan does not
  reopen either, and takes the slower, three-commit route over the one-commit shortcut of just
  shipping the tables — the route the reverted first attempt at this issue took and that ADR-0037
  explains does not work.
- **II. Demonstrable increments**: the three commits below are the increments. The first two are
  provably behavior-preserving on their own (a mixed exact key cannot match a single-stroke table,
  so it always falls through to the unchanged degrade result) and are demonstrated by the growing
  test suite; the third is where `cargo run -p monospace-cli`'s output actually changes, and that
  observation is what closes the increment in `docs/learning-log.md`.
- **III. One definition of green**: no gate change. `wasm` already names both crates this feature
  touches; nothing is widened and nothing needs disabling first.
- **IV. Claims are measured, not assumed**: SC-001 through SC-007 are run, not asserted. FR-014 and
  SC-003 stay named as accepted on observation, per the spec, rather than described as tested.
- **V. Structural vs. behavioral**: all three commits are `feat`. None is a `refactor`: the first
  changes every test that constructs an `Arm::Set` (a `Copy` enum variant gaining a payload cannot
  be done without touching them), the second adds tests alongside the new tables, the third changes
  what the demonstration draws. None of the three is mixed with a structural-only change.
- **VI. Decisions recorded when taken**:
  [ADR-0037](../../docs/decisions/0037-give-each-arm-its-own-stroke.md) already records the one
  decision this feature depends on. Nothing here is expensive enough to undo, or contested enough,
  to need a further ADR — including the naming of the four new functions, which follows the existing
  `double()`/`heavy()`/`light_round()` convention rather than choosing one.
- **VII. The core stays portable**: unchanged. `Stroke` stays a plain `String`
  ([ADR-0015](../../docs/decisions/0015-represent-a-stroke-as-a-string.md)); no CLI or terminal
  assumption enters `monospace-core`, and the CLI's only change is which catalogs it unions.

**Constraints and Dependencies**: no new dependency, so nothing to date-check against the
constitution's or the organization's dependency-freshness rule. Testing: unit tests are added in
both `monospace-core` and `monospace-glyph-sets`, meeting principle III's and the constitution's
Testing paragraph's minimum. Scope: this feature adds no crate, no CLI surface and no export format,
staying inside the constitution's _In scope for this phase_ (v1.5.0); nothing here proposes anything
from _Out of scope_.

No violations. Complexity Tracking is left empty.

### Post-design re-check

Phase 1 changes nothing about the shape above: the degrade step is additional logic behind the
existing `Cell::glyph_str`, the four tables are data following an established pattern, and the CLI
change is one line in an existing list. The gate re-check finds nothing new to justify.

## Project Structure

### Documentation (this feature)

```text
specs/060-allow-render-cells-with-mixed-arms-strok/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md         # Phase 1 output
├── quickstart.md        # Phase 1 output
└── contracts/
    └── mixed-arm-rendering.md
```

### Source Code (repository root)

```text
crates/
├── monospace-core/
│   └── src/
│       ├── cell.rs             # Arm::Set(Stroke); StrokeCell::key() reads per-arm strokes; the
│       │                       # missing degrade-key step is added here (FR-001..FR-004)
│       ├── render.rs           # Cell::glyph_str consults the new degrade step (via cell.rs)
│       ├── buffer.rs           # Arm::Set construction sites in stamping tests
│       └── shape/
│           ├── line.rs                # Arm::Set construction sites
│           └── fragment/
│               ├── segment.rs         # Arm::Set construction sites
│               ├── corner.rs          # Arm::Set construction sites
│               ├── border.rs          # Arm::Set construction sites
│               └── end.rs             # Arm::Set construction sites
├── monospace-glyph-sets/
│   └── src/
│       └── lib.rs               # LIGHT_DOUBLE, LIGHT_HEAVY, LIGHT_ROUND_DOUBLE,
│                                 # LIGHT_ROUND_HEAVY row tables and light_double(),
│                                 # light_heavy(), light_round_double(), light_round_heavy()
│                                 # (FR-006..FR-011)
└── monospace-cli/
    └── src/
        └── description.rs       # Description::render()'s GlyphCatalog::union([...]) gains the
                                  # four mixing catalogs (FR-013); assets/demo.json untouched
                                  # (FR-015)

docs/
├── model.md              # _Strokes, glyph sets and the catalog_ corrected: the four mixing sets
│                          # are no longer described as file-loaded only
└── glyph-sets.md          # opening note corrected (FR-012)
```

**Structure Decision**: No new crate, no new top-level directory. The per-arm stroke and the degrade
step land in `monospace-core`'s existing `cell.rs`, beside `StrokeCell::key()`; the four tables land
in `monospace-glyph-sets/src/lib.rs` beside `ascii()`/`double()`/`heavy()`/ `light_round()`,
following the pattern ADR-0036 and feature 056 established; the CLI change is one list in the one
function `description.rs` already has. The two documents corrected are the model and the glyph-sets
reference, the same two feature 054 and feature 056 corrected before.

## Complexity Tracking

_No violations to justify._
