# Implementation Plan: A reference carries a horizontal and a vertical offset

**Branch**: `083-a-reference-carries-a-horizontal-and-a-v-deciding` | **Date**: 2026-09-29 |
**Spec**: [spec.md](spec.md)

## Summary

`monospace-diagram`'s `Reference` grows the two offsets its row in the model's _Vocabulary_ already
names, `Position::resolve` adds them to the point the anchor resolves to, and the demonstration's
shipped description spells one — so an endpoint can stand off the side it hangs from, and the
evidence is in a file rather than in a picture. The five pictures the command-line application
prints do not move by a character, which is what proves the arithmetic. `monospace-core` gains
nothing, and the picture the fifth entry draws is the one it draws today.

## What is unusual about this feature

Four departures from the standing context, and nothing else. The description format grows its first
union, and all three spellings were measured rather than argued, because the choice is a real trade
between precise errors and every existing file (research.md Q2). A public struct grows two fields
that nothing sets until a later commit, which is expand/contract rather than one commit. One word of
a contract in another feature's directory is corrected in passing, because it contradicts the code
and this slice amends that same file (research.md Q7). And the gallery's claim that it is the only
carrier able to reach a reference becomes false the moment `at` can hold one (research.md Q6). No
new dependency, no new crate, and no change to `monospace-core`.

## Constitution Check

_GATE: passes before Phase 0, re-checked after Phase 1._

- **I. Process over product** — the offsets land in the crate above the core, which is the harder
  route and the one ADR-0040 and ADR-0041 were taken on. Nothing is short-circuited to reach the
  demonstration's picture sooner.
- **II. Demonstrable increments** — one slice, one increment, B1 through B5, closing with a
  `docs/learning-log.md` entry. The sheet holds three entries against its cap of seven, so the
  signal that a slice is too thick has not fired. The demonstration's fifth picture is the same
  diagram mutated in place, as pictures three and four are.
- **III. One definition of green** — the gate does not change. `wasm` already names
  `monospace-diagram`, no step is added, and the two-commit rule for a new check does not apply.
- **IV. Claims are measured** — every fact this plan states beyond the specification was observed:
  the three wire spellings probed against this crate's own `serde`, the 13 markers and 7 connectors
  counted from the tracked tree, `{21, 3} + (1, 1) = {22, 4}` for the demonstration's own file, and
  `"kind": "arrow"` refused by name. No picture is added to this plan or the sheet, because each
  entry's subject is a type, a wire spelling or a sentence — which is what principle IV says not to
  draw one for.
- **V. Structural and behavioral change never share a commit** — widening `Reference` is structural
  and lands alone, with nothing setting the new fields but zero; the arithmetic that reads them is
  behavioral; the wire change is a third commit. The gallery's rustdoc is corrected in the commit
  that makes it wrong, which is 082's Q7's rule.
- **VI. Decisions recorded at the altitude they belong to** — three domain entries, all answered by
  the maintainer on 2026-09-29, none answered in advance. One reaches past the crate to §1's `Delta`
  row, and lands in the building stage as a `docs` commit of its own rather than as prose here. Two
  questions a reader may look for are answered elsewhere on purpose: 082's D4 for the displacement
  and 082's Q3 for the anchor query (research.md Q4). The module-level answers are in
  [research.md](research.md) and in rustdoc, not on the sheet.
- **VII. The core stays portable** — `monospace-core` gains no item and no arithmetic. Both amounts
  are signed, as `Delta`'s fields already are, and the addition happens over a `Pos` the core
  already exports.
- **VIII. The record is sized to the decision** — `decisions.md` 52 lines of prose against a ceiling
  of 60 and three entries against a cap of seven, and `research.md` 81 against 100. Both fit. The
  one artifact that does not is `spec.md`, which arrived from the specifying stage and is recorded
  below rather than left unmentioned.

## Decisions

The sheet is [`decisions.md`](decisions.md). Part two does not begin until it is answered.

- Entries: 3 — domain: 3, module: 0, tooling: 0
- Answered: 2026-09-29, all three by the maintainer, each with the sheet's recommendation adopted.
  One consequence reaches past the code: D1 reuses `Delta`, so §1's `Delta` row is one clause wider,
  and that amendment follows the answer in the building stage rather than preceding it, which is
  where 082's D3's one-sentence change to §3 landed (`505fd0d`).

## Design _(part two)_

The map, run against the three answers. The detail is in [data-model.md](data-model.md); the public
surface is in [contracts/diagram-api.md](contracts/diagram-api.md), the file format in
[contracts/description-format.md](contracts/description-format.md), and the commands are in
[quickstart.md](quickstart.md).

