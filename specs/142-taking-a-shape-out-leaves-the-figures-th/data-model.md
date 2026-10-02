# Data model: taking a shape out freezes what hung from it

The rule is **one method on `Shape`**, and `Diagram::remove` is five lines that call it. That is the
shape the maintainer asked for and the reason this file is not a list of `Diagram` changes: `remove`
already reaches into every figure it holds, and the knowledge of _which_ positions a figure has and
_what_ a frozen one is belongs to the figure, not to the diagram holding it.

What is new is therefore **two crate-private methods** — no new type, no new field, no new
signature, and nothing public. The four answers are on [decisions.md](decisions.md), the seven
questions that were not the maintainer's are in [research.md](research.md), the commands are in
[quickstart.md](quickstart.md), and the order is in [tasks.md](tasks.md).

## `Endpoint::frozen_position` — the rule, and the three ways to answer nothing

```rust
impl Endpoint {
    pub(crate) fn frozen_position(&self, id: &ShapeId, diagram: &Diagram) -> Option<Position> {
        match &self.at {
            Position::Reference(reference) if &reference.id == id => {
                self.at.resolve(diagram).map(Position::Absolute)
            }
            _ => None,
        }
    }
}
```

Two arms, and the second carries three of them. **All three are the rule and none of them is a
defensive check**, which is why they are here rather than in `remove`:

| It answers `None` when                 | Because                                                                            |
| -------------------------------------- | ---------------------------------------------------------------------------------- |
| the position is not a reference        | an absolute point has nothing to freeze and already resolves to itself             |
| it names **another** figure            | **D3**: a removal reaches the references naming the removed shape and nothing else |
| it names `id` and **does not resolve** | a case the removal did not create, so the removal does not repair it               |

The third is the one worth a reader's attention. Measured on this branch: a connector whose `from`
names an identity that was **never added**, asked to freeze against a removal of that same identity,
answers `None` — so the reference survives as a `Reference` and still resolves to nothing, exactly
as before. A single `if let Some(point)` guard is therefore load-bearing; without it the
implementation would freeze to nothing or drop the reference, and **either is a second rule**.

`resolve` is asked rather than `anchor` and the offset added by hand, because the offset is the gap
and the gap is what moves with the side. Measured: the demonstration's arrow freezes to `{16, 5}`,
which is `#3`'s right side centre at `{16, 3}` plus the offset `(0, 2)` the sixth picture grew.

## `Shape::with_frozen_references` — all three kinds, and why `Option`

```rust
pub(crate) fn with_frozen_references(
    &self,
    id: &ShapeId,
    diagram: &Diagram,
) -> Option<Self> {
    match self {
        Self::Box { .. } | Self::Line { .. } => None,
        Self::Connector { from, to, stroke } => match (
            from.frozen_position(id, diagram),
            to.frozen_position(id, diagram),
        ) {
            (None, None) => None,
            (new_from, new_to) => Some(Self::Connector {
                from: Endpoint {
                    at: new_from.unwrap_or_else(|| from.at.clone()),
                    ..from.clone()
                },
                to: Endpoint {
                    at: new_to.unwrap_or_else(|| to.at.clone()),
                    ..to.clone()
                },
                stroke: stroke.clone(),
            }),
        },
    }
}
```

Three decisions here, and the first two are ones the maintainer settled on 2026-10-02 rather than
ones this plan took.

