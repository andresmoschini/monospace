# The diagram model

**Status:** Design intent, not observed behavior. Nothing described here is implemented. It is
written down first so that each feature spec can implement a slice of it and say which slice, rather
than restating the whole model or inventing its own vocabulary. Amend it when reality contradicts
it, and say so in the commit.

**Created:** 2026-09-14

This document describes the layer above [the buffer and render model](model.md): what holds a figure
after it has been drawn. [`model.md`](model.md) owns the buffer, the cell, stamping, glyph sets,
rendering and shapes, and it is deliberately not extended here — the two layers are separate crates
([ADR-0038](decisions/0038-hold-the-diagram-model-in-a-crate-above-the-core.md)) and separate
documents for the same reason.

Where that model's every value is constructed, used and dropped, a diagram is kept. It is the source
of truth, and the buffer becomes what it always was underneath: a working surface rebuilt from the
diagram whenever a picture is wanted. This is the answer to the question `model.md` left open under
_Open questions_ — how mutable state for editing is exposed — and the answer is that it is not
exposed by the core at all.

What this document does not describe: how a diagram is read from a file or written to one, how
positions are computed for a caller who does not want to give them, and anything about an
interactive application. Those are layers above this one.

## 1. Vocabulary

| Term        | Meaning                                                                         |
| ----------- | ------------------------------------------------------------------------------- |
| `Diagram`   | Shapes in an order, drawable, changeable; the source of truth                   |
| `Shape`     | One figure in a diagram: an identity, a position, and what kind of figure it is |
| `ShapeId`   | A shape's identity: a string, unique within its diagram                         |
| `Anchor`    | One of four named points a shape may offer: the center of each of its sides     |
| `Position`  | Either an absolute point or a reference                                         |
| `Reference` | A `ShapeId`, an `Anchor` on it, and a horizontal and vertical offset            |
| `Order`     | The sequence the diagram holds its shapes in; its front is drawn first          |
| `Ownership` | What a drawing records at every position: which shape the cell there belongs to |

A diagram's `Shape` and the core's `Shape` share a name and are different things. The core's is a
value that draws and answers nothing about itself; this one is stored, identified, replaced and
reordered, and draws by constructing the core's
([ADR-0039](decisions/0039-a-diagram-shape-is-its-own-entity.md)). Where both appear in one
sentence, the core's is named as the core's.

## 2. The diagram

A diagram holds shapes in an order and nothing else. It does not hold a buffer, a glyph catalog, a
window or a rendered picture: those are given to it when it draws, and it keeps none of them.

The order is a sequence, and its **front** is the end drawn first. A shape nearer the front decides
a cell before one behind it. Nothing about the order is a coordinate: it says which shape decides
first, and that is all it says.

Groups of shapes are out of this model. A shape belongs to a diagram directly, and there is no
nesting.

## 3. Identity

Every shape in a diagram has an identity, and it is a string. The diagram generates one when the
shape is added — `#1`, `#2`, and so on — and it is unique within that diagram.

An identity is what makes a shape findable after it has been placed: it is how a position refers to
another shape, how a change names what it is changing, and what a position on the screen resolves
back to. It survives every change to the shape it names, including replacing it and reordering it.

Identities being chosen by the caller, or edited after the fact, is an open question below.

## 4. Positions

A shape's position is either **absolute** — a point, in the same coordinates the buffer uses — or a
**reference**: the identity of another shape, an anchor point on it, and a horizontal and a vertical
offset. A reference is resolved by asking the referenced shape for that anchor and adding the
offsets, so it is derived from where that shape is now rather than from where it was when the
reference was made. Moving the referenced shape moves everything that hangs from it, which is the
whole point of having references at all.

**Only a connector's endpoint holds a reference, for now**
([ADR-0041](decisions/0041-resolve-a-position-through-a-reference.md)). Every other shape's position
is absolute, and _Attachment_ below is where the reference lives. A reference therefore names a box
or a line, both positioned absolutely, or an arrow, which answers no anchor point: the chain is one
link long, nothing resolves through anything else, and no cycle can be built. Issue #89 is where a
reference widens to any shape's position, and the cycle question belongs to it.

Nothing is the answer in two cases, and the model treats them as one:

| The reference names                        | Resolves to |
| ------------------------------------------ | ----------- |
| a shape the diagram does not hold          | nothing     |
| an anchor point that shape does not answer | nothing     |

