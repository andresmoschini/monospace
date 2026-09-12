---
description: "Task list for feature 056: ship the Double, Heavy and Light Round tables"
---

# Tasks: Ship the Double, Heavy and Light Round tables

**Input**: Design documents from `/specs/056-ship-the-double-heavy-and-light-round-ta/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md), [contracts/glyph-tables.md](contracts/glyph-tables.md),
[quickstart.md](quickstart.md)

**Tests**: Included. The constitution's Testing constraint makes unit tests for core logic the
minimum any spec may ask for, and this spec's own Independent Test and Success Criteria per story
name exactly what each story's tests must prove.

**Organization**: Tasks are grouped by user story (spec.md's P1/P2/P3) so each of the three tables
can be added and verified independently, per the plan's Constitution Check (principle V: the shared
helper extraction lands as its own `refactor` commit, strictly before the `feat` commit(s) that add
each table).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)

## Phase 1: Setup

**Purpose**: Correct the two documents first (the constitution requires the model to match before
code relies on it — plan.md Summary), stating the end state this feature reaches.

- [x] T001 [P] Correct §5 _Strokes, glyph sets and the catalog_ in `docs/model.md`: its sentence
      "`monospace-core` ships Light, `monospace-glyph-sets` ships ASCII; the rest are loaded from a
      file when someone asks for them" no longer holds once this feature lands — amend it to name
      Double, Heavy and Light Round alongside Light and ASCII as shipping built in as data, with
      only the mixing sets left loaded from a file (FR-011)
- [x] T002 [P] Correct the opening note of `docs/glyph-sets.md` (currently "The ASCII table below is
      carried as data by `monospace-glyph-sets`, and Light by `monospace-core`; Double, Heavy, Light
      Round and the mixing sets remain reference only") to say Double, Heavy and Light Round are now
      carried as data by `monospace-glyph-sets` too, leaving only the four mixing sets
      reference-only (FR-012)

**Checkpoint**: the documents describe the state this feature is about to reach; nothing in them is
false once the phases below land.

---

## Phase 2: Foundational (Blocking Prerequisites)

**Purpose**: Pull the row-to-catalog conversion `ascii()` already does out into one private helper,
so the three tables below add data and a one-line function each rather than pasting the same
conversion three more times (research.md's decision).

**⚠️ CRITICAL**: No user story task can begin until this phase is complete — every story's
`double()`/`heavy()`/`light_round()` calls the helper this phase creates.

- [x] T003 In `crates/monospace-glyph-sets/src/lib.rs`, extract the closure body of the existing
      `ascii()` into a private `fn build(table: &str, rows: &[Row]) -> GlyphCatalog` that maps each
      row to a `(GlyphKey, Glyph)` pair via `Stroke::from` on each non-empty side and `Glyph::new`
      on the character, panicking `"{table} table row {key:?} is not a valid glyph"` if a row's
      character is invalid; reimplement `ascii()` as `build("ASCII", ASCII)`. This is a
      **structural** change: `ascii()`'s signature, doc comment, `#[must_use]` and behavior are
      unchanged, and its existing two tests
      (`ascii_answers_every_non_empty_combination_of_its_own_stroke`,
      `a_box_rendered_with_ascii_alone_uses_only_ascii_box_characters`) pass unmodified — no test is
      added or changed by this task, per constitution principle V

**Checkpoint**: `cargo test -p monospace-glyph-sets` is green with the same two tests as before this
task, unmodified. This is the commit boundary: land T003 as its own `refactor` commit before any
task below.

---

## Phase 3: User Story 1 - Draw with the Double table (Priority: P1) 🎯 MVP

**Goal**: `monospace-glyph-sets` holds the Double table and exposes it as a catalog, the same way
`ascii()` already does.

**Independent Test**: a catalog built from `monospace_glyph_sets::double()` alone answers all
fifteen non-empty combinations of the `double` stroke, and a box rendered against it uses only the
Double table's own characters and space.

### Tests for User Story 1 ⚠️

- [x] T004 [P] [US1] In `crates/monospace-glyph-sets/src/lib.rs`'s `tests` module, add
      `double_answers_every_non_empty_combination_of_its_own_stroke` (mirrors the existing ASCII
      test, built over `monospace_core::Stroke::from("double")` and `super::double()`) and
      `a_box_rendered_with_double_alone_uses_only_double_box_characters` (mirrors the existing ASCII
      box test, asserting every character of the rendered output is one of `║╗╦╬╣╔╠═╩╝╚`, space or
      newline) (SC-001, SC-005). Both fail to compile until T005-T006 land.

### Implementation for User Story 1

