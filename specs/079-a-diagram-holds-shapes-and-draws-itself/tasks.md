# Tasks: A diagram holds shapes and draws itself

**Input**: Design documents from `/specs/079-a-diagram-holds-shapes-and-draws-itself/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/](contracts/), [quickstart.md](quickstart.md)

**Tests**: included, because the spec asks for them by name. TE-001 to TE-006 are unit or subprocess
tests and appear as tasks below; TE-007 is _Accepted on observation_ and appears as the
capture-and-compare pair T002 / T035.

**Organization**: grouped by the spec's three user stories. All three are P1 and land in order: the
crate that draws (US1), what the order means (US2), the application on top of it (US3).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: can run in parallel — a different file, no dependency on an unfinished task
- **[Story]**: which user story the task belongs to (US1, US2, US3)
- Paths are from the repository root

## Commit grouping

Principle II asks for one commit per task where the task reaches green alone, and one per group
where it does not. Three groups here do not reach green alone, and each is marked at its phase:

- **T003–T006** — a crate with no items and a `wasm` step naming it: green together, not separately
  (the step cannot name a crate the workspace does not have).
- **T007–T020** — `Shape` and its `draw` are dead code until `Diagram::draw` calls them, and
  `-D warnings` fails on dead code. The crate's first commit is the whole of US1.
- **T024–T033** — the CLI does not compile between dropping `mode` and building a `Diagram`. One
  commit for the application, its tests and its demonstration.

Every commit in this feature is behavioral: `feat`. No preparatory refactor is needed, so nothing
here is a `refactor` and nothing mixes (principle V).

---

## Phase 1: Setup (Shared Infrastructure)

**Purpose**: capture what SC-003 will be compared against, then stand the crate up empty and make
the gate see it.

- [ ] T001 Confirm the working tree is clean and on the implementation stage branch, opened with
      `cargo xtask spec stage 79 impl`
- [ ] T002 Capture the demonstration before anything changes:
      `cargo run -p monospace-cli > /tmp/demo-before.txt`, and keep the file until T035 (SC-003,
      TE-007)
- [ ] T003 Create `crates/monospace-diagram/Cargo.toml`: name `monospace-diagram`, the description
      from contracts/diagram-api.md, `version`/`edition`/`authors`/`license` inherited with
      `.workspace = true`, `monospace-core.workspace = true` as the only dependency, and
      `[lints] workspace = true` (FR-001)
- [ ] T004 Add `crates/monospace-diagram` to `[workspace].members` and
      `monospace-diagram = { path = "crates/monospace-diagram" }` to `[workspace.dependencies]` in
      `Cargo.toml`
- [ ] T005 [P] Create `crates/monospace-diagram/src/lib.rs` with the crate-level `//!` rustdoc
      saying what the crate holds and pointing at `docs/diagram-model.md` (FR-004)
- [ ] T006 [P] Add `"-p", "monospace-diagram"` to the `wasm` step's args in `xtask/src/main.rs`,
      beside the two crates it names today (FR-003)

**Checkpoint**: `cargo xtask check` is green and the `wasm` step names three crates.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: the shape types every story draws through. They carry no behavior a test can reach
until `Diagram::draw` exists, so they commit with User Story 1 (see **Commit grouping**).

**⚠️ CRITICAL**: no user story work can begin until this phase is complete

- [ ] T007 Create `crates/monospace-diagram/src/shape.rs` with
      `pub struct Endpoint { at: Pos, leaving: Direction, head: Glyph }`, public fields, rustdoc on
      the type and each field, and `impl From<Endpoint> for monospace_core::Endpoint` per
      data-model.md's `Endpoint` table (FR-007, research Q3)
- [ ] T008 Add `pub enum Shape` with the three struct variants `Box`, `Line` and `Arrow` and exactly
      the fields contracts/diagram-api.md lists, rustdoc on the enum and every variant, in
      `crates/monospace-diagram/src/shape.rs` (FR-007, FR-008, research Q1)
