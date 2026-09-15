---
description: "Task list for feature 080: a shape has an identity, and the order can change"
---

# Tasks: A shape in a diagram has an identity, and the order can change

**Input**: Design documents from `/specs/080-a-shape-in-a-diagram-has-an-identity-and/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [data-model.md](data-model.md),
[contracts/diagram-api.md](contracts/diagram-api.md), [quickstart.md](quickstart.md)

**Tests**: Requested by the spec's Testing expectations (TE-001..TE-008) and by the constitution's
minimum of unit tests for core logic; included below.

**Organization**: Tasks are grouped by user story. Per plan.md's Constitution Check (principle V),
the one structural change is its own `refactor` commit before any `feat` work begins.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)

## Path Conventions

Rust cargo workspace. `crates/monospace-diagram/src/` for the diagram library,
`crates/monospace-cli/src/` and `crates/monospace-cli/tests/` for the CLI, per plan.md's Project
Structure.

---

## Phase 1: Setup

No setup is needed: no dependency is added, no crate is added, and the toolchain is unchanged
(plan.md Technical Context).

---

## Phase 2: Foundational — capture the "before" picture

**Purpose**: FR-014 claims the first picture printed after this feature is byte for byte what the
CLI printed before it. That claim needs a "before" artifact to diff against once the feature is
built.

- [ ] T001 Run `cargo run -p monospace-cli > /tmp/demo-before.txt` and keep the file until Phase 6
      (quickstart.md "Before touching anything")

**Checkpoint**: baseline captured. User story work can begin.

---

## Phase 3: User Story 1 - A shape can be named after it has been placed (Priority: P1) 🎯 MVP

**Goal**: `Diagram::add` generates a unique, readable identity for each shape and hands it back;
nothing else about adding changes.

**Independent Test**: add three shapes to a diagram, keep what each addition hands back, and check
the three are different from one another (spec.md Independent Test, US1).

### Implementation for User Story 1

- [x] T002 [US1] Add `ShapeId`, a newtype over a private `String`, with a `Display` impl writing
      `#1`, `#2`, and so on, and derives `Clone, Debug, PartialEq, Eq` (no `Copy`, no constructor,
      no `From<&str>`, no `FromStr`, no accessor) in `crates/monospace-diagram/src/diagram.rs`, and
      rustdoc it as it is introduced (FR-001, FR-003, FR-006; data-model.md `ShapeId`;
      contracts/diagram-api.md `ShapeId`)
- [x] T003 [US1] Add `Placed`, a crate-private struct pairing an `id: ShapeId` with a
      `shape: Shape`, in `crates/monospace-diagram/src/diagram.rs` (data-model.md `Placed`)
- [x] T004 [US1] Change `Diagram`'s private fields from `shapes: Vec<Shape>` to
      `shapes: Vec<Placed>` plus `next: u32`, keeping `#[derive(Default)]` giving an empty diagram
      with a counter of 0, in `crates/monospace-diagram/src/diagram.rs` (data-model.md `Diagram`)
- [x] T005 [US1] Change `add(&mut self, shape: Shape)` to return `ShapeId`: increment `next`, build
      the identity's text from the new value, push `Placed { id, shape }` onto the end of `shapes`
      (still the front of the order, unchanged from spec 079), and return a clone of the identity;
      rustdoc the new return value, in `crates/monospace-diagram/src/diagram.rs` (FR-002, FR-005,
      FR-006; data-model.md `add`; contracts/diagram-api.md `add`)
- [x] T006 [US1] Update `draw` to visit `Placed` values and draw `placed.shape`, unchanged in
      behavior, in `crates/monospace-diagram/src/diagram.rs` (data-model.md `draw`)
- [x] T007 [US1] Re-export `ShapeId` alongside `Diagram` in `crates/monospace-diagram/src/lib.rs`
      (plan.md Project Structure)
- [x] T008 [P] [US1] Unit test: adding three shapes to one diagram yields three identities that
      differ from one another and read as `#1`, `#2`, `#3` in the order added, in
      `crates/monospace-diagram/src/diagram.rs` tests module (TE-001)
- [x] T009 [P] [US1] Unit test: adding the same shape value twice yields two different identities,
      in `crates/monospace-diagram/src/diagram.rs` tests module (spec.md US1 scenario 3)

**Checkpoint**: User Story 1 is fully functional and testable independently —
`cargo test -p monospace-diagram` passes, identities are generated and returned.

---

## Phase 4: User Story 2 - A shape moves one place toward the front or the back (Priority: P1)

