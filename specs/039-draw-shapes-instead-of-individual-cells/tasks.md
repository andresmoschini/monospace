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

- [x] T004 [US1] Add `Direction` and `Orientation` to `crates/monospace-core/src/geometry.rs`
      (`pub`); add the `Surface` trait, the `Shape` trait and the `Layer<'a>` adapter to a new
      `crates/monospace-core/src/shape.rs`; re-export `Shape`, `Surface`, `Layer`, `Direction` and
      `Orientation` from `crates/monospace-core/src/lib.rs`, each with rustdoc
- [x] T005 [US1] Add `Side` to `crates/monospace-core/src/cell.rs` (`pub(crate)`); add
      `crates/monospace-core/src/shape/fragment.rs` and its `corner`, `border` and `fill`
      submodules, implementing `Corner`, `Border` and `Fill` (all `pub(crate)`) per data-model.md's
      per-fragment cell rules, each with its own unit test; implement `BoxShape` (`pub`) in
      `crates/monospace-core/src/shape/box_shape.rs`, composing the three per data-model.md's
      decomposition table, with no pieces at all below 2 in either dimension (FR-021); add a
      test-only `Surface` that counts writes per position in `shape.rs`'s test module (FR-020);
      assert user story 1's scenarios 1 through 5 — the 6×3 box unfilled and filled, the 2×2 box,
      the undersized box drawing nothing, and the write count staying at 1 for all of them — with
      rustdoc saying `BoxShape` is complete and each fragment a fragment
- [x] T006 [US1] Refactor `crates/monospace-cli/src/main.rs` so `stamp_box` draws a `BoxShape`
      instead of its twelve hand-written stamps, leaving `crates/monospace-cli/tests/cli.rs`
      unedited

**T004 lays the skeleton** every fragment and figure depends on and changes no existing behavior — a
`feat` commit, since nothing in the crate uses these types yet, but still one that must compile and
pass the gate.

**T005 is one task rather than four** for a reason discovered while implementing, not planned in
advance: a `pub(crate)` item is dead code under `clippy::pedantic -D warnings` in the library's own
(non-test) compilation unless something outside its own unit test constructs it, and nothing but
`BoxShape` ever will construct `Corner`, `Border` or `Fill`. Building a fragment in isolation, ahead
of the figure that places it, cannot pass the gate no matter how it is tested — the same fact
[ADR-0028](../../docs/decisions/0028-give-each-fragment-its-own-cell-rule.md) names as the
vocabulary's cost, one layer more literal than that record anticipated. `Side` and the fragment
module tree are folded in for the same reason. `Corner`, `Border` and `Fill` remain three separate
types and three separate cell rules; only the commit boundary moves, not the design ADR-0028
recorded. `BoxShape` cannot be shown to draw anything until every one of its pieces, its guard and
its tests exist together either, which is the same reasoning feature 028's T004 recorded for its sum
type — splitting further would mean shipping a box that draws corners but not borders, a wrong
behavior invented to make a commit boundary. FR-029 is why none of the three fragments constructs a
cell it was not told to: each derives the one rule ADR-0028 assigns it and nothing else.

**T006 is the feature's `refactor` commit for this story**: it changes no output, by construction
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

- [x] T007 [US2] Add the `end` and `segment` submodules to
      `crates/monospace-core/src/shape/fragment.rs`, implementing `End` and `Segment` (both
      `pub(crate)`) per data-model.md's per-fragment cell rules, each with its own unit test —
      ADR-0029 for `End`; implement `Line` (`pub`) in `crates/monospace-core/src/shape/line.rs`: an
      `End` toward the forward side, a `Segment` when the length allows one, an `End` toward the
      backward side, per data-model.md's decomposition table; assert scenarios 1, 2, 3, 5, 6, 7 and
      8 of user story 2 — the horizontal and vertical pictures, the arm inspection at `(0, 0)`,
      lengths 2, 1 and 0, and the write count via the counting surface from T005 — with rustdoc
      saying `Line` is complete