| Change                                      | Where                                                           | Because                                                                                     |
| ------------------------------------------- | --------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| `Reference.offset: Delta`                   | `monospace-diagram/src/position.rs`                             | The third field §1 already names; `Delta::apply` stays the crate's only arithmetic (D1, Q1) |
| `Position::resolve`'s reference arm         | `src/position.rs`                                               | Ask the figure, ask its anchor, add the offset — §4's order, and Q1's only site (Q1)        |
| `Position::displaced_by`                    | `src/position.rs`                                               | Unchanged: the offsets are there and a displacement still does not reach them (Q4, B3)      |
| `Delta`'s rustdoc                           | `src/delta.rs`                                                  | The type's second meaning, which is the cost D1 names and §1's row has to carry             |
| fifteen `Reference` literals                | `position.rs`, `diagram.rs`, `gallery.rs`, `main.rs`            | Each gains `offset: Delta { dx: 0, dy: 0 }` — expand/contract's structural half             |
| the gallery's block label                   | `src/gallery.rs`                                                | Q6 measured: the label is written by hand, so a field does not move the snapshot by itself  |
| `At` and three mirrors                      | `monospace-cli/src/description.rs`                              | D2: internally tagged, the idiom `ShapeDescription` and `Terminal` already use              |
| `Endpoint.at: At`                           | `src/description.rs`                                            | The field's new type, and the `Position::Absolute` it spells by name                        |
| eight endpoint literals in Rust             | `description.rs`, `tests/cli.rs`                                | Six and two; the eleven `at`s of a box or a line do not change (ADR-0041 is the type)       |
| eighteen endpoint literals in six documents | `README.md`, `CONTRIBUTING.md`, two in `docs/`, two in `specs/` | One edit each, and the `render` step checks 14 of the 18                                    |
| the demonstration's tenth entry             | `crates/monospace-cli/assets/demo.json`                         | Q5: `to` names `#5`'s bottom with an offset of one in each axis, and draws the same point   |
| `demo_without_its_first_entry`              | `monospace-cli/src/main.rs`                                     | D3's consequence, measured: re-reading the text renumbers, so the helper renumbers too      |
| §1's `Delta` row                            | `docs/diagram-model.md`                                         | D1: the type's second use, one clause wider                                                 |
| one word in a contract                      | `specs/079-…/contracts/description-format.md`                   | Q7: `"kind": "arrow"` became `connector` in `c7539bc` and a file carrying it is refused     |

`monospace-core`, `Shape::anchor`'s visibility, `Diagram`'s surface and `xtask` are all unchanged,
and no ADR is written: the three answers sit inside
[ADR-0040](../../../docs/decisions/0040-let-each-shape-answer-its-own-anchor-points.md) and
[ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md), D2's format is
the one ADR-0035 keeps provisional, and D3 is a refusal that leaves §11 open. A fourth record for
two fields and a union would be the record that outranks its subject.

Seven commits, each green on its own, and the order is forced by principles II and V rather than
chosen:

1. `refactor(diagram)`: `Reference.offset` added and the fifteen literals set to zero, with
   `resolve`'s rustdoc saying the field is not read yet. Expand/contract's first half, and the
   reason it is not merged into the next one is that a field nothing reads and a field a method
   reads are a structural change and a behavioral one, which principle V will not put in a diff.
2. `feat(diagram)`: the addition itself, `resolve`'s rustdoc rewritten, the gallery's label edited
   by hand, and the tests for B1, B2 and B3. The label and its snapshot belong here rather than in
   commit 1 because a `refactor` may not move a snapshot.
3. `docs(model)`: §1's `Delta` row, one clause wider (D1). It follows the code rather than preceding
   it, which is where 082's D3's one-sentence change to §3 landed (`505fd0d`).
4. `docs(spec-079)`: the one word, its own commit saying what was wrong (Q7, and 082's Q7's rule).
5. `feat(cli)`: the wire — `At`, the three mirrors, `Endpoint.at`, the eight literals in Rust and
   the eighteen in six documents, and the two refusals. One commit and not two because a tag lands
   everywhere or nowhere: half of it refuses every description in the repository, including seven of
   the 13 markers `cargo xtask render` checks.
6. `feat(cli)`: the demonstration's tenth entry, the helper's renumbering, and the tests for B4 and
   B5. Separate from commit 5 because the file format changing and a picture being proved unchanged
   by it are two different claims, and this one has a diff with no JSON in it.
7. `docs`: the increment's entry in `docs/learning-log.md`.

**Re-checked after Phase 1**: no gate above changes and the design added no step to it. One public
field was widened and one method gained a step, both in the crate the `wasm` step already names, and
neither reaches a core item: the addition is `Delta::apply` — a saturating `saturating_add` on each
axis — over a `Pos` the core already exports. Principle VII reads the same after the design as
before it. The one rule the design accepted with nothing to verify it is the specification's own —
an offset that puts an endpoint inside the figure it hangs from draws it there, composing by the
rule two figures sharing a cell always obey — and it is named as such in all three artifacts rather
than described as tested.

