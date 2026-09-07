---
status: draft
date: 2026-09-07
---

# 0002 — Stamp below what is already there

## Why now

`stamp` has one mode and the model has two. The second one is what closes _Stamping_, and it turns
the equivalence of the two drawing orders — the property
[ADR-0008](../decisions/0008-compose-overlapping-cells-with-three-state-arms.md) rests on — from
prose into a test.

It comes before degradation and before loading glyph sets because it is the only one of the three
that touches nothing already shipped: no glyph, no catalog, no rendering. `Below` is also the mode
the layer above will reach for most, since the model names it the frequent path: stamping front to
back lets a caller stop early on cells that can no longer change, and stamping back to front does
not. Leaving it for later would mean that layer gets written against `Above` and migrated
afterwards.

## Scope

### In

- `StampMode`, with its two values, and `stamp` taking one.
- `Below`, which on an already-defined cell leaves the base stroke alone and writes only the sides
  the target has left `Unset`.
- `Cell::is_decided`, the predicate the model names as what makes stamping front to back worth
  using.
- `stamp` under `Below` leaving a decided target alone instead of rebuilding an identical cell.
- `monospace-cli` drawing two overlapping boxes twice, once with each mode, so the difference
  between them is visible on the terminal.

### Out

Every item names where it is handled instead.

- **How the buffer stores its cells.** The storage is private and no public signature mentions it,
  so replacing it changes nothing for a caller; and a dense grid would make `Buffer::new` able to
  fail, which spec 0001 promises it cannot. A later spec, once a consumer exists to give the
  measurement a workload.
- **Asking a buffer whether the cell at a position is decided.** Being decided is a property of a
  cell, and a caller holding a position reaches its cell through `cell` already. A position outside
  the window has no cell to ask about, so the question has no answer there rather than a debatable
  one. If the layer above turns out to want the shortcut, it is one method away.
- **A caller that stops early.** Skipping a figure, or a region, needs that layer above the buffer,
  which does not exist yet. This slice gives it the predicate and nothing else.
- **Erase, and reordering figures already stamped.** Out of the model itself: ADR-0008 records both
  as accepted costs of arms that cannot be undecided.
- **Everything spec 0001 left out and this one does not name** — degradation, a catalog from more
  than one set, loading sets from a file, walking a region — stays out, with the destinations that
  spec gave.

The near miss is what the front end leaves out rather than what it shows. The second box's interior
is never stamped, so the first box's cells show through it. A shape that hides what it covers is a
filled one, and filling it means stamping its interior, which needs the layer above the buffer that
does not exist yet. What the demo does show is both halves of the mechanism at once: which box owns
a side the two share, and the junction the other one leaves behind.

## Model slice

Completes _Stamping_ in [`docs/model.md`](../model.md), including _The two orders are equivalent_
and the sentence in the same section about answering cheaply whether a cell is already decided.

Nothing in _The buffer_, _Strokes, glyph sets and the catalog_ or _Rendering_ changes: this slice
adds no glyph and renders nothing that could not be rendered before.

_The cell_ is amended by this slice rather than implemented by it. It said a filled shape's border
closes its inner side and said nothing about the outer one, which left the two boxes below free to
close both and refuse every junction between them — a figure that does so renders identically on its
own, so nothing catches it until a second figure arrives. The model now asks for the outer side to
abstain, and says what closing a side a figure does not use actually means. That is the only change,
it is written before the code as the model's own header requires, and the commit that carries it
says so.

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
  caller is `monospace-cli`; the edit is mechanical and its output does not move.
- **`StampMode` derives what `Arm` derives** — `Debug`, `Clone`, `Copy`, `PartialEq`, `Eq` — for the
  same reason: it is a small field-less enum passed by value.
