---
description: "Task list for feature 060: ship the mixing tables"
---

# Tasks: Ship the mixing tables

**Input**: Design documents from `/specs/060-ship-the-mixing-tables/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/glyph-tables.md](contracts/glyph-tables.md),
[quickstart.md](quickstart.md)

**Tests**: Included. The constitution's Testing constraint makes unit tests for core logic the
minimum any spec may ask for, and this spec's own Independent Test and Success Criteria per story
name exactly what each story's tests must prove.

**Organization**: Tasks are grouped by user story (spec.md's P1/P2/P3) so each of the four mixing
tables can be added and verified independently. No Foundational phase is needed this time — unlike
feature 056, `build(table, rows)` already exists and already takes rows of any length (plan.md's
Constitution Check, principle V), so no structural change precedes the four stories.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)

## Phase 1: Setup

**Purpose**: Correct the two documents first, stating the end state this feature reaches, mirroring
feature 056's Phase 1 (plan.md's scope note beyond the spec's FR list; research.md's closing
section).

- [ ] T001 [P] Correct §5 _Strokes, glyph sets and the catalog_ in `docs/model.md`: its sentence
      naming the mixing sets as reference-only, loaded from a file, no longer holds once this
      feature lands — amend it to name the four mixing tables alongside the five single-stroke ones
      as data `monospace-glyph-sets` ships (data-model.md's "The documents" section)
- [ ] T002 [P] Correct the opening note of `docs/glyph-sets.md` (currently "the mixing sets remain
      reference only, with nothing loading them yet") to say the four mixing tables are now carried
      as data by `monospace-glyph-sets` too — no table of rows changes, only the sentence describing
      where a mixing set's rules come from

**Checkpoint**: the documents describe the state this feature is about to reach; nothing in them is
false once the phases below land.

---

## Phase 2: User Story 1 - Draw with the Light and Heavy mixing table (Priority: P1) 🎯 MVP

**Goal**: `monospace-glyph-sets` holds the Light and Heavy mixing table (complete: all 50
combinations) and exposes it as a catalog, the same way `ascii()`, `double()`, `heavy()` and
`light_round()` already do.

**Independent Test**: a catalog built from `light()`, `heavy()` and `mixing_light_and_heavy()`
answers all fifty combinations _Mixing Light and Heavy_ records with the character it publishes, and
renders a diagram with no light/heavy overlap byte-identically to a catalog built from `light()` and
`heavy()` alone.

### Tests for User Story 1 ⚠️

- [ ] T003 [P] [US1] In `crates/monospace-glyph-sets/src/lib.rs`'s `tests` module, add
      `mixing_light_and_heavy_answers_every_row_it_publishes`: for every row of the (soon-to-exist)
      `MIXING_LIGHT_AND_HEAVY` table, build the `GlyphKey` its four sides describe and assert
      `GlyphCatalog::union([GlyphCatalog::light(), heavy(), mixing_light_and_heavy()]).glyph(&key)`
      equals the row's published character (SC-002, acceptance scenario 1). Fails to compile until
      T005-T006 land.
- [ ] T004 [P] [US1] In the same `tests` module, add
      `a_catalog_of_light_heavy_and_their_mixing_table_matches_light_and_heavy_alone_without_a_light_heavy_overlap`:
      render a diagram containing a light box and a heavy box that do not touch, once against
      `GlyphCatalog::union([GlyphCatalog::light(), heavy(), mixing_light_and_heavy()])` and once
      against `GlyphCatalog::union([GlyphCatalog::light(), heavy()])`; assert the two outputs are
      byte-identical (acceptance scenario 2). Fails to compile until T005-T006 land.

### Implementation for User Story 1

- [ ] T005 [P] [US1] In `crates/monospace-glyph-sets/src/lib.rs`, add a private
      `const MIXING_LIGHT_AND_HEAVY: &[Row]` table holding exactly the fifty rows
      `docs/glyph-sets.md`'s _Mixing Light and Heavy_ section records, verbatim and in the same
      order, with `"light"` and `"heavy"` as the two stroke names (FR-002)
- [ ] T006 [US1] In `crates/monospace-glyph-sets/src/lib.rs`, implement
      `#[must_use] pub fn mixing_light_and_heavy() -> GlyphCatalog` as
      `build("Mixing Light and Heavy", MIXING_LIGHT_AND_HEAVY)` (FR-005, FR-007) (depends on T005)

**Checkpoint**: `cargo test -p monospace-glyph-sets` is green, including T003-T004's tests. This is
the MVP — the complete mixing table exists and is provably correct and non-intrusive on its own.

---

## Phase 3: User Story 2 - Draw with the Light and Double mixing table (Priority: P2)

**Goal**: `monospace-glyph-sets` holds the Light and Double mixing table (incomplete: 18 of 50
combinations) and exposes it as a catalog, independently of Story 1.