- [ ] T009 Add `pub(crate) fn draw(&self, surface: &mut impl Surface)` to `Shape` in
      `crates/monospace-diagram/src/shape.rs`: each variant constructs its `monospace_core` shape
      with every field mapped per data-model.md and draws it through `surface`, dropping no
      parameter (FR-009, research Q4)
- [ ] T010 Declare `mod shape;` and re-export `Endpoint` and `Shape` from
      `crates/monospace-diagram/src/lib.rs`, re-exporting no `monospace_core` type (research Q10)

**Checkpoint**: the shape half of the crate is written; it compiles green only once T012 calls it.

---

## Phase 3: User Story 1 - A figure survives being drawn (Priority: P1) 🎯 MVP

**Goal**: a diagram holds shapes and draws them into a window the caller gives it, unchanged by the
drawing and repeatable.

**Independent Test**: build a diagram, add two shapes, draw it into a window, then draw the same
diagram again into an equal window — both buffers are equal and the diagram is unchanged
(`cargo test -p monospace-diagram`).

### Implementation for User Story 1

- [ ] T011 [US1] Create `crates/monospace-diagram/src/diagram.rs` with `pub struct Diagram` holding
      a private `Vec<Shape>` whose last element is the front of the order, plus `new`,
      `impl Default` delegating to it, and `add` pushing, each with rustdoc (FR-005, FR-006,
      research Q2, Q8)
- [ ] T012 [US1] Implement `pub fn draw(&self, buffer: &mut Buffer)` in
      `crates/monospace-diagram/src/diagram.rs`: build one `Layer` bound to `StampMode::Below`,
      visit the shapes from the front of the order to the back through it, return nothing (FR-010 to
      FR-015, research Q5, Q6)
- [ ] T013 [US1] Declare `mod diagram;` and re-export `Diagram` from
      `crates/monospace-diagram/src/lib.rs`

### Tests for User Story 1

- [ ] T014 [US1] Add the `#[cfg(test)] mod tests` buffer helper in
      `crates/monospace-diagram/src/diagram.rs`: collect `Vec<Option<Cell>>` over a window with
      `Buffer::cell`, and render with `monospace_core::render` where the assertion is about the
      picture (research Q7)
- [ ] T015 [US1] Test repeatability in `crates/monospace-diagram/src/diagram.rs`: a diagram holding
      a box and a line, drawn twice into equal windows, produces equal buffers (TE-003, scenarios 1
      and 2)
- [ ] T016 [US1] Test that an empty diagram leaves its buffer exactly as it was, in
      `crates/monospace-diagram/src/diagram.rs` (scenario 3)
- [ ] T017 [US1] Test clipping in `crates/monospace-diagram/src/diagram.rs`: a box partly outside
      the window draws what falls inside and nothing else, with no error (TE-002, scenario 4)
- [ ] T018 [US1] Test a box against `monospace_core::BoxShape` drawn directly with the same
      parameters, in `crates/monospace-diagram/src/diagram.rs` (TE-005, scenario 5)
- [ ] T019 [US1] Test a line against `monospace_core::Line` the same way, in
      `crates/monospace-diagram/src/diagram.rs` (TE-005)
- [ ] T020 [US1] Test an arrow against `monospace_core::Arrow` the same way, both endpoints
      included, in `crates/monospace-diagram/src/diagram.rs` (TE-005)

**Checkpoint**: `cargo test -p monospace-diagram` passes and `cargo xtask check` is green. User
Story 1 is the MVP: the crate holds a picture and draws it. T007–T020 land as one `feat` commit.

---

## Phase 4: User Story 2 - The order decides who wins an overlap (Priority: P1)

**Goal**: the order is what decides a shared cell, and nothing else is offered for it.

**Independent Test**: the same two overlapping boxes in two diagrams in opposite orders draw two
different buffers, and each equals the two core shapes stamped back to front with `Above`.

**Note**: User Story 1's `draw` already visits front to back with `Below`, so this story adds no
production code. Its tasks are the tests SC-002 and SC-005 ask for — the story is unfinished without
them, since a behavior rule with no test named against it is an unfinished spec.

