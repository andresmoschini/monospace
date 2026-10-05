# The buffer and render model

**Status:** Design intent, not observed behavior. Nothing described here is implemented. It is
written down first so that each feature spec can implement a slice of it and say which slice, rather
than restating the whole model or inventing its own vocabulary. Amend it when reality contradicts
it, and say so in the commit.

**Created:** 2026-09-07

**Provenance:** The idea, part of the domain logic, the design and this model come from a private
project by the same author that will not be published; this repository replaces it as the public
implementation. What came across is the shape of the problem, rewritten as prose here and then
criticized as prior art rather than adopted as given — which is why the two rules it rests on are
argued below rather than asserted here: _The cell_ and _Stamping_ for how overlapping cells compose,
and _Rendering_ for what to draw when no character matches. No code carried over.

This document describes three mechanisms: how cells accumulate in a buffer, how a buffer becomes
characters, and how a shape puts cells in both without its caller computing any of them. Everything
above those — an input format, parsing, deciding where a figure goes — belongs to layers that are
not described here. The first of them now has a document of its own:
[`diagram-model.md`](diagram-model.md) owns what holds a figure after it has been drawn, and this
one stays the buffer and render model. Text and diagonals are out of the model entirely; they will
be designed when they arrive.

What this document owns beyond those three mechanisms is the domain's **open questions**, including
the ones about layers it does not describe yet. They are collected under _Open questions_, because
each of them is answered by writing model for the layer it belongs to, and answering it anywhere
else would leave the answer somewhere a spec has no reason to look. _Deliberately unresolved_ is a
different list: those are questions closed by choice, not waiting for one.

The buffer is a temporary working surface that helps render. It is not the document.

The two decisions this model rests on are argued where they belong rather than filed elsewhere: _The
cell_ and _Stamping_ for how overlapping cells compose, and _Rendering_ for what to draw when no
character matches.

## 1. Vocabulary

| Term            | Meaning                                                                        |
| --------------- | ------------------------------------------------------------------------------ |
| `Buffer`        | A window of cells, with an origin and a size                                   |
| `Cell`          | Either a base stroke and four arms, or one literal glyph                       |
| `Side`          | Top, right, bottom or left                                                     |
| `Arm`           | What a cell has on one side: `Set(stroke)`, `Closed` or `Unset`                |
| `Stroke`        | A name, nothing more                                                           |
| `BaseStroke`    | The cell's own stroke: what every arm is drawn in unless it carries one        |
| `Glyph`         | What a cell renders to: one grapheme cluster                                   |
| `GlyphKey`      | The four sides of a rule: a stroke name on each, or nothing                    |
| `GlyphRule`     | One `GlyphKey` mapped to a glyph                                               |
| `GlyphSet`      | A group of rules as they are written or loaded: one table                      |
| `GlyphCatalog`  | Every rule in play; sets go in in order and the first to claim a key keeps it  |
| `stamp`         | The single write operation                                                     |
| `Above`/`Below` | The two stamp modes: overwrite what is there, or only fill what is undecided   |
| `Surface`       | One write operation and no reader; what a shape draws into                     |
| `Shape`         | A value describing a figure, which draws itself into a surface                 |
| `ShapeId`       | The name a figure is drawn under, and what a stamp records beside the cell     |
| `Piece`         | A shape placed by another shape, given the positions it is to write            |
| `Direction`     | Up, right, down or left: a way to move in the plane                            |
| `Endpoint`      | Where a connector ends: a position, the direction it leaves in, and a terminal |

`Arm` names both the concept and its three-state value. `Side` names the four positions. If
implementing shows they need separating, `ArmState` is the obvious name for the value.

`Side` and `Direction` share four names and are not the same thing: a side is a place on a cell, a
direction is a way to move across the plane. A figure reasons in directions — a connector leaves an
endpoint in one — and a piece is told sides. They stay apart, and the translation happens where the
two meet.

## 2. The buffer

A buffer is a window. It has an origin `(x, y)` and a size `(width, height)`, and those give the
lowest and highest positions it answers for.

