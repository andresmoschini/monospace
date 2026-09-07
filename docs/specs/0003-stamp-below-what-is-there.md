---
status: agreed
date: 2026-09-07
---

# 0003 — Stamp below what is already there

## Why now

Spec 0002 put two overlapping boxes on the terminal and a junction where their borders meet.
Everything there is stamped `Above`, which is the only mode `stamp` has. The second mode is what
says which of two figures owns the sides they share, and there is now a picture to say it against:
the same two boxes, one argument different in the front end, and two characters different on screen.

It also closes _Stamping_ in the model, and turns the equivalence of the two drawing orders — the
property [ADR-0008](../decisions/0008-compose-overlapping-cells-with-three-state-arms.md) rests on —
from prose into a test. Two of the three tests the record names as carrying its decision land here;
spec 0001 already carries the third.

## Scope

### In

- `StampMode`, with its two values, and `stamp` taking one.
- `Below`, which on an already-defined cell leaves the base stroke alone and writes only the sides
  the target has left `Unset`.
- `Cell::is_decided`, per [ADR-0017](../decisions/0017-ask-the-cell-whether-it-is-decided.md).
- `stamp` under `Below` leaving a decided target alone instead of rebuilding an identical cell, per
  the same record.
- The front end drawing the pair of boxes a second time with `Below`, and labelling the two pairs.

### Out

Every item names where it is handled instead.

- **A positional query on the buffer.** ADR-0017 rejects it rather than postponing it, and says what
  a caller would have to settle before it could exist.
- **How the buffer stores its cells.** The storage is private and no public signature mentions it,
  so replacing it changes nothing for a caller; and a dense grid would make `Buffer::new` able to
  fail, which spec 0001 promises it cannot. A later spec, once a consumer exists to give the
  measurement a workload.
- **A caller that stops early.** Skipping a whole figure, or a region, needs the layer above the
  buffer, which does not exist yet. This slice gives it the predicate and nothing else.
- **Filled shapes.** As in spec 0002, the second box's interior is never stamped, so the first box
  shows through it in both pairs.
- **Erase, and reordering figures already stamped.** Out of the model itself: ADR-0008 records both
  as accepted costs of arms that cannot be undecided.
- **Everything spec 0001 left out and this one does not name** — degradation, a catalog from more
  than one set, loading sets from a file, walking a region — stays out, with the destinations that
  spec gave.

The near miss is the box. It does not change: the same eight cells spec 0002 settled are stamped
both ways, and the mode is the only difference between the two pairs. A demo that changed the figure
and the mode at once would show a difference nobody could attribute.

## Model slice

Completes _Stamping_ in [`docs/model.md`](../model.md), including _The two orders are equivalent_
and the sentence in the same section about answering cheaply whether a cell is already decided.

Nothing else moves. _The cell_ was amended by spec 0002 and is not touched again here, and _The
buffer_, _Strokes, glyph sets and the catalog_ and _Rendering_ are untouched: this slice adds no
glyph and renders nothing that could not be rendered before. Nothing here is new to the model — if
implementing needs a rule the model does not have, the model changes first.

## Public surface

```rust
pub enum StampMode { Above, Below }

impl Buffer {
    pub fn stamp(&mut self, at: Pos, cell: Cell, mode: StampMode);
}

impl Cell {
    pub fn is_decided(&self) -> bool;
}
```

- **The mode is a parameter rather than a second method.** ADR-0008 settled it when it named the
  operation `stamp(x, y, cell, mode)`; spec 0001 shipped without the parameter because there was
  only one mode to pass. The equivalence property wants the same thing: its test stamps one list of
  figures twice and changes nothing but the mode, which is awkward to write if the mode is spelled
  in the method name.
- **Every existing call site passes `StampMode::Above` and gets what it got before.** The only
  caller is `monospace-cli`; the edit is mechanical and the two pictures it already prints do not
  move.
- **`StampMode` derives what `Arm` derives** — `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` — for the
  same reason: it is a small field-less enum passed by value.
- **`is_decided` is on `Cell` and only on `Cell`.** ADR-0017 carries the argument, including why
  there is no `Buffer::is_decided(at)` and why `stamp` asks the cell rather than the position.

Every public item carries rustdoc as it is introduced. `StampMode` documents which side of the merge
each mode consults, since `Above` looking at the stamp's `Unset` arms and `Below` looking at the
target's is the pair a reader will swap.

