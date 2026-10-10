# The diagram model

**Status:** Design intent, not observed behavior. Nothing described here is implemented. It is
written down first so that each feature spec can implement a slice of it and say which slice, rather
than restating the whole model or inventing its own vocabulary. Amend it when reality contradicts
it, and say so in the commit.

**Created:** 2026-09-14

This document describes the layer above [the buffer and render model](model.md): what holds a figure
after it has been drawn. [`model.md`](model.md) owns the buffer, the cell, stamping, glyph sets,
rendering and shapes, and it is deliberately not extended here — the two layers are separate crates
and separate documents for the same reason. The core is the layer that draws and a diagram is the
layer that remembers, and a crate boundary is the only thing that keeps the second from reaching
into the first: a `Diagram` in the core would be its first long-lived mutable value, in a crate
where every other type is constructed, used and dropped, and a crate cannot reach a private item, so
"no more privileged than any other dependent" is the compiler's to enforce rather than a reviewer's.

Where that model's every value is constructed, used and dropped, a diagram is kept. It is the source
of truth, and the buffer becomes what it always was underneath: a working surface rebuilt from the
diagram whenever a picture is wanted. This is the answer to the question `model.md` left open under
_Open questions_ — how mutable state for editing is exposed — and the answer is that it is not
exposed by the core at all.

What this document does not describe: how a diagram is read from a file or written to one, how
positions are computed for a caller who does not want to give them, and anything about an
interactive application. Those are layers above this one, and two of them now have documents of
their own: [`editing-model.md`](editing-model.md) owns what it means to change a diagram, and
[`monospace-description`](../crates/monospace-description) owns the file format — the latter with no
document beside it, because nothing about it is decided here that is not decided by the JSON. This
one stays the diagram itself.

## 1. Vocabulary

| Term        | Meaning                                                                                                                       |
| ----------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `Diagram`   | Shapes in an order, drawable, changeable; the source of truth                                                                 |
| `Shape`     | One figure in a diagram: its kind, its parameters and its position; its identity and its place in the order are the diagram's |
| `ShapeId`   | A shape's identity: an ordinal, unique within its diagram when the diagram issued it                                          |
| `Anchor`    | One of four named points a shape may offer: the center of each of its sides                                                   |
| `Position`  | Either an absolute point or a reference                                                                                       |
| `Reference` | A `ShapeId`, an `Anchor` on it, and a horizontal and vertical offset                                                          |
| `Delta`     | How far a figure moves along each axis, or how far from it a reference stands: a horizontal and a vertical amount             |
| `Order`     | The sequence the diagram holds its shapes in; its front is drawn first                                                        |
| `Ownership` | What a drawing records at every position: which shape the cell there belongs to                                               |

A diagram's `Shape` and the core's `Shape` share a name and are different things. The core's is a
value that draws and answers nothing about itself; this one is stored, identified, replaced and
reordered, and draws by constructing the core's. The pattern is worth naming: the core answers _what
do I draw_, the diagram answers _what is this and where does it belong_, and a type that tried to
answer both would be answering the second for callers that only asked the first. Where both appear
in one sentence, the core's is named as the core's.

## 2. The diagram

A diagram holds shapes in an order and nothing else. It does not hold a buffer, a window or a
rendered picture. The buffer it writes into is given to it when it draws, and it keeps it no longer
than that drawing; a glyph catalog it is never given at all, because it never renders — see
_Drawing_.

The order is a sequence, and its **front** is the end drawn first. A shape nearer the front decides
a cell before one behind it. Nothing about the order is a coordinate: it says which shape decides
first, and that is all it says.

Two boxes written in this order, where the second is the front and decides the two cells they share
— its own border survives where the first box's does not, and the first box's does not compose with
anything because the second never left those sides open:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 6, "height": 4 } },
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "box", "id": 2, "at": { "x": 2, "y": 1 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" }
  ] }
-->

```text
┌──┐
│ ┌┼─┐
└─┼┘ │
  └──┘
```

<!-- /render -->

Groups of shapes are out of this model. A shape belongs to a diagram directly, and there is no
nesting.

## 3. Identity

