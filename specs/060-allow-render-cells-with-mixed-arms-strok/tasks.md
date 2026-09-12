---
description: "Task list for feature 060: allow render cells with mixed arms' strokes"
---

<!-- The branch and directory names are truncated by the tooling that creates them; "strok" is
     that truncation, not a word. cspell:ignore strok -->

# Tasks: Allow render cells with mixed arms' strokes

**Input**: Design documents from `/specs/060-allow-render-cells-with-mixed-arms-strok/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md),
[contracts/mixed-arm-rendering.md](contracts/mixed-arm-rendering.md), [quickstart.md](quickstart.md)

**Tests**: Included. The constitution's Testing constraint makes unit tests for core logic the
minimum any spec may ask for, and this spec's own Independent Test and Success Criteria per story
name exactly what each story's tests must prove. FR-014/SC-003 (the shipped demonstration's output)
stay accepted on observation, per the spec — no test pins them.

**Organization**: Tasks are grouped by user story (spec.md's P1/P1/P2/P2), but this feature's real
dependency order is not its priority order: User Story 3 (the four mixing tables) has to exist
before User Story 2 (the CLI wiring that loads them) can show anything, even though Story 2 is the
higher-priority story. The Dependencies section below states this explicitly; the phase order in
this file follows the buildable order, matching plan.md's three `feat` commits.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)

## Phase 1: Setup

No setup tasks. research.md already settles it: no new dependency, no new crate, no new ADR — the
toolchain is unchanged and `monospace-glyph-sets` already exists.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: give each arm its own stroke and add the missing second lookup ADR-0009 described but
never implemented — [ADR-0037](../../docs/decisions/0037-give-each-arm-its-own-stroke.md), FR-001
through FR-005. This is plan.md's first commit.

**⚠️ CRITICAL**: No user story task can begin until this phase is complete — no mixed `GlyphKey` can
even be built, and no test below compiles, until `Arm::Set` carries a `Stroke`.

- [x] T001 In `crates/monospace-core/src/cell.rs`, change `Arm::Set` to `Arm::Set(Stroke)`; drop
      `Copy` from `Arm`'s derive, keeping `Debug, Clone, PartialEq, Eq` (FR-001, data-model.md)
- [x] T002 In `crates/monospace-core/src/cell.rs`, update `StrokeCell::key()` to read each arm's own
      stroke (`Arm::Set(stroke) => Some(stroke.clone())`) instead of `self.base` (FR-002) (depends
      on T001)
- [x] T003 In `crates/monospace-core/src/cell.rs`, add `StrokeCell::degraded_key()`: the same shape
      as `key()`, but every `Set` side answers `Some(self.base.clone())` regardless of its own
      stroke; `Closed` and `Unset` stay `None` on both methods (FR-004) (depends on T001)
- [x] T004 In `crates/monospace-core/src/cell.rs`, update `Cell::glyph_str` to try
      `glyphs.glyph(&cell.key())` first and fall back to `glyphs.glyph(&cell.degraded_key())` on a
      miss, mirroring the two lookups in `docs/model.md`'s _Rendering_ (FR-002, FR-004) (depends on
      T002, T003)
- [x] T005 Update every `Arm::Set` construction site in `crates/monospace-core/src/cell.rs`'s own
      `tests` module to `Arm::Set(stroke)`, supplying the same stroke the cell's `base` already uses
      at each site — mechanical, no site changes which sides are `Set`/`Closed`/`Unset` (depends on
      T001). `crates/monospace-core/src/render.rs`'s own `tests` module carries the same `Arm::Set`
      construction sites tasks.md did not enumerate separately (it renders through `cell.rs`);
      updated alongside this task for the same mechanical reason.
- [x] T006 [P] Update every `Arm::Set` construction site in `crates/monospace-core/src/buffer.rs`
      (including its stamping tests) to `Arm::Set(stroke)` (depends on T001). `merge_strokes` also
      needed `bottom.<arm>.clone()` in place of a move, since `Arm` is no longer `Copy`; test
      assertions that compare a cell produced by an actual stamp/merge (not a bare input literal)
      were given the stroke each surviving arm actually carries after the merge, traced by hand
      against `merge_arm`/`merge_strokes`, rather than always the cell's own `base`.
- [x] T007 [P] Update every `Arm::Set` construction site in
      `crates/monospace-core/src/shape/line.rs` to `Arm::Set(stroke)` (depends on T001)
- [x] T008 [P] Update every `Arm::Set` construction site in
      `crates/monospace-core/src/shape/fragment/segment.rs` to `Arm::Set(stroke)` (depends on T001)
- [x] T009 [P] Update every `Arm::Set` construction site in
      `crates/monospace-core/src/shape/fragment/corner.rs` to `Arm::Set(stroke)` (depends on T001)
- [x] T010 [P] Update every `Arm::Set` construction site in
      `crates/monospace-core/src/shape/fragment/border.rs` to `Arm::Set(stroke)` (depends on T001)
- [x] T011 [P] Update every `Arm::Set` construction site in
      `crates/monospace-core/src/shape/fragment/end.rs` to `Arm::Set(stroke)` (depends on T001)

**Checkpoint**: `cargo test -p monospace-core` is green; every existing test's expected output is
unchanged (mechanical sites only carried the value already implied). This is plan.md's first `feat`
commit boundary.

---

## Phase 3: User Story 1 - Draw the junction two strokes make (Priority: P1) 🎯 MVP

**Goal**: a cell whose arms carry different strokes resolves through `key()` first and
`degraded_key()` second, exactly as ADR-0009 and `docs/model.md` describe; a uniform cell is
unaffected.

**Independent Test**: a `StrokeCell` with `light` on top/bottom and `heavy` on left/right, looked up
against a small ad hoc catalog holding a rule for that exact mixed key, resolves to that rule's
glyph rather than either stroke's own cross.

### Tests for User Story 1 ⚠️

- [x] T012 [P] [US1] In `crates/monospace-core/src/cell.rs`'s `tests` module, add
      `key_reads_each_arms_own_stroke`: a cell with `light` top/bottom and `heavy` left/right arms
      produces a `GlyphKey` naming `light`/`heavy` on their own sides, not the cell's `base`
      (Acceptance Scenarios 1-2, FR-002)
- [x] T013 [P] [US1] Add `degraded_key_collapses_every_set_arm_to_the_base_stroke`: the same mixed
      cell's `degraded_key()` names the cell's `base` stroke on every `Set` side regardless of what
      that arm itself carries (FR-004)
- [x] T014 [P] [US1] Add `glyph_str_tries_the_exact_key_then_the_degraded_key_then_none`: build an
      ad hoc `GlyphCatalog::from_rules` with one rule for the mixed key and confirm `glyph_str`
      finds it; rebuild with only a rule for the uniform base-stroke key and confirm the fallback
      finds that instead; rebuild with neither rule and confirm `None` (Acceptance Scenario 1,
      FR-002, FR-004)
- [x] T015 [P] [US1] Add
      `a_cell_whose_arms_all_carry_the_base_stroke_resolves_the_same_key_from_both_methods`: `key()`
      and `degraded_key()` produce equal `GlyphKey`s when every `Set` arm already carries the cell's
      own `base` stroke (Acceptance Scenario 4, SC-006's invariant)
- [x] T016 [US1] In `crates/monospace-core/src/buffer.rs`'s `tests` module, add a stamping test: two
      overlapping shapes of different strokes, stamped front-to-back and again back-to-front,
      produce the same buffer even though the cells involved end up with arms of two different
      strokes (Acceptance Scenario 5, FR-003, FR-005). Satisfied by the existing
      `front_to_back_with_below_equals_back_to_front_with_above` test: it already stamps three
      shapes of three different strokes (`double`/`light`/`heavy`) in both orders, and now that
      `Arm::Set` carries a `Stroke`, its assertion already proves the merged cell's arms carry two
      different strokes and both orders agree — no separate test needed.

**Checkpoint**: `cargo test -p monospace-core` is green, including T012-T016. The two-lookup
mechanism is proven; nothing yet has a real mixing table to draw from (that is User Story 3).

---

## Phase 4: User Story 3 - Build a catalog from a mixing table (Priority: P2)

**Goal**: `monospace-glyph-sets` ships the four mixing tables `docs/glyph-sets.md` already records,
each reachable the same zero-argument way `double()`/`heavy()`/`light_round()` already are, and no
key collides across any of the nine tables the crate now ships. This is plan.md's second commit —
and the prerequisite for User Story 2's CLI wiring below, even though it is a lower-priority story.

**Independent Test**: build a catalog from `light_round()`, `heavy()` and `light_round_heavy()`
alone; it holds exactly the fifty rows `docs/glyph-sets.md` records under _Mixing Light Round and
Heavy_, and its spot-checked rows render the character that section publishes.

### Tests for User Story 3 ⚠️

- [x] T017 [P] [US3] In `crates/monospace-glyph-sets/src/lib.rs`'s `tests` module, add
      `light_double_has_exactly_its_documented_rows_and_spot_checks_match` (18 rows; spot-check at
      least a four-armed crossing, a three-armed junction, a corner, one row with its two strokes
      exchanged between sides, and — per SC-002 — the two crossing characters the demonstration
      draws, `╪` and `╫`) (SC-001, SC-002). Fails to compile until T022-T023 land.
- [x] T018 [P] [US3] Add `light_heavy_has_exactly_its_documented_rows_and_spot_checks_match` (50
      rows; same spot-check shape, including `┿` and `╂`) (SC-001, SC-002). Fails to compile until
      T024-T025 land.
- [x] T019 [P] [US3] Add `light_round_double_has_exactly_its_documented_rows_and_spot_checks_match`
      (18 rows; same spot-check shape) (SC-001, SC-002). Fails to compile until T026-T027 land.
- [x] T020 [P] [US3] Add `light_round_heavy_has_exactly_its_documented_rows_and_spot_checks_match`
      (50 rows; same spot-check shape) (SC-001, SC-002). Fails to compile until T028-T029 land.
- [x] T021 [US3] Extend the existing union-order-independence test (currently over `ascii()`,
      `double()`, `heavy()`, `light_round()`) to build `GlyphCatalog::union` from all nine tables —
      the five single-stroke ones plus the four mixing ones from this feature — in at least two
      different orders, asserting a key from each of the nine answers correctly in both (FR-011,
      SC-007). Fails to compile until T022-T029 land. Renamed to
      `a_catalog_built_from_all_nine_tables_answers_each_regardless_of_order` since it no longer
      names only the single-stroke tables it started from.

### Implementation for User Story 3

- [x] T022 [P] [US3] In `crates/monospace-glyph-sets/src/lib.rs`, add a private
      `const LIGHT_DOUBLE: &[Row]` holding exactly the eighteen rows `docs/glyph-sets.md`'s _Mixing
      Light and Double_ section records, verbatim and in the same order (FR-006)
- [x] T023 [US3] Add `#[must_use] pub fn light_double() -> GlyphCatalog` as
      `build("Mixing Light and Double", LIGHT_DOUBLE)` (FR-010) (depends on T022)
