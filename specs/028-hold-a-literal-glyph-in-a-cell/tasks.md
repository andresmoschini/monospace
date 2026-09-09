---
description: "Task list for feature 028, hold a literal glyph in a cell"
---

# Tasks: Hold a literal glyph in a cell

**Input**: Design documents from `/specs/028-hold-a-literal-glyph-in-a-cell/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/](contracts/public-api.md),
[quickstart.md](quickstart.md)

**Tests**: Included. The spec requests them explicitly — SC-003 asks for a test per composition row
that mentions a literal, SC-004 for the equivalence of the two stamp orders, SC-005 for the existing
assertions keeping their values, SC-001 for the front end's whole output — and _Testing_ in the
constitution makes unit tests for core logic the minimum any spec may ask for.

**Tests are not separate tasks, and not written first.** The template's default asks for failing
tests before implementation; that cannot hold here, because _Demonstrable increments_ requires every
commit to leave the gate green, and a commit whose tests fail is not green. The tests for a behavior
ride in the same task and the same commit as the behavior. What the spec pins down is that each rule
has a test naming it, not the order in which the two are typed.

**Organization**: by user story. One task per commit, per _Demonstrable increments_, except the
three tasks marked as producing no commit.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story the task belongs to
- Exact file paths are in the descriptions

## Path Conventions

`crates/monospace-core/src/` holds the domain logic and `crates/monospace-cli/` is the consumer that
spends it. Tests live in `#[cfg(test)] mod tests` inside the module they cover, as every existing
module does, and the front end's whole output is asserted from `crates/monospace-cli/tests/cli.rs`.
No test file is added.

---

## Phase 1: Setup

**Purpose**: capture the evidence FR-004 and SC-005 are measured against, before anything changes.

- [x] T001 Capture the front end's current output to `../literal-baseline.txt`, outside the
      repository, following _Capture the baseline first_ in [quickstart.md](quickstart.md)

T001 produces no commit: its output is evidence for the pull request body, and committing it would
be a second copy of what `crates/monospace-cli/tests/cli.rs` already asserts. It is a task because
skipping it turns "the output did not move" from a measurement into a claim, which is what _Claims
are measured, not assumed_ forbids.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: the decision on the record, and the rename that makes the change easy — neither of them
delivers anything a reader can see, and nothing else can start until both are done.

- [x] T002 Write ADR-0026 in
      `docs/decisions/0026-represent-a-cell-as-a-sum-of-strokes-and-a-literal.md` and add its row to
      `docs/decisions/README.md`
- [x] T003 Rename `Cell` to `StrokeCell` in `crates/monospace-core/src/cell.rs`, leave
      `pub type Cell = StrokeCell;` in its place, re-export `StrokeCell` from
      `crates/monospace-core/src/lib.rs`, and move `merge` and `key_of` onto the new name in
      `crates/monospace-core/src/buffer.rs` and `crates/monospace-core/src/render.rs`

**T002 covers** principle VI. Its material is [research.md](research.md), R1: the four options with
their trade-offs, the concrete invalid value the chosen one makes unrepresentable — a cell that is a
letter and also has a stroke reaching its top side — and the maintainer's driver, which is wider
than this feature: as far as possible the model should make invalid states impossible to represent.
Status `accepted`, decision-makers the maintainer, dated the day it is written. It comes first
because every task after it depends on the shape it records.

**T003 is the only structural commit in the feature**, so it is `refactor` and it edits no test at
all — that is what the alias is for, and [research.md](research.md), R7 records the compile check
behind it. `git show --stat` on this commit must show no file under a `tests` module or `tests/`
directory. If it does, stop rather than argue the commit is structural anyway.

**Checkpoint**: the gate is green, the front end's output is unchanged by construction, and no test
was touched. US1 can begin.

---

## Phase 3: User Story 1 - A cell can hold a chosen glyph (Priority: P1) 🎯 MVP

**Goal**: a position can hold one chosen glyph, which renders as itself, refuses every junction, and
hides what a figure behind it would have drawn — with the two stamp orders still equivalent.

**Independent Test**: stamp a chosen glyph into an empty buffer and read it back out of the rendered
text; stamp a figure and a chosen glyph onto one position in both modes and both orders and read the
cells back; stamp three figures with a chosen glyph in the middle both ways and compare the buffers;
then diff the front end's output against the T001 baseline.

