# Data model: taking a shape out freezes what hung from it

The whole of the model is one method body. No new type, no new field, no new signature, and **no new
method at all** — which is why there is no entity table here: the specification names the
arrangement, §4 of [`docs/diagram-model.md`](../../docs/diagram-model.md) states the rule, §9 grows
no row, and this file is what the rule looks like as code.

The four answers it takes are on [decisions.md](decisions.md), named beside each; the seven
questions that were not the maintainer's are in [research.md](research.md); and the commands that
measure them are in [quickstart.md](quickstart.md).

## `Diagram::remove` — two passes, and why there are two

```rust
pub fn remove(&mut self, id: &ShapeId) {
    let Some(index) = self.find(id) else {
        return;
    };

    // Pass one reads and collects; pass two writes.
    let mut frozen: Vec<(usize, usize, Pos)> = Vec::new();
    for (at, placed) in self.shapes.iter().enumerate() {
        let Shape::Connector { from, to, .. } = &placed.shape else {
            continue;
        };
        for (slot, endpoint) in [(0, from), (1, to)] {
            if let Position::Reference(reference) = &endpoint.at
                && &reference.id == id
                && let Some(point) = endpoint.at.resolve(self)
            {
                frozen.push((at, slot, point));
            }
        }
    }
    for (at, slot, point) in frozen {
        if let Shape::Connector { from, to, .. } = &mut self.shapes[at].shape {
            if slot == 0 {
                from.at = Position::Absolute(point);
            } else {
                to.at = Position::Absolute(point);
            }
        }
    }

    self.shapes.remove(index);
}
```

Three things in that body are not free, and each is a fact rather than a taste.

**The two passes are forced by the borrow, not chosen for clarity.** `Position::resolve` takes
`&Diagram` and answers by asking the diagram for the figure a reference names — so a reference can
only be resolved while the figure is still held — while `remove` holds `&mut self`. D4 named this as
the one mechanical consequence to meet in the plan rather than in the compiler, and it is the whole
reason the body is two loops. Nothing in the slice needs a `Cell`, a `RefCell` or an interior
mutability, and nothing may reach for one: the same rule would then have to hold for every other
method that borrows the diagram while writing to it.

**The `slot` is not optional, and it was measured rather than argued.** A `Placed` is one figure and
one identity, and a connector may hang from the same figure at **both** ends — at different anchors,
with different offsets, so at two different points. Collecting `(index, point)` and writing "the
positions of figure `index` that still hold a reference naming `id`" would put the first end's point
into both ends. Measured on this branch with both ends naming one four-by-three box at the origin,
`from` on its right side with offset `(0, 0)` and `to` on its bottom with offset `(1, 0)`:

```text
from  →  Absolute({ 3, 1 })      to  →  Absolute({ 2, 2 })
```

which is two points, not one, and the wrong pairing draws a route nothing asked for. The triple is
what the second pass needs and no more.

**`Some(point)` is what leaves an unresolved reference alone**, and it is D3's answer rather than a
defensive check. Three of the five guards in that body are the rule and two are the arithmetic:

| Guard                                                   | What it carries                                                                |
| ------------------------------------------------------- | ------------------------------------------------------------------------------ |
| `let Some(index) = self.find(id) else { return }`       | A removal of an identity the diagram does not hold changes nothing at all      |
| `let Shape::Connector { from, to, .. } = &placed.shape` | Only a connector's endpoint can hold a reference — the type system, not a rule |
| `&reference.id == id`                                   | **D3**: the references naming the removed shape, and no other                  |
| `endpoint.at.resolve(self)`                             | A reference that resolves to nothing is not frozen, and not destroyed          |
| `Position::Absolute(point)`                             | What a frozen end becomes, and there is no third variant for it                |

The fourth guard is the one worth a reader's attention. Measured on this branch: a connector whose
`from` names an identity that was **never added**, in a diagram where a removal naming that same
identity happens, comes back afterwards still holding
`Reference { id: #9, anchor: Right, offset: (0, 0) }` — the removal did not create that broken
reference and does not repair it. D3's answer in the maintainer's own terms is that a removal
freezes **the references naming the removed shape and nothing else**, and a case the removal did not
create is not one it changed. A single `if let Some(point)` guard is therefore load-bearing: without
it the implementation would freeze to nothing, or drop the reference, and either is a second rule.

## What the freeze is not

Four things follow from the body above without being written anywhere, and each is measured on this
branch rather than argued:

- **A position pointing at some other shape is untouched.** One connector with `from` naming a box
  that stays and `to` naming a box that is taken out comes back with `from` **still a `Reference`**
  and `to` frozen to the point it was resolving to. The asymmetry is the rule, and it is the whole
  of what distinguishes D3's answer from "rewrite every reference naming the identity".
- **A chain is not walked.** A reference can only name a shape that answers an anchor, and a
  connector answers none, so the reference being frozen is always a reference to a box or a line.
  The freeze reaches one link and cannot reach further (ADR-0041).
- **The figure being taken out may itself hold references, and they are frozen before it is
  dropped.** Nothing excludes it from the first pass and nothing needs to: it is a connector, it is
  removed a few lines later, and the work spent on it is discarded with the shape. Excluding it
  would cost a comparison to buy nothing.
- **`find` answers the first match, so with two shapes under one identity the freeze resolves
  through the first.** `add_under` allows that, and this is pre-existing: `Position::resolve`
  already goes through `get`, which goes through `find`. The freeze inherits it rather than deciding
  it.

## The pictures the rule draws

Both are drawn rather than drawn by hand: the freeze was implemented as the body above, run, and the
tree reverted. The ten cells, the twelve and the "byte for byte" are what the run said.

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
fourth row of nothing is the signature here — not the fifth row of nothing that was 143's.

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
  an existing value. No new name for it is introduced anywhere, which is D2's condition and the
  reason §4 gains a sentence rather than a term.
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
- **`monospace-core`** — nothing in the body reaches it beyond the `Pos` the crate already
  re-exports and `resolve` already returns, and `remove` was already in this crate.
- **Every tracked picture** — 1916 renderings across 16 characterization files, none of which can
  express a removal, so the gallery snapshot is the only picture one change can move
  ([research.md](research.md) Q6).

## The derived arrangements, stated rather than discovered

The specification's _Edge cases_ names two arrangements the rule produces. Both are **derived from
the rule, not decided by it**, and both are pinned so a later slice that changes either has to say
so:

- **Two connectors from one box.** Each freezes at the point its own end was resolving to — P1
  applied twice rather than a cascade, because the two connectors are two entries in the order and
  the first pass visits each of them once.
- **A figure put back under the removed identity.** As a kind that answers the side, it composes
  with the frozen arm byte for byte; displaced, or as a kind that answers no anchor, it leaves the
  arrow where it was. **Nothing re-attaches it**, so there is nothing to go looking for the
  reference it was.
