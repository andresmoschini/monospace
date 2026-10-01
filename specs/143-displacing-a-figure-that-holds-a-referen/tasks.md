---
description: "Task list for feature 143: displacing a figure that holds a reference moves it"
---

<!-- The feature directory below is the issue title truncated at forty characters by
     `cargo xtask spec`, which landed inside a word: cspell:ignore referen -->

# Tasks: Displacing a figure that holds a reference moves it

**Input**: Design documents from `/specs/143-displacing-a-figure-that-holds-a-referen/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [data-model.md](data-model.md),
[research.md](research.md), [decisions.md](decisions.md), [quickstart.md](quickstart.md). **There is
no contract**: the slice changes no signature and no field, and `Shape::displaced_by`,
`Position::displaced_by` and `Diagram` are the surface 081, 082 and 083 each recorded, unchanged —
so writing the rule a second time in a fourth contract is the duplication principle VIII refuses,
and the ADR T023 writes is the one home (plan.md "Artifacts").

**Tests**: Requested by the spec's **Testing expectations** — six contract tests in
`monospace-diagram`, one test there **rewritten rather than deleted**, one new contract test in
`monospace-cli`, and **no characterization at all**. research.md Q3 measured 1916 renderings across
16 files and none of them can express this change, so no ADR-0053 report is owed and no
`cargo insta review` is run outside the gallery. The two claims this slice **accepts with nothing to
verify them** are named as such in the Polish phase (T026) rather than described as tested
(constitution, Testing and principle IV).

**Organization**: Tasks are grouped by the spec's behaviors B1 to B3, in the order
[plan.md](plan.md)'s five commits deliver them — that order is forced by constitution principles II
and V rather than chosen, so it is the priority order here too. **B1 and B2 share one commit** with
the arithmetic they read, and the split into two phases is for traceability, not a license to make
two.

**Order within a phase**: implementation before tests, where a test cannot exist before what it
reads. That inverts the usual listing, and it is deliberate: in Rust a test that names a method
which is not there does not fail, it does not compile. Task IDs are in execution order throughout.

## Format: `[ID] [P?] [Story] Description`

- **[P]**: Can run in parallel (different file, no dependency on an incomplete task)
- **[Story]**: Which user story this task belongs to (US1, US2, US3)
- Include exact file paths in descriptions

## Path Conventions

Rust cargo workspace. `crates/monospace-diagram/src/` for the diagram library,
`crates/monospace-cli/src/` for the CLI. The unit tests for both live in the
`#[cfg(test)] mod tests` of the file they cover, as the ones already there do — neither crate gets a
new test file, and `crates/monospace-diagram/tests/` does not exist and is not created. `xtask` does
not change, no gate step is added, and nothing under `xtask/` or `package.json` is touched. The two
records go to `docs/decisions/`, and `assets/demo.json` is **not** edited: the sixth picture is a
change `monospace-cli` makes in its own code, beside the four it already makes (B3.2, ADR-0035).

### The baseline, measured again on this branch

Every number below was measured on 2026-10-01 against the tree this branch points at, and is what
the tasks carry rather than what an artifact estimates.

```text
   28  monospace-cli unit tests after this slice   (27 today)
   19  monospace-cli integration tests              (unchanged)
  114  monospace-core                               (unchanged)
   72  monospace-diagram after this slice          (66 today: diagram.rs 52, position.rs 7,
                                                   shape.rs 3, gallery.rs 4)
   15  monospace-glyph-sets                         (unchanged)
   61  xtask                                        (unchanged)
```

`cargo test --workspace` is green at **27 / 19 / 114 / 66 / 15 / 61** today, which is exactly
`quickstart.md`'s count minus the seven tests this slice adds. A bare run prints **five** captioned
pictures (`grep -c '^[A-Z].*:$'` on `cargo run -q -p monospace-cli`) and a run given
`crates/monospace-cli/assets/demo.json` prints **thirteen** rows.

Two of the counts are the ones the sixth picture rests on, and T001 reads them off the shipped file
rather than quoting them: the third entry is a four-by-three box at `{9, 2}`, which the fifth
picture displaces four cells right into `x 13..16, y 2..4`; the fifth entry is a four-by-three box
at `{20, 1}` occupying `x 20..23, y 1..3`, and **the demonstration never displaces it**; the tenth
entry is the arrow, whose `from` the demonstration rehangs to `#3`'s right side with offset `(0, 0)`
— which `side_centre` answers `{16, 3}` — and whose `to` is the shipped reference to `#5`'s bottom
with offset `(1, 1)`, which is `{21, 3} + (1, 1)` = `{22, 4}`. After `dy: 2` the two endpoints stand
at `{16, 5}` and `{22, 6}`. Both are references at that point in the demonstration, which is why
today's sixth picture is the fifth byte for byte (research.md Q1; data-model.md "The demonstration's
sixth picture, by value").

---

## Phase 1: Setup

No setup is needed: no dependency is added, no crate is added, the toolchain is unchanged, and no
gate step is added — `wasm` already compiles `monospace-diagram` (plan.md Constitution Check,
principle III; principle VII). T001 captures what must not move before anything is edited.

---

## Phase 2: Foundational — the arithmetic and the arm land in one commit