- [x] T024 [P] [US3] Add a private `const LIGHT_HEAVY: &[Row]` holding exactly the fifty rows under
      _Mixing Light and Heavy_, verbatim and in the same order (FR-007)
- [x] T025 [US3] Add `#[must_use] pub fn light_heavy() -> GlyphCatalog` as
      `build("Mixing Light and Heavy", LIGHT_HEAVY)` (FR-010) (depends on T024)
- [x] T026 [P] [US3] Add a private `const LIGHT_ROUND_DOUBLE: &[Row]` holding exactly the eighteen
      rows under _Mixing Light Round and Double_, verbatim and in the same order (FR-008)
- [x] T027 [US3] Add `#[must_use] pub fn light_round_double() -> GlyphCatalog` as
      `build("Mixing Light Round and Double", LIGHT_ROUND_DOUBLE)` (FR-010) (depends on T026)
- [x] T028 [P] [US3] Add a private `const LIGHT_ROUND_HEAVY: &[Row]` holding exactly the fifty rows
      under _Mixing Light Round and Heavy_, verbatim and in the same order (FR-009)
- [x] T029 [US3] Add `#[must_use] pub fn light_round_heavy() -> GlyphCatalog` as
      `build("Mixing Light Round and Heavy", LIGHT_ROUND_HEAVY)` (FR-010) (depends on T028)
