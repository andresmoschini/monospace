---
description: "Task list for feature 082: a connector endpoint hangs from a box's side anchor"
---

# Tasks: A connector endpoint hangs from a box's side anchor

**Input**: Design documents from `/specs/082-a-connector-endpoint-hangs-from-a-box-s/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [data-model.md](data-model.md),
[contracts/diagram-api.md](contracts/diagram-api.md), [research.md](research.md),
[quickstart.md](quickstart.md)

**Tests**: Requested by the spec's **Testing expectations** — contract tests for every rule
observable as a picture, unit tests for the four side centers, and the rule that "a behavior rule
with no test named against it is an unfinished spec" (constitution, Testing). The one rule the spec
accepts with nothing to verify it is named in the Polish phase, not here (T041).

**Organization**: Tasks are grouped by the spec's behaviors B1 to B5, in the order
[plan.md](plan.md)'s seven commits deliver them — that order is forced by constitution principles II
and V rather than chosen, so it is the priority order here too. Phases 3 to 6 are four behaviors in
**one** commit, and the split is for traceability, not a license to make four.

**Order within a phase**: implementation before tests, where a test cannot exist before what it
reads. That inverts the usual listing, and it is deliberate: in Rust a test that names a method
which is not there does not fail, it does not compile. Task IDs are in execution order throughout.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different file, no dependency on an incomplete task)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, US5)
- Include exact file paths in descriptions

## Path Conventions

Rust cargo workspace. `crates/monospace-diagram/src/` for the diagram library, and
`crates/monospace-cli/src/` and `crates/monospace-cli/tests/` for the CLI. The unit tests for the
diagram crate live in the `#[cfg(test)] mod tests` of the file they cover, as the ones already there
do — the crate has no `tests/` directory and this slice adds no integration test to it.

### Two things the design had to be corrected into, and where they live now

`plan.md` was wrong twice about the structural commit, and both are fixed in commits `9170fb2` and
`f1bd9b7`. The corrected shape is what these tasks follow:

- **`Endpoint.at` becoming a `Position` and the rule that replaces
  `From<Endpoint> for monospace_core::Endpoint` are one indivisible change**, so both are in the
  `feat(diagram)` commit and neither is in a `refactor`. Any rule for getting a `Pos` out of a
  `Position` is behavior.
- **The three types have a `refactor` of their own**: `position.rs` lands holding nothing, declared
  and re-exported, which is expand/contract's first half. A public re-exported type is never dead
  code, so the gate has nothing to deny there.

There is therefore **no departure from principle V to record**, and the row `plan.md`'s Complexity
Tracking carried for one was removed rather than carried. No structural commit touches a test.

---

## Phase 1: Setup

No setup is needed: no dependency is added, no crate is added, the toolchain is unchanged, and no
gate step is added — `wasm` already names `monospace-diagram` (plan.md Constitution Check, principle
III; D1).

---

## Phase 2: Foundational — the three types, and a gallery that starts running

**Purpose**: the two commits that change no behavior at all, and that every later commit reads.
Nothing here may touch a test, and nothing here may change a picture.

- [x] T001 Run `cargo run -p monospace-cli > /tmp/demo-before.txt` and
      `cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-before.txt`, and
      keep both files until Phase 9. Then run `cargo run -p monospace-cli | sed -n '4,6p'` and read
      off the two numbers the fifth picture rests on: the third entry of
      `crates/monospace-cli/assets/demo.json` is a four-by-three box at `{9, 2}` whose right side
      center is `{12, 3}`, which is where the connector's `from` already sits at line 68, and
      displacing it four cells right puts its footprint at columns 13 to 16 where the only thing the
      picture writes is that same arm. Write both down; they are what a reviewer asks about
      otherwise (quickstart.md "Before touching anything"; data-model.md "The arithmetic";
      research.md Q4)
- [x] T002 [P] Create `crates/monospace-diagram/src/position.rs` holding three public types and
      nothing else: `Anchor` with `Top`, `Right`, `Bottom` and `Left` deriving
      `Debug, Clone, Copy, PartialEq, Eq`; `Reference { pub id: ShapeId, pub anchor: Anchor }`
      deriving `Debug, Clone, PartialEq, Eq` and **not** `Copy`, since `ShapeId` is a `String`; and
      `Position { Absolute(Pos), Reference(Reference) }` with the same four derives, plus
      `impl From<Pos> for Position` as the second way in. Rustdoc all three types and both of
      `Reference`'s fields as they are introduced (`missing_docs` is warned and denied in the gate).
      Add **no** `resolve` and **no** `displaced_by` in this commit — both are behavior, T014 and
      T025 own them, and an unused `pub(crate)` method is dead code the gate denies (data-model.md
      `Anchor`, `Reference` and `Position`; research.md Q1; D1, D2)
