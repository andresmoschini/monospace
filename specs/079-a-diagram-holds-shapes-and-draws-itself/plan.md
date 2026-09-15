# Implementation Plan: A diagram holds shapes and draws itself

**Branch**: `079-a-diagram-holds-shapes-and-draws-itself-plan` | **Date**: 2026-09-15 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `/specs/079-a-diagram-holds-shapes-and-draws-itself/spec.md`

## Summary

A fourth workspace member, `monospace-diagram`, holds a `Diagram` — a `Vec` of its own `Shape`
values and nothing else — and draws all of them into a `Buffer` the caller hands it. Its `Shape` is
a closed enum of three kinds, `Box`, `Line` and `Arrow`, each holding the parameters and the
positions of the core shape it constructs, exactly as
[ADR-0039](../../docs/decisions/0039-a-diagram-shape-is-its-own-entity.md) decided. Drawing builds
one `Layer` bound to `StampMode::Below`, visits the shapes from the front of the order to the back
through it, and stops at cells: no glyph catalog, no text, no measuring, no window of its own
([ADR-0042](../../docs/decisions/0042-draw-a-diagram-front-to-back-into-a-given-window.md)).
`monospace-cli` is then put on top of it: its private description types convert into
`monospace_diagram::Shape` instead of stamping core shapes themselves, the format loses its
per-shape `mode` field, and the shipped demonstration's two `mode: "below"` pairs become the
corresponding order so its output does not move. `monospace-core` gains nothing at all.

## Technical Context

**Language/Version**: Rust, edition 2024, the toolchain pinned in `rust-toolchain.toml` — unchanged
by this feature.

**Primary Dependencies**: None added. `monospace-diagram` depends only on `monospace-core`, a
workspace path dependency; `monospace-cli` gains the same kind of dependency on `monospace-diagram`
while keeping the one it has on `monospace-core`. No crate from crates.io is introduced, so the
seven-day publication rule has nothing to check this time.

**Storage**: N/A — a diagram is held in memory for as long as its caller holds it, and nothing in
this slice reads or writes one.

**Testing**: `cargo test --workspace` — unit tests inside `monospace-diagram` for the order, the
drawing, the equivalence of the two orders, clipping and repeatability (TE-001 to TE-005); the
existing unit tests in `crates/monospace-cli/src/description.rs` adjusted to the format without
`mode`; the existing subprocess tests in `crates/monospace-cli/tests/cli.rs` likewise, with the
overlap test there covering TE-006. TE-007 is observed at delivery rather than pinned, per the
spec's _Accepted on observation_.

**Target Platform**: The CLI runs natively. `monospace-core`, `monospace-glyph-sets` and now
`monospace-diagram` all compile for `wasm32-unknown-unknown`; the gate's existing `wasm` step gains
a third `-p` (FR-003).

**Project Type**: Rust cargo workspace — three libraries and one CLI binary; this feature adds the
third library.

**Performance Goals**: N/A — not a concern this feature touches. Adding a shape is O(1) and drawing
is one pass over the shapes, which is as much as this slice has to say about it.

**Constraints**: The shipped demonstration's rendered text must not move (SC-003), although its file
and the format it is written in both change. The core must gain nothing (FR-002). Every public item
of the new crate carries rustdoc as it is introduced (FR-004).

**Scale/Scope**: One new crate of three public types and two methods, one widened gate step, one
rewritten CLI conversion, one edited demonstration file, and the existing CLI tests adjusted for a
format with one field fewer.

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

- **I. Process over product**: followed. The slice is taken through the Spec Kit stages, and the
  crate boundary it builds is the one the ADRs chose for what it teaches about layering, not the
  shortest route to a picture.
- **II. Demonstrable increments**: the spec's three user stories are the three landings — the crate
  that draws, the application on top of it, the demonstration unchanged — and each leaves
  `cargo run -p monospace-cli` producing output. The increment closes with an appended
  `docs/learning-log.md` entry that also records the TE-007 observation.
- **III. One definition of green**: the only gate change is a third `-p` on the existing `wasm`
  step. That is widening a check to a new crate, not adding a kind of check, so the two-commit rule
  for a new check does not apply and there is nothing to make fail on purpose beyond confirming the
  step names the crate.
