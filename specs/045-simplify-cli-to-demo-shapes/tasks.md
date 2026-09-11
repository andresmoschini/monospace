---
description: "Task list for feature 045: simplify CLI to demo shapes"
---

# Tasks: Simplify CLI to demo shapes

**Input**: Design documents from `/specs/045-simplify-cli-to-demo-shapes/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/description-format.md](contracts/description-format.md),
[quickstart.md](quickstart.md)

**Tests**: Included. `plan.md`'s Technical Context requires subprocess tests in
`crates/monospace-cli/tests/cli.rs` and unit tests next to `Description`'s conversion code.

**Organization**: Tasks are grouped by user story (spec.md), in priority order. Every task lists an
exact file path. `[P]` marks a task with no ordering dependency on another incomplete task in the
same phase.

## Path Conventions

Existing two-crate workspace; this feature only touches `crates/monospace-cli/`, per plan.md's
Project Structure:

```text
crates/monospace-cli/
├── assets/demo.json      # new — the shipped demonstration description
├── src/main.rs           # thin: args → read/embed → parse → draw → print, error handling
├── src/description.rs    # new — Description/Canvas/ShapeDescription + conversion into core shapes
└── tests/cli.rs          # rewritten end-to-end coverage
```

## Phase 1: Setup

- [x] T001 Add `serde` (`=1.0.229`, `derive` feature) and `serde_json` (`=1.0.151`) as dependencies
      of `crates/monospace-cli/Cargo.toml` only, matching research.md's versions and rationale.

**Checkpoint**: The workspace still builds; `monospace-cli` has the two new dependencies and uses
neither yet.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: The description format's types, their `Deserialize` implementations, and their
conversion into `monospace_core` shapes (data-model.md), all private to `monospace-cli` (FR-019). No
user story can be implemented before this phase, since all three read a `Description`.

- [x] T002 In `crates/monospace-cli/src/description.rs`, define `Pos { x: i32, y: i32 }` and
      `Size { width: u32, height: u32 }` deriving `serde::Deserialize`, plus
      `From<Pos> for     monospace_core::Pos` and `From<Size> for monospace_core::Size`
      (data-model.md: Pos, Size).
- [x] T003 In `crates/monospace-cli/src/description.rs`, define an `Orientation` mirror enum
      (`"horizontal"` / `"vertical"`) and a `Leaving` mirror enum (`"up"` / `"right"` / `"down"` /
      `"left"`), each deriving `Deserialize` with `#[serde(rename_all = "lowercase")]`, plus
      conversions into `monospace_core::Orientation` and `monospace_core::Direction` (data-model.md:
      Line, Endpoint). Depends on T002 being in the same file.
- [x] T004 In `crates/monospace-cli/src/description.rs`, define a `StampMode` mirror enum (`"above"`
      / `"below"`) deriving `Deserialize` with `#[serde(rename_all = "lowercase")]`, plus a
      conversion into `monospace_core::StampMode`; any other string is already a data error by
      construction (FR-014, data-model.md: StampMode).
- [x] T005 In `crates/monospace-cli/src/description.rs`, add a `deserialize_glyph` function usable
      with `#[serde(deserialize_with = "deserialize_glyph")]` that calls
      `monospace_core::Glyph::new` on the deserialized string and reports a `None` result via
      `serde::de::Error::custom`, so a `fill` or `head` that is not exactly one grapheme cluster
      fails during the same `serde_json::from_str` call as every other data error (data-model.md:
      BoxShape.fill, Endpoint.head; research.md, "one `serde_json::Error` covers every failure path
      except a missing file").
- [x] T006 In `crates/monospace-cli/src/description.rs`, define `Canvas { origin: Pos, size: Size }`
      deriving `Deserialize` (data-model.md: Canvas). Depends on T002.
- [x] T007 In `crates/monospace-cli/src/description.rs`, define
      `Endpoint { at: Pos, leaving: Leaving, head: Glyph }` (the `head` field using T005's
      `deserialize_glyph`) deriving `Deserialize`, plus its conversion into
      `monospace_core::Endpoint` (data-model.md: Endpoint). Depends on T002, T003, T005.
- [x] T008 In `crates/monospace-cli/src/description.rs`, define `ShapeDescription` as an internally
      tagged enum (`#[serde(tag = "kind", rename_all = "lowercase")]`) with `Box`, `Line` and
      `Arrow` variants matching contracts/description-format.md field-for-field (each variant
      carries its own `mode: StampMode`), deriving `Deserialize` — an unrecognized `kind` is then
      reported by name (FR-014). Depends on T002–T007.
- [x] T009 In `crates/monospace-cli/src/description.rs`, implement
      `ShapeDescription::draw(&self, buffer: &mut monospace_core::Buffer)`: convert the matched
      variant into the corresponding `monospace_core::{BoxShape, Line, Arrow}` and stamp it via
      `monospace_core::Layer::new(buffer, mode)` using that variant's own stamp mode (FR-008,
      FR-009). Depends on T008.
