---
description: "Task list for feature 142: taking a shape out leaves what hangs from it not drawn"
---

<!-- The feature directory below is the issue title truncated at forty characters by
     `cargo xtask spec`, which landed inside a word: cspell:ignore referen -->

# Tasks: Taking a shape out leaves what hangs from it not drawn

**Input**: Design documents from `/specs/142-taking-a-shape-out-leaves-the-figures-th/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [data-model.md](data-model.md),
[research.md](research.md), [decisions.md](decisions.md), [quickstart.md](quickstart.md). **There is
no contract**: the slice changes no signature and no field, and `Diagram`, `Shape`, `Position` and
`add_under` are the surface 081, 082 and 083 each recorded, unchanged — so writing the rule a fourth
time is the duplication principle VIII refuses, and D1's answer puts the change to a recorded rule
in ADR-0041, revised in place.

**Tests**: Requested by the spec's **Testing expectations** — four contract tests and one gallery
block in `monospace-diagram`, one new contract test in `monospace-cli`, and **no characterization at
all**. research.md Q6 measured 1916 renderings across 16 files and none of them can express a
removal, so no ADR-0053 report is owed and no `cargo insta review` runs outside the gallery. The two
claims this slice **accepts with nothing to verify them** are named as such in the Polish phase
(T023) rather than described as tested (constitution, Testing and principle IV).

**Organization**: Tasks are grouped by the spec's behaviors B1 to B3, in the order
[plan.md](plan.md)'s six commits deliver them. **That order is forced, not chosen**, and it is worth
naming before the first phase: commit 2 cannot precede commit 1, because the seventh picture is the
_evidence_ for the rule rather than an independent change, and a seventh picture equal to the sixth
is the bug rather than the feature.

**[Story] labels follow the commits, not the numbering of the behaviors.** So **US1 is B1, US2 is B3
and US3 is B2**. The spec gives B1, B2 and B3 no priorities — its `P1`/`P2`/`P3` markers are on the
2026-10-01 _Clarifications_, which are not stories — so plan.md's commit order is the only priority
order there is. Read each phase's title, which carries the behavior label.

**Order within a phase**: implementation before tests, where a test cannot exist before what it
reads, and that inversion is deliberate: in Rust a test that names a method which is not there does
not fail, it does not compile. Task IDs are in execution order throughout.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different file, no dependency on an incomplete task)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Rust cargo workspace. `crates/monospace-diagram/src/` for the diagram library,
`crates/monospace-cli/src/` for the CLI. The unit tests for both live in the
`#[cfg(test)] mod tests` of the file they cover, as the ones already there do — neither crate gets a
new test file. `crates/monospace-diagram/tests/` **does not exist and is not created**: T003 writes
a scratch file there, runs it and deletes it, and nothing in this slice's evidence may rest on a
file the gate compiles. `xtask` does not change, no gate step is added, and nothing under `xtask/`
or `package.json` is touched. The records go to `docs/diagram-model.md` and
`docs/decisions/0041-*.md`, and `assets/demo.json` is **not** edited: the seventh picture is a
change `monospace-cli` makes in its own code, beside the six it already makes (B2.3, ADR-0035,
SC-001).

### The baseline, measured again on 2026-10-02 against this branch

```text
   28  monospace-cli unit tests after this slice   (28 today) — 1 added, 0 removed
   19  monospace-cli integration tests              (unchanged)
  114  monospace-core                               (unchanged)
   76  monospace-diagram after this slice          (72 today) — 4 added, 0 removed
   15  monospace-glyph-sets                         (unchanged)
   61  xtask                                        (unchanged)

    6  captioned pictures a bare run prints today   (7 after T012)
    0  bytes `git diff --stat` reports on crates/monospace-cli/assets/demo.json
   27  generated pictures the `render` step answers, across the tracked tree
```

`cargo test --workspace` is green at **28 / 19 / 114 / 72 / 15 / 61** today, which is
[quickstart.md](quickstart.md)'s count exactly, and a bare `cargo run -q -p monospace-cli` prints
**six** captioned pictures. Both were run on this branch rather than quoted, so the two numbers
every task below counts against are the ones this tree produces.

Two of these carry the rest. The third entry of the shipped description, `#3`, is a four-by-three
box at `{9, 2}` which the fifth picture displaces four columns right into `x 13..16, y 2..4`, and it
is the figure the seventh takes out; the tenth entry, `#10`, is the arrow, whose `from` is the
reference the fifth picture hangs from `#3`'s right side and whose `to` names `#5`'s bottom with
offset `(1, 1)`. `main.rs:127` already binds `the_hung_from` to `ShapeId::new("#3")` and
`main.rs:128` binds `the_arrow` to `ShapeId::new("#10")`, so T012 needs no new name.

---

## Phase 1: Setup

No setup is needed: no dependency is added, no crate is added, the toolchain is unchanged, and no
gate step is added — `wasm` already compiles `monospace-diagram` (plan.md Constitution Check,
principles III and VII). T001 captures what must not move before anything is edited, and T002 is the
one tool the slice needs that `cargo xtask setup` does not install.

- [x] T001 Capture the baseline everything below is compared against, **before** any edit, and keep
      all three until T017. `cargo run -p monospace-cli > /tmp/demo-before.txt`,
      `cargo run -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-before.txt`, and
      `grep -c '^[A-Z].*:$' /tmp/demo-before.txt` must answer **6** — a number measured rather than
      taken from the specification. `git diff --stat crates/monospace-cli/assets/demo.json` must be
      empty **now** as well as at the end, because the seventh picture is a step in the code and not
      a field in the format (B2.3). Also confirm
      `cargo run -q -p monospace-cli crates/monospace-cli/assets/demo.json | head -3` prints one
      picture and **no caption**, which is the guard that the split between the two modes holds
      (AGENTS.md, `monospace-cli` has two modes)
