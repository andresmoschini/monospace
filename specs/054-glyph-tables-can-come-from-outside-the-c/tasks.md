---
description: "Task list for feature 054: glyph tables can come from outside the core"
---

# Tasks: Glyph tables can come from outside the core

**Input**: Design documents from `/specs/054-glyph-tables-can-come-from-outside-the-c/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [research.md](research.md),
[data-model.md](data-model.md),
[contracts/glyph-set-extension-point.md](contracts/glyph-set-extension-point.md),
[quickstart.md](quickstart.md)

**Tests**: Included. The constitution's Testing constraint makes unit tests for core logic the
minimum any spec may ask for, and this spec's own quickstart.md names the tests each story must have
passing.

**Organization**: Tasks are grouped by user story (spec.md's P1/P2/P3) so each can be implemented
and verified independently, per the plan's Constitution Check (principle V: `from_rules`/`union`
land as one `feat`, the CLI switch and demo as a separate `feat`, kept apart).

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)

## Phase 1: Setup

**Purpose**: Correct the model and the reference data first (the constitution requires the model to
match before code relies on it — plan.md Summary), and scaffold the new crate the later phases fill
in.

- [x] T001 Correct §5 _Strokes, glyph sets and the catalog_ in `docs/model.md`: it currently says
      the common sets "ship with the library" (singular); amend it to say a set may ship with
      whichever library holds it, since `monospace-glyph-sets` now ships one too (FR-020)
- [x] T002 [P] Correct the opening note of `docs/glyph-sets.md` (currently "Nothing loads them yet")
      to say the ASCII table is now carried as data by `monospace-glyph-sets` and Light by
      `monospace-core`, while Double, Heavy, Light Round and the mixing sets remain reference only
      (FR-021)
- [x] T003 Create `crates/monospace-glyph-sets/Cargo.toml`: package metadata matching
      `crates/monospace-cli/Cargo.toml`'s shape (`name = "monospace-glyph-sets"`, a one-line
      `description`, `version.workspace = true`, `edition.workspace = true`,
      `authors.workspace = true`, `license.workspace = true`), a `monospace-core.workspace = true`
      dependency, and `[lints] workspace = true`
- [x] T004 Register `monospace-glyph-sets` in the root `Cargo.toml`: add
      `"crates/monospace-glyph-sets"` to `[workspace].members` and
      `monospace-glyph-sets = { path = "crates/monospace-glyph-sets" }` to
      `[workspace.dependencies]` (depends on T003)
- [x] T005 Create `crates/monospace-glyph-sets/src/lib.rs` with a crate-level (`//!`) doc comment
      describing what the crate holds, so `cargo build --workspace` succeeds with the new empty
      member (depends on T004)

**Checkpoint**: `cargo build --workspace` succeeds with three crates; the docs read correctly before
any code depends on them.

---

## Phase 2: User Story 1 - A table written outside the core answers keys like any other (Priority: P1) 🎯 MVP

**Goal**: `monospace-core` gains the extension point itself — `GlyphCatalog::from_rules` and
`GlyphCatalog::union` — with no new type, per research.md's decision.

**Independent Test**: a table defined in a test module (standing in for a library outside the core),
built into a catalog with `from_rules`, combined with `GlyphCatalog::light()` via `union`, answers
keys from both sides correctly regardless of order except where contested, per
contracts/glyph-set-extension-point.md.

### Tests for User Story 1 ⚠️

- [x] T006 [P] [US1] In `crates/monospace-core/src/glyph.rs`'s `tests` module, add unit tests for
      `GlyphCatalog::from_rules` and `GlyphCatalog::union` using rules defined in the test module
      itself (not `LIGHT`): a key only one side holds answers from that side regardless of union
      order (Acceptance Scenario 1.3); a key two catalogs both claim answers from whichever was
      given first to `union`, and reversing the order reverses the answer (Acceptance Scenario 1.4,
      SC-006); an empty `rules` iterator and a zero- or one-element `catalogs` list each produce a
      valid, unsurprising catalog (Edge Cases). These tests fail to compile until T007-T008 land.

### Implementation for User Story 1

- [x] T007 [US1] Implement
      `GlyphCatalog::from_rules(rules: impl IntoIterator<Item = (GlyphKey, Glyph)>) -> Self` in
      `crates/monospace-core/src/glyph.rs`: first claim wins within `rules`, via
      `HashMap::entry(..).or_insert(..)` (FR-001, FR-002)
