---
description: "Task list for feature 099: an arrow draws the same whichever endpoint is named first"
---

# Tasks: An arrow draws the same whichever endpoint is named first

**Input**: Design documents from `/specs/099-bug-with-arrows/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/arrow-rendering.md,
quickstart.md

**Tests**: Requested by the spec's Testing expectations — each rule below has a test named against
it, plus the grid-sweep snapshot behind SC-001.

**Organization**: This feature is a bug fix, not a set of independent stories. Plan.md's _The order
of the work_ fixes one linear commit sequence — each step depends on the file state the previous one
left, and the two user stories interleave inside it (US2's construction is what US1's tie-break in
step 4 rounds). The phases below follow that agreed sequence exactly; each task still carries the
`[US1]`/`[US2]` label of the story it advances, for traceability, and the two structural/dependency
steps carry none, per the checklist format's own rule for such steps.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies) — none apply here: every step
  changes `arrow.rs` or depends on the step before it, in the order the plan fixes.
- **[Story]**: US1 = "The same arrow, described from either end" (P1). US2 = "A route turns where
  the model says it turns" (P2).
- Include exact file paths in descriptions.

## Path Conventions

Single Rust workspace. All production changes are inside
`crates/monospace-core/src/shape/{arrow.rs,route.rs}`; tests for the named scenarios and the sweep
live in `arrow.rs`'s existing `#[cfg(test)] mod tests`; the sweep's pinned pictures live in
`crates/monospace-core/src/snapshots/`.

---

## Phase 1: US1 — repair the run direction (fixes D1)

**Goal**: `Route::draw` stops drawing a run backwards out of its starting cell when the run travels
toward a smaller coordinate.

**Independent Test**: render the bug report's arrow from either end and check both pictures are
`◄─────►`.

- [x] T001 [US1] `fix(core)`: in `Route::draw` (`crates/monospace-core/src/shape/route.rs`), hand
      `Segment` each run's cells starting from the run's lower-coordinate end regardless of path
      direction, so a run that travels left or up no longer draws backwards out of its starting
      cell. Verify by making SC-002's rendering fail first (`◄────►` misplacement), then pass.

**Checkpoint**: SC-002 and User Story 1's acceptance scenarios 1 and 4 hold. `cargo xtask check` is
green; the CLI demonstration's output is unchanged.

---

## Phase 2: Foundational — name the derivation's vocabulary

**Purpose**: extract the concepts `data-model.md` names — starting position, route rectangle, middle
— out of today's `derive_path`, with today's values including today's rounding, so step 3 has
something to rewrite. Structural only: no test added, none changed, no picture moves.

**⚠️ CRITICAL**: this MUST land as its own `refactor` commit, separate from the behavioral commits
around it, per the constitution's rule that structural and behavioral change never share a commit.

- [x] T002 `refactor(core)`: extract the starting position (`s`, `t`), the route rectangle and the
      middle as named values inside `derive_path` (`crates/monospace-core/src/shape/arrow.rs:144`),
      keeping every value identical to today's. Confirm no picture moves: `cargo test --workspace`
      passes unchanged before committing.

**Checkpoint**: `cargo xtask check` is green; behavior is byte-for-byte identical to before T002.

---

## Phase 3: US2 — derive the path by construction (fixes D2)

**Goal**: a route with more than one bend is built directly from runs and fixed coordinates — pinned
by the endpoints, free ones taking the middle narrowed to the interval their neighbors leave open —
instead of searched for and ranked by `closeness`.

**Independent Test**: render the three arrangements in User Story 2's table (`n = 4, 5, 6`) and
check each turns at the middle of its route rectangle.

- [x] T003 [US2] `fix(core)`: replace `search`, `is_valid`, `count_bends`,
      `compare_lexicographically` and the `closeness`-scored candidate list in
      `crates/monospace-core/src/shape/arrow.rs` with the runs-and-fixed-coordinates construction
      research.md Q2 sets out, still rounding a free middle toward the smaller coordinate (today's
      rounding, unchanged by this step). Verify by making User Story 2's `n = 4` and `n = 5` rows
      fail first, then pass; confirm `n = 6` and every picture pinned by feature 039 are unchanged.

**Checkpoint**: SC-004's first half holds (`n = 4` turns at `x = 2`, `n = 5` at `x = 3`, `n = 6` at
`x = 3`). `cargo xtask check` is green.

---

## Phase 4: US1 — break a tied turn toward the leaving endpoint

**Goal**: where a free fixed coordinate's middle falls between two cells, the construction takes the
one nearer the `from` endpoint's starting position, per ADR-0044.

**Independent Test**: render the ADR-0044 pair (`(0, 1)` leaving `down`, `(2, 6)` leaving `up`) from
each end and check the turn moves from row 3 to row 4.

- [x] T004 [US1] `fix(core)`: in the construction from T003
      (`crates/monospace-core/src/shape/arrow.rs`), change the one value that rounds a free middle
      to round toward `s` (the `from` endpoint's starting position) instead of toward the smaller
      coordinate. Verify by making the ADR-0044 pair's `▼`-first rendering fail first (today it
      turns at row 3 instead of row 4), then pass.

