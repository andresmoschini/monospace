---
status: "accepted"
date: 2026-09-14
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Let the buffer record who decided each cell

## Context and Problem Statement

Issue #62 asks that each cell record which shape wrote it, so that an editor can take a position on
the screen and resolve it back to the shape occupying it. That is the operation behind clicking on a
figure, and without it an editor has a picture and no way back into the model.

Two questions have to be answered together. Where the record lives — in the core, with the cells, or
in the diagram layer beside the buffer — and what it says when several shapes wrote the same
position, which is not a corner case but the normal outcome of two figures meeting: the model's
whole composition rule exists so that a crossing produces one junction cell written by both lines.

The second question is what decides the first. "Which shape wrote here" and "which shape's glyph is
this" are different questions, and only one of the two homes can tell them apart. A shape draws into
a `Surface` — one write operation and no reader, by [ADR-0031](0031-a-shape-draws-into-a-surface.md)
— so anything watching from outside sees stamps go past and cannot see what they did. The buffer is
where a stamp is merged, and it is the only place that knows whether a stamp decided anything.

## Decision Drivers

- The consumer is an editor resolving a click. One answer is what a click needs; a list is what a
  click would have to choose from anyway.
- [ADR-0042](0042-draw-a-diagram-front-to-back-into-a-given-window.md) draws from the front of the
  order to the back, and a `Below` stamp decides only what is still `Unset`. The front-most shape is
  therefore both the first to arrive and the one whose strokes win.
- The record should describe the picture rather than the visit. A shape that stamps a position and
  changes nothing there has put nothing on the screen, and an editor selecting it would be selecting
  something invisible.
