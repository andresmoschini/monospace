# Implementation Plan: A connector endpoint hangs from a box's side anchor

**Branch**: `082-a-connector-endpoint-hangs-from-a-box-s-deciding` | **Date**: 2026-09-28 |
**Spec**: [spec.md](spec.md)

## Summary

`monospace-diagram` gains three types, and the one place that reads them: `Anchor` names a side,
`Reference` names a shape and a side of it, and `Position` is either a point or a `Reference` — held
by a connector's endpoint and by nothing else, so ADR-0041's restriction is a field's type rather
than a sentence beside it. A position resolves itself,
`Position::resolve(&self, &Diagram) -> Option<Pos>`, and it is public (D5), so a caller that knows a
position and its diagram can ask; what does not resolve takes the whole connector out of the
picture, which is ADR-0041's rule reaching an observable case for the first time. `monospace-core`
gains nothing, which is 081's D1 settled rather than re-asked. The demonstration grows a fifth
picture: the box the arrow already hangs from, displaced, with the arrow landing on its new side.

## What is unusual about this feature

Four departures from the standing context, and nothing else. Three new public types in
`monospace-diagram`, in one new module, and one of them is not a leaf: `Position` names `Diagram`,
and so does `Shape::draw`, which takes the `&Diagram` it needs instead of an answer handed to it
(research.md Q1, Q2; D5). `Endpoint.at` changes type, so `description.rs` in the command-line
application wraps every `at` it reads — and the `From<Endpoint> for monospace_core::Endpoint` impl
cannot survive, since the core's endpoint holds a `Pos` and a `Position` has none to give it. One
dead file comes alive: `gallery.rs` is in `src/` and is not declared in `lib.rs`, so its three tests
and three snapshots have never run (research.md Q5). And a demonstration that prints five pictures
where it printed four. No new dependency, no new crate, no change to the description format, and no
change to `monospace-core`.

## Constitution Check

_GATE: passes before Phase 0, re-checked after Phase 1._

- **I. Process over product** — the anchors and the resolution both land in the crate above the
  core, which is the harder route and the one ADR-0040 and ADR-0041 were taken on. Nothing is
  short-circuited to reach the demonstration's picture sooner.
- **II. Demonstrable increments** — one slice, one increment, B1 through B5, closing with a
  `docs/learning-log.md` entry. The sheet holds five entries against its cap of seven, so the signal
  that the slice is too thick has not fired. The demonstration's fifth picture is the same diagram
  mutated in place, as pictures three and four are, rather than a second one built alongside it.
- **III. One definition of green** — the gate does not change. `wasm` already names
  `monospace-diagram`, the new types reach no core item, and no step is added, so the two-commit
  rule for a new check does not apply. What this slice does find is a check that was never running,
  and that is a `test` commit on its own rather than a new step (research.md Q5).
- **IV. Claims are measured** — the two facts this plan states beyond the specification are measured
  and not inferred: the gallery's three tests pass against their committed snapshots when the file
  is declared (Q5), and the demonstration's box is the third entry at `{9, 2}`, whose right side
  center is the point the arrow's `from` already holds (Q4). No picture is added to this plan or the
  sheet, because each entry's subject is a type or a sentence — which is what principle IV says not
  to draw a picture for.
- **V. Structural and behavioral change never share a commit** — two changes are structural and land
  first, each alone and with no test added or changed: `Shape::draw` losing the conversion it cannot
  keep, and the gallery being declared. Neither changes a picture. The two answers that reach past
  the code are `docs` commits of their own: one sentence in §3 (D3), and no amendment to §4 (D4).
- **VI. Decisions recorded at the altitude they belong to** — five domain entries, all answered. Two
  questions a reader may look for are answered elsewhere on purpose: 081's D1 for the core, and the
  specification's own clarification for what `remove` does to a reference (research.md Q6, Q8). The
  module-level answers are in [research.md](research.md) and in rustdoc, not on the sheet.
- **VII. The core stays portable** — `monospace-core` gains no item and no arithmetic, and Q2 makes
  that structural rather than merely intended: the diagram's endpoint can no longer be converted
  into the core's, so anything the core were to grow for this feature would have to be reached the
  long way round. The anchors are computed from fields the core already publishes.
- **VIII. Sized to the decision** — three artifacts carry a ceiling they exceeded and each is
  recorded below, which is what this file's Complexity Tracking is for. The specification's is the
  maintainer's own carry; the other two are this plan's.

## Decisions

The sheet is [`decisions.md`](decisions.md). Part two does not begin until it is answered.

- Entries: 5 — domain: 5, module: 0, tooling: 0
- Answered: 2026-09-29 — all five

## Design _(part two)_

