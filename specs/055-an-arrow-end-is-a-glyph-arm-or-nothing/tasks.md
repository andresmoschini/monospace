---
description: "Task list for feature 55: an arrow's end is a glyph or an arm"
---

# Tasks: An arrow's end is a glyph or an arm

**Input**: Design documents from `/specs/055-an-arrow-end-is-a-glyph-arm-or-nothing/`

**Prerequisites**: plan.md, spec.md, research.md, decisions.md, data-model.md,
contracts/core-api.md, contracts/diagram-api.md, contracts/description-format.md, quickstart.md

**Tests**: Requested by the spec's _Testing expectations_ — one named test per terminal, one per
stamp order, one for the body, and the 1856-rendering characterization as the standing record. Unit
tests in the core are the minimum the spec asks for.

**Organization**: The spec carries no user-story section — its own checklist says so, and puts the
two flows in _Behavior_ in its place. The three groups there are the three stories below, and the
order is the spec's own, forced rather than chosen: B2's arm terminal is a value B1 introduces, and
B3 asserts that nothing B1 and B2 did moved. Priorities are assigned here for the same reason.

The order is also the commit order, and the constitution's principle V is what puts the rename in
its own phase between the model and the terminal. `Endpoint::head` → `terminal` is structural and
carries no behavior; the tagged enum that replaces it is behavior and changes what a malformed file
reports. One commit cannot be both, and the workspace is only green with all three crates moved
together, so the split is expand/contract rather than one commit with a mixed prefix.

One consequence is recorded rather than hidden: three documents carrying a `<!-- render -->` marker
are rewritten twice, once for the rename and once for the tag. The alternative — renaming the field
and tagging it in one commit — is the mixed commit principle V forbids.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different files, no dependencies). A `[P]` task inside Phase 3 or
  Phase 4 is still committed **with** the phase's other tasks, because the workspace is green only
  when all of them have landed; the marker says the edits do not collide, not that the commits
  separate.
