---
description: "Task list for feature 081: a shape can be removed and replaced"
---

# Tasks: A shape can be removed and replaced

**Input**: Design documents from `/specs/081-a-shape-can-be-removed-and-replaced/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [data-model.md](data-model.md),
[contracts/diagram-api.md](contracts/diagram-api.md), [research.md](research.md),
[quickstart.md](quickstart.md)

**Tests**: Requested by the spec's **Testing expectations** — contract tests for every rule
observable as a picture, unit tests for the displacement and the reader, and the rule that "a
behavior rule with no test named against it is an unfinished spec" (constitution, Testing). The two
rules the spec accepts with nothing to verify them are named in the Polish phase, not here (T040).

**Organization**: Tasks are grouped by the spec's behaviors B1 to B5, in the order
[plan.md](plan.md)'s five commits deliver them — that order is forced by constitution principles II
and V rather than chosen, so it is the priority order here too. Phases 4, 5 and 6 are three
behaviors in **one** commit, and the split is for traceability, not a license to make three.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different file, no dependency on an incomplete task)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, US5)
- Include exact file paths in descriptions

## Path Conventions

Rust cargo workspace. `crates/monospace-diagram/src/` for the diagram library, and
`crates/monospace-cli/src/` and `crates/monospace-cli/tests/` for the CLI. The unit tests for the
diagram crate live in the `#[cfg(test)] mod tests` of the file they cover, as the ones already there
do — the crate has no `tests/` directory and this slice adds no integration test to it.

### The one place the design is incomplete

`data-model.md` widens `Shape`'s derives to `Clone, PartialEq, Eq`, and `research.md` Q4 justifies
it by listing every **leaf field type** — `Pos`, `Size`, `Direction`, `Orientation`, `Stroke`,
`Glyph`, `Terminal`. `Shape::Connector` holds a fourth: `crate::Endpoint`, a struct of three of
those leaf types, which derives `Clone, Debug` and nothing else. `#[derive(PartialEq)]` on `Shape`
will not compile until `Endpoint` derives it too. T008 is that widening, and it is a blocking
prerequisite for T010 rather than a nicety. It takes no decision and needs no entry on the sheet: it
is what the compiler asks for.

---

## Phase 1: Setup

No setup is needed: no dependency is added, no crate is added, the toolchain is unchanged, and no
gate step is added — `wasm` already names `monospace-diagram` (plan.md Constitution Check, principle
III; D1).

---

## Phase 2: Foundational — one private search, and the "before" picture

**Purpose**: the linear search four methods will share, committed on its own because constitution
principle V forbids a structural change sharing a commit with a behavioral one. Plus the two
baseline files two later claims are diffed against.

- [x] T001 Run `cargo run -p monospace-cli > /tmp/demo-before.txt` and
      `cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-before.txt`, and
      keep both files until Phase 8 (quickstart.md "Before touching anything")
- [x] T002 Add `fn find(&self, id: &ShapeId) -> Option<usize>` to `Diagram` in
      `crates/monospace-diagram/src/diagram.rs`, the linear scan `forward` and `backward` already
      spell out, comparing `ShapeId`s by value so an identity from another diagram is a well-formed
      value that matches nothing here; rewrite `forward` and `backward` to search through it,
      changing no behavior and adding or modifying no test (data-model.md `find`; research.md Q3;
      plan.md commit 1; constitution principle V)
- [x] T003 Confirm `cargo test -p monospace-diagram` passes with no test added, changed or removed,
      and `cargo xtask check` is green — which is what makes T002 a structural commit rather than a
      behavioral one, and a green run on its own proves only that the command ran (constitution
      principle IV)

**Checkpoint**: one search instead of four, and the baseline captured. User story work can begin.

**Commit**: `refactor(diagram):` T002 (T001 and T003 are not commits — the first is a captured file
and the second is the gate run the hook makes anyway).

---

## Phase 3: User Story 1 - A figure is displaced by a delta (B3, Priority: P1) 🎯 MVP

**Goal**: `Delta` carries a signed amount per axis, `Shape::displaced_by` builds a new figure from
an old one and a delta, and no diagram, buffer or counter changes while it does. Moving a figure is
`get`, `displaced_by` and `replace` composed by the caller — the diagram gains no `move` verb, and
§9 still counts five changes (D3).

