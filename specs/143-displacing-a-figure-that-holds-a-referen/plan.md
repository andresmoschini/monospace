<!-- The branch below is the issue title truncated at forty characters by `cargo xtask spec`, which
     landed inside a word. -->
<!-- cspell:ignore referen -->

# Implementation Plan: Displacing a figure that holds a reference moves it

**Branch**: `143-displacing-a-figure-that-holds-a-referen-deciding` | **Date**: 2026-10-01 |
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

A displacement grows a reference's offsets instead of returning it unchanged, so a figure holding a
reference draws as a translation of itself rather than bending its route, and the demonstration
grows a sixth captioned picture that shows it. One private addition in `delta.rs`, one arm in
`Position::displaced_by`, one `block()` call, one step in `main.rs`, and four paragraphs and a test
rewritten rather than contradicted.

## What is unusual about this feature

- **No public API changes and no model section is amended.** `Shape::displaced_by` and `replace` are
  082's surface and keep their signatures, and §4 already states the rule this slice makes true — so
  what §11 loses is a question, not a sentence.
- **The evidence is a picture that has to move to be worth anything.** Today's sixth picture is the
  fifth one, byte for byte, measured in research.md Q1. That is the bug reproducing in the shipped
  demonstration, and it is the reason this slice is demonstrable rather than merely correct.
- **The arithmetic is new and its sentence is not.** `delta.rs:43` calls itself the crate's only
  arithmetic and it takes a `Pos`; growing an offset is the first delta added to a delta, so that
  sentence is rewritten in the commit that makes it false rather than left standing.
- **Four places declare the behavior this slice reverses, and the specification names three** —
  research.md Q6, and D3.
- No dependency is added, `monospace-core` does not move, `xtask` does not change, no gate step is
  added, and nothing reaches for a terminal.

## Constitution Check

_GATE: passes before Phase 0, re-checked after Phase 1._

- **II. Demonstrable increments** — the sixth picture is the fifth with the arrow two rows lower and
  both boxes where they were, and the first five are byte for byte what they are. The arrangement
  itself is drawn today by nothing, so B1.1 and B1.2 are drawn by building the post-condition.
- **IV. Claims are measured** — every picture in the specification was reproduced on this branch
  (research.md Q2), the sixth-is-the-fifth claim was measured rather than asserted (Q1), and the
  1916 renderings that cannot move were counted per file (Q3).
- **V. Structural and behavioral never share a commit** — the arithmetic in `delta.rs` is a private
  addition nothing outside calls, so it can land as a `refactor` with the four `No test is added`
  guarantees intact, and the arm that uses it is the `feat` after it.
- **VI. Decisions at the altitude they belong to** — the rule is observable from outside the crate,
  so it is domain-level and the record is the maintainer's. D1 asks whether its subject is this case
  or 082's unrecorded rules too, and no ADR in the repository mentions displacement at all.
- **VIII. The record is sized to the decision** — three entries against a cap of seven. The
  specification is **249 attributable lines against a 120 ceiling**, measured on this branch with
  the count 083 and 148's own checklists used — 304 lines, less 41 blank, 10 fence markers and 4
  HTML comment lines, which are 1, 2, 3 and the one inside SC-005. Nineteen of those are this
  branch's: D2's gallery block in **B3**, **SC-004** and **Testing expectations**, and D3's naming
  of `shape.rs:179-183` beside the three already named. It goes to Complexity Tracking in part two.
- _The plan runs in two parts_ — this run filled Summary, this check, `research.md` and
  `decisions.md`, and stopped. `data-model.md`, `contracts/` and `quickstart.md` are part two, and
  writing them now would be taking the three answers rather than planning around them.

## Decisions

The sheet is [`decisions.md`](decisions.md). Part two does not begin until it is answered.

- Entries: 3 — domain: 3, module: 0, tooling: 0
- Answered: 2026-10-01 — all three, two as proposed and D2 wider than its proposal. D2 adds the
  gallery's third block beside the demonstration's sixth picture, so the specification amends **B3**
  and **SC-004** as well as D3's **Testing expectations**. Six module-level questions were answered
  in [`research.md`](research.md) instead, each undone by changing the code that gives it.

## Design _(part two)_

[What the answered sheet implies for the code: which modules are touched, what is added, what is
removed. The detail belongs in data-model.md and contracts/; this is the map.]

### Artifacts

`research.md` and `decisions.md` are part one's and are written. `data-model.md`, `contracts/` and
`quickstart.md` are part two's, and are named here only to say which this feature is likely to need
and which it will not — the constitution's _Development Workflow_ owns the list itself.

<!-- data-model.md and contracts/ are written where they carry something; a file restating the
     spec's entities in other words is the duplication principle VIII refuses. Say here which ones
     this feature does not need, and why. -->

## Complexity Tracking

> Fill ONLY for a Constitution Check violation that must be justified, or a principle VIII ceiling
> exceeded without splitting the feature.

| Departure                                   | Why needed                                                                                                                                                                                                        | Simpler alternative rejected because                                                                                                                                                                                                   |
| ------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `decisions.md` at 76 lines against a 60 one | Three domain entries at the constitution's five fields each, and three answers that each name a consequence outside the sheet: the record's subject, the gallery's third block, and the sections `spec.md` amends | The format costs about twelve lines an entry whatever the prose, and the fields that would fit in a line are the ones that drop the trade-off. Cutting the answers costs the consequence, and the consequence is what part two acts on |
| `plan.md` at 86 lines against an 80 one     | Part one's own summary, check and sheet tally, plus the three lines the answered sheet added back                                                                                                                 | Part two appends the design, the artifacts and the re-check, so the number this row states is already the one part two moves; writing it now and correcting it there is cheaper than leaving the overage unrecorded                    |
