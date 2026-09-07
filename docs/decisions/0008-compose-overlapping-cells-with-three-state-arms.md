---
status: accepted
date: 2026-09-07
decision-makers: Andrés Moschini
---

# Compose overlapping cells with three-state arms and two stamp modes

## Context and Problem Statement

Rendering a diagram means several figures writing into the same grid, and sooner or later two of
them land on the same position. A line crossing a box border is the ordinary case, not the exotic
one. Whatever a cell holds has to answer what happens on the second write.

Holding the finished character cannot answer it. `─` written over `│` has to become `┼`, and there
is no way to get there from the two characters alone without a merge table that grows with the
square of the alphabet — and that still fails the moment the two strokes differ in weight. The cell
has to hold what the figures meant, not what they looked like, and the character has to be derived
at the end.

That much was settled early. What was not settled is the harder half: when the second figure writes,
which of the two decides each side of the cell? Answering "the last one" makes a crossing
impossible, because the horizontal line would erase the vertical one's connections. Answering "the
first one" makes it impossible for a figure to sit on top of another. Both answers are needed,
sometimes in the same cell, and neither figure knows the other exists.

## Decision Drivers

- A crossing has to resolve without either figure knowing about the other. Figures are produced
  independently by whatever layer sits above, and `monospace-core` must not require them to
  coordinate ([`docs/brief.md`](../brief.md) §3: no assumptions from the consuming layer leaking
  into the core).
- The order in which figures are drawn belongs to the caller. Drawing front to back allows skipping
  cells that are already decided; drawing back to front does not. The model should not force one.
- A filled shape has to be able to refuse a connection into its interior. Otherwise anything drawn
  later punches a hole through it.
- Cheap to explain. The rule is going to be quoted in every later spec that stamps anything.

## Considered Options

- **A** — Store the glyph, and merge on write with a character-to-character table.
- **B** — Store four connections, each either absent or carrying a stroke name. Last writer wins.
- **C** — Store four arms in three states — set, closed, undecided — plus a base stroke for the
  cell, and let each write declare whether it goes above or below what is already there.
- **D** — Store a stack of layers per cell and resolve the whole stack at render time.

## Decision Outcome

Chosen option: **C, three-state arms and two stamp modes**, because the third state is the only
thing that lets a figure say "this side is not mine to decide", which is what makes two figures
compose without either one knowing the other exists.

A cell holds a base stroke and four arms. Each arm is `Set(stroke)`, `Closed`, or `Unset`. Writing a
cell is a single operation, `stamp(x, y, cell, mode)`, where the mode is `Above` — overwrite
everything except the arms the stamp itself leaves `Unset` — or `Below` — touch only the arms the
target still has `Unset`, and leave its base stroke alone.

The three states are not three shades of the same thing. `Closed` is a decision: no stroke goes this
way. `Unset` is an abstention. At render time they produce the same character, and that is exactly
why the distinction costs nothing to draw and buys everything while stamping.

### Consequences

- Good, because a crossing resolves itself. A horizontal segment abstains on its top and bottom
  sides, so whatever crosses it later fills them in, and neither figure had to be told about the
  other.
- Good, because the two drawing orders are equivalent: stamping front to back with `Below` produces
  the same buffer as stamping back to front with `Above`. The caller picks the order it prefers, and
  front to back can stop early on cells that are already fully decided.
- Good, because a filled shape closes its interior sides deliberately, and nothing stamped
  afterwards can connect into the fill.
- Bad, because a cell is no longer a character. It carries five stroke references, and the character
  is a lookup away — cost paid on every rendered cell.
- Bad, because a decided arm cannot be undecided. There is no erase, and no way to insert a figure
  between two that were already stamped. Three overlapping figures cannot be reordered after the
  fact; they have to be stamped again into a fresh buffer.
- Neutral, because neighboring cells are never reconciled. A cell may offer a connection to the
  right while its neighbor offers nothing to the left, and the result is a stroke that stops against
  a space. That is not a defect to fix: open borders need it, and arrowheads will need it.

### Confirmation

Enforced by review until the first spec implements it, and by tests from then on. Three of them
carry the decision:

- Stamping a list of figures front to back with `Below` and back to front with `Above` produces
  identical buffers. If this ever fails, one of the two modes has drifted.
- `Below` on a cell whose four arms are decided changes nothing.
- A segment stamped across a border produces a junction character, with neither figure referring to
  the other.

## Pros and Cons of the Options

### A — Merge glyphs on write

- Good, because the cell stays one character wide and the buffer is trivially printable.
- Bad, because the merge table is quadratic in the alphabet and undefined for most pairs. `┤` merged
  with `━` has no principled answer.
- Bad, because information is destroyed on every write. By the time a third figure arrives, the fact
  that the first stroke was light and the second heavy is gone.

### B — Four connections, last writer wins

- Good, because it is the smallest thing that can express a crossing at all.
- Bad, because it cannot express abstention. A figure that does not care about its top side has to
  choose between claiming it — which erases whatever crossed it — and leaving it absent, which is
  itself a claim that renders as a broken line.
- Bad, because it forces one drawing order on the caller, and the order that composes correctly is
  the one that cannot skip work.

### C — Three-state arms and two modes

- Good, because abstention is expressible, which is what makes independent figures compose.
- Good, because the two modes are exact mirrors, so the caller can trade order for performance.
- Bad, because there is one more state to reason about in every rule that touches a cell.

### D — A stack of layers per cell

- Good, because it is the only option where a figure can be inserted, removed or reordered after the
  fact.
- Bad, because the cost is unbounded: memory grows with the number of overlapping figures, and the
  render has to walk the whole stack per cell.
- Bad, because it moves the composition rules into the renderer, where they would have to run again
  on every render rather than once per stamp.

## Reversibility

Cheap today: nothing is implemented, and the whole model is three files of prose. It stops being
cheap once figures in the layer above are written against the arm semantics, because "this side is
not mine" is a habit that spreads through every figure that draws anything.

Option D remains reachable later without discarding this one — a stack of layers would hold cells of
exactly this shape — so the expensive part of the decision is the arm semantics, not the buffer.

## Confidence

High (85%).

What would change it: evidence that real diagrams need to insert a figure between two already
stamped, or to remove one. Two modes cannot do that, and the answer would be a layered buffer
holding these same cells. Nothing else about the model would have to move.

What would prove it wrong later: a figure that has to consult what is already in a cell before
deciding what to stamp. That would mean composition is not expressible as a per-cell merge, and the
whole approach would need revisiting.

## More Information

- The model this decision shapes, in full: [`docs/model.md`](../model.md).
- The related decision on what to draw when no character matches a combination:
  [ADR-0009](0009-degrade-a-cell-to-its-base-stroke.md).