**Independent Test**: draw a box, displace it two cells right, and compare against the picture the
same box added at the displaced position produces; then the same for a connector whose expected
picture is built with both endpoints at `y + 2`, so an implementation that moved one endpoint or
neither fails rather than passing on a connector that never moved (spec.md B3; quickstart.md B3).

### Tests for User Story 1

> Write these first and watch them fail — before `Delta` exists the failure is that the method does
> not exist, which is the cheapest possible failure (quickstart.md B1).

- [x] T004 [P] [US1] Contract test: a box displaced two cells right draws exactly what the same box
      added at the displaced position draws, comparing buffers with the `cells` helper already in
      the module, in `crates/monospace-diagram/src/diagram.rs` (B3.1, SC-004)
- [x] T005 [P] [US1] Contract test: a connector displaced two cells down draws exactly what the same
      connector added with **both** endpoints at `y + 2` draws, so an implementation that displaces
      one endpoint or neither fails rather than passing on a connector that never moved, in
      `crates/monospace-diagram/src/diagram.rs` (B3.2, SC-004; the case where the previous system
      shipped a silent no-op and a `// TODO: implement it`, research.md Q1)
- [x] T006 [P] [US1] Contract test: displacing a figure a diagram holds changes no cell of it — draw
      before and after the call and compare with `cells` — in
      `crates/monospace-diagram/src/diagram.rs` (B3.3, first half: a displacement builds a value and
      changes nothing)
- [x] T007 [P] [US1] Unit test: a figure displaced by nothing at all comes back equal to itself, for
      each of the three kinds, which is what the widened derives are for, in
      `crates/monospace-diagram/src/shape.rs` (B3.4; research.md Q4)

### Implementation for User Story 1

- [x] T008 [US1] Widen `Endpoint`'s derives from `Clone, Debug` to `Clone, Debug, PartialEq, Eq` in
      `crates/monospace-diagram/src/shape.rs`; every field it holds — `Pos`, `Direction` and
      `Terminal` — already derives all four, and this is what lets T010 compile (research.md Q4
      lists the leaf field types and not this one)
- [x] T009 [US1] Add the new module `crates/monospace-diagram/src/delta.rs` holding
      `Delta { pub dx: i32, pub dy: i32 }` deriving `Debug, Clone, Copy, PartialEq, Eq` — `Pos`'s
      derives, so the two read as one family — plus `impl From<(i32, i32)> for Delta` as the second
      way in, and `pub(crate) fn apply(self, at: Pos) -> Pos` doing the only arithmetic in the crate
      with `saturating_add` per axis, returning `Self` rather than `Option<Self>`; rustdoc the type
      and both fields as they are introduced (`missing_docs` is warned and denied in the gate)
      (data-model.md `Delta` and `Delta::apply`; research.md Q1, Q2; contracts/diagram-api.md
      `Delta`)
- [x] T010 [US1] Widen `Shape`'s derives from `Debug` to `Debug, Clone, PartialEq, Eq` in
      `crates/monospace-diagram/src/shape.rs`; every leaf type already supports the three and T008
      has made `Endpoint` one of them, so this is a derive list and nothing else, and `Clone` is
      what lets `replace` take a `Shape` by value (data-model.md `Shape`'s derives; research.md Q4;
      depends on T008). In the same change, correct the two comments in
      `crates/monospace-diagram/src/gallery.rs` that say `Shape` has no `Clone` — the doc on the
      helper that builds a list of values, and the comment in `the_order_decides_a_shared_cell`
      explaining why the two boxes are rebuilt rather than cloned. Search for the two comments that
      mention `Clone`; the other claims in that file are about `Diagram` and belong to T016, which
      is a later commit
- [x] T011 [US1] Add `Shape::displaced_by(&self, by: Delta) -> Self` in
      `crates/monospace-diagram/src/shape.rs`, one arm per variant: `Box` and `Line` displace their
      own `at` and copy `size`/`stroke`/`fill` and `len`/`orientation`/`stroke` through, `Connector`
      displaces `from.at` **and** `to.at` together and copies both `leaving` and `terminal` plus
      `stroke`; mark it `#[must_use]` so `clippy::must_use_candidate` stays clean under the gate's
      `pedantic` group, and rustdoc it (B3.1-B3.3; data-model.md `displaced_by`;
      contracts/diagram-api.md `displaced_by`)
