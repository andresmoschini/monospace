<!-- The maintainer's decision draft is a Spanish file outside this repository. -->
<!-- cspell:ignore decisiones -->

# Data model: a shape can be removed and replaced

What this slice adds to `monospace-diagram`, and what it changes in `monospace-cli`. Everything
feature 080 built stays as it is unless it is named here. Why each type is shaped this way is in
[research.md](research.md); the public surface alone is in
[contracts/diagram-api.md](contracts/diagram-api.md).

## `Delta` — new, public

How far a figure moves along each axis, and nothing else. The model's _Vocabulary_ gives it this row
and gives it no other.

| Field | Type  | Meaning                        |
| ----- | ----- | ------------------------------ |
| `dx`  | `i32` | How far the figure moves right |
| `dy`  | `i32` | How far the figure moves down  |

Both fields are public, as `Pos`'s are, and the type derives `Debug`, `Clone`, `Copy`, `PartialEq`
and `Eq` — `Pos`'s derives, so the two read as one family (research.md Q1, Q2). `Copy` is there
because a caller holding one delta applies it to several figures without cloning it.

It is signed because a coordinate may be negative, and a displacement is a difference between two of
them. `Size`, which could have been it, is `u32` and means an extent.

`From<(i32, i32)>` is the second way in, for a caller holding a pair of amounts rather than a delta.
It builds one and does nothing else.

### `Delta::apply(at)` — new, private to the crate

The one place arithmetic happens, and it happens twice: `at.x.saturating_add(self.dx)`, then the
same for `y`, giving a `Pos`.

Saturating rather than wrapping or failing, for the reason research.md Q2 gives — a wrapped
coordinate can land inside a window a caller could really hold, while a saturated one is past the
end of any window a `u32` width can describe, and so draws nothing whatever the window. It is a
function rather than an operator because the workspace has no `Add` for positions anywhere —
`impl (Add|Sub|AddAssign|SubAssign)<` matches nothing across `crates/**/*.rs` — and adding one is a
decision this slice does not need to take.

## `Shape` — changed

### `displaced_by(&self, by: Delta) -> Self` — new, public

Builds a new figure, this one moved, and touches nothing: no diagram, no buffer, no counter (B3.3).
What comes back is the bare figure — no identity, no place in the order — because a figure never
carried either, and the model's _Vocabulary_ row saying otherwise is corrected on this branch.

One arm per variant, and each moves every position that variant holds:

| Variant     | What moves                      | What is copied through                      |
| ----------- | ------------------------------- | ------------------------------------------- |
| `Box`       | `at`                            | `size`, `stroke`, `fill`                    |
| `Line`      | `at`                            | `len`, `orientation`, `stroke`              |
| `Connector` | `from.at` and `to.at`, together | both `leaving` and `terminal`, and `stroke` |

A connector is not the exception. Its route is derived from its two endpoints and never described by
the caller, so displacing both is sufficient; displacing one would be the silent no-op the previous
system shipped, along with its `// TODO: implement it` (research.md Q1). A delta of nothing gives
back the same figure, which is B3.4 and the reason `Shape` grows `PartialEq`.

The box, as written and then moved two rows down — the shape of B3.1, and the one thing about this
method that is easier to see than to read:

<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 6 } },
  "shapes": [
    { "kind": "box", "at": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 3 },
      "stroke": "light", "fill": "░" }
  ] }
-->

```text
┌──┐
│░░│
└──┘



```

<!-- /render -->
<!-- render:
{ "canvas": { "origin": { "x": 0, "y": 0 }, "size": { "width": 4, "height": 6 } },
  "shapes": [
    { "kind": "box", "at": { "x": 0, "y": 2 }, "size": { "width": 4, "height": 3 },
      "stroke": "light", "fill": "░" }
  ] }
-->

```text


┌──┐
│░░│
└──┘

```

<!-- /render -->

The method takes `&self` and gives back `Self` rather than consuming either, so it composes with
`Diagram::get`, which borrows: `diagram.get(&id).map(|s| s.displaced_by(by))` is the whole
read-and-displace step, and no clone happens at the call site.

What displacing a figure that holds a **reference** means is not decided here: no figure can hold
one yet, and issue #82 is where a `Position` arrives to answer it. The model states the first answer
under _Positions_, and the specification names that rule as one of two accepted with nothing to
verify it.

### `#[derive(Clone, PartialEq, Eq)]` — widened

`Shape` derives `Debug` today and gains `Clone`, `PartialEq` and `Eq` (research.md Q4). Every field
already supports all three, so this is a derive list and nothing else. B3.4 and B4.1 are the two
rules that need them, and they are the cheapest way to say either.

`Clone` has a second use, and it is why `replace` can take a `Shape` by value and hand nothing back:
a caller whose identity is not in the diagram has given its figure away, and `Clone` is how it keeps
one. Nothing in this slice needs it, and the crate is not `Copy` — a figure owns a stroke and a
fill.

## `Diagram` — changed

Three methods and one private helper.

### `find(&self, id) -> Option<usize>` — new, private

The linear search `forward` and `backward` already do, given a name (research.md Q3). It compares
`ShapeId`s, so an identity from another diagram is a well-formed value that matches nothing here,
which is the same answer the model's _Positions_ gives a reference to a shape that is not there.