**All three kinds are matched, and the two that are `None` today are matched on purpose.** A `Box`
and a `Line` hold no reference today — their own `at` is a `Pos` and not a `Position`, so the
model's restriction that only a connector's endpoint may name another figure is **the type
system's** rather than a rule someone remembers. The moment
[#89](https://github.com/andresmoschini/monospace/issues/89) widens it, each of those arms becomes a
`Position::Reference` arm, and **nothing above this match changes**. That is the whole reason the
arms are here: a `match` over a closed set of three is where the widening lands, and leaving them
out would mean the widening rewrites the method rather than two lines of it. Measured: a box and a
line both answer `None`, and so does a connector that names nothing and one that names another
figure.

**It returns `Option<Self>` rather than `Self`.** The alternative the maintainer left open was to
return the figure itself and let the caller assign unconditionally. `Option` is what this design
takes, for three reasons that are worth stating because they are not all about tidiness:

- **`None` is the ordinary answer, and the crate already has that idiom.** `Shape::anchor` answers
  `None` for a connector and `Position::resolve` answers `None` for a reference that does not
  resolve; neither is a failure and both say so in their rustdoc. A method whose whole vocabulary is
  "the references naming `id`" has an ordinary "there are none" and `None` is its word.
- **It is the shape that pays forward.** The alternative pays a clone for every figure on every
  removal, including the two that cannot change. When `#89` arrives and a box _can_ answer `Some`,
  this signature is already the one the caller handles.
- **It cannot report a rewrite that changed nothing.** A `Self` that came back equal to what went in
  is indistinguishable from one that genuinely rewrote to the same value, which is exactly the
  distinction a test wants to make about a no-op.

**Both endpoints are rebuilt together, and that is what removes the need for a slot.** A connector
may hang from the same figure at **both** ends, at different anchors with different offsets, so the
two frozen points differ. Measured on a four-by-three box at the origin, `from` on its right side at
offset `(0, 0)` and `to` on its bottom at offset `(1, 0)`:

```text
from  →  Absolute({ 3, 1 })
to    →  Absolute({ 2, 2 })
```

Two points, not one. An implementation that collected `(index, point)` and then wrote "the positions
of that figure still holding a reference to `id`" would put the first point into both ends and draw
a route nobody asked for. **This method has no such failure because it computes both answers before
it builds either endpoint** — the `(None, None)` arm is what says "this figure is not mine to
rewrite", and the two `unwrap_or_else` calls are what keep the endpoint that was not frozen exactly
as it was.

## `Diagram::remove` — five lines, and the one thing in it that is not obvious

```rust
pub fn remove(&mut self, id: &ShapeId) {
    let Some(index) = self.find(id) else {
        return;
    };

    for at in 0..self.shapes.len() {
        if let Some(shape) = self.shapes[at].shape.with_frozen_references(id, self) {
            self.shapes[at].shape = shape;
        }
    }

    self.shapes.remove(index);
}
```

### The loop indexes instead of iterating, and that is the whole design

**This is where the borrow decides the shape of the code, and it is worth being exact about it,
because D4's answer says something slightly different.**

D4 named this as its one mechanical consequence: "a reference can only be resolved while the shape
is still held, and `Position::resolve` takes `&Diagram` where `remove` holds `&mut self`. The
rewrite is therefore **two passes**". Measured on this branch, that reason does not hold:

| Form                                                             | Result                                                                                                        |
| ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------- |
| `for placed in &mut self.shapes` and write `placed.shape` inside | **`error[E0502]`** — the mutable borrow of `self.shapes` is live across the call and `self` is borrowed again |
| `for at in 0..self.shapes.len()`, one statement per figure       | **compiles**, and is the code above                                                                           |
| collect every resolution, then write them all                    | compiles, and is what a two-pass design looks like                                                            |

So **one pass is available**, and the constraint is real but narrower than D4 states it: what cannot
be done is hold a `&mut Shape` across the call to a method that wants the diagram. Indexing and
ending the borrow on each statement is enough. The first row is the form that fails, and it is the
form the maintainer's sketch is closest to — worth naming so the implementer meets it once and moves
on.

**Nothing about the shape being removed is excluded from the loop, and nothing needs to exclude
it.** A figure that is about to be removed is rewritten and then dropped. Excluding it would cost a
comparison and buy nothing: the work is discarded with the shape a line later.

**`index` is read before the loop and is still valid after it**, because the loop changes no
figure's place in the order and no figure's identity. That is the one thing in the body a reader has
to take on trust, and it wants a comment.

## What the freeze is not

Four things follow from the two methods without being written anywhere, and each is measured on this
branch rather than argued:

- **A position pointing at some other shape is untouched.** One connector with `from` naming a box
  that stays and `to` naming a box taken out comes back with `from` **still a `Reference`** and `to`
  frozen. The asymmetry is the rule, and it is the whole of what distinguishes D3's answer from
  "rewrite every reference naming the identity".
- **A chain is not walked.** A reference can only name a figure that answers an anchor, and a
  connector answers none, so the reference being frozen is always a reference to a box or a line.
  One link is the most there is (ADR-0041).
- **A figure put back under the removed identity is not re-hung.** Nothing went looking for the
  reference a frozen end was, and a frozen end is a point that no longer names anything.
- **`find` answers the first match, so with two shapes under one identity the freeze resolves
  through the first.** `add_under` allows that and this is pre-existing: `Position::resolve` already
  goes through `get`, which goes through `find`. The freeze inherits it rather than deciding it.

## The pictures the rule draws

Both are drawn rather than drawn by hand: the methods were implemented as above, run, and the tree
reverted. The ten cells, the twelve and the "byte for byte" are what the run said.

**The specification's arrangement**, at `window(12, 3)`: the box four by three at the origin, the
box three by three at `{8, 0}`, and a connector hanging from the first box's right side.

As written, and then with that box taken out:

```text
┌──┐    ┌─┐
│  ├────│ │
└──┘    └─┘

        ┌─┐
   ─────┤ │
        └─┘
```

The middle row's `{3, 1}` reads `─` rather than `├`, because the box's `│` used to compose with the
arrow's arm into one glyph and the arm now stands alone; a cell carrying one arm renders as the run
through it. This is the picture `spec.md` B1.1 draws by hand, and the measurement is what corrected
the specification's own count from `15 − 6` to ten, nine of them blank ([research.md](research.md)
Q7). Putting the box back under the same identity with `add_under` reproduces the left-hand picture
**byte for byte** — the frozen point is the point the reference was resolving to, so the same
composition recurs — and putting it back **displaced** leaves the arrow where it was rather than
hanging from it, which is P3's cost and B1.2's caveat.