- [x] T008 [US1] Implement `GlyphCatalog::union(catalogs: impl IntoIterator<Item = Self>) -> Self`
      in `crates/monospace-core/src/glyph.rs`, `#[must_use]`: folds each catalog's rules into one
      map in the order given, first claim wins across catalogs (FR-002, FR-003)
- [x] T009 [US1] Reimplement `GlyphCatalog::light()` in `crates/monospace-core/src/glyph.rs` to read
      `Self::from_rules(LIGHT.iter().map(...))` (converting each `Row` into a `(GlyphKey, Glyph)`
      pair the way it does today), keeping its signature, its doc comment and its `#[must_use]`
      unchanged, so every existing caller keeps compiling and keeps producing the same catalog
      (FR-005, FR-006, SC-008)

**Checkpoint**: `cargo test -p monospace-core` is green, including the new tests from T006; no
existing test in `monospace-core` changed (FR-005, SC-008). This is the MVP — the extension point
exists and is proven from outside the core's own data.

---

## Phase 3: User Story 2 - A diagram drawn in ASCII alone (Priority: P2)

**Goal**: `monospace-glyph-sets` holds the ASCII table and exposes it as a catalog, built the same
way `monospace-core`'s `light()` is built.

**Independent Test**: a catalog built from `monospace_glyph_sets::ascii()` alone renders a box as
`+`, `-` and `|`, and every character of any diagram rendered against it is printable ASCII or a
space.

### Tests for User Story 2 ⚠️

- [x] T010 [P] [US2] In `crates/monospace-glyph-sets/src/lib.rs`, add a `tests` module mirroring
      `monospace-core`'s Light tests: `ascii()` answers all fifteen non-empty combinations of the
      `ascii` stroke (SC-005, mirrors `light_answers_every_non_empty_combination_of_its_own_stroke`
      in `crates/monospace-core/src/glyph.rs`); a box drawn with `monospace_core::BoxShape` and
      rendered against a catalog built from `ascii()` alone produces only `+`, `-`, `|` or space
      characters, checked over every character of the output (SC-004). These tests fail to compile
      until T011-T012 land.

### Implementation for User Story 2

- [x] T011 [US2] In `crates/monospace-glyph-sets/src/lib.rs`, add a private `const ASCII: &[Row]`
      table (reusing `monospace-core`'s private `Row` tuple shape —
      `(Option<&'static str>, Option<&'static str>, Option<&'static str>, Option<&'static str>, &'static str)`
      — declared locally since it is not exported by `monospace-core`) holding the fifteen rows
      `docs/glyph-sets.md`'s _ASCII_ table records, verbatim and in the same order, with `"ascii"`
      as the stroke name (FR-011)
- [x] T012 [US2] In `crates/monospace-glyph-sets/src/lib.rs`, implement
      `pub fn ascii() -> monospace_core::GlyphCatalog`, `#[must_use]`, built via
      `GlyphCatalog::from_rules` over `ASCII` the same way `monospace-core::glyph::light()` builds
      its catalog over `LIGHT` (FR-007, FR-009, FR-012)
- [x] T013 [US2] Extend the `wasm` step's package list in `xtask/src/main.rs` with
      `"-p", "monospace-glyph-sets"` alongside `"monospace-core"`, so `cargo xtask check` proves
      this crate builds for `wasm32-unknown-unknown` too (FR-010)

**Checkpoint**: `cargo test -p monospace-glyph-sets` is green;
`cargo check -p monospace-core -p monospace-glyph-sets --target wasm32-unknown-unknown` succeeds.
User Stories 1 and 2 both work independently of the CLI.

---

## Phase 4: User Story 3 - The shipped demonstration shows both at once (Priority: P3)

**Goal**: the CLI renders its demo from a catalog that unions Light and ASCII, and the shipped
description gains ASCII shapes and two crossings that show which table answers a shared cell.

**Independent Test**: `cargo run -p monospace-cli` with no arguments prints one diagram containing
both box-drawing characters and `+`/`-`/`|` characters, with the two crossing cells reading from
whichever figure is in front.

### Implementation for User Story 3

- [x] T014 [US3] Add `monospace-glyph-sets.workspace = true` to `[dependencies]` in
      `crates/monospace-cli/Cargo.toml`
- [x] T015 [US3] In `crates/monospace-cli/src/description.rs`, change `Description::render` to build
      its catalog with `GlyphCatalog::union([GlyphCatalog::light(), monospace_glyph_sets::ascii()])`
      instead of `GlyphCatalog::light()` alone (FR-014)