- Coordinates are absolute. `x` grows right, `y` grows down, and both may be negative. Sizes are
  never negative, and zero is allowed: a window with no width or no height simply has no positions,
  which is empty rather than invalid.
- Each position inside the window holds a cell **or nothing**. An undefined cell is the absence of a
  cell, not a blank one.
- Stamping outside the window does nothing and is not an error. The buffer receives an absolute
  position and decides whether to attend to it.
- There is no erase. A buffer is filled and then discarded; undo belongs to the layers above.
- The buffer knows nothing about glyphs. It holds stroke names; translating to characters is the
  renderer's job.

## 3. The cell

A defined cell holds a base stroke, always present, and four arms — and not a character. Merging two
characters on the second write would need a table that grows with the square of the alphabet and is
undefined for most pairs, since `┤` merged with `━` has no principled answer, and it destroys the
difference between a light stroke and a heavy one by the time a third figure arrives. What a cell
holds is what the figures meant; the character is derived from it at the end.

Each arm is in one of three states:

| State         | Meaning                                          |
| ------------- | ------------------------------------------------ |
| `Set(stroke)` | A stroke runs to that side, in that style        |
| `Closed`      | No stroke runs to that side, and that is decided |
| `Unset`       | Whoever stamps next decides this side            |

`Unset` is what makes the rest work, and it is there because the two obvious answers are both wrong.
If the last writer decided every side, a horizontal line would erase the vertical one's connections
where they cross; if the first writer did, nothing could ever sit on top of what was stamped before
it. Both answers are needed, sometimes in the same cell, and neither figure knows the other exists —
so a side has to be able to say _not mine to decide_ as well as _mine_, and that is what the third
state is for.

It does not mean "not known yet", it means **"not mine to decide"**:

- A horizontal segment is stamped with its left and right arms `Set` and its top and bottom `Unset`,
  so anything crossing it later can connect.
- The top border of a filled shape is stamped with its inner side `Closed` on purpose, so that
  nothing stamped afterwards connects into the fill, and with its outer side `Unset`, so that
  anything arriving from outside still joins it.

Closing a side a figure does not use is a decision too, and it says "nothing may ever connect here".
A figure that closes every side it has no stroke on draws exactly like one that abstains on them —
at render time `Closed` and `Unset` are the same — and refuses every junction from the moment a
second figure reaches it. `Unset` is the default for a side a figure has no opinion about; `Closed`
is for the sides it is protecting.

An undefined cell and a cell with four `Closed` arms are **not the same thing**. The first is the
absence of a cell; the second is a decision.

### A cell can be a literal instead

Most cells are a base stroke and four arms, as above, and the character they draw is derived. A cell
can instead hold **a literal glyph**: the character it renders to, chosen rather than derived. Text
needs that, and so does a fill that hides what it covers rather than letting it show through.

A literal has no arms of its own. For composing, it behaves as a cell whose four arms are `Closed`:
nothing connects into a character. Which of the two kinds a cell ends up being belongs to the figure
in front, exactly as the base stroke does.

## 4. Stamping

The only write is `stamp(x, y, cell, mode)`. What is stamped has the same type as what is stored: a
cell. That works because `Unset` means the same thing on both sides of the operation — "not mine to
decide" — so there is no state the stamp needs and the stored cell cannot hold.

| Target                        | `Above`                                                                        | `Below`                                                                   |
| ----------------------------- | ------------------------------------------------------------------------------ | ------------------------------------------------------------------------- |
| Undefined cell                | Defines it entirely                                                            | Defines it entirely                                                       |
| Arms, stamping arms           | Writes the base stroke and every arm, except the arms the stamp leaves `Unset` | Leaves the base stroke alone; writes only the arms the target has `Unset` |
| Arms, stamping a literal      | Becomes the literal                                                            | Stays arms, and its `Unset` sides close                                   |
| A literal, stamping arms      | Becomes arms; the sides the stamp leaves `Unset` come out `Closed`             | Unchanged                                                                 |
| A literal, stamping a literal | Becomes the incoming literal                                                   | Unchanged                                                                 |

