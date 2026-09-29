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
  centre is the point the arrow's `from` already holds (Q4). No picture is added to this plan or the
  sheet, because each entry's subject is a type or a sentence — which is what principle IV says not
  to draw a picture for.
- **V. Structural and behavioral change never share a commit** — two changes are structural and land
  first, each alone and with no test added or changed: `Shape::draw` losing the conversion it cannot
  keep, and the gallery being declared. Neither changes a picture. If D4's answer asks for a §4
  amendment it is a `docs` commit, not folded into either.
- **VI. Decisions recorded at the altitude they belong to** — five domain entries on the sheet, D5
  answered in this session and the other four not. Two questions a reader may look for are answered
  elsewhere on purpose: 081's D1 for the core, and the specification's own clarification for what
  `remove` does to a reference (research.md Q6, Q8). The module-level answers are in
  [research.md](research.md) and in rustdoc, not on the sheet.
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
- Answered: D5 on 2026-09-29; the other four _pending_

## Complexity Tracking

> Fill ONLY for a Constitution Check violation that must be justified, or a principle VIII ceiling
> exceeded without splitting the feature.

| Departure                                         | Why needed                                                                                                                                                          | Simpler alternative rejected because                                                                                                                                                                                                   |
| ------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `spec.md` at 243 attributable lines against a 120 | The specification's own 2026-09-28 clarification, which measured the split and chose to carry the overage, restated in `5246003` after that commit's own edit to B1 | A second deciding cycle — issue, branch, specification, sheet and pull request — to reach a number the first one already carries knowingly                                                                                             |
| `research.md` at 148 lines against a 100 ceiling  | Eight questions, five of them carrying a measurement taken this session and cited to a line                                                                         | Compressing below a measurement's citation line costs the measurement, and the file is the only place two of the five are written down. Merging Q6 and Q8 saves four lines and one reader                                              |
| `decisions.md` at 73 lines against a 60 ceiling   | Five entries the constitution asks for at five fields each, and prettier wraps a field at 100 columns                                                               | The format costs about twelve lines an entry whatever the prose, and the fields that would fit in a line are the ones that drop the trade-off. Dropping an entry takes a decision off the table                                        |
| `plan.md` at 86 lines against an 80 ceiling       | Part one's own summary, constitution check, sheet tally and the table above                                                                                         | The ceiling is written for a plan written in one pass. Part two appends the design and the re-check, and the constitution's own rule puts it on its own branch after the sheet is answered, so the two halves cannot share one ceiling |
