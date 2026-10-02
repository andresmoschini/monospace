---
description:
  "Task list for feature 142: taking a shape out leaves the figures that hung from it where they
  were"
---

<!-- The feature directory below is the issue title truncated at forty characters by
     `cargo xtask spec`, which landed inside a word: cspell:ignore referen -->

# Tasks: Taking a shape out leaves the figures that hung from it where they were

**Input**: Design documents from `/specs/142-taking-a-shape-out-leaves-the-figures-th/`

**Prerequisites**: [plan.md](plan.md), [spec.md](spec.md), [data-model.md](data-model.md),
[research.md](research.md), [decisions.md](decisions.md), [quickstart.md](quickstart.md). **There is
no contract**, and the reason is in `plan.md`'s _Artifacts_: the slice changes no signature, no
field and no type, `Diagram`'s whole surface is what
[081](../../081-a-shape-can-be-removed-and-replaced/contracts/diagram-api.md) recorded with
`remove`'s **behavior** changing under it, and that is what the ADR is for. Writing the rule a
second time in a fourth contract is the duplication principle VIII refuses.

**Tests**: Requested by the spec's **Testing expectations** — five contract tests in
`monospace-diagram`, **one test there rewritten rather than deleted**, one new contract test in
`monospace-cli`, two more there **renamed**, one gallery block and **no characterization at all**.
research.md Q6 measured 1916 renderings across 16 files and none of them can express a removal, so
no ADR-0053 report is owed and `cargo insta review` is run once, on the gallery.

**Organization**: Tasks are grouped by the spec's behaviors B1 to B3, in the order
[plan.md](plan.md)'s five commits deliver them — that order is forced by constitution principles II
and V rather than chosen, so it is the priority order here too. **B1 and B2 share one commit** with
the body they read, and the split into two phases is for traceability, not a license to make two.

**Order within a phase**: implementation before tests, where a test cannot exist before what it
reads. That inverts the usual listing and it is deliberate: in Rust a test naming a method that is
not there does not fail, it does not compile. Task IDs are in execution order throughout.

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
records go to `docs/decisions/` and four sections of `docs/diagram-model.md` are amended, and
`assets/demo.json` is **not** edited: the seventh picture is a change `monospace-cli` makes in its
own code, beside the six it already makes (B3.2, ADR-0035).

### The baseline, measured again on this branch

Every number below was measured on 2026-10-02 against the tree this branch points at, and is what
the tasks carry rather than what an artifact estimates.

```text
   28  monospace-cli unit tests after this slice   (28 today) — 1 added, 2 renamed, 0 removed
   19  monospace-cli integration tests              (unchanged)
  114  monospace-core                               (unchanged)
   77  monospace-diagram after this slice          (72 today) — 5 added, 1 rewritten (T005), 0 removed
   15  monospace-glyph-sets                         (unchanged)
   61  xtask                                        (unchanged)
```

`cargo test --workspace` is green at **28 / 19 / 114 / 72 / 15 / 61** today, which is exactly
`quickstart.md`'s target minus the six tests this slice adds and the one it rewrites — the count is
unchanged by T005, which rewrote a test rather than adding one, so **77** is still the target. A
bare run prints **six** captioned pictures (`grep -c '^[A-Z].*:$'` on
`cargo run -q -p monospace-cli`) and `cargo xtask render --check` answers **26** generated pictures.

**One measurement below is the reason this file is longer than a list.** The body of `remove` was
written, the whole workspace tested, and **exactly one existing test went red**:
`taking_the_referenced_figure_out_stops_the_connector_and_changes_nothing_else` at
`diagram.rs:2505`. The specification named none of them, and research.md Q6's grep was for the
_behavior_ rather than for this issue's number. Its name and both halves of its claim are false
under the freeze, so it is rewritten rather than deleted — **T005** — and 143's fifth place is this
slice's first. Nothing else in either crate moved: the demonstration's existing fourth step removes
`#1`, which holds no reference, and the six pictures it prints are byte for byte what they were
(research.md Q6, and measured again here).

---

## Phase 1: Setup

No setup is needed: no dependency is added, no crate is added, the toolchain is unchanged, and no
gate step is added — `wasm` already compiles `monospace-diagram` (plan.md Constitution Check,
principles III and VII). T001 captures what must not move before anything is edited.

- [ ] T001 Run the three baseline commands and keep the output:
      `cargo run -q -p monospace-cli > /tmp/demo-before.txt` then
      `grep -c '^[A-Z].*:$' /tmp/demo-before.txt`, which must print **6**;
      `cargo run -q -p monospace-cli crates/monospace-cli/assets/demo.json > /tmp/file-before.txt`;
      and `cargo xtask render --check`, which must answer **26** generated pictures. **Count rather
      than assume** — a bare run's picture count is the one number this slice moves and every claim
      about the first six rests on it (B3.1, B3.2; quickstart.md "Before touching anything")

---

## Phase 2: Foundational — the rule, and every sentence that said otherwise

**Purpose**: one method body and the three pieces of the crate's own text that declare the opposite
behavior. **Nothing here may touch a test**, because there is no structural commit to make: the gate
runs `cargo clippy --workspace --all-targets -- -D warnings`, the body uses nothing new, and
principle V has no structural half to separate — `Position`, `Reference`, `Endpoint`, `Shape`,
`resolve` and `Diagram`'s signatures are all untouched (data-model.md "What does not change").

The whole of the rule is the body below, and D4's answer is what put it inside `remove` rather than
beside it — no flag, no separate change, and therefore no sixth row in §9's table of five:

