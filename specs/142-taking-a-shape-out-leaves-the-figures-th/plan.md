<!-- The branch below is the issue title truncated at forty characters by `cargo xtask spec`, which
     landed inside a word. -->
<!-- cspell:ignore referen -->

# Implementation Plan: Taking a shape out leaves the figures that hung from it where they were

**Branch**: `142-taking-a-shape-out-leaves-the-figures-th-deciding` | **Date**: 2026-10-02 |
**Spec**: [spec.md](spec.md)

<!--
  Ceiling: 80 lines (constitution, principle VIII).

  This file is filled across two runs, and the split is a rule, not a convenience
  (constitution, _The plan runs in two parts_).

  Part one — Phase 0, on the deciding branch. Filled Summary, this check, and produced
  research.md and decisions.md. It stopped there and did NOT produce data-model.md or
  quickstart.md, because writing them is taking the decisions the sheet was asking about.

  Part two — Phase 1, on the building branch, after the maintainer answered decisions.md on
  2026-10-02. Fills Design and Complexity Tracking, and produces data-model.md, quickstart.md
  and tasks.md. This comment goes with it.

  The technical context this project would fill in is fixed and lives elsewhere: Rust edition
  2024 at the pinned toolchain, a virtual cargo workspace under crates/, no storage, a terminal
  consumer, and the WebAssembly boundary the gate enforces. Repeating it per feature is the
  duplication principle VIII refuses. State only what is unusual about THIS feature.
-->

## Summary

Taking a shape out freezes what hung from it. Every position carrying a reference to the removed
shape becomes the absolute point it was resolving to, so the connector stays drawn exactly where it
was — not re-routed, not moved. The work is two crate-private methods in `monospace-diagram` and a
five-line body that calls them, four amended sections in
[`docs/diagram-model.md`](../../docs/diagram-model.md), two records touched and one written, one
seventh picture in the shipped demonstration with the arrow still in it, and tests that pin both the
freeze and the fact that a removal has become visible in the picture.

## What is unusual about this feature

- **The rule contradicts a sentence the model already states.** §9 says "Nothing is rewritten and
  nothing cascades", and the freeze is a rewrite reaching figures the removal never named. §9 is
  amended by losing that sentence, not by adding one beside it.
- **The previous deciding stage declined this rule for want of a record** and wrote the question
  down instead. Every measurement that stage took is of the behavior being replaced, which is why
  [D1](decisions.md) asked whether the measuring work merges first or travels with the freeze. **Its
  answer closes that pull request without merging**, so this building branch starts from a tree
  where nothing measures the old rule any more.
- **The freeze dissolves a question rather than answering it.** How anything would tell a removal
  from a shape that was not there goes away, because a removal's figures stay and the two pictures
  therefore differ. Nothing has to remember that a removal happened, and §11 gains nothing — the
  deciding stage that merged would have put two bullets there and did not, having declined to record
  the question.
- **Two of the specification's counts were derived rather than measured, and one of the two
  derivations was wrong.** Part one carried `15 − 6 = 9` for the arrangement and `22 − 10 = 12` for
  the demonstration, so that a reader could check the numbers were not asserted
  ([research.md](research.md) Q1, Q3, Q4). Part two measured them: the demonstration's **twelve
  holds**, and the arrangement's is **ten, not nine**, because `{3, 1}` changes glyph rather than
  going blank (Q7). **An asserted count and an asserted glyph are what a hand-drawn picture gets
  wrong**, and the specification's pictures were hand-drawn on the spot from a measurement that did
  not exist yet.
- **The rewrite is a method on `Shape` and not on `Diagram`**, at the maintainer's direction on
  2026-10-02: `Diagram` already reaches into every figure it holds, and which positions a figure has
  belongs to the figure. **All three kinds are matched** and the two that cannot hold a reference
  answer `None` today, so the day #89 widens the restriction it is two lines rather than the method.
- **D4's one named mechanical consequence did not survive the compiler.** It says the rewrite is two
  passes; measured, one pass compiles and the form that fails is `&mut self.shapes` — measurement 5.
- No dependency is added, `monospace-core` does not move, `xtask` does not change, no gate step is
  added, and nothing reaches for a terminal.

