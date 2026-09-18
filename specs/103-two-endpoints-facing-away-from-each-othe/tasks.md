---
description: "Task list for feature 103: an arrow's route is ranked rather than bounded"
---

<!-- cspell:ignore othe -->

# Tasks: An arrow's route is ranked rather than bounded

**Input**: Design documents from `/specs/103-two-endpoints-facing-away-from-each-othe/`

**Prerequisites**: plan.md, spec.md, research.md, data-model.md, contracts/route-derivation.md,
quickstart.md

**Tests**: Requested by the spec's Testing expectations — each rule in
`contracts/route-derivation.md` has a test named against it, and the sweep's reviewed snapshot
carries what a named test cannot reach.

**Organization**: This feature is not a set of independent stories either. Plan.md's _The order of
the work_ fixes one linear commit sequence, because every step edits the same file and each depends
on the state the previous one left — issue 104 (User Story 2) is repaired in code User Story 1's
commit deletes next, on purpose (research.md Q3). The phases below follow that agreed sequence
exactly; each task still carries the `[US1]`/`[US2]` label of the story it advances, for
traceability, and the two cross-cutting steps (the window and the learning log) carry none.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies) — none apply here: every step
  changes `arrow.rs` or depends on the step before it, in the order the plan fixes.
- **[Story]**: US1 = "Endpoints facing away are joined, by one rule everywhere" (P1). US2 = "A
  two-cell free span turns toward the endpoint the arrow leaves from" (P2).
- Include exact file paths in descriptions.

## Path Conventions

Single Rust workspace. The only production file that changes is
`crates/monospace-core/src/shape/arrow.rs`; its named tests live in the module's existing
`#[cfg(test)] mod tests`; the sweep's eight pinned pictures live in
`crates/monospace-core/src/snapshots/`. `route.rs` and every other crate are untouched.

---

## Phase 1: Foundational — measure and widen the sweep's window

**Purpose**: settle research.md Q1's contradiction before either behavioral change lands, so that a
rendering showing two heads with a gap can only mean an empty route (FR-011). This blocks both
stories: the spec fixes the window first precisely so neither later diff is a whole-file reshuffle.

**⚠️ CRITICAL**: this MUST land as its own `test` commit, ahead of both behavioral commits, per
research.md Q1 and the spec's own ordering.

- [x] T001 Measure the extreme cell any route reaches over the whole sweep grid with a throwaway
      check against the spike's derivation, to settle which of the two recorded figures is right:
      the arithmetic in research.md Q1 (window already wide enough) or ADR-0046's "twelve clipped".
      Report the finding — do not edit either ADR or the spec from this task; a wrong figure is
      superseded, not corrected in place (constitution principle VI).