An `Unset` arm on the stamp never writes anything, in either mode. A `Closed` arm does write:
closing is a decision, and it is what lets a filled shape stamped below stop a later one from
connecting into it.

The last three rows are one rule seen from four sides: a literal composes as a cell with four
`Closed` arms, and which kind the cell ends up being belongs to the figure in front, exactly as the
base stroke does. The `Closed` arms are not decoration. Without them a literal would be opaque in
one order and transparent in the other, and the equivalence below would stop holding the moment a
literal sat between two figures.

### The two orders are equivalent

The rules above imply a property worth keeping as a test:

> Stamping front to back with `Below` produces exactly the same buffer as stamping back to front
> with `Above`.

Either way the base stroke ends up owned by the topmost figure, and each arm ends up owned by the
topmost figure that decided it, with abstentions falling through to the ones behind.

The practical consequence is performance. Going front to back allows stopping early, because a cell
whose four arms are decided and whose base stroke is set can no longer change. Going back to front
rewrites every cell once per figure. That makes `Below` the frequent path, and it is worth being
able to answer cheaply whether a cell is already decided.

The cost sits on the same side. A decided arm cannot be undecided, and a stamp is the only write
there is, so a figure cannot be inserted between two that were already stamped and three overlapping
figures cannot be reordered after the fact: they have to be stamped again into a fresh buffer. A
stack of layers per cell would be the way out of that, and it stays reachable without discarding any
of this — a stack holds cells of exactly this shape — so the arm semantics are the expensive part of
the rule and the buffer is not.

## 5. Strokes, glyph sets and the catalog

A stroke is only a name. It has no attributes and no declaration of its own; it exists because cells
and rules mention it.

A rule maps a combination of four arms to a glyph. Each side of the key carries a stroke name or is
empty.

A glyph set is a group of rules as they are written: one of the tables in
[`glyph-sets.md`](glyph-sets.md), or a file someone loads. A set may cover a single stroke or a
mixture of them, and the system gives neither any special behavior. They are direct mappings, which
is precisely why someone can write one by hand.

Sets reach a catalog two ways. The common ones ship with whichever library holds them, built in as
data rather than parsed at startup — `monospace-core` ships Light, `monospace-glyph-sets` ships
ASCII, Double, Heavy, Light Round and the four sets that mix two strokes. A set someone writes for
their own diagrams is the other way: loaded from a file when they ask for it. That difference is
about where a set comes from, not about how its rules behave: once a set is in a catalog, nothing
tells a built-in rule from a loaded one, and nothing tells which library a built-in rule shipped
from either.

The catalog is where rules end up. Sets go into it in order, the first rule to claim a key keeps it,
and answering a key afterwards is one lookup. The grouping does not survive that: once a catalog is
built, nothing can tell which set a rule came from, and no result would change if it could. Order is
a property of how a catalog is built, not of what it holds — which is also how "render this in ASCII
alone" is expressed, by building a catalog from that set and no other.

Incomplete sets are allowed. Each stroke's single-stroke set should be complete, since that is what
the degradation rule below falls back on.

## 6. Rendering

The renderer takes a buffer, a catalog and an absolute rectangle: a position `(x, y)` and a size
`(width, height)`.

For each position in the rectangle:

1. If there is no cell, it is a space.
2. Otherwise build the key from the four arms. **When building the key, `Unset` and `Closed` are the
   same**: both mean no stroke on that side.
3. Look the exact key up in the catalog.
4. If it is not there, move **every connected arm to the cell's base stroke** and look up again.
   Arms with no stroke stay without one.
5. If that is not there either, there is no glyph.

That list is for a cell of arms. A cell holding a literal glyph renders as that glyph: no key is
built, no lookup happens, and degradation never applies to one.

There are no further attempts and no special conventions: two lookups. When a combination does not
exist, the whole cell is drawn with the stroke the topmost figure imposed.

