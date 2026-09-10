---
description: "Task list for feature 039, draw shapes instead of individual cells"
---

# Tasks: Draw shapes instead of individual cells

**Input**: Design documents from `/specs/039-draw-shapes-instead-of-individual-cells/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/public-api.md](contracts/public-api.md),
[quickstart.md](quickstart.md)

**Tests**: Included. The spec requests them explicitly — SC-001 asks for every acceptance picture,
SC-002 for the direction comparisons, SC-003 for every direction-family row, SC-004 and SC-006 for
two guards made to fail on purpose, SC-005 for what a test is allowed to know, SC-006 for the write
counter — and _Testing_ in the constitution makes unit tests for core logic the minimum any spec may
ask for.

**Tests are not separate tasks, and not written first.** _Demonstrable increments_ requires every
commit to leave the gate green, and a commit whose tests fail is not green. The tests for a piece of
behavior ride in the same task and the same commit as that behavior. What SC-001 through SC-006 pin
down is that each picture and each rule has a test naming it, not the order in which the two are
typed.

**Organization**: by user story, following the plan's four increments. One task per commit, per
_Demonstrable increments_, except the tasks marked as producing no commit.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies on incomplete tasks)
- **[Story]**: Which user story the task belongs to
- Exact file paths are in the descriptions

## Path Conventions

`crates/monospace-core/src/` holds the domain logic; `crates/monospace-cli/` is the consumer that
spends it. Tests live in `#[cfg(test)] mod tests` inside the module they cover, as every existing
module in the crate does. No new test file is added outside that convention.

---

## Phase 1: Setup

**Purpose**: capture the evidence FR-024 and SC-008 are measured against, before anything changes.
No dependency is added by this feature (Technical Context, plan.md) and the toolchain and Node
version already in place cover it, so there is nothing else to initialize — inventing a task here
would be the removal test in _One definition of green_ applied to a task instead of a check.

- [x] T001 Capture `cargo run -p monospace-cli`'s current output to `../draw-shapes-baseline.txt`,
      outside the repository, to diff against after the refactor commit that ends Phase 3

T001 produces no commit: its output is evidence for the pull request body, and committing it would
be a second copy of what `crates/monospace-cli/tests/cli.rs` already asserts. Skipping it turns "the
output did not move" from a measurement into a claim, which _Claims are measured, not assumed_
forbids.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: the two documents the _Preparatory work_ section of [plan.md](plan.md) says land before
the first line of shape code — the decision on the record, then the model amended for it. Neither
delivers anything a reader can see, and nothing in Phase 3 onward can start until both are done.

- [x] T002 Write ADR-0031 in `docs/decisions/0031-a-shape-draws-into-a-surface.md` — what a shape
      draws into, carrying the `Surface` trait's shape from research.md Q1 and the dispatch choice
      from Q3 — and add its row to `docs/decisions/README.md`
- [x] T003 Amend `docs/model.md` so a shape draws into a `Surface` rather than a buffer: the
      _Vocabulary_ row for `Shape`, the second paragraph of _Shapes_, and the last sentence of the
      fragment paragraph in _Complete and fragment_, and add `Surface`'s own row to _Vocabulary_

**T002 covers** principle VI: the decision is expensive to undo once three figures, six fragments
and a test surface are written against it, and it is exactly the kind of thing a reader asks "why is
this like this?" about. Status `accepted`, decision-makers the maintainer, dated the day it is
written. It comes first because every task after it depends on the shape it records.

**T003 is a docs-only commit** and touches no code. It is what _The model owns the design_ requires
before any task below may depend on `Surface`.

**Checkpoint**: the record and the model agree with each other and with research.md. No code exists
yet. US1 can begin.

---

## Phase 3: User Story 1 - A box draws itself (Priority: P1) 🎯 MVP

**Goal**: calling code describes a box — position, size, stroke, optional fill — and gets one drawn,
computing no position and writing no cell itself.

**Independent Test**: draw a 6×3 box into an empty buffer and compare the rendered text to the
picture in acceptance scenario 1; draw the same box filled and compare again; draw a 2×2 box and
confirm it is four corners and nothing else; ask for a box below 2 in either dimension and confirm
the buffer stays empty; draw each of those against a buffer that counts writes per position and
confirm no position is written more than once; then run `cargo run -p monospace-cli` and diff its
output against the T001 baseline.