## Constitution Check

_GATE: passes before Phase 0, re-checked after Phase 1._

- **I. Process over product** — the slice is thin because the rule is one sentence; the work of
  narrowing it was done in the specification's clarifications rather than skipped here.
- **II. Demonstrable increments** — the seventh picture is the sixth with the box's twelve cells
  gone and the arrow's ten unchanged, so the increment is something a reader sees rather than a rule
  stated only in a test.
- **III. One definition of green** — no new check, so the two-commit rule for adding one does not
  apply.
- **IV. Claims are measured** — every number the specification carries is either measured on the
  branch the previous stage ran against or derived from two such measurements by arithmetic shown in
  the specification. **Nothing here is asserted from a picture nobody ran.**
- **V. Structural and behavioral never share a commit** — the model's four sections and the two
  records are two `docs` commits, the freeze and its tests a `feat` beside them, and the seventh
  picture a second `feat`. `Position` gains no variant, `Shape` gains no arm and `Diagram` gains no
  method — but `Endpoint` and `Shape` gain **two crate-private methods that each need the other's
  caller**, and the _Re-check_ below measures that they cannot land apart. The structural half is
  available as a shape and is not green, which is a different thing from not existing.
- **VI. Decisions at the altitude they belong to** — the rule is observable from outside the crate,
  so all four sheet entries are domain-level and the maintainer's. Nothing else here is a decision:
  the seven questions answered in research.md are each undone by changing the code that gives them.
- **VII. The core stays portable** — nothing in `monospace-core` moves and the crate it would reach
  is never compiled for the rule, so the `wasm` step compiles what it compiles today.
- **VIII. The record is sized to the decision** — four entries against a cap of seven, and **all
  four adopted as proposed**. Re-measured on this branch in part two with 143's count — total lines,
  less blank, less fence markers, less the HTML comment lines at the head, less every generated
  picture and the description it comes from: `spec.md` **139 against 120**, `decisions.md` **76
  against 60**, `research.md` **99 against 100** and this file **290 against 80**. Three overages go
  below; the fourth is this one, and it is the same overage 082, 083, 148 and 143 filed.
- _The plan runs in two parts_ — **part two has now run**, on the building branch, after the four
  answers. It filled Design, re-checked this gate against the design, and produced
  [`data-model.md`](data-model.md), [`quickstart.md`](quickstart.md) and [`tasks.md`](tasks.md).
  Nothing from the building branch that described the opposite behavior is carried forward: the
  previous part two was written against the rule the freeze replaces, and the three artifacts are
  the ones this specification's answers produce.

## Decisions

The sheet is [`decisions.md`](decisions.md). **Answered**: 2026-10-02 — all four, each adopted as
proposed, and three of them wider than the proposal asked.

- Entries: 4 — domain: 4, module: 0, tooling: 0. **The four answers of the first run of this stage
  are void**; they answered whether to record the question, which this slice no longer asks.
- **[D1](decisions.md) settles the order as well as the slice** — this deciding pull request
  finishes the specification, the implementation is a **new** pull request, and the building pull
  request describing the opposite behavior is **closed without merging**.
- **[D2](decisions.md)** is one new ADR for the freeze, one dated `## Revisions` line in ADR-0041
  for the one consequence it contradicts, and a `commitment: working` added to ADR-0041's front
  matter. ADR-0041 stays `working` on the maintainer's own reasoning — reversing the resolution rule
  is **a new short ADR naming what it replaces**, not a migration. **The new record is
  `load-bearing`**, corrected from the `working` this plan first proposed: the constitution's own
  test asks whether **something outside the module depends on it**, and `monospace-cli` calls
  `Diagram::remove` and its seventh picture exists because of the freeze. That answer is yes, so the
  record takes the full template at 150 lines — including the **two options drawn side by side**,
  which the constitution's _Show the rendering_ holds a `load-bearing` record to and which did not
  fit under 60.
- **[D3](decisions.md)** is stated in the maintainer's own terms: a removal freezes **the references
  naming the removed shape and nothing else**, and a reference naming an identity that was never
  there is left as it is.
