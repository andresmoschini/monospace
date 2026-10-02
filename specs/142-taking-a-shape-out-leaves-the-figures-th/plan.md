<!-- The branch below is the issue title truncated at forty characters by `cargo xtask spec`, which
     landed inside a word. -->
<!-- cspell:ignore referen -->

# Implementation Plan: Taking a shape out leaves what hangs from it not drawn

**Branch**: `142-taking-a-shape-out-leaves-the-figures-th-deciding` | **Date**: 2026-10-01 |
**Spec**: [spec.md](spec.md)

<!--
  Ceiling: 80 lines (constitution, principle VIII).

  Part two — Phase 1, run after the maintainer has answered decisions.md. Fills Design and produces
  the design artifacts. This comment goes with it.
-->

## Summary

The rule the model already states, made visible: the demonstration grows a seventh captioned picture
in which the arrow is gone because the box it hung from is gone, the rule is pinned by tests that
fail on a removal answering differently from a missing identity, the two open questions are written
where a reader meets them rather than only where they go looking, and one paragraph of rustdoc that
denies the behavior is corrected. No rule changes and nothing in the public API moves.

## What is unusual about this feature

- **No rule is taken, so most of the work is prose and one test.** The code that moves is a seventh
  step in `monospace-cli`'s demonstration, four contract tests in `monospace-diagram`, and a test
  helper's tuple growing by one.
- **The evidence is a picture the demonstration already draws** — measured at 22 cells, the box's
  twelve and the arrow's ten, by building the seventh picture and differing it against the sixth
  ([research.md](research.md) Q4).
- **`add_under` left three claims behind that it falsified**: two paragraphs in `diagram.rs` and one
  consequence line in ADR-0041. The slice names one; D3 asks about the other (Q5).
- **One number in the specification is wrong**, measured: B3.4 counts the arrow's cells at ten where
  this arrangement draws six (Q3, D4).
- No dependency is added, `monospace-core` does not move, `xtask` does not change, no gate step is
  added, and nothing reaches for a terminal.

## Constitution Check

_GATE: passes before Phase 0, re-checked after Phase 1._

- **I. Process over product** — the slice is thin because the question is what it is, and the work
  of narrowing it was done in the specification's clarifications rather than skipped here.
- **II. Demonstrable increments** — the seventh picture is the sixth with 22 cells gone and the
  other six byte for byte what they are, so the increment is something a reader sees rather than a
  rule stated only in a test.
- **III. One definition of green** — no new check, so the two-commit rule for adding one does not
  apply. The gate answered twelve steps and 26 generated pictures before and after this run.
- **IV. Claims are measured** — every number and every picture in the specification was reproduced
  on this branch ([research.md](research.md) Q1–Q6), and one count was found wrong rather than
  carried over (D4).
- **V. Structural and behavioral never share a commit** — the documents and the rustdoc corrections
  are one `docs` commit, and the seventh step and the contract tests are a `feat` beside it.
  `demonstrated_pictures` becoming a seven-tuple is a structural change to a test helper, and it
  travels inside the `feat` rather than as a `refactor` of its own: a helper nothing calls would
  fail the gate's clippy step, and Q4 rejects the `Vec` alternative for the same reason.
- **VI. Decisions at the altitude they belong to** — the rule is observable from outside the crate,
  so all four entries are domain-level and the maintainer's. Nothing else here is a decision: the
  six questions answered in research.md are each undone by changing the code that gives them.
- **VII. The core stays portable** — nothing in `monospace-core` moves and `monospace-diagram`'s
  public API is untouched, so the `wasm` step compiles what it compiles today.
- **VIII. The record is sized to the decision** — four entries against a cap of seven. `research.md`
  is at its 100-line ceiling. **`spec.md` is 259 attributable lines against a 120 ceiling**,
  measured here as 319 lines less 35 blank, 6 fence markers and 2 HTML comments, of which 17 are the
  one generated picture and its description that principle VIII exempts — the 238 it arrived at,
  widened by the three answers that each named a consequence outside the sheet. **`decisions.md` is
  98 attributable lines against a 60 one**, 111 less 13 blank, which the sheet crossed on being
  answered rather than on being written. Three overages go to Complexity Tracking below.