- [ ] T004 [US1] Add `Side` to `crates/monospace-core/src/cell.rs` (`pub(crate)`); add `Direction`
      and `Orientation` to `crates/monospace-core/src/geometry.rs` (`pub`); add the `Surface` trait,
      the `Shape` trait and the `Layer<'a>` adapter to a new `crates/monospace-core/src/shape.rs`;
      re-export `Shape`, `Surface`, `Layer`, `Direction` and `Orientation` from
      `crates/monospace-core/src/lib.rs`, each with rustdoc
- [ ] T005 [US1] Add `crates/monospace-core/src/shape/fragment.rs` with the run-loop and
      rectangle-loop helpers the fragments in data-model.md's "The fragments" share, and declare the
      `fragment` and `shape` submodules from `shape.rs`
- [ ] T006 [P] [US1] Implement the `Corner` fragment (`pub(crate)`) in
      `crates/monospace-core/src/shape/fragment/corner.rs`: one cell, `Set` on the two `Side`s it
      opens toward and `Unset` on the other two, with a unit test per cell rule
- [ ] T007 [P] [US1] Implement the `Segment` fragment (`pub(crate)`) in
      `crates/monospace-core/src/shape/fragment/segment.rs`: a run, `Set` on both sides along it and
      `Unset` on the two across it, with a unit test
- [ ] T008 [P] [US1] Implement the `End` fragment (`pub(crate)`) in
      `crates/monospace-core/src/shape/fragment/end.rs`: one cell, `Set` on the one side the stroke
      runs toward and `Unset` on the other three, with a unit test — ADR-0029
- [ ] T009 [P] [US1] Implement the `Border` fragment (`pub(crate)`) in
      `crates/monospace-core/src/shape/fragment/border.rs`: a run, `Set` along it, `Closed` facing
      the interior side it derives from the one `Side` it is told, `Unset` outward, with a unit test
- [ ] T010 [P] [US1] Implement the `Fill` fragment (`pub(crate)`) in
      `crates/monospace-core/src/shape/fragment/fill.rs`: a rectangle written as `Cell::Literal`,
      with a unit test
- [ ] T011 [P] [US1] Implement the `Head` fragment (`pub(crate)`) in
      `crates/monospace-core/src/shape/fragment/head.rs`: one cell written as `Cell::Literal`, with
      a unit test — ADR-0029
- [ ] T012 [US1] Implement `BoxShape` (`pub`) in `crates/monospace-core/src/shape/box_shape.rs`:
      compose `Corner`, `Border` and `Fill` per data-model.md's decomposition table, with no pieces
      at all below 2 in either dimension (FR-021); add a test-only `Surface` that counts writes per
      position in `shape.rs`'s test module (FR-020); assert user story 1's scenarios 1 through 5 —
      the 6×3 box unfilled and filled, the 2×2 box, the undersized box drawing nothing, and the
      write count staying at 1 for all of them — with rustdoc saying `BoxShape` is complete
- [ ] T013 [US1] Refactor `crates/monospace-cli/src/main.rs` so `stamp_box` draws a `BoxShape`
      instead of its twelve hand-written stamps, leaving `crates/monospace-cli/tests/cli.rs`
      unedited

**T004 and T005 lay the skeleton** every fragment and figure depends on and change no existing
behavior — a `feat` commit, since nothing in the crate uses these types yet, but still one that must
compile and pass the gate.

**T006 through T011 are marked `[P]`**: six separate files, none naming another, each depending only
on T004 and T005. FR-029 is why none of them constructs a cell it was not told to: each derives the
one rule ADR-0028 assigns it and nothing else.

**T012 is one task rather than several** because `BoxShape` cannot be shown to draw anything until
its three fragments, its guard and its tests all exist together — the same reasoning feature 028's
T004 recorded for its sum type. Splitting it would mean shipping a box that draws corners but not
borders, which is a wrong behavior invented to make a commit boundary.

**T013 is the feature's `refactor` commit for this story**: it changes no output, by construction
per the byte-for-byte correspondence research.md already checked between `stamp_box`'s twelve stamps
and `BoxShape`'s decomposition, and it edits no test — `git show --stat` on this commit must show no
file under `crates/monospace-cli/tests/`. If it does, stop rather than argue the commit is
structural anyway.

**Checkpoint**: `cargo run -p monospace-cli` matches the T001 baseline byte for byte, and
`crates/monospace-core/src/shape/` compiles and is tested independently of the CLI. US1 is a
complete increment: a box draws itself even though only one call site has been moved onto it yet.

---