### Tests for User Story 2

- [ ] T021 [US2] Test the equivalence of the two orders in
      `crates/monospace-diagram/src/diagram.rs`: two overlapping boxes drawn front to back with
      `Below` produce the buffer that stamping the same two core shapes back to front with `Above`
      produces (TE-001, scenario 1)
- [ ] T022 [US2] Test that the order matters in `crates/monospace-diagram/src/diagram.rs`: the same
      two overlapping boxes in opposite orders produce different buffers, and in each the front-most
      shape's stroke decides the shared cells (TE-004, scenarios 2 and 5)
- [ ] T023 [US2] Test that a crossing is composition, not occlusion, in
      `crates/monospace-diagram/src/diagram.rs`: a horizontal and a vertical line that cross make a
      junction glyph, and a filled box in front of a line hides it where they overlap (scenarios 3
      and 4)

**Checkpoint**: `cargo test -p monospace-diagram` passes, and every claim about which figure wins is
backed by a test that fails if the drawing order or the stamp mode changes (SC-005). One `feat`
commit.

---

## Phase 5: User Story 3 - The command-line application draws through a diagram (Priority: P1)

**Goal**: the application builds a diagram and asks it to draw; its format loses `mode`; its output
does not move.

**Independent Test**: `cargo run -p monospace-cli` before and after the change prints the same
bytes, although `assets/demo.json` has changed and the format no longer has `mode`.

### Implementation for User Story 3

- [ ] T024 [US3] Add `monospace-diagram.workspace = true` to `crates/monospace-cli/Cargo.toml`,
      keeping the dependency on `monospace-core` for `Buffer`, `GlyphCatalog` and `render`
- [ ] T025 [US3] Delete the private `StampMode` enum and its `From` impl from
      `crates/monospace-cli/src/description.rs`, and drop the `mode` field from all three
      `ShapeDescription` variants (FR-017)
- [ ] T026 [US3] Point `Endpoint`'s conversion in `crates/monospace-cli/src/description.rs` at
      `monospace_diagram::Endpoint` instead of `monospace_core::Endpoint`
- [ ] T027 [US3] Replace `ShapeDescription::draw` with a conversion into `monospace_diagram::Shape`
      in `crates/monospace-cli/src/description.rs`, so no `monospace_core` shape and no `Layer` is
      constructed there any more (FR-016, FR-021)
- [ ] T028 [US3] Rewrite `Description::render` in `crates/monospace-cli/src/description.rs` to build
      a `Diagram`, add the `shapes` array in the order it is written, draw into a `Buffer` of the
      `canvas`, and render that buffer with the catalog it already unions (FR-016, FR-018, FR-019)
- [ ] T029 [US3] Update the module rustdoc in `crates/monospace-cli/src/description.rs` and the
      crate rustdoc in `crates/monospace-cli/src/main.rs`: the contract is now this feature's
      `contracts/description-format.md`, and what the application draws comes through
      `monospace-diagram`
- [ ] T030 [P] [US3] Remove every `"mode"` field from `crates/monospace-cli/assets/demo.json` and
      swap the two pairs whose second shape carried `mode: "below"` — the boxes at `{7,1}`/`{9,2}`
      and the boxes at `{18,0}`/`{20,1}` — so the box that decided the shared cells is written last
      (FR-020, research Q9)

### Tests for User Story 3

- [ ] T031 [US3] Update the unit-test fixtures in `crates/monospace-cli/src/description.rs` to the
      format without `mode`, and delete `an_unrecognized_mode_fails_to_deserialize` with the field
      it checked
- [ ] T032 [P] [US3] Update the fixtures in `crates/monospace-cli/tests/cli.rs` to the format
      without `mode`, including the doc comments on
      `the_two_crossings_read_from_whichever_figure_is_in_front` and
      `crossings_between_the_new_tables_mix_or_degrade_depending_on_table_coverage`, which explain a
      crossing in terms of the field that is gone
