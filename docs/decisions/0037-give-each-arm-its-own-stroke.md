---
status: accepted
date: 2026-09-12
decision-makers: Andrés Moschini
---

# Give each arm its own stroke, ending the one-stroke-per-cell restriction

## Context and Problem Statement

[ADR-0012](0012-one-stroke-per-cell.md) dropped the per-arm stroke the model describes and gave a
cell a single stroke, its base, in which every `Set` arm draws. It was explicit that this was a
restriction rather than a different rule, and explicit about when it would end: `Arm::Set` "gains a
payload later", once there is something for a mixed key to match. That condition has arrived.
[ADR-0009](0009-degrade-a-cell-to-its-base-stroke.md) settled what happens when no character exists,
and `docs/glyph-sets.md` carries the four tables that hold the characters that do — Light with
Double (18 rows), Light with Heavy (50), Light Round with Double (18), Light Round with Heavy (50).

Under the restriction those tables are unreachable. `StrokeCell::key` builds every `Set` side from
the cell's base stroke, so the key a cell can produce never names two strokes, and a rule keyed on
two strokes can never be the one that answers it. Shipping the tables changes nothing on its own —
that is what the reverted first spec of issue #60 demonstrated. The visible consequence is in the
command-line demonstration today: where a light box crosses a double one it prints `╬`, the
character the double box would draw alone, and the light box's stroke is simply lost.

So the decision is not whether cells may mix strokes — the model says they may, and
[ADR-0008](0008-compose-overlapping-cells-with-three-state-arms.md) chose three-state arms with
`Set(stroke)` in its outcome. The decision is how a cell says which stroke runs to a given side now
that the answer can differ per side, and what that costs.

## Decision Drivers

- The model, under _Rendering_ and _Worked examples_, already specifies the mixed key and the two
  lookups. What a slice can express should be a restriction of the model, never a variation on it —
  the driver ADR-0012 chose under, applied in the other direction now that the restriction has
  outlived its reason.
- Two cells that render the same character should compare equal. A representation with more than one
  way to say "a light stroke runs to the top" makes `PartialEq` disagree with the render, and every
  test that compares buffers inherits the disagreement.
- The base stroke must keep exactly the one job
  [ADR-0009](0009-degrade-a-cell-to-its-base-stroke.md) gives it: the stroke every connected arm
  moves to when the exact key has no character.
- Nothing about stamping may change. The three states and the two modes in ADR-0008 are what make
  two figures compose without either knowing the other exists, and this decision must leave them
  untouched.

## Considered Options

- **A** — `Arm::Set(Stroke)`: every connected arm names its own stroke, and the base stroke is read
  only when degrading.
- **B** — `Arm::Set` keeps meaning "the cell's base stroke" and a second variant, `Arm::SetWith`,
  carries a stroke that differs from it.
- **C** — Leave `Arm` as it is and put four optional per-side stroke overrides alongside the four
  arms in `StrokeCell`.

## Decision Outcome

Chosen option: **A, `Arm::Set(Stroke)`**, because it is the form the model already describes, so
there is one way to express a connected side and the key falls straight out of the four arms.

`StrokeCell::key` stops consulting `base` and reads each arm's own stroke; `base` is then read in
exactly one place, the degradation step, which is the role ADR-0009 wrote for it. A cell whose arms
all carry its base stroke renders exactly as it does today, so every diagram that mixes no strokes
is unaffected, and every one that does becomes able to find a mixing rule when a catalog holds one.

### Consequences

- Good, because a mixed key becomes representable, which is the whole of what stands between the
  four mixing tables and the junctions they describe.
- Good, because the base stroke is now read for one purpose only, and a test that pins degradation
  pins it — under the restriction, the base stroke was doing two jobs and a mistake in either
  surfaced as the same wrong character.
- Good, because it deletes a divergence rather than adding a concept: `cell.rs` currently carries a
  doc comment explaining that arms have no stroke "per ADR-0012", against a model that says they do.
- Bad, because `Arm` stops being `Copy`. It gains an owned `Stroke`, every construction site in the
  core, the command-line crate and the tests changes, and the arms of one cell are cloned where they
  were copied.