- [x] T004 [US1] Make `Cell` the sum of `Strokes(StrokeCell)` and `Literal(Glyph)` in
      `crates/monospace-core/src/cell.rs`, deleting the alias and adding
      `From<StrokeCell> for Cell`, the widened `is_decided` and their rustdoc; add the literal's
      case to `merge` in `crates/monospace-core/src/buffer.rs`; add the literal's branch and the
      unified lifetime to `crates/monospace-core/src/render.rs`; adapt every construction site and
      add the tests

**T004 covers**: FR-001 to FR-003, FR-005 to FR-009, FR-011 and FR-012, with FR-004 measured by the
baseline diff at the checkpoint. In detail:

- The sum, `From` and the derives are [contracts/public-api.md](contracts/public-api.md)'s _Added_
  section; `is_decided` answers `true` for a literal, which is what makes three of the five
  composition rows fall out of branches that already exist ([research.md](research.md), R2).
- `merge` gains one case — a literal in the bottom role contributing four `Closed` sides — and stays
  total, returning a literal that arrives on top rather than reaching for `unreachable!()`
  ([research.md](research.md), R3). The four `Closed` sides live in the merge, not in the type.
- `render` answers a literal with its own text, builds no key for it, and unifies the buffer's
  borrow with the catalog's so both can be returned ([research.md](research.md), R4). `key_of`
  narrows to a stroke cell.
- Its tests are the rows of _Stamping_ that mention a literal, per stamp mode (SC-003); a literal
  rendering as itself with a catalog that has no rule for it; `is_decided` for both kinds; and the
  three-figure stack asserting both walk orders end at the same cell (SC-004). Existing assertions
  are adapted mechanically and keep their expected values (SC-005).
- Rustdoc on `Cell` says which of the two kinds wins where they meet, and `is_decided`'s says a
  literal is decided by definition (FR-011).

**Why this is one task and not three.** The moment `Cell` is an enum, the merge and the renderer
have to answer for both kinds or the crate does not compile, so there is no smaller commit that is
green. Splitting it would mean shipping a renderer that draws a literal as a space, which is a wrong
behavior invented to make a commit boundary.

**Checkpoint**: the gate is green, the front end's output still matches the T001 baseline byte for
byte, and `crates/monospace-cli/` is absent from the diff. US1 is a complete increment: the library
can hold a chosen glyph even if nothing on the terminal shows it yet.

---

## Phase 4: User Story 2 - The terminal shows both halves at once (Priority: P2)

**Goal**: the single box has a filled interior, and the overlapping pair shows occlusion next to the
junctions it already showed, once per stamp mode.

**Independent Test**: run `cargo run -p monospace-cli` and compare the whole output, trailing spaces
included, against the assertion in `crates/monospace-cli/tests/cli.rs`.

- [ ] T005 [US2] Stamp the two interior positions of the box with a literal `░` in
      `crates/monospace-cli/src/main.rs`, keeping the box built once and stamped twice, and update
      the asserted output in `crates/monospace-cli/tests/cli.rs` from a real run

**T005 covers**: FR-010, and SC-001, SC-002 and SC-007. The fill is part of the same `stamp_box`
function, so the box is still built once and stamped twice, as spec 0002's acceptance list required.
The glyph comes from `Glyph::new("░").expect(..)` with a message saying why it cannot fail
([research.md](research.md), R6).

**The asserted string is written from what the binary prints**, not from the input draft. The
draft's three pictures are derived and it says so; [quickstart.md](quickstart.md) keeps them as a
prediction to compare against. If the run and the prediction agree, say so in the commit message. If
they differ, _Stamping_ in [`docs/model.md`](../../docs/model.md) decides which is right, and a real
disagreement with the model is a stop-and-report rather than an edit to the expected string.

**Checkpoint**: the gate is green, the two pair pictures differ from each other and from the T001
baseline, and each shows a fill covering a border of the other box in one and the border surviving
in the other.

---

## Phase 5: Polish & Cross-Cutting Concerns

- [ ] T006 Make the equivalence test fail on purpose by letting a literal contribute `Unset` instead
      of `Closed` in `crates/monospace-core/src/buffer.rs`, then restore, following _Verify the
      equivalence test actually bites_ in [quickstart.md](quickstart.md)