**Independent Test**: a catalog built from `light()`, `double()` and `mixing_light_and_double()`
answers all eighteen covered combinations with the table's published character, and a combination
the table does not cover still degrades to one stroke exactly as it would without the table.

### Tests for User Story 2 ⚠️

- [ ] T007 [P] [US2] In `crates/monospace-glyph-sets/src/lib.rs`'s `tests` module, add
      `mixing_light_and_double_answers_every_row_it_publishes` — same shape as T003, but over the
      (soon-to-exist) `MIXING_LIGHT_AND_DOUBLE` table's eighteen rows and
      `GlyphCatalog::union([GlyphCatalog::light(), double(), mixing_light_and_double()])` (SC-001,
      acceptance scenario 1). Fails to compile until T009-T010 land.
- [ ] T008 [P] [US2] In the same `tests` module, add
      `an_uncovered_light_double_combination_still_degrades`: pick one non-empty combination of
      `light` and `double` sides that is absent from `MIXING_LIGHT_AND_DOUBLE`'s eighteen keys (one
      of the thirty-two `docs/glyph-sets.md` says degrade), and assert
      `GlyphCatalog::union([GlyphCatalog::light(), double(), mixing_light_and_double()]).glyph(&key)`
      equals `GlyphCatalog::union([GlyphCatalog::light(), double()]).glyph(&key)` (SC-005,
      acceptance scenario 2). Fails to compile until T009-T010 land.

### Implementation for User Story 2

- [ ] T009 [P] [US2] In `crates/monospace-glyph-sets/src/lib.rs`, add a private
      `const MIXING_LIGHT_AND_DOUBLE: &[Row]` table holding exactly the eighteen rows
      `docs/glyph-sets.md`'s _Mixing Light and Double_ section records, verbatim and in the same
      order, with `"light"` and `"double"` as the two stroke names (FR-001)
- [ ] T010 [US2] In `crates/monospace-glyph-sets/src/lib.rs`, implement
      `#[must_use] pub fn mixing_light_and_double() -> GlyphCatalog` as
      `build("Mixing Light and Double", MIXING_LIGHT_AND_DOUBLE)` (FR-005, FR-007) (depends on T009)

**Checkpoint**: `cargo test -p monospace-glyph-sets` is green, including T007-T008's tests. Stories
1 and 2 both work independently; Story 2 proves an incomplete table is still worth shipping and that
what it does not cover keeps degrading unchanged.

---

## Phase 4: User Story 3 - Draw with the Light Round and Heavy mixing table (Priority: P3)

**Goal**: `monospace-glyph-sets` holds the Light Round and Heavy mixing table (complete: all 50
combinations, `docs/glyph-sets.md`'s copy of _Mixing Light and Heavy_ with `light` replaced by
`light-round`) and exposes it as a catalog, independently of Stories 1 and 2.

**Independent Test**: a catalog built from `light_round()`, `heavy()` and
`mixing_light_round_and_heavy()` answers all fifty combinations with the table's published
character.

### Tests for User Story 3 ⚠️

- [ ] T011 [P] [US3] In `crates/monospace-glyph-sets/src/lib.rs`'s `tests` module, add
      `mixing_light_round_and_heavy_answers_every_row_it_publishes` — same shape as T003, over the
      (soon-to-exist) `MIXING_LIGHT_ROUND_AND_HEAVY` table's fifty rows and
      `GlyphCatalog::union([light_round(), heavy(), mixing_light_round_and_heavy()])` (SC-004,
      acceptance scenario 1). Fails to compile until T012-T013 land.

### Implementation for User Story 3

- [ ] T012 [P] [US3] In `crates/monospace-glyph-sets/src/lib.rs`, add a private
      `const MIXING_LIGHT_ROUND_AND_HEAVY: &[Row]` table holding exactly the fifty rows
      `docs/glyph-sets.md`'s _Mixing Light Round and Heavy_ section records, verbatim and in the
      same order, with `"light-round"` and `"heavy"` as the two stroke names (FR-004)
- [ ] T013 [US3] In `crates/monospace-glyph-sets/src/lib.rs`, implement
      `#[must_use] pub fn mixing_light_round_and_heavy() -> GlyphCatalog` as
      `build("Mixing Light Round and Heavy", MIXING_LIGHT_ROUND_AND_HEAVY)` (FR-005, FR-007)
      (depends on T012)

**Checkpoint**: `cargo test -p monospace-glyph-sets` is green, including T011's test. Stories 1
through 3 all work independently; Story 3 proves the pattern repeats mechanically under
`light-round`.

---

## Phase 5: User Story 4 - Draw with the Light Round and Double mixing table (Priority: P3)