Degrading the whole cell rather than as much of the mixture as still resolves is what makes the
answer to _why that character?_ a single sentence: the figure that owns the cell imposes its stroke
on all of it. The base stroke is written by the figure that last claimed the cell, so the rule reads
as a rule about figures rather than about tables, it can be predicted without knowing which sets are
loaded, and adding a set can only add exact matches — it can never silently redraw a cell that was
already resolving. Keeping as much of the mixture as fits is the alternative, and it keeps more of
the drawing: it is the only rule that keeps anything at all in a cell mixing three strokes, since no
three-stroke set exists. What it costs is that the answer then depends on which sets happen to be
loaded, so adding one can redraw cells that had nothing to do with it, and _why that character?_
becomes _because the search found this combination first_. A fallback stroke declared per stroke,
substituted along a chain before the base stroke, was declined for the mirror of that reason, and
_Deliberately unresolved_ sets out what it costs. Either stays available additively: the search is
an addition around the same two lookups, the chain a map consulted before them, and nothing in the
buffer, the cell or the tables changes to adopt either.

"No glyph" and "no cell" produce the same thing: a space in the text output and, in the coordinate
output that may come later, a position simply not emitted. With the single-stroke sets complete,
that case only arises when a cell has no connected arm at all, or when the set for its base stroke
was never loaded.

The text output is a rectangle of exactly `width` × `height`: trailing spaces are not trimmed, the
last line ends with a newline, and positions falling outside the buffer's window are spaces.

## 7. Shapes

A **shape** is a value describing a figure: constructed where it is used, drawn, discarded. It has
no mutable state and no lifecycle, so drawing the same shape twice produces the same writes. It has
no opinion about any other shape either — two that overlap compose through the stamp, in the order
the caller draws them, exactly as any two stamps at those positions would.

Shapes are the layer directly above the buffer, and a shape draws into a **surface** rather than
into the buffer itself: one write operation and no reader, so a fragment cannot inspect what lies
beneath it even by accident. A surface is a trait carrying `stamp` and nothing else, and the one
adapter the crate ships binds a buffer to a stamp mode for it. Both halves are there to keep
opinions out. A surface with no reader makes "a fragment never inspects the buffer" something the
compiler refuses rather than a rule a reviewer has to check on every fragment written after it, and
binding the mode once at construction is what keeps `StampMode` out of every shape: a shape that
named one would be taking an opinion on how it composes with figures behind it, which is the one
thing a shape has no opinion about. The price is that the surface's `stamp` takes a `Cell`, so a
shape defined outside the crate builds its cells by hand and passes through none of the rules
_Complete and fragment_ below gives the fragments inside it.

Shapes exist so that a caller describes a figure instead of computing positions and characters.
Every position and every character inside a figure is the figure's own business. Deciding _where_ a
figure goes is still not: the caller says where.

### Pieces

A shape is made either of cells or of other shapes, and a caller cannot tell which from the outside:
both are drawn the same way. One made of other shapes places **pieces** and writes no position
itself; the pieces write. It computes the concrete geometry of every piece it places, and no two of
its pieces write the same position. A piece receives the positions it is to write as part of its
description rather than choosing them, and that is what makes "no position written twice" a property
of the decomposition instead of something a guard has to enforce.

A shape may leave positions inside the figure it describes unwritten. An unfilled box writes nothing
in its interior, because an interior is the absence of a figure rather than a figure covering
something.

A shape does not report what it covers. It draws, and drawing is the whole of what it does: which
positions a figure occupies is a question for the layer that decides where figures go, and that
layer does not exist yet. The extent this section used to define was withdrawn because the three
things that read it were bookkeeping rather than drawing, and each has a cheaper replacement:
comparing a filled box with an unfilled one is a fact about the rectangle rather than a question
anyone asks the shape, a write outside the bounds is an extra character in the expected buffer of a
test, and a compositor partitioning its extent among its pieces is the "no position is written more
than once" property under _Properties worth testing_. A bounding rectangle was declined on the same
test, and what brings one back in its place is a caller that needs to know what another shape
occupies — routing around an obstacle, or sizing a canvas to its content — which is also the first
layer to have to decide what a bound means for a figure whose written positions are sparse.