Every shape in a diagram has an identity, and it is an ordinal — a number, `1`, `2`, and so on, and
never `0`. A caller may choose the identity a shape is added under, or let the diagram issue one.
The identities **the diagram issues** are unique within that diagram. An identity a caller supplies
is **not checked**: two shapes may carry one, both are held, and the second is a shape no identity
names until the first is removed.

**Zero is not an identity**, and that is the one rule the type itself carries rather than this
document. It is a rule about the value rather than about a diagram: nothing here ever issued a `0`,
and a diagram whose first addition is `1` says so. Who issues an identity and whether two may share
one are the questions above, and they stay here.

An identity is what makes a shape findable after it has been placed: it is how a position refers to
another shape, how a change names what it is changing, and what a position on the screen resolves
back to. It survives every change to the shape it names, including replacing it and reordering it —
and, where the identity is the caller's rather than the diagram's, it survives the shape being
**listed** somewhere else, because an identity does not move when the order it is written in does.

Editing an identity after the fact is an open question below. Handing one to the diagram is a
different matter and is settled: `ShapeId::new` builds an identity from its ordinal, and a change
that puts a shape under one puts it there under that ordinal. **The type itself is the core's** — it
is what a stamp records, and the core cannot hold what only this layer names — while the rules above
stay here, because which identities a diagram issues is not a question the core has.

## 4. Positions

A shape's position is either **absolute** — a point, in the same coordinates the buffer uses — or a
**reference**: the identity of another shape, an anchor point on it, and a horizontal and a vertical
offset. A reference is resolved by asking the referenced shape for that anchor and adding the
offsets, so it is derived from where that shape is now rather than from where it was when the
reference was made. Moving the referenced shape moves everything that hangs from it, which is the
whole point of having references at all.

**The gap from that side may also be written in the side's own words.** An `out` of one cell is one
cell away from the shape, whichever of the four sides names it — `(0, -1)` out of a `top`, `(+1, 0)`
out of a `right`, `(0, +1)` out of a `bottom` and `(-1, 0)` out of a `left`, which is the one axis
the side does not run in. It is **added to** the offset rather than put in its place, so a figure
may write either one or both, and one that writes neither is a reference standing on the border. It
is signed, so a negative `out` is a point inside the shape; like every other amount in the format it
is not checked against the side or clamped, so an `out` and an offset pushing one axis in opposite
directions add and may cancel, and nothing reports it. **What a reference stores is the sum**, which
is why nothing in it records which of the two spellings it was written in and a displacement reaches
both exactly as it reaches the offset it already reached. The amount _along_ the side gets no second
spelling, because it is the other of the two amounts the offset already carries — and one pair of
screen amounts is one gap, which is what tells a gap from a slide.

**Only a connector's endpoint holds a reference, for now.** Every other shape's position is
absolute, and _Attachment_ below is where the reference lives. A reference therefore names a box or
a line, both positioned absolutely, or a connector, which answers no anchor point: the chain is one
link long, nothing resolves through anything else, and no cycle can be built. That is a restriction
of scope rather than a second rule about resolution, and what it buys is the third way of failing
removed rather than handled. Issue #89 widens a reference to any shape's position, and the cycle
question belongs to it. Whichever of the two doors opens first — that issue, or a connector that
answers anchor points — is the slice that has to make resolution cycle-safe and say what a cycle
resolves to, and it inherits the intended answer rather than a blank page: nothing, like any other
reference that does not resolve.

Displacing a figure moves every position it holds. A position that is a reference has no coordinates
to add to, so a displacement reaches its offsets instead, and displacing a connector whose endpoint
holds a reference slides that endpoint rather than carrying the connector with the shape it hangs
from. That is what makes a displacement a property of one figure rather than of a diagram.

**Taking a shape out freezes what hung from it.** Every reference **naming the removed shape**
becomes the absolute point it was resolving to at that moment, so what hung from it stays drawn
exactly where it stood rather than being re-routed, dropped or repaired. A reference naming
**another** shape is left exactly as it was, and so is one naming an identity the diagram does not
hold: the removal did not create that broken reference and does not repair it. Only the figures that
name the removed shape are rewritten, and only in the position that named it — which leaves every
other figure byte for byte as a claim about a whole diagram rather than about the figure that moved,
and no single arrangement establishes it for all of them. A frozen end is an ordinary absolute
position, with no second kind of point and no name for it. The loop behind the rule is not visible
from here and does not belong here.

