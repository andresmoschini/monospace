---
status: "accepted"
date: 2026-09-14
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Draw a diagram front to back, into a window the caller gives

## Context and Problem Statement

Issue #62 makes the diagram responsible for drawing itself: it is the source of truth, and the
buffer is rebuilt from it rather than edited. It also says that the order the diagram holds its
shapes in decides which are drawn in front of which.

Two things have to be settled before that operation can exist, and both are about the one call.

The first is the window. A buffer is a window with an origin and a size, and stamping outside it
does nothing. Somebody has to choose that window, and the question hides two that have different
owners: how much the diagram occupies, and which part of it is wanted. The second is the caller's
and nobody else's — a terminal draws what fits at the scroll position it is at, and no measurement
of the content answers that. The first is the diagram's, and
[ADR-0040](0040-let-each-shape-answer-its-own-anchor-points.md) has just made it unable to answer: a
shape may answer no anchor point at all, so the diagram holds nothing it could measure itself from.

The second is the direction. [The model](../model.md), under _The two orders are equivalent_, has
already established that stamping front to back with `Below` produces exactly the same buffer as
stamping back to front with `Above`, and names the first as the path that can stop early because a
fully decided cell can no longer change. Both are available; the diagram has to pick one, and the
pick is visible in more than performance, because
[ADR-0043](0043-let-the-buffer-record-who-decided-each-cell.md) reads the order to decide who owns a
cell.

## Decision Drivers

- Which part of a diagram to draw belongs to the caller by nature. A screen with a scroll position
  is the case, and it stays the case on the day the diagram can measure itself: a viewport is not a
  substitute for a measurement, it is a different question.
- ADR-0040 leaves nothing to measure either, so an auto-sized canvas would additionally have to be
  built on shapes that may decline to say where they are.
- The model already proves the two orders equivalent, so nothing about appearance rides on this
  choice. What rides on it is the early stop, and which shape reaches a position first.
- [ADR-0017](0017-ask-the-cell-whether-it-is-decided.md) put the decided-cell skip in `stamp` for a
  layer that did not exist yet, and said so: "the predicate is public for the sake of a layer that
  does not exist yet". This is that layer, and front to back is what cashes it.
- The maintainer's instruction, taken with the alternatives in front of them: the caller gives the
  window, and the front is drawn first with `Below`.

## Considered Options

For the window:

- **A** — The caller gives an origin and a size, and the diagram draws inside it.
- **B** — The diagram computes the window from its shapes.
- **C** — Optional: the caller's window if given, a computed one otherwise.

For the direction:

- **D** — Front to back with `Below`.
- **E** — Back to front with `Above`.

## Decision Outcome

Chosen: **A** and **D**. The diagram draws into a window the caller supplies, visiting its shapes
from the front of the order to the back, stamping every cell with `Below`.

What the order means follows from that, and is worth stating plainly because "in front" invites a
reading the model does not support. **The order is composition, not occlusion.** A shape in front
does not hide what is behind it; it decides first. Its base stroke wins, and each of its arms wins
on the sides it decided, while the sides it left `Unset` fall through to whatever is behind — which
is how two lines crossing produce a junction instead of one interrupting the other. Where a figure
does need to hide what it covers, it does so by writing literal glyphs, which is what a box's fill
already is. That is the whole of the opacity this phase has, and it is enough:
[ADR-0008](0008-compose-overlapping-cells-with-three-state-arms.md) owns the composition rule and
nothing here touches it.

### Consequences

- Good, because the diagram needs nothing measurable from its shapes, so ADR-0040's permission to
  answer nothing costs nothing here.
- Good, because a caller that already knows its canvas — a terminal of a known size, a test with an
  expected picture, an editor scrolled somewhere — says so once instead of discovering what the
  diagram decided and cropping the result afterwards.
- Good, because front to back is the path the model names as the one that can stop early, and
  ADR-0017's predicate finally has the caller it was made public for. No claim is made here about
  how much that saves; there is still no workload measured, and ADR-0017 said the same.
- Good, because the front-most shape is the first to decide a cell, which is exactly the owner
  ADR-0043 records. Under option E it is the last to paint over one, so the record would be
  overwritten as the drawing goes rather than written once.
