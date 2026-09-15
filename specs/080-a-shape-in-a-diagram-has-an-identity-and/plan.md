# Implementation Plan: A shape in a diagram has an identity, and the order can change

**Branch**: `080-a-shape-in-a-diagram-has-an-identity-and-plan` | **Date**: 2026-09-15 | **Spec**:
[spec.md](spec.md)

**Input**: Feature specification from `/specs/080-a-shape-in-a-diagram-has-an-identity-and/spec.md`

## Summary

`monospace-diagram` gains one public type and two public methods. `ShapeId` is a newtype over a
private `String` holding the identity's whole text — `#1`, `#2`, and so on — with a `Display` that
writes it and no way in: no constructor, no `FromStr`, no accessor, so the only identities that
exist are the ones the diagram handed out (FR-003). The text is the representation rather than a
number dressed up at printing time, because _Identity_'s open question about editing an identity is
one this repository expects to answer, and an edited identity is text (research.md Q1). `Diagram`
holds a `Vec<Placed>` instead of a `Vec<Shape>` — `Placed` being a crate-private pair of an identity
and a shape — plus a `u32` counter that numbers the next identity; `add` returns the identity it
just generated, and `forward` and `backward` take one by reference, find it in the order and swap
its entry with its neighbor, doing nothing at all when there is no such entry or when it is already
at that end. Neither returns anything, because FR-009 rules out both an error and a report.
`monospace-cli` then draws twice: its conversion hands back the identity of the first entry in the
description, the application renders the diagram as written, moves that shape one place forward, and
renders it again into a second buffer, each picture under a caption. The core gains nothing, the
description format gains nothing, and the shipped demonstration file does not change.

## Technical Context

**Language/Version**: Rust, edition 2024, the toolchain pinned in `rust-toolchain.toml` — unchanged
by this feature.

**Primary Dependencies**: None added, and none changed. The workspace's existing path dependencies
are all this needs, so the seven-day publication rule has nothing to check this time.

**Storage**: N/A — a diagram lives for as long as its caller holds it, and nothing here reads or
writes one.

**Testing**: `cargo test --workspace` — new unit tests in `monospace-diagram` for the identities
(TE-001) and for the five order properties observed by drawing (TE-002 to TE-006), reusing the
`cells` helper that crate's tests already have to compare two buffers by value; the existing
subprocess test in `crates/monospace-cli/tests/cli.rs` that pins a whole run adjusted to read the
first of two pictures, and a new one covering TE-007 on a two-box description. TE-008 is observed at
delivery rather than pinned, per the spec's _Accepted on observation_.

**Target Platform**: The CLI runs natively; `monospace-diagram` compiles for
`wasm32-unknown-unknown` and the gate's `wasm` step already names it, so `xtask` does not change.

**Project Type**: Rust cargo workspace — three libraries and one CLI binary. No crate is added.

**Performance Goals**: N/A. `add` stays O(1) amortized and now allocates the identity's text once; a
move is a linear search of the order comparing that text, and one swap, on a diagram whose order is
a handful of shapes.

**Constraints**: The picture the application prints for a description must not move (FR-014),
although the output around it does. The core gains nothing. The description format gains no field
and its types stay private to `monospace-cli` (FR-019). Every public item carries rustdoc as it is
introduced (FR-006).

**Scale/Scope**: One new public type, one changed signature, two new methods, two changed private
fields, one removed private helper, one rewritten function in `main.rs`, and the tests for all of
it.

## Constitution Check

_GATE: Must pass before Phase 0 research. Re-check after Phase 1 design._

- **I. Process over product**: followed. The slice goes through the Spec Kit stages, and the typing
  of the identity is chosen for what it teaches — a newtype whose private field is what makes FR-003
  structural instead of a convention — rather than for the shortest route to a reorder. Holding the
  identity as text costs an allocation this slice does not need, and buys that the type survives
  editing arriving later; the maintainer took that trade.
