<!-- The branch below is the issue title truncated at forty characters by `cargo xtask spec`, which
     landed inside a word: cspell:ignore carri -->

# Implementation Plan: A description names its shapes, and carries the ordinal the next one takes

**Branch**: `148-a-description-names-its-shapes-and-carri-deciding` | **Date**: 2026-09-30 |
**Spec**: [spec.md](spec.md)

<!--
  Ceiling: 80 lines (constitution, principle VIII).

  This file is filled across two runs, and the split is a rule, not a convenience
  (constitution, _The plan runs in two parts_).

  Part one — Phase 0. Fills Summary, Constitution Check, and produces research.md and
  decisions.md. It stops there. data-model.md, contracts/ and quickstart.md are NOT written:
  writing them is taking the decisions the sheet is asking about.

  Part two — Phase 1, run after the maintainer has answered decisions.md. Fills Design and
  Complexity Tracking, and produces the design artifacts. This comment goes with it.

  The technical context this project would fill in is fixed and lives elsewhere: Rust edition 2024
  at the pinned toolchain, a virtual cargo workspace under crates/, no storage, a terminal consumer,
  and the WebAssembly boundary the gate enforces. Repeating it per feature is the duplication
  principle VIII refuses. State only what is unusual about THIS feature.
-->

## Summary

Every entry of a description names its shape, and the description carries the ordinal the next shape
takes — so a reference names the shape the file names rather than the place it is written, and a
shape added to a diagram read from a file continues that file's numbering. The reader passes both
through, `monospace-diagram` grows two methods, and 114 identities land in 44 descriptions without
one picture moving.

## What is unusual about this feature

- **A public API grows in `monospace-diagram`**, for the first time because something outside it
  chooses an identity. That is what makes three sections of the model and an ADR part of this slice
  rather than a later one.
- **26 files change and none of them draws**: 25 tracked markers, `assets/demo.json`, and the shape
  literals in four Rust files. Every picture must come out byte for byte, and the gate re-draws all
  25 of them, so a wrong edit is a red gate rather than a quiet difference.
- **The format gains two required fields** — `id` on every entry and `next_id` beside the canvas.
  research.md Q3 measures the asymmetry that makes it safe: a misspelled `id` is a missing-field
  error, and a misspelled extra key is dropped in silence as it always has been.
- No dependency is added, `monospace-core` does not move, and nothing in the new API reaches for a
  terminal.

## Constitution Check

_GATE: passes before Phase 0, re-checked after Phase 1._

<!-- Against the principles by name, not a recital. One line each, and only where the feature
     touches the principle. Name the principle that is at risk and say how the plan satisfies it. -->

- **II. Demonstrable increments** — the slice's evidence is that nothing demonstrable changes: the
  same five captioned pictures character for character, and research.md's three files drawing one
  picture from either order of the two boxes.
- **IV. Claims are measured** — the specification's "63 shapes in the 24 markers" sits in
  research.md Q4 beside the measured 65 in 25, and the three files were run rather than argued. D2's
  answer is a promise the code deliberately does not make, and it is written down as one.
- **V. Structural and behavioral never share a commit** — the 114 identities can land in all 44
  descriptions as one `refactor` while the reader still drops them in silence, which is measured;
  the two methods and the fields becoming required are the `feat` after it.
- **VI. Decisions at the altitude they belong to** — §3, §9 and §11 are amended and one
  `load-bearing` ADR is written, because a caller choosing an identity is observable from outside
  the crate. The four decisions are on the sheet; the six module-level answers are not.
- **VIII. The record is sized to the decision** — four entries against a cap of seven, and the 89
  lines the specification names are in JSON and in a format contract rather than in prose.
- _The plan runs in two parts_ — this run filled Summary, this check, `research.md` and
  `decisions.md`, and stopped. `data-model.md`, `contracts/` and `quickstart.md` are part two, and
  writing them now would be taking the decisions rather than planning them.

## Decisions

The sheet is [`decisions.md`](decisions.md). Part two does not begin until it is answered.

- Entries: 4 — domain: 4, module: 0, tooling: 0
- Answered: 2026-09-30, all four with the proposal adopted. Two consequences reach past the code and
  are part two's to place: §3's amendment and the ADR, which follow the answer rather than preceding
  it, which is where 082's D3's one-sentence change to §3 landed (`505fd0d`).

## Design _(part two)_

The map, run against the four answers. The detail is in [data-model.md](data-model.md); the
library's public surface is in [contracts/diagram-api.md](contracts/diagram-api.md), the file format
in [contracts/description-format.md](contracts/description-format.md), and the commands are in
[quickstart.md](quickstart.md).