- [x] T012 [US1] Declare `mod delta;` beside the existing `mod diagram;` and `mod shape;`, and
      re-export `pub use delta::Delta;` beside the existing re-exports, in
      `crates/monospace-diagram/src/lib.rs` (plan.md Project Structure; contracts/diagram-api.md
      `Delta`)

**Checkpoint**: User Story 1 is fully functional and testable independently —
`cargo test -p monospace-diagram` passes, and a figure is displaced without anything changing.

**Commit**: `feat(diagram):` T008-T012, ticking T004-T007 with them, and the two `gallery.rs`
comments inside T010 with them (plan.md commit 2).

---

## Phase 4: User Story 2 - A caller can read a figure back by its identity (B4, Priority: P1)

**Goal**: `get` is the crate's first reader and its only one — one query by an identity the caller
already holds, borrowing, and nothing coming back the other way. What it returns is the bare figure
the caller added: the identity and the place in the order are the diagram's, held beside the figure
rather than inside it (B4.1).

**Independent Test**: add a figure, keep what `add` handed back, ask `get` for that identity, and
compare the two by value (spec.md B4; quickstart.md B4).

### Tests for User Story 2

- [x] T013 [P] [US2] Unit test: `get` on the identity `add` handed back returns a figure equal by
      value to the one added, which is what `Shape`'s and `Endpoint`'s widened derives are for, in
      `crates/monospace-diagram/src/diagram.rs` (B4.1; research.md Q4)
- [x] T014 [P] [US2] Unit test: `get` on an identity the diagram does not hold — one kept from
      another diagram — returns `None` and leaves the drawn buffer exactly as it was, with no error,
      no report and no panic, in `crates/monospace-diagram/src/diagram.rs` (B4.2, SC-003)

### Implementation for User Story 2

- [x] T015 [US2] Add `pub fn get(&self, id: &ShapeId) -> Option<&Shape>` to `Diagram` in
      `crates/monospace-diagram/src/diagram.rs`, borrowing through `find` and adding **no**
      `cloned()` beside it; mark it `#[must_use]` for the gate's `pedantic` group, and rustdoc it as
      contracts/diagram-api.md writes it (D2; B4.1-B4.3; data-model.md `get`;
      contracts/diagram-api.md `get`)
- [x] T016 [US2] Correct the **four** places in `crates/monospace-diagram/src/gallery.rs` that say a
      `Diagram` "offers no way to read them back" — the module doc, the sentence inside the `WHAT`
      description, the doc on `block`, and the ADR-0030 sentence beside it — so each says instead
      that a diagram offers one query by an identity and no listing of the shapes it holds, which is
      what D2 already answered. Find them by searching for `no way to read`, `privately` and
      `no reader`; the first draft of this task named three and the file holds four. The `WHAT` text
      is rendered into all three of that module's committed snapshots, so re-accept them with
      `cargo insta review` afterwards (`cargo-insta` is installed by hand, not by
      `cargo xtask setup`)

**Checkpoint**: a figure can be read back by the identity that names it, and nothing else about the
crate's surface has changed.

**Commit**: `feat(diagram):` T015, ticking T013 and T014 with it, and T016 with it as well — the
gallery is where the claim that T015 makes false is written, and a correction belongs in the commit
that got it wrong (constitution, Fixing a commit).

---

## Phase 5: User Story 3 - A shape can be taken out (B1, Priority: P1)

**Goal**: `remove` drops the entry an identity names, closes the gap, touches nothing else, cannot
fail, and hands back nothing.

**Independent Test**: draw a diagram of several figures, take one out by its identity, draw again,
and compare the two buffers — they differ, and each figure that stayed draws what it drew on its own
(spec.md B1; quickstart.md B1).

### Tests for User Story 3

- [x] T017 [P] [US3] Contract test: a diagram of several figures drawn before and after one is taken
      out produces different buffers, and each figure that stayed draws what it drew on its own, in
      `crates/monospace-diagram/src/diagram.rs` (B1.1, SC-001)