- [Principle VII](../../.specify/memory/constitution.md#vii-the-core-stays-portable): the core knows
  nothing about diagrams, and anything added to it is a promise inherited by the WebAssembly
  boundary and by every later phase. A token the core stores and never interprets is the narrowest
  form the addition can take.
- The maintainer's decision, taken with the alternatives in front of them: the core records it, the
  cell belongs to the shape at the front, and a list of owners is what the record grows into later.

## Considered Options

- **A** — The core records it: the buffer carries an owner per position, an opaque token it stores
  and never interprets.
- **B** — The diagram records it: each shape draws through a `Surface` of the diagram's own that
  notes the positions that shape stamped, building a map beside the buffer.
- **C** — Either home, keeping every writer per position instead of one.

## Decision Outcome

Chosen option: **A**, with one owner per position for now.

The buffer carries, beside its cells, the identity of the shape each position belongs to. That
identity is an opaque token: the core stores it, hands it back, and never interprets it, orders it
or learns what a shape is; the diagram holds the mapping from a token to a `ShapeId`. Drawing
therefore produces one thing rather than two, and "what is at this position" is a question the
buffer answers.

Beside the cells rather than inside `Cell`. A cell is a value that is built, compared and merged
freely, and an owner is not part of what a cell is — two cells that look the same are the same cell
whoever wrote them. Keeping the record out of that value is also what keeps the list, when it
arrives, out of something the tests copy everywhere.

**The cell belongs to the front-most shape that decided it.** Ownership follows the order rather
than the sequence of arrivals, and the two coincide only because the drawing runs front to back. A
stamp that changes nothing takes nothing: a `Below` stamp onto an already decided cell leaves the
buffer untouched and leaves the owner untouched with it. This is the part only the core can do,
because comparing what a stamp produced against what was already there is not available to a writer
that has no reader.

**`Surface` is not what this travels through.** ADR-0031 made a surface one write operation with no
reader, and nothing here adds to it. The writer does not change in the middle of a shape's drawing,
so the buffer is told whose stamps these are before that shape draws, and a shape still sees exactly
what it saw. Whether that is a scoped wrapper implementing `Surface` or a setting on the buffer
belongs to the spec that implements it; neither changes the trait.

Two things follow, and the second is a limit worth stating.

**A position with no owner is a position nothing decided.** A position no shape wrote and a position
whose every arm is still `Unset` both answer nothing.
[ADR-0041](0041-resolve-a-position-through-a-reference.md) agrees from the other side: a shape whose
position does not resolve is not drawn, so it decides nothing and owns nothing.

**A cell is owned whole, and a junction is where that shows.** Where the front-most shape left a
side `Unset` and a shape behind it decided that arm, the glyph is made by both and the cell belongs
to the front one. When an editor needs the rest, the answer is a list of owners per position,
front-most first — not an owner per arm. What a person clicks on is a character, and the arms inside
one character cannot be pointed at separately, so a list is the shape the answer has and an
arm-level record would be a precision nobody could use.

### Consequences

- Good, because the record sits with the data it describes. There is no second structure to keep in
  step with a buffer that is rebuilt from the diagram anyway, and resolving a position is one
  lookup.
- Good, because the record can be about the glyph rather than about the visit. Only the buffer sees
  a stamp merge, so only the buffer can decline to give a position away to a shape that changed
  nothing there.
- Good, because growing one owner into a list is a change inside the buffer and to one return type.
  Drawing is unaffected, and nothing stored depends on which of the two shapes the record has.
- Good, because the addition is narrow. `Cell`, `Surface` and the composition rule are untouched,
  and an opaque token adds no concept to the core's vocabulary.
- Bad, because the core's public API grows an item nothing in the core uses, which is the widening
  principle VII warns about. That the token is opaque is a defense, not a denial.
- Bad, because every buffer carries the record whether anybody asked for it or not, so a caller
  drawing shapes that have no identities pays for a column it never reads.
- Bad, because information is still discarded while one owner is kept. A junction belongs to one of
  the two lines that made it and the other is recorded nowhere, so "what contributed to this glyph"
  cannot be asked until the list arrives.
- Neutral, because the record is only as current as the last drawing. It is rebuilt whenever the
  diagram draws, which is the model issue #62 asks for, and a mutation invalidates it until then.

### Confirmation

By test, and the interesting cases are small: two overlapping boxes, where every position of the
overlap resolves to the front one; two crossing lines, where the junction resolves to whichever is
in front; a position no shape wrote, which resolves to nothing; and a shape stamped onto a cell
already decided, which does not take it. Changing the order and drawing again must change the
answers, which is what proves the record follows the order rather than the list.

## Pros and Cons of the Options

### A — The core records it

- Good, because the owner sits with the cell it describes, and every consumer of a buffer can ask.
- Good, because the buffer is the only place that can tell a stamp that decided something from one
  that changed nothing, so the record can be about the picture.
- Good, because it is small: a token per position, and a way to say who is drawing.
- Bad, because a concept no other consumer of a buffer uses is now in the core's public API.

### B — The diagram records it, outside the buffer

- Good, because the core is untouched, which is the strict reading of principle VII.
- Good, because it composes: the recorder wraps the layer the diagram already draws through, which
  is what ADR-0031's one-method trait was for.
- Bad, because it can only count arrivals. A surface has no reader, so a recorder wrapped around one
  watches a stamp go past without learning what it did, and the record answers who reached a
  position rather than whose glyph is there.
- Bad, because it is a second structure to keep in step with a buffer that is rebuilt anyway, and
  every consumer of the picture has to be handed both halves.

### C — Every writer, not one

- Good, because nothing is discarded, and a click is answered by taking the first of the list.
- Good, because "what contributed to this glyph" becomes a question anyone can ask, which is a real
  one for a junction cell.
- Bad, because it is more state per position for a consumer that does not exist, and every caller
  today would take the first element and drop the rest.
- Neutral, because it is where this record expects to go. Arriving there costs a change of type
  rather than a change of design.

## Reversibility

Cheap toward the list, and a reversal rather than a move in the other direction.

Growing one owner into a list, front-most first, changes what the buffer stores and what one query
returns; drawing is unaffected and nothing persisted depends on it. Going back to option B would
mean giving up the effect-based answer, since a recorder outside the buffer cannot see what a stamp
did — so it is not a relocation of this decision but a reversal of the reason for it.

Nothing here is permanent in the diagram's sense: the record is produced by a drawing and discarded
with it, so no stored diagram depends on the choice.

## Confidence

High (80%).

What would change it: an editor that needs everything contributing to a cell — selecting a junction
and being offered both lines. That is the list, and it is a small change made on top of this one.

What would prove it wrong: the token turning out to need to be more than opaque. If the core has to
compare tokens for anything but equality, order them, or look something up by one, then a diagram
concept has crossed the boundary after all and B was the honest home.

What would not change it: the memory a token costs at every position. It was weighed and accepted,
and measuring it comes before optimizing it — there is no workload to measure against yet.

## More Information

- [ADR-0031](0031-a-shape-draws-into-a-surface.md) — the one-write-no-reader trait this record
  leaves alone, and the reason a recorder outside the buffer can only count arrivals.
- [ADR-0042](0042-draw-a-diagram-front-to-back-into-a-given-window.md) — the drawing order that
  makes the front-most shape the first to decide a cell.
- [ADR-0041](0041-resolve-a-position-through-a-reference.md) — why a shape that is not drawn owns
  nothing.
- [The model](../model.md), _The two orders are equivalent_ — where the base stroke and each arm end
  up owned by the topmost figure that decided them, which is the composition-level statement this
  record follows at the level of a whole cell.