Nothing is the answer in two cases, and the model treats them as one:

| The reference names                        | Resolves to |
| ------------------------------------------ | ----------- |
| a shape the diagram does not hold          | nothing     |
| an anchor point that shape does not answer | nothing     |

A shape whose position does not resolve is **not drawn**. It writes nothing, owns no position, and
answers no anchor point of its own, so a reference to it resolves to nothing in turn. There is no
error and no report: an unresolved reference is a normal state of a diagram being built, not a
fault. This is _Degenerate arrangements_ from [`model.md`](model.md) applied one layer up. A way to
ask which shapes a diagram could not draw is issue #88.

## 5. Anchor points

A shape may offer four named points: the center of its top side, of its right side, of its bottom
side and of its left side.

An anchor point is an absolute position, computed from the shape's own position when it is asked
for. **A shape answers each of them itself, and may answer none**: the four are what can be asked,
not what must exist. Deriving all nine from one bounding rectangle was declined because a rectangle
is a poor description of some figures — the center of an L-shaped connector's rectangle is off the
connector, and a center that is not on the figure is worse than no center at all, since it resolves
and puts a box somewhere nobody pointed. Answering nothing is an ordinary answer, and it is what
lets a figure with no honest answer wait for a slice of its own rather than be given a fictional
one.

A **box** answers all four, from its position and its size.

A **line** answers as a flat box: it has no thickness, so on a horizontal line the top center and
the bottom center are one point, its middle, while the left center is its left end and the right
center its right end. A vertical line is the same seen sideways.

An **connector** answers nothing for now. A route with bends has no honest answer to most of these,
and none of them is needed to draw one.

Four, rather than the nine issue #62 lists. What the corners and the center are waiting for is a
consumer, and issue #90 is where they arrive: an anchor exists to hang a connector's endpoint from,
and a side is the only one of the nine an endpoint has an unambiguous relationship with. It is also
what keeps the question in _Attachment_ answerable, since a side has an outside and a corner does
not.

## 6. Attachment

A connector's endpoint is a position, the direction the connector leaves it in, and a terminal —
that is `model.md`'s definition. What this layer adds is that the position may be a reference, and
an endpoint is the only position in this model that may be one. An endpoint attached to a shape
moves when that shape moves, and a connector with an endpoint whose reference does not resolve is
not drawn, by the rule in _Positions_: it is a shape whose position does not resolve. A connector
with two attached endpoints is the figure this model exists to make possible.

A **frozen** endpoint is attached to **nothing**: it is an absolute point like any other, so there
is no side for it to name and no figure to name one against. That is the whole of what a removal
does to an attachment, and _Positions_ is where the rule it follows is stated.

The direction the connector leaves in and the terminal it carries are not derived from the
attachment. They are the caller's. Deriving a direction from which side of a shape was attached to
is an open question below, and every anchor being a side is what keeps it answerable: a side has an
outside to leave through.

A connector's `from` endpoint standing on the right border of a box, and its `to` endpoint out in
the open. The cell at `(3, 1)` reads `├`, and neither figure wrote a junction there — the box's
border and the connector's arm terminal composed into one, which is the whole point of a side being
an anchor rather than a corner:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 8, "height": 3 } },
  "next_id": 3,
  "shapes": [
    { "kind": "box", "id": 1, "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light" },
    { "kind": "connector", "id": 2,
      "from": { "at": { "kind": "point", "x": 3, "y": 1 }, "leaving": "right",
                "terminal": { "kind": "arm" } },
      "to":   { "at": { "kind": "point", "x": 7, "y": 1 }, "leaving": "left",
                "terminal": { "kind": "arm" } },
      "stroke": "light" }
  ] }
-->

```text
┌──┐
│  ├────
└──┘
```

<!-- /render -->

## 7. Drawing