- **`is_decided` is on `Cell`, and only on `Cell`.** A defined cell always carries a base stroke —
  the type says so — so the model's condition, "four arms decided and a base stroke set", reduces to
  the four arms, and the answer needs nothing but the cell itself. Being decided is what a cell is,
  not what a position is: a position outside the window has no cell to ask about, and answering
  either way for it would be inventing a fact rather than reporting one.
- **`stamp` is the predicate's first caller, not a future one.** It has the target cell in hand when
  it needs the answer, so it asks the cell directly; a query taking a position would repeat the
  lookup `stamp` has just done. That keeps the predicate from being public surface with nothing but
  a test behind it, which is the cost
  [ADR-0011](../decisions/0011-expose-cell-for-testing-stamping.md) had to accept for `cell`.

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

Rules 7 and 8 are two of the three tests ADR-0008 names as carrying its decision. The third, a
segment stamped across a border, is already in spec 0001.

One requirement of this slice is deliberately not among the rules, because nothing can observe it:
under `Below`, `stamp` must recognize a decided target and leave it alone, rather than build an
identical cell and store it. The resulting buffer is the same either way, so no test can fail for
it, and this spec makes no claim about it being faster — that would need a measurement, and there is
no workload to measure yet. It is stated here because it is the reason `Cell::is_decided` exists at
all, and because the model names skipping decided cells as what makes stamping front to back worth
choosing. What keeps it from being removed later as a redundant branch is a comment at the branch
saying so; the acceptance list below is what gets it read once.

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
the target's arms, not on the stamp's. Only the top, which the target left `Unset`, is written. The
left stays `Unset`, since neither cell claimed it.

With `Above` the same stamp decides everything it names and abstains only on the left, which is spec
0001's rule 5. It renders as a space because the catalog is built from the light set alone and has
no rule mentioning `double`, exactly as in spec 0001's example of a key with no rule.

**A decided cell ignores a `Below` stamp.** A buffer of size 1 x 1 at the origin. Stamp
`(light; S, C, S, C)` with `Above`, which renders `│` and has no `Unset` arm, so rule 6 makes it
decided. Then stamp `(double; S, S, S, S)` with `Below`. Nothing changes, and it still renders:

```text
│
```

This is rule 7, and it is the case the unobservable requirement above is about: the merge that would
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

**Stamping outside the window.** A buffer at origin `(0, 0)` of size 1 x 1, with
`(light; S, C, S, C)` stamped at `(0, 0)`. A stamp of `(heavy; S, S, S, S)` at `(1, 0)` changes
nothing, with either mode.

**Deciding a cell.** `(light; S, C, S, C)` is decided, since no arm is `Unset`.
`(light; U, S, C, S)` is not, and neither is `(light; S, S, S, U)` — one abstention is enough.

**Two boxes, twice.** The command-line front end draws a 4 x 3 box at `(0, 0)`, then the same box
again at `(2, 1)`, into a buffer at origin `(0, 0)` of size 6 x 4, and renders the whole of it. The
first box goes into an empty buffer, so by rule 3 the mode does not affect it; the second is what
each run stamps differently.

The box abstains on every side that faces outwards and closes every side that faces its own
interior:

| Cell of the box     | Cell                  | Glyph |
| ------------------- | --------------------- | ----- |
| Top-left corner     | `(light; U, S, S, U)` | `┌`   |
| Top edge            | `(light; U, S, C, S)` | `─`   |
| Top-right corner    | `(light; U, U, S, S)` | `┐`   |
| Left edge           | `(light; S, C, S, U)` | `│`   |
| Right edge          | `(light; S, U, S, C)` | `│`   |
| Bottom-left corner  | `(light; S, S, U, U)` | `└`   |
| Bottom edge         | `(light; C, S, U, S)` | `─`   |
| Bottom-right corner | `(light; S, U, U, S)` | `┘`   |