A shape whose position does not resolve is **not drawn**. It writes nothing, owns no position, and
answers no anchor point of its own, so a reference to it resolves to nothing in turn. There is no
error and no report: an unresolved reference is a normal state of a diagram being built, not a fault
(ADR-0041). This is _Degenerate arrangements_ from [`model.md`](model.md) applied one layer up. A
way to ask which shapes a diagram could not draw is issue #88.

## 5. Anchor points

A shape may offer four named points: the center of its top side, of its right side, of its bottom
side and of its left side.

An anchor point is an absolute position, computed from the shape's own position when it is asked
for. **A shape answers each of them itself, and may answer none**
([ADR-0040](decisions/0040-let-each-shape-answer-its-own-anchor-points.md)): the four are what can
be asked, not what must exist. Answering nothing is an ordinary answer, and it is what lets a figure
with no honest answer wait for a slice of its own rather than be given a fictional one.

A **box** answers all four, from its position and its size.

A **line** answers as a flat box: it has no thickness, so on a horizontal line the top center and
the bottom center are one point, its middle, while the left center is its left end and the right
center its right end. A vertical line is the same seen sideways.

An **arrow** answers nothing for now. A route with bends has no honest answer to most of these, and
none of them is needed to draw one.

Four, rather than the nine issue #62 lists. What the corners and the center are waiting for is a
consumer, and issue #90 is where they arrive: an anchor exists to hang a connector's endpoint from,
and a side is the only one of the nine an endpoint has an unambiguous relationship with. It is also
what keeps the question in _Attachment_ answerable, since a side has an outside and a corner does
not.

## 6. Attachment

An arrow's endpoint is a position, the direction the arrow leaves it in, and a head — that is
`model.md`'s definition and it is unchanged. What this layer adds is that the position may be a
reference, and an endpoint is the only position in this model that may be one. An endpoint attached
to a shape moves when that shape moves, and an arrow with an endpoint whose reference does not
resolve is not drawn, by the rule in _Positions_: it is a shape whose position does not resolve. An
arrow with two attached endpoints is the figure this model exists to make possible.

The direction the arrow leaves in and the head it carries are not derived from the attachment. They
are the caller's, as before. Deriving a direction from which side of a shape was attached to is an
open question below, and every anchor being a side is what keeps it answerable: a side has an
outside to leave through.

## 7. Drawing

A diagram draws into a window the caller gives it — an origin and a size — together with whatever
else the render needs. The diagram measures nothing and sizes nothing: a shape may answer no anchor
point at all, so there is nothing to measure a canvas from, and what falls outside the window is
clipped exactly as a stamp outside a buffer's window has always been
([ADR-0042](decisions/0042-draw-a-diagram-front-to-back-into-a-given-window.md)).

Shapes are visited from the front of the order to the back, and every cell is stamped with `Below`.
By _The two orders are equivalent_ in [`model.md`](model.md), that produces the same buffer as
visiting them back to front with `Above`, so nothing about the picture depends on the choice.

**The order is composition, not occlusion.** A shape in front does not hide what is behind it; it
decides first. Its base stroke wins, its arms win on the sides it decided, and the sides it left
`Unset` fall through — which is how two crossing lines produce a junction rather than one
interrupting the other. A figure that does hide what it covers does so by writing literal glyphs,
which is what a box's fill already is. That is the whole of the opacity this phase has.

Drawing is repeatable and changes nothing about the diagram. Drawing the same diagram twice into two
equal windows produces two equal buffers.

## 8. Ownership

Drawing records, at every position, which shape the cell there belongs to
([ADR-0043](decisions/0043-let-the-buffer-record-who-decided-each-cell.md)). The record lives in the
buffer, beside its cells, so a drawn diagram is one thing rather than two and asking what is at a
position is asking the buffer. What the buffer keeps is a token it never interprets; the mapping
from that token to a `ShapeId` is the diagram's.

**A cell belongs to the front-most shape that decided it.** A shape that stamps a position and
changes nothing there takes nothing, because ownership is about what is on the screen rather than
about who passed by. A position nothing decided — one no shape wrote, or one whose every arm is
still `Unset` — belongs to nobody, and so does every position outside the window that was drawn
into.

A cell is owned whole. Where the front-most shape left a side `Unset` and a shape behind it decided
that arm, the glyph is made by both and the cell belongs to the front one. The rest is not recorded
yet: when an editor needs it, what it gets is a list of owners for the position, front-most first,
because a character is what a person clicks on and the arms inside one cannot be pointed at
separately.

