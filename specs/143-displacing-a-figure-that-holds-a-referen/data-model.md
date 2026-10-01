# Data model: displacing a figure that holds a reference grows its offsets

The whole of the model is one addition and one changed arm. Neither is a new type, a new field or a
new signature, which is why there is no entity table here: the specification names the shapes, §4 of
[`docs/diagram-model.md`](../../docs/diagram-model.md) states the rule, and this file is what the
rule looks like as code.

The three answers it takes are on [decisions.md](decisions.md), named beside each; the six questions
that were not the maintainer's are in [research.md](research.md); and the commands that measure them
are in [quickstart.md](quickstart.md).

## `Delta` — a second way to add one delta to another

```rust
impl Delta {
    /// Grows this delta by `by`, one axis at a time.
    ///
    /// The second of the two additions in this crate, and it saturates for the reason `apply`
    /// does: a wrapped amount could land inside a window a caller could really hold, and a
    /// saturated one cannot. It keeps no memory, so a displacement back does not undo a
    /// saturating one.
    pub(crate) fn grow(self, by: Delta) -> Delta {
        Delta {
            dx: self.dx.saturating_add(by.dx),
            dy: self.dy.saturating_add(by.dy),
        }
    }
}
```

| Field  | Type  | Meaning                                                     |
| ------ | ----- | ----------------------------------------------------------- |
| `dx`   | `i32` | How far the figure moves right. Negative moves it left.     |
| `dy`   | `i32` | How far the figure moves down. Negative moves it up.        |
| `self` | `i32` | How far the reference stands from its side on the same axes |

Two `Delta`s in and one `Delta` out, `i32` on both axes and `Delta` on the axis. It is the only new
item in the slice, and research.md Q5 says where it sits and why: beside `apply`, in `delta.rs`,
because `delta.rs:43` calls itself "the only arithmetic in this crate" and putting a second addition
in `position.rs` would make that sentence false in a second place rather than in one.

The three alternatives Q5 weighed, none of which changes the model:

- **A public `Delta::add`.** Rejected because nothing outside the crate needs it. The altitude test
  asks whether anything outside can observe it, and a caller adding two deltas has no consumer here.
- **An `Add` impl.** Rejected for the reason `apply` already gives: the workspace has no `Add` over
  a position anywhere, and adding one is a decision this crate does not need to take.
- **Routing through `Pos`.** Rejected because it would borrow the core's type for a job the core has
  nothing to do with.

`Delta`'s derives do not change. It was `Copy` because a caller applies one delta to as many figures
as it likes, and this makes a delta compose with another without cloning either.

## `Position::displaced_by` — the reference arm, grown

```rust
pub(crate) fn displaced_by(&self, by: Delta) -> Self {
    match self {
        Self::Absolute(at) => Self::Absolute(by.apply(*at)),
        Self::Reference(reference) => Self::Reference(Reference {
            id: reference.id.clone(),
            anchor: reference.anchor,
            offset: reference.offset.grow(by),
        }),
    }
}
```

The `Absolute` arm is untouched — it is 082's rule and this slice does not change it. The
`Reference` arm replaces a `clone()` with three field moves, and **two of the three move nothing**:

| The reference's field | What the displacement does to it | Why                                                          |
| --------------------- | -------------------------------- | ------------------------------------------------------------ |
| `id: ShapeId`         | unchanged, cloned                | It names **which** figure the position hangs from, not where |
| `anchor: Anchor`      | unchanged                        | Which of its four sides, also not where                      |
| `offset: Delta`       | **grows by `by`**                | The gap from that side, in cells, on each screen axis        |

`ShapeId` is a `String`, which is why `Reference` is not `Copy` and why the arm clones rather than
copies — the reason is unchanged and is not this slice's to fix.

Three properties fall out of that rather than being written:

- **A displacement builds a value and cannot fail**, so a `Reference` naming a shape the diagram
  does not hold comes back the same reference with a larger offset, and it still resolves to nothing
  when the figure is drawn. There is no error path to add and no report to remove (SC-003).