`forward` and `backward` come to search through it too, which is a `refactor` commit of its own
before any of the three below lands: same behavior, existing tests passing unchanged, none added or
modified. Leaving four searches of one order spelled four ways is a structural change to be made on
its own, not smuggled into a `feat` that changes behavior.

### `get(&self, id: &ShapeId) -> Option<&Shape>` — new, public

The crate's first reader, and the only one: one query by an identity the caller already holds, and
nothing coming back the other way (B4.3).

- It borrows, and there is no `cloned()` beside it (D2). What comes back is the bare figure the
  caller added, with the identity and the place in the order held beside it in the diagram rather
  than inside it (B4.1). Nothing about the figure says where it sits.
- An identity the diagram does not hold is `None`, and the diagram is unchanged (B4.2).
- It is not a general reader: no listing of a diagram's identities, no count, no order, and no way
  to ask which shape decided a position. That last is §11's open question, settled by issue #86.

### `remove(&mut self, id: &ShapeId)` — new, public

1. `find` the entry. No entry — return, having changed nothing: no error, no report, no panic
   (B1.2).
2. Otherwise drop that entry from `shapes`, and touch nothing else.

`Vec::remove` closes the gap, so the order of what stayed is the order it was — a removal changes
the holding and nothing else. Swapping the removed entry with the last would be the same length of
code and would reorder everything behind the gap, which is why it was rejected (research.md Q3).

It returns nothing (D5): there is no history, there is nothing to undo, and the
`Option<(ShapeId, Shape)>` alternative is a value no caller could use. What it says about references
is also nothing, which D6 confirms and issue #82 settles where there is a `Reference` to test
against.

`next` is not touched. The counter only rises, so a shape added after a removal is numbered after it
and the removed identity is never handed out again (B1.3, SC-005). No code beyond the absence of a
decrement is needed for that, because `add` already increments before use.

### `replace(&mut self, id: &ShapeId, shape: Shape)` — new, public

1. `find` the entry. No entry — return, having changed nothing, and the shape handed in is not added
   either: there is no way to name a shape into existence (B2.4).
2. Otherwise overwrite that entry's `shape`, and touch nothing else.

The identity and the place in the order survive, and everything the figure owned is the new figure's
— its kind, its parameters and its position. The diagram never reaches into a box and widens it: it
takes a wider box and puts it where the old one was. A replacement may change the kind outright, a
box becoming a line, and nothing of the previous figure survives it (D4). Overwriting the entry
rather than removing one and adding another is what keeps the identity and the place in the order
for free; there is no guard on the change of kind, because the model never asked for one.

It takes the shape by value and returns nothing, for the reason `remove` returns nothing (D5) — the
caller already held the value — and for the same reason it cannot fail.

## The `monospace-cli` side

`description.rs` is unchanged: `into_diagram` already hands back a bare `Diagram` and already
discards the identities `add` returns, and nothing here needs one of them (B5.9, research.md Q6).
`assets/demo.json` is unchanged too, because the displacement's amount is a constant in the
demonstration's own code and not a field in the format (B5.8).

### `demonstrate` — changed

```text
window(), into_diagram()                    -> origin, size, diagram
caption 1, draw, render                             as written
caption 2, forward(#1), draw, render                the order changed
caption 3, get(#1) -> displaced_by(by),
          replace(#1, moved), draw, render          the position changed
caption 4, remove(#1), draw, render                 the figure is gone
```

One diagram, mutated in place between the pictures, exactly as the reorder is today. The identity is
`#1` spelled by hand at each call rather than looked up: a description names its shapes by position,
so the demonstration already knows which entry it means and has nothing to read back. B5.9 asks for
exactly that, and `get` is here for the caller who holds an identity and no figure.

Picture three's amount is a constant in this function, next to a comment saying it is an assumption
about the demonstration rather than a rule about descriptions — the treatment the reorder's already
gets. Its value is chosen so the moved figure lands on cells the shipped description already draws,
which is what makes pictures two and three tell a reorder from a displacement.

Four pictures, not three, and the fourth appended after the pair rather than interleaved with it, so
that all four are about one figure: it moved, it was displaced, and then it is gone (B5.4).

### The tests that read the demonstration — one changes, one only says "two"

`demonstrated_pictures` in `main.rs` splits the output once on `\n\n` and takes a two-tuple. With
four pictures it splits three times and returns four, and the two tests using it compare the first
two — the reorder, which is what they are about — so they keep their meaning unchanged.

`first_demonstrated_picture` in `tests/cli.rs` needs no change at all, and that is worth saying
rather than leaving to be found: it takes the **first** block, and the new pictures are appended
below rather than inserted, so the coordinates `char_at` reads at `y + 1` do not move either
(research.md Q6). What does change is the comment above it, which says "the first of the two
captioned pictures" — a claim about a count that is now wrong, corrected in the commit that makes it
wrong.

A description holding no shapes, or exactly one, prints four identical pictures and fails nothing
(B5.7): `forward`, `replace` and `remove` are all no-ops on an identity the diagram does not hold,
and there is no branch to get wrong. Given a path the application still prints one picture and
nothing else, which is what `cargo xtask render` embeds (B5.6, SC-007).