**Goal**: `forward` and `backward` move the shape named by an identity one place in the order, never
fail, and touch nothing else.

**Independent Test**: build a diagram from two partially overlapping opaque boxes, draw it, move the
back one forward by its identity, draw again, and compare the two buffers (spec.md Independent Test,
US2).

### Implementation for User Story 2

- [x] T010 [US2] Add `forward(&mut self, id: &ShapeId)` to `Diagram`: find the index of the entry
      whose `id` equals the argument (return unchanged if none), return unchanged if it is already
      the last element, otherwise swap it with the element after it; rustdoc it, in
      `crates/monospace-diagram/src/diagram.rs` (FR-007, FR-009, FR-010, FR-011, FR-006;
      data-model.md `forward`/`backward`; contracts/diagram-api.md `forward`)
- [x] T011 [US2] Add `backward(&mut self, id: &ShapeId)` to `Diagram`: same lookup, return unchanged
      if it is already the first element, otherwise swap it with the element before it; rustdoc it,
      in `crates/monospace-diagram/src/diagram.rs` (FR-008, FR-009, FR-010, FR-011, FR-006;
      data-model.md `forward`/`backward`; contracts/diagram-api.md `backward`)
- [x] T012 [P] [US2] Unit test: two partially overlapping opaque boxes, drawn before and after the
      back one moves forward, produce different buffers, and the second equals what the same two
      boxes added in the opposite order produce, in `crates/monospace-diagram/src/diagram.rs` tests
      module, reusing the `cells` helper (TE-002)
- [x] T013 [P] [US2] Unit test: the same expected buffer comes from moving the front one backward
      instead, against the same two boxes, in `crates/monospace-diagram/src/diagram.rs` tests module
      (TE-003)
- [x] T014 [P] [US2] Unit test: moving the front-most forward and moving the back-most backward each
      leave the drawn buffer unchanged, in `crates/monospace-diagram/src/diagram.rs` tests module
      (TE-004; spec.md US2 scenarios 3-4; edge case: a diagram holding one shape)
- [x] T015 [P] [US2] Unit test: naming an identity the diagram does not hold — one kept from another
      diagram — leaves the drawn buffer unchanged through both `forward` and `backward`, with no
      panic, in `crates/monospace-diagram/src/diagram.rs` tests module (TE-005; spec.md US2 scenario
      5; edge cases: identity from another diagram, empty diagram)
- [x] T016 [P] [US2] Unit test: a shape moved forward and then backward by the same identity draws
      exactly what it drew at the start, in `crates/monospace-diagram/src/diagram.rs` tests module
      (TE-006; spec.md US2 scenario 6)
- [x] T017 [P] [US2] Unit test: two unfilled boxes with the same stroke whose shared cells every
      side leaves `Unset` draw identically before and after a reorder, in
      `crates/monospace-diagram/src/diagram.rs` tests module (edge case: overlap with no cell in
      common)

**Checkpoint**: User Stories 1 AND 2 both work independently — `cargo test -p monospace-diagram`
covers identity generation and every order-changing property.

---

## Phase 5: User Story 3 - The shipped demonstration shows a reorder (Priority: P2)

**Goal**: the CLI renders a description, moves the back-most shape one place forward by its
identity, and renders again, each picture captioned.

**Independent Test**: run the application with no arguments; two captioned pictures appear,
identical except in the top-left pair of overlapping opaque boxes (spec.md Independent Test, US3).

### Implementation for User Story 3

- [x] T018 [US3] `refactor` commit: remove `Description::buffer` and move its one line,
      `Buffer::new(origin, size)`, to the call site in `render_description`, changing no behavior
      and adding no test, in `crates/monospace-cli/src/description.rs` and
      `crates/monospace-cli/src/main.rs` (plan.md Constitution Check principle V; data-model.md
      `Description::buffer` — removed)
- [ ] T019 [US3] Change `Description::into_diagram` to return `(Diagram, Option<ShapeId>)`: the
      diagram built from `shapes` in order, and the identity of the first entry (`None` when
      `shapes` is empty), in `crates/monospace-cli/src/description.rs` (FR-015; data-model.md
      `Description::into_diagram`)
- [ ] T020 [US3] Rewrite `render_description` in `crates/monospace-cli/src/main.rs` to: build the
      window, call `into_diagram` for the diagram and the back-most identity, print a caption, draw
      into a buffer and render the first picture, print a blank line, call `forward` on the
      back-most identity when it is `Some` (skipping the move when it is `None`), print a second
      caption, draw into a second buffer and render the second picture (FR-013, FR-014, FR-015,
      FR-018; data-model.md `render_description`)