```rust
// crates/monospace-diagram/src/diagram.rs — `remove` at line 156
pub fn remove(&mut self, id: &ShapeId) {
    let Some(index) = self.find(id) else {
        return;
    };

    // Pass one reads and collects; pass two writes. Two passes because `Position::resolve`
    // takes `&Diagram` and a reference only resolves while the figure it names is held.
    let mut frozen: Vec<(usize, usize, Pos)> = Vec::new();
    for (at, placed) in self.shapes.iter().enumerate() {
        let Shape::Connector { from, to, .. } = &placed.shape else {
            continue;
        };
        for (slot, endpoint) in [(0, from), (1, to)] {
            if let Position::Reference(reference) = &endpoint.at
                && &reference.id == id
                && let Some(point) = endpoint.at.resolve(self)
            {
                frozen.push((at, slot, point));
            }
        }
    }
    for (at, slot, point) in frozen {
        if let Shape::Connector { from, to, .. } = &mut self.shapes[at].shape {
            if slot == 0 {
                from.at = Position::Absolute(point);
            } else {
                to.at = Position::Absolute(point);
            }
        }
    }

    self.shapes.remove(index);
}
```

- [ ] T002 In `crates/monospace-diagram/src/diagram.rs`, replace the body of `remove` at **line
      156** with the one above, and add `Position` to the file's `use crate::{…}` at **line 9** — it
      names `Position` twice and today imports only `Shape`. Four things in the body are the rule
      rather than style, and each is measured in `data-model.md`: the **two passes** are forced by
      the borrow, since `Position::resolve` takes `&Diagram` and the shape is only held while
      nothing has been written; the **`slot`** is not optional, because one connector may hang from
      the same figure at both ends and those are two different points — measured, `from` → `{3, 1}`
      and `to` → `{2, 2}` for a four-by-three box at the origin; **`&reference.id == id`** is D3's
      answer, the references naming the removed shape and nothing else; and
      **`let Some(point) = …`** is what leaves an unresolved reference alone rather than freezing it
      to nothing. `index` is read before the two passes and is still valid after them, because
      neither changes the length of `self.shapes` — say so in a comment, since it is the one thing
      in the body a reader has to take on trust. **Keep `remove`'s signature, its visibility and its
      `&mut self`**, and add **no** method, no variant and no field anywhere (B1.1, SC-001; D3, D4;
      data-model.md "`Diagram::remove` — two passes, and why there are two")

- [ ] T003 In the same file, rewrite the **two sentences of `remove`'s own rustdoc** that T002 makes
      false, in the same commit that makes them false — a rustdoc is code. The first is **line
      146**, which reads "Takes the shape named by `id` out of the diagram, **changing nothing
      else**", and the second is **line 151**, which reads "a removal changes the holding and
      **nothing else**". D4's answer names the first of the two as the sentence this slice corrects.
      Say what the removal now does to a reference naming it — each becomes the absolute point it
      was resolving to, and nothing else — point at §4 _Positions_ for the rule, and **keep** the
      three paragraphs that are still true: the gap in the order closes, a removal of an identity
      the diagram does not hold changes nothing with no error, no report and no panic, `remove`
      hands back nothing and there is no history to undo, and the counter is untouched so an
      identity is never handed out twice. The last of those is what keeps a put-back possible at
      all, which B1.2 depends on (SC-003; D4; data-model.md "`Diagram::remove` — two passes, and why
      there are two")

- [ ] T004 In the same file, correct the doc comment of
      `a_figure_put_back_under_the_removed_identity_draws_the_connector_again` at **lines
      2543-2552** — **and this one is false twice over, so fix both halves in this commit and not
      one now and one later.** Its paragraph claims that "`remove` frees an identity permanently —
      `add` never hands one out twice, and **there is no `add_under`** — so a removal followed by an
      addition cannot put anything back under the removed one's identity, and the reference stays
      **unresolved** for good". research.md Q5 measured the first falsehood: `add_under` is `pub`
      and has been since 148. The freeze makes the second: after a removal the reference is
      **frozen**, not unresolved, so the sentence describes a state that can no longer be reached by
      this route. The test itself goes through `replace` rather than through a removal and its
      assertions **do not move** — keep every one of them, keep the case and keep the line it still
      earns, which is that a kind change is covered by the same arrangement because a five-cell
      line's right side center is its last cell. Note in the comment what a removal now does
      instead, and that #142 answered the question the comment used to carry (Q5; B1.2;
      data-model.md "The pictures the rule draws")

**Checkpoint**: a removal freezes what hung from it, and every sentence in the crate that said it
did not is corrected. The workspace does **not** compile cleanly yet — T005's test is red by
construction until it is rewritten, which is the next task and lands in this same commit.

**Commit**: `feat(diagram):` T002-T004 (plan.md commit 1, production half). T002 and T003 touch the
same file and land together; **neither carries `[P]`**, because a `[P]` task is one that can land
and be checked on its own and neither of these can.

---

## Phase 3: User Story 1 - A removal leaves what hung from it where it was (B1, Priority: P1)

**Goal**: taking a figure out leaves every reference naming it drawn where it already stood, and the
crate's gallery grows the block that draws it — drawn by the code rather than written by hand, so a
rule that stops holding drops a snapshot instead of nothing (ADR-0064). §4 and §6 gain the sentence
that says what a frozen position **is**, and that sentence follows the code rather than preceding
it.

**Independent Test**: build the arrangement §4 names — a box four cells by three at the origin, a
box three cells by three at `{8, 0}`, and a connector whose `from` is a reference to the first box's
right side carrying an offset of nothing, leaving rightward, with its `to` a plain point at `{8, 1}`
— take the first box out, and it must draw

```text
        ┌─┐
   ─────┤ │
        └─┘
```

with the arrow's route cells byte for byte what they were and the box's ten drawn cells gone. Put
the box back under the same identity and the whole picture returns byte for byte, with the arrow's
`from` a plain point rather than a reference again (spec.md B1.1, B1.2, B1.3; SC-001).

### Implementation for User Story 1

