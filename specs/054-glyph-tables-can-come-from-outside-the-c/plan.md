# Implementation Plan: Glyph tables can come from outside the core

**Branch**: `054-glyph-tables-can-come-from-outside-the-c-plan` | **Date**: 2026-09-11 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `/specs/054-glyph-tables-can-come-from-outside-the-c/spec.md`

## Summary

`monospace-core` gains a small extension point on the type it already has:
`GlyphCatalog::from_rules` builds a catalog from one ordered group of rules, and
`GlyphCatalog::union` merges several catalogs into one, first claim wins, with no way to ask
afterwards which one answered a key. No new type is introduced — a table from outside the core is
simply a `GlyphCatalog` built from its own rows, since a table and a catalog already answer to the
same contract. A new crate, `monospace-glyph-sets`, depends on the core through that public API
alone and holds the ASCII table verbatim from `docs/glyph-sets.md`. The CLI builds its catalog by
union-ing the core's Light table and the new crate's ASCII table, and the shipped demo description
gains ASCII shapes and two crossings — ASCII in front once, Light in front once — so a no-argument
run shows both tables answering from one catalog. `docs/model.md` and `docs/glyph-sets.md` are
corrected first, since the constitution requires the model to match before code relies on it.

## Technical Context

**Language/Version**: Rust, edition 2024, the toolchain pinned in `rust-toolchain.toml` — unchanged
by this feature.

**Primary Dependencies**: None added. `monospace-glyph-sets` depends only on `monospace-core` (a
workspace path dependency); no crate from crates.io is introduced, so the dependency-freshness rule
has nothing to check this time.

**Storage**: N/A.

**Testing**: `cargo test --workspace` — unit tests in `monospace-core` for `from_rules` and
`union`'s ordering and first-claim-wins rule, unit tests in `monospace-glyph-sets` mirroring the
Light table's existing completeness test, and the existing subprocess tests in
`crates/monospace-cli/tests/cli.rs` extended to cover the demo's new ASCII shapes and its two
crossings.

**Target Platform**: The CLI runs natively, as it does today. `monospace-core` and
`monospace-glyph-sets` both compile for `wasm32-unknown-unknown`, extending the gate's existing
`wasm` step to the new crate (FR-010).

**Project Type**: Rust cargo workspace — two libraries and one CLI binary; this feature adds the
second library.

**Performance Goals**: N/A — not a concern this feature touches.

**Constraints**: Every diagram rendered before this feature renders byte-identically after it,
except the shipped demo (FR-005, SC-008). A catalog built from the ASCII table alone produces only
printable ASCII characters or spaces (SC-004).

**Scale/Scope**: One new crate holding fifteen rows of data, a handful of new public items on
`monospace-core`'s existing `glyph` module, one CLI wiring change, one edited demo file, and two
corrected documents.

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

- **I. Process over product**: followed — Spec Kit stages, no shortcut taken.
- **II. Demonstrable increments**: the slice is thin and already bounded by the spec's own user
  stories (P1 extension point, P2 ASCII-only rendering, P3 demonstration). Each lands as its own
  green commit or commit group; an appended `docs/learning-log.md` entry closes the increment.
- **III. One definition of green**: the only gate change is widening the existing `wasm` step to a
  second `-p`, not a new kind of check — nothing to prove by disabling it first.
- **IV. Claims are measured, not assumed**: SC-001 through SC-009 are all run, not asserted, at
  implementation and review time; none of them is recorded as satisfied before that.
- **V. Structural vs. behavioral**: the core's new `GlyphCatalog::from_rules` and
  `GlyphCatalog::union` are additive — existing tests and existing output are unchanged — so they
  land as a `feat` commit (new public surface, new tests) rather than a `refactor`, since a
  `refactor` commit may add no test. The CLI's switch to a union of two catalogs and the demo's new
  shapes are a separate `feat`, kept apart from the extension-point commit.
- **VI. Decisions recorded when taken**: the only architectural decision this feature needs — Light
  stays in the core, everything else moves out — is already
  [ADR-0036](../../docs/decisions/0036-hold-every-table-but-light-outside-the-core.md), recorded in
  the spec stage. The shape of the extension point itself (two functions on the existing catalog
  type, not a new type or a builder) is cheap to change later and carries no consequence beyond this
  crate, so it is recorded in `research.md` rather than a new ADR, per the spec's own Assumptions.
- **VII. The core stays portable**: the new public items add no CLI or terminal assumption; the
  `wasm` step is extended to prove it of both libraries rather than one.

**Constraints and Dependencies**: no new dependency, so nothing to date-check. Testing: unit tests
are added in both libraries, and the existing CLI subprocess tests are extended — meeting the
minimum principle III and _Testing_ ask for. Scope: the third crate is exactly what the
constitution's _In scope for this phase_ (v1.5.0) admits, naming `monospace-glyph-sets` and
ADR-0036; nothing else in this plan proposes a further crate or anything from _Out of scope_.

No violations. Complexity Tracking is left empty.

## Project Structure

### Documentation (this feature)

```text
specs/054-glyph-tables-can-come-from-outside-the-c/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md         # Phase 1 output
├── quickstart.md         # Phase 1 output
└── contracts/
    └── glyph-set-extension-point.md
```

### Source Code (repository root)

```text
crates/
├── monospace-cli/
│   ├── assets/demo.json          # gains ASCII shapes and two crossings (FR-015..FR-018)
│   ├── src/description.rs        # builds its catalog by union of Light + ASCII (FR-014)
│   └── tests/cli.rs              # extended for the new demo content
├── monospace-core/
│   └── src/glyph.rs               # GlyphCatalog gains from_rules and union (FR-001..FR-005)
└── monospace-glyph-sets/          # NEW — holds the ASCII table (FR-007, FR-011..FR-013)
    ├── Cargo.toml
    └── src/lib.rs

docs/
├── model.md                       # _Strokes, glyph sets and the catalog_ corrected (FR-020)
└── glyph-sets.md                  # opening note corrected (FR-021)

Cargo.toml                         # workspace members and workspace.dependencies gain the new crate
xtask/src/main.rs                  # the `wasm` step's `-p` list gains monospace-glyph-sets (FR-010)
```

**Structure Decision**: A third workspace member, `monospace-glyph-sets`, sitting beside
`monospace-cli` and `monospace-core` under `crates/`, exactly as `docs/decisions/0036` and the
constitution's amended scope describe. It depends on `monospace-core` the same way `monospace-cli`
does — a workspace path dependency declared once in the root `Cargo.toml`'s
`[workspace.dependencies]` and referenced with `.workspace = true`. No new top-level directory and
no change to how the workspace is organized otherwise.

## Complexity Tracking

_No violations to justify._