- [x] T008 [US2] Add the join test to `crates/monospace-core/src/shape/line.rs`'s test module: a
      horizontal line and a vertical line whose ends land on `(0, 0)` render `┌` there, in both
      drawing orders and under both `StampMode`s (scenario 4) — what confirms ADR-0029

**T007 is one task for the same reason T005 was**: `End` and `Segment` are as dead as `Corner` was
until something outside a unit test constructs them, and `Line` is what does. It is `Line`'s own
decomposition table this time rather than `BoxShape`'s, but the constraint that forces one commit is
the same one, not a second instance of a different rule.

**T008 is a separate task from T007** because it exercises `Line` against itself rather than adding
anything to what `Line` draws: it is the scenario a caller-supplied end glyph could not satisfy, and
splitting it out is what makes that comparison visible in the log as its own commit. `Line` already
exists after T007, so this commit adds no fragment and raises no dead-code question.

**Checkpoint**: US1 and US2 both work independently, and no file US1 built was touched adding US2 —
the first evidence for SC-007.

---

## Phase 5: User Story 3 - An arrow between two endpoints draws itself (Priority: P3)

**Goal**: calling code describes two endpoints, each a position, a leaving direction and a head
glyph, and gets an arrow: a head at each endpoint pointing outward and a derived route between them.

**Independent Test**: draw each of the nine pictured arrows into an empty buffer and compare the
rendered text; draw the first two, and separately the three that share endpoint positions `(2, 0)`
and `(8, 2)`, and confirm the texts within each group differ; for every row of the direction
families table, either compare against a picture or confirm the buffer is untouched; draw every
arrow above against the counting surface and confirm no position is written more than once.

- [ ] T009 [US3] Add the `head` submodule to `crates/monospace-core/src/shape/fragment.rs`,
      implementing `Head` (`pub(crate)`) per data-model.md's cell rule, with a unit test — ADR-0029;
      implement `Endpoint` and `Arrow` (both `pub`) in `crates/monospace-core/src/shape/arrow.rs` —
      deriving the route path per research.md Q5's lattice enumeration, fewest-bends and
      nearest-middle tie-break — and `Route` (`pub(crate)`) in
      `crates/monospace-core/src/shape/route.rs`, placing `Corner` and `Segment` per data-model.md's
      bend rule; assert user story 3's scenarios 1, 2, 4, 5, 6, 7, 8, 9 and 10 — the nine pinned
      pictures — and scenarios 3 and 11, the two comparisons SC-002 requires, with rustdoc saying
      `Arrow` is complete and `Route` is a compositor
- [ ] T010 [US3] Add the direction families table's two remaining rows to
      `crates/monospace-core/src/shape/arrow.rs`'s test module: the identical-directions row,
      covering both the geometry where a path fits and the one where the route comes out empty, with
      its expected picture produced by running the code rather than pinned in advance; and the
      same-position row (scenario 13), asserting only that the call returns normally — completing
      the seven-row coverage SC-003 requires
- [ ] T011 [US3] Add the write-count test to `crates/monospace-core/src/shape/arrow.rs`'s test
      module: every arrow above, drawn against the counting surface, reports a maximum of 1 per
      position, including at the positions where a route bends (scenario 12, FR-020)

**T009 is one task**, for the same dead-code reason as T005 and T007 and also because `Arrow` cannot
be shown to draw anything until its path derivation, `Route` and its two heads all exist together;
SC-005 forbids a test that inspects a raw path instead of a rendered picture, so there is no smaller
commit whose tests would assert anything meaningful even if the dead-code constraint allowed one.

**T010's expected picture for the identical-directions row is written from a real run**, per _Claims
are measured, not assumed_ and per quickstart.md: write the test with an empty expectation, run it,
read what came out, and only then paste it in.

**Checkpoint**: all three user stories work independently. The direction families table has no row
without a test — SC-003 — and every figure in the spec has been drawn against the counting surface
at least once.