- **IV. Claims are measured, not assumed**: SC-003 is confirmed by capturing
  `cargo run -p monospace-cli` before the change and diffing after, once, at delivery — and the spec
  already names it as a requirement with nothing automatic behind it. The equivalence of the two
  orders (TE-001) is not taken from `docs/model.md` on trust: a test builds both buffers and
  compares them.
- **V. Structural vs. behavioral**: every commit here is behavioral (`feat`). The new crate is
  additive; the CLI's switch to it changes the format, so it is a `feat` too, not a `refactor`, and
  it carries its own test changes. No preparatory refactor is needed — nothing in the core or the
  CLI has to move before this can be added — so nothing mixes.
- **VI. Decisions recorded when taken**: the architectural decisions are already recorded, in
  ADR-0038, ADR-0039 and ADR-0042, and this plan takes none. What is left is type design inside one
  new crate — the enum's shape, which end of the `Vec` is the front, whether the arrow's endpoint is
  the diagram's own type — all cheap to undo and invisible outside the crate, so they are recorded
  in [research.md](research.md), which is exactly the split _Where a rationale goes_ describes.
- **VII. The core stays portable**: `monospace-core` gains no item, no dependency and no knowledge
  of diagrams; `monospace-diagram` draws through `Surface` and `Layer` like any other caller. The
  new crate names no terminal, no command line and no file, and the `wasm` step is what proves it.

**Constraints and Dependencies**: no new dependency, so nothing to date-check. Testing meets the
constitution's minimum and the spec's own TE list. Scope: `monospace-diagram` is named in the
constitution's _In scope for this phase_, added when ADR-0038 was taken, so this plan widens
nothing; nothing here reaches for an item in the spec's **Out of scope** table.

No violations. Complexity Tracking is left empty.

**Re-checked after Phase 1**: the design added one public type the spec did not name — the diagram's
own `Endpoint` (research.md Q3) — and nothing else. It carries no decision this list rules on: it is
three mirrored fields inside the new crate, not a widening of scope or a departure from a principle.
Every other gate above reads the same after the design as before it.

## Project Structure

### Documentation (this feature)

```text
specs/079-a-diagram-holds-shapes-and-draws-itself/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── checklists/
│   └── requirements.md  # from the spec stage
└── contracts/
    ├── diagram-api.md         # what monospace-diagram exposes
    └── description-format.md  # the CLI format after losing `mode`
```

### Source Code (repository root)

```text
crates/
├── monospace-cli/
│   ├── assets/demo.json       # loses every `mode`; two pairs swap order (FR-017, FR-020)
│   ├── src/description.rs     # converts into diagram shapes and draws a Diagram (FR-016..FR-019)
│   └── tests/cli.rs           # fixtures lose `mode`; the overlap test carries TE-006
├── monospace-core/            # UNCHANGED (FR-002)
├── monospace-diagram/         # NEW — the diagram and its shapes (FR-001, FR-005..FR-015)
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs             # crate docs and re-exports
│       ├── diagram.rs         # Diagram: the order, `add`, `draw`
│       └── shape.rs           # Shape, its three kinds, Endpoint, and their conversions
└── monospace-glyph-sets/      # UNCHANGED

Cargo.toml                     # workspace members and workspace.dependencies gain the new crate
xtask/src/main.rs              # the `wasm` step's `-p` list gains monospace-diagram (FR-003)
```

**Structure Decision**: A fourth workspace member, `crates/monospace-diagram`, beside the three that
exist, depending on `monospace-core` the way `monospace-cli` and `monospace-glyph-sets` do — a path
dependency declared once in the root `Cargo.toml`'s `[workspace.dependencies]` and referenced with
`.workspace = true`. Inside it, two modules behind a `lib.rs` that re-exports them, mirroring how
`monospace-core` keeps `buffer`, `shape` and the rest private and re-exports the names: the split is
`diagram.rs` for what holds the order and draws, `shape.rs` for what a shape is and how it becomes a
core shape. No new top-level directory.

## Complexity Tracking

_No violations to justify._
