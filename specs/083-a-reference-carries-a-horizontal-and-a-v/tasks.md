---
description: "Task list for feature 083: a reference carries a horizontal and a vertical offset"
---

# Tasks: A reference carries a horizontal and a vertical offset

**Input**: Design documents from `/specs/083-a-reference-carries-a-horizontal-and-a-v/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [data-model.md](data-model.md),
[contracts/diagram-api.md](contracts/diagram-api.md),
[contracts/description-format.md](contracts/description-format.md), [research.md](research.md),
[decisions.md](decisions.md), [quickstart.md](quickstart.md)

**Tests**: Requested by the spec's **Testing expectations** — five contract tests, no
characterization. A behavior rule with no test named against it is an unfinished spec (constitution,
Testing), and the one rule this slice accepts with nothing to verify it is **named as such** in the
Polish phase (T034) rather than described as tested.

**Organization**: Tasks are grouped by the spec's behaviors B1 to B5, in the order
[plan.md](plan.md)'s seven commits deliver them — that order is forced by constitution principles II
and V rather than chosen, so it is the priority order here too. Phases 3 to 5 are three behaviors in
**one** commit, and the split is for traceability, not a license to make three.

**Order within a phase**: implementation before tests, where a test cannot exist before what it
reads. That inverts the usual listing, and it is deliberate: in Rust a test that names a field which
is not there does not fail, it does not compile. Task IDs are in execution order throughout.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different file, no dependency on an incomplete task)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4, US5)
- Include exact file paths in descriptions

## Path Conventions

Rust cargo workspace. `crates/monospace-diagram/src/` for the diagram library, and
`crates/monospace-cli/src/` and `crates/monospace-cli/tests/` for the CLI. The unit tests for the
diagram crate live in the `#[cfg(test)] mod tests` of the file they cover, as the ones already there
do — the crate has no `tests/` directory and this slice adds no integration test to it.

### Four numbers the design gives, measured again before they are acted on

Three of the design's four do not reproduce as written. They are recorded here rather than in
[plan.md](plan.md) because [AGENTS.md](../../../AGENTS.md) is where a fact that lives in the code
and in no document belongs, and because the constitution's principle IV asks for the number rather
than the estimate. **The tasks below carry the measured figures; the artifacts' prose keeps its own,
and the difference is the maintainer's to see rather than a correction made in passing.**

- `plan.md` and `data-model.md` say **fifteen** `Reference` literals across `position.rs`,
  `diagram.rs`, `gallery.rs` and `main.rs`. `grep -c 'Reference *{'` returns 15 lines across those
  four files, and one of them is `pub struct Reference {` in
  `crates/monospace-diagram/src/position.rs:47` — the declaration, not a literal. There are
  therefore **fourteen literal sites**, and T003 is the one that edits them.
- `contracts/description-format.md` says **eighteen** endpoint spellings in six documents, in its
  prose, while its own table sums to **sixteen** (`2, 2, 2, 2, 4, 4`). Measuring
  `git ls-files '*.md'` with the rule `xtask` itself walks the tree with gives sixteen connector
  endpoints in six files: `README.md` 2, `CONTRIBUTING.md` 2, `docs/diagram-model.md` 2,
  `docs/model.md` 2, `specs/055-an-arrow-end-is-a-glyph-arm-or-nothing/spec.md` 4,
  `specs/081-a-shape-can-be-removed-and-replaced/decisions.md` 4. The prose's corollary — "four that
  are not" checked by the gate — is the same slip read the other way: sixteen minus
  `CONTRIBUTING.md`'s two is **fourteen**, which is what the contract says the `render` step covers.
  T023 re-spells sixteen and T033 records the slip.
- The eighteen _shape-level_ `at` spellings in those same six files are the box and line ones, and
  they do not change: a position other than a connector endpoint's may not be a reference
  (ADR-0041). A diff that touches one is a diff that widened the union further than ADR-0041 allows.
- The test counts in [quickstart.md](quickstart.md) — 55 in `monospace-diagram`, 15 in
  `monospace-cli`'s unit tests, 15 in its integration tests — **do** reproduce, counted as `#[test]`
  per file: `diagram.rs` 43, `gallery.rs` 4, `position.rs` 5, `shape.rs` 3; `description.rs` 7,
  `main.rs` 8; `cli.rs` 15. T001 records them as the baseline.

---

## Phase 1: Setup

No setup is needed: no dependency is added, no crate is added, the toolchain is unchanged, and no
gate step is added — `wasm` already names `monospace-diagram` (plan.md Constitution Check, principle
III; D1).

---

## Phase 2: Foundational — the third field, unset

**Purpose**: expand/contract's first half. A public struct grows two fields that nothing sets and
nothing reads, and every literal in the workspace is brought up to date so the crate builds. Nothing
here may touch a test and nothing here may change a picture.

- [x] T001 Run `cargo run -p monospace-cli > /tmp/demo-before.txt` and
      `cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-before.txt`, and
      keep both files until Phase 9. Then run `cargo run -p monospace-cli | sed -n '2,5p'` and read
      off the two numbers B5 rests on: the fifth entry of `crates/monospace-cli/assets/demo.json` is
      a four-by-three box at `{20, 1}` occupying columns 20 to 23 and rows 1 to 3, and the tenth
      entry's `▲` terminal stands at `(22, 4)`. `side_centre` puts that box's bottom centre at
      `{20 + 3 / 2, 1 + 2}` = `{21, 3}`, and `{21, 3} + (1, 1)` is `{22, 4}` — the point the file
      spells outright at `assets/demo.json:73`. If those numbers do not come out of the picture the
      way `side_centre` and `Delta::apply` say they do, the tenth entry is wrong before a line of it
      is written. Record the test counts as T001's second half: `cargo test --workspace` green at
      **55 / 15 / 15** (quickstart.md "Build and test the workspace")