**Goal**: `monospace-glyph-sets` holds the Light Round and Double mixing table (incomplete: 18 of 50
combinations, `docs/glyph-sets.md`'s copy of _Mixing Light and Double_ with `light` replaced by
`light-round`) and exposes it as a catalog, independently of Stories 1 through 3.

**Independent Test**: a catalog built from `light_round()`, `double()` and
`mixing_light_round_and_double()` answers all eighteen covered combinations with the table's
published character, and an uncovered combination still degrades.

### Tests for User Story 4 ⚠️

- [ ] T014 [P] [US4] In `crates/monospace-glyph-sets/src/lib.rs`'s `tests` module, add
      `mixing_light_round_and_double_answers_every_row_it_publishes` — same shape as T007, over the
      (soon-to-exist) `MIXING_LIGHT_ROUND_AND_DOUBLE` table's eighteen rows and
      `GlyphCatalog::union([light_round(), double(), mixing_light_round_and_double()])` (SC-003,
      acceptance scenario 1). Fails to compile until T016-T017 land.
- [ ] T015 [P] [US4] In the same `tests` module, add
      `an_uncovered_light_round_double_combination_still_degrades` — same shape as T008, over the
      analogous uncovered `light-round`/`double` combination (SC-005, acceptance scenario 2). Fails
      to compile until T016-T017 land.

### Implementation for User Story 4

- [ ] T016 [P] [US4] In `crates/monospace-glyph-sets/src/lib.rs`, add a private
      `const MIXING_LIGHT_ROUND_AND_DOUBLE: &[Row]` table holding exactly the eighteen rows
      `docs/glyph-sets.md`'s _Mixing Light Round and Double_ section records, verbatim and in the
      same order, with `"light-round"` and `"double"` as the two stroke names (FR-003)
- [ ] T017 [US4] In `crates/monospace-glyph-sets/src/lib.rs`, implement
      `#[must_use] pub fn mixing_light_round_and_double() -> GlyphCatalog` as
      `build("Mixing Light Round and Double", MIXING_LIGHT_ROUND_AND_DOUBLE)` (FR-005, FR-007)
      (depends on T016)

**Checkpoint**: `cargo test -p monospace-glyph-sets` is green, including T014-T015's tests. All four
mixing tables now answer independently, two complete and two incomplete, matching the spec's four
stories.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: prove the cross-story requirement the individual stories don't each cover (FR-007),
wire the four new tables into the shipped demonstration (FR-008, FR-009), and close the increment
per constitution principle II.

- [ ] T018 In `crates/monospace-glyph-sets/src/lib.rs`'s `tests` module, add
      `a_catalog_built_from_any_of_the_nine_tables_answers_each_regardless_of_order` (extends the
      existing
      `a_catalog_built_from_two_or_more_single_stroke_tables_answers_each_regardless_of_order`
      pattern to all nine): build `GlyphCatalog::union` over all five single-stroke tables and all
      four mixing tables, and its reverse, and assert one key each mixing table alone defines
      answers identically and correctly in both, unaffected by union order (FR-007, SC-007) (depends
      on T006, T010, T013, T017)
- [ ] T019 Correct `crates/monospace-cli/assets/demo.json`'s two box pairs that pair a stroke
      outside the four mixing tables: the box at `(32,9)` changes its `stroke` from `"light"` to
      `"light-round"` and the box at `(34,10)` from `"light-round"` to `"double"` (now a Light
      Round+Double pair); the box at `(40,9)` changes its `stroke` from `"heavy"` to `"light-round"`
      and the box at `(42,10)` from `"double"` to `"heavy"` (now a Light Round+Heavy pair). No other
      field on those four boxes and no other shape in the file changes (FR-009, FR-010). Before
      committing, check the existing Light+Double crossing (boxes at `(8,9)`/`(10,10)`) against
      `MIXING_LIGHT_AND_DOUBLE`'s eighteen covered keys (research.md's decision, principle IV — run
      it, don't assume it); if its key is not covered, move that pair's `at` positions only (its
      `stroke` fields already name the right pair) until it lands on a covered row
- [ ] T020 In `crates/monospace-cli/src/description.rs`'s `Description::render`, add
      `monospace_glyph_sets::mixing_light_and_double()`, `mixing_light_and_heavy()`,
      `mixing_light_round_and_double()` and `mixing_light_round_and_heavy()` to the existing
      `GlyphCatalog::union([...])` call, alongside the five single-stroke tables already there
      (FR-008) (depends on T006, T010, T013, T017)
- [ ] T021 In `crates/monospace-cli/tests/cli.rs`, run `cargo run -p monospace-cli` against the
      corrected fixture and catalog (T019, T020) and read off the actual character at each of the
      six crossing positions `(2,11)`, `(10,11)`, `(18,11)`, `(26,11)`, `(34,11)`, `(42,11)`
      (principle IV — measured, not assumed); update
      `crossings_between_the_new_tables_degrade_to_whichever_figure_is_in_front`'s six expected
      characters and its doc comment to match, and rename the test (e.g.
      `crossings_between_the_mixed_tables_show_the_mixing_tables_character`) to reflect that these
      six positions now mix rather than degrade (SC-006) (depends on T019, T020)