- [x] T002 `test(core)`: set `SWEEP_ORIGIN` and `SWEEP_SIZE`
      (`crates/monospace-core/src/shape/arrow.rs:865-869`) to the values T001 measured, and add a
      mechanical assertion in the sweep that every cell any arrangement writes lies inside them
      (FR-011, R-10). Verify per constitution principle IV: shrink the window on purpose and watch
      the new assertion fail, then restore it and watch it pass. Regenerate the snapshot with
      `cargo insta review` — expect the text of all 1856 renderings to move and the picture of none
      to (quickstart.md's table, SC-006 first row).

**Checkpoint**: `cargo xtask check` is green. `cargo run -p monospace-cli` is unchanged. No picture
in the sweep has moved, only the window it sits inside.

---

## Phase 2: User Story 2 — a two-cell free span turns toward the endpoint the arrow leaves from (P2)

**Goal**: where the coordinate the bends leave free spans exactly two cells, the route turns at the
cell nearer the endpoint the arrow leaves from, so exchanging the two endpoints moves the turn
rather than leaving the picture unchanged.

**Independent Test**: render `(0, 0)` leaving `right` and `(3, 1)` leaving `left` from each end and
check each picture against the middle of the free span, computed from the two starting positions
alone. Passes or fails without any of User Story 1.

- [x] T003 [US2] `fix(core)`: replace the `waypoints.len() > 3` proxy at
      `crates/monospace-core/src/shape/arrow.rs:318` with a score of the route's distance from the
      middle, rounding toward the endpoint the arrow leaves from (ADR-0044), so the free-span-of-two
      case stops rounding away from it. In the same commit, add the named test for this scenario to
      `arrow.rs`'s test module — picture written a row per source line — asserting the first order
      turns at `x = 1` and the second at `x = 2` (FR-005, R-5, SC-004). This test MUST pass
      **unchanged** through Phase 3 (research.md Q3): it is written against today's derivation and
      re-run, not rewritten, once the Dijkstra search replaces it. Verify by making both pictures
      fail first (today they are drawn the wrong way round), then pass.

**Checkpoint**: SC-004 holds. `cargo xtask check` is green. `cargo run -p monospace-cli` is
unchanged.

---

## Phase 3: User Story 1 — endpoints facing away are joined, by one rule everywhere (P1) 🎯 MVP

**Goal**: the route an arrow draws is the one _The route of an arrow_ ranks first — fewest bends,
then shortest, then nearest the middle, then the side the travel puts to its right — with nothing
left that bounds where a path goes, and no family of arrangements keeping a construction of its own.

**Independent Test**: describe `(0, 0)` leaving `up` and `(0, 2)` leaving `down`, render it, and
check every cell between the two heads is on one unbroken route reaching each head against its own
leaving direction; render the double-escape arrangement from both ends and count the bends; render a
mirrored arrangement and check which side it passes on.

- [x] T004 [US1] `feat(core)`: in `crates/monospace-core/src/shape/arrow.rs`, introduce `Cost`
      (fields `bends`, `length`, `from_middle`, `hand`, in that order, deriving `Ord` so the
      lexicographic comparison is the derive rather than a written comparator — data-model.md) and
      `Lattice` (per axis: the line each starting position pins, the line beside each of them, the
      middle, and one line outside the route rectangle; deduplicated to at most 7 distinct lines per
      axis). Replace `derive_path`'s body with a Dijkstra over `(node, heading)` states minimized by
      `Cost`, keeping its signature unchanged so `impl Shape for Arrow` does not move. Delete
      `three_waypoint`, `zigzag`, `path_bends`, `direction_between` and `opposite_orientation`. Keep
      `RouteRectangle`, `middle` and `midpoint` verbatim (data-model.md). Regenerate the snapshot
      with `cargo insta review` — expect exactly 290 renderings to move (quickstart.md's table,
      SC-006) — and accept nothing without reading it against _The route of an arrow_ in
      `docs/model.md`.
- [x] T005 [US1] `test(core)` (same commit as T004): rename
      `identical_directions_in_line_gives_an_empty_route` and re-pin its picture to the joined route
      the rule now draws for two endpoints five cells apart on one row both leaving `right` — the
      one test in the workspace whose picture moves outside this feature's own scenarios (SC-009,
      research.md Q6).
- [x] T006 [US1] `test(core)` (same commit as T004): add a named test to `arrow.rs`'s test module,
      picture written a row per source line, for endpoints facing away being joined however far
      apart they are — `(0, 0)` leaving `up` / `(0, 2)` leaving `down` and the same pair moved
      further apart render the same shape with longer runs; also pin the four-bend outside wrap for
      `(0, 0)` leaving `up` / `(1, 2)` leaving `down` (R-1; spec's User Story 1 acceptance scenarios
      1, 3 and 6).
- [x] T007 [US1] `test(core)` (same commit as T004): add a named test for the former double escape —
      `(0, 0)` leaving `left` / `(2, 1)` leaving `right` — asserting the route takes four bends
      rather than six and that the two orders mirror each other (R-4, FR-007, SC-005; spec's User
      Story 1 acceptance scenario 5).