## Behavior

1. `stamp` takes a mode. With `StampMode::Above` it behaves exactly as spec 0001's `stamp`: rules 3,
   4 and 5 of that spec, unchanged.
2. A position outside the window leaves the buffer unchanged in either mode, and reports nothing.
3. Stamping an undefined position with `Below` defines it exactly as `Above` does: the base stroke
   and all four arms are taken from the stamp, `Unset` arms included.
4. Stamping a defined position with `Below` leaves its base stroke alone, whatever the stamp
   carries.
5. Stamping a defined position with `Below` writes only the sides the target has `Unset`, taking
   them from the stamp. A side the target has already decided keeps its value, and a side both leave
   `Unset` stays `Unset`.
6. A cell is decided when none of its four arms is `Unset`. A defined cell always has a base stroke,
   so nothing else enters the question.
7. Stamping `Below` onto a decided cell leaves it unchanged.
8. Stamping a list of figures front to back with `Below` gives the same buffer as stamping the same
   list back to front with `Above`.

Rules 7 and 8 are the two tests ADR-0008 names that spec 0001 could not carry.

One requirement is not among the rules, because nothing can observe it: under `Below`, `stamp` must
recognize a decided target and leave it alone rather than build an identical cell and store it.
ADR-0017 accepts it with its cost stated — no test can fail for it, and this spec makes no claim
about it being faster.

## Examples

As in spec 0001, `S` abbreviates `Set`, `C` is `Closed`, `U` is `Unset`, and a cell is written
`(base stroke; top, right, bottom, left)`.

**The two modes on identical input.** A buffer at origin `(0, 0)` of size 1 x 1. Stamp
`(light; U, S, C, U)` at `(0, 0)` with `Above`, then stamp `(double; S, C, S, U)` at the same
position, once with each mode:

| Mode    | Resulting cell         | Rendered |
| ------- | ---------------------- | -------- |
| `Above` | `(double; S, C, S, U)` | a space  |
| `Below` | `(light; S, S, C, U)`  | `└`      |

With `Below`, three of the second stamp's four decisions are refused. Its `double` base stroke does
not take, by rule 4. Its `Closed` right and its `Set` bottom do not take either, because the target
had already decided both sides — that is rule 5, and it is the one to get wrong: the condition is on
the target's arms, not on the stamp's. Only the top, which the target left `Unset`, is written, and
the left stays `Unset` because neither cell claimed it.

With `Above` the same stamp decides everything it names and abstains only on the left, which is spec
0001's rule 5. It renders as a space because the catalog is built from the light set alone and has
no rule mentioning `double`, exactly as in spec 0001's example of a key with no rule.

**A decided cell ignores a `Below` stamp.** A buffer of size 1 x 1 at the origin. Stamp
`(light; S, C, S, C)` with `Above`, which renders `│` and has no `Unset` arm, so rule 6 makes it
decided. Then stamp `(double; S, S, S, S)` with `Below`: nothing changes and it still renders `│`.
This is rule 7, and it is the case the unobservable requirement is about — the merge that would
produce this cell is skipped, and the cell is the same as if it had run.

**The two orders agree.** Three figures overlap at one position. Front to back they are A,
`(double; U, S, U, C)`; B, `(light; S, C, U, S)`; and C, `(heavy; C, S, S, S)`.

| Front to back with `Below` | Cell after             | Back to front with `Above` | Cell after             |
| -------------------------- | ---------------------- | -------------------------- | ---------------------- |
| A                          | `(double; U, S, U, C)` | C                          | `(heavy; C, S, S, S)`  |
| B                          | `(double; S, S, U, C)` | B                          | `(light; S, C, S, S)`  |
| C                          | `(double; S, S, S, C)` | A                          | `(double; S, S, S, C)` |

Both end at `(double; S, S, S, C)`. The base stroke is `double`, the topmost figure's, either way:
with `Below` because A defined the cell and rule 4 protected it afterwards, with `Above` because A
wrote last. The bottom arm is `Set` because C is the only figure with an opinion about it, and both
orders let that opinion through — the two figures in front of it abstain.

One position carries the property. Stamping is per position and neighboring cells are never
reconciled, which ADR-0008 records as a deliberate consequence, so a wider buffer would repeat this
comparison rather than test anything further.