- _The plan runs in two parts_ — this run filled Summary, this check, `research.md` and
  `decisions.md`, and stopped. `data-model.md`, `contracts/` and `quickstart.md` are part two, and
  writing them now would be taking four answers rather than planning around them.

## Decisions

The sheet is [`decisions.md`](decisions.md). Part two does not begin until it is answered.

- Entries: 4 — domain: 4, module: 0, tooling: 0
- Answered: 2026-10-02 — all four, each the proposal adopted, and three of the four naming a
  consequence outside the sheet that this branch therefore carries too.
- The two questions of §11 are on no sheet, and this slice answers neither. When one is answered
  they go on one sheet or on two **in that order** — whether anything should happen at all first,
  and how anything would tell a removal from an identity nothing holds second — because the second
  is a consequence of the first, and a sheet that answered them the other way round would be
  answering a question whose answer had not been chosen. [research.md](research.md) Q3 is the
  evidence that it is one question and not a preference: routes A and B reach the same picture byte
  for byte, and in route C the identity is still in the diagram and `get` finds it.

## Design _(part two)_

The map, run against the four answers. The values and the measurements are in
[data-model.md](data-model.md) and the commands that take them are in
[quickstart.md](quickstart.md). **Nothing here adds a rule, and that is what the sheet decided** —
the work is one step in the demonstration, one block in the gallery, four tests, two paragraphs and
four documents.

| Change                                            | Where                              | Because                                                                                                    |
| ------------------------------------------------- | ---------------------------------- | ---------------------------------------------------------------------------------------------------------- |
| `remove(&the_hung_from)` and one caption          | `crates/monospace-cli/src/main.rs` | B2.1: the seventh picture, one call, **no `if let`** — measurement 2                                       |
| `demonstrated_pictures` and its eleven call sites | `src/main.rs`                      | Measurement 1: no test forces it; the specification does, and the helper is the only way to name a picture |
| the gallery's fourth `block()` and its snapshot   | `monospace-diagram/src/gallery.rs` | D2: beside the three it holds, and measurement 3 on what it can draw                                       |
| four contract tests                               | `src/diagram.rs`                   | B1.1, B1.2, B1.3 and B3's three routes — the first tests here that can fail on a decision nobody has taken |
| two doc comments `add_under` falsified            | `src/diagram.rs:2352`, `:2546`     | D3: `add_under` is `pub` and B1.2 measures what that permits                                               |
| B3.4, from ten to six                             | `spec.md`                          | D4, and the measurement that found it                                                                      |
| §9's removal paragraph, one sentence              | `docs/diagram-model.md`            | P3 and SC-002: where a reader takes the rule for settled                                                   |
| §11's **two** bullets, the second as three routes | `docs/diagram-model.md`            | SC-002: what a reader goes looking for                                                                     |
| ADR-0041 revised in place — no second record      | `docs/decisions/0041-*.md`         | D1, and the constitution's test: a reader cannot cite one without the other                                |

**What does not move**, and the list is the shape of the slice: `Diagram`'s whole surface, every
signature and every `#[must_use]`; `Position`, `Reference`, `Anchor`, `Delta`, `Shape`, `Endpoint`
and `ShapeId`, no field added and no derive changed; §4, which is **not** amended because its
two-row table already answers the question; `assets/demo.json` and the description format;
`monospace-core`, which has no `remove` at all; `xtask` and the gate. So principle VII reads the
same after the design as before it, and principle III's two-commit rule for a new check does not
apply — **no step is added**.

### Three measurements that came back from the design

Each was taken by adding the change, running it, and reverting it. **All three constrain the plan
rather than overturn it**, and none is a decision the sheet does not hold.

**One — the seven-tuple is forced by the specification, not by any test.** Added the seventh step
alone, without touching the helper, and ran the suite: **exactly one test turns red**,
`a_bare_run_prints_six_captioned_pictures_…`, which asserts a count. Every other test passes
unchanged — including `an_empty_description_demonstrates_as_six_identical_pictures` and
`one_shape_demonstrates_as_two_copies_of_itself_and_then_an_empty_window`, which Q4 predicted would
need care. So the helper widens because **the specification asks the twenty-two cells to be asserted
and the helper is the only way to name a picture without pinning a caption's wording.** That also
settles principle V, and it sharpens what part one recorded: a `refactor` of the helper cannot be
separated at all, because a tuple's arity is part of every destructuring site's type — widening the
returns does not compile until the eleven callers widen too. The `refactor` shape _is_ available
(widen it by handing back the sixth twice) and is **not taken**, because a commit that changes
nothing is the fix principle III refuses to manufacture. Part one gave the `Vec` alternative's
rejection; this is why the seven-tuple has no split of its own either.