Composition goes to arbitrary depth and no shape depends on knowing how deep it sits: one placed as
a piece is drawable the same way at the top level. Defining a new kind of shape touches no existing
shape, because there is no central list of them to touch.

### Complete and fragment

Two kinds, and one shape can be both at once — complete to its caller and a compositor to its own
pieces.

A **complete** shape's description defines the finished figure. Everything visible in it — ends,
corners, heads, fill — is its own business, and it draws finished with no further intervention. This
is what a library user sees.

A **fragment**'s description defines only the cells it writes. It adds no end and no decoration of
its own accord, and it exists to be placed by a shape that has already decided the geometry. Where a
fragment needs something about its surroundings in order to choose what to write — whether the
position beside it belongs to a sibling of the same figure, say — that arrives as part of its
description. A fragment never inspects the surface it draws into and never inspects its siblings.

What a fragment writes follows from what it is — a corner, a border run, an interior, an end, a head
— rather than from a cell handed to it. The arms of a border run are _The cell_'s decision already,
so the figure placing one names which side of itself it is and nothing more, and the rule lives with
the piece instead of being restated by every figure that has one. A border run told which side of
its figure it is derives both its orientation and the side it closes, so "a horizontal border whose
interior is to its left" cannot be built at all. The alternative — one geometric leaf taking the
cell as a parameter — would have had every figure build a base stroke and four arms before it could
place a piece, which is one rule of _The cell_ written out once per figure, and `Arm` and the choice
between arms and a literal would have become part of what a box, a line and a connector each know.
The same split separates the sides a piece is told from the directions the figure above it reasons
in: turning a connector's leaving direction into a starting position happens in the figure, and a
path's steps into a border run's two sides happen in the route.

### The initial set

Three figures, and each of them names the **stroke** its cells are drawn in: it is the base stroke
of every stroke cell the figure writes. The core holds no default for it. What a diagram looks like
is the caller's to say, and a constant inside the core would be an appearance decision no glyph set
could reach.

A **box** is a position, a size and a stroke, with a fill as an option: corners, border runs and an
interior, placed correctly. Which pieces it has depends on its size rather than on its kind, so a
2×2 box is four corners with no run at all. Its arms are the ones _The cell_ already fixes — `Set`
along the run, `Unset` outward, and `Closed` on the side facing its own interior only when the box
carries a fill, `Unset` there too when it does not — so a stroke reaching a box from outside always
joins its border, and one reaching the interior side stops only when the box is filled. A fill is a
chosen glyph, in the sense of _A cell can be a literal instead_.

A **line** is a position, a length, an orientation and a stroke. It names no glyph of its own; what
its two end cells hold is the next section.

An **connector** is two **endpoints** and a stroke. An endpoint is a position, the direction the
connector leaves it in, and a **terminal**; `at` is the cell the terminal hangs from. What a
terminal may write is vocabulary this document owns, so naming one more of them is a change here and
in the figure, and to whatever states it as a rule. A glyph terminal's glyph points opposite to the
direction that endpoint leaves in.

### What a terminal writes

A **terminal** is what an endpoint contributes to the cell at `at`, and it is a **glyph** or an
**arm**. It writes that cell and nothing else.

An **arm** writes one arm — the one the connector arrives on — in the connector's own stroke, and
leaves its other three sides `Unset`, so it renders through the glyph set like every other stroke
cell and whatever reaches it afterwards may still join it. The price is that an arm is not visible
as an arm: measured in the tables of [`glyph-sets.md`](glyph-sets.md), every single-stroke set
already answers the four single-arm keys, so what makes a cell an end is which sides it leaves
undecided rather than the character it draws. A chosen glyph was the other answer, and two things
weighed against it. A glyph the caller supplies does not follow the style: two lines of the same
stroke rendered against ASCII would carry whatever characters their callers happened to pass, and
that would make the end the one place where a caller decides how a drawing looks. And a chosen glyph
refuses to join, so two lines meeting at right angles at a shared end could not make a corner —
which is what a diagram wants there. An end's job is to say where the stroke stops and what may join
it there, and arms say that where a literal cannot. The measurement still holds and only the
conclusion drawn from it went the other way: an end really is indistinguishable from a segment in
the text, and what makes it an end is the sides it leaves undecided.

