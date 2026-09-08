---
status: accepted
date: 2026-09-07
decision-makers: Andrés Moschini
---

# Mirror the decided-cell skip in `Above`, accepting a second unverifiable branch

## Context and Problem Statement

[ADR-0017](0017-ask-the-cell-whether-it-is-decided.md) accepted one branch nothing can verify:
`stamp` under `Below` skips a decided target instead of merging into an identical cell. It settled
at medium confidence (60%) for that branch specifically, and named what would lower it further: "a
second unverifiable branch arriving for the same reason — one is a considered cost, two is a habit."

Unifying `merge_arm_above` and `merge_arm_below` into one `merge_arm(top, bottom)` surfaced exactly
that second branch. The unified rule is symmetric: whichever cell plays `top` decides outright when
it has no `Unset` arm, regardless of `bottom`. That is true for `Above` (`top` is the incoming
stamp) exactly as it is for `Below` (`top` is the target) — [`docs/model.md`](../model.md) states it
that way already, under _Stamping_, without mentioning a mode: "the base stroke ends up owned by the
topmost figure, and each arm ends up owned by the topmost figure that decided it." Code that only
implements that sentence in one direction is telling half of it.

The question is whether to write the `Above` half of that symmetry into `stamp`, given ADR-0017
named doing so as the trigger to reconsider rather than an obvious next step.

## Decision Drivers

- The model states the topmost-figure rule without reference to a stamp mode; a reader who has seen
  the `Below` branch and not this one has no way to tell whether the asymmetry is intentional or an
  oversight.
- ADR-0017's own warning: a second branch nothing can verify, arriving for the same reason, is named
  there as a habit forming rather than a second considered cost.
- Unlike `Below`'s skip, `Above` never clones today — `cell` arrives owned and its base moves for
  free either way — so there is no allocation this branch avoids. Verified by reading `merge` before
  writing this record, not assumed.
- Before this decision, nothing in the test suite or the CLI reached the branch at all. Confirmed by
  making it panic on purpose and running every test and the CLI binary: zero hits.

## Considered Options

- **A** — Add the `Above` branch, accepting a second unverifiable one.
- **B** — Leave it asymmetric, as ADR-0017's confidence note assumed would happen absent a reason to
  do otherwise.
- **C** — Express the skip inside `merge` itself, so both directions get it without a per-mode
  branch in `stamp`.

## Decision Outcome

Chosen option: **A, add it**, because the asymmetry was an accident of `Below` being the one that
happened to need it for a real cost, not a statement that `Above` behaves differently — and the
model's own sentence does not distinguish the two.

The cost ADR-0017 flagged is accepted with eyes open rather than avoided: this is the second branch
that no test can prove necessary, and this record exists because ADR-0017 said that would matter.

### Consequences

- Good, because `stamp`'s four match arms now read as two pairs, one per mode, each pair a shortcut
  and its fallback — the code shows the same rule twice instead of showing it once and leaving the
  other direction implicit.
- Bad, because it is exactly the second unverifiable branch ADR-0017's confidence note warned about.
  Deleting it changes no test and no rendered output; only inspection and this record say it is
  deliberate.
- Bad, because unlike `Below`'s skip, this one avoids no allocation. Its only justification is
  symmetry with the model's prose, not a measured cost.
- Neutral, because a test now exercises the branch — confirmed by making it panic on purpose first —
  closing a coverage gap that existed the moment it was written. That test would still pass if the
  branch were deleted, since the fallback computes the same value; coverage and verifiability are
  different things, and only the first changed here.

### Confirmation

By a test (`above_with_a_decided_stamp_wins_outright`) that stamps a fully decided cell `Above` onto
an already-defined target and asserts the result equals the stamp. Its reachability was confirmed
the same way ADR-0017's branch would have to be: by making the branch panic on purpose, running the
suite, and observing the panic, before writing the real assertion.

Nothing confirms that the branch is _necessary_ — same as `Below`'s. No test can fail for either
one's removal, and the quality gate has no opinion about it.

## Pros and Cons of the Options

### A — Add the `Above` branch

- Good, because the code stops implying `Below` is special when the underlying rule is not.
- Bad, because it is a second branch of the exact kind ADR-0017 flagged as a habit rather than a
  cost.

### B — Leave it asymmetric

- Good, because it keeps the branch count at the one ADR-0017 already accepted, and adds nothing
  unverifiable.
- Bad, because the asymmetry then has no reason behind it beyond history — `Below` needed a clone
  avoided, `Above` never asked for anything, and a reader is left to guess which of those is true.

### C — Move the skip inside `merge`

- Good, because both directions would get it from one place, not two call-site branches.
- Bad, because it was tried and rejected in the same conversation that produced this record: `merge`
  would need both arguments borrowed to check before cloning, which makes `Above` clone a base
  stroke it does not clone today. The saving on one side would cost the other side the saving it
  already has.

## Reversibility

Cheap, and cheaper than ADR-0017's own branch: deleting the guard and its comment removes it
entirely, and the one test written for it still passes against the fallback path, so nothing else
has to change.

## Confidence

Medium (55%) — slightly under ADR-0017's 60% for the same branch shape, because that record's own
terms for lowering it are exactly what happened here: a second branch, for the same kind of reason.

What would raise it: a second consumer of `merge` that relies on the symmetry to stay correct, which
would make the shortcut load-bearing rather than decorative.

What would lower it further: a third such branch. ADR-0017 already named two as a habit; a third
would be the pattern repeating rather than completing.

## More Information

- [ADR-0017](0017-ask-the-cell-whether-it-is-decided.md), whose confidence note this record answers.
- [ADR-0008](0008-compose-overlapping-cells-with-three-state-arms.md), for the topmost-figure rule
  this makes symmetric.
- The sentence this rests on: [`docs/model.md`](../model.md), under _Stamping_.