## Phase 4: User Story 2 - A straight line draws itself (Priority: P2)

**Goal**: calling code describes a line — position, length, orientation, stroke — and gets one
drawn, whose two outermost cells carry only the arm the line runs on.

**Independent Test**: draw a horizontal line of length 5 and compare the rendered text; confirm the
position one past the end holds no cell; draw a vertical line and compare; draw a horizontal and a
vertical line whose ends land on one position and confirm that position renders the corner the two
make, in both drawing orders and under both stamp modes; draw lines of length 2, 1 and 0 and confirm
each returns normally and writes no position more than once.

- [ ] T014 [US2] Implement `Line` (`pub`) in `crates/monospace-core/src/shape/line.rs`: an `End`
      toward the forward side, a `Segment` when the length allows one, an `End` toward the backward
      side, per data-model.md's decomposition table; assert scenarios 1, 2, 3, 5, 6, 7 and 8 of user
      story 2 — the horizontal and vertical pictures, the arm inspection at `(0, 0)`, lengths 2, 1
      and 0, and the write count via the counting surface from T012 — with rustdoc saying `Line` is
      complete
- [ ] T015 [US2] Add the join test to `crates/monospace-core/src/shape/line.rs`'s test module: a
      horizontal line and a vertical line whose ends land on `(0, 0)` render `┌` there, in both
      drawing orders and under both `StampMode`s (scenario 4) — what confirms ADR-0029

**T015 is a separate task from T014** because it exercises `Line` against itself rather than adding
anything to what `Line` draws: it is the scenario a caller-supplied end glyph could not satisfy, and
splitting it out is what makes that comparison visible in the log as its own commit.

**Checkpoint**: US1 and US2 both work independently, and no file `Line` needed was touched by adding
it — the first evidence for SC-007.

---

## Phase 5: User Story 3 - An arrow between two endpoints draws itself (Priority: P3)

**Goal**: calling code describes two endpoints, each a position, a leaving direction and a head
glyph, and gets an arrow: a head at each endpoint pointing outward and a derived route between them.

**Independent Test**: draw each of the nine pictured arrows into an empty buffer and compare the
rendered text; draw the first two, and separately the three that share endpoint positions `(2, 0)`
and `(8, 2)`, and confirm the texts within each group differ; for every row of the direction
families table, either compare against a picture or confirm the buffer is untouched; draw every
arrow above against the counting surface and confirm no position is written more than once.

- [ ] T016 [US3] Implement `Endpoint` and `Arrow` (both `pub`) in
      `crates/monospace-core/src/shape/arrow.rs` — deriving the route path per research.md Q5's
      lattice enumeration, fewest-bends and nearest-middle tie-break — and `Route` (`pub(crate)`) in
      `crates/monospace-core/src/shape/route.rs`, turning the derived path into `Corner` and
      `Segment` pieces per data-model.md's bend rule; assert user story 3's scenarios 1, 2, 4, 5, 6,
      7, 8, 9 and 10 — the nine pinned pictures — and scenarios 3 and 11, the two comparisons SC-002
      requires, with rustdoc saying `Arrow` is complete and `Route` is a compositor
- [ ] T017 [US3] Add the direction families table's two remaining rows to
      `crates/monospace-core/src/shape/arrow.rs`'s test module: the identical-directions row,
      covering both the geometry where a path fits and the one where the route comes out empty, with
      its expected picture produced by running the code rather than pinned in advance; and the
      same-position row (scenario 13), asserting only that the call returns normally — completing
      the seven-row coverage SC-003 requires
- [ ] T018 [US3] Add the write-count test to `crates/monospace-core/src/shape/arrow.rs`'s test
      module: every arrow above, drawn against the counting surface, reports a maximum of 1 per
      position, including at the positions where a route bends (scenario 12, FR-020)

**T016 is one task** for the same reason T012 was: `Arrow` cannot be shown to draw anything until
its path derivation, `Route` and its two heads all exist together, and SC-005 forbids a test that
inspects a raw path instead of a rendered picture, so there is no smaller commit whose tests assert
anything meaningful.

**T017's expected picture for the identical-directions row is written from a real run**, per _Claims
are measured, not assumed_ and per quickstart.md: write the test with an empty expectation, run it,
read what came out, and only then paste it in.

**Checkpoint**: all three user stories work independently. The direction families table has no row
without a test — SC-003 — and every figure in the spec has been drawn against the counting surface
at least once.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [ ] T019 Verify the box's one guard is reached: delete the "below 2 in either dimension" condition
      in `crates/monospace-core/src/shape/box_shape.rs`, confirm user story 1's fourth scenario test
      fails, then restore (SC-004)