- [x] T018 [P] [US3] Contract test: taking out an identity the diagram does not hold — one kept from
      another diagram — leaves the buffer exactly as it was, with no error, no report and no panic,
      in `crates/monospace-diagram/src/diagram.rs` (B1.2, SC-003)
- [x] T019 [P] [US3] Unit test: take `#1` out, add a figure, and the identity handed back is `#3`
      rather than `#1`, asserted by the identity's own text rather than by a picture, in
      `crates/monospace-diagram/src/diagram.rs` (B1.3, SC-005 — the counter only rises, so this
      needs no code beyond the absence of a decrement, because `add` already increments before use)
- [x] T020 [P] [US3] Contract test: the last shape taken out leaves an empty diagram, and drawing
      one leaves the buffer as it was, in `crates/monospace-diagram/src/diagram.rs` (edge case: one
      shape taken out leaves an empty diagram)

### Implementation for User Story 3

- [x] T021 [US3] Add `pub fn remove(&mut self, id: &ShapeId)` to `Diagram` in
      `crates/monospace-diagram/src/diagram.rs`: `find` the entry, return having changed nothing
      when there is none, otherwise `Vec::remove` that index so the order of what stayed is the
      order it was — not a swap with the last, which would reorder everything behind the gap — and
      do not touch `next`; rustdoc it (D5, D6; B1.1-B1.3; data-model.md `remove`;
      contracts/diagram-api.md `remove`; research.md Q3)

**Checkpoint**: a shape can be taken out by its identity, observed by drawing, and an identity is
never reissued.

**Commit**: with T015 and T027 — plan.md commit 3 is one `feat(diagram)` for `get`, `remove` and
`replace` together, ticking T013, T014, T017, T018, T019, T020, T022, T023, T024, T025 and T026
alongside. The three phases are for traceability, not a license to make three commits.

---

## Phase 6: User Story 4 - A different shape can be put under an identity (B2, Priority: P1)

**Goal**: `replace` overwrites the entry an identity names. The identity and the place in the order
survive, and everything the figure owned — its kind, its parameters and its position — is the new
figure's. The diagram never reaches into a box and widens it; it takes a wider box and puts it where
the old one was.

**Independent Test**: hold a box, put a wider box under its identity, and compare against the
picture that wider box added on its own produces; then put a line under the same identity and
compare against the line's (spec.md B2; quickstart.md B2).

### Tests for User Story 4

- [x] T022 [P] [US4] Contract test: a box put back as a **wider box** draws exactly what that wider
      box added on its own produces — pinned against the wider box's own picture, not against the
      box it replaced — in `crates/monospace-diagram/src/diagram.rs` (B2.1, SC-002)
- [x] T023 [P] [US4] Contract test: a box put back as a **line** draws the line, kind included, and
      nothing of the previous figure survives — again pinned against the line's own picture, which
      is what turns "nothing survives" into a checked claim rather than a comparison of two
      pictures, in `crates/monospace-diagram/src/diagram.rs` (B2.2, SC-002; the pair spec.md draws
      by hand and calls hypothetical)
- [x] T024 [P] [US4] Contract test: a figure overlapping another, put back under its own identity
      **unchanged**, resolves the overlap as it did, which is what shows a replacement is not a
      reorder — the order did not move — in `crates/monospace-diagram/src/diagram.rs` (B2.3)
- [x] T025 [P] [US4] Contract test: a shape put under an identity the diagram does not hold leaves
      the picture alone **and adds nothing** — a diagram that held two figures still holds two, and
      none of them is the one handed in, so there is no way to name a shape into existence, in
      `crates/monospace-diagram/src/diagram.rs` (B2.4, SC-003)
- [x] T026 [P] [US4] Contract test: draw, displace, `replace` — and the only cells differing from
      the drawing before are the ones the displaced figure used to hold; this is the second half of
      B3.3, and it is the half that needs `replace` to exist, so it lands here rather than in Phase
      3, in `crates/monospace-diagram/src/diagram.rs` (B3.3; quickstart.md B3)

### Implementation for User Story 4