| Change                                                | Where                                                                           | Because                                                                                                                 |
| ----------------------------------------------------- | ------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `numbered_from(next: u32) -> Diagram`                 | `monospace-diagram/src/diagram.rs`                                              | D1/Q1: the ordinal **the next `add` takes**, stored as itself, so a public seeding method has no off-by-one             |
| `add_under(&mut self, id: ShapeId, shape: Shape)`     | `src/diagram.rs`                                                                | D3/Q2: the caller chooses, and it hands back nothing because it cannot fail (B3.1)                                      |
| what `next` means                                     | `src/diagram.rs`                                                                | The same number, named for what it is: the last ordinal issued becomes the one the next `add` takes                     |
| `ShapeId`'s rustdoc                                   | `src/diagram.rs`                                                                | "unique within that diagram" becomes a sentence about the identities **the diagram issues** (D2)                        |
| `Description.next_id`, and an `id` on all three kinds | `monospace-cli/src/description.rs`                                              | Q3: required in the type, so a missing one is refused by name like `canvas` and `shapes`                                |
| `into_diagram`                                        | `src/description.rs`                                                            | `numbered_from(next_id)`, then `add_under` per entry in array order — the order is still the order                      |
| `demo_without_its_first_entry`'s renumbering          | `crates/monospace-cli/src/main.rs`                                              | It goes with the comment that says why, being the cost this slice removes (Q5)                                          |
| three comments saying a file names shapes by position | `src/main.rs`                                                                   | They become false in the reader commit and are corrected in the commit that makes them so (083's Q6)                    |
| 114 `id` values, 44 `next_id` values                  | seven tracked documents, `CONTRIBUTING.md`, `assets/demo.json`, four Rust files | Measured below, and `cargo xtask render` re-draws 25 of them                                                            |
| §3 twice, §9's `add` row, §11                         | `docs/diagram-model.md`                                                         | D2 is why "unique" cannot stay a promise, and §11's first trigger has fired (Q6)                                        |
| one ADR, `load-bearing`                               | `docs/decisions/`                                                               | Principle VI's own test: `monospace-cli` calls both methods, and reversing means 114 identities leaving 44 descriptions |

`add`, `new`, `find`, `get`, `remove`, `replace`, `forward`, `backward` and `draw` keep their
signatures; so do `Shape`, `Position`, `Reference` and every leaf type. `monospace-core` does not
move, `xtask` does not change, and no gate step is added — so principle III's two-commit rule for a
new check does not apply.

### What the mechanical change actually measures

Q4 said "25 markers … 65 shapes across them, in eight files". Measured on 2026-09-30 with `xtask`'s
own rule: **25 markers, 65 shapes, in seven files**, plus `CONTRIBUTING.md`'s one marker shown
inside a Markdown fence — one shape, and the walker steps over it, exactly as it steps over a
decision sheet's template. Beside those, `assets/demo.json` holds 26 entries, and the four Rust
files hold 17 `"canvas"` literals and 22 shape-level `"kind"` spellings between them. So:

| What                                           | How many | Checked by the gate            |
| ---------------------------------------------- | -------- | ------------------------------ |
| `id` values to write                           | 114      | 65 of them, through `render`   |
| `next_id` values to write, one per description | 44       | 25 of them, through `render`   |
| descriptions that stop reading without them    | 44       | 25 through `render`, 19 by eye |

`sweep.rs` is four edits rather than 1856: its cases are built as text from three shape templates
and one crossing pair (`sweep.rs:65`, `:131`), and **a snapshot pins a rendering, not a
description** — so none of the eight characterization files moves, and no ADR-0053 report is owed.

### Six commits, and the order is forced

1. `refactor(cli)`: the 114 identities and 44 counters land in every description, and the reader
   drops them in silence — measured, three ways, in research.md's opening section. No picture moves,
   no test is added or changed, and the `render` step re-draws all 25.
2. `feat(diagram)`: `numbered_from` and `add_under`, with the contract tests for B1.3 and B2. They
   are one commit because neither is usable alone — a diagram that can be seeded but not named under
   is nothing, and a diagram that can be named under but not seeded issues from `#1` regardless.
3. `feat(cli)`: the reader — both fields required, `into_diagram` passing them through, the two
   refusals, and the demonstration's comments and workaround. **It cannot land before commit 1**,
   because it is the commit that refuses all 44 descriptions.
4. `docs(model)`: §3's two amendments, §9's `add` row and §11 — following the code, which is where
   082's D3's one-sentence change to §3 landed (`505fd0d`).
5. `docs(adr)`: the record and its row in `docs/decisions/README.md`, as a whole row rather than a
   fragment of one.
6. `docs`: the increment's entry in `docs/learning-log.md`.

### Re-checked after Phase 1

No gate above changes, and the design added no step to it. Both new methods live in
`monospace-diagram`, which the `wasm` step already compiles, and neither reaches a core item: the
counter is a `u32` and the placement is a `Vec::push` — the same `Placed` an `add` pushes. Principle
VII reads the same after the design as before it, and principle II still holds on the evidence
research.md measured: the five captioned pictures come out character for character, and the three
files it read draw one picture from either order of the two boxes.

Two claims are **accepted with nothing to verify them**, named here per principle IV rather than
described as tested: that a stale `next_id` hands back an identity already in use — D2's answer is a
promise the code deliberately does not make, and the specification's _Edge cases_ says so — and that
a repeated `id` is a shape nobody can name. Both are pinned as behaviors rather than as bugs, so a
later slice that decides to report them has to say so rather than discover it.

**And one number from part one is corrected here, rather than quietly.** The Summary and the
Constitution Check above say "91 identities land in 26 files"; measured, it is **114 identities
across 44 descriptions**. Part one counted the 25 markers and `demo.json` and missed
`CONTRIBUTING.md`'s one and the 22 in Rust, and it read "descriptions" as "files". The correction is
in this commit and says what was wrong, because the number sizes the mechanical work and someone
could have acted on it.

### Artifacts

`data-model.md`, `contracts/diagram-api.md` and `contracts/description-format.md`, each carrying
something the other two do not. **Two contracts and not one**, for 083's reason: the library's
public surface gains two methods, which is what 079, 080, 081, 082 and 083 each recorded, and the
file format gains two required fields, which is its own external interface and its own reader. This
one supersedes
[083's](../../083-a-reference-carries-a-horizontal-and-a-v/contracts/description-format.md) and
[083's diagram-api](../../083-a-reference-carries-a-horizontal-and-a-v/contracts/diagram-api.md),
which stay the record of what the format and the surface were until this slice. **No second contract
for the demonstration**, whose pictures cannot be generated into a document at all: a marker reads a
description the format holds now, but the _change_ between two pictures is code, and `quickstart.md`
holds the commands that measure it. **No `<!-- render: -->` picture in any of the three**, and that
is research.md's finding rather than an omission: every option pair on the sheet draws the same
picture, and a named shape is not expressible in the format `xtask` reads (ADR-0064).

## Complexity Tracking

> Fill ONLY for a Constitution Check violation that must be justified, or a principle VIII ceiling
> exceeded without splitting the feature.

| Departure                                                  | Why needed                                                                                                                                                                                                                                                                                                                                                                                                                           | Simpler alternative rejected because                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                                       |
| ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `spec.md` at 194 lines of prose against a 120 ceiling      | It arrived from the specifying stage at this length, and its 2026-09-30 clarification carries three of the questions whose answers are on the sheet                                                                                                                                                                                                                                                                                  | The clarification cannot be shortened without dropping what the maintainer decided, and 081, 082 and 083 each arrived over the same ceiling — 273, 258 and 210 by the count this row uses. A fourth deciding cycle would reach the same number knowingly, and this row is filed so the pattern stays visible rather than becoming the norm. Two of the 194 are a `cspell` directive this plan's gate run added, since `cargo xtask spec` truncates a branch name at forty characters and this one landed inside a word                                                                     |
| `plan.md` against its 80 ceiling                           | Part one's summary, constitution check and sheet tally, then part two's map of eleven changes, the measured mechanical change, six commits, the re-check, the correction and the artifact note                                                                                                                                                                                                                                       | The ceiling is written for a plan written in one pass, and the constitution's own rule puts part two on its own branch after the sheet is answered, so the two halves cannot share one ceiling — 082 recorded the same overage for the same reason. Dropping the measured counts would leave the 114 identities with nothing behind them, and dropping the correction would leave a wrong number standing in a place a reader acts on                                                                                                                                                      |
| ADR-0066 at 164 lines against a 150 `load-bearing` ceiling | The record holds six alternatives, and two of them are close enough to the chosen one that their Good half is worth keeping — the reversed counter convention and advancing past a written `#N` are both things a later reader would otherwise re-propose. Three of the six were cut to one paragraph each, and the section on why there is no picture in the record is kept whole because it is the finding rather than an omission | The constitution's first answer to an exceeded ceiling is to split the decision, and that was measured rather than assumed: the three rejected-together options were fused into a single subsection and the two remaining subsections keep theirs, which is the split that follows how close each option was. What is left is 14 lines of the two costs this decision accepts on purpose — a stale `next_id` and a repeated identity — and cutting either would leave an ADR with costs the reader cannot see, which principle VI's "an ADR with no costs listed is a sales pitch" forbids |
