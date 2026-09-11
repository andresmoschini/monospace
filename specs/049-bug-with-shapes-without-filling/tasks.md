---
description: "Task list template for feature implementation"
---

# Tasks: Fix shapes without filling closing their inner arms

**Input**: Design documents from `/specs/049-bug-with-shapes-without-filling/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, quickstart.md

**Tests**: Included — the constitution's Testing constraint makes unit tests for core logic the
minimum any spec may ask for, and this feature is exactly a core-logic rendering rule.

**Organization**: One user story (P1) — the entire spec is this one bug. No Phase 4/5 exists because
there is no second or third story to schedule.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (e.g., US1)
- Exact file paths are included in every description

## Path Conventions

Single project, per plan.md: `crates/monospace-core/src/...` for the fix, `docs/model.md` for the
one design-doc correction. No `monospace-cli` file is touched.

---

## Phase 1: Setup

No setup tasks. The workspace, toolchain and gate already exist; this feature adds no dependency, no
new module and no new crate (plan.md, Technical Context).

---

## Phase 2: Foundational (Blocking Prerequisite)

**Purpose**: `docs/model.md` currently states the box's interior-closing rule two ways that disagree
with each other (research.md, "`docs/model.md` disagrees with itself"). The model owns the design,
so this is corrected before the code that follows it, per the constitution's "if a slice needs a
rule the model does not have, the model changes first."

- [x] T001 In `docs/model.md`, section 7 ("The initial set"), correct the box's description
      (currently: "`Set` along the run, `Closed` on the side facing its own interior, `Unset`
      outward") to state that the interior-facing side is `Closed` only when the box has a fill, and
      `Unset` otherwise — matching section 3 ("The cell")'s already-correct statement about a filled
      shape's border. See data-model.md's "`docs/model.md` — _The initial set_" entry for the exact
      wording to align.

**Checkpoint**: The model document is internally consistent again. User Story 1 work can now begin.

---

## Phase 3: User Story 1 - An unfilled shape lets a crossing stroke show through (Priority: P1) 🎯 MVP

**Goal**: A stroke crossing into an unfilled shape's interior renders as a crossing (`┼`), not a
closed junction — while a filled shape keeps closing that side exactly as today.

**Independent Test**: Render two overlapping unfilled boxes positioned so one's border crosses into
the other's interior at two cells; both cells must render `┼`. Give either box a fill and the cells
at which it is the one being entered must render the closed junction (`┴`, `├`, `┬`, `┤` as
appropriate), exactly as they do before this fix.

### Tests for User Story 1 ⚠️

> Write these first; both must fail against the code as it stands today (research.md's reverted
> probe already measured what "fail" looks like: `┴` and `┤` instead of `┼`).

- [x] T002 [US1] In `crates/monospace-core/src/shape/box_shape.rs`, add two tests to the existing
      `tests` module using only `BoxShape`'s current public API (no new field yet). First: two
      unfilled 6×4 boxes at `(0, 0)` and `(4, 2)` on a 9×5 buffer, both drawn `Above`, render `┼` at
      both cells where their borders overlap (acceptance scenario 1 and 3, SC-001, SC-003). Second:
      the same layout with the top box's `fill` set renders the closed junctions `┴` and `┤` at
      those same two cells instead of crossings, unchanged from today (FR-002, acceptance scenario
      2).

### Implementation for User Story 1

- [x] T003 [US1] In `crates/monospace-core/src/shape/fragment/border.rs`, give `Border` a new field,
      `closes_interior`, of type `bool`. In `Border::draw`'s `arm_toward`, change the branch for the
      side opposite `self.side` so it stamps `Arm::Closed` when `closes_interior` is true and
      `Arm::Unset` when it is false, instead of always `Arm::Closed`. Update this file's three
      existing `Border` test literals to set `closes_interior: true`, so they keep asserting today's
      filled-interior behavior unchanged. Add one new test in the same module asserting that
      `closes_interior: false` stamps `Arm::Unset` on the interior-facing side.
- [x] T004 [US1] In `crates/monospace-core/src/shape/box_shape.rs`, in `BoxShape::draw`, compute
      whether the box closes its interior from `self.fill.is_some()` once, and pass that value as
      `closes_interior` to all four `Border` placements (top, bottom, left, right). Depends on T003
      for the field to exist. This is what makes T002's two tests pass.

**Checkpoint**: T002's two tests pass; every other test in `border.rs` and `box_shape.rs` (77 total
before this feature) still passes unchanged, confirming FR-003 / SC-002.

---

## Phase 4: Polish & Cross-Cutting Concerns

- [x] T005 [P] Run `quickstart.md`'s validation by running `monospace-cli` against
      `specs/049-bug-with-shapes-without-filling/quickstart-example.json` (see quickstart.md for the
      exact command); it must print the crossing output quickstart.md records under "After the fix".
      Then add `"fill": "░"` to the second shape in that file and re-run: it must print the
      closed-junction output instead.
- [x] T006 [P] Run `cargo test --workspace` and confirm all pre-existing tests plus T002's and
      T003's new ones pass, with no other test's expected output changed (SC-002).
- [ ] T007 Append an entry to `docs/learning-log.md` for this increment, per constitution principle
      II: what was learned about Rust design (giving an existing fragment one more caller-supplied
      fact instead of inventing a new fragment or a post-hoc pass) and about working this way
      (measuring the bug and the fix with a reverted probe test before writing the plan, per
      principle IV).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: None — empty.
- **Foundational (Phase 2)**: No dependencies; BLOCKS Phase 3 (the model document must agree with
  itself before the code that follows it lands).
- **User Story 1 (Phase 3)**: Depends on Phase 2. T002 can be written independently of T003/T004 (it
  only calls `BoxShape`'s existing public API) but will fail until T004 lands. T003 before T004
  (T004 uses the field T003 adds).
- **Polish (Phase 4)**: Depends on Phase 3 being complete.

### Within User Story 1

T002 (tests, written first) → T003 (`Border` gets the field) → T004 (`BoxShape` wires it in, T002
now passes).

### Parallel Opportunities

Limited, by design: this is one small, sequential change to two files, not a feature wide enough to
split across contributors. T005 and T006 in Polish touch no source file and can run in either order
or together; nothing else is safely parallel (T002, T003 and T004 each depend on or are depended on
by their neighbor, and T001 precedes all of Phase 3 by the constitution's own rule, not merely by
convention).

---

## Implementation Strategy

### MVP First (and only)

This feature _is_ the MVP — one user story, one priority. Complete Phase 1 (nothing to do) → Phase 2
(T001) → Phase 3 (T002–T004) → Phase 4 (T005–T007), stop, and it is done: no second story to add
afterward.
