<!-- The branch below is the issue title truncated at forty characters by `cargo xtask spec`, which
     landed inside a word. -->
<!-- cspell:ignore referen -->

# Implementation Plan: Taking a shape out leaves the figures that hung from it where they were

**Branch**: `142-taking-a-shape-out-leaves-the-figures-th-deciding` | **Date**: 2026-10-02 |
**Spec**: [spec.md](spec.md)

<!--
  Ceiling: 80 lines (constitution, principle VIII).

  Part one only. The previous run of this stage merged part one; the building branch carries a part
  two written against the rule the freeze replaces, and nothing of it is carried here. Part two —
  Phase 1, run after the maintainer has answered decisions.md — fills Design and produces
  data-model.md, quickstart.md and tasks.md. This comment goes with it.
-->

## Summary

Taking a shape out freezes what hung from it. Every position carrying a reference to the removed
shape becomes the absolute point it was resolving to, so the connector stays drawn exactly where it
was — not re-routed, not moved. The work is one rule in `monospace-diagram`, five amended sections
in [`docs/diagram-model.md`](../../docs/diagram-model.md), one record, one seventh picture in the
shipped demonstration with the arrow still in it, and tests that pin both the freeze and the fact
that a removal has become visible in the picture.

## What is unusual about this feature

- **The rule contradicts a sentence the model already states.** §9 says "Nothing is rewritten and
  nothing cascades", and the freeze is a rewrite reaching figures the removal never named. §9 is
  amended by losing that sentence, not by adding one beside it.
- **The previous deciding stage declined this rule for want of a record** and wrote the question
  down instead. Every measurement that stage took is of the behavior being replaced, which is why
  [D1](decisions.md) asks whether the measuring work merges first or travels with the freeze.
- **The freeze dissolves a question rather than answering it.** How anything would tell a removal
  from a shape that was not there goes away, because a removal's figures stay and the two pictures
  therefore differ. Nothing has to remember that a removal happened, and §11 gains nothing — the
  deciding stage that merged would have put two bullets there and did not, having declined to record
  the question.
- **Two of the specification's counts are derived rather than measured.** The nine and the twelve
  are 15 − 6 and 22 − 10, arithmetic on the removal's measured cost and the arrow's measured
  footprint ([research.md](research.md) Q1, Q3, Q4). The building stage takes the real ones; the
  derivation is here so a reader can check the numbers are not asserted.
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
- **V. Structural and behavioral never share a commit** — the documents and the record are one
  `docs` commit, the freeze and its tests a `feat` beside it. `Position` gains no variant and
  `Diagram` gains no method under the proposal, so the structural half is empty either way.
- **VI. Decisions at the altitude they belong to** — the rule is observable from outside the crate,
  so all four sheet entries are domain-level and the maintainer's. Nothing else here is a decision:
  the six questions answered in research.md are each undone by changing the code that gives them.
- **VII. The core stays portable** — nothing in `monospace-core` moves and the crate it would reach
  is never compiled for the rule, so the `wasm` step compiles what it compiles today.
- **VIII. The record is sized to the decision** — four entries against a cap of seven. **`spec.md`
  is 131 against a 120 ceiling, `decisions.md` 62 against 60, `research.md` 99 against 100**, and
  this file 84 against 80. Three overages go to Complexity Tracking below.
- _The plan runs in two parts_ — this run filled Summary, this check, `research.md` and
  `decisions.md`, and stopped. **`data-model.md`, `quickstart.md` and `tasks.md` are absent and are
  not carried from the building branch**: all three were written against the rule the freeze
  replaces, and a design describing the opposite behavior beside this specification is the defect
  this revision exists to remove. Writing them now would also be taking the four answers rather than
  planning around them.

## Decisions

The sheet is [`decisions.md`](decisions.md). Part two does not begin until it is answered.

- Entries: 4 — domain: 4, module: 0, tooling: 0. **The four answers of 2026-10-02 are void**; they
  answered whether to record the question, which this slice no longer asks.
- Answered: **pending**. The four are: whether the freeze is one slice or two ([D1](decisions.md));
  where it is recorded, given ADR-0041's one contradicted consequence ([D2](decisions.md)); whether
  a removal freezes only the references that resolve as it is taken out ([D3](decisions.md)); and
  whether the rewrite lives inside `remove` or is a change of its own ([D4](decisions.md)).
- **All four go in the order they are numbered.** [D3](decisions.md) and [D4](decisions.md) state
  the rule the code implements; [D2](decisions.md) says where it is written down; [D1](decisions.md)
  says how much of it merges, and it is answered last because it is the only one whose answer
  depends on the other three.

## Complexity Tracking

> Fill ONLY for a Constitution Check violation that must be justified, or a principle VIII ceiling
> exceeded without splitting the feature.

| Departure                         | Why needed                                                                                                                                                                      | Simpler alternative rejected because                                                                                                                                                                                                                                                                                                   |
| --------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `spec.md`, **131 against 120**    | Three clarifications answered in session, three behaviors each carrying a measured count, four named model sections, edge cases, testing expectations and four success criteria | Rewrapping buys nothing — prettier reflows rather than refusing. **Splitting**: B1 and B2 are one decision; B3 is the demonstrable increment under principle II, and moving it out re-opens the 2026-10-01 clarification that refused "a rule recorded only in a test". **No split was taken, and this files it rather than doing it** |
| `decisions.md`, **62 against 60** | Four domain entries at the constitution's fields — proposal, altitude, reversal cost, rejected alternative — plus D3's two pictures                                             | The format costs about a dozen lines an entry whatever the prose says, and the fields that would fit in one line are the ones that drop the trade-off                                                                                                                                                                                  |
| `plan.md`, **84 against 80**      | Nine principles named where the feature touches them, the four entries named in the order they go, and three overages filed                                                     | 082, 083 and 148 filed the same overage for the same reason: the constitution puts part two on its own branch, so the two halves cannot share one ceiling                                                                                                                                                                              |