- [ ] T005 [US1] **Rewrite, not delete**,
      `taking_the_referenced_figure_out_stops_the_connector_and_changes_nothing_else` at
      `crates/monospace-diagram/src/diagram.rs:2505`, and rename it to what is now true —
      `taking_the_referenced_figure_out_leaves_the_connector_where_it_was_and_changes_nothing_else`.
      **This is the one existing test the freeze makes red**, measured by writing T002's body and
      running the workspace, and the specification named none: research.md Q6's inventory was a
      `grep` for the behavior rather than for this issue's number, and the same lesson 143 paid for
      a fifth place. Both halves of the name are false under the freeze — the connector does not
      stop, and the diagram does change beyond the holding — so a rename is not a nicety here, it is
      the claim. Keep the arrangement and keep `the_unrelated_box()`, which is the shape that had
      nothing to do with either end and is what makes the second half of the name mean something:
      its cells must be unchanged. What changes is the first assertion, from "the connector drew
      nothing" to "the connector drew exactly what it drew before, minus the cell the removed box's
      border used to share with its arm" — `{3, 1}` reads `─` now and read `├` then, and **that cell
      is the whole difference** between the two pictures, which is why the assertion cannot be
      `assert_eq!` on the buffer and cannot be "nothing moved" either. Its doc comment cites 081's
      B3.1 and SC-004; cite B1.1 and SC-001 beside it and say what changed and why (B1.1, SC-001;
      data-model.md "The pictures the rule draws")

- [ ] T006 [US1] Contract test: `a_removal_freezes_what_hung_from_the_removed_shape` in
      `crates/monospace-diagram/src/diagram.rs`, **asked and then drawn**, which is the spec's
      wording and the order it asks for. By value: build the arrangement through the helpers already
      in that module, resolve `from` **before** the removal and hold the point, take the box out,
      then assert `from.at` is `Position::Absolute` equal to the point it resolved to — and carry an
      `assert_ne!` that it is **not** still a `Reference`, or a body that froze nothing would
      satisfy the first half by doing exactly what the code did before. By picture: the arrow's
      route cells are byte for byte what they were, `{3, 1}` reads `─` and not `├`, the box's ten
      drawn cells are blank, **and no other cell moved** — the last asserted as `differing(…)`
      against a **no-connector baseline** rather than a quoted number, using the `cells`, `drawn`,
      `draw_of` and `differing` helpers already there. That is the specification's own wording and
      the reason research.md Q7's two corrections happened: the count **10** is not `15 − 6`,
      because `{3, 1}` changes glyph rather than going blank. A number written into the test is the
      thing that was wrong (B1.1, SC-001; quickstart.md "Commit 1")