- **[D4](decisions.md)** was strengthened past the proposal: it must not be **possible** to remove a
  shape without updating the references to it, so there is no flag, no separate change and no
  opt-out — and therefore no sixth row in §9's table. **Its one named mechanical consequence is
  wrong**, and measurement 5 below is what shows it: the rewrite is **not** forced to be two passes,
  and the rewrite lives on `Shape` rather than in `Diagram` at the maintainer's direction. What
  survives in D4 is the sentence underneath its conclusion: _a reference can only be resolved while
  the shape it names is still held._

## Design _(part two)_

The map, run against the four answers. The values and the three pictures are in
[data-model.md](data-model.md), the commands are in [quickstart.md](quickstart.md), and the order is
in [tasks.md](tasks.md).

**The rule is a method on `Shape`, not on `Diagram`**, at the maintainer's direction on 2026-10-02:
`Diagram` already reaches into every figure it holds, and which positions a figure has and what a
frozen one is belongs to the figure. `remove` becomes five lines that call it.

| Change                                                                                    | Where                              | Because                                                                                                                                                                 |
| ----------------------------------------------------------------------------------------- | ---------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Endpoint::frozen_position`, and `Shape::with_frozen_references` over **all three kinds** | `monospace-diagram/src/shape.rs`   | The rewrite is the figure's, and the two kinds that cannot hold a reference answer `None` **today** — the arms are there so #89 widens two lines rather than the method |
| `remove`'s body: five lines, and a loop that **indexes** rather than iterates             | `src/diagram.rs`                   | D4 for the freeze being inside `remove`; measurement 5 below for why it indexes                                                                                         |
| `remove`'s own rustdoc, two sentences                                                     | `src/diagram.rs`                   | D4 names "changing nothing else" as the sentence this slice corrects, and the paragraph after it repeats it                                                             |
| one doc comment false twice over                                                          | `src/diagram.rs`                   | Q5 measured the `add_under` half; the freeze makes the other. The test's assertions do not move                                                                         |
| one existing test, rewritten rather than deleted                                          | `src/diagram.rs`                   | Measurement 1 below: it is the only one that goes red, and the specification named none                                                                                 |
| the gallery's fourth `block()` and its snapshot                                           | `src/gallery.rs`                   | B3.4, and measurement 3 below on reaching it and on its window                                                                                                          |
| the seventh picture and its caption                                                       | `crates/monospace-cli/src/main.rs` | B3.1: one `remove(&the_hung_from)` and one `push_str`, beside the six already there. **No `if let`** — D4 again                                                         |
| `demonstrated_pictures` and its ten call sites                                            | `src/main.rs`                      | A six-tuple becomes seven, and every caller that names one picture now names two                                                                                        |
| §4 and §6 gain a sentence; §10 gains a property; §9 loses one                             | `docs/diagram-model.md`            | The specification's four sections, all **amended**; §11 is untouched and §9's table grows no row                                                                        |
| one `load-bearing` ADR, and one dated line in ADR-0041                                    | `docs/decisions/`                  | D2. `monospace-cli` calls `remove`, so the constitution's own test says `load-bearing` and the full 150-line template                                                   |

`Position`, `Reference`, `Endpoint` and `Shape` keep their fields and variants, and `resolve`,
`displaced_by`, `Delta` and the whole of `Diagram`'s surface — including `remove`'s own signature —
keep what they have. **Two crate-private methods are added and neither is public**, and there is
nothing else new in the slice: no type, no field, no variant, no signature. No dependency is added,
`monospace-core` does not move, `xtask` does not change and no gate step is added, so principle
III's two-commit rule for a new check does not apply.

### Five measurements that came back from the design

Each of these was taken by writing the code, running it, and reverting the tree — a `#[cfg(test)]`
module inside the file being measured, which is what [research.md](research.md) Q7 did and what
leaves no trace of the spike in `git status`. All five corrected something the plan would otherwise
have asserted, and two of them corrected a **decision** rather than a number.

