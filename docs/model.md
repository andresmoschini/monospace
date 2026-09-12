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
argued in [ADR-0008](decisions/0008-compose-overlapping-cells-with-three-state-arms.md) and
[ADR-0009](decisions/0009-degrade-a-cell-to-its-base-stroke.md) instead of asserted here. No code
carried over.

This document describes three mechanisms: how cells accumulate in a buffer, how a buffer becomes
characters, and how a shape puts cells in both without its caller computing any of them. Everything
above those — an input format, parsing, deciding where a figure goes — belongs to layers that do not
exist yet and are not described here. Text and diagonals are out of the model entirely; they will be
designed when they arrive.

What this document owns beyond those three mechanisms is the domain's **open questions**, including
the ones about layers it does not describe yet. They are collected under _Open questions_, because
each of them is answered by writing model for the layer it belongs to, and answering it anywhere
else would leave the answer somewhere a spec has no reason to look. _Deliberately unresolved_ is a
different list: those are questions closed by choice, not waiting for one.

The buffer is a temporary working surface that helps render. It is not the document.

The two decisions this model rests on are recorded separately:
[ADR-0008](decisions/0008-compose-overlapping-cells-with-three-state-arms.md) for how overlapping
cells compose, and [ADR-0009](decisions/0009-degrade-a-cell-to-its-base-stroke.md) for what to draw
when no character matches.

## 1. Vocabulary

| Term            | Meaning                                                                       |
| --------------- | ----------------------------------------------------------------------------- |
| `Buffer`        | A window of cells, with an origin and a size                                  |
| `Cell`          | Either a base stroke and four arms, or one literal glyph                      |
| `Side`          | Top, right, bottom or left                                                    |
| `Arm`           | What a cell has on one side: `Set(stroke)`, `Closed` or `Unset`               |
| `Stroke`        | A name, nothing more                                                          |
| `BaseStroke`    | The cell's own stroke: what every arm is drawn in unless it carries one       |
| `Glyph`         | What a cell renders to: one grapheme cluster                                  |
| `GlyphKey`      | The four sides of a rule: a stroke name on each, or nothing                   |
| `GlyphRule`     | One `GlyphKey` mapped to a glyph                                              |
| `GlyphSet`      | A group of rules as they are written or loaded: one table                     |
| `GlyphCatalog`  | Every rule in play; sets go in in order and the first to claim a key keeps it |
| `stamp`         | The single write operation                                                    |
| `Above`/`Below` | The two stamp modes: overwrite what is there, or only fill what is undecided  |
| `Surface`       | One write operation and no reader; what a shape draws into                    |
| `Shape`         | A value describing a figure, which draws itself into a surface                |
| `Piece`         | A shape placed by another shape, given the positions it is to write           |
| `Direction`     | Up, right, down or left: a way to move in the plane                           |
| `Endpoint`      | Where an arrow ends: a position, the direction it leaves in, and a head       |

`Arm` names both the concept and its three-state value. `Side` names the four positions. If
implementing shows they need separating, `ArmState` is the obvious name for the value.

`Side` and `Direction` share four names and are not the same thing: a side is a place on a cell, a
direction is a way to move across the plane. A figure reasons in directions — an arrow leaves an
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

A defined cell holds a base stroke, always present, and four arms. Each arm is in one of three
states:

| State         | Meaning                                          |
| ------------- | ------------------------------------------------ |
| `Set(stroke)` | A stroke runs to that side, in that style        |
| `Closed`      | No stroke runs to that side, and that is decided |
| `Unset`       | Whoever stamps next decides this side            |

`Unset` is what makes the rest work. It does not mean "not known yet", it means **"not mine to
decide"**:

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
beneath it even by accident. [ADR-0031](decisions/0031-a-shape-draws-into-a-surface.md) records the
trait, and the one adapter the crate ships that binds a buffer to a stamp mode for it. Shapes exist
so that a caller describes a figure instead of computing positions and characters. Every position
and every character inside a figure is the figure's own business. Deciding _where_ a figure goes is
still not: the caller says where.

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
layer does not exist yet. [ADR-0030](decisions/0030-drop-extent-until-a-caller-needs-it.md) records
why the extent this section used to define was withdrawn, and what would bring a bounding rectangle
back in its place.

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
the piece instead of being restated by every figure that has one.
[ADR-0028](decisions/0028-give-each-fragment-its-own-cell-rule.md) records that, and the split
between the sides a piece is told and the directions the figure above it reasons in.

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

An **arrow** is two **endpoints** and a stroke. An endpoint is a position, the direction the arrow
leaves it in, and the glyph of the head that sits there. A head occupies the endpoint position
itself and points opposite to the direction that endpoint leaves in.

### An end is an arm; a head is a glyph

A **line's end** is the cell where the stroke stops. It carries the one arm the line runs on and
leaves its other three sides `Unset`, so it renders through the glyph set like every other stroke
cell — which is what makes an end follow the diagram's style instead of its caller's taste — and so
that whatever arrives there afterwards may still join it. Two lines meeting at right angles with an
end at the same position compose into the corner the two of them make, in either stamp order.

