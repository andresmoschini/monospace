---
status: "accepted"
date: 2026-09-14
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Record a cell's owner beside the buffer, and let the first writer keep it

## Context and Problem Statement

Issue #62 asks that each cell record which shape wrote it, so that an editor can take a position on
the screen and resolve it back to the shape occupying it. That is the operation behind clicking on a
figure, and without it an editor has a picture and no way back into the model.

Two questions have to be answered together. Where the record lives — in the core, alongside the
cells, or in the diagram layer beside the buffer — and what it says when several shapes wrote the
same position, which is not a corner case but the normal outcome of two figures meeting: the model's
whole composition rule exists so that a crossing produces one junction cell written by both lines.

[ADR-0031](0031-a-shape-draws-into-a-surface.md) is what makes the second question tractable. A
shape draws into a `Surface` — one write operation and no reader — and the diagram is what hands it
one. Whatever watches the writes can sit there.

## Decision Drivers

- [Principle VII](../../.specify/memory/constitution.md#vii-the-core-stays-portable): a shape's
  identity is a diagram concept, and the core knows nothing about diagrams. Anything added to `Cell`
  or `Buffer` for this is a promise inherited by the WebAssembly boundary and by every later phase.
- [ADR-0042](0042-draw-a-diagram-front-to-back-into-a-given-window.md) draws from the front of the
  order to the back, which makes "the first shape to write here" mean "the front-most shape that
  reached here".
- The consumer is an editor resolving a click. One answer is what a click needs; a list is what a
  click would have to choose from anyway.
- The maintainer's decision, taken with the alternatives in front of them: the front-most shape owns
  the position.

## Considered Options

- **A** — The core records it: `Cell` or `Buffer` carries an owner, an opaque token the core stores
  and never interprets.
- **B** — The diagram records it: each shape draws through a `Surface` of the diagram's own that
  notes the positions that shape stamped, building a map beside the buffer. The first writer keeps
  the position.
- **C** — B, but keeping every writer per position, in order from front to back.

## Decision Outcome

Chosen option: **B**, because `Surface` already is the place every write passes through, so the
record costs the core nothing and the diagram everything it needs.

Drawing a diagram therefore produces two things: the buffer, as before, and a map from position to
the identity of the shape that wrote there first. Since ADR-0042 draws from the front of the order
backwards, the first writer is the front-most shape that reached the position, which is the one a
click should select.

The map holds only positions inside the window the drawing was given. The buffer ignores a stamp
that falls outside its window, and a map that named a position the render has no character for would
be answering about something nobody can see.

One thing has to be said plainly, because the record is easy to over-read. **The map says who
reached a position first, not whose glyph it is.** A shape writes a cell whether or not the write
changes anything: a `Below` stamp onto an already decided cell leaves the buffer untouched, and the
surface cannot tell, because it has no reader. Drawing front to back keeps that harmless for the
question being asked — a later writer is behind the first by construction — but the two do come
apart in one case worth knowing: where the front-most shape left a side `Unset` and a shape behind
it decided that arm, the glyph is partly the second shape's and the position belongs to the first.

### Consequences

- Good, because the core gains nothing. `Cell`, `Buffer` and `Surface` stay exactly as they are, and
  the whole feature is a `Surface` implementation in the crate that has the identities.
- Good, because it composes rather than replaces. The recorder wraps the layer the diagram was
  already drawing through, which is what ADR-0031's one-method trait was for; the counting surface
  the core uses in its own tests is the same move made for a different question.
- Good, because the answer to a click is one shape, decided by the order the user can see and
  change, rather than a list the editor has to disambiguate with a rule of its own.
- Bad, because information is discarded. A junction cell belongs to one of the two lines that made
  it, and the other is not recorded anywhere, so "what is at this position" and "what contributed to
  this glyph" stop being the same question and only the first can be asked.
- Bad, because the record counts writes rather than effects, as described above. A shape that stamps
  a position and changes nothing there still claims it if it is the front-most to arrive.
- Bad, because the map is only as current as the last drawing. It is rebuilt whenever the diagram
  draws, which is the model issue #62 asks for — the buffer is rebuilt from the diagram — but it
  does mean a mutation invalidates the map until the next drawing.
- Neutral, because option C stays available. Keeping every writer is a change to what the recorder
  stores and to what the query returns, with nothing else affected.

### Confirmation

By test, and the interesting cases are small: two overlapping boxes, where every position of the
overlap resolves to the front one; two crossing lines, where the junction resolves to whichever is
in front; and a position no shape wrote, which resolves to nothing. Changing the order and drawing
again must change the answers, which is what proves the map follows the order rather than the list.

Mechanically, the core is confirmed untouched by the diff: an owner appearing in `monospace-core` is
this decision being abandoned.

## Pros and Cons of the Options

### A — The core records it

- Good, because the owner sits with the cell it describes, and any consumer of a buffer gets it for
  free.
- Good, because the core could then answer "what is at this position" without a second structure to
  keep in step.
- Bad, because it puts a diagram concept into the crate that is supposed to know only about drawing,
  and the token would be meaningless to every other consumer of a buffer.
- Bad, because it would have to travel through `Surface`, which would stop being one write operation
  and start carrying who is writing — the exact thing ADR-0031 removed.

### B — The diagram records it, first writer wins

- Good, because the core is untouched and the mechanism already exists.
- Good, because it answers the editor's question directly.
- Bad, because it forgets every writer but the first.

### C — The diagram records every writer

- Good, because nothing is discarded, and a click is answered by taking the first of the list.
- Good, because "what contributed to this glyph" becomes a question anyone can ask, which is a real
  one for a junction cell.
- Bad, because it is more state per position for a consumer that does not exist, and the query would
  return a list that every caller today would immediately take the first element of.

## Reversibility

Cheap in every direction that matters.

Moving to C is a change to the recorder and to one query's return type; nothing about drawing
changes. Moving to A is the expensive one, because it widens the core's public API, and it is
expensive precisely in the way principle VII warns about.

Nothing here is permanent: the map is produced by a drawing and discarded with it, so no stored
diagram depends on the choice.

## Confidence

High (85%).

What would change it: an editor that needs to know everything that contributed to a cell — selecting
a junction and being offered both lines, say. That is option C, and it is a small change.

What would prove it wrong: the distinction between reaching a position and deciding its glyph
turning out to confuse users of the editor. The failure would look like clicking on what appears to
be a line and selecting the box behind it, and it would be observed rather than argued.

## More Information

- [ADR-0031](0031-a-shape-draws-into-a-surface.md) — the trait this record hangs from, and the
  reason a shape cannot know what it draws over.
- [ADR-0042](0042-draw-a-diagram-front-to-back-into-a-given-window.md) — the drawing order that
  makes the first writer the front-most shape.
- [The model](../model.md), _The two orders are equivalent_ — where the base stroke and each arm end
  up owned by the topmost figure that decided them, which is the composition-level statement this
  record approximates at the level of whole shapes.