**One existing test goes red, and the specification named none.** Measured: with the two methods and
`remove`'s body in place and nothing else touched, `cargo test --workspace` is green at **28 / 19 /
114 / 72 / 15 / 61** except for **one** failure —
`taking_the_referenced_figure_out_stops_the_connector_and_changes_nothing_else`, whose name and both
halves of whose claim are false under the freeze. research.md Q6 built its inventory by grepping for
the _behavior_ rather than for this issue's number, and it found the prose but not the test; 143
paid for the same gap a fifth time in a different file. It is rewritten as **T005** rather than
deleted, and it is the reason `tasks.md`'s baseline says **one rewritten**. Nothing else in either
crate moved, **and the same one test moved under the earlier design** — moving the rewrite onto
`Shape` changed which file the work is in and not one test's outcome. The demonstration's fourth
step removes `#1`, which holds no reference, so the six pictures the shipped run prints are byte for
byte what they were.

**One connector with both ends on the same figure freezes to two different points** — and that is
the arrangement that decides where the code lives. Measured on a four-by-three box at the origin,
`from` on its right side at offset `(0, 0)` and `to` on its bottom at offset `(1, 0)`:

```text
from  →  Absolute({ 3, 1 })
to    →  Absolute({ 2, 2 })
```

The first design this plan proposed collected `(index, point)` and wrote "the positions of that
figure still holding a reference to `id`", which puts the first point into both ends and draws a
route nobody asked for. **No arrangement this slice already builds would have caught it.** The shape
the maintainer asked for has no such failure at all, because it computes both answers before it
builds either endpoint — which is why `data-model.md` makes that the argument for the method's shape
shape rather than a note about a `slot`, and why [tasks.md](tasks.md) writes a test for the
arrangement twice on purpose: T007 asks the method and T013 asks the diagram.

**The gallery's fourth block needs the window its two siblings have.** The opposite of 143's third
block, and the reason is the same in both cases with the sign flipped: a displacement lands the
connector on a row the window does not have, and **a removal lands nothing**. Measured: run at
`window(8, 3)` the block draws the arrow standing where it stood, and run at `window(8, 4)` the
fourth row comes back empty. So the block keeps `window(8, 3)` and a snapshot that grew a fourth row
of nothing is the signature here of a window widened for no reason. It is also reached from a
**third** `Diagram` in that test rather than from the block beside it, because the block above
leaves `#1` displaced four cells right and a removal of a figure the reference is already resolving
somewhere else freezes the arrow **displaced** — a fourth picture of a different claim from B1.1's.

**The seventh picture reaches one of the arrow's two references, not both.** At the sixth picture
`#10` holds a `from` rehung to `#3` and the shipped `to` naming `#5`; `remove(&#3)` freezes the
first and leaves the second a `Reference`. Measured: a bare run prints **seven** captioned pictures,
the sixth and the seventh differ in **exactly twelve cells**, all twelve of them the whole
`x 13..16, y 2..4` rectangle and **all twelve blank**, and the first six are byte for byte what the
run prints today. That is B3.1's claim confirmed rather than restated, and it is also the sharpest
statement of D3's answer available anywhere in the slice: **the demonstration's own arrow has one
reference frozen and one still live, in the same picture.**

**D4's stated reason for two passes is false, and one pass compiles.** D4 named its one mechanical
consequence as _`Position::resolve` takes `&Diagram` where `remove` holds `&mut self`. The rewrite
is therefore two passes._ Measured, all three of these:

| Form                                                            | Result                                                          |
| --------------------------------------------------------------- | --------------------------------------------------------------- |
| `for placed in &mut self.shapes`, writing `placed.shape` inside | **`error[E0502]`** — the mutable borrow is live across the call |
| `for at in 0..self.shapes.len()`, one statement per figure      | **compiles**, and is what `tasks.md` T002 writes                |
| collect every resolution, then write them all                   | compiles, and is what a two-pass design looks like              |

So the constraint D4 named is real and is what makes the second row necessary — **a `&mut Shape`
cannot be held across a call that wants the diagram** — but it does not reach the conclusion. One
pass is available, with no `Vec`, no allocation and nothing collected. What survives in D4 is the
sentence underneath its conclusion, _a reference can only be resolved while the shape it names is
still held_, which is why the loop runs before `self.shapes.remove(index)`. **This is a correction
to an answered sheet entry and the sheet says D4's answer stands**, so it is recorded here rather
than changed; if the maintainer wants `decisions.md` to carry the corrected reason, that is a fifth
line in D4 and nothing in the code moves.