- [x] T003 Declare `mod position;` beside the existing `mod delta;`, `mod diagram;` and `mod shape;`
      in `crates/monospace-diagram/src/lib.rs`, and re-export
      `pub use position::{Anchor, Position, Reference};` beside the existing re-exports (depends on
      T002; contracts/diagram-api.md)
- [x] T004 Confirm the workspace still builds, `cargo test -p monospace-diagram` still reports 37
      tests, and `cargo xtask check` is green — and diff the demonstration's output against
      `/tmp/demo-before.txt` to confirm no picture changed. This is what makes T002-T003 a
      structural commit rather than a behavioral one, and a green run on its own proves only that
      the command ran (constitution principles IV and V)
- [x] T005 [P] Add `#[cfg(test)] mod gallery;` to `crates/monospace-diagram/src/lib.rs`, and confirm
      `cargo test -p monospace-diagram` now reports 40 tests, all green against the three committed
      snapshots **without** a `cargo insta review` — if `review` offers one, something else changed
      and this task is in the wrong commit. The module's own doc claims `kind_of` is the
      completeness guarantee, and this is what starts enforcing it (research.md Q5)
- [x] T006 [P] In `crates/monospace-diagram/src/gallery.rs`, rename `block`'s second line from
      `order:` to `change:` and say so in the `WHAT` description beside it — the line is what was
      done between two drawings of one diagram, which 080 could only make a change to the order of,
      and this slice's block displaces a figure. `WHAT` is a `concat!` of seven string fragments, so
      re-accept the three snapshots with `cargo insta review` (`cargo-insta` is installed by hand,
      not by `cargo xtask setup`) and then read the rendered sentence with `git diff --word-diff`: a
      hand re-wrap of a `concat!` drops a word silently, the code compiles and the snapshots match,
      and that diff is the only place it shows (data-model.md "The gallery")
- [x] T007 Confirm `cargo test --workspace` is green and `cargo xtask check` is green after T005 and
      T006. Phase 2 ends with every snapshot this slice touches accounted for

**Checkpoint**: three public types exist and nothing uses them, and a gallery that had never run now
does. User story work can begin.

**Commit**: `refactor(diagram):` T002 and T003, then `test(diagram):` T005 and T006 (plan.md commits
1 and 2). T001, T004 and T007 are a captured file and two gate runs, not commits.

---

## Phase 3: User Story 1 - A box and a line answer the four centers of their sides (B1, Priority

P1) 🎯 MVP

**Goal**: each kind answers the four side centers or none of them, and `None` is an ordinary answer
rather than a failure. A box answers all four from its own position and size, a line answers them as
a flat box, and a connector answers none — which is what keeps a chain of references one link long.