- [ ] T007 Delete each `is_decided` shortcut from `Buffer::stamp` in
      `crates/monospace-core/src/buffer.rs` in turn, confirm the suite still passes, then restore,
      following _Verify the two shortcuts are still optimizations_ in [quickstart.md](quickstart.md)
- [ ] T008 [P] Point the frozen `docs/specs/0005-hold-a-literal-glyph-in-a-cell.md` at
      `specs/028-hold-a-literal-glyph-in-a-cell/` with a "Replaced by" note, as
      `docs/specs/0004-give-a-glyph-a-type-of-its-own.md` already does for feature 006
- [ ] T009 Append the increment's entry to `docs/learning-log.md`

T006 and T007 produce no commit and no test. What they produce is an observation for the pull
request body, and they are there because _Claims are measured, not assumed_ asks a new failure path
to be made to fail on purpose rather than assumed to work, and asks a claim about two branches being
optimizations to be run rather than reasoned. T007 has a second use: if a shortcut turns out to be
load-bearing, the merge is not total and that is a design change to discuss, not a guard to keep
quietly.

T008 is a documentation commit. Feature 006 left the equivalent note to the pull request that landed
its spec; here the spec is already committed on this branch, so it lands with the code instead. The
note names this feature as what supersedes the draft wherever the two differ — which they do, since
planning changed the size of the work in three places.

T009 is what ends the increment, per _Demonstrable increments_. Candidates the increment actually
produced, each with what was tried and what it turned out to be: the type alias that let a rename of
33 call sites stay structural; the discovery that a literal being decided makes three composition
rows fall out of branches that already existed; and the lifetime unification in the renderer, where
two borrows with different owners had to meet.

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies, and it must happen before T003 or the baseline is worthless
- **Foundational (Phase 2)**: after T001, and it blocks both stories
- **US1 (Phase 3)**: after T003
- **US2 (Phase 4)**: after US1, because it spends what US1 built
- **Polish (Phase 5)**: T006 and T007 after T004; T008 any time; T009 last

### Within and between the stories

T001 → T002 → T003 → T004 → T005 → T006 → T007 → T009, with T008 free. Every arrow is a real
dependency:

- T003 must not run before T001, or there is nothing to compare the output against.
- T004 depends on the decision T002 records and the name T003 introduces.
- T005 needs the kind of cell T004 adds.
- T006 and T007 need the merge T004 writes.

### Parallel Opportunities

**One, and it is small.** T008 carries `[P]`: it touches only
`docs/specs/0005-hold-a-literal-glyph-in-a-cell.md` and depends on nothing in the code, so it can
land at any point after the spec. Every other task is strictly ordered — T003, T004, T006 and T007
all edit `crates/monospace-core/src/buffer.rs` or the type its callers name, and the two stories are
sequential because US2 consumes US1. That is why this feature is one branch and one pull request
rather than one per story.

---

## Implementation Strategy

### MVP first (US1 only)

1. T001, to have something to compare against.
2. T002, the decision on the record before any code depends on it.
3. T003, then T004, each with `cargo xtask check` green and each its own commit.
4. **Stop and validate**: diff the output against the baseline, confirm `crates/monospace-cli/` is
   absent from both diffs, and confirm T003 touched no test.
5. This is a complete increment. The library can hold a chosen glyph, with every composition rule
   tested, even if nothing on the terminal has changed yet.

### Incremental delivery

1. Foundational → the decision recorded and the rename done, with no behavior moved.
2. US1 → a cell can hold a chosen glyph, and the rendered output has not moved.
3. US2 → the terminal shows the fill and the occlusion, and the output moves for the first time.
4. Polish → two failure paths observed, the frozen draft pointed at its replacement, the learning
   log appended, and the pull request opened with the diffs and the observations as its evidence.

### One task, one commit

Every task except T001, T006 and T007 is one commit that leaves `cargo xtask check` green. T003 is
the feature's only `refactor` and edits no test; T004 and T005 are `feat`; T002, T008 and T009 are
`docs`. No task mixes a structural change with a behavioral one, so none of them owes _Structural
and behavioral change never share a commit_ an exception — [plan.md](plan.md) records how the alias
made that literal rather than argued.
