---
description:
  "Task list for feature 148: a description names its shapes, and carries the ordinal the next one
  takes"
---

<!-- The feature directory below is the issue title truncated at forty characters by
     `cargo xtask spec`, which landed inside a word: cspell:ignore carri -->

# Tasks: A description names its shapes, and carries the ordinal the next one takes

**Input**: Design documents from `/specs/148-a-description-names-its-shapes-and-carri/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [data-model.md](data-model.md),
[contracts/diagram-api.md](contracts/diagram-api.md),
[contracts/description-format.md](contracts/description-format.md), [research.md](research.md),
[decisions.md](decisions.md), [quickstart.md](quickstart.md)

**Tests**: Requested by the spec's **Testing expectations** — six contract tests, no
characterization, and a new test file is not among them. A behavior rule with no test named against
it is an unfinished spec (constitution, Testing). The two claims this slice accepts with nothing to
verify them are **named as such** in the Polish phase (T040) rather than described as tested.

**Organization**: Tasks are grouped by the spec's behaviors B1 to B4, in the order
[plan.md](plan.md)'s six commits deliver them — that order is forced by constitution principles II
and V rather than chosen, so it is the priority order here too. **B1 and B2 each straddle two
commits**, and the split is recorded per phase rather than resolved by moving a task: the diagram
half lands with `numbered_from` in `feat(diagram)` and the file half with the reader in `feat(cli)`,
because a reader that reads an identity the crate will not accept is a decision the sheet has not
taken.

**Order within a phase**: implementation before tests, where a test cannot exist before what it
reads. That inverts the usual listing, and it is deliberate: in Rust a test that names a field which
is not there does not fail, it does not compile. Task IDs are in execution order throughout.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different file, no dependency on an incomplete task)
- **[Story]**: Which user story this task belongs to (US1, US2, US3, US4)
- Include exact file paths in descriptions

## Path Conventions

Rust cargo workspace. `crates/monospace-diagram/src/` for the diagram library, and
`crates/monospace-cli/src/` and `crates/monospace-cli/tests/` for the CLI. The unit tests for the
diagram crate live in the `#[cfg(test)] mod tests` of the file they cover, as the ones already there
do — the crate has no `tests/` directory and this slice adds no integration test to it. `xtask` does
not change and no gate step is added, so nothing is touched under `xtask/` or `package.json`.

### The mechanical change, counted again

Every number below was measured on 2026-09-30 against this feature's `building` branch, and is what
the tasks carry. `cargo run -q -p xtask -- render --check` prints
`25 generated picture(s) match their descriptions`, and that number is the one every later step is
checked against: **if it changes, a marker stopped being walked and a description is not being
re-drawn at all.**

```text
   25  markers the gate re-draws, 65 shapes across them, in 7 files
   26  entries in crates/monospace-cli/assets/demo.json
   22  shape-level "kind" spellings in 4 Rust files  (description.rs 3, main.rs 3, sweep.rs 5, cli.rs 11)
  ───
  114  id values
```

Per file, and the `next_id` each description takes — which is **one past the highest `#N` that
description writes**, and nothing more, because every entry in this repository keeps writing `#1`,
`#2`, … in the order it already lists its entries (spec.md Clarification, Q3):

| File                                                          | Markers | Shapes | Per marker                                                        |
| ------------------------------------------------------------- | ------- | ------ | ----------------------------------------------------------------- |
| `docs/diagram-demo.md`                                        | 12      | 38     | 3, 5, 2, 2, 2, 4, 5, 5, 4, 2, 2, 2                                |
| `docs/diagram-model.md`                                       | 3       | 6      | 2, 2, 2                                                           |
| `README.md`                                                   | 1       | 6      | 6                                                                 |
| `specs/055-an-arrow-end-is-a-glyph-arm-or-nothing/spec.md`    | 2       | 6      | 3, 3                                                              |
| `specs/081-a-shape-can-be-removed-and-replaced/decisions.md`  | 4       | 6      | 2, 2, 1, 1                                                        |
| `specs/081-a-shape-can-be-removed-and-replaced/data-model.md` | 2       | 2      | 1, 1                                                              |
| `docs/model.md`                                               | 1       | 1      | 1                                                                 |
| `CONTRIBUTING.md`                                             | 1       | 1      | 1 — inside a ````markdown` fence, so **no gate step re-draws it** |

Two markers in the tree carry no description the gate reaches, and neither is edited except the
second: `.specify/templates/decisions-template.md` holds one inside a
````markdown`fence with`"shapes": [ … ]` in it, so there is nothing to re-spell.

**43 descriptions carry a `next_id`**, not 44: 25 markers, `CONTRIBUTING.md`'s one,
`assets/demo.json`, and 16 in Rust. The seventeenth `"canvas"` in Rust is
`crates/monospace-cli/tests/cli.rs:411`, a string **truncated on purpose** so the parser refuses it,
and it gets neither field — T006 is where that is spelled out.

The baseline `cargo test --workspace` is green at **23 / 18 / 114 / 62 / 15 / 61** (`monospace-cli`
unit, `monospace-cli` integration, `monospace-core`, `monospace-diagram`, `monospace-glyph-sets`,
`xtask`), and T001 records it.

---

## Phase 1: Setup

No setup is needed: no dependency is added, no crate is added, the toolchain is unchanged, and no
gate step is added — `wasm` already names `monospace-diagram` (plan.md Constitution Check, principle
III). T001 captures what must not move before anything is edited.

---

## Phase 2: Foundational — the identities land and the reader does not read them

**Purpose**: 114 `id` values and 43 `next_id` values land in every description in the repository,
and the reader **drops them in silence** — measured, three ways, in research.md's opening section.
Nothing here may touch a test and nothing here may change a picture.

