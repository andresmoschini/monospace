# Implementation Plan: Fix shapes without filling closing their inner arms

**Branch**: `049-bug-with-shapes-without-filling` | **Date**: 2026-09-11 | **Spec**:
[spec.md](./spec.md)

**Input**: Feature specification from `/specs/049-bug-with-shapes-without-filling/spec.md`

## Summary

An unfilled box's border currently stamps `Arm::Closed` on the side facing its own interior no
matter what, so a stroke crossing into that interior renders as a closed junction (`├`, `┬`, ...)
instead of a crossing (`┼`). The fix gives `Border` — the one fragment that decides this side — a
`closes_interior: bool` field, and has `BoxShape` pass `self.fill.is_some()` into every `Border` it
places. `Corner` already leaves its non-open sides `Unset`, never `Closed`, so it needs no change.
Confirmed by a reverted probe test (see research.md): the fix produces the reported crossing where
none of the shapes involved has a fill, keeps closing the junction wherever the shape drawn on top
does have one, and leaves every one of the 77 pre-existing tests unchanged.

## Technical Context

**Language/Version**: Rust, edition 2024, pinned in `rust-toolchain.toml` (unchanged by this
feature)

**Primary Dependencies**: None added. The fix touches only `monospace-core`'s existing `shape`
module; no crate is added, upgraded or removed.

**Storage**: N/A

**Testing**: `cargo test` (unit tests in `crates/monospace-core/src/shape/fragment/border.rs` and
`crates/monospace-core/src/shape/box_shape.rs`, run via `cargo test --workspace` for the whole gate)

**Target Platform**: Terminal (via `monospace-cli`); `monospace-core` also targets
`wasm32-unknown-unknown` per the constitution's portability gate — this fix changes no public API
and adds no dependency, so it does not touch that boundary

**Project Type**: Library (`monospace-core`) + minimal CLI consumer (`monospace-cli`) — unchanged

**Performance Goals**: N/A — one extra `bool` comparison per border cell drawn, immeasurable against
existing rendering cost

**Constraints**: Every diagram with no overlapping shapes MUST render byte-for-byte identically to
today (SC-002); no new public API on `BoxShape` or elsewhere is introduced (Assumptions in spec.md)

**Scale/Scope**: Two files in `monospace-core` (`shape/fragment/border.rs`, `shape/box_shape.rs`),
plus a correction to `docs/model.md`'s box description so it matches _The cell_'s own rule (see
research.md) — no change to `monospace-cli`, no new fragment, no new shape kind

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

| Principle                                                | Check                                                                                                                                                                                                                             | Status |
| -------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------ |
| I. Process over product                                  | Goes through Spec Kit (this is stage 2 of 3 for issue #049); no shortcut taken                                                                                                                                                    | PASS   |
| II. Demonstrable increments                              | One thin slice: a single rendering rule, one acceptance example, tasks.md will name exactly the tasks.md checkboxes each commit ticks                                                                                             | PASS   |
| III. One definition of green                             | No new check is being added to `cargo xtask check`; the existing gate is what tasks.md's commits must pass                                                                                                                        | PASS   |
| IV. Claims are measured, not assumed                     | The bug and the fix were both measured against a reverted probe test before this plan was written (research.md); the "before" and "after" outputs quoted there and in quickstart.md were run, not guessed                         | PASS   |
| V. Structural and behavioral change never share a commit | This is a `fix`, not a `refactor`: the new `closes_interior` field and its use are one behavioral change with no separable structural step, so one `fix` commit (or a few, per tasks.md) is correct, not a violation of the split | PASS   |
| VI. Decisions recorded when taken                        | No new ADR needed — this applies ADR-0028's existing rule to one more fact `Border` needs, it does not add a new kind of decision (research.md, "Whether this needs an ADR")                                                      | PASS   |
| VII. The core stays portable                             | Change is entirely inside `monospace-core`; `monospace-cli` is untouched; no CLI, TUI or terminal assumption enters the core                                                                                                      | PASS   |
| Testing (Constraints)                                    | Unit tests in `monospace-core` cover both the crossing and the fill-keeps-closing behavior; tasks.md will name them                                                                                                               | PASS   |
| Dependencies (Constraints)                               | None added                                                                                                                                                                                                                        | PASS   |
| In/out of scope (Constraints)                            | Touches only `monospace-core`/`monospace-cli`, the phase's declared scope; introduces no WASM bindings, web front-end, GUI, persistence, collaboration or export format                                                           | PASS   |

No violation to record in Complexity Tracking.

## Project Structure

### Documentation (this feature)

```text
specs/049-bug-with-shapes-without-filling/
├── spec.md                  # already merged (stage 1)
├── plan.md                  # this file
├── research.md              # Phase 0 output
├── data-model.md            # Phase 1 output
├── quickstart.md            # Phase 1 output
└── quickstart-example.json  # runnable example backing quickstart.md
```

No `contracts/` directory: this fix changes no external interface. `BoxShape`'s public fields are
unchanged, `monospace-cli`'s description format (`specs/045-.../contracts/description-format.md`) is
unaffected, and the only type gaining a field (`Border`) is `pub(crate)`.

### Source Code (repository root)

```text
crates/
├── monospace-core/
│   └── src/
│       └── shape/
│           ├── box_shape.rs        # BoxShape::draw: pass closes_interior to each Border
│           └── fragment/
│               └── border.rs       # Border: new closes_interior field, branches Closed/Unset
└── monospace-cli/                  # untouched by this feature

docs/
└── model.md                        # box description in "The initial set" corrected to match
                                     # "The cell"'s own rule (see research.md)
```

**Structure Decision**: Single-project layout already in place (`monospace-core` + `monospace-cli`,
per the constitution's declared scope for this phase). This feature adds no new module and no new
crate — it is confined to the two files named above, plus the one documentation correction.

## Complexity Tracking

_No violations recorded — table intentionally empty; see Constitution Check above._