**Purpose**: one private addition to `Delta` and one changed arm, plus the three paragraphs and one
sentence that declare the no-op this slice reverses. **Nothing here may touch a test**, because
there is no structural commit to make: the gate runs
`cargo clippy --workspace --all-targets -- -D warnings`, `dead_code` is a warning, and the flag
turns it into an error, so `Delta::grow` without its caller is **method `grow` is never used**, as
an error, and there is no split to make. Part one of [plan.md](plan.md) promised a `refactor` of its
own and measured that it is not green; this phase is the correction (plan.md "The arithmetic cannot
be the `refactor` part one promised"; constitution principle V).

The whole of the rule is one addition and one arm, and neither is a new type, a new field or a new
signature:

```rust
// crates/monospace-diagram/src/delta.rs — beside `apply`, and the crate's second addition
impl Delta {
    pub(crate) fn grow(self, by: Delta) -> Delta {
        Delta {
            dx: self.dx.saturating_add(by.dx),
            dy: self.dy.saturating_add(by.dy),
        }
    }
}
```

```rust
// crates/monospace-diagram/src/position.rs — the `Absolute` arm above it is 082's rule
Self::Reference(reference) => Self::Reference(Reference {
    id: reference.id.clone(),
    anchor: reference.anchor,
    offset: reference.offset.grow(by),
}),
```

- [ ] T002 In `crates/monospace-diagram/src/delta.rs`, add
      `pub(crate) fn grow(self, by: Delta) -> Delta` beside `Delta::apply`, growing each axis with
      `saturating_add`, and **rewrite the sentence that makes it false in the same edit**: `apply`'s
      rustdoc at **line 42** reads "The only arithmetic in this crate, and it saturates rather than
      wrapping", which T003's arm turns false. Say on `apply` that it is one of the two additions in
      this crate and that both saturate, keep its saturation rule and its reason (a wrapped
      coordinate can land inside a window a caller could really hold) **on `apply`, where it is
      argued**, and give `grow` its own rustdoc naming three things: it is `pub(crate)` because
      nothing outside the crate needs it, it saturates for the reason `apply` does, and **it keeps
      no memory**, so a displacement back does not undo a saturating one. `Delta`'s two public
      fields, its derives and its `From<(i32, i32)>` are unchanged — data-model says so, and a diff
      that touches one is a diff that widened the type (Q5; data-model.md "`Delta` — a second way to
      add one delta to another")
- [ ] T003 In `crates/monospace-diagram/src/position.rs`, give `Position::displaced_by`'s
      `Reference` arm the three fields the rule names — the second block in this phase's
      introduction — replacing `reference @ Self::Reference(_) => reference.clone()` at **line
      143**. `id` and `anchor` move nothing — they name **which** figure and **which** side, not
      where — and `id` is cloned because `ShapeId` is a `String`, which is why `Reference` is not
      `Copy` and is this slice's to change nothing about. **The `Absolute` arm is 082's rule and
      does not move.** `Shape::displaced_by`'s signature, its three arms and its `pub` visibility
      are not touched here (B1.1, B1.2, SC-001; data-model.md "`Position::displaced_by` — the
      reference arm, grown"; plan.md commit 1)

- [ ] T004 In `crates/monospace-diagram/src/position.rs`, rewrite the **two** rustdoc paragraphs
      T003 makes false, in the same commit that makes them false — a rustdoc is code. The first is
      `resolve`'s, at **lines 110-113**: it reads "A displacement is the other half and it is
      **not** reached here: a reference's offsets travel with the figure it hangs from, so nothing
      adds to them. That is #143's decision and a deliberate no-op rather than an omission", and all
      three halves are now false — the displacement **is** reached here, from the other side. Keep
      the paragraph's real content, which is that both `?`s come before the addition and that a
      reference resolving to nothing stays nothing, and point at §4 _Positions_ for the rule. The
      second is `displaced_by`'s own, at **lines 126-138**: "A [`Position::Reference`] is left
      exactly as it went in", "**This is a silent no-op on purpose**", and "Walking the arithmetic
      into this arm is #143's decision and not this method's to take" all go, and what replaces them
      says what the arm does and what each field does (SC-002; Q6; data-model.md "What does not
      change"; plan.md commit 1)
- [ ] T005 [P] In `crates/monospace-diagram/src/shape.rs`, rewrite the **fourth** of the four places
      that declare the no-op, at **lines 179-183** — `Shape::displaced_by`'s own rustdoc, which ends
      on "What displacing such a figure should mean in general is #143's to settle". Say what the
      connector's `from.at` and `to.at` do now, that an endpoint holding a reference **slides**
      rather than traveling with the figure it hangs from, and that §4 _Positions_ states the
      destination. **Keep** the three paragraphs above it, which are true: every position this
      variant holds moves, a connector is not the exception, and `&self -> Self` composes with
      `Diagram::get`. This paragraph is the one a caller reads first — `Shape::displaced_by` is
      `pub` where `Position::displaced_by` is `pub(crate)` — which is D3's reason for naming it at
      all (D3; Q6; spec.md "What this slice rewrites rather than contradicts"; plan.md commit 1)

**Checkpoint**: displacing a figure that holds a reference grows its offsets. Every test below can
now exist, and none of the three crates' **existing** tests may have moved except the one T006
rewrites.

**Commit**: `feat(diagram):` T002-T005 (plan.md commit 1). T002 and T003 touch different files and
are edited in parallel, but **neither lands alone** — neither carries `[P]` because a `[P]` task is
one that can land and be checked on its own.

---

## Phase 3: User Story 1 - A displacement reaches a reference's offsets (B1, Priority: P1)

**Goal**: displacing a connector whose endpoint holds a reference slides that endpoint by the same
delta the figure moved by, so the figure draws as a translation of itself rather than bending its
route. §4 _Positions_ already states the rule and this slice makes it true, and the crate's gallery
gains the picture that says what the rule draws — drawn by the code rather than written by hand, so
a rule that stops holding drops a snapshot instead of nothing (D2; ADR-0064).

**Independent Test**: build the arrangement §4 names — a four-by-three box at the origin and a
connector whose `from` is a reference to its right side with offset `(0, 0)`, leaving rightward,
`to` at `{7, 1}` leaving leftward, both arm terminals — displace the **connector** two cells down,
and it must draw

```text
    ┌──┐
    │  │
    └──┘
       ─────
```

with the box exactly where it was. Displace it four cells right instead and the gap between border
and endpoint grows by four while the box does not move. Displace a figure holding **two** references
and both offsets grow by the same amount and the figure is rigid (spec.md B1.1, B1.2, B1.3; SC-001).

### Implementation for User Story 1

- [ ] T006 [US1] Rewrite `a_displacement_moves_a_point_and_leaves_a_reference_alone` at
      `crates/monospace-diagram/src/position.rs:428` **rather than deleting it**, and rename it to
      what is now true — `a_displacement_moves_a_point_and_grows_a_references_offset`. Three things
      change and one does not: the bare reference's `offset` grows by `by` while its `id` and
      `anchor` come back equal, the absolute point still moves, and **both** sides carry an
      `assert_ne!` — a `displaced_by` that changed nothing at all would satisfy every equality here
      by doing exactly what the code does today, which is the bug this slice exists to end. What
      does not change is that "a reference comes back equal to itself" is still true of a
      displacement of nothing; only its reason was wrong, which is why the test is rewritten and not
      removed. Its doc comment's opening line is stale and goes with it: it reads "User Story 4,
      spec's B4.1", and that B4.1 is **082's** scenario, which said the opposite. Name this slice's
      B1.1 instead (B1.1, B1.2, SC-001; data-model.md "`Shape::displaced_by` — unchanged in
      behavior, and now says so"; quickstart.md "Commit 1")
- [ ] T007 [P] [US1] Contract test: `a_displacement_grows_a_references_offsets` in
      `crates/monospace-diagram/src/diagram.rs`, **asked and then drawn**, which is the spec's
      wording and the order it asks for. By value: build the connector through the `arm_connector`
      helper already in that module — it takes two `Position`s, so a reference is expressible
      directly — displace it by `Delta { dx: 0, dy: 2 }`, and assert the `from` reference's `offset`
      is `Delta { dx: 0, dy: 2 }` while its `id` and `anchor` are equal to what went in, the `to`
      point is `{7, 3}`, and each comparison carries its own `assert_ne!` on the value that moved.
      Then **draw**: the displaced connector must draw exactly the cells the connector built from
      the two positions the rule yields — `from` a reference to the same side with offset `(0, 2)`
      and `to` at `{7, 3}` — using the `cells`, `drawn`, `draw_of` and `differing` helpers already
      in that module, and `differing` between the arrangement as written and the displaced one must
      be non-empty. The two halves are the point: the value half catches a rule that grew the wrong
      field and the drawn half catches one that grew nothing, which the value half alone would also
      catch but a reader cannot see (B1.1, B1.2, SC-001; quickstart.md "Commit 1")
- [ ] T008 [US1] Contract test: `a_displacement_grows_both_offsets_of_one_connector` in
      `crates/monospace-diagram/src/diagram.rs` — **B1.3**, which the spec calls "the rule above,
      twice, and one test". Both endpoints are references this time, to two different boxes, each
      with its own offset, and one displacement by `Delta { dx: 3, dy: 2 }` grows both by that
      amount with neither `id` nor `anchor` changed; then draw it and compare against the same
      connector with both offsets written at their grown values, which is what "translated rigidly"
      means as cells rather than as an intention. A diagram of two boxes is the fixture; a shape
      holding **one** reference cannot answer this, and an implementation that moved only the first
      endpoint would pass T007 (B1.3, SC-001)
- [ ] T009 [US1] In `crates/monospace-diagram/src/gallery.rs`, add the **third** `block()` to
      `an_endpoint_hangs_from_a_side_and_follows_it`, and **reach it from the arrangement as
      written** rather than from the block beside it. That is the one thing plan.md's measurement 2
      settles and it is easy to get backwards: the second block displaces the **box** four cells
      right, which leaves **both** of the connector's endpoints on `{7, 1}`, so a third block built
      on it that displaced the connector would leave both endpoints on `{7, 3}` and draw nothing at
      all — degenerate the way 083's own B2.1 is, one step further along. So the block builds a
      **second `Diagram`** in the same test — six lines, `small_box(at(0, 0), None)` then
      `hanging_connector`, which is the arrangement as written — and displaces that diagram's own
      connector by `Delta { dx: 0, dy: 2 }`, with the reason in a comment beside it. It asks for
      **`window(8, 4)`**, because the displaced connector lands on the fourth row and `window(8, 3)`
      clips it out entirely; **a snapshot that grew a fifth row of nothing is the signature of the
      window being too short**, so check it rather than accepting it. Reuse the test's existing
      `labelled` string and the `block()` helper, which takes a size per block, and add **no**
      `<!-- render: -->` marker anywhere (Q4; D2; B3.4, SC-004; plan.md "The gallery's third block
      cannot be reached from the block beside it"). Then update the rustdoc above the test, which
      says "The first block is … and the second is …", and run
      `cargo insta test --review -p     monospace-diagram -- an_endpoint_hangs_from_a_side_and_follows_it`
      and **read** the offered snapshot: the first two blocks must be byte for byte what they are,
      and the third must be exactly B1.1's picture — a box three rows tall with the connector's
      route on the fourth row, three cells clear of the border — because the block measures to the
      claim rather than to a new one (B1.1, B3.4; quickstart.md "The gallery's third block")
- [ ] T010 [US1] Contract test: `an_offset_that_saturated_stays_saturated` in
      `crates/monospace-diagram/src/diagram.rs` — the **first** of the spec's two derived
      arrangements, derived from the rule rather than decided by it, and pinned so a later slice
      that changes it has to say so. Build the connector with a `from` offset of
      `Delta { dx: i32::MAX, dy: 0 }`, displace it by `Delta { dx: 1, dy: 0 }` and assert the offset
      is **still** `i32::MAX` and **not equal** to a wrapping value; then displace it back by
      `Delta { dx: -1, dy: 0 }` and assert it is `i32::MAX` again, because `grow` keeps no memory.
      Draw it as well: the connector draws **nothing** while `the_unrelated_box()` — the helper
      already in that module — is unchanged, which is the asymmetry between the two saturations: an
      absolute position that saturates draws nothing and an offset that saturating draws a very far
      away endpoint. This is the arithmetic of B1 asked of the same value three times (spec.md _Edge
      cases_; SC-001; data-model.md "The two derived arrangements, stated rather than discovered")

**Checkpoint**: a displacement reaches a reference's offsets, the gallery draws what the rule draws,
and the rule is not a no-op dressed as one.

**Commit**: `feat(diagram):` T002-T010, ticking T006-T010's boxes with it (plan.md commit 1). T002
and T005 land here too — no user story phase begins before the rule it tests.

---

## Phase 4: User Story 2 - The two directions are one slice and two rules (B2, Priority: P1)

**Goal**: displacing the figure a reference hangs from still carries the endpoint with it and leaves
the gap unchanged — the rule #83 pinned, untouched — while displacing the figure that **holds** the
reference moves the endpoint and leaves the other figure where it was. Both are in one test because
an implementation that reached the same place in both directions, by rewriting the shape a reference
names, would satisfy each on its own and draw neither picture.

**Independent Test**: displace the box four cells right and the endpoint follows the side it hangs
from with the gap unchanged; displace the connector two cells down and the figure does **not** bend,
it slides. Ask both in one test and both must hold (spec.md B2.1, B2.2, B2.3; SC-002).

### Tests for User Story 2

- [ ] T011 [P] [US2] Contract test: `both_directions_move_the_endpoint_differently` in
      `crates/monospace-diagram/src/diagram.rs` — **B2.3**, and its doc comment carries the reason
      the spec gives for one test rather than two. Each half displaces the **same arrangement** by a
      delta and asserts what the endpoint did: displacing the box four cells right leaves the gap
      between border and endpoint unchanged and the route's length unchanged, which is 083's own
      snapshot and the assertion that must **keep** passing; displacing the connector two cells down
      moves the endpoint with the figure and leaves the box exactly where it was, which is what T007
      would not catch on its own because T007 asks nothing about the figure being displaced. Assert
      both by resolved position **and** by drawing, and carry the `assert_ne!` that says each
      displacement changed something (B2.1, B2.3, SC-002; quickstart.md "Commit 1")
- [ ] T012 [US2] Contract test: `a_reference_that_resolves_to_nothing_still_does` in
      `crates/monospace-diagram/src/diagram.rs` — **SC-003**, and the assertion that makes it worth
      writing is that **the offsets grew** as well as that the reference still resolves to nothing.
      A `displaced_by` that grew nothing would satisfy "resolves to nothing" by doing exactly what
      the code does today, which is the bug this slice exists to end. **Two** cases, because there
      are two ways to resolve to nothing: a reference to an identity the diagram does not hold, and
      a reference to an anchor its kind does not answer — a connector, which is the answer that
      keeps a chain of references one link long. Each asserts the figure holding it is **absent from
      the output**, that every other shape's cells are **unchanged** (`the_unrelated_box()` is the
      shape that has nothing to do with either end), and that nothing panics or fails (SC-003;
      spec.md _Edge cases_; quickstart.md "Commit 1")
- [ ] T013 [US2] Contract test: `displacing_the_box_and_then_the_connector` in
      `crates/monospace-diagram/src/diagram.rs` — the **second** derived arrangement, and the one
      the spec's edge case spells out in full. Displace the box two cells down, then displace the
      connector two cells down, and the connector ends up **four** down with its gap grown by two,
      because each displacement belongs to one figure and both figures moved. Assert the resolved
      endpoints by coordinate, not by picture: what is being claimed is arithmetic, and a route's
      cells in a small window say less about it than `{7, 4}` and the box's own `{0, 2}` do. Say in
      the doc comment that this is **derived from the rule rather than decided by it**, and that no
      displacement moves an anchor and its holder together and keeps the gap — moving the box
      carries the endpoint, moving the endpoint grows the gap — so a later slice that changes it has
      to say so rather than discover it in a picture (spec.md _Edge cases_; SC-002; data-model.md
      "The two derived arrangements, stated rather than discovered")
- [ ] T014 [US2] **Make the rule fail on purpose before trusting it.** Restore the old arm —
      `reference @ Self::Reference(_) => reference.clone()` — and run
      `cargo test -p monospace-diagram both_directions_move_the_endpoint_differently` and
      `cargo test -p monospace-diagram a_reference_that_resolves_to_nothing_still_does`: both must
      go **red**, which is what shows they ask the rule rather than the code. Then put the arm back
      and confirm both green again. A green run only proves the command ran (quickstart.md "Commit
      1"; constitution principle IV)

**Checkpoint**: the two directions are distinguishable from outside and neither broke.

**Commit**: the rest of `feat(diagram):` — T011-T014 tick with T002-T010 (plan.md commit 1). The
three behaviors in B1 and B2 are one commit, and the split into two phases is for traceability
rather than a license to make two.

---

## Phase 5: User Story 3 - The arrow moves, and two pictures carry the evidence (B3, Priority: P1)

**Goal**: the shipped demonstration grows a **sixth** captioned picture — the fifth with the arrow
two rows lower and **both boxes standing exactly where they stood**. Today's sixth picture is the
fifth byte for byte, measured, because at that point both of the arrow's endpoints are references
and a displacement reaches neither: the bug reproducing in the shipped binary, which is what makes
this slice demonstrable rather than merely correct.

**Independent Test**: `cargo run -q -p monospace-cli` prints **six** captioned pictures; the sixth
is the fifth with the arrow two rows lower and nothing else moved, which is asked cell by cell
rather than read; the first five are byte for byte what they were; a path still prints one picture
and nothing else; and `git diff --stat crates/monospace-cli/assets/demo.json` is empty (spec.md
B3.1, B3.2, B3.3; SC-004).

The step itself, which is main.rs's own code beside the four changes it already makes, and **not** a
field in the description format (ADR-0035):

```rust
if let Some(moved) = diagram
    .get(&the_arrow)
    .map(|shape| shape.displaced_by(Delta { dx: 0, dy: 2 }))
{
    diagram.replace(&the_arrow, moved);
}
out.push_str("\nWith the arrow displaced as well:\n");
out.push_str(&picture(&diagram, &catalog, origin, size));
```

### Implementation for User Story 3

- [ ] T015 [US3] In `crates/monospace-cli/src/main.rs`, add the sixth step to `demonstrate` — the
      block above, after the fifth picture's push. Name the delta beside the two the function
      already carries (`by` is three down and `four_right` is four right), **beside them and not by
      reusing either**: this function holds its deltas deliberately rather than taking them from the
      format or the binary, and a sixth step that moved three rows would land the arrow on cells the
      shipped description already draws. The `if let` is not optional: `get` and `replace` are
      no-ops on an identity the diagram does not hold, which is what keeps a one-shape description
      demonstrating at all. Give the caption the exact wording the quickstart's `sed` splits on —
      `"\nWith the arrow displaced as well:\n"` — and add a comment saying the sixth picture is the
      fifth with the arrow two rows lower and **both boxes where they were**, because that is the
      claim a reader checks with their eyes (B3.1, SC-004; ADR-0035; quickstart.md "B3.1")
- [ ] T016 [US3] In the same file, turn `demonstrated_pictures` from a five-tuple into a
      **six**-tuple and fix its doc comment, which reads "The demonstration's five pictures" and
      "the five are then comparable with each other". Its `next_picture` closure needs **no**
      change: it splits on the blank line, strips the trailing newline and puts one back, and the
      sixth block is the last of the output so the normalization the first five already get applies
      to it identically — which is what research.md Q1 measured, and the reason the sixth compares
      equal to the fifth today rather than one character apart. **Of its ten call sites, six need an
      edit and four do not**: the ones that destructure positionally or index `pictures.0` to
      `pictures.4` are
      `the_tenth_entry_naming_a_reference_leaves_all_five_pictures_exactly_as_they_were` (two sites,
      lines 404 and 405),
      `the_third_picture_moves_one_figure_and_the_fourth_takes_that_     figure_out` (line 508),
      `the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it` (line 552),
      `an_empty_description_demonstrates_as_five_identical_pictures` (line 631) and
      `one_shape_demonstrates_as_two_copies_of_itself_and_then_an_empty_window` (line 661); the four
      that bind `(first, ..)` or `(first, second, ..)` are at lines 274, 319, 440 and 452 and
      compile unchanged against a six-tuple. **Count them before editing rather than from this
      list**, and if it comes out as something other than ten call sites write down what it came out
      as (B3.1, B3.3; research.md Q1; constitution principle IV)
- [ ] T017 [US3] In the same file, update the three tests whose **names and counts** say five.
      `a_bare_run_prints_five_captioned_pictures_the_first_being_the_description_as_written` becomes
      `a_bare_run_prints_six_captioned_pictures_the_first_being_the_description_as_written`, with
      the `assert_eq!` count at **5** becoming **6** and the doc comment's two mentions of five
      corrected; it still pins **no caption's wording**, and the sixth is a caption like the other
      five. `an_empty_description_demonstrates_as_five_identical_pictures` becomes
      `..._six_identical_pictures` with a sixth `assert_eq!` beside the five, because an empty
      description has no `#10` and the sixth step is a no-op on an identity it does not hold — which
      is `get` returning `None` and is what the test is for.
      `one_shape_demonstrates_as_two_copies_of_itself_and_then_an_empty_window` keeps its name,
      which is still true, gains `assert_eq!(pictures.4, pictures.5)` — the sixth equal to the
      fifth, for the same reason — and has its doc comment's "five pictures" corrected. **No other
      test's name changes**: `the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it` still
      describes the fifth, and `a_path_prints_one_picture_and_nothing_else` still describes a path
      (B3.3; quickstart.md "Commit 2")
- [ ] T018 [US3] In the same file, rename
      `the_tenth_entry_naming_a_reference_leaves_all_five_pictures_exactly_as_they_were` and extend
      it, because its name and its claim both come out of date with a sixth picture. The name
      becomes
      `the_tenth_entry_naming_a_reference_leaves_the_first_five_pictures_exactly_as_they_were`: the
      first five are compared against what they were, and the **sixth** is compared between the two
      runs — it has no "before", because it is this slice's own step — with a comment saying so. The
      sixth compares equal because both runs displace the same arrow: the run that spells `to`
      outright has it grown from `{22, 4}` and the run that names a reference to `#5`'s bottom with
      offset `(1, 1)` has that offset grown to `(1, 3)` and resolving to `{22, 6}`. The `assert_eq!`
      between the two five-tuples at lines 406-409 becomes six, the doc comment's "all five pictures
      come out byte for byte what they were" says the first five, and its last assertion — that the
      fourth and the fifth still differ, because the demonstration's own change and not the file's
      is what the fifth shows — is unchanged and still passes (B3.2, B3.3, SC-004)
- [ ] T019 [P] [US3] Contract test: `the_sixth_picture_moves_only_the_arrow` in
      `crates/monospace-cli/src/main.rs` — **the claim B3.1 makes and nothing else pins**: the sixth
      differs from the fifth **only** in the cells the arrow holds before and after, which is the
      only statement that says both boxes stood still. Model it on
      `the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it` and reuse the `differing` helper
      beside it: `differing(&fifth, &sixth)` must be non-empty, and every cell it names must lie
      inside the cells `#3` held as `x 13..16, y 2..4` or `#5` held as `x 20..23, y 1..3` — quoted
      in the test rather than read from the code that produces them, because a contract test that
      asks the demonstration the same questions it answers itself checks nothing. Add the second
      half `the_fifth_picture…` already carries in the other direction: cells inside each box's
      footprint that the fifth wrote and the sixth does not would be the box moving, and there must
      be none. Its doc comment names B3.1 and SC-004. Then **correct the command
      [quickstart.md](quickstart.md) gives for it**:
      `cargo test -p monospace-cli the_sixth_picture     -- --exact` filters **nothing** against
      this name, because `--exact` matches a whole name and a prefix is not one — the command that
      runs it is `cargo test -p monospace-cli     the_sixth_picture`, and a check that silently runs
      zero tests is the same silent no-op this slice exists to end (B3.1, SC-004; quickstart.md
      "Commit 2"; constitution principle IV)
- [ ] T020 [US3] The three diffs, run in one place, and they are the other side of every claim in
      this phase. `cargo run -q -p monospace-cli > /tmp/demo-after.txt` against
      `/tmp/demo-before.txt` **cannot** be diffed whole — the fifth block's caption is the split
      point, so compare `sed -n '1,/With the arrow now hanging/p'` on both and it must print
      **nothing**; `cargo run -q -p monospace-cli crates/monospace-cli/assets/demo.json` against
      `/tmp/file-before.txt` must print nothing either, since a file's picture is exactly the one it
      was; and `git diff --stat crates/monospace-cli/assets/demo.json` must print **nothing**, which
      is B3.2 and SC-004's cost claim — a diff there means the sixth picture was put in the file
      rather than in the code, and the format would then be carrying a field ADR-0035 keeps out of
      it. Watch the **same** diff come back with differences this time, which is the measurement
      research.md Q1 could only take against a temporary spike (B3.1, B3.2, B3.3, SC-004;
      quickstart.md "Commit 2")

**Checkpoint**: a person who runs the application sees six pictures, the sixth with the arrow two
rows lower and both boxes standing still, and the file those pictures came from is untouched.

**Commit**: `feat(cli):` T015-T020, ticking their checkboxes with it (plan.md commit 2). T020 is
three diffs and not a commit.

---

## Phase 6: Records this slice writes and amends

**Purpose**: three `docs` commits, none of them a behavior, and so none carrying a `[Story]` label.
They are placed here rather than in the Polish phase because plan.md orders them after the code —
the first of them is `docs(model)` and **follows** the rule, which is where 082's D3's one-sentence
change to §3 landed (`505fd0d`).

- [ ] T022 In `docs/diagram-model.md`, take §11 _Open questions_' third bullet **out** — _What does
      displacing a figure that holds a reference mean?_ — and replace it with a bullet about
      **moving a set**: whether moving a set of figures keeps their gaps is still open, and what
      would settle it is the first consumer that displaces more than one figure at a time, which is
      a selection. Name in that bullet what this slice settled and what it left: that the two
      directions are **not** the same rule, and that nothing moves a set. The question's own trigger
      has fired twice, in 082 and in 083, and this is the slice it named. **Amend no other
      section**: §4 _Positions_ already states the rule this slice makes true and §9 _Changing a
      diagram_ already points at it, so both are left byte for byte — plan.md's table says so and
      the spec's _What this slice implements_ says so (B1; §11's own trigger; plan.md commit 3)
- [ ] T023 In `docs/decisions/`, write the record D1 answered — **one ADR for this rule alone**,
      beside and citing
      [ADR-0041](https://github.com/andresmoschini/monospace/blob/main/docs/decisions/0041-resolve-a-position-through-a-reference.md),
      which is the record the same position rests on today. Take the next free number in the
      directory — **0067** is free today, and if it is not when this task runs, take the next one
      and write down what it was rather than inventing a number — and name the file
      `0067-displace-a-figure-holding-a-reference-by-growing-its-offsets.md`, verb first. Fill
      **`docs/decisions/adr-template.md` whole**, at **`load-bearing`**, because the constitution's
      own test says so rather than taste: `monospace-cli` calls the method, so something outside the
      module depends on it, and reversing the rule leaves every consumer's displaced reference where
      it stood. Its ceiling is **150 lines**. Three things it must carry that a thin record would
      not: (1) **the pictures**, because its options differ in what they draw, which the
      constitution's _Show the rendering_ holds a `load-bearing` record to — the chosen rule's
      translation, the declined alternative's cascade (the place the reference resolves to, which
      moves **another** figure and contradicts both §4's "a displacement is a property of one
      figure" and §9's "nothing cascades"), and today's silent no-op, each labelled on the spot as
      Generated or Hypothetical; (2) the three D1 answers in the template's own fields, not the
      sheet's format — **consider** the record over 082's whole displacement subject and declined it
      for the reason D1 gives, which is that a record is sized to its subject and the other three
      rules were never weighed; and (3) a **Confirmation** naming the six tests and the gallery
      block T007-T013 add, since that is how anyone tells the decision is being followed. **Do not
      revise ADR-0041** — its subject is resolution, a displacement resolves nothing, and a 178-line
      record would only grow. **Add no `<!-- render: -->` marker to it or to anything else**: a
      marker reads a description and a description cannot displace anything (Q4; D1; ADR-0035,
      ADR-0064; principle VI; plan.md "Artifacts")
- [ ] T024 [P] Add the row for T023's record to `docs/decisions/README.md` **as a whole row** —
      link, title and status written out together. A partial edit to that table leaves the rest of
      the row on the line below and prettier then reflows the damage rather than rejecting it, which
      is silent corruption of the one table every record is listed in (AGENTS.md "Facts that are in
      the code and in no document")

**Commit**: `docs(model):` T022, then `docs(adr):` T023 and T024 (plan.md commits 3 and 4).

---

## Phase 7: Polish & Cross-Cutting Concerns

- [ ] T025 Run `cargo xtask check` and confirm every step is green, **including `wasm`** — which
      compiles `monospace-core`, `monospace-diagram` and `monospace-glyph-sets`, so it covers the
      new arithmetic with no change to `xtask` and no new check, which is why principle III's
      two-commit rule does not apply. The new method is `pub(crate)` inside `monospace-diagram` and
      reaches nothing from the core: two `saturating_add` calls on `i32`. Watch the **output**
      rather than the exit code: `rustfmt` reports that `group_imports` needs nightly and exits 0,
      and so does a step that finds something it cannot fix. The `render` step must still answer
      **25** markers and `numbering` must be green, which is where `0067` shows as taken rather than
      free (SC-006; plan.md Constitution Check, principles III and VII)
- [ ] T026 Confirm the two claims this slice **accepts with nothing to verify it** are named as such
      rather than described as tested, as principle IV asks and plan.md's re-check does on the spot:
      that **an endpoint pushed outside the window by the gap growing is clipped without a report**
      — the model's own rule for any figure, carried forward and nothing new; and that **a displaced
      figure leaves every other shape byte for byte**, which is a claim about the whole diagram
      rather than about the value that moved, and which T012 checks only for the shapes it names.
      Look for the first in §4 and §9 of `docs/diagram-model.md` and the spec's _Edge cases_, and
      the second in T012's doc comment and T023's **Confirmation**, and leave each where it is —
      **this task is a check that the record says so, not an edit** (constitution principle IV;
      plan.md "Re-checked after Phase 1")
- [ ] T027 Take the counts this slice claims and check them rather than asserting them:
      `cargo test --workspace` green at **28 / 19 / 114 / 72 / 15 / 61**, which is today's **27 / 19
      / 114 / 66 / 15 / 61** plus seven tests and no test removed but T006's. Confirm no `.snap.new`
      file is left behind under `crates/monospace-diagram/src/snapshots/gallery/`, that
      `a_path_prints_one_picture_and_nothing_else` passes **unchanged** — it is the guard that a
      path still prints one picture and nothing else, which is what `cargo xtask render` embeds —
      and that `git status` shows no `scratch_143.rs` and no leftover spike from research.md's
      measurements. **No characterization report is owed**: research.md Q3 measured 1916 renderings
      across 16 files and none of them can express this change, because `monospace-core` has no
      displacement at all and `sweep.rs` never reads the demonstration (SC-004, SC-005; research.md
      Q3; constitution, Testing)
- [ ] T028 Append an entry to `docs/learning-log.md` for this increment: what was learned about Rust
      design and idiom, what was learned about working this way, and optionally a trade-off worth
      remembering. Three are already paid for and worth writing down rather than rediscovering — a
      **private method nothing calls is red under `-D warnings`**, so "structural first, then
      behavioral" has no room when the new thing is private and unused and part one of the plan had
      promised the split; saturation is the right default for a _second_ addition because a wrapped
      coordinate can land inside a window a caller could really hold, and keeping no memory is why a
      displacement back does not undo a saturating one; and **four places declared the no-op this
      slice reverses while the specification named three**, which is what a
      `grep -rn "143" --     include=*.rs` is for and what reading the spec is not (constitution
      principle II; Q5, Q6)

**Commit**: `docs:` T028. T025-T027 are a gate run and two observations, not commits.

---

## What this slice must not add

Each of these is a decision already answered on the sheet or a rule the model states. Reaching past
one means stopping and asking, not deciding and recording afterwards (constitution, No decision
outside the sheet).

- **No way to move a set of figures, no selection and no group.** Nothing displaces more than one
  figure today — there is no selection, no group and no consumer that moves a set — and the
  arrangement two displacements produce is derived, not decided. What would force an answer is the
  first consumer that displaces more than one figure at a time (P2 clarification; spec.md "What this
  slice does not decide")
- **No amendment to §4 or §9 of `docs/diagram-model.md`, and no sixth change in §9's table of
  five.** §4 already states that a displacement reaches a reference's offsets and §9 already points
  there; §11's bullet is the only thing that comes out (plan.md "Artifacts"; spec.md "What this
  slice implements")
- **No `<!-- render: -->` marker in any artifact**, and no hand-drawn picture standing unlabelled. A
  marker reads a description and a description cannot displace anything, and separately **no marker
  in the repository can hold the demonstration's canvas**: the widest is 24 columns and the tallest
  12 rows, where the demonstration is 50 by 13 (Q4; ADR-0064; SC-005)
- **No second meaning for an offset** — no way to say "one cell out from that side" without naming
  the axis. The displacement reads the offset in the screen axes, which keeps it one addition
  ([#146](https://github.com/andresmoschini/monospace/issues/146); spec.md "What this slice does not
  decide")
- **No chain of references longer than one link, no anchor on a connector and no cycle.** A
  displacement changes a reference's offsets and nothing about which figure it names or which side
  it hangs from, and a connector still answers no anchor
  ([#89](https://github.com/andresmoschini/monospace/issues/89); ADR-0041)
- **No report of a figure the diagram could not draw** — neither a figure pushed out of the window
  nor a gap that reaches nothing is a fault to report here
  ([#88](https://github.com/andresmoschini/monospace/issues/88); SC-003)
- **No error path on a displacement.** It builds a value and cannot fail, so there is no `Result`,
  no `Option` and nothing to report; a reference naming a shape the diagram does not hold comes back
  the same reference with a larger offset (data-model.md "`Position::displaced_by`"; SC-003)
- **No change to `leaving` or `terminal`**, and nothing derived from an anchor. §6 says both are the
  caller's, so a displaced connector may leave a side in a direction that draws something nobody
  wants ([#89](https://github.com/andresmoschini/monospace/issues/89))
- **No fourth contract, and no revision of ADR-0041.** The three contracts 081, 082 and 083 recorded
  are unchanged and writing the rule again is the duplication principle VIII refuses; the record the
  maintainer wrote in D1 is the one home (plan.md "Artifacts"; D1)
- **No characterization test and no ADR-0053 report.** 1916 renderings across 16 files, none of
  which can express the change (research.md Q3)
- **No `refactor` commit for `Delta::grow`.** It is dead code until the arm calls it and
  `-D warnings` turns that into an error, so one `feat` carries both halves and principle V has no
  structural change to govern (plan.md "The arithmetic cannot be the `refactor` part one promised")
- **No new dependency, no new crate, no new gate step, no change to `xtask`, and nothing in
  `monospace-core`** — the core has no displacement at all and `displaced_by` appears nowhere under
  `crates/monospace-core` (principle III, principle VII; research.md Q3)
- **No field in `assets/demo.json` and nothing in the description format.** Twenty-four markers read
  it and a sixth picture costs one call (B3.2; ADR-0035; SC-004)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: nothing to do; T001 captures the baseline everything else is compared to.
- **Foundational (Phase 2)**: depends on nothing, and **blocks both user story phases** — a test
  that names a method which is not there does not fail, it does not compile.
- **User Story 1 (Phase 3)**: depends on Phase 2. No dependency on US2 or US3.
- **User Story 2 (Phase 4)**: depends on Phase 2, and shares commit 1 with US1. Its test T011
  asserts what US1's rule did **not** break, so it is worth reading beside T007 rather than instead
  of it.
- **User Story 3 (Phase 5)**: depends on Phases 2 to 4 and **cannot precede them**. plan.md is
  explicit that commit 2 cannot come first, because the sixth picture is the evidence for the rule
  rather than an independent change, and a sixth picture equal to the fifth is the bug rather than
  the feature.
- **Records (Phase 6)**: T022 depends on Phase 2 — it follows the code rather than preceding it.
  T023 and T024 depend on T022's subject and on nothing else.
- **Polish (Phase 7)**: depends on all of them.

### Within Each User Story

- US1: T006 reads T003; T007, T008 and T010 read the arm and the `diagram.rs` helpers; T009 reads
  the same rule through `Shape::displaced_by` and moves the one snapshot in the slice.
- US2: T011-T013 all read T002 and T003 and may be written in any order among themselves; **T014
  comes last**, because a deliberate red is one agent's work and it needs all three tests in place.
- US3: T015 before T016 (the tuple cannot hold six pictures the demonstration does not print), T016
  before T017-T019 (each of those takes the tuple apart), T019 after T015 (it compares the fifth
  with the sixth), and T020 after all of them.

### Parallel Opportunities

Parallelism here is genuinely lower than in 083 and 148, and the reason is worth stating rather than
padding: **twenty of the twenty-eight tasks edit one of two files**, `diagram.rs` (six tests) and
`main.rs` (six edits and one test). Five tasks carry `[P]`, and they are exactly the ones whose file
is free while another task runs:

- T005 with T002, T003 and T004 — `shape.rs` against `delta.rs` and `position.rs`.
- T007 with T006 and T009 — `diagram.rs` against `position.rs` and `gallery.rs`. T008 and T010 are
  the same file and are **not** marked `[P]` against T007: they can be written in any order among
  themselves, but only one agent should hold `diagram.rs` at a time.
- T011 with T014 — T014 changes nothing and restores what it changed, so it waits for T011 and the
  two tests beside it.
- T019 with T015-T018 — all six are `main.rs`, and this is the one of them an agent can take while
  the demonstration's step and the tuple are still landing.
- T024 with T023 — `docs/decisions/README.md` against `docs/decisions/0067-*.md`.

Two tasks are deliberately **not** marked `[P]` and are worth naming: **T009** is the only task that
moves a snapshot and `cargo insta review` is a single reviewer, and **T014** is a deliberate red
that one agent should run and restore.

---

## Implementation Strategy

### The demonstrable increment is Phases 2 to 5, and there is no smaller one

There is no MVP phase to stop at, and saying so is more useful than naming one. Phase 3 alone is a
`feat` whose evidence is a gallery snapshot a reader has to go looking for; Phase 5 alone is a sixth
picture that is byte for byte the fifth, which is the bug rather than the feature. The rule and the
picture that proves it were shipped together in plan.md's commit order for exactly that reason, and
the smallest increment this slice has is **commits 1 and 2 together**.

### MVP First (Phases 2 to 5, two commits)

1. Phase 2 → displacing a figure that holds a reference grows its offsets, and the three paragraphs
   and one sentence that said otherwise now say what it does (commit 1's production half).
2. Phases 3 and 4 → six contract tests and the gallery's third block: the rule by value and drawn,
   two references rigidly, both directions distinguishable, nothing that resolves to nothing drawn,
   and the two derived arrangements pinned (commit 1).
3. Phase 5 → the sixth picture, its caption, the six-tuple and its callers, and the test that pins
   the sixth against the fifth cell by cell (commit 2).
4. **STOP and VALIDATE**: `cargo test --workspace` green at 28 / 19 / 114 / 72 / 15 / 61;
   `cargo run -q -p monospace-cli` prints six captioned pictures and the sixth is the fifth with the
   arrow two rows lower and both boxes where they were;
   `git diff --stat crates/monospace-cli/assets/demo.json` is empty; a path prints one picture and
   nothing else.

### Incremental Delivery

1. Phase 2 → the rule lands, alone in a commit it cannot be split from (commit 1, first half).
2. Phases 3 and 4 → the six tests and the gallery block, and the rule is now something a broken
   implementation fails (commit 1, second half).
3. Phase 5 → the shipped run carries the evidence, and the demonstration reproduces the defect no
   longer (commit 2).
4. Phase 6 → §11's bullet is replaced by one about moving a set, and the rule has one home that is
   not a fourth contract (commits 3 and 4).
5. Phase 7 → the gate is green, the two untested claims are named as untested, and the increment is
   closed (commit 5).

Each commit follows plan.md: **one** `feat(diagram)` carrying both halves rather than a `refactor`
and a `feat`, one `feat(cli)`, three `docs`, and every commit leaves `cargo xtask check` green
(constitution principles II, III and V).

### Parallel Team Strategy

With two agents:

1. Agent A takes Phase 2 and then Phase 3; Agent B takes the records in Phase 6 once T022's subject
   exists — but **not before**, because a record that follows nothing is a record of an intention.
2. Inside Phase 5, the sixth step and its test are one file and do not divide.

The honest answer is that this slice is **not** a parallel slice. Twenty of twenty-eight tasks touch
one of two files, and the two commits that matter are ordered by what they are evidence for.