**Two — the seventh step needs no guard, and three degenerate descriptions prove it.** The third,
fifth and sixth steps each wrap their change in `if let`, because `get` returning `None` would
panic. `remove` needs none: `Diagram::remove`'s own rustdoc already says an identity the diagram
does not hold "changes nothing, with no error, no report and no panic". Measured on the seventh
step: an empty description gives **seven identical blank** pictures; a one-shape description gives
pictures `2..6` all equal, so the sixth equals the seventh; and **the shipped `demo.json` with `#3`
renamed gives seven pictures with the sixth equal to the seventh** — the case Q4 does not cover and
the one that would catch a seventh doing something other than a removal. The sixth is worth having
for exactly the reason the seventh is, so `one_shape_demonstrates_…` **gains**
`assert_eq!(pictures.5, pictures.6)` beside the `assert_eq!(pictures.4, pictures.5)` it already
carries.

**Three — the gallery's fourth block measures to an empty picture, and the reason is the
arrangement.** D2's block draws **nothing**: `wrote 0 of 24 positions`. Measured for all three ways
of reaching it, and **all three give zero** — the arrangement as written, `from_as_written` after
its connector is displaced (**0 of 32**), and `diagram` after its box is displaced (**0 of 24**).
The gallery holds **one box and one connector**, and the connector's `from` names the only other
figure, so **there is no survivor**: the block is B1.1's picture without the `#2` that stands in it.

Two consequences, both in [data-model.md](data-model.md). The block is **reached from the
arrangement as written through a third `Diagram`** — the second one block three already needed —
because a removal on either diagram the test holds is a no-op on what remains, and the block would
then be blank for a second and different reason with the comment beside it saying the first. And the
block is **still worth having**: it is a snapshot, so a removal that ever began drawing the route,
or freezing it where it resolved, would write cells into that window and move it.

**The one thing that would change it is not taken.** Widening the arrangement to hold a second box
would let the block carry B1.1's picture rather than its degenerate case — and it moves all three
blocks the snapshot already holds, which is not D2's answer and not a decision this slice may take.
It is named here so the maintainer can raise it at review, rather than being done and discovered.

### Six commits, and the order is forced

1. `feat(diagram)`: the four contract tests, the two doc comments `add_under` falsified, and the
   gallery's fourth block and its snapshot. It **precedes** commit 2 because the gallery block is
   the one picture that fails loudly, and a snapshot accepted after the demonstration's seventh is a
   picture of a rule already in motion.
2. `feat(cli)`: the seventh picture, its caption, the seven-tuple and its callers, and the test that
   counts the twenty-two and checks every one blanks. It cannot precede commit 1 — the picture is
   the evidence for the rule rather than an independent change.
3. `docs(spec-142)`: **B3.4 from ten to six**, naming the measurement. It follows the code because
   the count is measured rather than corrected on sight, and the measurement is what
   `the_three_routes_…` now asserts — so the two land in the order the reason does. **One line, and
   its own commit** so a reader can see a corrected claim from a measurement rather than from an
   edit.
4. `docs(model)`: §9's one-sentence note and §11's two bullets, following the code, which is where
   082's and 083's model amendments landed (`505fd0d`).
5. `docs(adr)`: ADR-0041's correction and its `## Revisions` section, on the building branch as D1
   requires. **No row in `docs/decisions/README.md`**, because the title and the status do not
   change — which is also the safe outcome, that table being one prettier-aligned grid where a
   partial edit corrupts it silently.
6. `docs`: the increment's entry in `docs/learning-log.md`.

### Re-checked after Phase 1

**No gate above changes and the design added no step to it.** Principle II's increment is the
seventh picture, and it is the sixth with 22 cells blanked and the other six byte for byte what they
are. Principle III holds because no check is added. Principle V has **nothing to govern**:
measurement 1 shows the one structural change in the slice is inseparable from the behavioral one,
so there are no two halves. Principle VI is unchanged — all four entries were domain-level and are
the maintainer's, and the three new module-level questions this design settled (measurements 1 to 3)
are each undone by changing the code that gives them, so they belong in `research.md`'s manner and
not in a record.

