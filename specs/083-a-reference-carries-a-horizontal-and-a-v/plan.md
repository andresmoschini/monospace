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

Not written. Part one stops here: writing `data-model.md`, `contracts/` or `quickstart.md` is taking
the decisions D1 to D3 are asking about.

## Complexity Tracking

| Departure                                             | Why needed                                                                                                                                                                                                                | Simpler alternative rejected because                                                                                                                                                                                                                               |
| ----------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `spec.md` at 210 lines of prose against a 120 ceiling | It arrived from the specifying stage at this length, and the specification's own 2026-09-29 clarification carries most of it: the three questions put to the maintainer and their answers, each of which changed the spec | The two clarifications cannot be shortened without dropping what the maintainer decided, and the Edge cases section enumerates arrangements the general rule already covers rather than arguing one. A second deciding cycle would reach the same number knowingly |
| `spec.md` again, the same way                         | 081 is at 273 and 082 at 258, so this is the third specification in a row over the ceiling and none of the three records it                                                                                               | Recording it as a departure once per feature hides that the pattern is the specification template, not this slice — which is worth the maintainer seeing rather than filing under 083                                                                              |