---

## Phase 6: Polish & Cross-Cutting Concerns

- [ ] T012 Verify the box's one guard is reached: delete the "below 2 in either dimension" condition
      in `crates/monospace-core/src/shape/box_shape.rs`, confirm user story 1's fourth scenario test
      fails, then restore (SC-004)
- [ ] T013 Verify the write counter can fail: change `Route`'s draw in
      `crates/monospace-core/src/shape/route.rs` so two of its pieces share a bend, confirm the
      counting surface reports 2 at that position, then restore (SC-006)
- [ ] T014 Verify the gate on a fresh clone: `git clone` this repository elsewhere, run
      `cargo xtask setup && cargo xtask check` there, and confirm it passes (SC-009)
- [ ] T015 Append this increment's entry to `docs/learning-log.md`

T012 and T013 produce no commit and no test of their own. What they produce is an observation for
the pull request body: _Claims are measured, not assumed_ asks a guard to be shown reachable and a
counter to be shown able to fail, rather than assumed to work because the code compiles. T012 also
has a second use: if the guard turns out to reach nothing, FR-018 requires it be removed rather than
kept.

T014 produces no commit either. It is the fresh-clone run SC-009 asks for specifically because files
written by hand skip the transformations Git applies on checkout, so a working copy can be green
while the repository is broken.

T015 is what ends the increment, per _Demonstrable increments_. Candidates the increment actually
produced: the dead-code constraint that collapsed each fragment's commit into its figure's, which
this file did not anticipate and had to be rewritten around mid-implementation; what the lattice
tie-break in research.md Q5 turned out to need in practice; what the counting `Surface` caught or
failed to catch; and whether the six-fragment split paid for itself the way ADR-0028 predicted, once
the unit of delivery turned out to be the figure rather than the fragment.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies, and it must happen before T006 or the baseline is worthless
- **Foundational (Phase 2)**: after T001, and it blocks every task in Phase 3 onward
- **US1 (Phase 3)**: after T003
- **US2 (Phase 4)**: after T004, but not after T005 or T006 — `Line` shares no file with `BoxShape`
  or with the CLI refactor and needs only the skeleton T004 built
- **US3 (Phase 5)**: after T004; not after `Line` — US3 has no dependency on US2, per the spec's own
  priority ordering
- **Polish (Phase 6)**: T012 after T005; T013 after T009; T014 and T015 after every prior task

### Within and between the stories

T001 is independent of T002–T003. T004 gates T005, T007 and T009. T005 gates T006 and T012. T007
gates T008. T009 gates T010, T011 and T013. T015 is last.

- T006 must not run before T001, or there is nothing to compare the output against.
- T012 needs the guard T005 writes; T013 needs the `Route` T009 writes.

### Parallel Opportunities

**None.** Every fragment is folded into the commit of the one figure that constructs it outside a
test, so there is no fragment left to build ahead of or alongside anything else. T005, T007 and T009
each depend only on T004 and are otherwise independent of one another — a team of three could start
all three the moment T004 lands — but within this session they run in sequence.

---

## Implementation Strategy

### MVP first (US1 only)

1. T001, to have something to compare against.
2. T002, then T003 — the decision on the record before any code depends on it, and the model amended
   for it.
3. T004, then T005, each with `cargo xtask check` green and each its own commit.
4. T006, the refactor.
5. **Stop and validate**: diff `cargo run -p monospace-cli`'s output against the T001 baseline,
   confirm T006 touched no test, and confirm the counting surface reports 1 everywhere T005 draws.
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

Every task except T001, T012, T013 and T014 is one commit that leaves `cargo xtask check` green.
T006 is this feature's only `refactor` and edits no test; T002, T003 and T015 are `docs`; every
other task is `feat`. No task mixes a structural change with a behavioral one, so none of them owes
_Structural and behavioral change never share a commit_ an exception.