- [ ] T033 [US3] Make the overlap test in `crates/monospace-cli/tests/cli.rs` assert against the
      text the two corresponding core shapes stamped back to front with `Above` produce, rather than
      against a literal picture (TE-006, scenario 3)

**Checkpoint**: `cargo test -p monospace-cli` passes and `cargo run -p monospace-cli` still prints a
diagram. T024–T033 land as one `feat` commit.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [ ] T034 Run the whole gate: `cargo xtask check` is green, including `wasm` on three crates
      (SC-004)
- [ ] T035 Confirm SC-003 by observation: `cargo run -p monospace-cli > /tmp/demo-after.txt` and
      `diff /tmp/demo-before.txt /tmp/demo-after.txt` reports nothing (TE-007)
- [ ] T036 Confirm the `wasm` step reaches the new crate: put
      `#[cfg(target_arch = "wasm32")] compile_error!("reached");` in
      `crates/monospace-diagram/src/lib.rs`, watch `wasm` fail while `build` and `test` stay green,
      take it back out (principle IV, quickstart _The gate_) — verification, not a commit
- [ ] T037 Walk [quickstart.md](quickstart.md) end to end on a fresh `cargo build --workspace` and
      `cargo test --workspace`, and write a description file by
      [contracts/description-format.md](contracts/description-format.md) to see the format without
      `mode` render
- [ ] T038 Append the increment's entry to `docs/learning-log.md`: what was learned about Rust
      design, what was learned about working this way, and the TE-007 observation from T035 with
      what was compared (principle II)
- [ ] T039 Tick the boxes in this file that the work completed, and open the implementation pull
      request against `main`

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: T002 must run before any file changes — the "before" side of SC-003 exists
  only until the first edit.
- **Foundational (Phase 2)**: depends on Setup. Blocks all three stories.
- **User Story 1 (Phase 3)**: depends on Foundational, and commits with it.
- **User Story 2 (Phase 4)**: depends on User Story 1's `draw`. Adds no production code.
- **User Story 3 (Phase 5)**: depends on User Story 1's public API. Independent of User Story 2's
  tests.
- **Polish (Phase 6)**: depends on all three stories.

### Within Each Story

- T007 before T008 (the enum's `Arrow` holds `Endpoint`), T008 before T009, T009 before T010
- T011 before T012 before T013; T014 before every test from T015 on
- T024 before T025–T028; T025–T028 before T031–T033 (the fixtures follow the format)
- T030 is independent of the Rust changes but is only observable once T028 lands

### Parallel Opportunities

Genuinely few: almost every task in the new crate writes `diagram.rs` or `shape.rs`, and the tests
all live in `diagram.rs`'s test module, so they are sequential by file rather than by dependency.

- T005 and T006 touch different files once T003 and T004 exist
- T030 (`assets/demo.json`) alongside T025–T029 (`src/`)
- T032 (`tests/cli.rs`) alongside T031 (`src/description.rs`)

---

## Implementation Strategy

### MVP First

1. Phase 1 — the crate exists and the gate sees it
2. Phase 2 and Phase 3 — one commit: the crate holds shapes and draws them (SC-001)
3. **Stop and validate**: `cargo test -p monospace-diagram`, `cargo xtask check`

### Incremental Delivery

1. MVP → a diagram draws itself, tested
2. Plus User Story 2 → the order's meaning is pinned by tests (SC-002, SC-005)
3. Plus User Story 3 → the shipped application runs on it, output unmoved (SC-003)
4. Plus Polish → gate green on three crates, learning log appended

Each step leaves `cargo run -p monospace-cli` producing output, which is what principle II asks of
every commit that lands.

### One session per phase

Per `CLAUDE.md`, `/speckit-implement` re-reads what it needs from this directory. The three commit
groups above are three natural session boundaries.

---

## Notes

- [P] means a different file and no unfinished dependency
- Every commit here is `feat`: the crate is additive, and the CLI's switch changes the format
- TE-007 has nothing automatic behind it and is named as such in the spec; T035 is the whole of it
- `monospace-core` is not touched by any task in this file (FR-002)