### Five commits, and the order is forced

1. `feat(diagram)`: the two crate-private methods on `Endpoint` and `Shape`, `remove`'s five-line
   body, the **two** sentences of its own rustdoc and the one doc comment that is false twice over,
   the rewritten test, six new contract tests, and the gallery's fourth block and its snapshot.
   `shape.rs` and `diagram.rs` are one commit's worth of changes between them — the methods are dead
   code without the caller, which the _Re-check_ below measures — so the production half and the
   tests beside it share a commit.
2. `feat(cli)`: the seventh picture, its caption, the seven-tuple and its callers, the two renamed
   tests, and the test that pins the seventh against the sixth cell by cell. It cannot precede
   commit 1, because the picture is the evidence for the rule rather than an independent change.
3. `docs(model)`: §4's rule, §6's frozen endpoint, §9's lost sentence and §10's new property,
   following the code — which is where 082's D3's one-sentence change to §3 landed (`505fd0d`).
4. `docs(adr)`: the record, the dated line and the commitment in ADR-0041, and the row in
   `docs/decisions/README.md`, as a whole row and never a fragment of one.
5. `docs`: the increment's entry in `docs/learning-log.md`.

### Re-checked after Phase 1

No gate above changes and the design added no step to it. The whole of the rule is two crate-private
methods and a five-line body inside a method that already existed, in a crate the `wasm` step
already compiles, reaching nothing from `monospace-core` beyond the `Pos` that `Position::resolve`
already returned. Principle VII reads the same after the design as before it.

**Principle V reads differently from what part one recorded, and measurement 6 is why there is still
no `refactor` commit.** There _are_ two new methods now, so a structural change exists where the
earlier design had none. Measured on this branch: with both methods in place and `remove` untouched,
`cargo clippy --workspace --all-targets -- -D warnings` fails with
`error: method 'frozen_position' is never used` and
`error: method 'with_frozen_references' is never used` — `dead_code` is a warning and the flag turns
it into an error. **A crate-private method nothing calls is red under this gate**, so one `feat`
carries the methods and their caller together, exactly as 143 found for `Delta::grow`. The
structural half is available as a shape and is not green, which is a different thing from not
existing.

**Two claims are accepted with nothing to verify them**, named here per principle IV rather than
described as tested. The first is that a removal leaves **every other figure** byte for byte, which
is a claim about the whole diagram rather than about the figure that moved: the rewritten test
compares the whole picture against a diagram of the shape it names and no single fixture can
establish it for every diagram. The second is **two shapes under one identity** — `add_under` allows
it, `find` answers the first match, and the freeze inherits that rather than deciding it, so a
removal of one of them freezes at the **first** one's side and leaves the other standing under the
same name. Both are named where a reader meets them rather than only beside this paragraph: the
first in the rewritten test's doc comment and the ADR's **Confirmation**, the second in
`data-model.md`'s "What the freeze is not" and the ADR's **Reversibility**.

**What is verified, and by what.** Every existing test in both crates except the one that had to be
rewritten, the six new contract tests, the gallery's fourth block and the demonstration's seventh
picture — which is the whole of the suite, since no characterization file can express a removal. The
seventh picture's **twelve cells** are asserted against the sixth rather than quoted, because the
arrow's footprint is neither contiguous nor a rectangle and nothing that reads it off a picture gets
it right — which is how the specification's own `15 − 6` was wrong and how Q7 corrected it.

### Artifacts

`data-model.md`, `quickstart.md` and `tasks.md`, and **no contract**.

The contract is not written because the slice changes no signature, no field and no type.
`Diagram`'s surface is what
[081](../../081-a-shape-can-be-removed-and-replaced/contracts/diagram-api.md) recorded, `remove`
among them, and what changes here is the **behavior** of one already recorded item rather than its
shape. That is what the ADR is for: the rule is observable from outside the crate, which makes it
domain-level, and the constitution puts a domain-level decision in `docs/decisions/` with the ADR
beside it. Writing the rule a second time in a fourth contract is the duplication principle VIII
refuses.

