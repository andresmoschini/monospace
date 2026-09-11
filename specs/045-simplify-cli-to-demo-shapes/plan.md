# Implementation Plan: Simplify CLI to demo shapes

**Branch**: `045-simplify-cli-to-demo-shapes` | **Date**: 2026-09-11 | **Spec**: [spec.md](spec.md)

**Input**: Feature specification from `/specs/045-simplify-cli-to-demo-shapes/spec.md`

## Summary

Replace `monospace-cli`'s hardcoded demo (FR-018) with a binary that reads an optional JSON file
path, deserializes it into a canvas and an ordered list of shape descriptions, converts each
description into the matching `monospace-core` shape, draws them in order, and prints the result —
or, given no path, does the same with a JSON file embedded in the binary at compile time (FR-022,
FR-023). `monospace-core` gains nothing (FR-019, ADR-0035): the description format, its parsing and
its error messages live entirely in `monospace-cli`, using `serde` + `serde_json` (see
[research.md](research.md)).

## Technical Context

**Language/Version**: Rust, edition 2024, pinned in `rust-toolchain.toml` (unchanged by this
feature).

**Primary Dependencies**: `serde` 1.0.229 (`derive` feature) and `serde_json` 1.0.151, added to
`monospace-cli` only — both published 2026-07, confirmed with the maintainer (research.md).
`monospace-core`'s dependencies are unchanged.

**Storage**: N/A — the application only reads (FR-011); no file is written.

**Testing**: `cargo test --workspace`. `crates/monospace-cli/tests/cli.rs`'s existing end-to-end
test is replaced (Assumptions) with subprocess tests covering: no arguments, an explicit path to the
shipped demonstration, a hand-written file, a missing path, malformed JSON, an unrecognized shape
kind, and more than one argument. Unit tests for `Description`'s conversion into `monospace-core`
shapes live next to that code.

**Target Platform**: Same as today — a terminal-invoked binary, no OS-specific behavior added.

**Project Type**: Existing two-crate Cargo workspace (`monospace-core`, `monospace-cli`); this
feature only touches `monospace-cli`.

**Performance Goals**: None stated by the spec; not a concern for a file this small.

**Constraints**: No panic on any input file (FR-016); deterministic output (FR-017); stdout empty on
any failure (FR-015); `monospace-core`'s public API and dependencies unchanged (FR-019).

**Scale/Scope**: One binary, one new module, one embedded demonstration file, three shape kinds.

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

- **I. Process over product**: followed — spec written, ADR-0035 recorded before this plan (per the
  spec's own Dependencies section), this plan next, a learning-log entry at the increment's end.
- **II. Demonstrable increments**: FR-003 keeps `cargo run -p monospace-cli` producing output on
  every commit. Tasks (next phase) are expected to slice thin, per the spec's three user stories in
  priority order.
- **III. One definition of green**: no new check is added to `cargo xtask check`; existing gates
  (fmt, clippy, tests, wasm build of `monospace-core`) are unaffected since only `monospace-cli`
  changes.
- **IV. Claims are measured**: quickstart.md's scenarios are run and their actual output recorded
  during implementation, not assumed from this plan.
- **V. Structural vs. behavioral change**: replacing the hardcoded demo is a behavioral change
  (`feat`); any preparatory extraction of `main.rs` into modules before that change lands is its own
  `refactor` commit, unchanged behavior, per Kent Beck's rule.
- **VI. Decisions recorded when taken**: ADR-0035 already records the one decision this feature
  takes ahead of code depending on it (keeping the format out of `monospace-core`). No further ADR
  is anticipated; a genuinely new one taken during implementation gets its own record before code
  depends on it.
- **VII. The core stays portable**: `monospace-core` is untouched — no new type, dependency or
  public function — so the WebAssembly boundary is not exercised by this feature.

No violations. Complexity Tracking is empty.

## Project Structure

### Documentation (this feature)

```text
specs/045-simplify-cli-to-demo-shapes/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md         # Phase 1 output
├── quickstart.md         # Phase 1 output
├── contracts/
│   └── description-format.md
└── tasks.md              # Phase 2 output (/speckit-tasks — not this command)
```

### Source Code (repository root)

```text
crates/
├── monospace-core/       # untouched by this feature (FR-019)
└── monospace-cli/
    ├── assets/
    │   └── demo.json      # the shipped demonstration description (FR-022, FR-023)
    ├── src/
    │   ├── main.rs         # thin: args → read/embed → parse → draw → print, error handling
    │   └── description.rs  # Description/Canvas/ShapeDescription + conversion into core shapes
    └── tests/
        └── cli.rs          # rewritten end-to-end coverage (see Technical Context: Testing)
```

**Structure Decision**: no new crate and no change to the workspace's members. Everything this
feature adds lives inside `monospace-cli`: a sibling module to `main.rs` for the description types
and their conversion, and a non-`src` `assets/` directory holding the one JSON file that both the
embedded demonstration and any hand-written file are examples of the same format.

## Complexity Tracking

_No violations — table intentionally left empty._