- [x] T005 [P] [US1] In `crates/monospace-glyph-sets/src/lib.rs`, add a private
      `const DOUBLE: &[Row]` table holding exactly the fifteen rows `docs/glyph-sets.md`'s _Double_
      table records, verbatim and in the same order, with `"double"` as the stroke name on every
      non-empty side (FR-001)
- [x] T006 [US1] In `crates/monospace-glyph-sets/src/lib.rs`, implement
      `#[must_use] pub fn double() -> GlyphCatalog` as `build("Double", DOUBLE)` (FR-004, FR-007)
      (depends on T003, T005)

**Checkpoint**: `cargo test -p monospace-glyph-sets` is green, including the two new tests from
T004; no existing test changed. This is the MVP — a second stroke to choose besides Light and ASCII
exists and answers completely.

---

## Phase 4: User Story 2 - Draw with the Heavy table (Priority: P2)

**Goal**: `monospace-glyph-sets` holds the Heavy table and exposes it as a catalog, independently of
Story 1.

**Independent Test**: a catalog built from `monospace_glyph_sets::heavy()` alone answers all fifteen
non-empty combinations of the `heavy` stroke, and a box rendered against it uses only the Heavy
table's own characters and space.

### Tests for User Story 2 ⚠️

- [x] T007 [P] [US2] In `crates/monospace-glyph-sets/src/lib.rs`'s `tests` module, add
      `heavy_answers_every_non_empty_combination_of_its_own_stroke` and
      `a_box_rendered_with_heavy_alone_uses_only_heavy_box_characters` (asserting every character is
      one of `┃┓┳╋┫┏┣━┻┛┗`, space or newline), the same shape as T004 (SC-002, SC-005). Both fail to
      compile until T008-T009 land.

### Implementation for User Story 2

- [x] T008 [P] [US2] In `crates/monospace-glyph-sets/src/lib.rs`, add a private
      `const HEAVY: &[Row]` table holding exactly the fifteen rows `docs/glyph-sets.md`'s _Heavy_
      table records, verbatim and in the same order, with `"heavy"` as the stroke name (FR-002)
- [x] T009 [US2] In `crates/monospace-glyph-sets/src/lib.rs`, implement
      `#[must_use] pub fn heavy() -> GlyphCatalog` as `build("Heavy", HEAVY)` (FR-005, FR-007)
      (depends on T003, T008)

**Checkpoint**: `cargo test -p monospace-glyph-sets` is green, including T007's tests; Stories 1 and
2 both work independently of each other.

---

## Phase 5: User Story 3 - Draw with the Light Round table (Priority: P3)

**Goal**: `monospace-glyph-sets` holds the Light Round table and exposes it as a catalog,
independently of Stories 1 and 2, including its four corners that differ from Light's.