The map, run against the five answers. The detail is in [data-model.md](data-model.md) and
[contracts/diagram-api.md](contracts/diagram-api.md); the commands are in
[quickstart.md](quickstart.md).

| Change                                        | Where                                              | Because                                                                                           |
| --------------------------------------------- | -------------------------------------------------- | ------------------------------------------------------------------------------------------------- |
| `Anchor`, `Reference`, `Position`             | `monospace-diagram/src/position.rs`, new module    | Three rows of the model's _Vocabulary_, and only `Position` holds a behavior (Q1, D1)             |
| `Shape::anchor` and the two helpers           | `src/position.rs`                                  | The four side centers, one arm per kind, `pub(crate)` because drawing is the only reader (Q3, B1) |
| `Shape::draw`                                 | `src/shape.rs`                                     | Takes the `&Diagram` it needs; its connector arm resolves both endpoints before writing (Q2)      |
| `Endpoint.at`                                 | `src/shape.rs`                                     | `Pos` becomes `Position` — the only field whose type changes, and the whole of D2                 |
| `From<Endpoint> for monospace_core::Endpoint` | `src/shape.rs`                                     | Removed: the core's endpoint holds a `Pos` and a `Position` has none to give it (Q2)              |
| `Position::resolve`                           | `src/position.rs`                                  | Public and `#[must_use]`, with no offset arithmetic because there are no offsets (D5, D1)         |
| `Position::displaced_by`                      | `src/position.rs`                                  | A displacement adds to a point; a reference has none to add to until #83 (B4)                     |
| `displaced_by`'s connector arm                | `src/shape.rs`                                     | Both endpoints now go through it, so a hanging end stands still (B4)                              |
| thirteen `Endpoint` literals touched          | `src/shape.rs`, `src/diagram.rs`, `src/gallery.rs` | Twelve gain `at: … .into()`; the thirteenth goes with the test it pins (Q1)                       |
| one test removed                              | `src/shape.rs`                                     | It pinned the conversion above; its claim returns by drawing (Q2, see Complexity Tracking)        |
| `#[cfg(test)] mod gallery;`                   | `src/lib.rs`                                       | Three tests and three snapshots have never run (Q5)                                               |
| `order:` becomes `change:`                    | `src/gallery.rs`                                   | A displacement is a change to a figure, not to the order                                          |
| a fourth gallery block, and its snapshot      | `src/gallery.rs`, `src/snapshots/gallery/`         | The only carrier in the crate that can reach a reference (Q4, ADR-0064)                           |
| `at: Position::Absolute(…)`                   | `monospace-cli/src/description.rs`                 | One line, in commit 1: the field's new type breaks the `into()` there                             |
| the fifth picture, and the tests reading it   | `monospace-cli/src/main.rs`, `tests/cli.rs`        | B5; five pictures, appended rather than interleaved                                               |
| one sentence in §3                            | `docs/diagram-model.md`                            | D3: a caller may spell an identity while the diagram still issues them                            |
| three "no figure can hold a reference yet"    | `specs/081-…/contracts/diagram-api.md`             | A contract describes what a caller sees now (Q7)                                                  |

`monospace-core`, `assets/demo.json`, the description format and `xtask` are unchanged — which is
the whole of D1 and of B5.8. **No ADR is written.** The five answers sit inside
[ADR-0040](../../../docs/decisions/0040-let-each-shape-answer-its-own-anchor-points.md) and
[ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md), whose _Decision
Outcome_ already states that a position is absolute or a reference, that resolving one means asking
the referenced shape for its anchor, and that only a connector's endpoint may hold one; D4 is the
refusal to amend §4, and D3's answer is that one sentence. The records 079 through 081 wrote did not
need one either, and a fifth for a refinement of two is the record that outranks its subject.

Seven commits, each green on its own, and the order is forced by principles II and V rather than
chosen:

1. `refactor(diagram)`: the three types, `Endpoint.at` becoming one, the `From` conversion gone, the
   drawing method's argument, the twelve literals wrapped, `description.rs`'s one line, and the one
   test that pinned the conversion removed. No picture changes, which is the whole claim, and it is
   first because every later commit reads the types it leaves and because the workspace has to build
   at every commit.
2. `test(diagram)`: the gallery declared, and its second line's name widened from `order:` to
   `change:`. 37 tests become 40 against snapshots exactly as committed, and the one `insta` review
   this slice offers is here rather than inside commit 3.
3. `feat(diagram)`: the anchors, the resolution, the non-resolution that takes a whole connector
   out, the displacement that leaves a reference alone, the tests for B1 through B4, and the
   gallery's fourth block. It is one commit and not four because the anchor query has no public
   consumer, so a commit carrying it alone is dead code and the gate denies it.