- [ ] T022 Check every row of `MIXING_LIGHT_AND_DOUBLE`, `MIXING_LIGHT_AND_HEAVY`,
      `MIXING_LIGHT_ROUND_AND_DOUBLE` and `MIXING_LIGHT_ROUND_AND_HEAVY` (T005, T009, T012, T016)
      against `docs/glyph-sets.md`'s matching row, key by key — the property the tests above don't
      check directly, since they prove completeness and non-collision but not that each character is
      the _correct_ one for its key (mirrors feature 056's SC-004 practice, quickstart.md)
- [ ] T023 Run the whole gate on a fresh clone (`cargo xtask check`, per constitution principle IV)
      and the manual checks in `quickstart.md` (`cargo build --workspace`, `cargo test --workspace`,
      `cargo run -p monospace-cli`, `cargo test -p monospace-cli`,
      `cargo check -p monospace-glyph-sets --target wasm32-unknown-unknown`); confirm the
      demonstration's output differs from before this feature at exactly the six corrected crossing
      positions and nowhere else (FR-010, SC-006), and that a diagram touching none of the four
      mixing tables' keys still renders byte-identically (edge case)
- [ ] T024 Append an entry to `docs/learning-log.md` for this increment: what was learned about Rust
      design (four more tables of the same shape a single generic helper already served, and what
      that says about the value of the feature-056 extraction) and about working this way, with
      evidence from what was tried

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies — start immediately. T001 and T002 are independent of each
  other and of every later phase.
- **User Story 1 (Phase 2)**: no dependency on Setup or any other story — `build` already exists
- **User Story 2 (Phase 3)**: independent of Story 1
- **User Story 3 (Phase 4)**: independent of Stories 1 and 2
- **User Story 4 (Phase 5)**: independent of Stories 1 through 3
- **Polish (Phase 6)**: T018 and T020 depend on all four stories (T006, T010, T013, T017); T019 is
  independent of the stories but must land before T021; T021 depends on T019 and T020; T022 depends
  on T005, T009, T012, T016; T023-T024 depend on every phase above being complete

### Within Each User Story

- Tests (the first task pair in each story) are written first and fail to compile until their
  story's implementation tasks land, mirroring feature 056's pattern
- Within each story, the `const` row table precedes the public function that calls `build` over it

### Parallel Opportunities

- T001 and T002 (independent documents)
- Stories 1 through 4 (T003-T006, T007-T010, T011-T013, T014-T017) touch the same file but disjoint
  regions (different consts, different functions, different test names) and can be drafted in
  parallel by different people before being merged into one `lib.rs`
- Within each story, its `[P]`-marked test task(s) and `const` task touch different regions of the
  same file and can be drafted in parallel; the function task depends on both

---

## Parallel Example: User Story 1

```bash
# Draft together, independently of every other story:
Task: "Add mixing_light_and_heavy() tests in crates/monospace-glyph-sets/src/lib.rs"
Task: "Add MIXING_LIGHT_AND_HEAVY const table in crates/monospace-glyph-sets/src/lib.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: User Story 1 — the complete Light and Heavy mixing table
3. **STOP and VALIDATE**: `cargo test -p monospace-glyph-sets` green, per the Independent Test above
4. This alone gives a light/heavy overlap its mixed junction instead of a degraded one — the reason
   the issue was raised, on the table that needs no incompleteness caveat

### Incremental Delivery

1. Setup → the documents describe the end state
2. User Story 1 → the Light and Heavy mixing table ships (MVP)
3. User Story 2 → the Light and Double mixing table ships, independently, proving an incomplete
   table is still worth shipping
4. User Story 3 → the Light Round and Heavy mixing table ships, independently, proving the pattern
   repeats under `light-round`
5. User Story 4 → the Light Round and Double mixing table ships, independently, completing the four
6. Polish → the no-collision property across all nine tables is proven (FR-007), the shipped
   demonstration is corrected and rewired (FR-008, FR-009), every row is checked against the
   document, the gate is green on a fresh clone, and the learning-log entry closes the increment

### Parallel Team Strategy

With multiple developers:

1. One person completes Setup (T001-T002)
2. In parallel:
   - Developer A: User Story 1 (Light and Heavy)
   - Developer B: User Story 2 (Light and Double)
   - Developer C: User Story 3 (Light Round and Heavy)
   - Developer D: User Story 4 (Light Round and Double)
3. Stories complete independently; the four land as separate `feat` commits into the same file, in
   any order, since none of the four mixing tables contests another's keys (FR-007) — Polish follows
   once all four public functions exist