A **glyph** writes one chosen glyph, in the sense of _A cell can be a literal instead_, supplied by
the caller — and nothing connects into one, because a literal is decided on every side. It is that
answer because a glyph points and no set holds a rule that points: `▲ ► ◄ ▼` are in no set, and
`╾ ╼` are claimed in both mixing sets that hold them by keys meaning heavy on one side and light on
the other.

That asymmetry is a limit of the data rather than a preference. Heads that follow the glyph set
would be the better answer, and they are an open question below. One change would reverse the half
above as well, and it stays available rather than settled: four new single-arm rules rendering
`╶ ╴ ╵ ╷` would make an end both derived and visibly an end. Those four keys are claimed today, so
adding them is not a tweak but a change to what existing cells render, which is why it is a question
for a slice of its own rather than a line to add.

### The route of a connector

A connector's route is a **path** between its two endpoint positions: its first step is the
direction the connector leaves its `from` endpoint in, its last step arrives at the `to` endpoint
against that endpoint's own leaving direction, and its runs alternate between horizontal and
vertical. A path visits no position twice, and it passes through neither endpoint position, because
a terminal is there — so one that would have to cross an endpoint is not a path at all. The path
writes no endpoint cell of its own, whatever a terminal puts there.

Nothing bounds where a path may go. Two endpoints facing away from each other along one line are
joined by a route that travels around the outside, however far apart they are:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": -1 }, "size": { "width": 2, "height": 6 } },
  "next_id": 2,
  "shapes": [ { "kind": "connector", "id": "#1",
    "from": { "at": { "kind": "point", "x": 0, "y": 0 }, "leaving": "up",
      "terminal": { "kind": "glyph", "glyph": "▼" } },
    "to": { "at": { "kind": "point", "x": 0, "y": 3 }, "leaving": "down",
      "terminal": { "kind": "glyph", "glyph": "▲" } },
    "stroke": "light" } ] }
-->

```text
┌┐
▼│
 │
 │
▲│
└┘
```

<!-- /render -->

Where several paths exist, which one the connector draws is the connector's own business. The rule
that ranks them, and the reason each of its terms is there, are the `Design notes` of
[`shape::connector`](../crates/monospace-core/src/shape/connector.rs); nothing outside that module
observes the choice beyond the picture it produces, so changing it amends nothing here.

Where no path exists the route is empty and the connector is its two terminals.

Nothing bounds a path, and the bound this section used to place is gone for a measured reason. It
was the rectangle its two starting positions span, and a rectangle one cell thick holds no
alternating path at all, so an arrangement whose route had to escape sideways had nowhere to run
inside it. What that bound was reaching for is the shortest path, and saying so directly costs the
model a construct instead of earning it a second one. The arrangements that still draw no route are
one family rather than a spread: an endpoint standing on the cell the route would arrive at, its two
heads adjacent.

### Degenerate arrangements

A shape whose parameters are degenerate, or describe something impossible, never fails: no error, no
panic, and the call returns. What it draws is whatever the general rule yields for those parameters,
and a degenerate arrangement does not acquire an exception of its own in order to draw something
else. A line of length 1, a connector whose two endpoints coincide: each is permitted rather than
rejected, and what comes out follows the rule rather than a guard. Restricting one later is a change
to the rule, made when it turns out to cause a problem — not a case bolted on in advance.

## 8. Worked examples

All four are verified against [`glyph-sets.md`](glyph-sets.md).

**An exact match.** Top arm `light`, right arm `double`, the other two without a stroke. The key
exists, so it renders `╘`. The base stroke plays no part.