- **II. Demonstrable increments**: the three user stories are the three landings — identities,
  moving, the demonstration — and each leaves `cargo run -p monospace-cli` producing output. The
  increment closes with an appended `docs/learning-log.md` entry that also records the TE-008
  observation.
- **III. One definition of green**: the gate does not change. Its `wasm` step already names
  `monospace-diagram`, so no check is added and the two-commit rule for a new check does not apply.
- **IV. Claims are measured, not assumed**: every claim about what a move does to a picture is made
  by a test that builds both buffers and compares them, never by reading the model. FR-014 is
  confirmed by capturing `cargo run -p monospace-cli` before the change and diffing after, once, at
  delivery; the spec already names TE-008 as a requirement with nothing automatic behind it, and
  [quickstart.md](quickstart.md) has the commands.
- **V. Structural and behavioral change never share a commit**: one preparatory `refactor` comes
  first — removing `Description::buffer` in favor of `Buffer::new` at the call site, because the
  second picture needs a second buffer from a description that `into_diagram` consumes (research.md
  Q7). It changes no behavior and adds no test. Everything after it is `feat`.
- **VI. Decisions recorded when taken**: the spec already settled that this slice takes no ADR, and
  the design bore that out — the eight questions in [research.md](research.md) are all answered
  inside one crate or inside the CLI, and each is undone by changing the code that answers it.
- **VII. The core stays portable**: `monospace-core` gains no item and no knowledge of identities.
  `ShapeId` is a `String` and a `Display`, so it names no platform, and the existing `wasm` step is
  what proves it.

**Constraints and Dependencies**: no new dependency, so nothing to date-check. Testing meets the
constitution's minimum and the spec's own TE list. Scope: `monospace-diagram` is already named in
the constitution's _In scope for this phase_ and no crate is added; nothing here reaches for a line
in the spec's **Out of scope** table — in particular nothing reads the order back, and no identity
can be built by a caller.

No violations. Complexity Tracking is left empty.

**Re-checked after Phase 1**: the design added one crate-private type the spec did not name,
`Placed` (research.md Q2), and one return value on a private CLI method (Q6). Neither is a public
item, neither is a decision this list rules on, and neither widens the slice. Every gate above reads
the same after the design as before it.

## Project Structure

### Documentation (this feature)

```text
specs/080-a-shape-in-a-diagram-has-an-identity-and/
├── plan.md              # This file
├── research.md          # Phase 0 output
├── data-model.md        # Phase 1 output
├── quickstart.md        # Phase 1 output
├── checklists/
│   └── requirements.md  # from the spec stage
└── contracts/
    └── diagram-api.md   # what monospace-diagram exposes after this slice
```

There is no `description-format.md` here: the format does not change (FR-019), and spec 079's
contract stays its record.

### Source Code (repository root)

```text
crates/
├── monospace-cli/
│   ├── assets/demo.json       # UNCHANGED (FR-017)
│   ├── src/description.rs     # into_diagram returns the first entry's identity; buffer() removed
│   ├── src/main.rs            # renders twice, captioned, with the move between (FR-013..FR-018)
│   └── tests/cli.rs           # the whole-output test reads picture one; a new test covers TE-007
├── monospace-core/            # UNCHANGED
├── monospace-diagram/
│   └── src/
│       ├── diagram.rs         # ShapeId, Placed, the counter, add's return, forward, backward
│       ├── lib.rs             # re-exports ShapeId alongside Diagram
│       └── shape.rs           # UNCHANGED — a shape carries no identity
└── monospace-glyph-sets/      # UNCHANGED

xtask/src/main.rs              # UNCHANGED — the wasm step already names monospace-diagram
```

**Structure Decision**: No new module and no new crate. `ShapeId` and `Placed` go in `diagram.rs`
rather than in a module of their own, for a mechanical reason and a design one: the identity is the
diagram's to generate, and putting it beside `Diagram` is what lets `add` build one from a private
field without exposing a constructor to do it. `shape.rs` is untouched, which is the same statement
in the file layout that `Placed` makes in the types — the identity belongs to the diagram, not to
the shape.

## Complexity Tracking

_No violations to justify._