4. `docs(spec-081)`: the contract that said a figure cannot hold a reference, corrected — its own
   commit, saying what was wrong, because it is a record rather than code (Q7, and the rule in the
   constitution's _Fixing a commit_).
5. `docs(model)`: the one sentence §3 gains (D3). It lands before the demonstration that reads most
   naturally against it.
6. `feat(cli)`: the fifth picture, and the tests that read the demonstration. B5.
7. `docs`: the increment's entry in `docs/learning-log.md`.

**Re-checked after Phase 1**: no gate above changes, and the design added no step to it. One module
and three types went into the crate the `wasm` step already names, and none of them reaches a core
item or does arithmetic the core cannot do: the four side centers are computed from `at`, `size`,
`len` and `orientation`, every one of which the core already publishes as `pub`. Principle VII reads
the same after the design as before it. The one rule the design found that is accepted with nothing
to verify it is the specification's own — a shape whose position does not resolve offers no anchor
point — and it is named as such in all three artifacts rather than described as tested.

**One correction to the check above**, found by designing rather than by planning, and recorded in
Complexity Tracking rather than by rewriting the bullet: commit 1 does remove a test.
`the_mirrors_terminal_reaches_the_cores_intact` pinned the conversion `Shape::draw` cannot keep, so
it goes with it. No test is added or changed, the separation of structural from behavioral holds,
and no picture differs — but "no test is added or changed" is not true as written, and a plan whose
own gate note is false is worse than one that exceeds a ceiling.

### Artifacts

`data-model.md`, `contracts/diagram-api.md` and `quickstart.md`, all three of them carrying
something the other two do not. **No `contracts/description-format.md`**: the wire format is
unchanged, `at` is still a point on it, and
[079's](../../079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md) stays the
record of what it is. **No second contract for the demonstration**, whose pictures cannot be
generated into a document at all — a marker reads a description the format does not hold
(research.md Q4) — and 079's contract already records that a path prints one picture and nothing
else. **No `<!-- render: -->` picture in any of the three**, and that is a finding rather than an
omission: the subject of this slice is exactly what a marker cannot reach, so the picture of a
reference belongs to the gallery and `data-model.md` names the block that holds it.

## Complexity Tracking

> Fill ONLY for a Constitution Check violation that must be justified, or a principle VIII ceiling
> exceeded without splitting the feature.

| Departure                                                 | Why needed                                                                                                                                                                                                                                                                                                                                 | Simpler alternative rejected because                                                                                                                                                                                                                                                                                                                      |
| --------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `spec.md` at 243 attributable lines against a 120         | The specification's own 2026-09-28 clarification, which measured the split and chose to carry the overage, restated in `5246003` after that commit's own edit to B1                                                                                                                                                                        | A second deciding cycle — issue, branch, specification, sheet and pull request — to reach a number the first one already carries knowingly                                                                                                                                                                                                                |
| `research.md` at 148 lines against a 100 ceiling          | Eight questions, five of them carrying a measurement taken this session and cited to a line                                                                                                                                                                                                                                                | Compressing below a measurement's citation line costs the measurement, and the file is the only place two of the five are written down. Merging Q6 and Q8 saves four lines and one reader                                                                                                                                                                 |
| `decisions.md` at 79 lines against a 60 ceiling           | Five entries the constitution asks for at five fields each, plus a sixth on the one that answers D5, and prettier wraps a field at 100 columns                                                                                                                                                                                             | The format costs about twelve lines an entry whatever the prose, and the fields that would fit in a line are the ones that drop the trade-off. Dropping an entry takes a decision off the table                                                                                                                                                           |
| `plan.md` at 170 lines against an 80 ceiling              | Part one's own summary, constitution check and sheet tally, then part two's map of fifteen changes, seven commits, the ADR paragraph, the artifacts note, the re-check and the correction the re-check found                                                                                                                               | The ceiling is written for a plan written in one pass. Part two appends the design and the re-check, and the constitution's own rule puts it on its own branch after the sheet is answered, so the two halves cannot share one ceiling. Compressing the map is compressing the reasons, and `tasks.md` follows those reasons rather than re-deriving them |
| Principle V's "no test is added or modified", in commit 1 | The `From<Endpoint> for monospace_core::Endpoint` impl this slice deletes is the subject of `the_mirrors_terminal_reaches_the_cores_intact`, so the test goes with it. It is **removed**, and none is added or changed; no picture differs, which is the commit's own claim and is checked by diffing the demonstration's output across it | Keeping the conversion alive for the commit's sake by feeding it a `Pos` from somewhere — the fictional answer ADR-0040 refuses, and the reason Q2 measured that it cannot survive. Re-pinning the same claim in the same commit is a test **added** to a structural change, which the rule forbids in the other direction                                |