- [ ] T021 [US3] Add the comment FR-016 requires at the `forward` call in
      `crates/monospace-cli/src/main.rs`, stating that moving the first entry is a
      demonstration-only assumption relying on the shipped demonstration's first two entries being
      two partially overlapping opaque boxes (FR-016; data-model.md "comment FR-016 asks for")
- [ ] T022 [US3] Update the existing subprocess test in `crates/monospace-cli/tests/cli.rs` that
      pinned the whole run to instead read the first of the two pictures, unchanged in what it
      asserts about that picture (FR-014; quickstart.md "Build and test the workspace")
- [ ] T023 [P] [US3] Add a new subprocess test in `crates/monospace-cli/tests/cli.rs` on a
      description of two partially overlapping opaque boxes: it prints two captioned pictures, the
      first equal to the two boxes in the order written and the second equal to the two boxes in the
      opposite order, found by the blank line between them and pinning neither caption's wording
      (TE-007)
- [ ] T024 [P] [US3] Add a subprocess test in `crates/monospace-cli/tests/cli.rs` on a description
      holding fewer than two shapes (including an empty one): the two pictures are identical and the
      run succeeds (spec.md US3 scenario 4; edge case: no shapes)

**Checkpoint**: all user stories are independently functional — `cargo test -p monospace-cli`
passes, and the CLI demonstrates a reorder.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [ ] T025 Run `cargo run -p monospace-cli > /tmp/demo-after.txt` and
      `diff /tmp/demo-before.txt /tmp/demo-after.txt`; confirm the diff is only the two new caption
      lines, a blank line, and the picture printed a second time (FR-014; quickstart.md)
- [ ] T026 Read the two pictures from the run above and confirm they differ only in the top-left
      pair of overlapping boxes, where the box in front is the other one; this is TE-008 and SC-004,
      accepted on observation rather than pinned by a test
- [ ] T027 Run `cargo xtask check` and confirm it is green, including the `wasm` step covering
      `monospace-diagram` (SC-006)
- [ ] T028 Append an entry to `docs/learning-log.md` for this increment: what was learned about Rust
      design and idiom, what was learned about working this way, and the TE-008/SC-004 observation
      from T026 (constitution principle II)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Foundational (Phase 2)**: No dependencies — capture the baseline before any code changes.
- **User Story 1 (Phase 3)**: Depends on Phase 2. No dependency on other stories.
- **User Story 2 (Phase 4)**: Depends on Phase 3 (`ShapeId` and `Placed` must exist before `forward`
  and `backward` can be added).
- **User Story 3 (Phase 5)**: Depends on Phase 4 (the CLI moves a shape by its identity, so
  `Diagram::forward` must exist).
- **Polish (Phase 6)**: Depends on Phase 5 completing.

Unlike the template's default, these three stories cannot run in parallel: US2 needs the type US1
introduces, and US3 needs the method US2 introduces. Priorities (P1, P1, P2) set the order within
that chain rather than a choice of which to build first.

### Within Each User Story

- US1: the type and field changes (T002-T004) before `add` (T005) before `draw` (T006) before the
  re-export (T007); tests (T008-T009) after `add` exists.
- US2: `forward` and `backward` (T010-T011) can be written together since they touch the same file
  but are independent methods; tests (T012-T017) after both exist.
- US3: the `refactor` commit (T018) first and alone, per principle V; then `into_diagram` (T019),
  then `render_description` (T020), then the comment (T021); the existing test update (T022) after
  T020; new tests (T023-T024) after T020.

### Parallel Opportunities

- T008-T009 (US1 tests) in parallel once T005 lands.
- T012-T017 (US2 tests) in parallel once T010-T011 land.
- T023-T024 (US3 new tests) in parallel once T020 lands.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 2: capture the baseline.
2. Phase 3: User Story 1 — identities exist and are returned.
3. **STOP and VALIDATE**: `cargo test -p monospace-diagram` passes; three added shapes yield three
   different identities.

### Incremental Delivery

1. Phase 2 → baseline captured.
2. Phase 3 → identities generated and handed back (MVP).
3. Phase 4 → a shape can be moved by its identity, observed by drawing.
4. Phase 5 → the shipped CLI demonstrates the reorder.
5. Phase 6 → the gate is green, the observation is recorded, the increment is closed.

Each commit follows plan.md's Constitution Check: one `refactor` commit (T018) with no behavior
change and no test, everything else `feat`, and every commit leaves `cargo xtask check` green
(constitution principle II).