**Independent Test**: ask a four-by-three box at the origin for each of the four, and compare with
the absolute points — the right one is `{3, 1}`, which is the endpoint
[§6 _Attachment_](../../../docs/diagram-model.md#6-attachment) already shows standing on that
border; then ask the same of a line and of a connector, and get the flat box's answers and four
`None`s (spec.md B1; quickstart.md B1).

### Implementation for User Story 1

- [x] T008 [US1] Add `pub(crate) fn anchor(&self, anchor: Anchor) -> Option<Pos>` to `Shape` in
      `crates/monospace-diagram/src/shape.rs`, one arm per variant: `Box` all four from its own `at`
      and `size`, `Line` all four as a flat box, `Connector` `None` for all four. Beside it in
      `crates/monospace-diagram/src/position.rs`, add
      `pub(crate) fn side_centre(at: Pos, size: Size,     anchor: Anchor) -> Pos` and
      `pub(crate) fn flat_size(len: u32, orientation: Orientation) ->     Size` — the first is the
      middle of one side of a figure occupying `at` through `at + (size - 1)`, the second is
      `(len, 1)` horizontal and `(1, len)` vertical. Use `saturating_sub` on the extent and
      `saturating_add` on each axis, so a box one cell wide falls out of the general rule, and a
      floor division so a four-cell box has its top and bottom centers on cells 1 and 2 rather than
      on a half-cell. Rustdoc all three, and keep the signature crate-private: nothing outside the
      crate asks a figure for an anchor, and the direct query is the specification's first consumer
      that needs one without drawing (B1.1-B1.3; data-model.md "The arithmetic";
      contracts/diagram-api.md "`Shape::anchor` is not here")

### Tests for User Story 1

- [x] T009 [P] [US1] Contract test: a box's four side centers, each compared with the absolute point
      the picture depends on rather than with the other three, in
      `crates/monospace-diagram/src/position.rs` — the anchors have no consumer outside this file's
      own tests, and that is why the query is `pub(crate)` (B1.1; research.md Q3)
- [x] T010 [P] [US1] Contract test: a box one cell wide and a box one cell tall, each asked for all
      four, in `crates/monospace-diagram/src/position.rs` (spec.md edge case: the four side centers
      coincide in pairs **by the general rule rather than by an exception** — a test that only asks
      a 4×3 would pass on an implementation that special-cases it)
- [x] T011 [P] [US1] Contract test: a line's four, each equal to the same point read as a flat box —
      a horizontal line answers its top and bottom centers with its middle, one point asked twice,
      and its left and right centers with its two ends — and a vertical line the same seen sideways.
      Both orientations, because one of them is the one an implementation gets wrong by
      transposition, in `crates/monospace-diagram/src/position.rs` (B1.2)
- [x] T012 [P] [US1] Contract test: a connector's four, each asked for and answered `None`, with no
      error, no report and no panic, in `crates/monospace-diagram/src/position.rs` (B1.3)

**Checkpoint**: every kind answers the four side centers or none, asked and compared by value rather
than by picture.

---

## Phase 4: User Story 2 - An endpoint hangs from a side, and comes with it (B2, Priority: P1)

**Goal**: an endpoint's position may be a reference, a reference resolves to a point, and a
connector whose endpoint does not resolve is not drawn at all. Moving the referenced shape moves
everything hanging from it, which is the whole point of having references.

**Independent Test**: a connector whose `from` is a reference to a box's right side, compared
against the same connector with `from` at the absolute point that reference resolves to — each side
pinned against the point rather than against the other, so a reference resolving to the wrong place
fails rather than passing on a pair that are wrong together. Then displace the box four cells right
and compare again (spec.md B2; quickstart.md B2).

### Implementation for User Story 2

- [x] T013 [US2] In `crates/monospace-diagram/src/shape.rs`: change `Endpoint.at` from `Pos` to
      `Position`; delete `impl From<Endpoint> for monospace_core::Endpoint` and its rustdoc; change
      `Shape::draw` to `pub(crate) fn draw(&self, surface: &mut impl Surface, diagram: &Diagram)`;
      and rewrite its `Connector` arm to resolve both endpoints **before writing anything** —
      `let (Some(from_at), Some(to_at)) = (from.at.resolve(diagram), to.at.resolve(diagram)) else     { return; };`
      — then build the core's `Connector` by hand from the two resolved points, the way every other
      arm already builds its core shape. The `Box` and `Line` arms ignore the argument, which is
      ADR-0041's restriction as a signature rather than a rule enforced at run time. Ask both
      endpoints in one `let` so one unresolvable end is never drawn as an arm with a missing head
      (D2; B2.4; data-model.md `Shape::draw`; research.md Q2). In the same change, wrap the twelve
      `Endpoint` literals with `at: Pos { … }.into()` in `shape.rs`, `diagram.rs` and `gallery.rs` —
      **counted, not estimated**: five in `shape.rs` less the one inside the deleted test, four in
      `diagram.rs`, four in `gallery.rs` — and change `crates/monospace-cli/src/description.rs` to
      `at: Position::Absolute(endpoint.at.into())`, which is the one line outside the diagram crate
      the type change forces. That `.into()` is there today and resolves `description::Pos` into the
      core's `Pos`; the explicit variant is what makes it resolve into a `Position`, and
      `From<Pos> for Position` does not reach that file (data-model.md "`description.rs`")
- [x] T014 [P] [US2] Add `#[must_use] pub fn resolve(&self, diagram: &Diagram) -> Option<Pos>` to
      `Position` in `crates/monospace-diagram/src/position.rs`, in three steps: `Absolute(at)` is
      `Some(*at)`; `Reference(reference)` asks `diagram.get(&reference.id)` for the figure and takes
      `None` when the diagram holds no such shape; then asks that figure for the anchor and takes
      `None` again when the kind does not answer it. Public, because a caller that knows a
      `Position` and its `Diagram` can ask where it stands; `#[must_use]`, because it answers and
      changes nothing. **Add no offset arithmetic** — D1 says there are no offsets until #83, and
      adding two zeros is an entry the removal test takes out. Rustdoc the missing third field as
      arriving with #83 so the absence reads as a step not yet taken (D5, D1; B2.4, B3.1;
      contracts/diagram-api.md `Position`; research.md Q3; depends on T012)

### Tests for User Story 2

- [x] T015 [P] [US2] Contract test: a reference draws exactly what the absolute point draws, both
      sides pinned against the point and not against each other, in
      `crates/monospace-diagram/src/diagram.rs` using the `cells` and `drawn` helpers already there
      (B2.1, SC-001)