- [x] T027 [US4] Add `pub fn replace(&mut self, id: &ShapeId, shape: Shape)` to `Diagram` in
      `crates/monospace-diagram/src/diagram.rs`: `find` the entry, return having changed nothing
      when there is none so the shape handed in is not added either, otherwise overwrite that
      entry's `.shape` and touch nothing else — overwriting rather than removing one entry and
      adding another is what keeps the identity and the place in the order for free; add **no**
      guard on a change of kind, because the model never asked for one; rustdoc it (D4; B2.1-B2.4;
      data-model.md `replace`; contracts/diagram-api.md `replace`)

**Checkpoint**: a different figure can be put under an identity and drawn, and the identity and the
place in the order are the only things that survive either way.

**Commit**: with T015 and T021 — plan.md commit 3.

---

## Phase 7: User Story 5 - The shipped demonstration shows a removal beside a displacement and a

reorder (B5, Priority: P2)

**Goal**: a bare run prints four captioned pictures about one figure — it moved, it was displaced,
and then it is gone — and a run given a path still prints one picture and nothing else, because that
is what `cargo xtask render` embeds in a document.

**Independent Test**: `cargo run -p monospace-cli` with no arguments prints four captioned pictures;
`cargo run -p monospace-cli crates/monospace-cli/assets/demo.json` prints one picture and nothing
else, byte for byte what it printed before (spec.md B5; quickstart.md B5).

### Tests for User Story 5

- [ ] T028 [P] [US5] Contract test: a bare run prints four captioned pictures, the first the
      description as written, found by splitting the output on the blank line and pinning no
      caption's wording, in `crates/monospace-cli/src/main.rs` (B5.1, SC-006)
- [ ] T029 [P] [US5] Contract test: the third picture differs from the second only in the displaced
      figure's own cells, and the fourth differs from the third only in the cells that figure
      occupied — each now the cell the figure behind it decides, or empty where no figure decides
      one, so a removal leaves a gap rather than a hole punched in what was around it, in
      `crates/monospace-cli/src/main.rs` (B5.3, B5.5, SC-006)
- [ ] T030 [P] [US5] Contract test: a description holding no shapes, and one holding exactly one,
      each print four identical pictures and the run succeeds, in `crates/monospace-cli/src/main.rs`
      (B5.7 — `forward`, `replace` and `remove` are all no-ops on an identity the diagram does not
      hold, and there is no branch to get wrong)
- [ ] T031 [P] [US5] Subprocess test: the existing
      `the_demo_path_passed_explicitly_prints_the_demonstrations_first_picture` in
      `crates/monospace-cli/tests/cli.rs` needs **no** change to its body, and that is worth
      confirming rather than assuming — it takes the first block and the new pictures are appended
      below rather than inserted, so the coordinates `char_at` reads at `y + 1` do not move; run it
      and record that it still passes (B5.6, SC-007; research.md Q6)

### Implementation for User Story 5

- [ ] T032 [US5] Change `demonstrated_pictures` in `crates/monospace-cli/src/main.rs` from a
      two-tuple to a four-tuple, splitting the output three times on `\n\n`, and update its four
      destructuring call sites so `rendering_once_is_the_demonstrations_first_picture` and
      `two_overlapping_boxes_demonstrate_in_opposite_orders` keep comparing the first two pictures
      with their meaning unchanged (data-model.md "The tests that read the demonstration";
      research.md Q6)