- **The offset saturates**, so an offset already at the end of the window's coordinates stays there
  and a later displacement back does not return it — `grow` keeps no memory. An absolute position
  that saturates draws nothing and an offset that saturates draws a very far away endpoint, and that
  is the one asymmetry between the two (the specification's _Edge cases_).
- **A displacement of nothing returns the reference equal to itself**, which is what the widened
  derives are for and what `a_figure_displaced_by_nothing_comes_back_equal_to_itself` already pins.

## `Shape::displaced_by` — unchanged in behavior, and now says so

`Shape::displaced_by` keeps its signature, its three arms and its `&self -> Self`, and **all three
arms already displaced every position they hold** — a `Box` and a `Line` their own `at`, a
`Connector` `from.at` and `to.at` together. 082 established that, and this slice changes nothing
about it.

What changes is one paragraph of its rustdoc, which is the fourth of the four places Q6 measured
that declare the no-op this slice reverses. It is the one worth naming twice over:
`Shape::displaced_by` is `pub` where `Position::displaced_by` is `pub(crate)`, so it is the
paragraph a caller reads first, and it is the only one of the four that asks the question rather
than answering it — it ends on "what displacing such a figure should mean in general is #143's to
settle". The other three are `resolve`'s doc at `position.rs:110-113`, `displaced_by`'s own doc at
`position.rs:126-138`, and the test `a_displacement_moves_a_point_and_leaves_a_reference_alone` at
`position.rs:428`, which is rewritten rather than deleted because "a reference comes back equal to
itself" is still true and only its reason is wrong.

## What does not change

Nothing below is touched, and the list is the shape of the slice:

- **`Reference`'s three fields** — `offset` is already a `Delta` and grows where it stands.
- **`resolve`** — a displacement never reached it, and it does not now. Both `?`s still come before
  the offset is added, so a reference that resolves to nothing is still nothing.
- **`Diagram`'s whole surface** — `add`, `add_under`, `get`, `replace`, `remove`, `forward`,
  `backward`, `draw`, `numbered_from`.
- **`assets/demo.json` and the description format** — the sixth picture is a change `monospace-cli`
  makes in its own code, beside the four it already makes, so no field is added and no marker is
  regenerated (B3.2, SC-004).
- **`monospace-core`** — the core has no displacement at all (`displaced_by` appears nowhere under
  `crates/monospace-core`, Q3), so nothing here reaches it. `Delta::grow` is `i32` arithmetic on the
  crate's own fields.
- **Every tracked picture** — 1916 renderings across 16 characterization files, none of which can
  express the change (Q3).

## The two derived arrangements, stated rather than discovered

The specification's _Edge cases_ names two arrangements the rule produces and no caller can reach.
Both are **derived from the rule, not decided by it**, and both are pinned so a later slice that
changes either has to say so:

- **The box down two, then the connector down two.** Each displacement belongs to one figure and
  both figures moved, so the connector ends four down with its gap grown by two. Nothing moves an
  anchor and its holder together and keeps the gap — displacing the box carries the endpoint, and
  displacing the connector grows the gap.
- **A saturated offset.** `grow` saturates and keeps no memory, so an offset that has reached the
  end of the window's coordinates stays there and a displacement back does not return it.

## The two pictures the rule yields

For the arrangement below — a box four cells by three at the origin, and a connector whose `from` is
a reference to that box's right side carrying an offset of nothing, leaving rightward, with its `to`
at `{7, 1}` leaving leftward — displacing the **connector** gives two pictures. Both were drawn by
building the values the rule yields, not by hand (research.md Q2, reproduced below).

**Down two cells.** Rigid translation, no bend, and the box where it was:

```text
┌──┐
│  │
└──┘
   ─────
```

**Right four cells.** The endpoint stands four cells right of the border the box still draws, and
the gap grew by four — a displacement is a property of one figure, and which figure that is has to
be visible from the picture:

```text
┌──┐
│  │   ─────
└──┘
```

Both contrasts are against the same arrangement as written, which draws `│  ├────` — one row with
the arrow welded to the border. And displacing the **box** four cells right is 083's rule and its
snapshot, unchanged: the endpoint follows the side it hangs from and the gap does not change.

## The demonstration's sixth picture, by value

The shipped demonstration's tenth entry is the arrow. At the sixth picture it holds **two
references**, which is why the step is a no-op today (research.md Q1, measured) and why the sixth
picture comes out byte for byte the fifth:

| Its endpoint | The reference it holds             | Where it stands | Where it stands after `dy: 2` |
| ------------ | ---------------------------------- | --------------- | ----------------------------- |
| `from`       | `#3`'s right side, offset `(0, 0)` | `{16, 3}`       | `{16, 5}`                     |
| `to`         | `#5`'s bottom, offset `(1, 1)`     | `{22, 4}`       | `{22, 6}`                     |

So the sixth picture is the fifth with the arrow two rows lower and **both boxes standing exactly
where they stood** — `#3` on `x 13..16, y 2..4` and `#5` on `x 20..23, y 1..3`. That is SC-004, and
the whole cost of it is one `displaced_by` call on `#10` and one caption line.

The first five pictures come out byte for byte what they are today, and a path still prints one
picture and nothing else, because the sixth is a step in `monospace-cli`'s own code rather than a
field in the format (B3.2, B3.3).

## The gallery's third block

D2's answer is one more `block()` in `an_endpoint_hangs_from_a_side_and_follows_it`, beside the two
it holds, and the specification is explicit that the third is **drawn by `displaced_by` rather than
built by hand** — so a rule that stops holding drops a snapshot instead of nothing (B3.4).

Two things about it were measured rather than assumed, and both are in _Design_:

- It is reached from **the arrangement as written**, through a second `Diagram` in the same test,
  because the two blocks beside it leave both of the connector's endpoints on one cell and a third
  that displaced it would draw nothing.
- It asks for **`window(8, 4)`**, because the displaced connector lands on the fourth row and
  `window(8, 3)` clips it out entirely.

With both, the block is the down-two picture above, drawn by the code this slice changes. The
snapshot grows a four-row picture and sixteen surface rows, and **its two existing blocks are byte
for byte what they are**.