- [ ] T007 [US1] Contract test: `a_shape_put_back_under_the_removed_identity_is_not_re_attached` in
      the same file — **B1.2 and B1.3 together**, because both are about what comes back. Put the
      box back **in place** under the removed identity with `add_under` and assert the picture is
      byte for byte the pre-removal one, while `get` answers a figure whose `from.at` is a plain
      `Position::Absolute` and **not** a reference again: nothing went looking for the reference it
      was, and P3 is what makes the two halves compatible. Then the other half of P3's cost, which
      the specification names in the same scenario: put a figure back **displaced** under the same
      identity and assert the arrow did **not** move to it, because a frozen end is a point and a
      displacement reaches any absolute position's coordinates. And `add` after the removal hands
      back **`#4`**, never `#1`, while the arrow is still drawn — the identity is the diagram's to
      issue and a put-back does not un-issue it (B1.2, B1.3, SC-001; data-model.md "The derived
      arrangements, stated rather than discovered")

- [ ] T008 [US1] In `crates/monospace-diagram/src/gallery.rs`, add the **fourth** `block()` to
      `an_endpoint_hangs_from_a_side_and_follows_it`, with the change named `the box taken out`. Two
      things about it were measured rather than assumed, and both are the kind of thing that is easy
      to get backwards. **It is reached from a third `Diagram` in the same test, not from the block
      beside it**: the two blocks above leave `#1` displaced four cells right, and a removal of a
      figure the arrow's reference is already resolving somewhere else freezes the arrow
      **displaced** — a fourth picture of a different claim, not B1.1's. **The window stays
      `window(8, 3)`**, which is the opposite of what 143's third block beside it needed: that one
      landed the connector on a fourth row and wanted `window(8, 4)`, and a removal lands nothing —
      running it at `window(8, 4)` comes back with a fourth empty row, so **a snapshot that grew a
      fourth row of nothing is the signature here of a window widened for no reason**. The block
      itself is drawn once, in [data-model.md](data-model.md) under "The pictures the rule draws",
      and it is not repeated here: three copies of one measured picture is the restating principle
      VIII refuses. VIII refuses. Reuse the test's existing `labelled` string and the `block()`
      helper, and **add no `<!-- render: -->` marker anywhere**: a description carries no field that
      takes a shape out, so no marker in the repository can hold a picture of this rule (ADR-0064,
      ADR-0035). Update the rustdoc above the test, which today says "The first block is … and the
      second is …" and then describes the third at length, so that the fourth is named beside them —
      and **say in it that the block is drawn by `remove` itself** rather than built by hand, which
      is the whole reason it is in the gallery. Then run
      `cargo insta test --review -p monospace-diagram -- an_endpoint_hangs_from_a_side_and_follows_it`
      and **read** the offered snapshot: the first three blocks must be byte for byte what they are,
      and the fourth must be the block `data-model.md` draws, because the block measures to the
      claim rather than to a new one (B1.1, B3.4; D2; quickstart.md "The gallery's fourth block")

**Checkpoint**: a removal leaves what hung from it where it was, the gallery draws what the rule
draws, and the one existing test that said the opposite now says the opposite thing correctly.

**Commit**: the rest of `feat(diagram):` — T005-T008 tick with T002-T004 (plan.md commit 1). T006
and T007 read T002's body, so they cannot exist before it.

---

## Phase 4: User Story 2 - A removal stops being one of three routes to one picture (B2, Priority: P1)

**Goal**: the three routes that reached one picture byte for byte reach three pictures now, and the
taken-out one is the odd one out — its arrow is still there. So §11's second question dissolves
rather than being answered: no record grows, no change gains a flag, and the two pictures simply
differ.

**Independent Test**: build three diagrams — a reference to an identity **never added**, the same
reference after the shape was **taken out**, and the same reference over an identity **still held by
a figure answering no side** — draw all three, and the taken-out picture must equal neither of the
other two while the other two still equal each other (spec.md B2.1; SC-002).

### Tests for User Story 2

- [ ] T009 [US2] Contract test: `the_three_routes_to_one_picture_are_not_one_picture_now` in
      `crates/monospace-diagram/src/diagram.rs` — **B2.1**, and its doc comment must carry the
      reason the spec gives for one test rather than three. The **taken-out route is compared
      against both of the others and against neither**, which only one place can ask: separately,
      each is a claim about a diagram, and the claim here is about the difference between two of
      them. Keep the still-held route's picture **equal to the never-added route's**, byte for byte
      — that pair is unchanged by this slice and the equality is the control which says the third is
      what moved. Assert the whole buffer rather than a cell count, because the routes differ
      **everywhere the arrow stood** and a count would not say where. Its doc comment names B2.1 and
      SC-002 and says that the third picture's differing is what dissolves §11's second question
      rather than answering it (B2.1, SC-002; P3)

- [ ] T010 [US2] Contract test: `two_connectors_from_one_figure_both_freeze_at_their_own_points` in
      the same file — the specification's _Edge cases_, which is P1 applied **twice** and not a
      cascade. Two connectors hang from the same box at **different anchors and different offsets**,
      so the two frozen points are different, and each connector freezes at its own: assert each
      `from` is `Absolute` equal to what **its own** end resolved to, with an `assert_ne!` between
      the two points. A body that resolved once and wrote twice, or a cascade that walked from the
      box to the first connector and on, would pass T006 and fail here — which is the reason the
      arrangement is not the one T006 already builds. Draw it as well, since the claim is that the
      picture holds two arrows where it held none (spec.md _Edge cases_; SC-001)

- [ ] T011 [US2] Contract test: `a_removal_touches_nothing_else` in the same file — **D3's "and
      nothing else"**, which is a claim about the whole diagram and so deserves its own. Three
      diagrams, one assertion each, and each is a way the rule could be wrong: (1) a connector whose
      `from` names an identity that was **never added**, in a diagram where a removal naming
      **that** identity happens, comes back still holding its `Reference` — the removal did not
      create that broken reference and does not repair it, which is what the `let Some(point)` guard
      is for and what freezing to nothing would silently destroy; (2) a connector with `from` naming
      a figure that **stays** and `to` naming the one taken out keeps a `Reference` in `from` while
      `to` is frozen — the asymmetry **is** the rule, and a body that rewrote every reference naming
      the identity would pass every other test here; (3) **one connector with both ends naming the
      same figure gets two different points**, which is the case a `(index, point)` collection gets
      wrong by writing the first point into both ends, and which is why `data-model.md` makes the
      slot a named part of the body rather than an implementation detail (D3; data-model.md
      "`Diagram::remove` — two passes, and why there are two")

- [ ] T012 [US2] **Make the rule fail on purpose before trusting it.** Keep T002's first pass and
      drop the second, and run the five tests T005-T010 name — or narrower and louder: assert that
      dropping the write pass turns `a_removal_freezes_what_hung_from_the_removed_shape` **red**
      while `a_removal_touches_nothing_else`'s third case still passes, which is what shows the
      tests ask the rule and not the code's tidiness. Then put the loop back and confirm green. A
      green run only proves the command ran (quickstart.md "Commit 1"; constitution principle IV)

**Checkpoint**: a removal is visible without anything remembering it, and the three routes that
reached one picture no longer do.

**Commit**: the rest of `feat(diagram):` — T009-T012 tick with T002-T008 (plan.md commit 1). The
three behaviors in B1 and B2 are one commit, and the split into two phases is for traceability
rather than a license to make two.

---

## Phase 5: User Story 3 - The demonstration grows a seventh picture, with the arrow in it (B3, Priority: P1)

**Goal**: the shipped demonstration prints **seven** captioned pictures and the seventh is the sixth
with the box gone and **the arrow still standing exactly where it stood**. Today's shipped run
removes a figure at its fourth step and no reference with it, so nothing in the binary shows the
defect this slice ends; the seventh picture is where a reader sees it.

**Independent Test**: `cargo run -q -p monospace-cli` prints **seven** captioned pictures; the
seventh is the sixth with exactly the box's twelve cells gone and **the arrow's ten unchanged**,
asked cell by cell rather than read; the first six are byte for byte what they were; a path still
prints one picture and nothing else; and `git diff --stat crates/monospace-cli/assets/demo.json` is
empty (spec.md B3.1, B3.2; SC-001).

The step itself, which is `main.rs`'s own code beside the six changes it already makes, and **not**
a field in the description format (ADR-0035):

```rust
diagram.remove(&the_hung_from);
out.push_str("\nWith the box the arrow hangs from taken out:\n");
out.push_str(&picture(&diagram, &catalog, origin, size));
```

### Implementation for User Story 3

- [ ] T013 [US3] In `crates/monospace-cli/src/main.rs`, add the seventh step to `demonstrate` — the
      block above, after the sixth picture's push and **before** `out` is returned. `diagram.remove`
      hands back nothing, so unlike the four steps above it there is **no `if let`** and no `get`:
      that is D4's answer, and it is why this step reads differently from the five beside it — a
      caller cannot ask for the old behavior by wrapping it in a conditional. Use `the_hung_from`,
      the `ShapeId::new("#3")` the function **already carries** at line 127 for the fifth picture,
      and do not add a second one: `#3` is the box the demonstration hangs the arrow from, so the
      same identity is what the seventh step takes out. Give the caption the exact wording
      `quickstart.md`'s `sed` splits on. Add a comment saying the seventh is the sixth with the box
      gone and **the arrow exactly where it stood**, because that is the claim a reader checks with
      their eyes and it is not obvious from a `remove` (B3.1, SC-001; D4; ADR-0035)
- [ ] T014 [US3] In the same file, turn `demonstrated_pictures` from a six-tuple into a
      **seven**-tuple and fix its doc comment, which reads "The demonstration's **six** pictures"
      and "the six are then comparable with each other", and whose `expect` says "six captioned
      pictures". Its `next_picture` closure needs **no** change: it splits on the blank line, strips
      the trailing newline and puts one back, and the seventh block is the last of the output so the
      normalization the first six already get applies to it identically. **Of its ten call sites,
      six need an edit and four do not**: the ones that destructure positionally or index
      `pictures.0` to `pictures.5` are
      `the_tenth_entry_naming_a_reference_leaves_the_first_five_pictures_exactly_as_they_were` (two
      sites, lines 446 and 448),
      `the_third_picture_moves_one_figure_and_the_fourth_takes_that_figure_out` (line 558),
      `the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it` (line 602),
      `an_empty_description_demonstrates_as_six_identical_pictures` (line 687) and
      `one_shape_demonstrates_as_two_copies_of_itself_and_then_an_empty_window` (line 718); the four
      that bind `(first, ..)` or `(first, second, ..)` are at lines 301, 353, 490 and 502 and
      compile unchanged against a seven-tuple. **Count them before editing rather than from this
      list**, and if it comes out as something other than ten call sites write down what it came out
      as (B3.1, B3.2)
- [ ] T015 [US3] In the same file, update the two tests whose **names and counts** say six.
      `a_bare_run_prints_six_captioned_pictures_the_first_being_the_description_as_written` becomes
      `..._seven_...`, with the `assert_eq!` count at **6** becoming **7** and the doc comment's
      mentions of six corrected; it still pins **no caption's wording**, and the seventh is a
      caption like the other six. `an_empty_description_demonstrates_as_six_identical_pictures`
      becomes `..._seven_identical_pictures` with a seventh `assert_eq!` beside the six, because an
      empty description holds no `#3` and the seventh step is a no-op on an identity it does not
      hold — which is `find` returning `None` and `remove` returning before its first pass, and is
      what the test is for. **No other test's name changes**:
      `one_shape_demonstrates_as_two_copies_of_itself_and_then_an_empty_window` keeps its name,
      which is still true, gains `assert_eq!(pictures.5, pictures.6)` — the seventh equal to the
      sixth, for the same reason — and has its doc comment's picture count corrected.
      `the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it` still describes the fifth, and
      `a_path_prints_one_picture_and_nothing_else` still describes a path (B3.1; quickstart.md
      "Commit 2")
- [ ] T016 [US3] In the same file, extend
      `the_tenth_entry_naming_a_reference_leaves_the_first_five_pictures_exactly_as_they_were` to
      the seventh, because its two `assert_eq!`s destructure six. Its **name stays as it is** and
      the reason is worth a comment rather than a rename: the claim is that naming the far endpoint
      as a reference left the first five exactly as they were, which is still true and is not this
      slice's to restate. The **sixth** it already compares between the two runs and **the seventh
      joins it**, both for the same stated reason — they have no "before", being the demonstration's
      own steps, and both runs produce them. Add the comment saying the seventh is equal for a
      reason that is the **freeze**: `remove(&#3)` freezes the arrow's `from` at `{16, 5}` in both
      runs whatever route it took to get there, which is the same "same cell, two routes to it"
      claim its existing comment makes about the sixth (B3.2, SC-001)
- [ ] T017 [US3] Contract test: `the_seventh_picture_takes_the_box_away_and_leaves_the_arrow` in
      `crates/monospace-cli/src/main.rs` — **the claim B3.1 makes and nothing else pins**: the
      seventh differs from the sixth **only** in the cells `#3` held, which is the only statement
      that says the arrow stood still. Model it on
      `the_fifth_picture_moves_the_box_and_takes_the_arrow_with_it` and reuse the `differing` helper
      beside it: `differing(&sixth, &seventh)` must name **twelve** cells, every one of them inside
      the rectangle `#3` occupied as `x 13..16, y 2..4` — quoted in the test rather than read from
      the code that produces them, because a contract test that asks the demonstration the same
      questions it answers itself checks nothing. Add the second half in the other direction, as the
      fifth-picture test already does: **every cell the arrow held in the sixth must be identical in
      the seventh**, which is where a bare "twelve cells differ" is not enough, because a removal
      that moved the arrow elsewhere and dropped two box rows would satisfy it. Its doc comment
      names B3.1 and SC-001 and says the count is asserted rather than quoted, and why: the arrow's
      footprint is neither contiguous nor a rectangle, so nothing that reads it off a picture gets
      it right (B3.1, SC-001; research.md Q4, Q7)
- [ ] T018 [US3] The three diffs, run in one place, and they are the other side of every claim in
      this phase. `cargo run -q -p monospace-cli` against `/tmp/demo-before.txt` **cannot** be
      diffed whole — the sixth caption is the split point, so compare
      `sed -n '1,/With the arrow displaced as well:/p'` on both and it must print **nothing**;
      `cargo run -q -p monospace-cli crates/monospace-cli/assets/demo.json` against
      `/tmp/file-before.txt` must print nothing either, since a file's picture is exactly the one it
      was; and `git diff --stat crates/monospace-cli/assets/demo.json` must print **nothing**, which
      is B3.2 and SC-001's cost claim. Then `grep -c '^[A-Z].*:$'` on the new run, which must print
      **7**. Watch the **same** first diff come back with differences when the freeze is removed,
      which is the measurement research.md Q1 could only take against a temporary spike (B3.1, B3.2;
      quickstart.md "Commit 2")

**Checkpoint**: a person who runs the application sees seven pictures, the seventh with the box gone
and the arrow exactly where it stood, and the file those pictures came from is untouched.

**Commit**: `feat(cli):` T013-T018, ticking their checkboxes with it (plan.md commit 2). T018 is
three diffs and not a commit. It cannot precede commit 1: the seventh picture is the **evidence**
for the rule rather than an independent change, and a seventh that draws no arrow is the defect
rather than the feature.

---

## Phase 6: Records this slice writes and amends

**Purpose**: two `docs` commits, none of them a behavior, and so none carrying a `[Story]` label.
They are placed here rather than in the Polish phase because plan.md orders them after the code —
the first of them is `docs(model)` and **follows** the rule, which is where 082's D3's one-sentence
change to §3 landed (`505fd0d`).

- [ ] T019 In `docs/diagram-model.md`, amend **four** sections and add nothing else. **§4
      _Positions_**, after the paragraph on displacement, gains the rule in a paragraph of its own:
      a removal replaces every reference **naming the removed shape** with the absolute point it was
      resolving to at that moment, so what hung from it stays drawn where it stood; a reference
      naming **another** shape is not touched, and neither is one naming an identity that was never
      there — a case the removal did not create. Say that the two passes are not visible from here
      and do not belong here. **§6 _Attachment_** gains the consequence: a frozen endpoint is
      attached to **nothing**, which is a point like any other and has no side to name — and **do
      not** add a second meaning for an attachment, a new anchor, or a way to name the figure a
      point came from. **§9 _Changing a diagram_** **loses** the sentence "Nothing is rewritten and
      nothing cascades" and its two clauses after it, which is the constitution's _Understanding
      changes_ turned into prose rather than a sentence added beside it; its table of five grows
      **no row**, and `remove`'s row says what it now does in the same words the table uses ("Takes
      a shape out of the diagram", with the freezing named beside the table rather than inside the
      cell). **§10 _Properties worth testing_** gains **one** bullet: taking a figure out leaves
      every figure that referenced it drawn exactly where it was, and the rest of the diagram
      unchanged — which is a property now, not a rule with no test. **§11 is untouched**: its second
      question is **dissolved rather than answered**, so nothing goes on the sheet and nothing goes
      in §11 (B1, B2, B3.4; SC-003; plan.md "Artifacts"; spec.md "What this slice implements")