- [x] T016 [P] [US2] Contract test: the referenced box displaced four cells right draws the picture
      where the same box added at the displaced position would draw, and the route re-routes to the
      endpoint that did not move — the reach pinned by coordinates with `differing`, so a resolution
      cached at construction rather than asked at drawing fails, in
      `crates/monospace-diagram/src/diagram.rs` (B2.2, SC-002, SC-005)
- [x] T017 [P] [US2] Contract test: a connector with one endpoint absolute and one a reference, each
      placed by its own rule, in `crates/monospace-diagram/src/diagram.rs` (B2.3)
- [x] T018 [P] [US2] Contract test: the three non-resolutions ADR-0041 enumerates — a reference to
      an identity the diagram does not hold, a reference to an anchor a kind does not answer, and a
      connector with one endpoint that resolves and one that does not — each asserting the connector
      is **absent from the output** and that every other shape's cells are **unchanged**, in
      `crates/monospace-diagram/src/diagram.rs` (B2.4, SC-003; the second half is what distinguishes
      "the connector is not drawn" from "the drawing stopped", and a connector drawn partly passes
      the first)
- [x] T019 [P] [US2] Contract test: a reference to an identity nothing holds yet, then a shape added
      under it — a **box** and the connector draws hanging from its side; a **line** and it lands on
      the line's own end or its middle; a **connector** and it stops drawing, which is the same
      answer as a reference to a shape that was never there — in
      `crates/monospace-diagram/src/diagram.rs` (B2.5; spec.md edge case)
- [x] T020 [P] [US2] Contract test replacing `the_mirrors_terminal_reaches_the_cores_intact` deleted
      with the conversion in T013: a connector with a glyph terminal and one with an arm, each drawn
      through a diagram, produce the buffer the same core `Connector` drawn directly produces, in
      `crates/monospace-diagram/src/diagram.rs`. A stronger pin than the conversion was, because it
      goes through the four hand-built lines rather than naming them

- [x] T021 [US2] Add the gallery's fourth block to `crates/monospace-diagram/src/gallery.rs`: a
      small box with no fill, a connector whose `from` is a reference to that box's right side, and
      the box displaced four cells right — two blocks in one snapshot, the before and the after,
      which is the shape `the_order_decides_a_shared_cell` already has. The `shapes:` line is
      written by hand because a `Diagram` has no listing, exactly as the two existing blocks' are.
      Re-accept with `cargo insta review` and **read** the snapshot: the first block must be §6's
      picture, which is what makes B2.1 visible rather than asserted, and the second must differ
      from it in the box's cells and the arrow's route and nowhere else (B2.1, B2.2; research.md Q4;
      ADR-0064 — a `<!-- render: -->` marker cannot reach this, so the gallery is the only carrier
      in the crate that can)

**Checkpoint**: an endpoint hangs from a side, follows it when it moves, and takes its whole
connector out of the picture when it cannot be placed.

---

## Phase 5: User Story 3 - Taking a referenced shape out leaves the reference alone (B3, Priority

P1)

**Goal**: the model's general answer, made observable for the first time. The reference is left
exactly as it was, it does not resolve, the shape that held it is not drawn, and nothing is
rewritten, reported or panicked on.

**Independent Test**: a connector hanging from a box plus a second box that has nothing to do with
either — draw, take the first box out, draw again. The connector draws nothing, the second box draws
exactly what it drew, and putting a box back under that identity makes the connector draw again
(spec.md B3; quickstart.md B3).

### Tests for User Story 3

- [x] T022 [P] [US3] Contract test: the referenced box taken out leaves the connector drawing
      nothing and the unrelated box drawing exactly what it drew **on its own** — compared cell by
      cell against the buffer that box produces alone, not against the first picture — in
      `crates/monospace-diagram/src/diagram.rs` (B3.1, SC-004)
- [x] T023 [P] [US3] Contract test: a box put back under the removed one's identity makes the
      connector draw again, hanging from wherever the **new** box is rather than where the old one
      was, in `crates/monospace-diagram/src/diagram.rs` (B3.2, SC-004)

### Implementation for User Story 3