**Two of part one's measurements came back from the design, and one of them is corrected where it is
wrong.** That is the cost of writing the sheet before the code, and the arrangement the constitution
asks for anyway.

- Q2's "13 markers carry a real description and 7 contain a connector" **reproduces exactly**, and
  reproducing it took the rule `xtask` walks the tree with: a marker shown inside a fence is an
  illustration of the grammar rather than an instance of one (`render.rs:211`), which is what takes
  the count from 22 raw occurrences to the 13 the `render` step prints. The design adds the one site
  that enumeration did not name: `CONTRIBUTING.md` shows a marker inside a ````markdown` fence as
  the grammar of a marker, its description carries a connector, and **a reader copies it**. So the
  wire commit re-spells eight markers rather than seven, and the gate checks fourteen of those
  eighteen spellings rather than all eighteen. Everything else in Q2 stands, and part one's own
  Constitution Check keeps its numbers.
- Q6's "the block's label comes from the shape's own `Debug`, so a `Reference` that gains a field
  prints it and the snapshot moves" is **wrong in its mechanism**, and the sentence is fixed in its
  own `docs` commit (principle IV, and _Fixing a commit_: a claim someone could have acted on). The
  label is a string written by hand — which the gallery's own `WHAT` says twice over — so the
  snapshot does not move by itself, and the label has to be edited to
  `Reference(#1, Right, offset (0,0))`, because a label cannot disagree with the values it names.
  Q6's answer of _add no block for the offset_ survives; the reason it gave for the snapshot moving
  is corrected with it.

**And one consequence the sheet did not foresee, which the maintainer answered on 2026-09-29.** D3's
`"shape": "#5"` is a position in a list, and `demo_without_its_first_entry` removes an entry from
the list and reads the text again — so the same `"#5"` names a different box and the test that
compares the fourth picture against it fails. Measured, both ways: the two renderings differ from
row 3 down. The helper renumbers the one reference it moves, and its doc comment says why. This is
the cost D3's own _If this is wrong_ named — ADR-0041's silent hole, in the other direction —
arriving inside the demonstration's own test suite, and it is the argument for D3 rather than
against it.

### Artifacts

`data-model.md`, `contracts/diagram-api.md` and `contracts/description-format.md`, each carrying
something the other two do not. **Two contracts and not one**: the library's public surface grew a
field, which is what 079, 080, 081 and 082 each recorded, and the file format grew a union, which is
its own external interface and its own reader. `contracts/description-format.md` supersedes
[079's](../../079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md), which
stays the record of what the format was — and that file is corrected in place for the one word it
got wrong in a commit of its own, because a contract that contradicts the code is worse than none
(082's Q7). **No second contract for the demonstration**, whose pictures cannot be generated into a
document at all: a marker reads a description the format does hold now, but the _change_ between two
pictures is code, and quickstart.md holds the commands that measure it. **No `<!-- render: -->`
picture in any of the three**, and that is a finding rather than an omission: this slice's subject
is an endpoint one whole cell clear of a border, and B1.2 draws that by hand because a marker has no
way to say what the demonstration does between two drawings.

## Complexity Tracking

| Departure                                             | Why needed                                                                                                                                                                                                                                                                                                          | Simpler alternative rejected because                                                                                                                                                                                                                                                                                                                                |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `spec.md` at 210 lines of prose against a 120 ceiling | It arrived from the specifying stage at this length, and the specification's own 2026-09-29 clarification carries most of it: the three questions put to the maintainer and their answers, each of which changed the spec                                                                                           | The two clarifications cannot be shortened without dropping what the maintainer decided, and the Edge cases section enumerates arrangements the general rule already covers rather than arguing one. A second deciding cycle would reach the same number knowingly                                                                                                  |
| `spec.md` again, the same way                         | 081 is at 273 and 082 at 258, so this is the third specification in a row over the ceiling and none of the three records it                                                                                                                                                                                         | Recording it as a departure once per feature hides that the pattern is the specification template, not this slice — which is worth the maintainer seeing rather than filing under 083                                                                                                                                                                               |
| `plan.md` at 188 lines against an 80 ceiling          | Part one's summary, constitution check and sheet tally, then part two's map of thirteen changes, seven commits, the re-check, the two measurements that came back and the artifact note — where the measurements are the design's own findings and dropping them would leave a wrong mechanism standing as a reason | The ceiling is written for a plan written in one pass, and the constitution's own rule puts part two on its own branch after the sheet is answered, so the two halves cannot share one ceiling. Compressing the map is compressing the reasons, and `tasks.md` follows those reasons rather than re-deriving them — which is what 082 recorded for the same overage |