- **[Story]**: US1 = "One field, a tag on it, and two things it can name" (P1, spec's B1). US2 = "An
  arm terminal composes; a glyph terminal covers" (P2, spec's B2). US3 = "Nothing about the body
  moves" (P3, spec's B3).
- Include exact file paths in descriptions.

## Path Conventions

Single Rust workspace, four crates. A test lives in the `#[cfg(test)] mod tests` of the file that
owns the behavior, unless it is the end-to-end one in `crates/monospace-cli/tests/cli.rs`. The
core's tests sit beside the code in `crates/monospace-core/src/shape/arrow.rs`; the eight pinned
pictures sit in `crates/monospace-core/src/snapshots/characterization/`. Measurements written for
quickstart's scenarios go under `target/`, which is untracked — they are measurements, not
artifacts.

---

## Phase 1: Setup — the baseline, before the first edit

**Purpose**: three facts quickstart's later scenarios diff against, captured while the code still
draws today's picture. None can be produced afterwards.

- [ ] T001 Run `cargo xtask check` and record that it is green before anything is edited. This is
      the only definition of green (constitution, principle III), and a red baseline is a finding to
      report rather than to fix inside this feature.
- [ ] T002 [P] Capture the bare demonstration's current output with
      `cargo run -q -p monospace-cli > target/before-055.txt`. Phase 6 diffs against this file, and
      after the rename the binary no longer reads the key the file was written with.
- [ ] T003 [P] Run `cargo test -p monospace-core sweep` and record that all 1856 renderings across
      the eight files in `crates/monospace-core/src/snapshots/characterization/` pass with nothing
      pending. This is what makes the sweep a demonstrated guard before it is a claimed one
      (constitution, principle IV).

**Checkpoint**: the gate is green, the demonstration's output is on disk, and the sweep is known to
hold. Nothing has changed.

---

## Phase 2: Foundational — the model grows the vocabulary

**Purpose**: the model changes first and ahead of any Rust, in the same increment, because a slice
that needs a rule the model does not have has the model grow it before the code does. These five
places are research.md Q9's inventory, and one commit carries all of them — a vocabulary table
reading "and a head" beside a definition reading "and a terminal" is a state no reader could act on.

**⚠️ CRITICAL**: this MUST land before Phase 3, and before any Rust in Phase 4.

- [ ] T004 `docs`: replace the endpoint's definition in _The initial set_ in `docs/model.md` with
      the drafted sentence from research.md Q9 — an endpoint is a position, the direction the arrow
      leaves it in, and a **terminal**; `at` is the cell the terminal hangs from; and what a
      terminal may write is vocabulary this document owns, so naming one more of them is a change
      here and in the figure and not a change to any record.
- [ ] T005 `docs`: retitle _An end is an arm; a head is a glyph_ to _What a terminal writes_ in
      `docs/model.md` and carry the three drafted paragraphs of research.md Q9, keeping ADR-0029's
      argument entire — the title enumerated two members, and a third falsifies it — and keeping the
      open question below it.
- [ ] T006 `docs`: add the one clause to _The route of an arrow_ in `docs/model.md` saying the path
      writes no endpoint cell whatever the terminal puts there. A clause, not a rule, and not a term
      in `Cost` (research.md Q2).
- [ ] T007 `docs`: update the open question in `docs/model.md` §11 that T005's retitled section
      points at, so it speaks of where a terminal's glyph comes from rather than where a head's does
      (decisions.md D4). The question stays open; this slice does not answer it.
- [ ] T008 [P] `docs`: amend §6 _Attachment_ in `docs/diagram-model.md` — the two sentences that
      call `model.md`'s definition unchanged, and the one that names "the head it carries" — so the
      endpoint this layer mirrors gains the same terminal and nothing claims otherwise.
- [ ] T009 [P] `docs`: update the three citations of the retitled section in the module docs of
      `crates/monospace-core/src/shape/fragment/head.rs`,
      `crates/monospace-core/src/shape/fragment/end.rs` and
      `crates/monospace-core/src/shape/line.rs`, which name a section that T005 no longer leaves in
      the tree.

**Checkpoint**: `cargo xtask check` is green, `cargo run -q -p monospace-cli` is unchanged, and no
picture moved. No ADR is added, revised or reopened: ADR-0063 took this decision on 2026-09-27 and
names the one condition that would reopen it, which naming one more thing an endpoint can write does
not meet (research.md Q6).

---

## Phase 3: Foundational — the field is renamed

**Purpose**: `Endpoint::head` becomes `terminal` in three crates, on the wire and in every document
that spells the key, with the value still a bare glyph string. Nothing observable changes, which is
what makes it structural: principle V requires the commit to add and modify no test, so the call
sites inside existing tests move and no assertion does.

**⚠️ CRITICAL**: one `refactor(core,diagram,cli)` commit carries T010–T018. The core's field rename
breaks the diagram crate's build on its own, and the diagram's breaks the CLI's, so no subset of
these reaches green alone.

- [ ] T010 [P] `refactor(core)`: rename `Endpoint::head` to `Endpoint::terminal` in
      `crates/monospace-core/src/shape/arrow.rs` — the field, its rustdoc, the two reads in
      `Arrow::draw`, and the `endpoint()` test helper — keeping the type `Glyph`. The field keeps
      its position in the struct; only its name and its documentation's subject change.
- [ ] T011 [P] `refactor(diagram)`: rename the mirror's field and the line in its `From` in
      `crates/monospace-diagram/src/shape.rs`, keeping the type `Glyph`. The mirror itself is kept
      and its own rustdoc's reason is unchanged (research.md Q5).
- [ ] T012 [P] `refactor(diagram)`: rename the field in the two test fixtures at
      `crates/monospace-diagram/src/diagram.rs:285,290`.
- [ ] T013 [P] `refactor(cli)`: rename the private `Endpoint` field, the line in its `From`, and the
      test `a_multi_grapheme_head_fails_to_deserialize` with its JSON string in
      `crates/monospace-cli/src/description.rs`, keeping `deserialize_glyph` on a bare string. The
      rename follows the field, not the other way round (research.md Q3).
- [ ] T014 [P] `refactor(cli)`: rename the JSON key on both endpoints in
      `crates/monospace-cli/assets/demo.json`, the value still the bare string it is today.
- [ ] T015 [P] `refactor(cli)`: rename the JSON key on the arrow's two endpoints in
      `crates/monospace-cli/tests/cli.rs:134-135`. The picture pinned at line 150 must not move.
- [ ] T016 `refactor(docs)`: rename the key in the three documents that carry a `<!-- render -->`
      marker — `README.md:56-57`, `docs/model.md:343-344`,
      `specs/055-an-arrow-end-is-a-glyph-arm-or-nothing/spec.md:78-79,110-111` — then run
      `git add -A && cargo xtask render` and `git diff`. **No picture may change**: a glyph terminal
      renders the glyph it always did, and that empty diff is what a structural commit changing no
      behavior looks like from the outside.
- [ ] T017 [P] `refactor(docs)`: rename the key at `CONTRIBUTING.md:235-236`, inside the
      four-backtick fence that illustrates the render marker rather than instantiating it.
- [ ] T018 [P] `refactor(docs)`: rename the key at
      `specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md:78-79`, the
      contract this feature's own supersedes. Leave
      `specs/045-simplify-cli-to-demo-shapes/contracts/description-format.md:60-61` exactly as it
      is: it is the record of what feature 045 shipped, and it already documents a `mode` field the
      binary stopped reading, which is the precedent (research.md Q7).

**Checkpoint**: `cargo xtask check` is green, `cargo run -q -p monospace-cli` is byte-identical to
`target/before-055.txt`, the sweep's eight files are untouched, and no test was added and no
assertion changed.

---

## Phase 4: User Story 1 — One field, a tag on it, and two things it can name (P1) 🎯 MVP

**Goal**: an endpoint's terminal is one tagged value that is either a chosen glyph or a single arm,
and a description names it with a `kind`. The field is present for both values, so a terminal the
model has not named yet can arrive as a third tag without changing the shape of a description.

**Independent Test**: the spec's B1 arrangement rendered through the binary — two 3×3 boxes on an
11×3 canvas, an arrow between them, each endpoint standing on the nearer box's border cell. A glyph
terminal at both ends draws the picture in _B1_ unchanged; the same arrangement with arm terminals
draws the one in _B2_. Then quickstart Scenario 5's four files, each of which must be refused by
name. None of this needs User Story 2 or 3.

**One commit**: T019–T031 land together as `feat(core,diagram,cli)`. Adding `Terminal` changes the
core's `Endpoint`, which the diagram's `From` constructs and the CLI's builds, so no prefix of the
set compiles. Write each test first and watch it fail — the four in T019–T022 fail to compile until
T023 and T028 land, which is the failure worth watching — then land the set green.

### Tests for User Story 1

- [ ] T019 [P] [US1] `test(core)`: contract test that a glyph terminal writes a literal nothing
      composes into — in `crates/monospace-core/src/shape/arrow.rs`, stamp the arrow, then stamp an
      arm-bearing neighbor over one endpoint and assert the cell is still `Cell::Literal`. Pins
      spec's B1 scenario 1 and data-model invariant 3.
- [ ] T020 [P] [US1] `test(core)`: contract test that an arm terminal writes one arm and three
      undecided sides, in the arrow's own stroke — in `crates/monospace-core/src/shape/arrow.rs`,
      asserting the `StrokeCell` with `Arm::Set(self.stroke)` on the side `leaving` names and
      `Unset` on the other three, for all four leaving directions. Pins spec's B1 scenario 2 and
      data-model invariant 4.
- [ ] T021 [P] [US1] `test(cli)`: contract tests for quickstart Scenario 5's four refusals in
      `crates/monospace-cli/src/description.rs` — an unrecognized `kind` names both accepted
      variants, a multi-grapheme `glyph` keeps the message the old `head` produced byte for byte, a
      missing field names `terminal`, and the externally tagged `"terminal": "arm"` is refused. Pins
      SC-005 and spec's B1 scenario 3.
- [ ] T022 [P] [US1] `test(diagram)`: contract test that the mirror's terminal reaches the core's
      intact, both variants surviving the `From` — in `crates/monospace-diagram/src/shape.rs`. A
      caller takes `Terminal` from `monospace_core`, and this is what says so.

### Implementation for User Story 1

- [ ] T023 [US1] `feat(core)`: add `Terminal` to `crates/monospace-core/src/shape/arrow.rs` —
      `Glyph { glyph: Glyph }` and `Arm`, deriving `Debug`, `Clone`, `PartialEq` and `Eq`, and not
      `Copy` because `Glyph` owns its text — and change `Endpoint::terminal` to hold it. `Clone`
      because `Shape::draw` takes `&self`; no constructor, no `Default`, no accessor, and the enum
      is closed at two and not `#[non_exhaustive]`.
- [ ] T024 [US1] `feat(core)`: add the private `Direction` → `Side` mapping beside `Terminal` in
      `crates/monospace-core/src/shape/arrow.rs` — `Right`→`Side::Right`, `Down`→`Side::Bottom`,
      `Left`→`Side::Left`, `Up`→`Side::Top`. `Side` is crate-private, so this is a function and not
      a field on the data. It is the leaving direction's own side and not its opposite: the arm
      faces the route, which begins one step from `at` in that direction.
- [ ] T025 [US1] `feat(core)`: rewrite `Arrow::draw` in `crates/monospace-core/src/shape/arrow.rs`
      to match on each endpoint's terminal — `Glyph` through `Head`, `Arm` through
      `shape::fragment::end::End` with the arrow's own stroke — and to call `derive_path` with the
      two `at`s and the two `leaving`s exactly as it does today. `derive_path` takes no terminal and
      cannot be made to, so it does not move.
- [ ] T026 [P] [US1] `feat(core)`: re-export `Terminal` from `crates/monospace-core/src/lib.rs`
      beside `Endpoint`, since a caller takes it from the crate root.
- [ ] T027 [US1] `feat(diagram)`: change the mirror's field to `monospace_core::Terminal` in
      `crates/monospace-diagram/src/shape.rs` and drop the line it loses from the `From` — the
      terminal moves across whole. This crate re-exports nothing; a caller already takes `Pos`,
      `Direction` and `Glyph` from `monospace_core` and now takes `Terminal` the same way.
- [ ] T028 [US1] `feat(cli)`: add the private wire `Terminal` in
      `crates/monospace-cli/src/description.rs` —
      `#[serde(tag = "kind", rename_all = "lowercase")]`, `Glyph { glyph }` carrying
      `deserialize_glyph` on the named field, `Arm` a unit variant — and convert it in the same
      `From` that already converts `at` and `leaving`. Internally tagged is what puts FR-014's check
      where it fits: `Glyph(Glyph)` does not compile (research.md Q3).
- [ ] T029 [US1] `feat(cli,docs)`: write the tagged form across the eight files that carry the key —
      `crates/monospace-cli/assets/demo.json:67-68`, `crates/monospace-cli/tests/cli.rs:134-135`,
      `crates/monospace-cli/src/description.rs:254-255`, `README.md:56-57`, `docs/model.md:343-344`,
      `specs/055-an-arrow-end-is-a-glyph-arm-or-nothing/spec.md:78-79,110-111`,
      `CONTRIBUTING.md:235-236`,
      `specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md:78-79`. The
      value is `{"kind": "glyph", "glyph": "◄"}`; the pictures stay what they are.
- [ ] T030 [US1] `feat(core)`: add the `Design notes` rustdoc in
      `crates/monospace-core/src/shape/arrow.rs` — what a terminal writes, why a glyph and an arm
      are two values of one field rather than a field and its absence, and why the set is closed at
      two. This is the altitude ADR-0063 put the rule at: module-level, observable only in the
      picture, so it takes no ADR and amends no model document.
- [ ] T031 [US1] `feat(docs)`: run `git add -A && cargo xtask render` and `git diff --stat`. **Every
      picture must be byte-identical** — a glyph terminal renders the glyph it always rendered,
      which is SC-001's first half measured rather than asserted.

**Checkpoint**: SC-001's first half and SC-005 hold. `cargo xtask check` is green,
`cargo run -q -p monospace-cli` still matches `target/before-055.txt`, and all 1856 renderings are
untouched. This is the MVP — the terminal has two values and the wire can name either.

---

## Phase 5: User Story 2 — An arm terminal composes; a glyph terminal covers (P2)

**Goal**: the shared cell is the figure in front's, and the two terminals differ observably in it —
a glyph decides every side, an arm leaves three undecided, so what survives a collision is the order
the caller wrote the shapes in.

**Independent Test**: the spec's B2 arrangement, two 3×3 boxes and an arrow, rendered twice with the
second box above the arrow in `shapes` and once below it, once per terminal. Needs User Story 1 for
the arm; needs nothing from User Story 3.

**No implementation by design**: decisions.md D3 answers that a glyph and an arm landing on one cell
have no rule of their own — the general stamping rule in `docs/model.md` _Stamping_ already decides
it, and the two pictures are rows three and four of that table (research.md Q4). T034 and T035 are
therefore the whole increment, and they land as one `test(diagram)` commit. **If either fails,
stop.** The defect would be in what a terminal writes, and a rule for a glyph and an arm on one cell
is a decision `decisions.md` does not hold: write the entry and ask the maintainer rather than
resolving it here (constitution, _No decision outside the sheet_).

- [ ] T032 [US2] Write quickstart Scenario 2's description verbatim to
      `target/arm-both-ends-below.json` — the three shapes with the second box after the arrow in
      `shapes` — and render it with
      `cargo run -q -p monospace-cli -- target/arm-both-ends-below.json`. The picture must be the
      second one in the spec's _B2_: the border cell at each end a junction, `├` and `┤`, and the
      left box's right-hand border surviving. Untracked, so it is a measurement and not an artifact.
- [ ] T033 [US2] Write the same three shapes with the second box moved above the arrow in `shapes`
      to `target/arm-both-ends-above.json` and render it. The two outputs must be equal byte for
      byte — the assertion SC-003 cannot make on a picture alone, and the reason the `shapes` array
      order is the only thing the two files differ in now that no shape carries a `mode`.
- [ ] T034 [P] [US2] `test(diagram)`: contract test that the same arrangement with glyph terminals
      at both ends draws two different pictures, and that the one drawn between the boxes has lost
      the left box's right-hand border cell — in `crates/monospace-diagram/src/shape.rs`, two
      `Diagram`s differing only in the order shapes were added. Pins SC-004 and spec's B2
      scenario 1.
- [ ] T035 [US2] `test(diagram)`: contract test that the same arrangement with arm terminals at both
      ends draws one identical picture in both orders, the border cell a junction in each — in
      `crates/monospace-diagram/src/shape.rs`, the assertion T033 measures, made permanent. Pins
      SC-003 and spec's B2 scenario 2.

**Checkpoint**: SC-003 and SC-004 hold. `cargo xtask check` is green and no picture pinned anywhere
moved.

---

## Phase 6: User Story 3 — Nothing about the body moves (P3)

**Goal**: the route is derived from the two positions and the two leaving directions alone, so the
body is the same five cells whatever the terminal is, the path writes no endpoint cell at all, and
none of the 1856 pinned renderings — every one of them a glyph terminal — changes.

**Independent Test**: run `cargo test -p monospace-core sweep` and read the count; then diff the
bare demonstration against `target/before-055.txt`. Needs User Story 1 for the arm; needs nothing
from User Story 2.

- [ ] T036 [US3] `test(core)`: contract test that one arrow with each of the two terminals in turn
      writes the same body — in `crates/monospace-core/src/shape/arrow.rs`, rendering both into the
      same window and asserting the two texts differ at the two endpoint cells and nowhere else.
      Pins SC-002 and spec's B3 scenario 1.
- [ ] T037 [US3] `test(core)`: contract test that the path writes no endpoint cell whatever the
      terminal is — in `crates/monospace-core/src/shape/arrow.rs`, using the existing
      `CountingSurface` to count writes per position for both terminals, asserting each terminal
      writes its own position exactly once and no route position is one of the two. Pins spec's B3
      scenario 2 and data-model invariant 2.
- [ ] T038 [US3] Run `cargo test -p monospace-core sweep` and record that all 1856 renderings across
      the eight files in `crates/monospace-core/src/snapshots/characterization/` are unchanged. **If
      one moved, do not accept the diff**: report how many cases moved, in which families, and three
      of them before and after, which is what the constitution asks for in place of a review nobody
      can keep.
- [ ] T039 [US3] Read `crates/monospace-cli/tests/cli.rs` and confirm no test pins the cell D5's
      change moves, at `(13, 3)` of the demonstration's first picture. If one does, it is renamed
      and re-pinned against the new picture, the way feature 039 re-pinned
      `identical_directions_in_line_are_joined_around_the_outside` — never deleted.
- [ ] T040 [US3] `feat(cli)`: change the demonstration's `from` to `{"kind": "arm"}` in
      `crates/monospace-cli/assets/demo.json:67`, leaving its `to` a glyph (decisions.md D5). This
      is its own `feat(cli)` commit and the only behavior change left in the slice.
- [ ] T041 [US3] Run `cargo run -q -p monospace-cli | diff target/before-055.txt -` and record the
      result: one changed line per picture, both on the arrow's row, `│◄────┐` becoming `│─────┐`,
      and nothing else in the diff. The arm joins nothing because the box it would have joined ends
      one cell short of it — the sentence decisions.md D5 hangs that picture on.

**Checkpoint**: SC-002, SC-006 and spec's B3 scenario 3 hold. `cargo xtask check` is green, exactly
one cell of each demonstrated picture moved, and the sweep is untouched.

---

## Phase 7: Polish & cross-cutting concerns

- [ ] T042 [P] `docs`: append the increment's entry to `docs/learning-log.md` — what was learned
      about Rust design and idiom (one tagged enum carrying the sum, and a private `Direction` →
      `Side` mapping as a function rather than a field because `Side` is crate-private), what was
      learned about working this way, and the trade-off worth remembering: under principle V a
      wire-format rename and the type that replaces it cannot share a commit, so three documents
      carrying a render marker are rewritten twice.
- [ ] T043 [P] Confirm nothing in `docs/decisions/` was added, revised or reopened. ADR-0063 took
      this decision on 2026-09-27 and names the one condition that would reopen it — the model's
      account of what an endpoint may write being replaced rather than extended. This slice extends
      it (research.md Q6).
- [ ] T044 [P] Confirm the three new contracts each supersede exactly one existing file in exactly
      one respect, and that `specs/045-simplify-cli-to-demo-shapes/contracts/description-format.md`
      is byte-identical to what it was. A second copy of a superseded contract is the defect, and
      one file covering all three surfaces is the one a reader cannot find by the interface it
      describes (constitution, principle VI _Subject_).
- [ ] T045 Run `git add -A && cargo xtask render` and confirm `git diff --stat` reports nothing,
      then run `cargo xtask check` and read its output rather than trusting the exit code alone.
- [ ] T046 `docs`: tick exactly the checkboxes in this file that the completed commits completed —
      one commit per task where a task reached green alone, one per group where it did not — and say
      in the message which story each commit advances (constitution, principle II).

---

## Dependencies & Execution Order

One linear order, and the phases are its commits. The chain is forced at every link:

- **Phase 1** (Setup): no dependencies. T002 and T003 are `[P]` and unblock nothing — they exist so
  a later claim can be measured rather than assumed.
- **Phase 2** (the model's vocabulary) blocks Phases 3 and 4. The model changes first, ahead of any
  Rust, because a slice that needs a rule the model does not have has the model grow it before the
  code does. T004–T007 all edit `docs/model.md` and land in one commit, so they are sequential; T008
  and T009 touch other files and are `[P]`.
- **Phase 3** (the rename) blocks Phase 4. The field has to be called `terminal` before a tagged
  value can be carried in it, and the wire has to read `terminal` before it can read
  `{"kind": ...}`. All nine tasks land in one `refactor` commit because no subset compiles.
- **Phase 4** (User Story 1, P1) blocks Phases 5 and 6: an arm terminal has to exist before either
  can be tested, and the demonstration's `from` cannot become one before.
- **Phase 5** (User Story 2, P2) depends on Phase 4 only. It is pure assertion and can be developed
  in parallel with Phase 6 by someone else, since it touches no file Phase 6 touches.
- **Phase 6** (User Story 3, P3) depends on Phase 4. T036–T038 are one `test(core)` commit;
  T039–T041 are one `feat(cli)` commit, and T038's sweep run sits between them so the second is
  checked too.
- **Phase 7** (Polish) is last, and T045 re-checks the whole gate after every behavioral commit
  rather than once at the end.

### Within a phase

- Tests are written before the implementation and watched fail — a compile error is a failure worth
  watching where the type does not exist yet — then land in the same commit as what they pin.
- Models before services, services before endpoints: T023 and T024 before T025, T027 and T028 before
  the wire in T029.
- The `[P]` tasks inside Phases 3 and 4 collide on no file, and still commit together.

---

## Parallel Opportunities

21 of the 46 tasks are marked `[P]`. The three clusters worth naming:

```text
Phase 1:  T002 (demo output to target/)          T003 (sweep green)
Phase 2:  T008 (docs/diagram-model.md §6)        T009 (three module docs)
Phase 3:  T010 core   T011+T012 diagram          T013+T014+T015 cli
          T017 CONTRIBUTING.md                   T018 the 079 contract
Phase 4:  T019 core   T020 core   T021 cli        T022 diagram     T026 lib.rs
```

- T011 and T012 are both `monospace-diagram` but different files, so they are separable; they commit
  together because T010's field rename breaks the crate until both are in.
- Once Phase 2 lands, Phases 3 and 4 are strictly ordered and no parallelism is safe between them.
- Phases 5 and 6 are independent of each other once Phase 4 is green: Phase 5 touches
  `crates/monospace-diagram/src/shape.rs`, Phase 6 touches
  `crates/monospace-core/src/shape/arrow.rs` and `crates/monospace-cli/`.
- T042, T043 and T044 are three independent verifications at the end.

---

## Implementation Strategy

### MVP first (Phases 1–4, T001–T031)

Stop after Phase 4 and the slice is demonstrable: the model owns the vocabulary, the field is called
`terminal`, a terminal is a glyph or an arm, the wire names either with a `kind`, and **nothing
visible has changed** — the demonstration, the sweep and every pinned picture are where they were.
That is SC-001 and SC-005, and it is the point at which the rename's cost is already paid and the
risk is gone. Phases 5 and 6 add assertions and one demonstration cell rather than capability.

### Incremental delivery

1. Phases 1–3: baseline, model, rename. Three commits, `cargo run -q -p monospace-cli` unchanged at
   every one.
2. Phase 4: the terminal and its tag. One `feat` commit. The MVP.
3. Phase 5: the composition distinction, asserted in both orders. One `test` commit, no behavior.
4. Phase 6: the body's invariance and the demonstration's one cell. One `test` commit and one `feat`
   commit.
5. Phase 7: the learning-log entry and the three verifications.

Each phase ends at a checkpoint that is green on its own, and every commit in it is one a reviewer
can read for a single reason.

---

## Notes

- Every task in Phases 3 and 4 carries its Conventional Commits prefix inline, because the split
  between them is the constitution's principle V and not a matter of taste: a `refactor` commit adds
  and modifies no test, and a `feat` commit may.
- T010–T015 do modify call sites inside existing tests. No assertion changes, and the eight
  characterization files are the evidence: a structural commit that moved a picture would be a
  behavioral one.
- `cargo xtask render` walks `git ls-files '*.md'`, so a marker edited but not staged is invisible
  to it and the step reports nothing at all. Every render task in this file stages first.
- Measurements belong under `target/`. The two files T032 and T033 write are the ones quickstart
  calls worth keeping, and `target/` is where a file that is not an artifact goes.
- Stop and ask rather than resolve: a rule for a glyph and an arm on one cell (Phase 5), a third
  terminal, whether a terminal's glyph may come from a glyph set, an endpoint hanging from a shape's
  side anchor, and what a diagram derives for the leaving direction. Each is named in the spec's
  _What this slice does not decide_, and none of them may be settled here.