- [x] T024 [US3] Add **no** code for this behavior, and say so in the commit rather than leaving it
      to be found. `remove` is 081's, `Diagram::remove`'s rustdoc already says an identity this
      diagram does not hold "changes nothing, which is the same answer the model's _Positions_ gives
      a reference to a shape that is not there", and what we would rather it did is
      [#142](https://github.com/andresmoschini/monospace/issues/142)'s to answer. Touching `remove`
      to make a difference here would be deciding a question the spec explicitly hands to an issue
      (spec.md Clarifications, 2026-09-28; research.md Q8; constitution, No decision outside the
      sheet)

**Checkpoint**: the general answer is observable and the reference is untouched.

---

## Phase 6: User Story 4 - Displacing a figure that hangs from another leaves the hanging end where

it is (B4, Priority: P1)

**Goal**: a displacement adds coordinates to an absolute position and a reference has none to add to
yet, so a connector with one endpoint of each kind is displaced through its absolute end alone. The
difference is in the picture rather than in a promise.

**Independent Test**: a connector with one endpoint absolute and one a reference, displaced two
cells down — the absolute endpoint moves two cells down, the referenced one does not move at all,
and the route is drawn between the two (spec.md B4; quickstart.md B4).

### Implementation for User Story 4

- [x] T025 [US4] Add `pub(crate) fn displaced_by(&self, by: Delta) -> Self` to `Position` in
      `crates/monospace-diagram/src/position.rs`: `Absolute(at)` becomes `Absolute(by.apply(*at))`
      and `reference @ Reference(_)` comes back cloned unchanged, with a comment naming #143 as what
      replaces the second arm. In `crates/monospace-diagram/src/shape.rs`, change
      `Shape::displaced_by`'s `Connector` arm from `by.apply(from.at)` and `by.apply(to.at)` to
      `from.at.displaced_by(by)` and `to.at.displaced_by(by)`. The `Box` and `Line` arms are
      unchanged, because their `at` is still a `Pos` and `by.apply` is still the crate's only
      arithmetic (B4.1; data-model.md "`displaced_by` — a reference does not move")

### Tests for User Story 4

- [x] T026 [P] [US4] Contract test: a connector with one endpoint absolute and one a reference,
      displaced two cells down, draws the picture the same connector added with the absolute
      endpoint at `y + 2` and the referenced one **where it already was** draws. The expected
      picture is built from the two positions rather than pinned as text, so an implementation that
      moved both or neither fails, in `crates/monospace-diagram/src/diagram.rs` (B4.2, SC-005; a
      displacement that did nothing passes on a connector that never hung from anything)
- [x] T027 [P] [US4] Contract test: a box and a line displaced take all four of their side centers
      with them, and a connector hanging from one goes with them, in
      `crates/monospace-diagram/src/diagram.rs` (B4.3; the other side of B4 from the rule above, and
      what makes a displacement a property of a position rather than of a figure)
- [x] T028 [US4] Rewrite the last paragraph of `Shape::displaced_by`'s rustdoc in
      `crates/monospace-diagram/src/shape.rs`, which says a figure cannot hold a reference "yet" and
      that "the issue that introduces one settles it" — it is wrong the moment this slice lands, so
      it is corrected in the commit that makes it wrong, and it now says what the second arm of
      `Position::displaced_by` does and which issue carries the real rule (research.md Q7;
      constitution, Fixing a commit)

**Checkpoint**: the hanging end stands still and the free end moves, and both facts are pictures.

**Commit**: Phases 3, 4, 5 and 6 together as one `feat(diagram):` — T009-T012, T013-T021, T022-T024,
T025-T028, ticking every test above with them (plan.md commit 3). The four phases are for
traceability, not a license to make four commits: the field's new type, the conversion's death and
the rule that replaces them cannot be separated without a commit that does not build.

---

## Phase 7: Records this slice makes stale

**Purpose**: two `docs` commits, neither of them a behavior, and so neither carrying a `[Story]`
label. They are placed here rather than in the Polish phase because plan.md orders them between the
diagram feature and the demonstration, and the first of them is due the moment the feature lands.

- [x] T029 [P] Correct the **three** places in
      `specs/081-a-shape-can-be-removed-and-replaced/contracts/diagram-api.md` that say a figure
      cannot hold a reference — the `displaced_by` bullet's closing lines, the `Reference`-free
      "What is still not here" bullet, and the closing sentence of that section — so each says
      instead what is true now: an endpoint's position may be a reference, and this slice is where
      it does. A contract describes what a caller sees, and leaving the one artifact a caller reads
      describing a crate that cannot do what this slice gives it is the failure; the other four
      records research.md Q7 names — 081's `data-model.md`, `quickstart.md`, `tasks.md` and
      `checklists/requirements.md` — are a merged spec rather than a contract and stand, as does
      `specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/diagram-api.md` (research.md Q7;
      `4a851ac`'s rule as research.md Q7 applies it; constitution, Fixing a commit)
- [x] T030 [P] Add one sentence to §3 _Identity_ in `docs/diagram-model.md` saying a caller may
      spell an identity while the diagram still issues them, and amend **nothing else**: §4 stays as
      written (D4), §5 needs no amendment, and §11's "Can a caller choose an identity?" stays open
      because its own trigger is the first slice reading a diagram from a file, which this is not.
      `ShapeId::new` already spells one and this slice's demonstration spells a second, so the
      sentence describes code that exists rather than a rule being added (D3; decisions.md D3;
      research.md Q1; ADR-0040/0041 already hold the two decisions the other four answers refine)

**Commit**: `docs(spec-081):` T029, then `docs(model):` T030 (plan.md commits 4 and 5).

---

## Phase 8: User Story 5 - The shipped demonstration shows a box moved with its connection (B5

Priority: P2)

**Goal**: a bare run prints five captioned pictures, and the fifth is the slice's one claim — the
box the arrow already hangs from, displaced four cells right, with the arrow landing on its new side
and re-routing to the end that did not move. A run given a path still prints one picture and nothing
else, because that is what `cargo xtask render` embeds in a document.

**Independent Test**: `cargo run -p monospace-cli` with no arguments prints five captioned pictures
and the fifth differs from the fourth in exactly the box's old cells, its new cells and the
connector's route; `cargo run -p monospace-cli crates/monospace-cli/assets/demo.json` prints one
picture and nothing else, byte for byte what it printed before (spec.md B5; quickstart.md B5).

### Implementation for User Story 5

- [ ] T031 [US5] Change `demonstrated_pictures` in `crates/monospace-cli/src/main.rs` from a
      four-tuple to a five-tuple, splitting the output four times on `\n\n`, and update its five
      destructuring call sites so `rendering_once_is_the_demonstrations_first_picture` and
      `two_overlapping_boxes_demonstrate_in_opposite_orders` keep comparing the first pictures with
      their meaning unchanged (data-model.md "The tests that read it")
- [ ] T032 [US5] Grow `demonstrate` in `crates/monospace-cli/src/main.rs` to a fifth captioned
      picture over the **same** diagram mutated in place, the way pictures three and four already
      are: after `remove(&ShapeId::new("#1"))`, read the connector back with `get(&the_arrow)`,
      `match` it, and `replace` it with the same connector whose `from` is
      `Position::Reference(Reference { id: the_hung_from, anchor: Anchor::Right })`; then
      `get(&the_hung_from)`, displace it, `replace` it, and draw. **Read the connector back rather
      than restating it** — `to`, its direction, its terminal and the stroke are then the
      description's by construction, and the only thing the fifth picture changes about the arrow is
      where it starts, which is what holds the slice's claim instead of asserting it. Write
      `leaving: Direction::Right` and `terminal: Terminal::Arm` out beside `Anchor::Right`: §6 says
      the direction and the terminal are the caller's and are not derived from the attachment, and
      deriving the direction from the side is
      [#89](https://github.com/andresmoschini/monospace/issues/89)'s. The caption is one line,
      because the tests find each picture by splitting on the blank line before it and the newline
      after the caption (B5.1; data-model.md "`demonstrate` — a fifth picture")
- [ ] T033 [US5] In `demonstrate` in `crates/monospace-cli/src/main.rs`, carry the fifth picture's
      amount as a constant beside the one the third picture already carries —
      `Delta { dx: 4, dy: 0 }` is spec.md B2.2's four cells to the right, beside a comment saying
      the destination was measured against the shipped description's own rows and not chosen by eye
      — and bind the two identities by hand, `ShapeId::new("#3")` for the box and
      `ShapeId::new("#10")` for the connector, each with a comment saying why they are written out:
      a description names its shapes by position, so the demonstration already knows which entry it
      means and has nothing to read back, and a figure added before either would make the written
      value name the wrong shape. Add no field to the description format, add no command-line
      argument, and leave `crates/monospace-cli/assets/demo.json` and the rest of
      `crates/monospace-cli/src/description.rs` unchanged (B5.1, B5.8; research.md Q4; ADR-0035)

### Tests for User Story 5

- [ ] T034 [P] [US5] Contract test: a bare run prints **five** captioned pictures, the first the
      description as written, found by splitting the output on the blank line and pinning no
      caption's wording, in `crates/monospace-cli/src/main.rs` (B5.1, SC-006)
- [ ] T035 [P] [US5] Contract test: the fifth differs from the fourth in exactly the box's old
      cells, its new cells and the connector's route — the same shape of claim the third picture's
      own test makes, reached with the `differing` helper already there, in
      `crates/monospace-cli/src/main.rs` (B5.1, SC-006)
- [ ] T036 [P] [US5] Contract test: a description holding no shapes, and one holding exactly one,
      each demonstrate five pictures and succeed, in `crates/monospace-cli/src/main.rs` (B5.7 — the
      no-shapes case prints five **identical** pictures, since every call is a no-op on an identity
      the diagram does not hold and there is no branch to get wrong; the one-shape case succeeds
      too, and every call in the fifth picture is a no-op as well, which is the trade the
      specification's Clarifications take knowingly and a reader of the fifth picture can see)
- [ ] T037 [P] [US5] Confirm the existing
      `the_demo_path_passed_explicitly_prints_the_demonstrations_first_picture` in
      `crates/monospace-cli/tests/cli.rs` needs **no** change to its body, and record that it still
      passes — it takes the first block and the new picture is appended below rather than inserted,
      so the coordinates it reads do not move. In the same commit, correct the comment over
      `first_demonstrated_picture` in the same file, which names a count of four (B5.2, SC-007;
      data-model.md "The tests that read it")

**Checkpoint**: a person who runs the application sees the slice's one claim without reading a test.

**Commit**: `feat(cli):` T031, T032 and T033, ticking T034-T037 with it (plan.md commit 6).

---

## Phase 9: Polish & Cross-Cutting Concerns

- [ ] T038 Run `cargo run -p monospace-cli > /tmp/demo-after.txt` and
      `diff /tmp/demo-before.txt /tmp/demo-after.txt` — the diff is one appended caption-and-picture
      block and nothing else; then `diff /tmp/file-before.txt /tmp/file-after.txt`, which must print
      nothing at all, since a file's picture is exactly the one it was (B5.2, B5.8, SC-007)
- [ ] T039 Read the fifth picture from that run and confirm the box the arrow hangs from has moved
      four cells right and the arrow has landed on its new side and re-routed to the end that did
      not move — so the slice's claim is in a picture rather than in a test (B5.1, SC-006)
- [ ] T040 Run the two acceptance scenarios from quickstart.md: a description whose `shapes` is
      empty, and one holding a single shape, each printing five pictures and exiting successfully;
      and then read the fifth picture of a **two**-shape description by hand, where neither written
      identity names a real entry, because that is the case the tests cannot see (B5.7;
      data-model.md "`demonstrate` — a fifth picture")
- [ ] T041 Confirm the one rule the spec accepts with nothing to verify it is **named as such**
      rather than described as tested: a shape whose own position does not resolve offers no anchor
      point, so a reference to it resolves to nothing in turn. No position but an endpoint's may
      hold a reference yet, so the rule is unreachable from a test today; it is in the model because
      that document is design intent. Look for it in `docs/diagram-model.md` §4, in
      `specs/082-a-connector-endpoint-hangs-from-a-box-s/contracts/diagram-api.md` and in
      [quickstart.md](quickstart.md), and leave it as it is — this task is a check that the record
      says so, not an edit (constitution principle IV; spec.md Testing expectations)
- [ ] T042 Run `cargo xtask check` and confirm it is green, including the `wasm` step — which
      already names `monospace-diagram`, so it covers the three new types and the changed field with
      no change to `xtask` and no new check. Watch the output rather than the exit code: a step that
      finds something it cannot fix still exits 0 (SC-008; plan.md Constitution Check, principle
      III)
- [ ] T043 Append an entry to `docs/learning-log.md` for this increment: what was learned about Rust
      design and idiom, what was learned about working this way, and optionally a trade-off worth
      remembering. Two candidates are already paid for and are worth writing down rather than
      rediscovering: a `concat!` re-wrap drops a word silently and only a word-level diff shows it,
      and a field's type change cannot be separated from the rule that reads it, so the "structural
      half" of such a refactor is the types arriving unused (constitution principle II)

**Commit**: `docs:` T043. T038-T042 are observations and the gate run, not commits.

---

## What this slice must not add

Each of these is a decision already answered on the sheet or a rule the model states. Reaching past
one means stopping and asking, not deciding and recording afterwards (constitution, No decision
outside the sheet).

- **No item in `monospace-core`**, no arithmetic there, and nothing the core would grow for an
  attached endpoint. The core's `Endpoint.at` is a `Pos` and the diagram's is a `Position`, so the
  diagram's endpoint can no longer be converted into the core's — which is the argument D1 turns on,
  and the anchors are computed from `at`, `size`, `len` and `orientation`, every one of which the
  core already publishes as `pub` (D1; principle VII; research.md Q6)
- **No offset on a `Reference`**, so no way to land an endpoint beside a side rather than on it. D1
  said two fields, and #83 adds the third (D1)
- **No `Position` on `Box.at` or `Line.at`**, so no chain of references longer than one link and no
  cycle. #89 widens it and inherits the cycle obligation with it (D2; ADR-0041)
- **No public `Shape::anchor` and no method that attaches an endpoint**. A caller names a
  `Reference` and puts it in the field; the direct query is the first consumer that needs an anchor
  without drawing (research.md Q3; spec.md "What this slice does not decide")
- **No way to ask which references did not resolve**, and no error, report or panic for one that
  does not. #88 is where that is answered (B2.4, B3.3; ADR-0041)
- **No amendment to §4** (D4), and none to §5, §6 or §9 either — the model already describes this
  end to end, and the slice says which part of it it implements
- **No `add_under`, no way to choose an identity, and no listing of a diagram's identities**. §11's
  own trigger is the first slice reading a diagram from a file (D3; spec.md "What this slice does
  not decide")
- **No corner and no center on an `Anchor`** — four sides, and #90 is the other five (spec.md "What
  this slice does not decide")
- **No reference in the description format**, no field added to it, no new command-line argument,
  and no change to `crates/monospace-cli/assets/demo.json` (B5.8; ADR-0035; research.md Q4)
- **No new dependency, no new crate, and no new gate step** (D1; principle III)
- **No characterization test**. Every rule here is about a picture or a value, and a range too wide
  to read by hand is not what this slice produces (spec.md Testing expectations)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: nothing to do.
- **Foundational (Phase 2)**: no dependencies. The baseline is captured before any edit, and the two
  commits that change no behavior land before anything depends on them.
- **User Story 1 (Phase 3)**: depends on Phase 2. No dependency on another story. It does not need
  T013's field change — `Shape::anchor` takes an `Anchor` and returns an `Option<Pos>` — but it does
  need the gate not to deny dead code, so it cannot be committed on its own.
- **User Story 2 (Phase 4)**: depends on Phase 3, and on T012 for T014. It is the phase that changes
  the field, and T013 is the task every other change in this phase reads.
- **User Stories 3 and 4 (Phases 5 and 6)**: depend on Phase 4, and Phase 6 additionally on T013.
  Neither depends on the other.
- **Records (Phase 7)**: depends on Phase 6 — T029's correction is due the moment the feature lands,
  and T030's sentence is about the demonstration Phase 8 builds.
- **User Story 5 (Phase 8)**: depends on Phases 4 and 6 — the demonstration calls `get`, `replace`
  and `remove`, and rebuilds a `Connector` whose endpoint holds a `Reference`.
- **Polish (Phase 9)**: depends on Phase 8.

### Within Each User Story

- US1: T008, then T009-T012 in parallel. T008 is what they read.
- US2: T013 and T014 in parallel, then T015-T020 in parallel, then T021. Note the compile order is
  the reverse of the usual listing, and that is why: a test naming a method that is not there does
  not fail, it does not compile.
- US3: T022-T023 in parallel; T024 is a check, not code.
- US4: T025, then T026-T027 in parallel, and T028 with them.
- US5: T031-T033 before T034-T037, which read what they produce; T037 any time after T032.

### Parallel Opportunities

- T002 with T005 and T006 — three files, and T006 is the only one that moves a snapshot.
- T009-T012 (US1 tests) in parallel once T008 lands.
- T013 and T014 in parallel — `shape.rs` and `position.rs`.
- T015-T020 (US2 tests) in parallel once T013 and T014 land.
- T022-T023 (US3 tests) in parallel.
- T026-T027 (US4 tests) in parallel once T025 lands.
- T029 and T030 in parallel — two different files, and neither is code.
- T034-T037 (US5 tests) in parallel once T031-T033 land.
- T038-T042 in the Polish phase are observations and can be run in any order.

Twenty-four tasks carry `[P]`. The four stories inside plan.md commit 3 cannot be worked in parallel
in practice, because one commit carries them.

---

## Implementation Strategy

### MVP First (User Story 1 Only)

1. Phase 2 — three public types exist and nothing uses them; a gallery that had never run now does.
2. Phase 3 — every kind answers the four side centers or none of them.
3. **STOP and VALIDATE**: `cargo test -p monospace-diagram` passes; a 4×3 box's right center is
   `{3, 1}`; a one-cell box's answers coincide by the general rule; a connector's four are `None`.

The MVP is not the slice's headline, and that is worth saying rather than letting the checkpoint
imply otherwise: the anchors are the substrate B2 hangs on, and a side center nobody can name is
only half the promise. US1 alone is demonstrable, which is what makes it a checkpoint rather than
the deliverable.

### Incremental Delivery

1. Phase 2 → the types arrive, the gallery runs (commits 1 and 2).
2. Phases 3-6, one commit → an endpoint hangs from a side, follows it when it moves, vanishes when
   it cannot be placed, and stands still when the figure holding it is displaced (commit 3).
3. Phase 7 → the two records the feature makes stale are corrected, and §3 says a caller may spell
   an identity (commits 4 and 5).
4. Phase 8 → the shipped CLI demonstrates the claim to anyone who runs it (commit 6).
5. Phase 9 → the gate is green, the one untestable rule is named as such, the increment is closed
   (commit 7).

Each commit follows plan.md: one `refactor` (T002-T003) and one `test` (T005-T006) that change
nothing anyone can see, the rest `feat` or `docs`, and every commit leaves `cargo xtask check` green
(constitution principles II and V).