A diagram draws into a buffer the caller gives it, and that buffer's origin and size are the window.
It writes cells and stops there: turning them into text is the caller's, with the glyph catalog the
caller holds. The diagram measures nothing and sizes nothing: a shape may answer no anchor point at
all, so there is nothing to measure a canvas from, and what falls outside the window is clipped
exactly as a stamp outside a buffer's window has always been. Which part of a diagram to draw is the
caller's question in any case — a terminal draws what fits at the scroll position it is at, and no
measurement of the content answers that — so the window being the caller's does not depend on the
diagram's inability to measure itself.

Shapes are visited from the front of the order to the back, and every cell is stamped with `Below`.
By _The two orders are equivalent_ in [`model.md`](model.md), that produces the same buffer as
visiting them back to front with `Above`, so nothing about the picture depends on the choice.

**The order is composition, not occlusion.** A shape in front does not hide what is behind it; it
decides first. Its base stroke wins, its arms win on the sides it decided, and the sides it left
`Unset` fall through — which is how two crossing lines produce a junction rather than one
interrupting the other. A figure that does hide what it covers does so by writing literal glyphs,
which is what a box's fill already is. That is the whole of the opacity this phase has.

Neither line knows the other is there, and neither one is in front in any way that matters — the
front-most one is the horizontal, and the cell where they cross is `┼` rather than a `│` with a gap
in it:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 7, "height": 5 } },
  "next_id": 3,
  "shapes": [
    { "kind": "line", "id": 1, "at": { "x": 3, "y": 0 }, "len": 5, "orientation": "vertical",
      "stroke": "light" },
    { "kind": "line", "id": 2, "at": { "x": 0, "y": 2 }, "len": 7, "orientation": "horizontal",
      "stroke": "light" }
  ] }
-->

```text
   │
   │
───┼───
   │
   │
```

<!-- /render -->

Drawing is repeatable and changes nothing about the diagram. Drawing the same diagram twice into two
equal windows produces two equal buffers.

## 8. Ownership

Drawing records, at every position, which shape the cell there belongs to. The record lives in the
buffer, beside its cells rather than inside one, so a drawn diagram is one thing rather than two,
asking what is at a position is asking the buffer, and two cells that look the same are the same
cell whoever wrote them. **What the buffer keeps is the `ShapeId` itself**, which the core now holds
for exactly this reason, and it is asked for with a position **counted from the window's own
corner** — the column and row a click arrives as, rather than a point in the plane — so a caller
that draws a rectangle asks about that rectangle without reading the window's origin first. The
buffer is the only place that could hold the record at all: a shape writes through a surface with no
reader, so anything watching from outside sees stamps go past without learning what they did, and
telling a stamp that decided something from one that changed nothing is not available to a writer
that cannot read.

The identity rides on the layer a figure draws through rather than on the write itself, so a figure
is never told which identity it draws under and a figure with an identity parameter — one that could
be drawn under two, or under none — is not something this model has. A figure drawn by nobody owns
nothing, which is what a shape the core draws on its own gallery gets.

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

| Change   | What it does                                                                            |
| -------- | --------------------------------------------------------------------------------------- |
| add      | Puts a shape at the front of the order and gives it an identity it has not given before |
| remove   | Takes a shape out of the diagram                                                        |
| replace  | Puts a different shape under an identity, in the same place in the order                |
| forward  | Moves a shape one place toward the front of the order                                   |
| backward | Moves a shape one place toward the back of the order                                    |

None of them can fail. Naming a shape the diagram does not hold changes nothing, which is the same
answer _Positions_ gives a reference to a shape that is not there.

**The identity `add` gives is one the diagram has not handed out before**, which is what makes it
worth taking rather than a number that happens to be free. A caller who wants a name of their own
puts a shape under one instead, and that is a different change: it hands back nothing, and it does
not move the counter, so what it is called and what the diagram issues next are two separate
questions.

**A shape is a value, and changing one is replacing it.** Shapes are immutable. A diagram does not
reach into a box and widen it; it takes a box that is wider and puts it where the old one was. What
survives a replacement is what the diagram owns — the identity and the place in the order — and
everything the shape owns is the new shape's: its kind, its parameters and its position.