The record belongs to the drawing that produced it. Changing the diagram does not update it; drawing
again produces a new one.

## 9. Changing a diagram

Five changes, each naming a shape by its identity except the first:

| Change   | What it does                                                             |
| -------- | ------------------------------------------------------------------------ |
| add      | Puts a shape at the front of the order and gives it an identity          |
| remove   | Takes a shape out of the diagram                                         |
| replace  | Puts a different shape under an identity, in the same place in the order |
| forward  | Moves a shape one place toward the front of the order                    |
| backward | Moves a shape one place toward the back of the order                     |

None of them can fail. Naming a shape the diagram does not hold changes nothing, which is the same
answer _Positions_ gives a reference to a shape that is not there.

**A shape is a value, and changing one is replacing it.** Shapes are immutable. A diagram does not
reach into a box and widen it; it takes a box that is wider and puts it where the old one was. What
survives a replacement is what the diagram owns — the identity and the place in the order — and
everything the shape owns is the new shape's: its kind, its parameters and its position.

Moving is a replacement like any other, with a shape positioned somewhere else, and this model says
no more about it than that. What moving an arrow with attached endpoints should do to those
attachments is left to the slice that implements movement, and it is an open question below rather
than a rule here.

Removing a shape leaves every reference to it unresolved, and those shapes stop being drawn. Nothing
is rewritten and nothing cascades: the references stay as they were, and re-adding a shape with the
same identity would make them resolve again.

Forward and backward at the end they are already at do nothing.

## 10. Properties worth testing

- Drawing the same diagram twice into equal windows produces equal buffers.
- Drawing a diagram front to back with `Below` produces the same buffer as stamping the same shapes
  back to front with `Above`.
- Replacing a shape with one positioned elsewhere and drawing again moves everything attached to it
  by the same amount.
- A reference to an absent shape and a reference to an anchor that is not answered produce the same
  thing: the shape that held the reference is absent from the output and the rest of the diagram is
  unchanged.
- A shape's identity survives a replacement and a reorder.
- In an overlap, every position resolves to the front-most shape that decided it, and changing the
  order changes the answer.
- A shape that stamps a position without changing anything there does not take it.
- A position no shape decided resolves to nothing, and so does one outside the window.

## 11. Open questions

Each of these is waiting for an answer, and the answer is prose in this document plus the ADR it
needs. A feature spec that needs one of them answered amends this document first and then implements
the slice.

- **Can a caller choose an identity?** Today the diagram generates them and nothing else can. Issue
  #62 anticipates editable identities without asking for them. What would settle it: the first slice
  where a caller has a name worth keeping — reading a diagram from a file is the obvious one.
- **What does an arrow anchor to?** Arrows answer no anchor point, so nothing can hang off one. What
  would settle it: a figure that has to attach to a connector, most likely a label on it. Answering
  this is also one of the two ways a chain of references becomes longer than one link, which is what
  brings the cycle question back — see _Positions_.
- **How does a shape move?** Moving is a replacement with a shape positioned elsewhere, and nothing
  above that is decided. Whether moving an arrow whose endpoints are attached shifts their offsets,
  leaves them where they are, or is not a move at all is the part left open. What would settle it:
  the first slice that implements movement.
- **Does an attachment decide the direction an arrow leaves in?** Attaching to a box's right side
  and leaving leftward is expressible today and draws something nobody wants. What would settle it:
  the first slice where the caller's direction and the anchor's side are routinely the same.
- **How big is a diagram?** Nothing measures one. The window stays the caller's either way — which
  part of a diagram to draw is a question about a viewport, not about the content
  ([ADR-0042](decisions/0042-draw-a-diagram-front-to-back-into-a-given-window.md)) — so what is
  missing is a way to ask a diagram what it occupies. What would settle it: a consumer that has to
  choose a window with nothing to base it on, an exporter being the obvious one.
- **What does a group of shapes do to this model?** Issue #58 asks for one, and groups are
  deliberately out of scope here. A group contains shapes, which is the first thing in this document
  that would want to speak about shapes generically rather than by kind.
- **Is a diagram serializable, and in what?**
  [ADR-0035](decisions/0035-keep-the-cli-demo-format-out-of-the-model.md) keeps the command-line
  application's file format out of the model, and nothing here reverses that. What would settle it:
  the first consumer that has to save a diagram rather than build one.