**Degradation.** Base stroke `heavy`; top arm `light`, right `double`, left `heavy`, bottom without
a stroke. No character exists for that combination. Every connected arm moves to `heavy`, and the
key `(heavy, heavy, —, heavy)` exists: it renders `┻`.

**A crossing that does exist.** A vertical `double` line crossed by a horizontal `light` one:
`(double, light, double, light)`. The key is in the mixing set and renders `╫`, whichever of the two
figures set the base stroke.

**A degraded T.** Right arm `light`, bottom `light`, left `double`, top without a stroke, base
stroke `light`. That mixture has no character: everything moves to `light` and it renders `┬`. With
a `double` base stroke it would have rendered `╦`. The detail of the mixture is lost, the shape is
not.

## 9. Properties worth testing

- A single-stroke set covers all 15 of its combinations, so a catalog built from one answers every
  key a cell of that stroke can produce.
- Front to back with `Below` and back to front with `Above` produce the same buffer.
- That equivalence survives a literal in the middle of the stack, which is what the literal's four
  `Closed` arms are for.
- Stamping outside the window changes nothing.
- `Below` on a fully decided cell changes nothing.
- The exact key wins over the degraded one; with neither, a space.
- An undefined cell and a cell with four `Closed` arms render the same unless a set defines the
  empty key. The test records the behavior; the question of whether that is wanted is deliberately
  left open.
- A shape writes no position more than once in one drawing. The decomposition is what produces that,
  so a counter per position tests the decomposition rather than a guard.
- Two lines whose ends land on one position render the corner the two of them make, whichever order
  they are drawn in.

## 10. Deliberately unresolved

- **Fallback chains between strokes.** Each stroke declaring a stroke to fall back to, substituted
  along that chain before the base stroke. Declined, as _Rendering_ sets out: it introduces an
  ordering question that does not otherwise exist, since the chain and the base stroke could each go
  first and the two give different characters for the same cell, and the discipline it needs —
  declaring a chain only for a genuine variant — cannot be checked by anything. It would come back
  if a second variant stroke were declared and copying whole tables again became the cost.
- **Text and diagonals.** Out of the model, not merely out of the first slice. Connectors were on
  this list until _Shapes_ was written and are not on it any more.
- **A coordinate-and-glyph output**, as an alternative to the string.
- **Color.** It fits the degradation rule — one owner imposes, the rest yield — and would be a good
  way to test whether that rule generalizes.

## 11. Open questions

Unlike the list above, these are open rather than closed: each is waiting for an answer, and the
answer is model prose for the layer it belongs to. A feature spec that needs one of them answered
amends this document first and then implements the slice, the same way spec 0002 amended _The cell_.

An answered question leaves this list, and its answer's home is named where it was. _How does the
core expose mutable state for editing?_ has left: the core exposes none, and
[the diagram model](diagram-model.md) holds a diagram above it that the buffer is rebuilt from.

- **What minimal diagram description does the core accept?** A custom DSL and a structured input are
  both on the table. What would settle it: the first slice that has to read a diagram from outside
  the process, which is also the first one that makes the choice visible in the public API.
- **How is layout computed?** A fixed grid the caller places figures on, or positions computed from
  the description. The buffer takes coordinates either way, so this is a question about the layer
  above it, not about stamping. Half of it is answered already: a diagram lets one figure sit
  relative to another, per [the diagram model](diagram-model.md). Placing figures nobody gave
  coordinates for is the half still open.
- **What comes after the first three shapes?** _Shapes_ answers the initial set — a box, a line and
  a connector — and the question that is left is everything with a shape of its own that is not one
  of them: a rounded corner, a double line. Each needs either its rule keyed like the rest or a
  chosen glyph, which is the route a head took. What would settle it: the first slice that needs one
  of them.
- **Where does a terminal's glyph come from, once a glyph set can hold one?** _What a terminal
  writes_ has the caller supply it, because no set holds a rule that points and the two characters
  that could have been keyed are already claimed. What would settle it: a slice that gives a set its
  own heads, which also has to decide whether the caller's glyph then becomes an override or goes
  away.