**Checkpoint**: SC-003 and FR-003 hold. `cargo xtask check` is green.

---

## Phase 5: The named scenarios

**Purpose**: one test per rule named in `contracts/arrow-rendering.md`, each picture written a row
per source line rather than as one escaped string, per the spec's testing expectations.

- [x] T005 [US1] `test(core)`: add the bug report's scenario (SC-002) to
      `crates/monospace-core/src/shape/arrow.rs`'s test module — render from both ends, assert both
      pictures equal the row-per-line `◄─────►`.
- [x] T006 [US1] `test(core)`: add the ADR-0044 pair's scenario (SC-003) to the same test module —
      render from both ends, assert the two pictures are equal except for which of rows 3 and 4
      holds the turn.
- [x] T007 [US2] `test(core)`: add User Story 2's table (SC-004, `n = 4, 5, 6`) to the same test
      module — assert each arrangement's vertical run sits at `x = 2`, `x = 3` and `x = 3`
      respectively.
- [x] T008 [US2] `test(core)`: add a test pinning the shipped demonstration's turn (FR-006) to the
      same test module — compute the middle from its two endpoint positions (`x` 14 to 22,
      middle 18) and assert the route turns there, rather than asserting against a transcribed
      picture.
- [x] T009 [US1] `test(core)`: add a test pinning the colliding-heads cell (C-6, FR-004) to the same
      test module — both endpoints at one position, assert the rendered glyph is the `to` endpoint's
      head, without changing the behavior.

**Checkpoint**: all five named tests pass. `cargo xtask check` is green.

---

## Phase 6: Add the `insta` dev-dependency

- [x] T010 `build(core)`: add `insta = "1.48.0"` under `[dev-dependencies]` in
      `crates/monospace-core/Cargo.toml` (published 2026-06-11, already verified in research.md Q4
      as more than seven days old). Its own commit, so the addition is visible in the log and in
      `Cargo.lock`; report here any transitive dependency it pulls in that was published within the
      last week.

**Checkpoint**: `cargo xtask check` is green with the new dev-dependency present but unused by any
committed test yet.

---

## Phase 7: US1 — the grid sweep and its reviewed snapshot

**Goal**: every one of the 1856 renderings the spec counts is checked by a mechanical sweep
assertion and pinned as a snapshot reviewed once against _The route of an arrow_.

**Independent Test**: the sweep test renders the full grid from both ends and the snapshot review
confirms each picture against the model.

- [x] T011 [US1] `test(core)`: add the sweep over all 1856 renderings to
      `crates/monospace-core/src/shape/arrow.rs`'s test module, with mechanical assertions for C-2
      (both endpoint positions render their own head) and C-3 (no position is written twice, via the
      existing `CountingSurface`).
- [x] T012 [US1] `test(core)`: add the `insta` snapshot assertion (C-1, SC-001) over the same sweep
      in `crates/monospace-core/src/shape/arrow.rs`, run `cargo insta review` once, and commit the
      resulting `.snap` file under `crates/monospace-core/src/snapshots/` only after reading every
      picture against _The route of an arrow_ in `docs/model.md`.

**Checkpoint**: SC-001 holds. `cargo xtask check` is green.

---

## Phase 8: Polish

- [x] T013 `docs`: append the increment's entry to `docs/learning-log.md` — what was learned about
      Rust design and idiom (the runs-and-fixed-coordinates construction versus search-and-score),
      what was learned about working this way, and any trade-off worth remembering later.

---

## Dependencies & Execution Order

This feature has one execution order, not several parallel tracks — each phase's file state is what
the next phase edits:

- T001 → T002 → T003 → T004 → {T005, T006, T007, T008, T009} → T010 → {T011, T012} → T013.
- T002 (Foundational) blocks T003, because T003 rewrites the vocabulary T002 names.
- T003 blocks T004, because T004 changes one value inside the construction T003 introduces.
- T004 blocks Phase 5, because T005–T009 assert the post-fix pictures.
- T010 blocks T012, because T012 uses `insta`'s `assert_snapshot!`.
- T013 is last: the learning-log entry closes the increment.

### Within Phase 5

T005–T009 touch the same test module but assert independent scenarios; they are not marked `[P]`
because they land in one `test(core)` commit group per the plan, each ticked off as it is added.

## Implementation Strategy

### MVP first

T001 alone settles SC-002 — the whole bug report's headline case — and is demonstrable on its own
(`cargo run -p monospace-cli` unchanged, `cargo xtask check` green). Stop there if only the reported
defect needs fixing; the remaining phases settle User Story 2 and the standing regression check.

### Incremental delivery

Each phase ends at a checkpoint that is independently green and independently commit-able, per
principle II: a commit exists only where the tree is green, so each numbered task above is either
its own commit (where it reaches green alone) or grouped with its phase's other tasks (Phase 5) into
one commit that reaches green together.