**The gallery's fourth block**, at `window(8, 3)`, from the arrangement that test already holds —
the box and the connector, with no figure beside them:

```text
┌──┐
│  ├────
└──┘


   ─────

```

**And the window does not change**, which is the measurement worth having. The third block beside it
needed `window(8, 4)` because the displaced connector landed on the fourth row; a removal removes
rows rather than adding them, and the fourth row of `window(8, 4)` came back empty in the run. So
the fourth block asks for the same `window(8, 3)` its two siblings do, and a snapshot that grew a
fourth row of nothing is the signature here of a window widened for no reason.

## The demonstration's seventh picture, by value

The shipped demonstration's third entry is `#3`, a four-by-three box at `{9, 2}` that the fifth
picture displaces four cells right into `x 13..16, y 2..4`. At the sixth picture the arrow `#10`
holds **two** references: the `from` the demonstration rehung to `#3`'s right side with offset
`(0, 0)`, which `side_centre` answers `{16, 3}` and the sixth picture's displacement grows to offset
`(0, 2)` — so it resolves to `{16, 5}` — and the shipped `to`, which names `#5`'s bottom with offset
`(1, 1)`, grown to `(1, 3)` and resolving to `{22, 6}`.

The seventh step is one call, `diagram.remove(&the_hung_from)`, and it reaches **one** of those two:

| Its endpoint | Names                    | After the seventh                                         |
| ------------ | ------------------------ | --------------------------------------------------------- |
| `from`       | `#3`, which is taken out | **`Absolute({ 16, 5 })`** — the point it was resolving to |
| `to`         | `#5`, which stays        | **unchanged**, still `Reference { #5, Bottom, (1, 3) }`   |

Measured on this branch with the step added: a bare run prints **seven** captioned pictures, the
sixth and the seventh differ in **exactly twelve cells**, all twelve of them the whole
`x 13..16, y 2..4` rectangle and **all twelve blank** in the seventh, and the first six are byte for
byte what the run prints today. The arrow's ten cells are untouched — which is not a small claim but
the claim the whole slice is named after: the arrow's footprint is neither contiguous nor a
rectangle, so nothing that reads it off a picture gets it right, and it is why the count is asserted
against a baseline in [quickstart.md](quickstart.md) rather than quoted.

## What does not change

Nothing below is touched, and the list is the shape of the slice:

- **`Position`, `Reference`, `Endpoint` and `Shape`** — a frozen end is an existing variant holding
  an existing value. `Endpoint` gains two crate-private methods and nothing else. No new name for a
  frozen position is introduced anywhere, which is D2's condition and the reason §4 gains a sentence
  rather than a term.
- **`resolve`, `displaced_by` and `Delta`** — a displacement is the other half and is reached from
  the other side. `resolve`'s two `?`s still come before the offset is added, so a reference that
  resolves to nothing is still nothing however large its offset.
- **`Diagram`'s whole surface** — `add`, `add_under`, `get`, `replace`, `forward`, `backward`,
  `draw`, `numbered_from`. `remove`'s signature is unchanged too; what changes is that a caller can
  no longer get the old behavior by asking for it.
- **§9's table of five** — no row, because the rewrite is inside `remove` rather than a change of
  its own, which is D4's answer. `remove`'s row and its rustdoc are what move.
- **`assets/demo.json` and the description format** — the seventh picture is a change
  `monospace-cli` makes in its own code, beside the six it already makes (ADR-0035, B3.2).
- **`monospace-core`** — nothing in either method reaches it beyond the `Pos` the crate already
  re-exports and `resolve` already returns, and `remove` was already in this crate.
- **Every tracked picture** — 1916 renderings across 16 characterization files, none of which can
  express a removal, so the gallery snapshot is the only picture one change can move
  ([research.md](research.md) Q6).

## The derived arrangements, stated rather than discovered

The specification's _Edge cases_ names two arrangements the rule produces. Both are **derived from
the rule, not decided by it**, and both are pinned so a later slice that changes either has to say
so:

- **Two connectors from one figure.** Each freezes at the point its own end was resolving to — P1
  applied twice rather than a cascade, because the loop visits each figure once and each connector
  is its own figure.
- **One connector with both ends on the same figure.** Two different points, measured above, and the
  reason `Shape::with_frozen_references` computes both before it builds either.