- [x] T002 [P] `cargo install cargo-insta`. `cargo xtask setup` is `npm ci` and nothing else, and
      the gallery's fourth block (T006) is the **only** `cargo insta review` this slice runs, so a
      missing binary stalls at the one place where a snapshot has to be read rather than accepted
      blind (quickstart.md, Prerequisites; AGENTS.md, "Facts that are in the code")

---

## Phase 2: Foundational — the counts, and the arrangement as code

**Purpose**: the two numbers the amendment in Phase 6 rests on, and the three values four tests read
by name. **Neither is a commit**: T003 leaves no file behind and T004 lands inside commit 1.

**⚠️ CRITICAL**: T004 is the arrangement, and T005-T011 cannot be written without it — a test that
names a helper which is not there does not fail, it does not compile.

- [x] T003 Confirm the two counts with a scratch test, then delete it. Write
      `crates/monospace-diagram/tests/scratch_142.rs`, build the arrangement of
      [data-model.md](data-model.md)'s first table twice — once with the connector and once with
      **no connector at all** — and difference the two pictures. Expected: the connector's footprint
      is **exactly six cells**, `{3, 1}` and `{8, 1}` turning `├`/`┤` back to `│` and
      `{4, 1}`–`{7, 1}` written, and a **seven means the arrangement is wrong rather than the
      specification**. Then take the removal on the arrangement with the connector and difference
      **that** against the same arrangement with `#1` removed: expected **fifteen cells change and
      fourteen of them turn blank**, and the fifteenth does not — `{8, 1}` turns `┤` into `│`, which
      is the far box's own left border becoming visible now the arrow is no longer welded to it. A
      test written as "every cell the removal touched is blank" fails on that cell, and T005's own
      assertions are written against it. **The fifteen and the six overlap in `{3, 1}` and `{8, 1}`,
      so they do not add up.** Delete the scratch file before committing and confirm `git status`
      shows no `crates/monospace-diagram/tests/`: nothing in this slice's evidence may rest on a
      file the gate compiles, which is what `cargo xtask fix` will not catch (B1.1, B3.4;
      data-model.md "B1.1's picture, and what `remove` costs it"; constitution principle IV)