- [x] T030 [US3] Manually check every row of `LIGHT_DOUBLE`, `LIGHT_HEAVY`, `LIGHT_ROUND_DOUBLE` and
      `LIGHT_ROUND_HEAVY` (T022, T024, T026, T028) against `docs/glyph-sets.md`'s matching row, key
      by key — the property the row-count and spot-check tests don't verify directly, since they
      prove completeness and a sample of characters but not that every character is the transcribed
      one (quickstart.md's "Row-for-row against the document"). Done with a throwaway script parsing
      both the Markdown tables and the Rust `const` arrays and diffing them row by row, rather than
      eyeballing four hundred-plus cells by hand — a stronger check of the same property, not a
      different one; every row matched.
- [x] T031 [P] [US3] Correct `docs/model.md`'s _Strokes, glyph sets and the catalog_ sentence "only
      the mixing sets are left loaded from a file when someone asks for them" to name the four
      mixing tables alongside ASCII, Double, Heavy and Light Round as data `monospace-glyph-sets`
      ships (FR-012, research.md)
- [x] T032 [P] [US3] Correct `docs/glyph-sets.md`'s opening note ("the mixing sets remain reference
      only, with nothing loading them yet") the same way (FR-012)

**Checkpoint**: `cargo test -p monospace-glyph-sets` is green, including T017-T021. All nine tables
this project ships now answer independently and in combination, with no key collision. This is
plan.md's second `feat` commit boundary.

---

## Phase 5: User Story 4 - Keep the junctions no table covers exactly as they are (Priority: P2)

**Goal**: prove this feature only narrows degradation, never changes it — an uncovered mixture, or a
mixture no table pairs at all, still degrades to the cell's base stroke exactly as before.

**Independent Test**: render a light/double arm combination outside the eighteen `LIGHT_DOUBLE`
records, once with `light_double()` in the catalog and once without it; both renders produce the
same character.

- [x] T033 [P] [US4] In `crates/monospace-core/src/cell.rs`'s `tests` module, add
      `an_uncovered_mixture_degrades_to_the_base_stroke_regardless_of_a_partial_mixing_rule_set`:
      build an ad hoc catalog covering only some combinations of two strokes plus the base-stroke
      fallback rule; a combination it does not cover renders the same with and without that partial
      rule set present (Acceptance Scenario 1 mechanism, FR-004)
- [x] T034 [P] [US4] Add `light_and_light_round_degrade_with_no_mixing_table_pairing_them`: a cell
      mixing `light` and `light-round` arms, rendered against a catalog holding both single-stroke
      tables but no table pairing them, degrades to the cell's base stroke (Acceptance Scenario 2)
- [x] T035 [P] [US4] Add `heavy_and_double_degrade_with_no_mixing_table_pairing_them`: mirrors T034
      for `heavy` and `double` (Acceptance Scenario 3)
- [x] T036 [US4] In `crates/monospace-glyph-sets/src/lib.rs`'s `tests` module, add
      `an_uncovered_light_double_combination_degrades_the_same_with_or_without_the_mixing_table`:
      pick one of the light/double per-arm combinations `LIGHT_DOUBLE`'s eighteen rows do not
      record; render it against a catalog including `light_double()` and against one without it;
      both produce the cell's base-stroke character (SC-005) (depends on T023)

**Checkpoint**: `cargo test -p monospace-core` and `cargo test -p monospace-glyph-sets` both stay
green. Nothing this feature ships changes what an uncovered mixture renders.

---

## Phase 6: User Story 2 - See the difference in the shipped demonstration (Priority: P1)

**Goal**: the command-line demonstration's catalog is built from all nine tables, so its
light/double and light/heavy crossings draw the mixed characters instead of a uniform cross — with
no change to the shipped demo file. This is plan.md's third commit, and depends on User Story 3's
four functions already existing.

**Independent Test**: run `cargo run -p monospace-cli` with no arguments before and after this story
lands; the two outputs differ only at the eight crossing positions named in spec.md's User Story 2.

- [x] T037 [US2] In `crates/monospace-cli/src/description.rs`, add
      `monospace_glyph_sets::light_double()`, `light_heavy()`, `light_round_double()` and
      `light_round_heavy()` to `Description::render()`'s `GlyphCatalog::union([...])` list,
      alongside the five tables already there (FR-013) (depends on T023, T025, T027, T029)
- [x] T038 [US2] Run `cargo run -p monospace-cli` with no arguments; compare the last figure group
      of its output to the "prints instead" block in spec.md's User Story 2, and confirm every other
      character is byte-identical to before this feature — accepted on observation, not pinned by a
      test (FR-014, SC-003). Observed by running the binary both before and after this task's change
      (via `git stash`) and diffing the two outputs byte for byte: exactly 8 character positions
      differ, all within the last figure group, and the new text matches spec.md's "prints instead"
      block exactly.
- [x] T039 [US2] Run `git diff --stat crates/monospace-cli/assets/demo.json` and confirm it prints
      nothing — the shipped demo file is unchanged (FR-015, SC-004)
- [x] T040 [US2] Append an entry to `docs/learning-log.md` for this increment: what was learned
      about Rust design (the missing second lookup, per-arm strokes) and about working this way,
      with the observation from T038 as evidence, per constitution principle II

**Checkpoint**: `cargo run -p monospace-cli` shows the mixed junctions; the demo file is untouched;
the observation is recorded. This is plan.md's third and final `feat` commit boundary.

---

## Phase 7: Polish & Cross-Cutting Concerns

**Purpose**: the whole-workspace and portability checks that no single story owns.

- [x] T041 Run `cargo xtask check` on a fresh clone (constitution principle IV) and
      `cargo test --workspace`, confirming every phase above is still green together, not only in
      isolation. Both green: a `git clone` of this branch, followed by `cargo xtask setup` and
      `cargo xtask check`, passes all 10 checks; `cargo test --workspace` passes 127 tests across
      `monospace-core`, `monospace-glyph-sets`, `monospace-cli` and `xtask`.
- [x] T042 [P] Run `cargo check -p monospace-core --target wasm32-unknown-unknown`, confirming the
      owned `Stroke` per arm compiles for the WebAssembly target as it did before (principle VII)
- [x] T043 [P] Run `cargo check -p monospace-glyph-sets --target wasm32-unknown-unknown`, confirming
      the four new tables compile for the same target

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: none — no tasks.
- **Foundational (Phase 2)**: no dependency on Setup; BLOCKS every user story — nothing below
  compiles until `Arm::Set` carries a `Stroke` (T001-T004) and every construction site follows
  (T005-T011).
- **User Story 1 (Phase 3)**: depends only on Foundational.
- **User Story 3 (Phase 4)**: depends only on Foundational — independent of Story 1's tests, but
  read on: Story 2 below depends on it.
- **User Story 4 (Phase 5)**: depends on Foundational (T033-T035) and, for T036 specifically, on
  User Story 3's `light_double()` (T023).
- **User Story 2 (Phase 6)**: depends on Foundational **and** on User Story 3's four functions
  (T023, T025, T027, T029) — despite being priority P1 while Story 3 is P2, Story 2's CLI wiring has
  nothing to load until Story 3 ships the tables. This is the one place in this feature where
  priority order and build order diverge; plan.md's three commits already reflect it.
- **Polish (Phase 7)**: depends on every phase above.

### Within Each User Story

- Story 1: tests (T012-T016) exercise logic Foundational already implements — no separate
  implementation subsection.
- Story 3: tests (T017-T021) are written to fail to compile until their matching `const`/function
  pair lands, mirroring feature 056's pattern; within each table, the `const` precedes the function
  that calls `build` over it.
- Story 4: tests only, no implementation — this story proves the absence of a behavior change.
- Story 2: T037 (wiring) precedes T038-T039 (observation), which precede T040 (the learning-log
  entry closing the increment).

### Parallel Opportunities

- T006-T011 (six different files) can be updated in parallel once T001 lands.
- T012-T015 (different test functions in the same file) can be drafted in parallel; T016 (a
  different file) is independent of all four.
- T017-T020 (one test function per table) and T022, T024, T026, T028 (one `const` per table) can be
  drafted in parallel; each table's function task depends only on that table's own `const`.
- T033-T035 (independent test functions) can be drafted in parallel; T036 depends on T023.
- T031-T032 (independent documents) can run in parallel with each other and with Story 3's tests.
- T042-T043 (independent crates) can run in parallel.

---

## Parallel Example: User Story 3

```bash
# Once Phase 2 (Foundational) has landed, draft together:
Task: "Add LIGHT_DOUBLE const table in crates/monospace-glyph-sets/src/lib.rs"
Task: "Add LIGHT_HEAVY const table in crates/monospace-glyph-sets/src/lib.rs"
Task: "Add LIGHT_ROUND_DOUBLE const table in crates/monospace-glyph-sets/src/lib.rs"
Task: "Add LIGHT_ROUND_HEAVY const table in crates/monospace-glyph-sets/src/lib.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 2: Foundational (CRITICAL — blocks all stories).
2. Complete Phase 3: User Story 1.
3. **STOP and VALIDATE**: `cargo test -p monospace-core` green, per the Independent Test above. The
   two-lookup mechanism is proven, even with no mixing table shipped yet.

### Incremental Delivery

1. Foundational → the per-arm stroke and the missing second lookup exist; every existing test still
   passes unchanged.
2. User Story 1 → the mechanism is proven with ad hoc catalogs.
3. User Story 3 → the four real mixing tables ship, independently reachable, colliding with nothing.
4. User Story 4 → uncovered mixtures are proven to degrade exactly as before.
5. User Story 2 → the CLI's shipped demonstration is wired to the four tables; the visible
   difference the issue asked for appears; the observation is recorded.
6. Polish → the whole gate, on a fresh clone, and both crates' WebAssembly target.

### Parallel Team Strategy

With multiple developers:

1. One person completes Foundational (T001-T011) — everything else depends on it.
2. Once Foundational is done:
   - Developer A: User Story 1 (mechanism tests)
   - Developer B: User Story 3 (the four tables)
3. Once User Story 3's four functions exist:
   - Developer A or B: User Story 4 (degradation tests)
   - Developer A or B: User Story 2 (CLI wiring and the observation)
4. Polish last, once every story above is green together.