- [ ] T020 In `docs/decisions/`, write the record D2 answered — **one ADR for the freeze alone**.
      Take the next free number in the directory — **0068** is free today, and if it is not when
      this task runs, take the next one and write down what it was rather than inventing a number —
      and name the file `0068-freeze-what-hung-from-a-removed-shape-where-it-stood.md`, verb first.
      Fill **`docs/decisions/adr-template.md`** **whole**, at **`load-bearing`**, and the reason is
      the constitution's own test rather than taste: `monospace-cli` calls `Diagram::remove` and its
      seventh picture exists because of the freeze, so **something outside the module depends on
      it**. Its ceiling is therefore **150 lines**. **This corrects a judgment this plan's first run
      made** — it proposed `working` on D2's reasoning that reversing the freeze is a new short ADR
      rather than a migration, and the maintainer overruled it on 2026-10-02. Note what the
      correction costs and what it buys: the full template requires **Considered Options with the
      two options drawn side by side**, which the constitution's _Show the rendering_ holds a
      `load-bearing` record to and which did not fit under 60; it requires **Pros and Cons of each
      option**; and it requires a **Confidence with a percentage**. D2's condition — **as small as
      it can be** — is a ceiling the record has to meet rather than a tone to strike, and the
      template's own words are the measure (Q5, Q7; D2; ADR-0035, ADR-0064; principle VI; principle
      VIII) to strike. Four things it must carry: (1) the **rule and the one thing it replaces**,
      which are §9's "nothing is rewritten" and ADR-0041's "an editor can delete a shape without
      repairing everything that referenced it" — **nothing else**; (2) **no new vocabulary, no name
      for the frozen position, and nothing said about removals in general**, which is D2's condition
      and the reason §4 gains a sentence rather than a term; (3) the **pictures**, because the two
      options differ in what they draw — the freeze's picture and today's, the arrangement as
      written and with the box taken out, both reproduced from [data-model.md](data-model.md) and
      each labelled on the spot as **Hypothetical**, since no `<!-- render: -->` marker can reach a
      removal; (4) a **Confirmation** naming the six tests and the gallery block T005-T011 and T017
      add, since that is how anyone tells the decision is being followed. **Do not argue that the
      cost is small, and do not write that reversing it repeatedly is expensive** — the constitution
      puts that cost in a test rather than in prose where it reads as an instruction to stop
      thinking (Q5, Q7; D2; ADR-0035, ADR-0064; principle VI; principle VIII)