The price is that an end is not visible as an end. Measured in the tables of
[`glyph-sets.md`](glyph-sets.md): every single-stroke set already answers the four single-arm keys —
Light with `─` and `│`, ASCII with `-` and `|` — and `╴ ╵ ╶ ╷` appear in no set at all. So a line
renders as a run of segments does, and what makes its end an end is which sides it leaves undecided
rather than the character it draws.

An **arrow's head** is the other answer, because a head points and no set holds a rule that points:
`▲ ► ◄ ▼` are in no set, and `╾ ╼` are claimed in both mixing sets that hold them by keys meaning
heavy on one side and light on the other. A head is therefore a chosen glyph, in the sense of _A
cell can be a literal instead_, supplied by the caller — and nothing connects into one.

That asymmetry is a limit of the data rather than a preference. Heads that follow the glyph set
would be the better answer, and they are an open question below.
[ADR-0029](decisions/0029-draw-a-line-end-as-one-arm.md) records the decision, what it reverses, and
what would reverse it back.

### The route of an arrow

An arrow's route is derived from the two endpoint positions and the two directions, and from nothing
else. The head glyphs change no position it occupies.

Each endpoint has a **starting position**: one step from it in that endpoint's own leaving
direction. That step goes into the figure when the direction heads toward the other end, and out of
it when the direction heads away. The route runs between the two starting positions and stays inside
the **route rectangle**, the smallest rectangle containing both of them — which exceeds the
rectangle the two endpoints span by exactly one cell on each side a direction points away from, and
by nothing anywhere else. Within that bound the route takes the fewest bends its two directions
allow.

Concretely, the route is a **path** from one endpoint position to the other whose first step is the
first endpoint's leaving direction, whose last step arrives at the second endpoint against that
endpoint's leaving direction, whose runs alternate between horizontal and vertical, and which stays
inside the route rectangle. The route is the one of those with the fewest bends, and where several
share the fewest, the one that turns at the middle of the route rectangle on whichever coordinate
the bends leave free. The two endpoint positions carry the heads, so the route writes the path
without its two ends: a bend belongs to exactly one piece of the route, and no piece of the route
writes where a head does.

Where no such path exists the route is empty and the arrow is its two heads. That is the rule's
answer rather than an exception to it. It happens where the route rectangle is one cell thick and an
alternating path cannot fit inside it: the two endpoints at one position, or two identical
directions with the endpoints in line on that axis. Drawing nothing where a path _does_ exist would
be a defect, and has been one — an earlier implementation of this idea silently drew nothing for two
endpoints facing away from each other, which the rule above routes around instead.

### Degenerate arrangements

A shape whose parameters are degenerate, or describe something impossible, never fails: no error, no
panic, and the call returns. What it draws is whatever the general rule yields for those parameters,
and a degenerate arrangement does not acquire an exception of its own in order to draw something
else. A line of length 1, an arrow whose two endpoints coincide: each is permitted rather than
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

- **Fallback chains between strokes.** ADR-0009 lists them as the rejected option and says what
  would bring them back.
- **Text and diagonals.** Out of the model, not merely out of the first slice. Arrows were on this
  list until _Shapes_ was written and are not on it any more.
- **A coordinate-and-glyph output**, as an alternative to the string.
- **Color.** It fits the degradation rule — one owner imposes, the rest yield — and would be a good
  way to test whether that rule generalizes.

## 11. Open questions

Unlike the list above, these are open rather than closed: each is waiting for an answer, and the
answer is model prose for the layer it belongs to. A feature spec that needs one of them answered
amends this document first and then implements the slice, the same way spec 0002 amended _The cell_.

- **What minimal diagram description does the core accept?** A custom DSL and a structured input are
  both on the table. What would settle it: the first slice that has to read a diagram from outside
  the process, which is also the first one that makes the choice visible in the public API.
- **How is layout computed?** A fixed grid the caller places figures on, or positions computed from
  the description. The buffer takes coordinates either way, so this is a question about the layer
  above it, not about stamping.
- **What comes after the first three shapes?** _Shapes_ answers the initial set — a box, a line and
  an arrow — and the question that is left is everything with a shape of its own that is not one of
  them: a rounded corner, a double line. Each needs either its rule keyed like the rest or a chosen
  glyph, which is the route a head took. What would settle it: the first slice that needs one of
  them.
- **Where does an arrow's head glyph come from, once a glyph set can hold one?** _An end is an arm;
  a head is a glyph_ has the caller supply it, because no set holds a rule that points and the two
  characters that could have been keyed are already claimed. What would settle it: a slice that
  gives a set its own heads, which also has to decide whether the caller's glyph then becomes an
  override or goes away.
- **How does the core expose mutable state for editing?** Phase 3 edits a diagram interactively, and
  today's buffer is a write-once working surface. Whether editing mutates cells in place, rebuilds
  the buffer from a description, or keeps both, is a model question and not an implementation detail
  — it decides what the public API promises. What would settle it: the first spec that has to change
  something already stamped.