- [x] T001 Run `cargo run -p monospace-cli > /tmp/demo-before.txt` and
      `cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-before.txt`, and
      keep both files until Phase 6. Then run `cargo run -q -p xtask -- render --check`, which must
      print `25 generated picture(s) match their descriptions`, and `cargo test --workspace`, which
      must be green at **23 / 18 / 114 / 62 / 15 / 61**. Read off the two numbers B2.3 rests on: the
      first entry of `crates/monospace-cli/assets/demo.json` is a four-by-three box at the origin,
      the third is a four-by-three box at `{9, 2}`, and the tenth is the connector whose `▲`
      terminal stands at `(22, 4)`. Those are the figures `crates/monospace-cli/src/main.rs:111`,
      `:125` and `:126` name by hand as `#1`, `#3` and `#10`
- [x] T002 [P] In `crates/monospace-cli/assets/demo.json`, add `"id": "#1"` … `"id": "#26"` to the
      twenty-six entries in the order they are already listed, and `"next_id": 27` beside
      `"canvas"`. **Change nothing else in that file** — no `at`, no `size`, no stroke — because the
      tenth entry's `"shape": "#5"` is what names the box at `{20, 1}` today and must keep naming it
      afterwards, and an `id` assigned in array order is what makes that true rather than lucky (D1,
      D4; spec.md B2.3; data-model.md "The demonstration"; contracts/description-format.md "What has
      to be re-spelled")
- [x] T003 [P] In `crates/monospace-cli/src/description.rs`, add `"next_id": N` to the **four**
      description literals at lines 355, 369, 385 and 414, and `"id": "#1"` to the **three** shape
      literals at lines 371, 387 and 416 — beside `kind`, which is where every entry in this
      repository already puts its first field (Q3). Line 355's entry is `{ "kind": "triangle" }` and
      gets **no** `id`: the enum is internally tagged, so the tag is read first and the entry is
      refused before any field of it is looked at, and an identity on an object the format has no
      shape for would claim a shape the test says does not exist. Line 414 is the `format!` template
      `description_of` builds, so one `next_id` and one `id` there cover `ARROW`,
      `ARROW_WITHOUT_A_TERMINAL` and `ARROW_WITH_AN_AT` at once — **six** `next_id` values would be
      six edits where one site is the whole of it (Q3; research.md Q4; AGENTS.md "Facts that are in
      the code and in no document")
- [x] T004 [P] In `crates/monospace-cli/src/main.rs`, add `"next_id": 3` to the two-box description
      at line 318, `"next_id": 2` to `one_box_json()` at line 232 and to the empty description at
      line 611, and `"id": "#1"` … `"#2"` to the three shape literals at lines 234, 320 and 322. The
      empty description gets a `next_id` and **no** `id`: it lists no entries, and
      `an_empty_description_demonstrates_as_five_identical_pictures` is one of the tests that must
      pass unchanged through this whole slice (B5.7)
- [x] T005 [P] In `crates/monospace-cli/src/sweep.rs`, add `"next_id": 3` to the `description_of`
      template at line 68 and `"id": "#1"` to the three shape templates at lines 133, 137 and 141,
      and `"id": "#1"` / `"id": "#2"` to the crossing pair at lines 189 and 191. `3` and not `2`
      because `description_of` builds **one** header for both families and the crossing pair carries
      two identities: `next_id` has to be past the highest `#N` any case reaches, or the sweep's own
      file disagrees with itself. **This is five edits and not 1856** — the cases are built as text
      from three templates and one crossing pair — and a snapshot pins a rendering rather than a
      description, so **none of the eight files under
      `crates/monospace-cli/src/snapshots/characterization/` may move** and no ADR-0053 report is
      owed (plan.md "What the mechanical change actually measures"; quickstart.md "Commit 1")
- [x] T006 [P] In `crates/monospace-cli/tests/cli.rs`, add `"next_id": N` to the **eight** real
      description literals — lines 106 (`2`), 131 (`5`), 176 (`3`), 254 (`2`), 328 (`3`), 340 (`3`),
      372 (`2`) and 427 (`2`) — and `"id"` to the **eleven** shape literals at lines 108, 133, 135,
      137, 163, 179, 330, 332, 342, 344 and 374. Two of those need care. Line 163 is `A_BOX`, a bare
      entry spliced into two descriptions, so it carries `"id": "#1"` and **no** `next_id`; the
      description at 176 lists it beside the connector at 179, which is therefore `#2` and not `#1`.
      And line 427 is the `{ "kind": "triangle" }` file, which gets a `next_id` and **no** `id`, for
      the reason T003 gives. **Line 411 is not touched at all**: it is
      `write_description("broken", r#"{ "canvas": "#)`, a string truncated on purpose so
      `malformed_json_locates_the_problem_on_stderr_and_fails` fails for its own reason, and it is
      the seventeenth `"canvas"` in the tree that no `next_id` is written beside (Q3; B4.3)
- [x] T007 [P] In `docs/diagram-demo.md`, add `"id": "#1"` … to all **38** entries across its **12**
      markers, in the order each marker already lists them, and a `"next_id"` to each of the twelve
      beside `"canvas"`: **4, 6, 3, 3, 3, 5, 6, 6, 5, 3, 3, 3** in the order the markers appear.
      This is the largest single file in the change and the one the gate re-draws most often; take
      the twelve counts from the markers rather than from this line, and if one comes out different,
      write down what it came out as and stop (D4; research.md Q4)
- [x] T008 [P] In `docs/diagram-model.md`, add `"id"` to the **6** entries across its **3** markers
      and `"next_id": 3` to each of the three, and change **nothing else in the file**. The three
      sections this slice amends — §3, §9 and §11 — land in Phase 7 and not here, and a mechanical
      commit that also rewrites prose is a commit nobody can review (D4; plan.md commit 4)
- [x] T009 [P] In `README.md`, add `"id": "#1"` … `"id": "#6"` to the **6** entries of its one
      marker and `"next_id": 7` beside `"canvas"`, and change nothing else
- [x] T010 [P] In `docs/model.md`, add `"id": "#1"` to the **1** entry of its one marker and
      `"next_id": 2` beside `"canvas"`. The entry is written as `"shapes": [ { "kind": "connector",`
      on one line rather than `{ "kind"` at the start of a line, which is why a grep for
      `^ *{ "kind"` misses it and this file's count does not reproduce that way
- [x] T011 [P] In `CONTRIBUTING.md`, add `"id": "#1"` to the **1** entry inside the
      ````markdown`fence and`"next_id": 2`beside`"canvas"`. **No gate step will notice whether this
      is right or wrong**, because that marker is shown inside a fence as the grammar of a marker
      and the walker steps over it — so read it as the working example a reader copies rather than
      as a sample, and treat "the gate is green" as no evidence at all about this one (research.md
      Q4; AGENTS.md "Facts that are in the code and in no document")
- [x] T012 [P] In `specs/055-an-arrow-end-is-a-glyph-arm-or-nothing/spec.md`, add `"id"` to the
      **6** entries across its **2** markers and `"next_id": 4` to each of the two, in the order the
      markers appear
- [x] T013 [P] In `specs/081-a-shape-can-be-removed-and-replaced/data-model.md`, add `"id"` to the
      **2** entries across its **2** markers and `"next_id": 2` to each of the two
- [x] T014 [P] In `specs/081-a-shape-can-be-removed-and-replaced/decisions.md`, add `"id"` to the
      **6** entries across its **4** markers and a `"next_id"` to each of the four: **3, 3, 2, 2**
      in the order the markers appear. This is the file a decision sheet is written from, so the
      `id` values it carries are read by the next feature's own sheet
- [x] T015 Run `cargo run -q -p xtask -- render` and then `cargo run -q -p xtask -- render --check`,
      which rewrites and then re-checks every `<!-- render: -->` block from the description beside
      it — so **24 of the 25** are covered for free, and a description edited wrongly enough to stop
      parsing is a red `render` step rather than a picture that silently went blank. The count must
      still be **25**; then run `cargo run -p monospace-cli > /tmp/demo-after-1.txt` and
      `diff /tmp/demo-before.txt /tmp/demo-after-1.txt`, followed by
      `cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-after-1.txt` and
      `diff /tmp/file-before.txt /tmp/file-after-1.txt`, and **both diffs must print nothing**.
      **What this step does not catch, and what it is worth saying out loud**: an `id` given the
      **wrong** name changes nothing at all, because the reader drops the field and a marker draws
      the picture its entries describe. What turns this step red is a description that no longer
      parses, not a misnamed shape (constitution principle IV; quickstart.md "Commit 1")
- [x] T016 Deliberate red, and it belongs to **this** commit: comment out the renumbering loop in
      `demo_without_its_first_entry` — the `for shape in shapes.iter_mut()` that rewrites
      `shape["to"]["at"]["shape"]` from `"#5"` to `"#4"` — and run
      `cargo test -p monospace-cli the_third_picture_moves_one_figure_and_the_fourth_takes_that_figure_out`,
      which must now **fail**. Then put the loop back and watch it pass again. This is the check
      that the commit changed nothing: the file now carries `"id": "#5"` on the box at `{20, 1}` and
      the reader is still throwing it away, so `shapes.remove(0)` still shifts every name after it
      and the arrowhead still lands at `{20, 3}` instead of `{22, 4}`. **A reader that had already
      started reading `id` would pass this**, which is exactly what would make commit 1 a `feat`
      wearing a `refactor`'s prefix (constitution principles IV and V; quickstart.md "Commit 1")
- [x] T017 Confirm `cargo test --workspace` is green at **23 / 18 / 114 / 62 / 15 / 61** with **no
      test added and no test changed** — a structural commit that moves a test is a behavioral one
      wearing the other commit type's prefix — that `cargo run -q -p xtask -- render --check` still
      prints 25, and that `cargo insta review` is **not needed**: no file under
      `crates/monospace-cli/src/snapshots/characterization/` may have moved, and one that did is a
      rendering that changed rather than a description (constitution principles IV and V; plan.md
      commit 1)

**Checkpoint**: every description in the repository names its shapes and says where its numbering
resumes, and the reader has not noticed.

**Commit**: `refactor(cli):` T002-T014, one commit for all 114 identities across 13 files (plan.md
commit 1). T001, T015, T016 and T017 are a captured pair of files, a gate run and one deliberate
red-then-green experiment, not commits.

---

## Phase 3: User Story 1 - A caller may choose the identity a shape is added under (B1, Priority: P1)

**Goal**: a shape can be put under a name its caller chose, and every change that names an identity
answers to that name — and, on the file side, an `id` reaches the diagram, so a reference names the
shape the file names rather than the place it is written (spec.md B1.1, B1.2, B1.3; SC-001, SC-002).

**Independent Test**: on the diagram, `get`, `remove`, `replace`, `forward` and `backward` each find
a shape added under a chosen identity, and an identity the file holds for a **different** shape
changes nothing. Through a file, two descriptions differing only in the order of two non-overlapping
entries draw the same buffer when the reference names the box, while B1.1's pair — the reference
naming the **place** — draws what it draws today.

The method, exactly as `contracts/diagram-api.md` spells it:

```rust
/// Puts `shape` at the front of the order, under the identity `id`.
pub fn add_under(&mut self, id: ShapeId, shape: Shape) {
    self.shapes.push(Placed { id, shape });
}
```

It is `add` with the caller's name in place of the diagram's, it hands back nothing, and it **does
not touch `next`** — which is D2's accepted cost in one sentence (D2, Q1, Q2, Q3).

### Implementation for User Story 1 — the diagram half, `feat(diagram)`

- [x] T018 [US1] In `crates/monospace-diagram/src/diagram.rs`, rewrite the rustdoc on `ShapeId`,
      whose first line reads "normally generated by the diagram that placed it and **unique within
      it** (FR-001, FR-002)". Say instead that the identities **the diagram issues** are unique
      within it, and that an identity supplied by a caller through `add_under` is **not checked** —
      two shapes may carry one, and the second is a shape nobody can name. Change **no** field,
      **no** derive and **no** signature: `ShapeId` is the same type with a wider meaning, and what
      changes is the sentence the model carried. This is the one sentence §3 is amended twice to
      match, so it is written here rather than in the `docs` commit that follows the code (D2;
      data-model.md "`ShapeId` — same type, wider meaning"; plan.md commit 2)
- [x] T019 [US1] Add `add_under` to `impl Diagram` in `crates/monospace-diagram/src/diagram.rs`,
      immediately after `add`, with the rustdoc `contracts/diagram-api.md` gives it. It pushes one
      more `Placed` onto the back of `shapes` — the back is the front of the order, which is what
      `add` does too — takes the identity by value, and hands back nothing: a `Result` and an
      `Option` each have no case to report, because a repeated identity is **accepted** rather than
      refused. It must **not** touch `self.next`. `add`, `new`, `find`, `get`, `remove`, `replace`,
      `forward`, `backward` and `draw` keep their signatures and their bodies (D2, Q2; B1.3;
      data-model.md "`add_under` — new, public")
- [x] T020 [P] [US1] Contract test: **a chosen identity is found by that identity and by no other
      one**, in `crates/monospace-diagram/src/diagram.rs`, beside the tests already there. `get`,
      `remove`, `replace`, `forward` and `backward` each name the shape they were given, and an
      identity the diagram holds for a **different** shape changes nothing. The second half is the
      one that is easy to leave out and the one that keeps "unique" a claim about what the diagram
      issues rather than a promise it keeps — build the expected picture through the `draw_of` and
      `cells` helpers already in that module, and pin a `remove` by what it left behind rather than
      by a count (B1.3, SC-002; quickstart.md "Commit 2")
- [x] T021 [P] [US1] Contract test: `add_under` **does not move the counter** and **does not check
      the name**, in `crates/monospace-diagram/src/diagram.rs`. On a diagram numbered from 3,
      `add_under(ShapeId::new("#7"), …)` leaves the next `add` handing back `#4`, and two shapes put
      under one identity are both held with `get` returning the first. The second half is D2's cost
      pinned as a behavior rather than as a bug, so a later slice deciding to report it has to say
      so rather than discover it (D2; B3.1, SC-004; data-model.md "What the tests pin")

### Implementation for User Story 1 — the file half, `feat(cli)`

The whole of the conversion, as [data-model.md](data-model.md) gives it:

```rust
pub(crate) fn into_diagram(self) -> Diagram {
    let mut diagram = Diagram::numbered_from(self.next_id);
    for shape in self.shapes {
        diagram.add_under(ShapeId::new(shape.id), shape.shape.into());
    }
    diagram
}
```

- [x] T022 [US1] In `crates/monospace-cli/src/description.rs`, make the two fields required and
      rewrite the conversion to the form above. `Description` gains `next_id: u32` beside `canvas`
      and `shapes`; each of `ShapeDescription`’s three variants gains `id: String`, which means
      adding the field to three variants rather than to the enum, and the variant fields are read in
      declaration order so the wire order follows (D1, D3; Q3). **The order is still the order**:
      the array order remains the drawing order, so every picture in the repository comes out byte
      for byte what it did, and that is the claim the whole mechanical change rests on. This is a
      `feat` and **it cannot land before Phase 2**, because it is the commit that refuses all 43
      descriptions until they carry both fields (D1, D3; B1.1, B1.2; plan.md commit 3)

- [x] T023 [P] [US1] Contract test: **a reference follows the name, both directions**, in
      `crates/monospace-cli/src/description.rs`. Two descriptions over the same window holding the
      same two boxes and the same connector with their entries in opposite orders and the reference
      naming the box draw the **same** buffer; and B1.1's pair — the entries in both orders with the
      reference naming the **place**, which is what those files mean today — draws what it draws
      today, which is two different pictures. **Both directions, because a slice that made the name
      win in one and not the other would pass a single test.** Build both sides through
      `crate::glyph_catalog()` and `monospace_core::render` over a `Buffer`, which is the same path
      `render_once` takes, and compare the two by rendered string rather than by cells (B1.1, B1.2;
      SC-001; quickstart.md "B1.1 and B1.2 — the name beats the place, both directions")

**Checkpoint**: a caller may name a shape, every change answers to that name, and a file's `id`
reaches the diagram.

---

## Phase 4: User Story 2 - The ordinal the next shape takes survives being read (B2, Priority: P1)

**Goal**: a diagram may be told where its numbering resumes, and the ordinal is **the ordinal the
next `add` takes** rather than the count of what was read. A file naming `#1`, `#7` and `#9` with
`next_id: 10` hands back `#10` next, and the gaps are simply unused (spec.md B2.1, B2.2, B2.3;
SC-003).

**Independent Test**: `numbered_from(3)` then `add` hands back `#3`, asked by the identity's own
text. A diagram holding `#1`, `#2` and `#3` hands back a fourth identity no shape holds and a fifth
different one after that; an identity freed by a `remove` is never handed out again; and `add_under`
between two additions does not change what the second one gets.

**Storing the ordinal itself is the whole design of this method**, and the trap it removes is the
thing to check: a counter holding the **last issued** ordinal would need `numbered_from(27)` to
store `26`, and an off-by-one in a public seeding method is the kind nothing in the signature
catches (Q1).

```rust
/// An empty diagram whose next `add` takes the ordinal `next`.
#[must_use]
pub fn numbered_from(next: u32) -> Self {
    Self { next, ..Self::default() }
}
```

- [x] T024 [US2] Add `numbered_from` to `impl Diagram` in `crates/monospace-diagram/src/diagram.rs`,
      between `new` and `add`, with the rustdoc `contracts/diagram-api.md` gives it, and store the
      **ordinal itself** in the existing `next: u32` — no second field, and **no** subtraction
      anywhere. `add` already increments **before** use
      (`crates/monospace-diagram/src/diagram.rs:57`), which is what makes `new()` and
      `numbered_from(1)` agree in everything observable: `add` on either hands back `#1`. `Default`
      still derives and `new()` still delegates to it, so a caller with no opinion keeps writing
      `new` (D1; Q1; data-model.md "`numbered_from` — new, public")
- [x] T025 [P] [US2] Contract test: **the ordinal, asked rather than drawn**, in
      `crates/monospace-diagram/src/diagram.rs`. `numbered_from(3)` then `add` hands back `"#3"`; a
      diagram holding `#1`, `#2` and `#3` hands back a fourth identity no shape holds and a fifth
      different one; and `an_identity_is_never_handed_out_again_after_a_removal` — 081's test, which
      is already in that file — passes **unchanged** and is also run against a diagram seeded at 10,
      where taking `#11` out still leaves the next addition as `#13`. Assert on the identity's own
      text rather than on a picture: this is the one rule in the slice no picture can show (B2.1,
      B2.2, SC-003; quickstart.md "Commit 2"; data-model.md "What the tests pin")
- [x] T026 [P] [US2] Check B2.3 rather than write code for it: the three identities
      `crates/monospace-cli/src/main.rs:111`, `:125` and `:126` name by hand — `#1`, `#3` and `#10`
      — must be the same three figures they name today, which T002 guarantees by writing
      `"id": "#1"`, `"id": "#3"` and `"id": "#10"` on the very entries `assets/demo.json` lists
      first, third and tenth. Confirm it by reading the fifth picture of
      `cargo run -p monospace-cli`: the arrow is still hanging from the box at `{9, 2}` and that box
      is still displaced four cells right. **Nothing in `crates/monospace-cli/src/main.rs` changes
      here** — `demonstrate` writes the three identities out by hand rather than reading them back,
      and reading them back would need a listing a diagram deliberately does not offer (Q5; B2.3;
      data-model.md "The demonstration")

**Checkpoint**: a diagram can be told where its numbering resumes, and the number is the one the
next addition takes.

---

## Phase 5: User Story 3 - A file that does not agree with itself (B3, Priority: P1)

**Goal**: the two refusals and the two accepted costs. A missing `id` or a missing `next_id` is
**refused by name**, exactly as `canvas`, `shapes`, `leaving`, `terminal` and `at`'s `kind` already
are. A **repeated** `id` is not refused, a **reference to nothing** is not refused, and neither
errors, reports or panics (spec.md B3.1, B3.2, B3.3; SC-004).

**Independent Test**: two files, one missing `next_id` and one missing an entry's `id`, each report
``missing field `next_id` `` and ``missing field `id` `` on stderr with nothing on stdout and a
failure status. A file whose entries are named `right`, `left` and `arrow` draws byte for byte what
the ordinal-named file beside it draws. A file with two entries carrying `"id": "#1"` draws both,
exits successfully, and nothing warns.

**The asymmetry is the format's existing one and not something this slice adds**: a **misspelled**
`id` is caught, because a required field absent is a `missing field` error, while an unknown key
beside a valid entry is dropped in silence as every unknown field in this format has always been
(Q3; contracts/description-format.md "What a malformed file does").

- [x] T027 [P] [US3] Contract test: **a missing identity is refused by name**, in
      `crates/monospace-cli/src/description.rs`, beside the refusals already there. A description
      with no `next_id` reports ``missing field `next_id` `` and one whose box carries no `id`
      reports ``missing field `id` ``, each asserted on the **message** and not on the line and
      column that follow it, which are `serde`'s and move with the bytes. The
      `display_with_line_column` half of the claim is the rest of the format's existing behavior and
      is already pinned by `an_omitted_terminal_field_is_refused_by_name`; this test pins the two
      new names beside it (B3.2; Q3; contracts/description-format.md)
- [x] T028 [P] [US3] Contract test: **free text reads**, in
      `crates/monospace-cli/src/description.rs`. A description whose entries are named `right`,
      `left` and `arrow`, its connector naming `arrow`, draws **byte for byte** what the same
      description named `#1`, `#2` and `#3` draws — and its `next_id` is untouched by the
      substitution, which is what D1 chose over deriving an ordinal from the names. A format that
      insisted on an ordinal could not pass this test (B3.3, SC-001; data-model.md "Free text
      reads")
- [x] T029 [P] [US3] Contract test: **a repeated identity, the file half**, in
      `crates/monospace-cli/tests/cli.rs`. Two entries carrying `"id": "#1"` are **both** read, both
      shapes are drawn, the run exits successfully with nothing on stderr, and the file is not
      refused. The reference-resolves-through-the-first half is T021's at the diagram level; this is
      the same cost arriving through a wire, where a reader might plausibly have checked it and did
      not. **Pinned as an accepted cost, not as a bug**, so a later slice that decides to report it
      has to say so rather than discover it (B3.1; Q3; D2)
- [x] T030 [P] [US3] Confirm
      `a_file_naming_a_shape_it_does_not_hold_draws_the_box_and_no_connector_and_succeeds` in
      `crates/monospace-cli/tests/cli.rs` passes **unchanged** and record that it passed before the
      slice as well as after it. What changes underneath is what its `"shape": "#7"` names: a place
      before this slice, and afterwards an `id` no entry carries. The picture is the same either way
      — the box, no connector, a successful exit — and that is
      [ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md)'s silent
      hole arriving on a name rather than on a number, which is what B3.3 and
      [#88](https://github.com/andresmoschini/monospace/issues/88) record (B3.3, SC-004)

**Checkpoint**: a file that names its shapes is read, and a file that does not is refused by the
name of what it left out.

---

## Phase 6: User Story 4 - The demonstration keeps its five pictures (B4, Priority: P1)

**Goal**: a bare run of the application prints the same five captioned pictures it prints today,
character for character, and a path prints one picture and nothing else. **The evidence for this
slice is in the file and the pictures not moving is what proves it** (spec.md B4.1, B4.2; SC-005).

**Independent Test**: `diff /tmp/demo-before.txt /tmp/demo-after.txt` prints nothing at all, and so
does the same diff for the single-picture run over `crates/monospace-cli/assets/demo.json`.

**The workaround that goes**: `demo_without_its_first_entry` renumbers `"#5"` to `"#4"` because a
reference names a **place** in a list, so deleting the first listing shifts every name after it.
With an `id` on every entry the remaining twenty-five keep the names they were written with, the
loop has nothing left to do, and the same `assert_eq!` becomes SC-001's claim rather than a fixture.

```text
   before this slice                     after this slice
   ────────────────                      ───────────────
   shapes.remove(0)                      shapes.remove(0)
   for shape in shapes.iter_mut() {      diagram
     if … "#5" { … "#4" }                  .add_under(ShapeId::new("left"), left)
   }                                      .add_under(ShapeId::new("right"), right)
                                          .add_under(ShapeId::new("arrow"), arrow)
   "remove(0) shifts every name          "remove(0) shifts nothing:
    after it, and the loop undoes it"     every survivor kept its name"
```

- [x] T031 [US4] In `demo_without_its_first_entry` in `crates/monospace-cli/src/main.rs`, delete the
      renumbering loop — the `for shape in shapes.iter_mut()` that rewrites
      `shape["to"]["at"]["shape"]` from `"#5"` to `"#4"` — and the paragraph of its doc comment that
      explains it, which reads "**It also renumbers the one reference it moves, `"#5"` to `"#4"`,
      and that is not a convenience — it is the whole of what a positional identity costs**" and
      runs to the end of the comment. **Keep** the paragraphs above it: the helper is still rebuilt
      through `serde_json` rather than by hand so the description the test draws is the one the
      binary embeds, and only the first entry differs. Say in the commit message that this is the
      cost this slice removes rather than a fix to the helper (Q5; D3; data-model.md "The
      demonstration")
- [x] T032 [US4] Correct the two comments in `crates/monospace-cli/src/main.rs` that T022 makes
      false, in the same commit that makes them false (083's Q6, 082's Q7). The one at **line 104**
      reads "a description names its shapes **by position**, so the demonstration already knows
      which entry it means and has nothing to read", and the one at **line 121** reads the same
      sentence a second time for `#3` and `#10`. Both must now say a description names its shapes
      **by the identity it wrote**, and keep the rest: the identities are still written out at each
      call rather than read back, and `get` still offers no listing to read them from. A rustdoc is
      code, so it is corrected where it becomes wrong rather than in a later `docs` commit (Q5, Q6;
      plan.md commit 3)
- [x] T033 [P] [US4] Contract test: the demonstration prints the **five** pictures it prints today,
      in `crates/monospace-cli/src/main.rs`, found by splitting the output on the blank line and
      pinning no caption's wording, and `cargo run -p monospace-cli <path>` still prints one picture
      and nothing else, which is what `cargo run -p xtask -- render` embeds. Most of this is already
      pinned by
      `a_bare_run_prints_five_captioned_pictures_the_first_being_the_description_as_written` and
      `the_tenth_entry_naming_a_reference_leaves_all_five_pictures_exactly_as_they_were`; what this
      task adds is the **other side** of
      `the_third_picture_moves_one_figure_and_the_fourth_takes_that_figure_out`, which now passes
      because the reader reads `id` rather than because a loop undoes the shift — so assert it
      against the shipped description with its first entry left out, **without** any renumbering,
      and say in the test's doc comment that this assertion is SC-001 rather than a fixture (B4.1,
      B4.2, SC-001, SC-005; data-model.md "What the tests pin")
- [x] T034 [P] [US4] The three diffs, run in one place:
      `cargo run -p monospace-cli > /tmp/demo-after.txt` against `/tmp/demo-before.txt`,
      `cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-after.txt`
      against `/tmp/file-before.txt`, and `cargo run -q -p xtask -- render --check` — **all three
      must be silent, and the third must still print 25**. Three different things can go wrong and
      two of them draw the same picture as correct code, so this is where the demonstration is
      checked rather than read. The demonstration is the one place in the slice where a regression
      shows up as a picture rather than as a failed test (B4.1, B4.2, SC-005; quickstart.md "Commit
      3")

**Checkpoint**: a person who runs the application sees five pictures identical to the ones they saw
before, and the file those pictures came from now names its shapes instead of counting them.

**Commit**: `feat(cli):` T022, T027-T030, T031-T033, ticking their checkboxes with it (plan.md
commit 3). T026 and T034 are a read of the fifth picture and three diffs, not commits.

---

## Phase 7: Records this slice amends

**Purpose**: three `docs` commits, none of them a behavior, and so none carrying a `[Story]` label.
They are placed here rather than in the Polish phase because plan.md orders them between the wire
and the ADR, and the first is due the moment the reader refuses a description missing both fields.

- [x] T035 Amend §3 _Identity_ in `docs/diagram-model.md`, **twice, in the same paragraph**. The
      section reads "The diagram generates one when the shape is added — `#1`, `#2`, and so on — and
      it is **unique within that diagram**", and "Identities being chosen by the caller, or edited
      after the fact, is an open question below". The first amendment says a caller may choose the
      identity a shape is added under; the second narrows "unique" to the identities **the diagram
      issues** rather than a promise it keeps on a caller's behalf, because a caller-supplied one is
      not checked. Rewrite the third paragraph so it says the naming half is answered here and stops
      naming a trigger that has already fired. **Left alone: editing one after the fact**, which
      §3's own second paragraph keeps out (D2; spec.md "What this slice implements"; §11's _Can a
      caller choose an identity?)
- [x] T036 In `docs/diagram-model.md`, amend **two** more places and nothing else. §9 _Changing a
      diagram_'s `add` row reads "Puts a shape at the front of the order and gives it an identity",
      and it gains which identity it gives and that it is never one the diagram has handed out
      before. §11's first question — _Can a caller choose an identity?_ — comes out, and its
      surviving half, editing one, is rewritten in place rather than left naming a trigger that has
      fired. **Amend no other section**: §4 already says a reference resolves by asking for the
      anchor and adding the offsets, and §1's `Reference` row has named three fields all along (D2;
      spec.md "What this slice implements"; plan.md commit 4)
- [ ] T037 [P] Write
      `docs/decisions/0066-let-a-caller-choose-an-identity-and-say-where-the-numbering-resumes.md`
      from `docs/decisions/adr-template.md`, at **`load-bearing`** — by the constitution's own test,
      `monospace-cli` calls both new methods, so something outside the module depends on it, and
      reversing it means 114 identities leaving 43 descriptions. It holds D1, D2 and D3, names the
      alternatives each of them declined, and **carries no picture**: research.md Q6 measured that
      every option pair on the sheet draws the same picture, and everything observable about this
      decision is a **value** rather than a drawing — `numbered_from` shows itself only in what
      `add` hands back, `add_under` not moving the counter is invisible in any picture, two entries
      under one identity draw exactly what two entries under two draw, and free text draws the
      ordinal-named picture. Say that on the spot, so the absence is a finding and not an oversight,
      which is what plan.md does for the other three artifacts (constitution principle VI; Q6;
      ADR-0064)
- [ ] T038 [P] Add the row for ADR-0066 to `docs/decisions/README.md` **as a whole row** — link,
      title and status written out together. A partial edit to that table leaves the rest of the row
      on the line below, and prettier then reflows the damage rather than rejecting it (AGENTS.md
      "Facts that are in the code and in no document")

**Commit**: `docs(model):` T035 and T036 together, then `docs(adr):` T037 and T038 (plan.md commits
4 and 5).

---

## Phase 8: Polish & Cross-Cutting Concerns

- [ ] T039 Run `cargo xtask check` and confirm it is green, including the `wasm` step — which
      already compiles `monospace-core`, `monospace-diagram` and `monospace-glyph-sets`, so it
      covers both new methods with no change to `xtask` and no new check, which is why principle
      III's two-commit rule does not apply. Watch the output rather than the exit code: `rustfmt`
      reports that `group_imports` needs nightly and exits 0, and so does a step that finds
      something it cannot fix. The `render` step must still say **25** and the `numbering` step must
      still be green, which is where `0066` shows as taken rather than free (SC-007; plan.md
      Constitution Check, principle III)
- [ ] T040 Confirm the two rules this slice **accepts with nothing to verify them** are named as
      such rather than described as tested: that a stale `next_id` — an entry renamed, a `#2`
      deleted — hands back an identity already in use, so the shape that arrives is one nobody can
      name; and that a shape held under an identity a second entry also carries is simply a shape
      nobody can name. Look for the first in `docs/decisions/0066-*.md`, §3's amendment and
      `contracts/description-format.md`, and the second in T021, T029 and the specification's _Edge
      cases_, and leave each where it is — **this task is a check that the record says so, not an
      edit** (constitution principle IV; D2; spec.md Testing expectations)
- [x] T041 Make a **misspelled** `id` deliberate, once, by hand: change one entry's `"id"` to
      `"idd"` in `crates/monospace-cli/assets/demo.json`, run `cargo run -p monospace-cli`, and
      watch what happens. A required field that is not there is a `missing field` error, while an
      unknown key beside a valid entry is dropped in silence — which is why Phase 2's edit is an
      **addition** and not a rename, and why a future editor may spell any key wrongly without
      noticing. Then restore it and confirm `cargo run -q -p xtask -- render --check` still
      prints 25. Record both halves in the commit message rather than in a document (Q3;
      constitution principle IV)
- [ ] T042 Take the count of what Phase 2 actually wrote and check it against what the tasks said:
      114 `id` values across 13 files and 43 `next_id` values, with
      `crates/monospace-cli/tests/cli.rs:411`'s truncated string carrying neither. The count is a
      `grep`, not an estimate, and if it comes out as something other than 114 or 43, write down
      what it came out as rather than adjusting either number to make the other fit (constitution
      principle IV)
- [ ] T043 Append an entry to `docs/learning-log.md` for this increment: what was learned about Rust
      design and idiom, what was learned about working this way, and optionally a trade-off worth
      remembering. Three are already paid for and worth writing down rather than rediscovering — an
      identity derived from a **position in a list** makes any test helper that edits the list a
      place where every name after the edit shifts, and the loop that undid it was in the suite
      rather than in the code; a required field the reader does not read yet is what lets a
      114-value mechanical change land as one `refactor`, and a misspelled **extra** key staying
      silent is why it has to be an addition; and a counter that means _the ordinal the next `add`
      takes_ rather than _the last one issued_ is the difference between a public seeding method
      that reads correctly and one carrying an off-by-one (constitution principle II)

**Commit**: `docs:` T043. T039-T042 are a gate run and three observations, not commits.

---

## What this slice must not add

Each of these is a decision already answered on the sheet or a rule the model states. Reaching past
one means stopping and asking, not deciding and recording afterwards (constitution, No decision
outside the sheet).

- **No check that a caller-supplied identity is unused, and no `Result` or `Option` on
  `add_under`.** Two shapes may share one and both are held; the repair is one line, and the
  specification names it as a deliberate gap rather than a defect (D2; B3.1; Q2)
- **No counter that advances past any `#N` a caller writes.** `add_under` does not touch `next`, and
  deriving the ordinal from the identities instead of carrying it is the alternative D1 declined — a
  `#N` convention inside a crate whose `ShapeId` is any string (D1, D2; Q2; research.md Q2)
- **No way to ask a diagram what identities it holds**: no `ids()`, no `len`, no order. `get` stays
  the crate's only reader, and `add_under` does not become a listing for a consumer that does not
  exist (spec.md "What this slice does not decide"; Q5)
- **No way to edit an identity after the fact**, and no second model document. §11's open question
  is half this and half the naming above, and only the naming half fires; the surviving half stays,
  with editing as its trigger (spec.md "What this slice does not decide"; §11)
- **No report on an identity that resolved to nothing.** ADR-0041 accepts the silent hole and
  [#88](https://github.com/andresmoschini/monospace/issues/88) is where it is written down; a name
  in a file is a second way into the same hole and not a second kind of it (B3.3; spec.md "What this
  slice does not decide")
- **No `Serialize` anywhere in this path**, and no `ids: { next: 27 }`, no `"name"` for the field
  and no optional `next_id`. The format is read and never written, which is why D1 accepts a counter
  that can go stale rather than engineering it away, and `next_id` is a number rather than a
  container holding one (D1, D4; Q3; spec.md "What this slice does not decide")
- **No `at` on a `box` or a `line`**, so no chain of references longer than one link and no cycle: a
  `connector` answers no anchor point of its own, and an identity written on a connector never meets
  the one it names (ADR-0041; spec.md _Edge cases_)
- **No picture in ADR-0066, and no `<!-- render: -->` picture in any of the four artifacts.**
  research.md's finding is that every option pair draws the same picture, and a named shape is not
  expressible in the format `xtask` reads, so a marker would draw the position-named picture while
  claiming the named one (Q6; ADR-0064; plan.md "Artifacts")
- **No re-spelling of the shipped identities into names.** Every file in this repository keeps
  writing `#1`, `#2`, … in the order it already lists its entries; re-spelling them with names is a
  later commit on the same format and not a change to it (spec.md Clarification, Q3)
- **No change at all in `crates/monospace-core`, no new crate, no new dependency, and no new gate
  step.** Both new methods live in `monospace-diagram`, which the `wasm` step already compiles, and
  neither reaches a core item: the counter is a `u32` and the placement is a `Vec::push` of the same
  `Placed` an `add` pushes (principle VII; plan.md Constitution Check, principle III)
- **No characterization test, and no ADR-0053 report.** Every rule this slice adds is a rule of the
  model stated in one sentence, which is what a contract test is for, and a snapshot pins a
  rendering rather than a description — so none of the eight files under
  `crates/monospace-cli/src/snapshots/characterization/` moves (spec.md Testing expectations;
  ADR-0053)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: nothing to do.
- **Foundational (Phase 2)**: no dependencies. The baseline is captured before any edit, and the one
  commit that changes no behavior lands before anything depends on it.
- **User Story 1 (Phase 3)**: depends on Phase 2 — T022 refuses all 43 descriptions the moment it
  lands, so it cannot come before the identities are in the files. Its own diagram half (T018-T021)
  depends on nothing but Phase 1.
- **User Story 2 (Phase 4)**: T024 and T025 depend on Phase 3's T019 only in the sense that they
  read the same file; T026 depends on Phase 2's T002.
- **User Story 3 (Phase 5)**: depends on Phase 3's T022 — a required field refuses itself.
- **User Story 4 (Phase 6)**: depends on Phase 3's T022 and on Phase 2's T002.
- **Records (Phase 7)**: depends on Phase 3. T035 and T036 follow the code rather than preceding it,
  which is where 082's D3's one-sentence change to §3 landed (`505fd0d`). T037 and T038 depend on
  neither — the record is of a decision taken, not of code that landed.
- **Polish (Phase 8)**: depends on Phases 2 to 7.

### Within Each User Story

- US1: T018 and T019 before T020 and T021, which read what they produce. T020 and T021 in parallel
  with each other once T019 lands. T022 before T023, which reads a conversion that has to exist.
- US2: T024 before T025, which reads the counter T024 seeds. T026 reads T002's file and nothing
  else.
- US3: T027-T030 in parallel with each other, in two files.
- US4: T031 and T032 before T033, and both before T034's diffs.

### Parallel Opportunities

- T002-T014 all thirteen in parallel once T001 has captured the baseline: thirteen files, no
  dependency between them. This is the widest window in the slice.
- T018 and T019 before T020 and T021, which then run in parallel with each other.
- T022 before T023.
- T024 before T025.
- T026 in parallel with T024 and T025 — it reads a file, not code.
- T027, T028, T029 and T030 all four in parallel once T022 lands, in two files.
- T031 and T032 sequential in one file, then T033; T034 in parallel with T033.
- T035 and T036 sequential in one file; T037 and T038 in parallel with each other and with both.
- T039-T042 are observations and can be run in any order.

**Twenty-six tasks carry `[P]`.** The three widest windows are T002-T014 (thirteen files), T027-T030
(two files) and T037-T038 (two files). Two things cannot be worked in parallel in practice: T016's
deliberate red and T031, because one agent should be running a red at a time.

---

## Implementation Strategy

### The demonstrable increment is Phases 2 to 6, and there is no smaller one

This slice has no MVP in the usual sense, and the reason is worth stating rather than working
around: **nothing observable happens until the reader lands.** Phases 2 to 4 are invisible from
outside — 114 identities a reader throws away, and two methods nothing calls. Phase 5 is the first
place a file that names its shapes behaves differently, and Phase 6 is where the cost this slice
removes comes out of the suite. Stopping after Phase 4 would leave the repository holding 114
identities no code reads, which is a state the demonstration does not demonstrate and a reviewer
cannot see.

### MVP First (Phases 2 to 6, three commits)

1. Phase 2 — the identities land in every description and the reader drops them in silence. Green
   because nothing changes: the same 25 pictures, byte for byte (commit 1).
2. Phases 3 and 4 — `numbered_from` and `add_under` on the diagram, with their contract tests
   (commit 2). Neither method is usable alone and they are one commit for that reason.
3. Phases 3, 5 and 6 — the reader: both fields required, `into_diagram` passing them through, the
   two refusals, the two accepted costs, and the demonstration's workaround and comments. **STOP and
   VALIDATE**: the three diffs silent, 25 pictures, the demonstration byte for byte (commit 3).
4. Phases 7 and 8 — the model, the ADR and the learning log (commits 4, 5 and 6).

### Incremental Delivery

1. Phase 2 → the gate is green and nothing moved → commit 1
2. Phases 3 and 4 → two methods with their contract tests, no snapshot offered → commit 2
3. Phases 3, 5 and 6 → the reader refuses a file that names nothing and honours a file that names
   everything, and the demonstration is unchanged → **commit 3, the demonstrable state**
4. Phase 7 → §3 twice, §9, §11, and ADR-0066 → commits 4 and 5
5. Phase 8 → the gate, the checks nobody can automate, and the learning log → commit 6

### Parallel Team Strategy

With multiple developers:

1. Developer A takes Phase 2's thirteen files — the widest window in the slice, and the one where a
   count disagreement costs the most, so it is the one worth one person's undivided attention.
2. Once Phase 2 is merged, Phases 3 and 4 are one developer's: both are
   `crates/monospace-diagram/src/diagram.rs` and splitting them across two hands means two rebases
   in one file.
3. Phases 5 and 6 are one developer, and both touch `crates/monospace-cli/src/main.rs` and
   `crates/monospace-cli/tests/cli.rs` — the same reason not to split them.
4. Phase 7's T037 and T038 are one developer, and T035-T036 another: two files, no overlap.