- [x] T010 In `crates/monospace-cli/src/description.rs`, define
      `Description { canvas: Canvas, shapes: Vec<ShapeDescription> }` deriving `Deserialize`, and
      `Description::render(&self) -> String`: build a `monospace_core::Buffer` from `canvas`, call
      `ShapeDescription::draw` for every shape in `shapes`, in order, then return
      `monospace_core::render` of that buffer with `monospace_core::GlyphCatalog::light()`
      (data-model.md: Description; FR-005, FR-009). Depends on T006, T009.
- [x] T011 In `crates/monospace-cli/src/description.rs`, add unit tests next to the conversions
      (plan.md, Technical Context: Testing): a one-box `Description` renders the same text as a
      `BoxShape` drawn directly with the same parameters; an unrecognized `kind` fails to
      deserialize; an unrecognized `mode` fails to deserialize; a `fill` or `head` of more than one
      grapheme cluster fails to deserialize. Depends on T002–T010.

**Checkpoint**: `description.rs` exists with its own unit tests, but nothing in `main.rs` calls into
it yet, so it is not part of the compiled binary. Since Rust's dead-code check only clears once a
module is both declared and reached from `main`, this phase's tasks and User Story 1's
`mod description;` line (T012) will typically need to land in the same commit to keep the gate green
(constitution, principle II: "one commit per group where [a task] does not [reach green alone]") — a
grouping decision for implementation, not a change to the task list.

---

## Phase 3: User Story 1 - Render a diagram described in a file (Priority: P1) 🎯 MVP

**Goal**: Given an explicit path, the application reads, parses and renders that file's diagram.

**Independent Test**: write a file describing a single box, run
`cargo run -p monospace-cli -- <path>`, and compare the printed output to the same box as the
previous hardcoded demo drew it.

### Implementation for User Story 1

- [x] T012 [US1] In `crates/monospace-cli/src/main.rs`, add `mod description;` and, when
      `std::env::args().skip(1)` yields exactly one argument, read that path with
      `std::fs::read_to_string`, parse it with `serde_json::from_str::<description::Description>`,
      and `print!` the result of `.render()` — leaving the existing no-argument hardcoded demo in
      place for now (FR-001, FR-002). Depends on Phase 2 (T002–T011).

### Tests for User Story 1

- [x] T013 [US1] Add subprocess tests to `crates/monospace-cli/tests/cli.rs`: an explicit path to a
      hand-written single-box file prints exactly that box (acceptance scenario 1); a file with a
      box, a line and an arrow at stated positions prints all three composed (acceptance scenario
      2); the same file with its shapes reordered changes which one is drawn on top where they
      overlap (acceptance scenario 3); running the same file twice produces identical output
      (acceptance scenario 4, FR-017). Depends on T012.

**Checkpoint**: User Story 1 is independently testable — `cargo run -p monospace-cli -- <path>`
renders a hand-written file. No-argument behavior is unchanged.

---

## Phase 4: User Story 2 - See the current capabilities with no arguments (Priority: P2)

**Goal**: With no arguments, the application renders the embedded shipped demonstration; the
hardcoded demo is removed (FR-018).

**Independent Test**: run `cargo run -p monospace-cli` with no arguments and check that the printed
diagram shows all three shapes, a filled interior, and both stamp modes.

### Implementation for User Story 2

- [x] T014 [US2] Write `crates/monospace-cli/assets/demo.json`: a `Description` exercising all three
      shapes, a filled box interior, overlap, and both stamp modes shown as two overlapping
      arrangements side by side on one canvas (FR-010), in the format
      contracts/description-format.md states. Depends on Phase 2 (T002–T011).
- [x] T015 [US2] In `crates/monospace-cli/src/main.rs`, embed that file with
      `include_str!("../assets/demo.json")` (FR-022, FR-023) as the description text used when no
      argument is given, parsed and rendered through the same path as T012; remove `stamp_box`,
      `render_pair` and every other hardcoded shape from `main.rs` (FR-018, SC-006). Depends on
      T012, T014.

### Tests for User Story 2