Moving is a replacement like any other: the new shape is the old one displaced by a delta. A box and
a line are displaced through their own position, and a connector through both of its endpoints at
once — a connector is not the exception to this. Displacing a figure builds a value and changes
nothing in a diagram; replacing is what puts the value in. What a displacement means for a position
that is a reference is stated under _Positions_.

Removing a shape freezes what hung from it, which _Positions_ states: every reference naming it
becomes the absolute point it was resolving to, and nothing else about the diagram is rewritten. A
shape put back under the removed identity is not re-hung from, because nothing went looking for the
reference the frozen end was.

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
- Taking a figure out leaves every figure that referenced it drawn exactly where it was, and the
  rest of the diagram unchanged.
- In an overlap, every position resolves to the front-most shape that decided it, and changing the
  order changes the answer.
- A shape that stamps a position without changing anything there does not take it.
- A position no shape decided resolves to nothing, and so does one outside the window.
- A position resolves to the same shape whether the diagram is drawn front to back or back to front.
  **This is the property the viewport sentence of _Ownership_ is for**, because a caller rendering a
  rectangle other than the window asks with offsets relative to that rectangle and gets a truthful
  answer about a different cell: the offsets are the rectangle's, and no answer tells them
  otherwise.

## 11. Open questions

Each of these is waiting for an answer, and the answer is prose in this document, or in the module
that owns what the question is about where nothing outside it can observe the choice. A feature spec
that needs one of them answered amends this document first and then implements the slice.

- **Can a caller edit an identity after the fact?** Choosing one at the moment a shape is added is
  settled — see _Identity_ and _Changing a diagram_ — and issue #62 anticipated editing without
  asking for it. What would settle it: the first slice that edits rather than writes, which no
  caller has asked for yet.
- **What does a connector anchor to?** Connectors answer no anchor point, so nothing can hang off
  one. What would settle it: a figure that has to attach to a connector, most likely a label on it.
  Answering this is also one of the two ways a chain of references becomes longer than one link,
  which is what brings the cycle question back — see _Positions_.
- **Does moving a set of figures keep their gaps?** The two directions of a single displacement are
  settled and they are **not** the same rule: displacing the figure a reference hangs from carries
  the endpoint with it and leaves the gap, while displacing the figure that holds the reference
  slides the endpoint and grows the gap — see _Positions_. What is not settled is what a caller gets
  who moves several figures in a row, since each displacement belongs to one figure and the two
  compose into an arrangement nobody chose. Nothing displaces more than one figure today: there is
  no selection, no group and no consumer that moves a set. The arrangement two displacements compose
  into is already pinned, derived from the rule rather than decided by it, so a slice that decides
  it otherwise has to say so rather than discover it in a picture:
  [`displacing_the_box_and_then_the_connector`](../crates/monospace-diagram/src/diagram.rs). What
  would settle it: the first consumer that displaces more than one figure at a time, which is a
  selection.
- **Does an attachment decide the direction a connector leaves in?** Attaching to a box's right side
  and leaving leftward is expressible today and draws something nobody wants. What would settle it:
  the first slice where the caller's direction and the anchor's side are routinely the same.
- **How big is a diagram?** Nothing measures one. The window stays the caller's either way — which
  part of a diagram to draw is a question about a viewport, not about the content — so what is
  missing is a way to ask a diagram what it occupies. What would settle it: a consumer that has to
  choose a window with nothing to base it on, an exporter being the obvious one.
- **What does a group of shapes do to this model?** Issue #58 asks for one, and groups are
  deliberately out of scope here. A group contains shapes, which is the first thing in this document
  that would want to speak about shapes generically rather than by kind.
- **Is a diagram serializable, and in what?** Settled: in JSON, by
  [`monospace-description`](../crates/monospace-description), which is a layer above this one and
  holds no domain logic — it reads a file into a `Diagram` and knows nothing of this crate's
  operations. The format is **no longer provisional**, which is what the application saving a
  diagram settles: a format nobody has to write has no obligations, and one somebody's files depend
  on does. What the format still does not have is a **writer**, and what that would owe a reader —
  what changes, what breaks, whether a version is named — is unsettled and is the next question
  here. See [the crate](../crates/monospace-description/src/lib.rs).