- [x] T016 [US3] Add shapes whose `"stroke"` is `"ascii"` to
      `crates/monospace-cli/assets/demo.json`, on the existing canvas alongside the current
      `"light"` shapes, growing `canvas.size` if needed to fit them (FR-015, FR-016); leave every
      existing shape's position, size and stroke unchanged (FR-018)
- [x] T017 [US3] Add two crossings between an ASCII figure and a Light figure to
      `crates/monospace-cli/assets/demo.json`: one where the ASCII figure's `"mode"` is `"above"`
      relative to a Light figure already on the canvas, one where a Light figure's `"mode"` is
      `"above"` relative to an ASCII figure, so each shared cell is decided by which figure is in
      front (FR-017)
- [x] T018 [US3] Extend `crates/monospace-cli/tests/cli.rs`: assert the no-argument run's stdout
      contains at least one box-drawing character and at least one of `+`, `-`, `|` (SC-003), and
      assert the two crossing cells added in T017 — at their known positions in the demo's output —
      hold the front figure's table's character (SC-009): the ASCII-in-front crossing is `+`, `-` or
      `|`, the Light-in-front crossing is the matching box-drawing character

**Checkpoint**: `cargo run -p monospace-cli` with no arguments shows both tables on one canvas;
`cargo test -p monospace-cli` is green, with every pre-existing assertion unchanged (FR-018, SC-008)
and the new assertions from T018 passing.

---

## Phase 5: Polish & Cross-Cutting Concerns

**Purpose**: close the increment per constitution principle II.

- [x] T019 Run the whole gate on a fresh clone (`cargo xtask check`, per constitution principle IV)
      and the manual checks in `quickstart.md` (`cargo build --workspace`, `cargo test --workspace`,
      `cargo run -p monospace-cli`,
      `cargo check -p monospace-core -p monospace-glyph-sets --target wasm32-unknown-unknown`)
- [x] T020 Append an entry to `docs/learning-log.md` for this increment: what was learned about Rust
      design (the no-new-type decision in research.md — a table and a catalog sharing one contract)
      and about working this way, with evidence from what was tried

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: no dependencies — start immediately. T001 and T002 are independent of
  T003-T005.
- **User Story 1 (Phase 2)**: depends only on Setup completing (T005, so the workspace still
  builds); does not depend on the new crate existing
- **User Story 2 (Phase 3)**: depends on Setup (the crate scaffold, T003-T005) and on User Story 1
  (T007, `GlyphCatalog::from_rules`, which `ascii()` calls)
- **User Story 3 (Phase 4)**: depends on User Story 1 (T008, `GlyphCatalog::union`) and User Story 2
  (T012, `monospace_glyph_sets::ascii()`)
- **Polish (Phase 5)**: depends on Phases 2-4 all being complete

### Within Each User Story

- Tests (T006, T010) are written first and fail (to compile, since the functions they exercise do
  not exist yet) before the implementation tasks that follow them
- Within `crates/monospace-core/src/glyph.rs`, T007 (`from_rules`) precedes T008 (`union`) and T009
  (`light()` rewritten atop `from_rules`), since T008 and T009 build on T007
- Within `crates/monospace-cli/`, T014 (dependency) precedes T015 (the code that uses it), which
  precedes T016-T017 (demo content), which precede T018 (tests asserting on that content)

### Parallel Opportunities

- T001 and T002 (independent documents)
- T006 (core tests) and T010 (glyph-sets tests) touch different crates and could be drafted in
  parallel once their respective implementation tasks are staffed, though T010 cannot pass until
  Phase 2 lands `from_rules`

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Complete Phase 1: Setup
2. Complete Phase 2: User Story 1 — `GlyphCatalog::from_rules` and `GlyphCatalog::union`
3. **STOP and VALIDATE**: `cargo test -p monospace-core` green, per the Independent Test above
4. This alone proves the whole claim (spec.md: "it is the whole claim... every other story here is a
   consequence of it")

### Incremental Delivery

1. Setup → workspace builds with the new (empty) crate and corrected docs
2. User Story 1 → the extension point exists and is proven from outside `monospace-core`'s own data
   (MVP)
3. User Story 2 → `monospace-glyph-sets` ships the ASCII table; a diagram can be rendered in ASCII
   alone
4. User Story 3 → the CLI's no-argument run demonstrates both tables answering from one catalog
5. Polish → the gate is proven green on a fresh clone and the learning-log entry closes the
   increment