- [ ] T021 In `docs/decisions/0041-resolve-a-position-through-a-reference.md`, add **one dated
      line** under its `## Revisions` — **2026-10-02**, naming the new record and what it changes:
      the consequence at line ~93, "an editor can delete a shape without repairing everything that
      referenced it", is the one consequence this record's subject contradicts, because it now
      repairs what referred to it. **Do not re-argue the resolution rule, do not touch the other
      seven consequences, and do not grow the file** — D2's answer is that the alternative was a
      second record a reader would have to find beside the first. Add the **`commitment: working`**
      line to its front matter, which the file does not carry today, with a **dated `## Revisions`
      line saying why**: reversing the resolution rule is a new short ADR, not a migration.
      `cargo xtask numbering` is where a number not yet free shows up (D2; Q5)

- [ ] T022 [P] Add the row for T020's record to `docs/decisions/README.md` **as a whole row** —
      link, title and status written out together. A partial edit to that table leaves the rest of
      the row on the line below and prettier then reflows the damage rather than rejecting it, which
      is silent corruption of the one table every record is listed in (AGENTS.md "Facts that are in
      the code and in no document"). **T021 needs no row**: the table carries no commitment column,
      so ADR-0041's status is still `accepted` and its title has not changed

**Commit**: `docs(model):` T019, then `docs(adr):` T020, T021 and T022 (plan.md commits 3 and 4).

---

## Phase 7: Polish & Cross-Cutting Concerns

- [ ] T023 Run `cargo xtask check` and confirm every one of the **twelve** steps is green,
      **including `wasm`** — which compiles `monospace-core`, `monospace-diagram` and
      `monospace-glyph-sets`, so it covers the new body with no change to `xtask` and no new check,
      which is why principle III's two-commit rule does not apply. The body reaches the core only
      through the `Pos` that `resolve` already returned, and it is one `find`, one `Vec` and a loop.
      Watch the **output** rather than the exit code: `rustfmt` reports that `group_imports` needs
      nightly and exits 0, and so does a step that finds something it cannot fix. The `render` step
      must still answer **26** pictures — measured on this branch — and `numbering` must be green,
      which is where `0068` shows as taken rather than free (SC-004; plan.md Constitution Check,
      principles III and VII)
- [ ] T024 Confirm the two claims this slice **accepts with nothing to verify it** are named as such
      rather than described as tested, as principle IV asks and plan.md's re-check does on the spot:
      that **a removal leaves every other figure byte for byte**, which is a claim about the whole
      diagram and which T005 checks only for the one shape it names; and that **two shapes under one
      identity are a pre-existing edge** — `add_under` allows it, `find` answers the first match,
      and the freeze inherits that rather than deciding it. Look for the first in §4 and §9 of
      `docs/diagram-model.md`, §10's new bullet and T005's doc comment, and the second in
      `data-model.md`'s "What the freeze is not" and T020's **Reversibility**, and leave each where
      it is — **this task is a check that the record says so, not an edit** (constitution principle
      IV; plan.md "Re-checked after Phase 1")
- [ ] T025 Take the counts this slice claims and check them rather than asserting them:
      `cargo test --workspace` green at **29 / 19 / 114 / 77 / 15 / 61**, which is today's **28 / 19
      / 114 / 72 / 15 / 61** plus six tests and no test removed but T005's. Confirm no `.snap.new`
      file is left behind under `crates/monospace-diagram/src/snapshots/gallery/`, that
      `a_path_prints_one_picture_and_nothing_else` passes **unchanged** — it is the guard that a
      path still prints one picture and nothing else, which is what `cargo xtask render` embeds —
      and that `git status` shows no spike left from research.md's measurements and none from
      T018's. **No characterization report is owed**: research.md Q6 measured 1916 renderings across
      16 files and none of them can express a removal, because `remove` appears nowhere under
      `crates/monospace-core` and no sweep removes anything (SC-001, SC-004; research.md Q6)
- [ ] T026 Append an entry to `docs/learning-log.md` for this increment: what was learned about Rust
      design and idiom, what was learned about working this way, and optionally a trade-off worth
      remembering. Four are already paid for and worth writing down rather than rediscovering — a
      **method that reads `&self` while its caller holds `&mut self` is what forces two passes**,
      and the honest fix is the passes rather than a `Cell`; **a `Vec<(index, point)>` is not enough
      when one figure holds two of the things being rewritten**, which is the slot and which no test
      found until one was written on purpose; **one existing test went red and the specification
      named none**, so the measurement that catches this is running the workspace against a written
      body rather than grepping for the issue's number; and **a `#[cfg(test)]` scratch module inside
      the file it measures is the cheapest spike there is** — the tree carried no trace of every
      measurement in this plan (constitution principle II; research.md Q6, Q7)

**Commit**: `docs:` T026. T023-T025 are a gate run and two observations, not commits.

---

## What this slice must not add

Each of these is a decision already answered on the sheet or a rule the model states. Reaching past
one means stopping and asking, not deciding and recording afterwards (constitution, No decision
outside the sheet).

- **No flag, no separate change, and no sixth row in §9's table of five.** It must not be possible
  to remove a shape without updating the references to it, so there is no opt-out to add and no row
  (D4; SC-003)
- **No new vocabulary and no name for the frozen position.** A frozen end is `Position::Absolute`,
  which already exists, and §4 gains a sentence rather than a term (D2; data-model.md "What does not
  change")
- **No chain walk, no second resolution and no report.** The first pass visits each entry once and
  `resolve` is a lookup and an addition; a reference can only name a figure that answers an anchor,
  and a connector answers none, so one link is the most there is (ADR-0041; P2)
- **No repair of a reference the removal did not create.** A reference naming an identity that was
  never there is left exactly as it is, and is measured to still be a `Reference` afterwards (D3;
  T011)
- **No record of a removal.** Nothing remembers that one happened, no flag is stored on a position,
  and the two pictures simply differ — which is what dissolves §11's second question rather than
  answering it (B2.1, SC-002; SC-002)
- **No `<!-- render: -->` marker in any artifact.** A marker reads a description and a description
  carries no field that takes a shape out, so **no marker anywhere in the repository can hold a
  picture of this rule**; the gallery block is the carrier (ADR-0064, ADR-0035; B1.1)
- **No fourth contract, and no superseding ADR-0041.** The three contracts 081, 082 and 083 recorded
  are unchanged in signature and field, and writing the rule again is the duplication principle VIII
  refuses; D2's answer is a new ADR beside 0041 and one dated revision line in it (plan.md
  "Artifacts"; D2)
- **No amendment to §11 and no bullet added to it**, and no change to §1 through §3, §5, §7 or §8.
  §11's second question is withdrawn rather than answered (B2.1; SC-002)
- **No re-attachment on a put-back.** Nothing goes looking for the reference a frozen end was, and a
  figure put back under the removed identity leaves the arrow where it was rather than hanging from
  it (B1.2; P3)
- **No field in `assets/demo.json` and nothing in the description format.** Twenty-six markers read
  it and a seventh picture costs one call (B3.2; ADR-0035; SC-001)
- **No new dependency, no new crate, no new gate step, no change to `xtask`, and nothing in
  `monospace-core`** — `remove` appears nowhere under `crates/monospace-core` and the body reaches
  the core only through a `Pos` (principles III, VII; research.md Q6)
- **No characterization test and no ADR-0053 report.** 1916 renderings across 16 files, none of
  which can express a removal (research.md Q6)
- **No `refactor` commit.** Nothing structural happens: no type, no field, no variant, no signature
  and no new method, so principle V has nothing to separate (plan.md Constitution Check, principle
  V)
- **Nothing beyond the rule and the one thing it replaces.** The record is `load-bearing` and fills
  the whole template, and D2's condition — as small as it can be — is what keeps its _content_ to
  the rule and the two sentences it overrides, with no third section on removals in general (D2;
  principle VIII)

---

## Dependencies & Execution Order

### Phase Dependencies

- **Setup (Phase 1)**: nothing to do; T001 captures the baseline everything else is compared to.
- **Foundational (Phase 2)**: depends on nothing, and **blocks both user story phases** — a test
  that names a method which is not there does not fail, it does not compile, and T005's rewrite
  cannot exist before the body it is a rewrite of.
- **User Story 1 (Phase 3)**: depends on Phase 2. No dependency on US2 or US3.
- **User Story 2 (Phase 4)**: depends on Phase 2, and shares commit 1 with US1. Its T011 asserts
  what US1's rule did **not** break, so it is worth reading beside T006 rather than instead of it.
- **User Story 3 (Phase 5)**: depends on Phases 2 to 4 and **cannot precede them**. plan.md is
  explicit that commit 2 cannot come first, because the seventh picture is the evidence for the rule
  rather than an independent change, and a seventh picture with no arrow in it is the defect rather
  than the feature.
- **Records (Phase 6)**: T019 depends on Phase 2 — it follows the code rather than preceding it.
  T020 depends on T019's subject and on nothing else. T021 and T022 depend on T020.
- **Polish (Phase 7)**: depends on all of them.

### Within Each User Story

- US1: T005 reads T002 and is the same file; T006 and T007 read T002's body and T005's helpers; T008
  reads the same rule through `Diagram::remove` and moves the one snapshot in the slice.
- US2: T009-T011 all read T002 and may be written in any order among themselves; **T012 comes
  last**, because a deliberate red is one agent's work and it needs the tests above in place.
- US3: T013 before T014 (the tuple cannot hold seven pictures the demonstration does not print),
  T014 before T015-T017 (each of those takes the tuple apart), T017 after T013 (it compares the
  sixth with the seventh), and T018 after all of them.

### Parallel Opportunities

Parallelism here is lower than in 143 and the reason is worth stating rather than padding: **sixteen
of the twenty-six tasks edit one of two files** — `diagram.rs` takes ten and `main.rs` takes six —
and a seventeenth is the one snapshot move. Only **one** task carries `[P]`, where 143's list had
five, and the slice has two more places where it looks available and is not. Both facts are worth
recording, because a list padded to look parallel is a list that sends two agents into one file.

- **T022** with **T020** and **T021** — `docs/decisions/README.md` against the two record files. It
  is the only task whose file nothing else in the slice touches.
- **T004** looks parallel and is not: it is the fourth edit to `diagram.rs`, beside T002, T003 and
  T005, and a `[P]` task is one that can land and be checked on its own. It does not — its paragraph
  is false twice over and both halves are corrected together.
- **T015** looks parallel with T013 and T014 and is not, for the same reason and a sharper one: it
  takes the tuple apart, so it cannot compile until T014's seven-tuple has landed.

Two further tasks are deliberately **not** marked `[P]`: **T008** is the only task that moves a
snapshot and `cargo insta review` is a single reviewer, and **T012** is a deliberate red that one
agent should run and restore.

---

## Implementation Strategy

### The demonstrable increment is Phases 2 to 5, and there is no smaller one

There is no MVP phase to stop at, and saying so is more useful than naming one. Phase 2 alone is a
`feat` whose only evidence is one rewritten test that a reader has no reason to look at; Phase 5
alone is a seventh picture that is the sixth **without** an arrow in it, which is the defect rather
than the feature. The rule and the picture that proves it were shipped together in plan.md's commit
order for exactly that reason, and the smallest increment this slice has is **commits 1 and 2
together**.

### MVP First (Phases 2 to 5, two commits)

1. Phase 2 → taking a shape out freezes what hung from it, and the three sentences in the crate that
   said it did not now say what it does (commit 1's production half).
2. Phases 3 and 4 → six contract tests and the gallery's fourth block: the rule by value and drawn,
   the put-back, the three routes, two connectors on one figure, nothing else touched, and the one
   existing test rewritten (commit 1).
3. Phase 5 → the seventh picture, its caption, the seven-tuple and its callers, and the test that
   pins the seventh against the sixth cell by cell (commit 2).
4. **STOP and VALIDATE**: `cargo test --workspace` green at 29 / 19 / 114 / 77 / 15 / 61;
   `cargo run -q -p monospace-cli` prints seven captioned pictures and the seventh is the sixth with
   the box gone and the arrow exactly where it stood;
   `git diff --stat crates/monospace-cli/assets/demo.json` is empty; a path prints one picture and
   nothing else.

### Incremental Delivery

1. Phase 2 → the rule lands, alone in a commit it cannot be split from (commit 1, first half).
2. Phases 3 and 4 → the six tests and the gallery block, and the rule is now something a broken
   implementation fails (commit 1, second half).
3. Phase 5 → the shipped run carries the evidence, and the demonstration stops losing its arrow
   (commit 2).
4. Phase 6 → four sections of the model say what a frozen position is and §9 loses the sentence that
   said nothing is rewritten, and the rule has one home that is not a fourth contract (commits 3 and
   4).
5. Phase 7 → the gate is green, the two untested claims are named as untested, and the increment is
   closed (commit 5).

Each commit follows plan.md: one `feat(diagram)` carrying both halves, one `feat(cli)`, three
`docs`, and every commit leaves `cargo xtask check` green (constitution principles II, III and V).

### Parallel Team Strategy

With two agents:

1. Agent A takes Phase 2 and then Phases 3 and 4; Agent B takes the records in Phase 6 once T019's
   subject exists — but **not before**, because a record that follows nothing is a record of an
   intention.
2. Inside Phase 5, the seventh step and its test are one file and do not divide.

The honest answer is that this slice is **not** a parallel slice. Sixteen of twenty-six tasks touch
one of two files, and the two commits that matter are ordered by what they are evidence for.