This is not the box of spec 0001, which closed those outward sides rather than abstaining on them.
Drawn alone the two are indistinguishable, since a key treats `Closed` and `Unset` alike, so spec
0001's output and its example stay true of what they described. They stop being the same the moment
a second figure reaches the border: a side closed outwards refuses the junction, and two boxes drawn
that way cut holes in each other instead of joining. The model asked for the inner side of a border
to be `Closed` and said nothing about the outer one, which is the silence this slice ran into; it
now asks for both, and abstaining outwards is the same shape as spec 0001's abstaining stamp — a
cell that decides what is its and leaves the rest.

The boxes share four positions. At `(2, 1)` the first box has no cell, so the second defines it
whole in either mode; at `(3, 2)` the second has none, so the first one's `┘` stands, which is the
first box showing through an interior that was never stamped. The two that hold a cell from each
are:

| Position | First box             | Second box            | `Above`                     | `Below`                     |
| -------- | --------------------- | --------------------- | --------------------------- | --------------------------- |
| `(3, 1)` | `(light; S, U, S, C)` | `(light; U, S, C, S)` | `(light; S, S, C, S)` → `┴` | `(light; S, S, S, C)` → `├` |
| `(2, 2)` | `(light; C, S, U, S)` | `(light; S, C, S, U)` | `(light; S, C, S, S)` → `┤` | `(light; C, S, S, S)` → `┬` |

Under `Above` the second box decides every side it names and abstains outwards, so the first box's
stroke survives on the side that faces away from the second — spec 0001's rule 5. Under `Below` the
only sides written are the ones the first box left `Unset`, which are exactly the ones facing away
from it — rule 5 here. Neither box refers to the other, which is what ADR-0008's third test asks
for; spec 0001 carries that test with a segment crossing a border, and this is the same thing
between two borders.

With `StampMode::Above` the second box is in front, and the first one's edges stop against it:

```text
┌──┐
│ ┌┴─┐
└─┤┘ │
  └──┘
```

With `StampMode::Below` the first box is in front, and it is the second one's edges that stop:

```text
┌──┐
│ ┌├─┐
└─┬┘ │
  └──┘
```

The junctions are what say which box is in front: `┴` and `┤` open towards the first box, `├` and
`┬` towards the second.

The first line of each output is four characters and two spaces. This document cannot show them,
because the repository trims trailing whitespace, so the two outputs are exactly:

```text
"┌──┐  \n│ ┌┴─┐\n└─┤┘ │\n  └──┘\n"
"┌──┐  \n│ ┌├─┐\n└─┬┘ │\n  └──┘\n"
```

Both are derived from the rules above and from the Light table in
[`glyph-sets.md`](../glyph-sets.md), and neither has been observed yet: they are what the acceptance
list checks, not a record of a run.

## Acceptance

- [ ] `cargo xtask check` passes.
- [ ] `cargo run -p monospace-cli` prints the two pairs of boxes above, labelled by the mode that
      produced each. `crates/monospace-cli/tests/cli.rs` asserts both outputs, trailing spaces
      included, and replaces the assertion on the single box of spec 0001.
- [ ] The box the front end draws abstains outwards, per the table above. This replaces the cell
      table in spec 0001's box example, which stays the record of what that spec asked for: a box
      alone renders the same either way, and no test of spec 0001 changes because of it.
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
      fail for it: this item is read, not run, and rule 7's test is what keeps it honest.
- [ ] Every new public item has rustdoc, and the rustdoc on `Buffer::stamp` says which side of the
      merge each mode consults.

## Open questions

**How the unobservable requirement survives.** The branch that skips a decided target under `Below`
is required by this spec and provable by nothing in it. A comment at the branch and a reviewer are
what stand between it and a later cleanup that removes it as dead weight, and neither leaves a mark
that fails. What would settle it: a benchmark, which needs the workload the storage question is also
waiting for — the two would arrive together or not at all.

Whether the front end should show `Below` as a junction rather than as layering was asked and
answered: two overlapping boxes, which show layering only. The near miss under _Scope: out_ records
what that leaves out of the picture.