- Bad, because this cannot be a `refactor` commit under
  [principle V](../../.specify/memory/constitution.md): the tests that construct arms all change,
  and a structural commit may not modify a test. ADR-0012 recorded this cost in advance and it is
  being paid as recorded.
- Bad, because a cell that mixes nothing now stores its stroke's name five times over. `Stroke` is a
  `String` ([ADR-0015](0015-represent-a-stroke-as-a-string.md)), so that is four redundant
  allocations per connected cell, accepted here and worth measuring if rendering ever gets slow.
- Neutral, because stamping is untouched. `Above` still overwrites every arm the stamp itself does
  not leave `Unset` and still takes the stamp's base stroke; `Below` still writes only where the
  target is `Unset` and still leaves the target's base stroke alone. What changes is only that an
  arm surviving from the cell underneath keeps the stroke it was written with instead of being
  redrawn in the base stroke of whatever landed on top.

### Confirmation

By rendering: a cell whose arms carry two different strokes must produce the character the matching
mixing table publishes, and a cell whose arms all carry one stroke must produce what it produces
today. The command-line demonstration is the end-to-end confirmation — its light/double and
light/heavy crossings print `╬` and `╋` today and must print the mixed junctions instead, with the
rest of its output unchanged. Nothing in `cargo xtask check` can tell whether arms carry their own
stroke; beyond those tests this is enforced by review.

## Pros and Cons of the Options

### A — `Arm::Set(Stroke)`

- Good, because it is the model's own form, so there is nothing to unlearn and nothing to translate
  when reading `docs/model.md` next to the code.
- Good, because one connected side has exactly one representation, so buffer equality and rendered
  equality agree.
- Bad, because it is the most churn: every construction site, and `Arm` loses `Copy`.

### B — `Arm::Set` plus `Arm::SetWith(Stroke)`

Base stroke by default, override where a side differs from it.

- Good, because `Arm::Set` keeps its meaning and most construction sites — every shape, which has
  one stroke — never change.
- Good, because `Arm` stays `Copy` for the common case.
- Bad, because a light side is then expressible two ways in a cell whose base is light, and the two
  do not compare equal while rendering identically. Every test that compares cells or buffers has to
  know which form was used.
- Bad, because it puts the base stroke back into the key, so a stamp landing above changes what an
  untouched `Set` arm draws — the exact loss this decision exists to end, still present wherever a
  side was written as `Set`.

### C — Four per-side overrides beside the four arms

- Good, because `Arm` is untouched and stays `Copy`.
- Bad, because it makes "connected" and "which stroke" two fields that must agree, so a `Closed`
  side with an override, or a `Set` side without one, are states that mean nothing and nothing
  prevents.
- Bad, because a `StrokeCell` grows from five fields to nine to express what four can.

## Reversibility

Symmetrical with ADR-0012 and the same size: the compiler finds every site, and undoing it is
mechanical but touches the core, the command-line crate and the tests in one step. What grows the
cost is catalogs in the wild keyed on two strokes — the moment a mixing table outside this
repository is in use, reverting stops being a refactor and starts breaking someone's diagram.

## Confidence

High (90%).

Nothing available now would change it: the model has specified this since before the first slice,
ADR-0008 chose it, and ADR-0012 postponed rather than disputed it. What would prove it wrong is the
per-cell cost turning out to matter — five owned strings per connected cell — which would be an
argument for interning strokes, not for putting the restriction back.

## More Information

- [ADR-0012](0012-one-stroke-per-cell.md), superseded by this record; its status becomes
  `superseded by ADR-0037`.
- [ADR-0008](0008-compose-overlapping-cells-with-three-state-arms.md), which chose three-state arms
  and named the payload this record restores.
- [ADR-0009](0009-degrade-a-cell-to-its-base-stroke.md), the degradation rule that gives a per-arm
  stroke something to fall back from.
- [ADR-0036](0036-hold-every-table-but-light-outside-the-core.md), which decides where the four
  mixing tables live once they are shipped.
- `docs/model.md`, under _Rendering_ and _Worked examples_, which needs no change: it has described
  this all along.