- [x] T002 In `crates/monospace-diagram/src/position.rs`, add a third public field to `Reference` —
      `/// How far from that side, along each screen axis, in cells.` `pub offset: Delta` — leaving
      the derives `Debug, Clone, PartialEq, Eq` and **not** `Copy` as they are, because `ShapeId` is
      a `String` and the struct inherits the weaker of the two. Rewrite `Reference`'s type-level
      rustdoc, whose second paragraph reads "Two fields, and that is the whole of it … the offsets
      arrive with their own issue", so it names three fields and says the field is public as the
      other two are. In the same edit, rewrite the fourth paragraph of `Position::resolve`'s rustdoc
      — the one headed "**There is no offset arithmetic here, and that is a decision rather than an
      omission**" — so it says the field exists and **is not read yet**, which is what makes this
      commit structural rather than a lie. Change **no** rustdoc on `Delta`, and add **no** line to
      the `Reference(reference) =>` arm (D1; contracts/diagram-api.md `Reference`; plan.md commit 1)
- [x] T003 [P] Add `offset: Delta { dx: 0, dy: 0 }` to every `Reference` literal in the workspace —
      **fourteen** sites, counted rather than estimated: `position.rs:336` one,
      `crates/monospace-diagram/src/diagram.rs` eleven (lines 1486, 1530, 1620, 1629, 1676, 1680,
      1738, 1887, 1934, 1980 and 2064), `crates/monospace-diagram/src/gallery.rs` one, and
      `crates/monospace-cli/src/main.rs:172` one. The fifteenth match is the declaration T002
      edited, not a literal; if a `grep -c 'Reference *{'` over those four files does not print 15,
      stop and recount rather than editing what the count turns out to be. Change nothing else in
      those four files: the gallery's `shapes:` label and `Position::displaced_by`'s body are both
      read by later phases and neither moves here (plan.md commit 1; data-model.md "`Reference` — a
      third field")
- [x] T004 [P] Confirm `cargo test --workspace` is green at **55 / 15 / 15** with **no test added
      and no test changed** — a structural commit that moves a test is a behavioral one wearing the
      other commit type's prefix — and that `cargo xtask check` is green. Then run
      `cargo run -p monospace-cli > /tmp/demo-after-1.txt` and
      `diff /tmp/demo-before.txt /tmp/demo-after-1.txt`, which must print **no output**: every
      reference in the repository carries a zero offset and `assets/demo.json` still spells its far
      endpoint outright at this point, so there is nothing for the missing step to change. No
      `cargo insta review` is needed or wanted in this commit, because no snapshot moved
      (constitution principles IV and V; plan.md commit 1)
- [x] T005 Prove the new field is inert rather than assuming it, as constitution principle IV asks:
      restore `Position::resolve`'s old body — `shape.anchor(reference.anchor)` with no addition —
      and confirm `cargo test --workspace` is still green at the same counts, then put the body
      back. A `refactor` that quietly changed behavior would pass T004's picture diff on this
      demonstration, because the demonstration holds no non-zero offset yet (quickstart.md "Commit
      1"; principle IV)

**Checkpoint**: `Reference` carries a third field, every literal in the workspace sets it to zero,
and the code does not read it. User story work can begin.

**Commit**: `refactor(diagram):` T002 and T003 (plan.md commit 1). T001, T004 and T005 are a
captured file, a gate run and one deliberate red-then-green experiment, not commits.

---

## Phase 3: User Story 1 - An endpoint stands off the side (B1, Priority: P1) 🎯 MVP

**Goal**: `Position::resolve` asks the diagram for the figure, asks that figure for the anchor, and
adds `offset` to the point that came back — the order §4 _Positions_ states, and the whole of
research.md Q1's measurement. An endpoint attached to a box's side can now stand any number of cells
clear of that side, which is the arrangement that has no picture at all today.

**Independent Test**: ask `resolve` for a four-by-three box at the origin's right side with `(2, 0)`
and get `{5, 1}`, and its bottom with `(0, 1)` and get `{1, 3}` — each pinned against the absolute
point rather than against the other. Then draw: a connector hanging from that reference must produce
the same cells as the same connector standing at `{5, 1}` (spec.md B1.1, B1.2, B1.3; quickstart.md
B1; SC-001, SC-003).

The three steps, in the order §4 _Positions_ states them and the whole of research.md Q1's
measurement:

```rust
Self::Reference(reference) => {
    let shape = diagram.get(&reference.id)?;
    let point = shape.anchor(reference.anchor)?;
    Some(reference.offset.apply(point))
}
```

### Implementation for User Story 1

- [ ] T006 [US1] In `crates/monospace-diagram/src/position.rs`, give `Position::resolve`'s
      `Reference` arm its third step — the one shown in this phase's introduction — and only its
      third step. The two `?`s come **before** the addition, which is what makes a reference that
      resolves to nothing still nothing however large the offset is (SC-004). `Delta::apply` is
      `pub(crate)` and stays the crate's only arithmetic — it saturates, and the contract says why
      saturation is the right answer here. Rewrite `resolve`'s rustdoc paragraph T002 rewrote so it
      describes what the method now does, and keep the paragraph naming #143: a displacement does
      **not** reach these offsets, and that is a decision rather than an omission (D1, Q1, Q4; B1.1;
      data-model.md "`Position::resolve` — the three steps"; contracts/diagram-api.md `resolve`
      gains a step; plan.md commit 2)

- [ ] T007 [P] [US1] In `crates/monospace-diagram/src/delta.rs`, give `Delta`'s rustdoc its second
      reading. The module's first line reads "How far a figure moves along each axis, **and nothing
      else**" and the struct's says "**and no third thing**: the model's _Vocabulary_ gives a delta
      that row and gives it no other" — both are false the moment T006 lands, and this is D1's named
      cost. Say on each of them that a delta is also how far a reference stands from the side it
      hangs from, citing §1's `Reference` row. Change **no** field doc comment:
      `contracts/diagram-api.md` shows `Delta`'s two fields exactly as they are and says no item is
      added, removed or retyped, and `apply` keeps both its saturation rule and its crate-private
      visibility (D1; research.md Q1; data-model.md "`Reference` — a third field")
- [ ] T008 [P] [US1] Contract test: `resolve` asked rather than drawn — a four-by-three box at the
      origin, its right side with `Delta { dx: 2, dy: 0 }` resolving to `Some(Pos { x: 5, y: 1 })`
      and its bottom with `Delta { dx: 0, dy: 1 }` resolving to `Some(Pos { x: 1, y: 3 })`, in
      `crates/monospace-diagram/src/position.rs`. Each assertion pins the absolute point, never the
      other side of the comparison, so a resolve that added to the wrong thing fails rather than
      passing on a pair that is wrong together. Build the box through `Shape::anchor`, the
      crate-private query already there, so the test cannot disagree with the drawing by
      construction (B1.1, B1.2, SC-001; quickstart.md B1; data-model.md "What the tests pin")
- [ ] T009 [P] [US1] Contract test: **the `assert_ne!` is the point of the no-offset case** — a
      reference with `Delta { dx: 0, dy: 0 }` resolves to the same `Pos` the same anchor answers
      **and is not equal to** that bare `Pos`, in `crates/monospace-diagram/src/position.rs`, beside
      T008. A `resolve` that added nothing at all would pass every equality in T008; only the
      inequality between a no-offset reference and a point catches it. 082's displacement test works
      the same way and this one has to as well (B1.3; quickstart.md B1; data-model.md "What the
      tests pin")
- [ ] T010 [US1] In `crates/monospace-diagram/src/gallery.rs`, edit two strings by hand and
      re-accept the snapshot. The `shapes:` line of `an_endpoint_hangs_from_a_side_and_follows_it`
      reads `arm_connector(from = Reference(#1, Right) -> 7,1)` and becomes
      `Reference(#1, Right, offset (0,0))` — **a label cannot disagree with the values it names**.
      The rustdoc above that test claims the gallery "is the only carrier in the crate that can
      reach a reference at all — a `<!-- render: -->` marker reads a description, and the wire
      format holds a point", and both halves of the second one are false the moment `at` can hold a
      reference; correct it here, where it becomes wrong, and add no block for the offset. The label
      is a **string written by hand** (`gallery.rs:368`) and not any `Debug`, which is the
      correction research.md Q6 carries as a dated note. Run `cargo insta review` and **read** the
      offered snapshot: the first block must still be §6's picture and the second the same box
      displaced four cells right, and **nothing in either picture may move by a cell** — the only
      difference the diff may show is on the `shapes:` line. This is a `feat` and a `refactor` may
      not carry a snapshot, which is why the label is not in Phase 2 (Q6; ADR-0064; plan.md commit
      2; constitution, Fixing a commit)
- [ ] T011 [P] [US1] Contract test: the reference with `Delta { dx: 2, dy: 0 }` on the box's right
      side draws **exactly** the cells the same connector standing at the absolute point it resolves
      to draws, in `crates/monospace-diagram/src/diagram.rs` using the `cells`, `drawn`, `draw_of`
      and `differing` helpers already there. The offset is `2, 0` on `THE_SIDE_CENTRE` — the box is
      four by three at the origin, so its right side centre is `{3, 1}` and the endpoint is `{5, 1}`
      — and `the_window()` is 13 by 4, which holds the route. Assert the resolved point by
      coordinate **before** the picture, as
      `a_hanging_endpoint_draws_what_the_point_it_resolves_to_draws` does, and leave that
      zero-offset test where it is: it is T009's partner and a slice that replaced it with an offset
      case would have lost the case where nothing is added (B1.1, B1.2, SC-001, SC-003;
      data-model.md "What the tests pin")

**Checkpoint**: an endpoint can stand clear of a side, and the picture it draws is the picture the
resolved point draws.

---

## Phase 4: User Story 2 - The offset is a gap from the side, so it travels with it (B2, Priority: P1)

**Goal**: the offset is added to whatever the anchor answers **now**, so displacing the figure a
reference hangs from carries the endpoint with it and the gap between border and endpoint does not
change. A reference that resolves to nothing is still nothing, and a large offset does not make it
resolve.

**Independent Test**: draw the box at the origin and the connector hanging from a reference with
`(2, 0)`; displace the box four cells right and draw again; each must equal the same box drawn at
its new position with the endpoint at the absolute point the reference now resolves to. Then replace
the box with a line and check the offset is added to the line's own middle (spec.md B2.1, B2.2,
B2.3; quickstart.md B2; SC-002, SC-004).

### Tests for User Story 2

- [ ] T012 [P] [US2] Contract test: the box displaced with the offset unchanged, in
      `crates/monospace-diagram/src/diagram.rs`. Build the expected picture from the two positions —
      the box at `{4, 0}` and the connector starting at the new absolute point `{7, 1}` — rather
      than pinning it as text, so what is claimed is where the endpoint lands and not how a
      connector draws. Carry an `assert_ne!` on the offset beside the equality: **a `resolve` that
      ignored the field would pass the picture comparison** in this window, because the box's right
      side centre is still a cell the connector can start from. `differing` must be non-empty and
      must not reach past the far endpoint (B2.1, SC-002; quickstart.md B2; data-model.md "What the
      tests pin")
- [ ] T013 [P] [US2] Contract test: a box **replaced by a line** under an offset — the offset is
      added to the line's own side middle, not to the box's old one — in
      `crates/monospace-diagram/src/diagram.rs`. A horizontal line asked for its top and for its
      bottom centre is asked the same question twice and gets the offset added once, so the case
      that distinguishes the two answers is a horizontal line with a **non-zero** `dy`. Compare
      against the same line with the connector standing at the point the reference resolves to
      (B2.3; data-model.md "What the tests pin")
- [ ] T014 [P] [US2] Contract test: non-resolution with a large offset, by drawing, **twice** — a
      reference to an identity the diagram does not hold, and a reference to an anchor a kind does
      not answer, which is a connector and is the answer that keeps a chain of references one link
      long. Each carries an offset big enough to be somewhere, and each asserts the figure is
      **absent from the output** _and_ that every other shape's cells are **unchanged**, the second
      half being what distinguishes "the connector is not drawn" from "the drawing stopped" — a
      connector drawn partly passes the first. Use the `the_unrelated_box()` helper already in that
      module for the shape that has nothing to do with either end (B2.2, SC-004; spec.md edge case:
      a connector answers no side, so an offset added to nothing is still nothing; data-model.md
      "What the tests pin")
- [ ] T015 [P] [US2] Contract test: a box **one cell wide** with an offset — its two coincident side
      centres get the offset added once, and the offset is what separates them afterwards — in
      `crates/monospace-diagram/src/diagram.rs`. The degenerate figure is the one an implementation
      that special-cased the ordinary box gets wrong, and 082's
      `a_box_one_cell_wide_or_one_cell_tall_answers_the_same_rule` in `position.rs` is why the
      general rule is the claim rather than the coincidence (spec.md edge case; quickstart.md B2)

**Checkpoint**: the offset is a gap from the side rather than a point, and a large one on a
reference to nothing is still nothing.

---

## Phase 5: User Story 3 - A displacement still leaves a hanging end alone (B3, Priority: P1)

**Goal**: the offsets exist and a displacement still does not reach them. The absolute end of a
connector moves and the hanging one does not move at all, exactly as 082 pinned. This is the
intermediate state 082's D4 was written to allow, and #143 is the slice that walks past it.

**Independent Test**: a connector with one endpoint absolute and one a reference, displaced two
cells down — the absolute endpoint moves, the referenced one does not, and the route is drawn
between them. The test that exists today is the test; it must pass unchanged, which is what makes it
a check rather than a claim (spec.md B3.1; quickstart.md B3; SC-006; research.md Q4).

### Implementation for User Story 3

- [ ] T016 [US3] Add **no** code for this behavior, and say so in the commit rather than leaving it
      to be found. `Position::displaced_by`'s reference arm still comes back cloned unchanged, and
      wiring the offsets into it would be taking
      [#143](https://github.com/andresmoschini/monospace/issues/143)'s decision (D4, Q4; B3.1;
      contracts/diagram-api.md "`displaced_by` does not change"; constitution, No decision outside
      the sheet)
- [ ] T017 [P] [US3] Correct two rustdoc sentences that T002 and T003 made false, leaving both
      bodies alone. `Position::displaced_by` in `crates/monospace-diagram/src/position.rs` says "**A
      `Position::Reference` has none yet** — its identity names another figure" and the reason it
      does not move is no longer that it has no coordinates: it has an offset, and the displacement
      does not reach it. `Shape::displaced_by` in `crates/monospace-diagram/src/shape.rs` says "a
      reference names another figure rather than a point, so there are no coordinates to move",
      which reads the same way. Say on each of them what the arm now does and which issue carries
      the real rule. A rustdoc is code, so it is corrected in the commit that makes it wrong — 082's
      Q7's rule, which is what research.md Q7 applies to the same sentence elsewhere (Q7; ADR-0041;
      plan.md commit 2; constitution, Fixing a commit)

### Tests for User Story 3

- [ ] T018 [P] [US3] Confirm `a_displacement_moves_a_point_and_leaves_a_reference_alone` in
      `crates/monospace-diagram/src/position.rs` passes **unchanged**, and record that it passed
      before the slice as well as after it. It is the whole of B3, and the gap it leaves is named in
      the contract rather than left to be found. Its own `Reference` literal gained
      `offset:     Delta { dx: 0, dy: 0 }` in T003 and nothing else about it changed, which is what
      "unchanged" has to mean here (B3.1, SC-006; quickstart.md B3; research.md Q4)

**Checkpoint**: the hanging end stands still and the free end moves, and the record of why is a test
that did not have to be written.

**Commit**: Phases 3, 4 and 5 together as one `feat(diagram):` — T006-T011, T012-T015, T016-T018,
plus the gate run, ticking every test above with it (plan.md commit 2). The three phases are for
traceability, not a license to make three commits: the addition, the label and the two rustdoc
corrections are one behavior, and the offset is what makes the other two sentences false.

---

## Phase 6: Records this slice amends

**Purpose**: two `docs` commits, neither of them a behavior, and so neither carrying a `[Story]`
label. They are placed here rather than in the Polish phase because plan.md orders them between the
diagram feature and the wire, and the first of them is due the moment the arithmetic lands. Each is
its own commit: one amends the model, one corrects a record in another feature's directory.

- [ ] T019 [P] Add one clause to the `Delta` row of §1 _Vocabulary_ in `docs/diagram-model.md` — it
      reads "How far a figure moves along each axis: a horizontal and a vertical amount" and its
      second clause is the one that still holds, so the row gains the first meaning this slice gave
      the type. The `Reference` row above it needs **no** change: it has named three fields, and "a
      horizontal and vertical offset" all along. Amend **nothing else** — §4 already says a
      reference resolves by asking for the anchor and adding the offsets, §6 already says an
      endpoint's position may be a reference, and §11 stays open. The amendment follows the answer
      rather than preceding it, which is where 082's D3's one-sentence change to §3 landed
      (`505fd0d`) (D1; research.md Q7; plan.md commit 3)
- [ ] T020 [P] Correct the one word
      `specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md` says
      wrong: its connector section is headed `### "arrow"` and its example carries
      `"kind": "arrow"`, and the wire tag became `"connector"` in `c7539bc` under ADR-0065 —
      measured, a description carrying `arrow` is refused by name. Fix the heading and the example's
      `kind`, and **nothing else in that file**: its `"from"` and `"to"` `at` spellings are the
      record of what the format was, and this slice supersedes that file rather than rewriting it.
      Say in the commit message what was wrong, which is 082's Q7's rule and what the constitution
      asks of a correction someone could have acted on (Q7; plan.md commit 4; constitution, Fixing a
      commit)

**Commit**: `docs(model):` T019, then `docs(spec-079):` T020 (plan.md commits 3 and 4).

---

## Phase 7: User Story 4 - A description can name a reference and its two offsets (B4, Priority: P1)

**Goal**: a description file may name a reference, so an endpoint's `at` is tagged by `kind` and is
either a point or a reference — a file cannot say both, cannot say neither, and cannot say one by
mistake. A reference naming a shape the file's diagram does not hold is not an error: the figure
holding it is simply not drawn and the run succeeds, which is the cost ADR-0041 already accepts.

**Independent Test**: a description whose connector holds
`{"kind": "reference", "shape": "#1", "anchor": "bottom", "offset": {"dx": 1, "dy": 1}}` draws the
diagram that reference resolves to, while a description naming `#7` in a two-shape file draws the
box and **no connector at all** and exits successfully. A point written without a tag is refused
with ``missing field `kind` `` and `{"kind": "arrows", …}` is refused with
``unknown variant `arrows`, expected `point` or `reference` `` (spec.md B4.1, B4.2, B4.3;
quickstart.md "The wire"; SC-004).

The union, tagged the way `Terminal` and `ShapeDescription` already are in that file:

```rust
#[derive(Deserialize, Debug, Clone)]
#[serde(tag = "kind", rename_all = "lowercase")]
enum At {
    Point(Pos),
    Reference {
        shape: String,
        anchor: AnchorDescription,
        #[serde(default)]
        offset: OffsetDescription,
    },
}
```

### Implementation for User Story 4

- [ ] T021 [US4] In `crates/monospace-cli/src/description.rs`, add the three private mirror types
      and the union, and change one field's type. Nothing here is public and nothing here reaches
      `monospace_diagram` except through the existing conversion (ADR-0035). `At` is the enum shown
      in this phase's introduction. `AnchorDescription` is `#[serde(rename_all = "lowercase")]` with
      `Top`, `Right`, `Bottom` and `Left` and a `From` into `monospace_diagram::Anchor`;
      `OffsetDescription` is `{ dx: i32, dy: i32 }` deriving
      `Deserialize, Debug, Clone, Copy, Default` with a `From` into `monospace_diagram::Delta`, and
      `Default` is what makes `offset` optional and absent zero. `Point` is a **newtype** variant
      holding the file's own `Pos` rather than a struct variant with `x` and `y` written out —
      measured, not assumed: both spellings read `{"kind": "point", "x": 1, "y": 1}` and refuse
      `{"x": 1, "y": 1}` the same, and the newtype keeps one `Pos` in the file rather than a second
      pair of coordinates to convert. It carries a trap worth writing in the rustdoc: **an
      internally tagged newtype variant deserializes and does not serialize**, which is nothing
      today because the file is read and never written. Then change `Endpoint`'s `at` from `Pos` to
      `At` and its `From<Endpoint> for DiagramEndpoint` to match: `At::Point(at)` becomes
      `Position::Absolute(at.into())` and `At::Reference { shape, anchor, offset }` becomes
      `Position::Reference(Reference { id: ShapeId::new(shape), anchor: anchor.into(), offset: offset.into() })`.
      Rustdoc all four types and the field the way the rest of this file is documented (D1, D2, D3;
      Q1, Q2, Q3; data-model.md "`monospace-cli` — four private mirror types";
      contracts/description-format.md `at`)

- [ ] T022 [P] [US4] Add `"kind": "point"` to the **six** endpoint literals in
      `crates/monospace-cli/src/description.rs` — the two in
      `a_multi_grapheme_terminal_glyph_fails_to_deserialize` and the four inside the `ARROW` and
      `ARROW_WITHOUT_A_TERMINAL` constants the three terminal tests share. Counted, not estimated.
      The `at` spellings that belong to a `Box` or a `Line` in that file **do not change**, and a
      diff that touches one is a diff that widened the union further than ADR-0041 allows (D2;
      plan.md commit 5; quickstart.md "The wire")
- [ ] T023 [P] [US4] Add `"kind": "point"` to the **two** endpoint literals in
      `crates/monospace-cli/tests/cli.rs` — the connector in
      `a_file_with_a_box_a_line_and_a_connector_prints_all_three_composed` — and to the two in
      `crates/monospace-cli/assets/demo.json`'s tenth entry, whose `to` is replaced wholesale by
      T028 and whose `from` stays a point. Eight literals in Rust, six and two; the eleven `at`
      spellings that belong to a `box` or a `line` in the same files do not change, which is what a
      reader checking the diff should see (data-model.md "The tests pin"; plan.md commit 5)
- [ ] T024 [US4] Re-spell the **sixteen** endpoint `at`s in the six tracked documents, adding
      `"kind": "point"` to each and changing nothing else in those files: `README.md` two,
      `CONTRIBUTING.md` two, `docs/diagram-model.md` two, `docs/model.md` two,
      `specs/055-an-arrow-end-is-a-glyph-arm-or-nothing/spec.md` four, and
      `specs/081-a-shape-can-be-removed-and-replaced/decisions.md` four. **Sixteen is the measured
      count**, taken with the rule `xtask` itself walks the tree with — a marker shown inside a
      fence is an illustration of the grammar and is not an instance of one
      (`xtask/src/render.rs:211`) — and it is what `contracts/description-format.md`'s own table
      sums to, where its prose says eighteen. Take the count by hand before editing rather than from
      either number, and if it comes out something else again, write down what it came out as and
      stop: three records disagreeing about one count is worth more attention than a third guess
      (D2; research.md Q2; contracts/description-format.md "What has to be re-spelled")
- [ ] T025 [P] [US4] Run `cargo xtask render` and then `cargo xtask check`, which rewrites and then
      re-checks every `<!-- render: -->` block from the description beside it — so **fourteen of the
      sixteen** are covered for free and a description edited wrongly is a red `render` step rather
      than a quietly different picture, which is the cheapest possible check on a mechanical change
      of this size. **The two it does not check are `CONTRIBUTING.md`'s**: that marker is shown
      inside a ````markdown` fence as the grammar of a marker, so the walker steps over it, and its
      description is a working example a reader copies. Read that one by eye, as an example rather
      than as a sample (research.md Q2; plan.md commit 5; quickstart.md "The wire")
- [ ] T026 [P] [US4] Run the two refusals from [quickstart.md](quickstart.md) before the change and
      watch the second one **pass** — today `from.at` is a bare point, an unknown `kind` beside `x`
      and `y` is an unknown field, and the file renders. That silence is what the tag is there to
      end, and seeing it go is the cheapest proof that the union is real. Afterwards the first file
      must report ``missing field `kind` `` and the second
      ``unknown variant `arrows`, expected     `point` or `reference` ``, both on stderr with
      nothing on stdout and a failure status. The line and column that follow are `serde`'s and move
      with the bytes, which is why a test pins the message and not the position (D2; research.md Q2;
      contracts/description-format.md "What a malformed file does")

**Checkpoint**: a description can say `"at": {"kind": "reference", …}`, and every description in the
repository says the other thing — the same pictures, byte for byte.

**Commit**: `feat(cli):` T021-T026, ticking their checkboxes with it (plan.md commit 5). One commit
and not two because a tag lands everywhere or nowhere: half of it refuses every description in the
repository. US4's contract tests are **not** here — plan.md commit 6 carries them, with the
demonstration, because the file format changing and a picture being proved unchanged by it are two
different claims.

---

## Phase 8: User Story 5 - The far end hangs from a box (B5, Priority: P2)

**Goal**: the shipped demonstration's tenth entry names the bottom side of the fifth entry with an
offset of one in each axis, and **all five pictures come out byte for byte what they are**. The
evidence is in a file rather than in a picture, and the picture not moving is what proves the
arithmetic. The cost the design did not foresee is inside the demonstration's own test suite: a
reference is a position in a list, and a test that removes a listing shifts every identity after it.

**Independent Test**: `cargo run -p monospace-cli` prints five captioned pictures and
`diff /tmp/demo-before.txt` against it prints **nothing at all**; a run given a path prints one
picture and nothing else, byte for byte what it printed before (spec.md B5.1, B5.2; quickstart.md
B5; SC-005).

The tenth entry's `to`, which says the same place the file spells outright today by the other route:

```json
"to": {
  "at": { "kind": "reference", "shape": "#5", "anchor": "bottom",
          "offset": { "dx": 1, dy: 1 } },
  "leaving": "down",
  "terminal": { "kind": "glyph", "glyph": "▲" }
}
```

### Implementation for User Story 5

- [ ] T027 [US5] In `crates/monospace-cli/assets/demo.json`, change the tenth entry's `to` to the
      object shown in this phase's introduction, and nothing else in that file. `{21, 3} + (1, 1)`
      is `{22, 4}`, which is the point this entry spelled outright, so the rendered picture does not
      move by a character — the demonstration removes `#1`, displaces `#1` and displaces `#3`, and
      the fifth shape is none of them. Its `from` stays a point, now a tagged one (T023). The fifth
      picture still rehangs that `from` in code, because that is the demonstration's own change to
      the picture and not the file's (D3, Q5; B5.1; data-model.md "The demonstration's tenth entry";
      plan.md commit 6)

- [ ] T028 [US5] In `demo_without_its_first_entry` in `crates/monospace-cli/src/main.rs`, renumber
      the one reference the helper moves, `"#5"` to `"#4"`, and write in its doc comment why. The
      helper builds the demonstration with its first entry left out **as text** and reads it again,
      and reading it again issues the identities from scratch in array order, so the same `"#5"`
      names what was the sixth entry — the box at `{18, 0}`, whose bottom centre is `{19, 2}` and
      whose `+ (1, 1)` is `{20, 3}`. The fourth picture would then be a different picture from the
      one `the_third_picture_moves_one_figure_and_the_fourth_takes_that_figure_out` compares it to,
      and the difference begins at row 3. This is not a workaround bolted on: it is D3 answered in
      the only place the consequence is observable, and its doc comment should say so rather than
      describing the mechanics only (D3, Q7; data-model.md "The one test this moves"; plan.md commit
      6; spec.md "What this slice does not decide")

### Tests for User Story 5

- [ ] T029 [P] [US5] Contract test: the two refusals the union produces, in
      `crates/monospace-cli/src/description.rs` beside the terminal refusals already there — a point
      written without a tag reports ``missing field `kind` ``, and `{"kind": "arrows", …}` reports
      ``unknown variant `arrows`, expected `point` or `reference` ``. Assert on the **message**, not
      the line and column that follow it, which are `serde`'s and move with the bytes. Add the third
      case the probe measured: `{"kind": "point", "x": 1, "y": 1, "shape": "#5"}` is **accepted**
      and the stray `shape` is dropped in silence, because a tag is the rule and a field beside it
      is an unknown field exactly as a description carrying `mode` is today (B4.1; research.md Q2;
      contracts/description-format.md "What a malformed file does")
- [ ] T030 [P] [US5] Contract test: the three wire behaviors of B4 arriving through a file, in
      `crates/monospace-cli/tests/cli.rs`. **B4.1** a connector whose endpoint is a reference draws
      the diagram it resolves to. **B4.2** a description naming `"#7"` in a file holding one box
      draws the box and **no connector at all**, exits successfully, and says so in a comment — that
      is ADR-0041's silent hole arriving through a wire, and a large offset is what makes it a test
      rather than a comment. **B4.3** a reference carrying no `offset` at all draws exactly what a
      description naming the point it resolves to draws, byte for byte, which is the case
      `{"kind": "reference", "shape": "#1", "anchor": "right"}` exists for (B4.1, B4.2, B4.3,
      SC-004; quickstart.md B5)
- [ ] T031 [P] [US5] Contract test: the demonstration prints the **five** pictures it prints today,
      character for character, in `crates/monospace-cli/src/main.rs` — found by splitting the output
      on the blank line and pinning no caption's wording, and each picture compared against the
      output of the shipped file with that one entry changed. Read the fifth picture too: the box
      the arrow hangs from is still displaced four cells right, because the demonstration's own
      change and not the file's is what the fifth picture shows.
      `cargo run -p monospace-cli     <path>` still prints one picture and nothing else, which is
      what `cargo xtask render` embeds (B5.1, B5.2, SC-005; data-model.md "What the tests pin")
- [ ] T032 [P] [US5] Contract test:
      `the_third_picture_moves_one_figure_and_the_fourth_takes_that_figure_out` in
      `crates/monospace-cli/src/main.rs` passes **only because of T028**, and confirm that by
      renaming the reference back to `"#5"` and watching it fail on two columns of the arrow's route
      before restoring it. A test that would pass either way is a check of nothing, and this is the
      one place in the slice where a renumbering is load-bearing enough to be worth a deliberate red
      (B4.2, SC-004; quickstart.md B5; constitution principle IV)

**Checkpoint**: a person who runs the application sees five pictures identical to the ones they saw
before, while the file those pictures came from now names a side instead of spelling a point.

**Commit**: `feat(cli):` T027, T028, T029-T032, ticking T029-T032 with it (plan.md commit 6).

---

## Phase 9: Polish & Cross-Cutting Concerns

- [ ] T033 Run the diff three times over, because three different things can go wrong and two of
      them draw the same picture as correct code: once after T026, once after T027, and once now —
      `cargo run -p monospace-cli > /tmp/demo-after.txt` and
      `diff /tmp/demo-before.txt     /tmp/demo-after.txt`, which must print **no output at all**;
      then `cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-after.txt`
      and `diff /tmp/file-before.txt /tmp/file-after.txt`, which must also print nothing, since a
      file's picture is exactly the one it was (B5.1, B5.2, SC-005)
- [ ] T034 Run the acceptance scenarios from [quickstart.md](quickstart.md) that the tests do not
      cover: a description whose `shapes` is empty, printing five identical blank captioned pictures
      and exiting successfully; and one whose connector names a reference to a shape it does not
      hold, printing a box and **no connector at all** and exiting successfully. Read the fourth
      picture of a bare run with `sed -n '/With that same shape taken out/,/^$/p'` and confirm it is
      byte-identical to what it was — that is the picture T028 exists for (B4.2, B5.2)
- [ ] T035 Confirm the one rule this slice accepts with nothing to verify it is **named as such**
      rather than described as tested: an offset that puts the endpoint inside the figure it hangs
      from — a horizontal offset to the _left_ on a right side — draws it there, composing in the
      shared cell by the rule two figures sharing a cell always obey, rather than the offset being
      refused. Nothing in the model or the specification promises otherwise and no code path could
      refuse it. Look for it in `docs/diagram-model.md` §4, in
      `specs/083-a-reference-carries-a-horizontal-and-a-v/contracts/diagram-api.md` and in
      [spec.md](spec.md) _Edge cases_, and leave it as it is — this task is a check that the record
      says so, not an edit (constitution principle IV; spec.md Testing expectations)
- [ ] T036 Record the count disagreement T023 carries: three records give three numbers for the
      spellings to re-spell, and sixteen is the one measured. Say in `docs/learning-log.md`'s entry,
      or in this task's own commit message, which number came out of a `grep` and which two did not
      — a design that measured its own rule (`render.rs:211`) and then quoted the raw count is the
      kind of slip a later slice re-inherits silently (constitution principle IV; AGENTS.md "Facts
      that are in the code and in no document")
- [ ] T037 Run `cargo xtask check` and confirm it is green, including the `wasm` step — which
      already names `monospace-diagram`, so it covers the widened `Reference` and the changed
      `resolve` with no change to `xtask` and no new check, which is why principle III's two-commit
      rule does not apply. Watch the output rather than the exit code: `rustfmt` reports that
      `group_imports` needs nightly and exits 0, and so does a step that finds something it cannot
      fix (SC-007; plan.md Constitution Check, principle III)
- [ ] T038 Append an entry to `docs/learning-log.md` for this increment: what was learned about Rust
      design and idiom, what was learned about working this way, and optionally a trade-off worth
      remembering. Two are already paid for and worth writing down rather than rediscovering: a
      hand-written snapshot label cannot move by itself when the value it names grows a field, so a
      `refactor` that widens a type cannot carry the label and the behavioral commit must; and a
      reference named by its position in a list makes the demonstration's own text-mutating test
      helper a place where an identity shifts (constitution principle II)

**Commit**: `docs:` T038. T033-T037 are observations, two gate runs and one gate run respectively,
not commits.

---

## What this slice must not add

Each of these is a decision already answered on the sheet or a rule the model states. Reaching past
one means stopping and asking, not deciding and recording afterwards (constitution, No decision
outside the sheet).

- **No item in `monospace-core`, no arithmetic there, and nothing the core would grow for an
  offset.** Both amounts are signed, as `Delta`'s fields already are, and the addition happens over
  a `Pos` the core already exports (D1; principle VII; plan.md Constitution Check)
- **No second arithmetic in `monospace-diagram`.** `Delta::apply` is `pub(crate)` and stays the only
  place coordinates are added, and it saturates; a resolve that added `dx` and `dy` itself would
  state the saturation rule twice (D1; research.md Q1)
- **No public `Shape::anchor`**, and nothing new reaching past `Position::resolve` — the offset is
  added to what the anchor returns, never asked of it. The direct query is the first consumer that
  needs an anchor without drawing, and the spec says what settles it (Q4; contracts/diagram-api.md
  `Shape` and `Diagram`; spec.md "What this slice does not decide")
- **No displacement that reaches a reference's offsets.** §4 states the destination and the code
  does not walk there: [#143](https://github.com/andresmoschini/monospace/issues/143), and 082's D4
  is the answer not to amend §4 to describe the interim (B3, SC-006; D4, Q4)
- **No way to say "one cell out from that side" without naming the axis**, which the specification's
  own clarification refused in the screen axes so that #143's displacement keeps its one reading:
  [#146](https://github.com/andresmoschini/monospace/issues/146)
- **No `Position` on `Box.at` or `Line.at`**, so no chain of references longer than one link and no
  cycle, and no `at` in the union for anything but a connector's endpoint (ADR-0041; D2)
- **No `add_under`, no way for a file to choose an identity, and no listing of a diagram's
  identities.** `"shape": "#5"` is the identity the reader will issue and nothing more, and §11's
  own trigger has fired and been declined (D3; spec.md "What this slice does not decide")
- **No way to ask which references did not resolve, and no error, report or panic for one that does
  not** — a large offset on a reference to nothing adds to nothing, and the figure holding it is
  simply not drawn ([#88](https://github.com/andresmoschini/monospace/issues/88);
  contracts/description-format.md; ADR-0041)
- **No corner and no center on an `Anchor`**, and no `Anchor` outside the four sides: no offset
  checks itself against the side it is measured from, and a negative amount is simply a point on the
  far side (spec.md "Edge cases"; [#90](https://github.com/andresmoschini/monospace/issues/90))
- **No `Serialize` on `At` or on any description type.** An internally tagged newtype variant
  deserializes and does not serialize, and the file is read and never written; a caller wanting one
  would have to widen the variant (data-model.md "`monospace-cli` — four private mirror types")
- **No amendment to §4, §5, §6 or §11 of `docs/diagram-model.md`**, and no second model document.
  §1's `Delta` row is the single row this slice widens, and it is a clause rather than a rule (D1,
  D4; research.md Q7)
- **No fourth ADR.** The three answers sit inside ADR-0040 and ADR-0041, D2's format is the one
  ADR-0035 keeps provisional, and D3 is a refusal that leaves §11 open; a record for two fields and
  a union would outrank its subject (principle VI; plan.md "Design (part two)")
- **No new dependency, no new crate, and no new gate step** (D1; principle III)
- **No characterization test.** Every rule here is a rule of the model stated in one sentence, which
  is what a contract test is for (spec.md Testing expectations)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: nothing to do.
- **Foundational (Phase 2)**: no dependencies. The baseline is captured before any edit, and the one
  commit that changes no behavior lands before anything depends on it.
- **User Stories 1, 2 and 3 (Phases 3 to 5)**: depend on Phase 2, and on each other only through the
  commit they share. None depends on another.
- **Records (Phase 6)**: depends on Phase 3 — T019's clause is due the moment the arithmetic lands,
  and T020's correction is due the moment the wire refuses a file carrying the old tag. Neither is
  code, and neither is a test.
- **User Story 4 (Phase 7)**: depends on Phases 3 to 5, because the union converts into a `Position`
  that resolves through the arithmetic Phase 3 added. Its own tests are not here; they land in
  Phase 8.
- **User Story 5 (Phase 8)**: depends on Phase 7 — the tenth entry's `to` is a reference, which the
  union has to read, and the `from` it leaves in place is a tagged point.
- **Polish (Phase 9)**: depends on Phase 8.

### Within Each User Story

- US1: T006 and T007 before T008-T011, which read what they produce. T008 and T009 read T006's
  method; T010 reads the same struct through a different file; T011 reads the whole resolve.
- US2: T012-T015, all four reading T006, in any order among themselves.
- US3: T016 and T017 before T018, which confirms the test that already exists still passes.
- US4: T021 before T022-T024, which edit files the new types have to be in for anything to compile;
  T025 and T026 read all three.
- US5: T027 and T028 before T029-T032, which read the file the first edits and the helper the second
  edits.

### Parallel Opportunities

- T003 with T004's gate run and T001's capture — three files to the capture, and T004 is the only
  task that runs the gate.
- T007 with T008 and T009 — `delta.rs` against `position.rs`.
- T008, T009 and T011 in parallel once T006 lands: two in `position.rs`, one in `diagram.rs`.
- T010 alone — it is the only task that moves a snapshot, and `cargo insta review` is a single
  reviewer.
- T012-T015 (US2 tests) in parallel once T006 lands.
- T017 and T018 in parallel with each other; T018 is a confirmation and moves nothing.
- T019 and T020 in parallel — two different files, and neither is code.
- T022, T023 and T024 in parallel once T021 lands — three files, no dependency between them. T025
  and T026 read all three.
- T029-T032 in parallel once T027 and T028 land, in two files.
- T033-T037 in the Polish phase are observations and can be run in any order.

Twelve tasks carry `[P]`. The three behaviors inside plan.md commit 2 cannot be worked in parallel
in practice, because one commit carries them, and T032 depends on a deliberate red that only one
agent should be running.

---

## Implementation Strategy

### MVP First (User Stories 1 to 3, one commit)

1. Phase 2 — the third field exists and nothing reads it; every literal in the workspace sets it to
   zero and the code does not read it (commit 1).
2. Phases 3-5, one commit → an endpoint can stand clear of a side, the gap travels with the side, a
   large offset on a reference to nothing is still nothing, and a displacement still leaves the
   hanging end where it was (commit 2).
3. **STOP and VALIDATE**: `cargo test --workspace` green; `resolve` on a right side with `(2, 0)` is
   `{5, 1}`; the same reference draws what the point draws; displacing the box carries the endpoint
   and the gap does not change; a foreign identity draws nothing however large the offset; a line's
   own middle gets the offset, not the box's old one; 082's displacement test passes unchanged.

The MVP is the arithmetic and nothing else, and saying so is worth more than a checkpoint that
implies the slice is done: the wire, the format and the demonstration are what make the claim
visible to anyone who runs the binary, and a picture nobody can see is half a promise.

### Incremental Delivery

1. Phase 2 → the field arrives, unset (commit 1).
2. Phases 3-5, one commit → an endpoint stands off a side and the gap travels with it (commit 2).
3. Phase 6 → §1's `Delta` row is one clause wider, and 079's contract stops contradicting the code
   (commits 3 and 4).
4. Phase 7 → a description can name a reference, and the eighteen spellings become sixteen tagged
   ones with every picture byte-identical (commit 5).
5. Phase 8 → the shipped demonstration says the same place by the other route, and the picture does
   not move by a character (commit 6).
6. Phase 9 → the gate is green, the one untestable rule is named as such, the count disagreement is
   recorded, and the increment is closed (commit 7).

Each commit follows plan.md: one `refactor` that changes nothing anyone can see, two `feat(diagram)`
and two `feat(cli)` that change the evidence rather than the pictures, three `docs`, and every
commit leaves `cargo xtask check` green (constitution principles II and V).