- [x] T004 In `crates/monospace-diagram/src/diagram.rs`, give the test module the arrangement the
      four contract tests below share, as named helpers beside `the_box()`, `the_unrelated_box()`
      and `the_window()` which are already there — **not** spelled out four times, which is the
      duplication principle VIII refuses. It is three values: a **four-by-three box at `{0, 0}`**, a
      **three-by-three box at `{8, 0}`**, and a **connector** whose `from` is
      `Position::Reference(Reference { id: "#1", anchor: Anchor::Right, offset: Delta { dx: 0, dy: 0 } })`
      leaving `Direction::Right` with `Terminal::Arm`, whose `to` is `Pos { x: 8, y: 1 }` leaving
      `Direction::Left` with `Terminal::Arm`. Build it through the module's own `Diagram::add` so
      the identities are read back rather than spelled, and give the window as a `twelve-by-three`
      origin `{0, 0}` — the canvas B1.1's hand-drawn picture is measured on. Add **two** variants
      beside it, because the tests need both: one with the connector omitted, which is the baseline
      the **six** is read against, and one with a **connector under `#1`** in the box's place, which
      is route C. Name `#2` as the **only figure that survives** a removal — that is what makes the
      removal legible, and it is the reason the arrangement is not the gallery's one box and one
      connector (B1.1, B3.4; data-model.md "The arrangement, as values" and "The three routes, and
      why two of them are one picture")

**Checkpoint**: the two numbers are measured and the arrangement is nameable. Every test below can
now be written, and **no** existing test may move.

**Commit**: neither. T003 leaves no file and T004 is a private helper, and
`cargo clippy --all-targets -- -D warnings` turns `dead_code` into an error — so **T004 lands inside
commit 1, beside T005**, and it carries no `[P]` for the same reason T005 does not.

---

## Phase 3: User Story 1 - Taking a shape out draws nothing that hung from it (B1, Priority: P1)

**Goal**: the rule §4 and §9 already state, pinned where a reader meets it — a removal takes the
figure that hangs from the removed one with it, leaves every other figure byte for byte, and says
nothing about why. §9 keeps its rule and gains a sentence naming where the open question lives; the
crate's gallery gains the **fourth** block beside three that already mutate a diagram between them,
because a picture of a change is the one kind of picture no description file and no
`<!-- render: -->` marker can reach (ADR-0035, ADR-0064; D2).

**Independent Test**: build the arrangement as written, take `#1` out and draw it, and it must be
the far box alone:

```text
       ┌─┐
       │ │
       └─┘
```

Put the box back under the removed identity with `add_under` and the whole picture must come back
**byte for byte**. Then add a shape the ordinary way and the identity handed back must be `#4`, with
`get(&"#1")` answering `None` and the arrow still not drawn (B1.1, B1.2, B1.3; SC-003).

The step itself, which is `main.rs`'s own code and not a field in the description format — that is
T012's, and it belongs to US3.

### Implementation for User Story 1

- [x] T005 [US1] Contract test: `a_removal_and_a_missing_identity_draw_the_same_thing` in
      `crates/monospace-diagram/src/diagram.rs`. **B1.1 by value, and B3's route B beside route A**,
      which is why it is one test and not two: build T004's arrangement, take `#1` out, and draw it;
      then build the same arrangement with the connector's `from` naming an identity **never added**
      and draw that. The two buffers must be **equal**, cell for cell, and the buffer must be equal
      to `drawn(vec![the_far_box()], …)` — the far box alone, which is what says no other figure
      moved. Ask it by value first: `get(&"#1")` answers `None` after the removal and `get(&"#3")`
      still answers `Some`, so nothing is unfrozen. **The reason to write this test at all** is that
      `taking_the_referenced_figure_out_stops_the_connector_and_changes_nothing_else` at
      `diagram.rs:2512` already pins the same picture — so the new test adds the **reasons** beside
      a second assertion of the same buffer, namely that a removal and a missing identity are
      indistinguishable, which is §11's second question in the only form a test can ask it. Name
      B1.1 and SC-003 in the doc comment, and say in it that the arrangement's second box is the
      only survivor (B1.1; research.md Q1; SC-003)
- [x] T006 [P] [US1] In `crates/monospace-diagram/src/gallery.rs`, add the **fourth** `block()` to
      `an_endpoint_hangs_from_a_side_and_follows_it` at `gallery.rs:387`, with
      `change: "the box taken out"` and `window(8, 3)`, and add the fourth block to the `format!` at
      `gallery.rs:441`. **Reach it from the arrangement as written, through a third `Diagram`** —
      `small_box(at(0, 0), None)` then `hanging_connector(box_id)`, then `remove`, which is the
      pattern `gallery.rs:421-428` already set for the third block and six lines of comment. Do
      **not** reach it from either diagram the test holds: both are already mutated, and a removal
      on either writes **0 of 32** and **0 of 24** positions as well, so the block would be blank
      for a second and different reason while the comment beside it said the first. **Check that
      rather than accepting it** (quickstart.md, "The gallery's fourth block"). Then
      `cargo insta test --review -p monospace-diagram -- an_endpoint_hangs_from_a_side_and_follows_it`
      and **read** the offered snapshot rather than accepting it. Expected: **the first three blocks
      byte for byte what they are**, and a fourth that is **empty** — `wrote 0 of 24 positions` over
      an empty surface row. **That emptiness is the finding, not a defect**: the gallery's
      arrangement is one box and one connector, the connector's `from` names the only other figure,
      and so there is **no survivor**. The block is B1.1's picture without the `#2` that stands in
      it, which is why it is blank where B1.1's hand-drawn one is not. **It is still worth having**,
      because it is a snapshot: a removal that ever began drawing the route, or freezing it where it
      resolved, would write cells into this window and move it — which is precisely what a later
      slice taking the freeze would do. Update the rustdoc above the test, which says "The first
      block is … the second is … and the third is …", to name four, and say there why the picture is
      empty. **The one thing that would change it is widening the arrangement to hold a second
      box**, so the block could carry B1.1's picture rather than its degenerate case; that would
      move all three blocks the snapshot already holds, it is **not** D2's answer, and it is named
      here so the maintainer can raise it at review rather than find it in a snapshot (B1.1; D2;
      data-model.md "The gallery's fourth block measures to an empty picture"; plan.md "Three
      measurements")
- [x] T007 [US1] Contract test: `a_figure_put_back_under_the_removed_identity_draws_again` in
      `crates/monospace-diagram/src/diagram.rs` — **B1.2**, and the claim nothing in the crate
      states today. Take `#1` out of T004's arrangement, then put the box **back under the same
      identity** with `add_under(ShapeId::new("#1"), the_box)`, and assert the buffer is equal to
      the buffer drawn **before** the removal — **byte for byte**,
      `cells(&before, …) == cells(&after, …)`, not merely "the arrow is there". Also assert
      `get(&"#1")` answers `Some` afterwards, since that is the half the picture alone does not say.
      The doc comment carries **the reason this is one test**: §9 says re-adding a shape with the
      same identity would make the references resolve again, and this is the slice that finds out
      whether it can (B1.2; research.md Q2; SC-003)
- [x] T008 [US1] Contract test: `add_hands_back_an_identity_no_shape_holds_after_a_removal` in
      `crates/monospace-diagram/src/diagram.rs` — **B1.3**, "a caller walking into the hole". Take
      `#1` out, call plain `add` with the same box value, and assert the identity handed back is
      **`#4`** and **not** `#1`, that `get(&"#1")` answers `None` while `get(&"#4")` answers `Some`,
      and — the half that is not arithmetic — that **the arrow is still not drawn** and the picture
      is the far box alone. Say in the doc comment that this falls out of `Diagram`'s own counter
      rather than being a rule: `remove` does not touch `next`, so the counter never reissues, and
      `add_under` is the only way to spell an identity. **Nothing repairs the hole**, and a caller
      who re-adds without naming the identity sees a diagram that looks the same and hangs from
      nothing (B1.3; research.md Q2; data-model.md "B1.2 and B1.3, by value")
- [x] T009 [US1] In the same file, rewrite the **two** doc comments `add_under` falsified, in the
      same commit that measures them false — a rustdoc is code, and both sit beside a `pub` method
      that contradicts them on the branch that measured it false (D3). The first is at
      **`diagram.rs:2350-2355`**, on
      `a_figure_added_under_a_spelled_identity_is_what_a_hanging_endpoint_finds`, and its sentence
      "a diagram offers no way to name a shape into existence, so a spelled identity can only ever
      be the one an `add` is about to issue" is false since `add_under` **is** that way and is
      `pub`. **Keep** the rest of the paragraph, which is the reason the case is reachable at all —
      the connector is added **first** so that the figure under test is the one the counter names
      next. The second is at **`diagram.rs:2546-2553`**, on
      `a_figure_put_back_under_the_referenced_identity_draws_the_connector_again`, and "`remove`
      frees an identity permanently — `add` never hands one out twice, **and there is no
      `add_under`**" is false in its second and third clauses while its **first is true**. So
      **rewrite the paragraph's reason and do not delete it**: the case goes through `replace`
      because the identity is found and the kind answers the anchor, which is a reason about
      resolution rather than about removals, and that reason survives. Name `add_under` and B1.2 in
      the new text, and keep the existing line about `#142` being where a removal's silence is
      answered. **`replace`'s own rustdoc at `diagram.rs:170` says the same thing and is not
      touched** — it is about `replace`, not about `remove` (D3; research.md Q5; spec.md "Testing
      expectations")
- [x] T010 [US1] **Make the rules fail on purpose before trusting them.** Two one-line changes to
      the tests' own fixtures, both of which must go **red**, then both restored and green again:
      for T007, spell the put-back with `add` instead of `add_under` — `add` hands back `#4`, so the
      picture comes out **the far box alone** and the test goes red on the comparison, which is B1.3
      reproducing inside B1.2's fixture; for T008, assert `#1` instead of `#4` and confirm the
      failure message **names the counter** rather than the removal. A green run only proves the
      command ran, and T007's claim is the one a silent implementation would satisfy by doing
      exactly what the code does today (quickstart.md, "Make the rules fail on purpose before
      trusting them"; constitution principle IV)

**Checkpoint**: the rule is pinned four ways and the two paragraphs `add_under` falsified now say
what is true. Every test below can be written.

**Commit**: `feat(diagram):` T004-T010 (plan.md commit 1). **It precedes commit 2** because the
gallery block is the one picture that fails loudly, and a snapshot accepted after the
demonstration's seventh is a picture of a rule already in motion. T004-T005 and T007-T009 are one
file, so they can be written in any order among themselves but only one agent should hold
`diagram.rs` at a time; T006 is `gallery.rs` and carries `[P]` (see _Parallel Opportunities_).

---

## Phase 4: User Story 2 - Two rows of the model's table, and a removal is one of them (B3)

**Goal**: §4's table has two rows and B3 has three arrivals — a shape that was never there, a shape
that was taken out, and an identity that is still held by a figure answering no side. **This is the
first test in the repository that can fail on a decision nobody has taken**, and the one test rather
than three because each route is one arrangement and the claim is that they cannot be told apart.

**Independent Test**: draw the three routes and the arrow's **six** cells must be gone in all three,
and the whole picture must be the far box alone in A and B and the far box plus the replacement's
own two cells in C — with `get(&"#1")` still answering `Some` in C. Nothing in the diagram records
which route happened (B3.1, B3.2, B3.3, B3.4; SC-003).

### Implementation for User Story 2

- [x] T011 [US2] Contract test: `the_three_routes_to_one_picture` in
      `crates/monospace-diagram/src/diagram.rs` — **B3.1 to B3.4 as one test**, and its three
      assertions are three arrangements that each hold one of T004's variants. **A** — the
      connector's `from` names an identity never added. **B** — the same arrangement with `#1` taken
      out. **C** — the same arrangement with a **connector** put under `#1` in the box's place,
      which answers no anchor while `get(&"#1")` still answers `Some`. **Compare the arrow's
      footprint, not the whole buffer**: C is **not** the same buffer — it carries the replacement's
      own two cells — and a whole-buffer comparison would fail it for the wrong reason, so the
      comparison takes the six cells `{3, 1}`, `{4, 1}`–`{7, 1}` and `{8, 1}` out of each picture
      and compares those. **The six is asserted against T004's no-connector baseline rather than
      quoted**, because a test that compares three pictures against each other cannot find a count
      that is wrong in all three of them — and the reason for that method is written **at the
      comparison**, not only in the doc comment above the test, since it is what tells the next
      reader that C's difference is the point rather than a bug. **Check the reason is there**
      before calling this done (B3.4; D4; data-model.md "B3.4's six cells, measured as a
      difference"; quickstart.md, Commit 1)

**Checkpoint**: nothing in a diagram records which route happened, and that is now a test rather
than an observation.

**Commit**: the rest of `feat(diagram):` — T011 ticks with T004-T010 (plan.md commit 1). The phase
holds **one** task, and that is the count rather than a thin phase: the spec's _Testing
expectations_ name **one** test for B3 and say "as one test" in those words, because a test per
route would assert nothing the single comparison does not.

---

## Phase 5: User Story 3 - The demonstration grows by one picture (B2)

**Goal**: a person who runs `cargo run -p monospace-cli` sees **seven** pictures, and the seventh is
the sixth with the box the arrow hangs from taken out — **22 cells** gone, the box's twelve and the
arrow's ten, and every one of the twenty-two **turned blank**. The first six are byte for byte what
they are, `assets/demo.json` is untouched, and a path still prints one picture and nothing else.
Principle II's increment, and it is the seventh picture rather than a test because a rule recorded
only in a test is invisible to the person the rule is about.

**Independent Test**: `cargo run -q -p monospace-cli` prints seven captioned pictures; `differing`
between the sixth and the seventh names exactly 22 cells and **every one of them is blank**; the
first six diff clean against T001's capture; `git diff --stat` on `assets/demo.json` is empty (B2.1,
B2.2, B2.3; SC-001).

The step, which is `main.rs`'s own code beside the six changes it already makes and **not** a field
in the description format (ADR-0035), and **no `if let`** — unlike the third, fifth and sixth steps
beside it, because `Diagram::remove` is already a no-op on an identity the diagram does not hold
(`diagram.rs:146-155` says so outright):

```rust
diagram.remove(&the_hung_from);
out.push_str("\nWith the box the arrow hangs from taken out:\n");
out.push_str(&picture(&diagram, &catalog, origin, size));
```

### Implementation for User Story 3

- [ ] T012 [US3] In `crates/monospace-cli/src/main.rs`, add the **seventh** step to `demonstrate` —
      the block above, after the sixth picture's push at `main.rs:227-228`. `the_hung_from` is
      already bound at `main.rs:127`, so **no new name and no new delta**. The caption is the exact
      wording [data-model.md](data-model.md) gives — **no test pins it**, and no caption's wording
      is pinned today either. **The missing `if let` is the claim, not an oversight**, so say so in
      the comment beside the step: `get` returning `None` would panic and `remove` cannot, which is
      why this step is the one that breaks the pattern of the two above it — and T016 measures why
      it is safe on the three degenerate descriptions (B2.1; plan.md measurement 2; ADR-0035)
- [ ] T013 [US3] In the same file, turn `demonstrated_pictures` at **`main.rs:323`** from a
      six-tuple into a **seven**-tuple — the return type, the six `next_picture()` calls and the two
      `expect("six captioned pictures…")` messages — and fix the doc comment at `main.rs:311-322`,
      which reads "The demonstration's **six** pictures" and "the six are then comparable with each
      other". **Its `next_picture` closure needs no change**: it splits on the blank line, strips
      the trailing newline and puts one back, and the seventh block is the last of the output so the
      normalization the first six already get applies to it identically. **Of its eleven call sites,
      three need an edit and eight do not**: `main.rs:446`, `main.rs:447` and `main.rs:772`
      destructure all six positionally and must widen, while `main.rs:301`, `353`, `490`, `502`,
      `687` and `718` bind `(first, ..)`, `(first, second, ..)` or the whole tuple and compile
      unchanged against a seven-tuple. **Count them yourself rather than from this list**, and if it
      comes out as something other than eleven write down what it came out as (B2.1; research.md Q4;
      constitution principle IV)
- [ ] T014 [US3] In the same file, update the three tests whose **names and counts** say six.
      `a_bare_run_prints_six_captioned_pictures_the_first_being_the_description_as_written` at
      **`main.rs:481`** becomes
      `a_bare_run_prints_seven_captioned_pictures_the_first_being_the_description_as_written`, with
      the `assert_eq!` count at **6** becoming **7** and the doc comment's mentions of six
      corrected; it still pins **no caption's wording**.
      `an_empty_description_demonstrates_as_six_identical_pictures` at **`main.rs:686`** becomes
      `an_empty_description_demonstrates_as_seven_identical_pictures` with a **seventh**
      `assert_eq!(pictures.0, pictures.6)`, because an empty description has no `#3` and the seventh
      step changes nothing — a count of six there would be a count of pictures the helper happened
      to return. `one_shape_demonstrates_as_two_copies_of_itself_and_then_an_empty_window` at
      **`main.rs:717`** keeps its name, which is still true, and **gains
      `assert_eq!(pictures.5, pictures.6)`** beside the `assert_eq!(pictures.4, pictures.5)` it
      already carries — for exactly the reason the sixth is worth having, which is that a one-shape
      description has no `#3` either. **No other test's name changes**:
      `the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it` still describes the fifth,
      `the_sixth_picture_moves_only_the_arrow` still describes the sixth, and
      `a_path_prints_one_picture_and_nothing_else` still describes a path (B2.2, B2.3; research.md
      Q4)
- [ ] T015 [US3] Contract test: `the_seventh_picture` in `crates/monospace-cli/src/main.rs` — **the
      claim B2.1 makes and nothing else pins**. Model it on `the_sixth_picture_moves_only_the_arrow`
      at `main.rs:771` and reuse the `differing` helper at `main.rs:532`:
      `differing(&sixth,     &seventh)` must name **exactly 22** cells, and **every one of the
      twenty-two must be blank** in the seventh. **Both assertions, and the second is not
      optional**: twenty-two cells that _differ_ is half the claim, because a rule that drew
      something _in_ the removed box's place would also produce twenty-two differing cells. **The
      count is a count and the bound is exact**: quote the two territories rather than reading them
      off the picture — the box's twelve at `x 13..16, y 2..4` (including `(16, 3)`, which is inside
      `#3`'s own footprint and is the attachment cell the existing sixth-picture test already names
      as `THE_ATTACHMENT`) and the arrow's ten at `y 5..7`, four on its top row at `x 16..19`, two
      verticals, four on its bottom row at `x 19..22` — because the arrow's ten are **neither
      contiguous nor a rectangle**, which is where reading the count off the picture goes wrong
      (B2.1; data-model.md "The demonstration's seventh picture, cell by cell")
- [ ] T016 [US3] Measure the case research.md Q4 does **not** cover, because it is the one that can
      hide a mistake: take the shipped `demo.json`, **rename `#3`** — the figure the seventh step
      removes — and run `demonstrated_pictures` over the result. Expected: **seven** pictures, and
      the **sixth equal to the seventh**, since the seventh step is then a no-op on an identity the
      diagram does not hold. If the seventh differed there, the step would be doing something other
      than a removal. Do it in a scratch test beside the module, run it, and take it out again, or
      as a `serde_json` rewrite of `super::DEMO` in the shape
      `the_tenth_entry_naming_a_reference_leaves_the_first_five_pictures_exactly_as_they_were` at
      `main.rs:422` already uses — **and if it stays, then it is a test and it belongs to T014**;
      say which you chose. This is the third of the three degenerate descriptions that make the
      missing `if let` in T012 safe, and the other two are T014's two (B2.3; quickstart.md, Commit
      2; plan.md measurement 2)
- [ ] T017 [US3] The three diffs, run in one place, and they are the other side of every claim in
      this phase. `sed -n '1,/With the arrow displaced as well:/p'` on `/tmp/demo-before.txt` and on
      a fresh `cargo run -q -p monospace-cli` must print **nothing**, which is B2.2 asked rather
      than assumed — the sixth caption is the split point precisely so the seventh is not compared;
      the same command with the path `crates/monospace-cli/assets/demo.json` must print nothing
      against `/tmp/file-before.txt`; and `git diff --stat crates/monospace-cli/assets/demo.json`
      must print **nothing**, which is B2.3's cost claim — a diff there means the seventh picture
      was put in the file rather than in the code and the format is carrying a field ADR-0035 keeps
      out of it. Keep `/tmp/demo-before.txt` and `/tmp/file-before.txt` from T001 until this has run
      (B2.1, B2.2, B2.3, SC-001)

**Checkpoint**: a person who runs the application sees seven pictures, the seventh with the arrow
and its box gone and nothing drawn in their place, and the file those pictures came from is
untouched.

**Commit**: `feat(cli):` T012-T017, ticking their checkboxes with it (plan.md commit 2). **T012
cannot precede commit 1** — plan.md is explicit that the picture is the evidence for the rule rather
than an independent change. T012 lands **before** T013, because the helper cannot return seven
pictures the demonstration does not print, and T013 before T014-T016, because each of those takes
the tuple apart. All six are `main.rs`: they can be written in any order among themselves but **one
agent holds the file**, and none carries `[P]`.

---

## Phase 6: Records this slice writes and revises

**Purpose**: four `docs` commits, none of them a behavior, and so none carrying a `[Story]` label.
They are placed here rather than in the Polish phase because plan.md orders them **after** the code
— the first of them amends a count the measurement T011 now asserts, and the second is `docs(model)`
which follows the rule, which is where 082's and 083's model amendments landed (`505fd0d`).

- [ ] T018 [P] In `specs/142-taking-a-shape-out-leaves-the-figures-th/spec.md`, amend **B3.4**: the
      count of the arrow's cells goes from **ten to six**, naming the measurement — the four cells
      `{4, 1}`–`{7, 1}` its route writes and the two borders `{3, 1}` and `{8, 1}` it turns, read as
      the difference between this arrangement and the same arrangement with no connector in it.
      **Say where the ten belongs** while doing it: it is the demonstration's own `#10` in B2.1,
      where measurement confirms it, and two arrangements sharing a count is how a number from one
      reached the other. **One line, and its own commit**, so a reader can see a corrected claim
      come from a measurement rather than from an edit. It follows the code because the count is
      measured rather than corrected on sight, and the measurement is what T011 now asserts — so the
      two land in the order the reason does (D4; quickstart.md, Commit 3; principle IV)
- [ ] T019 In `docs/diagram-model.md`, take §9 _Changing a diagram_'s removal paragraph at
      **`docs/diagram-model.md:306-308`** and append **one sentence** naming where the open question
      lives. **Check it says only that this one rule has a question standing next to it** — a note
      claiming the other rules are settled would be a new claim the slice has no standing to make,
      and no rule here is final (P3's answer; constitution, _Understanding changes_). Keep the
      paragraph's rule exactly as it stands: the references stay as they were, nothing is rewritten
      and nothing cascades, and re-adding a shape with the same identity would make them resolve
      again. **§4 is not amended** — its two-row table already answers the question, and §9's table
      of five grows no row (B1.1; plan.md Design; spec.md "What this slice implements")
- [ ] T020 In the same file, add **two** bullets to §11 _Open questions_. The first is the issue's
      own question and it is the one on the sheet when it is answered: whether anything should
      happen at all to what hung from a removed shape, with the issue's proposal named — freeze each
      hanging position at the absolute point it was resolving to — and the trigger named as the
      first consumer that takes a shape out and expects the picture to keep what hung from it, **of
      which there is none**. The second is **how anything would tell a removal from a shape that is
      not there**, written as **three routes rather than as a sentence** — the identity never there,
      the identity taken out, the identity held by a figure answering no side — because that is what
      makes it answerable by somebody who has not read issue #142, which is SC-002's whole
      requirement. **Name that the second cannot be answered first**, and that the two go on one
      sheet or on two **in that order**, because a sheet that answered them the other way round
      would be answering a question whose answer had not been chosen. Amend no other section and add
      no other bullet (P2; SC-002; plan.md Decisions)
- [ ] T021 [P] In `docs/decisions/0041-resolve-a-position-through-a-reference.md`, correct the line
      at **line 94** that reads "Today identities are generated rather than written, so there is
      nothing to mistype" — **false**, and ADR-0066 made it false: a caller writes an identity,
      `add_under` checks nothing, and a misspelling is silent, which is the exact cost that
      paragraph describes as not yet existing. It is corrected **where the reader takes the
      resolution rule for settled**, which is what D1's answer chose, and **add a `## Revisions`
      section** the way
      [ADR-0067](../../docs/decisions/0067-displace-a-figure-holding-a-reference-by-growing-its-offsets.md)
      carries one — dated, and naming that a removal's silence is a standing question **and** that
      the identities are writable now. **One record, no second one, and no row in
      `docs/decisions/README.md`**: the title does not change and the status stays `accepted`, so
      the index is untouched — which is also the safe outcome, that table being one prettier-aligned
      grid where a partial edit corrupts it silently. Confirm the correction went **in place**:
      `grep -n     "nothing to mistype"` must return nothing (D1; research.md Q5; plan.md Design)

**Commit**: `docs(spec-142):` T018, `docs(model):` T019 and T020, `docs(adr):` T021 (plan.md commits
3, 4 and 5). T019 and T020 are one file and one commit; T018 and T021 are two further files, so
their **edits** divide and their **commits do not** — plan.md's order is the order they land in.

---

## Phase 7: Polish & Cross-Cutting Concerns

- [ ] T022 Run `cargo xtask check` and confirm every step is green, **including `wasm`** — which
      compiles `monospace-core`, `monospace-diagram` and `monospace-glyph-sets`, so it covers a
      slice that reaches nothing in the core (`remove`, `displaced_by` and `reference` appear
      **zero** times under `crates/monospace-core`, measured). Watch the **output** rather than the
      exit code: `rustfmt` reports that `group_imports` needs nightly and exits 0, and so does a
      step that finds something it cannot fix. The `render` step must still answer **27 generated
      pictures** — measured on this branch — and `numbering` must be green, and **no marker may
      move**: no artifact in this slice carries a `<!-- render: -->` marker, because a marker reads
      a description the file carries and a description cannot take a shape out (ADR-0035, ADR-0064;
      SC-004)
- [ ] T023 [P] Confirm the two claims this slice **accepts with nothing to verify it** are named as
      such rather than described as tested, as principle IV asks and plan.md's re-check does on the
      spot: that **a removal leaves every other figure byte for byte** — a claim about the whole
      diagram rather than about the figure that went, which the contract tests verify for the three
      figures T004's arrangement names and which no single fixture can establish for every diagram —
      and that **the gallery's blank block is worth keeping**, which is a judgement about what a
      reader makes of an empty window rather than a measurement. Look for the first in T005's and
      T008's doc comments, and for the second in **T006's**, where it belongs, and leave each where
      it is — **this task is a check that the record says so, not an edit** (constitution principle
      IV; plan.md "Re-checked after Phase 1")
- [ ] T024 Take the counts this slice claims and check them rather than asserting them:
      `cargo test --workspace` green at **29 / 19 / 114 / 76 / 15 / 61**, which is today's **28 / 19
      / 114 / 72 / 15 / 61** plus four contract tests and one CLI test and **no test removed**.
      Confirm `grep -c '^[A-Z].*:$'` on a bare run answers **7**;
      `a_path_prints_one_picture_and_nothing_else` passes **unchanged** — it is the guard that a
      path still prints one picture and nothing else, which is what `cargo xtask render` embeds; no
      `.snap.new` file is left under `crates/monospace-diagram/src/snapshots/gallery/`; and
      `git status` shows no `crates/monospace-diagram/tests/` and no scratch from T003 or T016. **No
      characterization report is owed**: research.md Q6 measured 1916 renderings across 16 files and
      none of them can express a removal, which is why the gallery's snapshot is the **only**
      picture in the repository a removal can move (SC-004; research.md Q6; constitution, Testing)
- [ ] T025 Append an entry to `docs/learning-log.md` for this increment: what was learned about Rust
      design and idiom, what was learned about working this way, and optionally a trade-off worth
      remembering. Four are already paid for and worth writing down rather than rediscovering — a
      **`remove` needs no `if let` where the three steps beside it each need one**, which is a fact
      about a method's documented no-op rather than about the call site, and is what makes three
      degenerate descriptions worth keeping; **a removal's silence and a missing identity are the
      same picture, and the only way to say so is a test that puts them in one assertion**; **an
      inventory of the prose a change falsifies is a hypothesis, and `add_under` falsified two doc
      comments and one `load-bearing` record while the specification named one place**; and **two
      arrangements sharing a count is how a number from one reached the other**, which is the whole
      of the ten becoming a six (constitution principle II; research.md Q3, Q5)

**Commit**: `docs:` T025 (plan.md commit 6). T022-T024 are a gate run and two observations, not
commits.

---

## What this slice must not add

Each of these is a decision already answered on the sheet or a rule the model states. Reaching past
one means stopping and asking, not deciding and recording afterwards (constitution, No decision
outside the sheet).

- **No rule, and no signature, field, variant or derive.** `Diagram`'s whole surface — `add`,
  `add_under`, `get`, `replace`, `remove`, `forward`, `backward`, `draw`, `numbered_from` — is
  untouched, and so are `Position`, `Reference`, `Anchor`, `Delta`, `Shape`, `ShapeId` and
  `Endpoint`. `Diagram::remove`'s own rustdoc at `diagram.rs:146-155` is **not** touched either: it
  already says what this slice is about, and it is why T012 needs no `if let` (plan.md "What does
  not move")
- **No amendment to §4 of `docs/diagram-model.md`, and no sixth row in §9's table of five.** §4's
  two-row table already answers the question; this slice shows that paragraph in a picture (plan.md
  Design)
- **No second record beside ADR-0041 and no row in `docs/decisions/README.md`.** D1's answer is one
  record revised in place, because the rule and the question cannot be cited apart (D1)
- **No `<!-- render: -->` marker in any artifact, and no hand-drawn picture standing unlabelled.** A
  marker reads a description the file carries and a description cannot take a shape out, so
  `cargo xtask render` regenerates nothing this slice touches and a changed marker in the gate is a
  mistake rather than an update (ADR-0064, ADR-0035; plan.md "Artifacts")
- **No freeze, and nothing that remembers a removal.** The issue's own answer — holding each hanging
  position at the absolute point it was resolving to — is **taken and not implemented**, not
  refused. The positions carrying those references are values today and rewriting them is arithmetic
  a later slice can do, so this slice does not cost it anything. **§11's two questions go on no
  sheet here**, and when one is answered they go in that order (spec.md "What this slice does not
  decide")
- **No report of a figure the diagram could not draw**, neither a figure pushed out of the window
  nor a reference that resolves to nothing: `#88`'s, and a slice whose subject is a rule has no
  standing to add a diagnostic query (ADR-0041; [#88](../../issues/88))
- **No fourth contract.** The three contracts 081, 082 and 083 recorded are unchanged, and D1's
  answer says a change to a recorded rule lives in the record it belongs to (plan.md "Artifacts")
- **No characterization test and no ADR-0053 report.** 1916 renderings across 16 files, none of
  which can express a removal (research.md Q6)
- **No `refactor` commit for the seven-tuple.** plan.md's measurement 1 is the reason: a tuple's
  arity is part of every destructuring site's type, so widening the returns does not compile until
  the eleven callers widen too. The `refactor` shape — widen it by handing back the sixth twice — is
  available and **not taken**, because a commit that changes nothing is the fix principle III
  refuses to manufacture. The rejected `Vec<String>` alternative is named in the same place (plan.md
  measurement 1; principles III, V)
- **No field in `assets/demo.json` and nothing in the description format.** Every marker in the tree
  reads it and the seventh picture costs one call (B2.3; ADR-0035)
- **No new dependency, no new crate, no gate step, no change to `xtask`, and nothing in
  `monospace-core`** — the core has no `remove` at all (principles III, VII; research.md Q6)
- **No arrangement that a removal does not touch, and no widening of the gallery's arrangement.**
  The gallery's one box and one connector give a blank fourth block, and **widening it to hold a
  second box** would move all three blocks the snapshot already holds. It is named so the maintainer
  can raise it at review, and it is **not taken** (data-model.md, D2)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: nothing to do; T001 captures the baseline everything else is compared
  against.
- **Foundational (Phase 2)**: depends on nothing, and **blocks all three user story phases** — a
  test that names a helper which is not there does not fail, it does not compile, and the count T011
  asserts is the count T003 confirms.
- **User Story 1 (Phase 3, B1)**: depends on Phase 2. **No dependency on US2 or US3.**
- **User Story 2 (Phase 4, B3)**: depends on Phase 2. **No dependency on US1** — T011 builds its own
  three arrangements out of T004's variants and asks nothing US1 wrote. It **shares commit 1** with
  US1, which is the only thing ordering them.
- **User Story 3 (Phase 5, B2)**: depends on Phase 2 and **cannot precede Phases 3 and 4**. plan.md
  is explicit that commit 2 cannot come first, because the seventh picture is the evidence for the
  rule rather than an independent change. Nothing in `monospace-diagram`'s production code moves, so
  the stories do not touch each other's files.
- **Records (Phase 6)**: T018 depends on T011, which asserts the count it amends. T019, T020 and
  T021 depend on the code rather than preceding it. T021 depends on nothing but D1's answer.
- **Polish (Phase 7)**: depends on all of them.

### Within Each User Story

- US1: T004 before T005, T007 and T008 — the helpers they name. T005 before T007 and T008 in no
  sense but one: they may be written in any order among themselves, and only one agent should hold
  `diagram.rs`. T009 is a rewrite of prose T007 measures false, and T010 comes **last**, because a
  deliberate red is one agent's work and it needs both tests in place.
- US2: one task. It reads T004's three variants and nothing else.
- US3: T012 before T013 — the helper cannot return seven pictures the demonstration does not print.
  T013 before T014, T015 and T016 — each takes the tuple apart. T015 after T012, since it compares
  the sixth with the seventh. T017 after all of them, because it is the diff against T001's capture.

### Parallel Opportunities

Parallelism here is lower than in 083 and 148, and the reason is worth stating rather than padding:
**eight of the twenty-five tasks edit one of two files** — `diagram.rs` (five tasks) and `main.rs`
(six tasks and one helper). Five tasks carry `[P]`, and they are exactly the ones whose file is free
while another task runs:

- T002 with T001 — installing `cargo-insta` against capturing the baseline.
- T006 with T005, T007, T008 and T009 — `gallery.rs` against `diagram.rs`. **This is the one
  judgement call here, and it is the opposite of 143's**: that slice's gallery block was **not**
  marked `[P]` because it displaced the connector through the `Shape::displaced_by` arm its commit 1
  was adding, so a half-finished commit would have moved the snapshot. **Nothing in
  `monospace-diagram`'s production code moves in this slice at all** — `remove` is already there and
  is not touched — so this snapshot depends only on the block T006 itself, and the two files do not
  meet.
- T018 with T019, T020 and T021 — `specs/142-*/spec.md` against `docs/diagram-model.md` and
  `docs/decisions/0041-*.md`. Three different files, no dependency between them, and D1's answer
  settles T021 on its own.
- T023 with T022 and T024 — a check that the record says so, against a gate run and a count.

Three things are deliberately **not** marked `[P]` and are worth naming: **T006** is the only task
that moves a snapshot and `cargo insta review` is a single reviewer; **T010** is a deliberate red
that one agent should run and restore; and **T004** is a private test helper that
`cargo clippy --all-targets -- -D warnings` turns into `method is never used` unless it lands beside
the first test that calls it, which is T005.

---

## Implementation Strategy

### The demonstrable increment is Phases 2 to 5, and there is no smaller one

There is no MVP phase to stop at, and saying so is more useful than naming one. Phase 4 alone is one
test and no picture. Phase 5 alone is a seventh picture with nothing pinning the rule under it.
Phase 3 alone is a `feat` whose evidence is a gallery snapshot a reader has to go looking for. The
rule and the picture that shows it were shipped together in plan.md's commit order for exactly that
reason, and the smallest increment this slice has is **commits 1 and 2 together**.

### MVP First (Phases 2 to 5, two commits)

1. Phase 2 → the two counts confirmed and the arrangement named, leaving no file and no commit of
   its own.
2. Phase 3 → four contract tests, the two corrected doc comments and the gallery's fourth block: the
   rule by value and drawn, a put-back that restores it byte for byte, an ordinal that does not
   reissue, and a blank snapshot that fails if a removal ever starts drawing (commit 1).
3. Phase 5 → the seventh picture, its caption, the seven-tuple and its callers, and the test that
   counts the twenty-two and checks every one blanks (commit 2).
4. **STOP and VALIDATE**: `cargo test --workspace` green at 29 / 19 / 114 / 76 / 15 / 61;
   `cargo run -q -p monospace-cli` prints seven captioned pictures and the seventh is the sixth with
   22 cells blank; `git diff --stat crates/monospace-cli/assets/demo.json` is empty; a path prints
   one picture and nothing else.

### Incremental Delivery

1. Phase 2 → the numbers are measured and the arrangement is nameable.
2. Phase 3 → the rule is something a broken implementation fails (commit 1).
3. Phase 5 → the shipped run carries the evidence, and the gap the issue describes is visible to a
   person rather than only to a test (commit 2).
4. Phase 6 → the corrected count, the two places a reader meets the question, and one record revised
   where the rule is written (commits 3, 4 and 5).
5. Phase 7 → the gate is green, the two untested claims are named as untested, and the increment is
   closed (commit 6).

Each commit follows plan.md: **one** `feat(diagram)` carrying the tuple's widening rather than a
`refactor` and a `feat`, one `feat(cli)`, and four `docs` — and every commit leaves
`cargo xtask check` green (constitution principles II, III and V).

### Parallel Team Strategy

With two agents:

1. Agent A takes Phase 2, then Phase 3. Agent B takes Phase 4's single test off the same fixture the
   moment T004 lands, and then Phase 6's `docs(decisions)` edit, which depends on D1's answer and on
   nothing else — but **not before**, because a record that follows nothing is a record of an
   intention.
2. Inside Phase 5, the seventh step, the tuple, the three renamed tests and the new test are one
   file and do not divide.

The honest answer is that this slice is **not** a parallel slice. Eight of twenty-five tasks touch
one of two files, and the two commits that matter are ordered by what they are evidence for.