**Independent Test**: a catalog built from `monospace_glyph_sets::light_round()` alone answers all
fifteen non-empty combinations of the `light-round` stroke, its four corner rows are `╮╭╯╰` (not
Light's `┐┌┘└`), and a box rendered against it uses only the Light Round table's own characters and
space.

### Tests for User Story 3 ⚠️

- [x] T010 [P] [US3] In `crates/monospace-glyph-sets/src/lib.rs`'s `tests` module, add
      `light_round_answers_every_non_empty_combination_of_its_own_stroke`,
      `a_box_rendered_with_light_round_alone_uses_only_light_round_box_characters` (asserting every
      character is one of `│╮┬┼┤╭├─┴╯╰`, space or newline), and
      `light_rounds_corners_differ_from_lights` (a box rendered against `light_round()` alone
      produces `╮╭╯╰` at its four corners, distinct from the `┐┌┘└` a box rendered against
      `monospace_core::GlyphCatalog::light()` alone produces at the same corners) (SC-003, SC-005).
      All three fail to compile until T011-T012 land.

### Implementation for User Story 3

- [x] T011 [P] [US3] In `crates/monospace-glyph-sets/src/lib.rs`, add a private
      `const LIGHT_ROUND: &[Row]` table holding exactly the fifteen rows `docs/glyph-sets.md`'s
      _Light Round_ table records, verbatim and in the same order, with `"light-round"` as the
      stroke name — including the four corner rows whose character differs from `Light`'s (FR-003)
- [x] T012 [US3] In `crates/monospace-glyph-sets/src/lib.rs`, implement
      `#[must_use] pub fn light_round() -> GlyphCatalog` as `build("Light Round", LIGHT_ROUND)`
      (FR-006, FR-007) (depends on T003, T011)

**Checkpoint**: `cargo test -p monospace-glyph-sets` is green, including T010's tests. All three
tables now answer independently and completely; this proves a table sharing most of its characters
with another (Light) still answers on its own.

---

## Phase 6: Polish & Cross-Cutting Concerns

**Purpose**: prove the cross-story requirement the individual stories don't each cover, then close
the increment per constitution principle II.

- [x] T013 In `crates/monospace-glyph-sets/src/lib.rs`'s `tests` module, add
      `a_catalog_built_from_two_or_more_single_stroke_tables_answers_each_regardless_of_order`:
      build `GlyphCatalog::union([ascii(), double(), heavy(), light_round()])` and its reverse,
      assert a key from each of the four tables answers correctly in both catalogs (FR-008, SC-006)
      (depends on T006, T009, T012)
- [x] T014 Check every row of `DOUBLE`, `HEAVY` and `LIGHT_ROUND` (T005, T008, T011) against
      `docs/glyph-sets.md`'s matching row, key by key — the property the tests above don't check
      directly, since they prove completeness and the character set but not that each character is
      the _correct_ one for its key (SC-004)
- [ ] T015 Run the whole gate on a fresh clone (`cargo xtask check`, per constitution principle IV)
      and the manual checks in `quickstart.md` (`cargo build --workspace`, `cargo test --workspace`,
      `cargo run -p monospace-cli`, `cargo test -p monospace-cli`,
      `cargo check -p monospace-glyph-sets --target wasm32-unknown-unknown`); confirm the CLI's
      shipped demonstration is byte-identical to before this feature (FR-010, SC-007)
- [ ] T016 Append an entry to `docs/learning-log.md` for this increment: what was learned about Rust
      design (extracting `build` before adding data, so a structural change and three behavioral
      ones never share a commit) and about working this way, with evidence from what was tried

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies — start immediately. T001 and T002 are independent of each
  other and of every later phase.
- **Foundational (Phase 2)**: no dependency on Setup; BLOCKS every user story below — T003 is what
  `double()`, `heavy()` and `light_round()` all call.
- **User Story 1 (Phase 3)**: depends only on Foundational (T003)
- **User Story 2 (Phase 4)**: depends only on Foundational (T003) — independent of Story 1
- **User Story 3 (Phase 5)**: depends only on Foundational (T003) — independent of Stories 1 and 2
- **Polish (Phase 6)**: T013 depends on all three stories (T006, T009, T012); T014 depends on T005,
  T008, T011; T015-T016 depend on every phase above being complete

### Within Each User Story

- Tests (T004, T007, T010) are written first and fail to compile until their story's implementation
  tasks land, mirroring feature 054's pattern
- Within each story, the `const` row table precedes the public function that calls `build` over it

### Parallel Opportunities

- T001 and T002 (independent documents)
- Once T003 lands, Stories 1, 2 and 3 (T004-T006, T007-T009, T010-T012) touch the same file but
  disjoint regions (different consts, different functions, different test names) and can be drafted
  in parallel by different people before being merged into one `lib.rs`
- Within each story, its `[P]`-marked test task and `const` task touch different regions of the same
  file and can be drafted in parallel; the function task depends on both

---

## Parallel Example: User Story 1

```bash
# Once Phase 2 (T003) has landed, draft together:
Task: "Add double() tests in crates/monospace-glyph-sets/src/lib.rs"
Task: "Add DOUBLE const table in crates/monospace-glyph-sets/src/lib.rs"
```

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: Foundational (CRITICAL — blocks all stories)
3. Complete Phase 3: User Story 1 — the Double table
4. **STOP and VALIDATE**: `cargo test -p monospace-glyph-sets` green, per the Independent Test above
5. This alone gives users a second stroke to choose besides Light and ASCII — the reason the issue
   was raised

### Incremental Delivery

1. Setup → the documents describe the end state; Foundational → the shared helper exists, `ascii()`
   unchanged in behavior
2. User Story 1 → the Double table ships (MVP)
3. User Story 2 → the Heavy table ships, independently
4. User Story 3 → the Light Round table ships, independently, proving a table sharing characters
   with Light still answers on its own
5. Polish → the mixing property (FR-008) is proven, every row is checked against the document, the
   gate is green on a fresh clone, and the learning-log entry closes the increment

### Parallel Team Strategy

With multiple developers:

1. One person completes Setup + Foundational (T001-T003)
2. Once Foundational is done:
   - Developer A: User Story 1 (Double)
   - Developer B: User Story 2 (Heavy)
   - Developer C: User Story 3 (Light Round)
3. Stories complete independently; the three land as separate `feat` commits into the same file, in
   any order, since none of the four single-stroke tables contests another's keys (FR-008)