- [ ] T020 Verify the write counter can fail: change `Route`'s draw in
      `crates/monospace-core/src/shape/route.rs` so two of its pieces share a bend, confirm the
      counting surface reports 2 at that position, then restore (SC-006)
- [ ] T021 Verify the gate on a fresh clone: `git clone` this repository elsewhere, run
      `cargo xtask setup && cargo xtask check` there, and confirm it passes (SC-009)
- [ ] T022 Append this increment's entry to `docs/learning-log.md`

T019 and T020 produce no commit and no test of their own. What they produce is an observation for
the pull request body: _Claims are measured, not assumed_ asks a guard to be shown reachable and a
counter to be shown able to fail, rather than assumed to work because the code compiles. T019 also
has a second use: if the guard turns out to reach nothing, FR-018 requires it be removed rather than
kept.

T021 produces no commit either. It is the fresh-clone run SC-009 asks for specifically because files
written by hand skip the transformations Git applies on checkout, so a working copy can be green
while the repository is broken.

T022 is what ends the increment, per _Demonstrable increments_. Candidates the increment actually
produced: what the lattice tie-break in research.md Q5 turned out to need in practice, what the
counting `Surface` caught or failed to catch, and whether the six-fragment split paid for itself the
way ADR-0028 predicted.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies, and it must happen before T013 or the baseline is worthless
- **Foundational (Phase 2)**: after T001, and it blocks every task in Phase 3 onward
- **US1 (Phase 3)**: after T003
- **US2 (Phase 4)**: after T012 and T004/T005, but not after T013 — `Line` shares no file with the
  CLI refactor and needs only the skeleton and the counting surface US1 built
- **US3 (Phase 5)**: after T004/T005 and after T006/T007 (`Corner` and `Segment`, which `Route`
  places); not after `Line` — US3 has no dependency on US2, per the spec's own priority ordering
- **Polish (Phase 6)**: T019 after T012; T020 after T016; T021 and T022 after every prior task

### Within and between the stories

T001 is independent of T002–T003. T004 and T005 gate T006 through T012. T006–T011 gate T012. T012
gates T013, T014 and T019. T016 gates T017, T018 and T020. T022 is last.

- T013 must not run before T001, or there is nothing to compare the output against.
- T012 needs every fragment T006–T011 add and the skeleton T004–T005 add.
- T014 needs `End` (T008) and `Segment` (T007), and the counting surface T012 introduces.
- T016 needs `Corner` (T006) and `Segment` (T007), and `Head` (T011).
- T019 needs the guard T012 writes; T020 needs the `Route` T016 writes.

### Parallel Opportunities

**T006 through T011**, the six fragments: different files, each depending only on T004 and T005 and
on none of the others. This is the feature's one real opportunity for several people or several
agents to work at once; every other task is ordered by what it composes.

---

## Implementation Strategy

### MVP first (US1 only)

1. T001, to have something to compare against.
2. T002, then T003 — the decision on the record before any code depends on it, and the model amended
   for it.
3. T004, T005, then T006–T011 (in parallel if staffed), then T012, each with `cargo xtask check`
   green and each its own commit.
4. T013, the refactor.
5. **Stop and validate**: diff `cargo run -p monospace-cli`'s output against the T001 baseline,
   confirm T013 touched no test, and confirm the counting surface reports 1 everywhere T012 draws.
6. This is a complete increment: a box draws itself, and the one place the repository draws a box
   already goes through it.

### Incremental delivery

1. Foundational → the decision recorded and the model amended, with no code yet.
2. US1 → a box draws itself, and `monospace-cli` proves it changes no output.
3. US2 → a line draws itself, touching no file US1 built (SC-007's first evidence).
4. US3 → an arrow draws itself, the largest slice and the one whose pictures are produced by running
   rather than by reading.
5. Polish → both guards shown reachable, the fresh clone shown green, the learning log appended, and
   the pull request opened with the diffs and the observations as its evidence.

### One task, one commit

Every task except T001, T019, T020 and T021 is one commit that leaves `cargo xtask check` green.
T013 is this feature's only `refactor` and edits no test; T002, T003 and T022 are `docs`; every
other task is `feat`. No task mixes a structural change with a behavioral one, so none of them owes
_Structural and behavioral change never share a commit_ an exception.
