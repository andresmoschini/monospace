# The buffer and render model

**Status:** Design intent, not observed behavior. Nothing described here is implemented. It is
written down first so that each feature spec can implement a slice of it and say which slice, rather
than restating the whole model or inventing its own vocabulary. Amend it when reality contradicts
it, and say so in the commit.

**Created:** 2026-09-07

This document covers two mechanisms and nothing else: how cells accumulate in a buffer, and how a
buffer becomes characters. Everything above it — an input format, parsing, placement, the shapes
themselves — belongs to layers that do not exist yet and are not described here. Text, arrows and
diagonals are out of the model entirely; they will be designed when they arrive.

The buffer is a temporary working surface that helps render. It is not the document.

The two decisions this model rests on are recorded separately:
[ADR-0008](decisions/0008-compose-overlapping-cells-with-three-state-arms.md) for how overlapping
cells compose, and [ADR-0009](decisions/0009-degrade-a-cell-to-its-base-stroke.md) for what to draw
when no character matches.

## 1. Vocabulary

| Term            | Meaning                                                                       |
| --------------- | ----------------------------------------------------------------------------- |
| `Buffer`        | A window of cells, with an origin and a size                                  |
| `Cell`          | A base stroke and four arms                                                   |
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

`Arm` names both the concept and its three-state value. `Side` names the four positions. If
implementing shows they need separating, `ArmState` is the obvious name for the value.

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

## 4. Stamping

The only write is `stamp(x, y, cell, mode)`. What is stamped has the same type as what is stored: a
cell. That works because `Unset` means the same thing on both sides of the operation — "not mine to
decide" — so there is no state the stamp needs and the stored cell cannot hold.

| Target               | `Above`                                                                        | `Below`                                                                   |
| -------------------- | ------------------------------------------------------------------------------ | ------------------------------------------------------------------------- |
| Undefined cell       | Defines it entirely                                                            | Defines it entirely                                                       |
| Already defined cell | Writes the base stroke and every arm, except the arms the stamp leaves `Unset` | Leaves the base stroke alone; writes only the arms the target has `Unset` |

An `Unset` arm on the stamp never writes anything, in either mode. A `Closed` arm does write:
closing is a decision, and it is what lets a filled shape stamped below stop a later one from
connecting into it.

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

Sets reach a catalog two ways. The common ones ship with the library, built in as data rather than
parsed at startup; the rest are loaded from a file when someone asks for them. That difference is
about where a set comes from, not about how its rules behave: once a set is in a catalog, nothing
tells a built-in rule from a loaded one.

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

There are no further attempts and no special conventions: two lookups. When a combination does not
exist, the whole cell is drawn with the stroke the topmost figure imposed.

"No glyph" and "no cell" produce the same thing: a space in the text output and, in the coordinate
output that may come later, a position simply not emitted. With the single-stroke sets complete,
that case only arises when a cell has no connected arm at all, or when the set for its base stroke
was never loaded.

The text output is a rectangle of exactly `width` × `height`: trailing spaces are not trimmed, the
last line ends with a newline, and positions falling outside the buffer's window are spaces.

## 7. Worked examples

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

## 8. Properties worth testing

- A single-stroke set covers all 15 of its combinations, so a catalog built from one answers every
  key a cell of that stroke can produce.
- Front to back with `Below` and back to front with `Above` produce the same buffer.
- Stamping outside the window changes nothing.
- `Below` on a fully decided cell changes nothing.
- The exact key wins over the degraded one; with neither, a space.
- An undefined cell and a cell with four `Closed` arms render the same unless a set defines the
  empty key. The test records the behavior; the question of whether that is wanted is deliberately
  left open.

## 9. Deliberately unresolved

- **Fallback chains between strokes.** ADR-0009 lists them as the rejected option and says what
  would bring them back.
- **Text, arrows and diagonals.** Out of the model, not merely out of the first slice.
- **A coordinate-and-glyph output**, as an alternative to the string.
- **Color.** It fits the degradation rule — one owner imposes, the rest yield — and would be a good
  way to test whether that rule generalizes.