So the record is **`load-bearing`**, on the constitution's own test rather than on taste:
`monospace-cli` calls `Diagram::remove`, so something outside the module depends on it, and
reversing the freeze changes what that caller draws. Its ceiling is therefore **150 lines** and it
takes the whole template — which is what D2's condition of _as small as it can be_ is measured
against rather than a free hand: **the rule and the one thing it replaces**, §9's "nothing is
rewritten" and ADR-0041's "an editor can delete a shape without repairing everything that referenced
it", and nothing else. **No new vocabulary and no name for the frozen position**, so §4 gains a
sentence rather than a term, and a frozen end is the `Position::Absolute` that already exists.
ADR-0041 keeps `working` and gains the commitment its front matter did not carry, because the
maintainer's reasoning for it — reversing the resolution rule is a new short ADR, not a migration —
is about _that_ rule and not about this one.

**No `<!-- render: -->` marker in any of the three artifacts**, and that is a fact rather than an
omission: a marker reads a description the file carries, and a description has no field that takes a
shape out, so no marker anywhere in the repository can hold a picture of this rule. The gallery
block is the carrier that can (ADR-0064, ADR-0035). The ADR is the one artifact where a
`load-bearing` record **is** held to showing the options, and the two pictures it carries are
reproduced from [data-model.md](data-model.md) and labelled **Hypothetical** on the spot, which is
what they are until commit 1 lands.

## Complexity Tracking

> Fill ONLY for a Constitution Check violation that must be justified, or a principle VIII ceiling
> exceeded without splitting the feature.

| Departure                                    | Why needed                                                                                                                                                                                                                         | Simpler alternative rejected because                                                                                                                                                                                                                                                                                                                                                                                                                                         |
| -------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `spec.md`, **139 against 120**               | Three clarifications answered in session, three behaviors each carrying a **measured** count, two of them pictures, plus edge cases, testing expectations and four success criteria                                                | **The prose alone is 128 — eight over with every picture free** — and eleven of the lines are the two hand-drawn measured pictures, which the constitution exempts for a _generated_ one on the reasoning that a picture replaces prose. **Splitting**: B1 and B2 are one decision, and B3 is the demonstrable increment under principle II. **No split was taken; this files it, and exempting a measured hand-drawn picture is a constitution question that takes an ADR** |
| `decisions.md`, **76 against 60**            | Four domain entries at the constitution's fields — proposal, altitude, reversal cost, rejected alternative — each with its answer and what the answer added, plus D3's two pictures                                                | The format costs about a dozen lines an entry whatever the prose says, and the fields that would fit in one line are the ones that drop the trade-off                                                                                                                                                                                                                                                                                                                        |
| `research.md`, **Q7 against a full ceiling** | Q7 is the measurement the specification's two pictures are drawn from, and Q1 to Q6 are of the rule being replaced                                                                                                                 | Dropping it would leave two pictures and two counts in the specification with no record of where they came from, which is the one thing this repository does not permit                                                                                                                                                                                                                                                                                                      |
| `plan.md`, **290 against 80**                | Part one's summary, check and sheet tally, plus part two's map of ten changes, **five measurements that corrected it** — one of which corrected a decision rather than a number — five commits, the re-check and the artifact note | The ceiling is written for a plan written in one pass, and the constitution's own rule puts part two on its own branch after the sheet is answered, so the two halves cannot share one ceiling — 082, 083, 148 and 143 filed the same overage for the same reason. Dropping the fifth measurement would leave **D4's stated reason for two passes standing as settled**, which is the one thing in this file a reader would act on                                           |
| `tasks.md`, **748, no ceiling**              | Twenty-seven tasks at the granularity 143 used, and the file carries the baseline measurement the implementer needs rather than an estimate of it                                                                                  | No ceiling applies and none is claimed; the row is filed because the file is the longest in the directory and a reader should know that was a choice. Cutting the per-task citations would bring it under 143's 688 without losing a task, which is why it was not done                                                                                                                                                                                                      |