- [x] T016 [US2] Replace `crates/monospace-cli/tests/cli.rs`'s `prints_the_box_then_both_pairs` test
      with subprocess tests asserting: no arguments prints the demonstration and exits successfully
      (acceptance scenario 1); running from a different working directory prints the same diagram
      (acceptance scenario 2, FR-022); passing `crates/monospace-cli/assets/demo.json` explicitly
      produces byte-identical output to no arguments (acceptance scenario 3). Depends on T015.

**Checkpoint**: User Stories 1 AND 2 both work independently; the CLI carries no diagram of its own
in code (FR-018, SC-006).

---

## Phase 5: User Story 3 - Be told plainly when a file cannot be drawn (Priority: P3)

**Goal**: Every input failure prints a message naming the problem to stderr, nothing to stdout, and
exits with a failure status; no input panics.

**Independent Test**: run against a path that does not exist and against a file holding malformed
text, and check each prints a message naming the problem and exits with a failure status.

### Implementation for User Story 3

- [x] T017 [US3] In `crates/monospace-cli/src/main.rs`, handle the two failures the pipeline can
      produce without printing anything to stdout first: a `std::fs::read_to_string` error prints a
      message naming the path to stderr and exits with a failure status (FR-012); a
      `serde_json::from_str` error prints its `Display` message to stderr and exits with a failure
      status (FR-013, FR-014, FR-016); success alone reaches the `print!` call (FR-015). Depends on
      T015.
- [x] T018 [US3] In `crates/monospace-cli/src/main.rs`, when more than one command-line argument is
      given, print a usage message to stderr and exit with a failure status instead of matching on
      it as a path (Edge Cases: "More than one command-line argument"). Depends on T012.

### Tests for User Story 3

- [x] T019 [US3] Add subprocess tests to `crates/monospace-cli/tests/cli.rs`: a path that does not
      exist prints nothing to stdout, names the path on stderr, and fails (acceptance scenario 1);
      malformed JSON prints nothing to stdout, locates the problem on stderr, and fails (acceptance
      scenario 2); an unrecognized shape kind prints nothing to stdout, names the kind on stderr,
      and fails (acceptance scenario 3); more than one argument prints a usage message and fails
      (Edge Cases). Depends on T017, T018.

**Checkpoint**: All three user stories are independently functional; SC-005 holds — no input file
panics, exits successfully with no diagram, or prints a partial diagram alongside an error.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [x] T020 [P] Run every scenario in `specs/045-simplify-cli-to-demo-shapes/quickstart.md` by hand
      and confirm the actual output, not an assumed one (constitution, principle IV).
- [x] T021 Run `cargo xtask check` on a fresh clone and confirm it is green (constitution, principle
      III and IV).
- [x] T022 [P] Append an entry to `docs/learning-log.md` for this increment: what was learned about
      Rust design and idiom, what was learned about working this way, and optionally a trade-off
      worth remembering (constitution, principle II).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: No dependencies — start immediately.
- **Foundational (Phase 2)**: Depends on Setup (needs `serde`/`serde_json`) — BLOCKS all user
  stories.
- **User Story 1 (Phase 3)**: Depends on Phase 2.
- **User Story 2 (Phase 4)**: Depends on Phase 2 and on User Story 1's `main.rs` wiring (T012),
  which it extends rather than duplicates.
- **User Story 3 (Phase 5)**: Depends on User Story 2's `main.rs` state (T015) and on User Story 1's
  argument-count branch (T012).
- **Polish (Phase 6)**: Depends on User Stories 1, 2 and 3 all being complete.

### User Story Dependencies

Unlike a typical multi-surface feature, all three stories edit the same `main()` function in
sequence — each adds a branch or a failure path the previous one did not need — so they are ordered
rather than independently parallel, even though each is independently testable once its phase lands.

### Parallel Opportunities

- Within Phase 2, T002, T003 and T004 define independent types but all land in the same new file, so
  they are sequenced by task order rather than marked `[P]`.
- T020 and T022 in Phase 6 touch different files with no dependency between them and can run in
  parallel.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup.
2. Complete Phase 2: Foundational (commit together with T012 if the gate requires it — see Phase 2's
   checkpoint).
3. Complete Phase 3: User Story 1.
4. **STOP and VALIDATE**: run quickstart.md scenario 3 by hand.

### Incremental Delivery

1. Setup + Foundational + User Story 1 → `cargo run -p monospace-cli -- <path>` renders a
   hand-written file; no-argument output is still the old hardcoded demo.
2. Add User Story 2 → the hardcoded demo is gone; no arguments renders the embedded demonstration
   (FR-018, principle II satisfied by the new source instead of the old one).
3. Add User Story 3 → every failure path is a message, not a panic or silent success.
4. Polish → quickstart validated, gate green on a fresh clone, learning-log entry appended.