- Bad, because a caller that wants the whole diagram has nothing to base its window on. Today that
  caller is the command-line application, which will hard-code one. Nothing measures the drawing to
  check the window was big enough, so a figure that falls outside is silently clipped — which is
  what the buffer has always done with a stamp outside its window.
- Bad, because the missing piece is easy to misname. It is not that the window should be the
  diagram's; it is that the diagram cannot yet be asked what it occupies, which is the question a
  caller with no viewport of its own wants answered before choosing one. That gap is the price of
  ADR-0040, and it is an addition rather than a reversal.
- Neutral, because option B stays available. The day enough kinds can be measured, an auto-sized
  window is a second entry point that computes a window and calls this one.

### Confirmation

By test. The equivalence of the two orders is already listed in the model under _Properties worth
testing_, so a diagram drawn front to back can be compared against the same shapes stamped back to
front with `Above` and must match. Clipping is confirmed by a shape placed partly outside the given
window: what falls inside is drawn, and nothing else appears.

## Pros and Cons of the Options

### A — The caller gives the window

- Good, because it asks nothing of the shapes, and works for every kind including the ones that
  answer no anchor point.
- Good, because it is the smallest possible API, and the buffer's existing behavior outside its
  window is the whole of the clipping rule.
- Good, because it is what a caller with a viewport would ask for anyway, so the argument for it
  survives ADR-0040 being revisited.
- Bad, because a caller that wants everything has to guess, and a bad guess is silent.

### B — The diagram computes the window

- Good, because "render this diagram" would need no second argument, which is what an exporter
  wants.
- Bad, because it requires every kind to be measurable, and ADR-0040 has just decided the opposite.
  A shape that answers nothing would either be excluded from the measurement or force a rectangle it
  cannot give.
- Bad, because it does not replace the window. A caller showing part of a diagram still has to say
  which part, so B is a second entry point rather than a different answer to this question.

### C — Optional window

- Good, because it covers both callers.
- Bad, because "how big is a diagram" would have two answers, and the automatic one would quietly
  clip a shape that could not measure itself. Two answers to one question is the duplication the
  constitution treats as a defect.

### D — Front to back with `Below`

- Good, because it can stop early, and because the front-most shape is the first to claim a cell.
- Bad, because it reads backwards: the list is drawn from its front, which is the opposite of the
  painter's order most people expect.

### E — Back to front with `Above`

- Good, because it is the familiar painter's algorithm, and each shape simply overwrites.
- Bad, because it rewrites every cell once per figure, and because the ownership record would be
  overwritten in the same way — the last shape to decide a cell would have to win, which is the same
  answer arrived at by the more expensive route.

## Reversibility

Both parts are cheap to revisit and neither is visible in the output.

Switching direction changes no picture, by the equivalence the model proves, so it is an internal
change plus whatever ADR-0043's ownership rule says. Adding a computed window is additive, as noted
above. What is not cheap is removing the window parameter once callers pass one, which is why the
optional form was rejected rather than postponed: adding an overload later is easier than taking an
argument away.

## Confidence

High (85%) on the direction, medium-high (75%) on the window.

What would change the window decision: an exporter, or any consumer that does not know its canvas in
advance. That is a real consumer rather than a hypothetical one, and it arrives with phase 3 — and
what it changes is that a diagram gains a way to say what it occupies, not that the window stops
being the caller's.

What would prove the direction wrong: nothing about appearance, since the orders are equivalent. It
would be proved wrong only if the early stop turned out to cost more than it saves, which needs a
workload nobody has yet.

## More Information

- [The model](../model.md), _The two orders are equivalent_ — the property this decision rests on.
- [ADR-0008](0008-compose-overlapping-cells-with-three-state-arms.md) — the composition rule that
  makes "in front" mean deciding first rather than hiding.
- [ADR-0017](0017-ask-the-cell-whether-it-is-decided.md) — the decided-cell skip, and the layer it
  was waiting for.
- [ADR-0043](0043-let-the-buffer-record-who-decided-each-cell.md) — what the drawing order decides
  about ownership.