- [ ] T033 [US5] Grow `demonstrate` in `crates/monospace-cli/src/main.rs` to four captioned pictures
      over **one** diagram mutated in place: as written, then after `forward(&ShapeId::new("#1"))`,
      then after `get(&ShapeId::new("#1")).map(|shape| shape.displaced_by(by))` followed by
      `replace(&ShapeId::new("#1"), moved)`, then after `remove(&ShapeId::new("#1"))`; append the
      fourth after the pair rather than interleaving it, so all four are about one figure, and name
      the identity by hand at each call rather than reading the figure back (B5.1-B5.4, B5.9;
      research.md Q6 — the demonstration is not the caller B4's reader is for)
- [ ] T034 [US5] Carry the third picture's displacement amount as a constant in `demonstrate` in
      `crates/monospace-cli/src/main.rs`, beside a comment saying it is an assumption about the
      demonstration rather than a rule about descriptions, the treatment the reorder's already gets;
      add no field to the description format, add no command-line argument, and leave
      `crates/monospace-cli/assets/demo.json` and `crates/monospace-cli/src/description.rs`
      unchanged (B5.8, B5.9; research.md Q6; ADR-0035)
- [ ] T035 [US5] Correct the comment over `first_demonstrated_picture` in
      `crates/monospace-cli/tests/cli.rs`, which says "the first of the two captioned pictures", in
      the same commit that makes it wrong, and change nothing else in the file (data-model.md "The
      tests that read the demonstration"; quickstart.md "Build and test the workspace")

**Checkpoint**: all five behaviors are independently functional — `cargo test -p monospace-cli`
passes, and a bare run tells a reorder, a displacement and a removal from one another.

**Commit**: `feat(cli):` T033, T034 and T035, ticking T028-T031 with it (plan.md commit 4).

---

## Phase 8: Polish & Cross-Cutting Concerns

- [ ] T036 Run `cargo run -p monospace-cli > /tmp/demo-after.txt` and
      `diff /tmp/demo-before.txt /tmp/demo-after.txt` — the diff is three appended
      caption-and-picture blocks and nothing else; then
      `diff /tmp/file-before.txt /tmp/file-after.txt`, which must print nothing at all, since a
      file's picture is exactly the one it was (B5.6, B5.8, SC-007; quickstart.md)
- [ ] T037 Read the four pictures from that run and confirm they are about one figure: the second
      differs from the first only in which of two overlapping figures wins their shared cells, the
      third only in where that same figure sits, and the fourth does not hold it at all — so a
      person who runs it tells a reorder, a displacement and a removal from one another without
      reading a test (B5.2-B5.5, SC-006)
- [ ] T038 Run the two acceptance scenarios from quickstart.md: a description whose `shapes` is
      empty, and one holding a single shape, each printing four identical pictures and exiting
      successfully (B5.7)
- [ ] T039 Confirm B4.3 still holds by reading `Diagram`'s signatures: there is no `ids()`, no
      `len`, no order, no way to ask which shape decided a position, and no `cloned()` beside `get`.
      It is a rule about what is absent, so the only way to keep it true is for nothing to add it
      (B4.3; contracts/diagram-api.md "What is still not here"; issue #86 is the other direction)
- [ ] T040 Confirm the two rules the spec accepts with nothing to verify them are **named as such**
      rather than described as tested: removing a shape leaves every reference to it unresolved, and
      displacing a reference reaches its offsets. No figure can hold a reference yet, so neither is
      reachable from a test, and issue #82 is where both can be asserted. Look for them in
      `docs/diagram-model.md` and leave them as they are — this task is a check that the record says
      so, not an edit (constitution principle IV; D6; spec.md Testing expectations)
- [ ] T041 Run `cargo xtask check` and confirm it is green, including the `wasm` step — which
      already names `monospace-diagram`, so it covers `Delta` and the three new methods with no
      change to `xtask` and no new check (SC-008; plan.md Constitution Check, principle III)
- [ ] T042 Append an entry to `docs/learning-log.md` for this increment: what was learned about Rust
      design and idiom, what was learned about working this way, and optionally a trade-off worth
      remembering (constitution principle II)

**Commit**: `docs:` T042. T036-T041 are observations and the gate run, not commits.

---

## What this slice must not add

Each of these is a decision already answered on the sheet or a rule the model states. Reaching past
one means stopping and asking, not deciding and recording afterwards (constitution, No decision
outside the sheet).

- **No item in `monospace-core`**: no `Delta`, no arithmetic, no `displaced` on the core's trait.
  The core's shapes are constructed, drawn and discarded, so nothing in it consumes a displacement,
  and a displacement there is rewritten in issue #82 when an endpoint becomes a `Position` (D1,
  principle VII).
- **No `move` verb on `Diagram`**. Moving a figure is `get`, `displaced_by` and `replace` composed
  by the caller, and §9 still counts five changes (D3).
- **No `ids()`, no `len`, no order, no way to ask which shape decided a position, and no `cloned()`
  beside `get`** (D2, B4.3; issue #86).
- **No `Position`, `Anchor`, `Reference` or offset**, and nothing that says a removal leaves
  references frozen. #81 decides nothing about references (D6; issue #82).
- **No undo, no history, and no value handed back** by `remove` (D5).
- **No new dependency, no new crate, no change to the description format, and no new command-line
  argument** (D1, B5.8; ADR-0035).
- **No `docs/model.md` amendment** — the core does not change, so the four amendments were to
  `docs/diagram-model.md` alone, and they have landed (D7).
- **No characterization test**. Every rule here is about a picture or a value, and a range too wide
  to read by hand is not what this slice produces (spec.md Testing expectations).

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: nothing to do.
- **Foundational (Phase 2)**: no dependencies. The baseline is captured before any edit, and the
  structural `find` lands on its own before anything depends on it.
- **User Story 1 (Phase 3)**: depends on Phase 2. No dependency on another story.
- **User Stories 2, 3 and 4 (Phases 4, 5 and 6)**: depend on Phase 2, and each depends on `find`
  from Phase 2. They do **not** depend on Phase 3, and not on one another — but plan.md commit 3
  delivers them together, and splitting that commit would put a behavioral change where the design
  asks for one.
- **User Story 5 (Phase 7)**: depends on Phases 4, 5 and 6 — the demonstration calls `get`,
  `replace` and `remove`, so all three must exist.
- **Polish (Phase 8)**: depends on Phase 7.

### Within Each User Story

- US1: T008 before T010, because `Shape` cannot derive `PartialEq` while `Endpoint` does not; T009
  before T011, because `displaced_by` takes a `Delta`; T011 before T012, because the re-export names
  a module that must compile. The two `gallery.rs` comments inside T010 land with T010 and need
  nothing else. Tests (T004-T007) fail first and pass with the implementation.
- US2: T015 before T016, because the gallery is corrected by the reader landing.
- US3: T021, then T017-T020. T019 and T020 are the two assertions about what `remove` does _not_
  touch — the counter and the buffer.
- US4: T027, then T022-T026. T026 needs both T027 and T011's `displaced_by`.
- US5: T033 before T029 and T030, which read what it produces; T032 alongside, since the tests read
  the four pictures through the helper it changes; T035 any time after T033.

### Parallel Opportunities

- T004-T007 (US1 tests) in parallel once T011 lands — all four read `displaced_by` and none touches
  another's file.
- T013-T014 (US2 tests) in parallel once T015 lands.
- T017-T020 (US3 tests) in parallel once T021 lands.
- T022-T026 (US4 tests) in parallel once T027 and T011 land.
- T028-T031 (US5 tests) in parallel once T032 and T033 land.
- T035 is independent of T033's body and can be written at any point in Phase 7.
- T036-T041 in the Polish phase are observations and can be run in any order.

Nineteen tasks carry `[P]`. The four stories inside plan.md commit 3 cannot be worked in parallel in
practice, because one commit carries them.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 2 — one search instead of four, baseline captured.
2. Phase 3 — `Delta` and `displaced_by` exist, and a figure can be displaced without anything
   changing.
3. **STOP and VALIDATE**: `cargo test -p monospace-diagram` passes; a box and a connector each draw
   where the same figure added there would draw, and a connector that moved one endpoint fails.

Note that the MVP here is not the slice's headline: `Delta` and `displaced_by` are what `replace`
and the demonstration are built on, and displacing a figure without changing a diagram is only half
the promise. US1 alone is demonstrable, which is what makes it a checkpoint rather than the
deliverable.

### Incremental Delivery

1. Phase 2 → one search, baseline captured.
2. Phase 3 → a figure is displaced (MVP).
3. Phases 4-6, one commit → a caller can read a figure back, take one out, and put a different one
   in its place, observed by drawing.
4. Phase 7 → the shipped CLI demonstrates all three changes about one figure.
5. Phase 8 → the gate is green, the two untestable rules are named as such, the increment is closed.

Each commit follows plan.md: one `refactor` (T002) with no behavior change and no test, everything
else `feat` or `docs`, and every commit leaves `cargo xtask check` green (constitution principles II
and V).