**Two claims are accepted with nothing to verify them**, named here per principle IV rather than
described as tested. The first is that **a removal leaves every other figure byte for byte** — a
claim about the whole diagram rather than about the figure that went, and the contract test verifies
it for the three figures the arrangement names and no single fixture can establish it for every
diagram. The second is that **the gallery's blank block is worth keeping**: what it will catch is
measurable and what a reader makes of an empty window is not, so it is named in the test's own doc
comment where a reader meets it. Both are in [data-model.md](data-model.md).

**What is verified, and by what.** The rule as it stands, pinned four ways: a removal against a
missing identity, the three routes as one test comparing the arrow's **six** cells, `add_under`
restoring the picture byte for byte, and `add` handing back `#4`. The six is asserted against the
**no-connector baseline** rather than quoted, because a test that compares three pictures against
each other cannot find a count that is wrong in all three.

### Artifacts

`data-model.md` and `quickstart.md`, and **no contract**.

**The contract is not written because the slice changes no signature, no field and no public
behavior.** `Diagram`, `Shape`, `Position` and `add_under` are the surface
[081](../../081-a-shape-can-be-removed-and-replaced/contracts/diagram-api.md),
[082](../../082-a-connector-endpoint-hangs-from-a-box-s/contracts/diagram-api.md) and
[083](../../083-a-reference-carries-a-horizontal-and-a-v/contracts/diagram-api.md) each recorded,
and all three are untouched; the description format is untouched; `contracts/diagram-api.md` of this
feature would be a fourth copy of a surface that has not moved. What changes is the **evidence** for
one already recorded item, and D1's answer says where a change to a recorded rule lives: ADR-0041,
revised in place.

**So the record is revised rather than added**, which follows from the constitution's own test
rather than from taste — a reader who takes the resolution rule for settled must meet the standing
question where the rule is written, and the rule and the question cannot be cited apart. ADR-0041 is
`load-bearing`, and `docs/decisions/README.md` is not edited at all: its row's title and status are
unchanged.

**No `<!-- render: -->` marker in either artifact**, and that is Q4's finding rather than an
omission: a marker reads a description the file carries, and **a description cannot take a shape
out** (ADR-0035, ADR-0064). So `cargo xtask render` regenerates nothing this slice touches, and a
changed marker in the gate is a mistake rather than an update.

## Complexity Tracking

> Fill ONLY for a Constitution Check violation that must be justified, or a principle VIII ceiling
> exceeded without splitting the feature.

| Departure                                                     | Why needed                                                                                                                                                                                             | Simpler alternative rejected because                                                                                                                                                                                                                                                                                                                                               |
| ------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `spec.md`, **259 against 120**                                | Four measured pictures, three clarifications, and three answers that each named a consequence outside the sheet                                                                                        | Compressing prose buys nothing — the words return on the next `cargo xtask fix`. Splitting separates B1, B2 and B3, which are one measurement                                                                                                                                                                                                                                      |
| `decisions.md`, **98 against 60**                             | Four domain entries at the constitution's fields each, four answers that each name a consequence outside the sheet, and the one rendered picture D4's subject is                                       | The format costs about a dozen lines an entry whatever the prose, and the fields that would fit in a line are the ones that drop the trade-off. Cutting the answers costs the consequence, and the consequence is what part two acts on                                                                                                                                            |
| `plan.md`, **250 against 80**                                 | Nine principles each named where the feature touches them, then part two's map of nine changes, **three measurements that each constrained the plan**, six commits, the re-check and the artifact note | The ceiling is written for a plan written in one pass, and the constitution's own rule puts part two on its own branch after the sheet is answered, so the two halves cannot share one ceiling — 082, 083 and 148 filed the same overage for the same reason. Dropping the three measurements would leave three wrong claims standing where a reader would act on them             |
| `ADR-0041`, **178 against 150 before this slice adds a line** | D1's answer revises it in place, so the two dated corrections land on a record that was **already over** its `load-bearing` ceiling when this branch started                                           | A second record would be the failure D1 rejected — a reader of the removal rule would have to find both, which the constitution's "cannot be cited without citing the other" test prevents. **Filing it here rather than leaving it**: an overage that predates the slice is still an overage this increment merges, and the correction is the two lines that make the record true |