**The boundaries.** A buffer at origin `(0, 0)` of size 1 x 1 holding `(light; S, C, S, C)`: a stamp
of `(heavy; S, S, S, S)` at `(1, 0)` changes nothing, with either mode, which is rule 2. And on
being decided, rule 6: `(light; S, C, S, C)` is, `(light; U, S, C, S)` is not, and neither is
`(light; S, S, S, U)` — one abstention is enough.

**The second pair.** The front end keeps the box and the pair spec 0002 settled, and draws the pair
a second time with the second box stamped `Below` instead of `Above`. The first box goes into an
empty buffer either way, so by rule 3 the mode does not affect it. Only the two positions where both
boxes have a cell differ:

| Position | First box             | Second box            | `Above`                     | `Below`                     |
| -------- | --------------------- | --------------------- | --------------------------- | --------------------------- |
| `(3, 1)` | `(light; S, U, S, C)` | `(light; U, S, C, S)` | `(light; S, S, C, S)` → `┴` | `(light; S, S, S, C)` → `├` |
| `(2, 2)` | `(light; C, S, U, S)` | `(light; S, C, S, U)` | `(light; S, C, S, S)` → `┤` | `(light; C, S, S, S)` → `┬` |

Under `Below` the only sides written are the ones the first box left `Unset`, which are exactly the
ones facing away from it, so the first box comes through whole and the second one's edges are the
ones that stop:

```text
Below:
┌──┐
│ ┌├─┐
└─┬┘ │
  └──┘
```

The junctions are what say which box is in front: `┴` and `┤` open towards the first box, `├` and
`┬` towards the second.

**The whole output.** Three blocks: the single box, the pair stamped `Above`, the pair stamped
`Below`, with the two pairs labelled. The first line of each pair is four characters and two spaces,
which this document cannot show because the repository trims trailing whitespace, so exactly:

```text
"┌──┐\n│  │\n└──┘\n\nAbove:\n┌──┐  \n│ ┌┴─┐\n└─┤┘ │\n  └──┘\n\nBelow:\n┌──┐  \n│ ┌├─┐\n└─┬┘ │\n  └──┘\n"
```

Every cell and every output above is derived from the rules and from the Light table in
[`glyph-sets.md`](../glyph-sets.md). None of it has been observed: it is what the acceptance list
checks, not the record of a run.

## Acceptance

- [ ] `cargo xtask check` passes.
- [ ] `cargo run -p monospace-cli` prints the three blocks above, with the two pairs labelled.
- [ ] `crates/monospace-cli/tests/cli.rs` asserts that whole output, trailing spaces included. The
      single box and the `Above` pair are unchanged character for character from what spec 0002
      asserted; what is added is a label line before each pair and the `Below` pair itself.
- [ ] The pair is drawn by one piece of code taking a mode, called twice. Two copies would let the
      two pictures differ for a reason other than the mode, which is the only thing the pair of them
      is there to show.
- [ ] One test per behavior rule, named after what it asserts. The tests for rules 1 to 7 read the
      buffer back through `cell`, per
      [ADR-0011](../decisions/0011-expose-cell-for-testing-stamping.md), so a fault in the catalog
      cannot fail a test whose name is about composition.
- [ ] A test for rule 8 stamps the three figures above front to back with `Below` and back to front
      with `Above`, into two buffers, and asserts the cell at the shared position is equal in both.
      It compares cells rather than rendered text, since two of the three strokes have no rule in
      the light catalog and both buffers would render as a space whether or not they agreed.
- [ ] `stamp` under `Below` returns without rebuilding the cell when the target is decided, and
      carries a comment saying why the branch is there. Nothing observable changes, so no test can
      fail for it: this item is read, not run, and it is the confirmation ADR-0017 says it is.
- [ ] Every new public item has rustdoc, and the rustdoc on `Buffer::stamp` says which side of the
      merge each mode consults.

## Open questions

**Whether the front end keeps growing a block per slice.** It prints one picture after spec 0001,
two after spec 0002 and three after this one, and every block so far has earned its place by showing
something the previous one could not. That stops being true at some point, and the point is not
visible from here. What would settle it: the first slice whose block adds nothing the others already
show, or an input format arriving, at which point the front end draws what it is given and stops
being a gallery.