- [x] T008 [US1] `test(core)` (same commit as T004): add a named test for a mirrored tie — `(0, 0)`
      leaving `down` / `(0, 2)` leaving `down` — asserting the route drawn is the one on the side
      the travel from the first to the second endpoint puts to its right (R-6, FR-006, SC-003;
      spec's User Story 1 acceptance scenario 7).

**Checkpoint**: SC-001, SC-003, SC-005 hold. `cargo xtask check` is green.
`cargo run -p monospace-cli` and every picture pinned by feature 039's acceptance scenarios are
unchanged (FR-010).

---

## Phase 4: Pin what the rule answers rather than what it draws

**Purpose**: three assertions that only make sense once the rule from Phase 3 exists and are about a
property of the rule rather than a particular picture.

- [x] T009 [US1] `test(core)`: add a named test pinning that the route is empty in exactly the two
      arrangements the model now names — an endpoint standing on the cell the route would have to
      arrive at (`(0, 0)` / `(0, 1)` both leaving `up`), and two endpoints at one position leaving
      the same direction (FR-003, R-2, SC-001).
- [x] T010 [US1] `test(core)`: add a named test for the sixteen arrangements whose two endpoints sit
      at one position — the sweep grid excludes coincident positions, so this is the only thing that
      covers them — asserting the four that leave in the same direction draw no route and the twelve
      that leave in different directions draw one (FR-012, R-9, SC-010).
- [x] T011 [US1] `test(core)`: add a named test counting the derivation's lattice states rather than
      timing it, asserting the count is equal for endpoints four, fifty and five hundred cells apart
      and never exceeds `7 x 7 x 4 = 196` (FR-008, R-8, SC-007). Keep the count to the lattice
      search alone, not `expand_waypoints` or `Route::draw`, which are linear in route length by
      design (research.md Q2).

**Checkpoint**: SC-007 and SC-010 hold. `cargo xtask check` is green.

---

## Phase 5: Polish

- [x] T012 `docs`: append the increment's entry to `docs/learning-log.md` — what was learned about
      Rust design and idiom (ranking by a derived `Ord` over a lattice search versus a catalogue of
      shapes), what was learned about working this way, and any trade-off worth remembering later.

---

## Dependencies & Execution Order

This feature has one execution order, not several parallel tracks — each phase's file state is what
the next phase edits:

- T001 → T002 → T003 → {T004, T005, T006, T007, T008} → {T009, T010, T011} → T012.
- T001 blocks T002, because T002 sets the constants T001 measures.
- T002 (Foundational) blocks T003 and Phase 3, because both behavioral commits land inside the
  widened window.
- T003 blocks Phase 3, because T004's Dijkstra rewrite replaces the scoring T003 repairs, and T003's
  named test must exist beforehand to be re-run unchanged afterward (research.md Q3).
- T004 blocks T005–T008, because they assert the pictures the rewrite in T004 draws; all five land
  in one `feat` commit per the plan.
- Phase 3 blocks Phase 4, because T009–T011 assert properties of the rule Phase 3 introduces.
- T012 is last: the learning-log entry closes the increment.

If T001 finds the arithmetic rather than ADR-0046's figure is the imprecise one, the spec's SC-002
split (256 visible, 12 clipped) is corrected in its own commit on this branch — constitution's
_Fixing a commit_ — without touching the order above.

### Within Phase 3

T005–T008 touch the same test module as T004 but assert independent scenarios; they are not marked
`[P]` because they land in one `feat(core)` commit per the plan, each ticked off as it is added.

## Implementation Strategy

### MVP first

Phases 1–3 alone settle the whole of issue 103 — SC-001, SC-003 and SC-005 — and are demonstrable on
their own (`cargo run -p monospace-cli` unchanged, `cargo xtask check` green). Stop there if only
the headline gap needs closing; Phase 4 settles the two properties the sweep cannot reach on its
own.

### Incremental delivery

Each phase ends at a checkpoint that is independently green and independently commit-able, per
principle II: a commit exists only where the tree is green, so each numbered task above is either
its own commit (T001 is a measurement, not a commit; T002 and T003 are each their own) or grouped
with its phase's other tasks (Phase 3, Phase 4) into one commit that reaches green together.
