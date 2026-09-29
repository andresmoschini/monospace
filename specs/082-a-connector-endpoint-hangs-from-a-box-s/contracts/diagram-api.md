# Contract: what `monospace-diagram` exposes after this slice

The public surface after feature 082. It is feature 081's
[`diagram-api.md`](../../081-a-shape-can-be-removed-and-replaced/contracts/diagram-api.md) with
three types added, one field's type changed, one conversion removed and one method added; that file
stays the record of what 081 shipped, and this one is what a caller sees now. Everything here
carries rustdoc as it is written.

Field-by-field meaning is in [data-model.md](../data-model.md); why the types are shaped this way is
in [research.md](../research.md), and the five answers in [decisions.md](../decisions.md) are named
beside each below.

## `Anchor` — new

```rust
/// One of the four sides of a figure a position can name.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anchor {
    Top,
    Right,
    Bottom,
    Left,
}
```

Four and no fifth: the model's §5 names the four side centers, and the other five of issue #62's
nine are [#90](https://github.com/andresmoschini/monospace/issues/90)'s, waiting for a consumer that
wants a corner or a center.

It is not `monospace_core::Direction`, which names which way a figure leaves rather than where it
is. The two coincide for a side today, and one name would mean the wrong thing the day a corner
arrives — a corner has two sides and no single direction out of it. `Copy` is `Pos`'s derive and the
enum holds nothing, so one can be passed and stored without cloning it.

## `Reference` — new

```rust
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reference {
    /// The identity of the shape the position hangs from.
    pub id: ShapeId,
    /// Which of that shape's four sides it hangs from.
    pub anchor: Anchor,
}
```

Both fields are public, as `Delta`'s are, so a caller names a reference as a value. `ShapeId::new`
builds the identity it holds, and `ShapeId::from(add)` is the one the diagram issued (D3).

- **Two fields, not three** (D1). §1 and §4 of the model both describe a reference as an identity,
  an anchor and two offsets, and the offsets arrive with
  [#83](https://github.com/andresmoschini/monospace/issues/83). This is the first two of the three,
  so a caller who wants an endpoint two cells off a side has no way to say it and places both
  figures by hand. Adding the third later is additive and moves no caller; the expensive direction
  would be amending the model down to two.
- **It is not `Copy`**, because `ShapeId` is a `String`. `Clone` is enough for every use here, and
  `Position` inherits the same.

## `Position` — new

```rust
/// Where something stands: a point, or a reference to a side of a shape.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Position {
    Absolute(Pos),
    Reference(Reference),
}

impl From<Pos> for Position {
    /// Wraps a point as a position that never fails to resolve.
}
```

`From<Pos>` is the second way in, for a caller holding a point rather than a position — the same
shape of thing as `From<(i32, i32)> for Delta`. It builds one and does nothing else.

- **`resolve` is public, and it is the only reader a reference gets** (D5):

  ```rust
  /// Where this position stands now, or `None` when it does not resolve.
  #[must_use]
  pub fn resolve(&self, diagram: &Diagram) -> Option<Pos>;
  ```

  A caller that knows a `Position` and the `Diagram` it belongs to can ask where it stands, rather
  than redraw the diagram to find out. It is `#[must_use]` because it answers and changes nothing.

- **Nothing that resolves is an error.** An identity the diagram does not hold and an anchor the
  kind does not answer are the same answer, `None`, and the diagram is unchanged. This is the
  model's §4 applied to a case it describes but the code had not reached.
- **There is no offset arithmetic, because there are no offsets** (D1). What #83 adds to `Reference`
  is what makes it worth adding here.

## `Shape` — changed

### `Endpoint.at` changes type

```rust
pub struct Endpoint {
    /// The endpoint's position. The terminal occupies this position itself.
    pub at: Position,
    /// The direction the connector leaves this endpoint in.
    pub leaving: Direction,
    /// What this endpoint contributes to the cell at `at`.
    pub terminal: Terminal,
}
```

**This is a breaking change to a public field**, and it is the only one in the crate. `Box.at` and
`Line.at` stay `Pos` on purpose: ADR-0041's restriction — only a connector's endpoint may hold a
reference, for now — is then the type system rather than a rule to get right at run time (D2). A
caller matching on `Endpoint` has to be recompiled; a caller matching on `Box` and `Line` does not.

`leaving` and `terminal` are untouched, and they stay the caller's rather than the anchor's: a
reference names a side and does not decide which way the connector leaves it or what it ends in (§6
_Attachment_). Deriving the direction from the side is
[#89](https://github.com/andresmoschini/monospace/issues/89).

### `From<Endpoint> for monospace_core::Endpoint` is gone

The core's `Endpoint.at` is a `Pos` and a `Position` has none to give it without a diagram, so the
conversion had nothing to convert to. It was never part of the diagram's own contract — it reached
into the core's type — and its replacement is four lines where the connector is drawn.

A consequence worth naming: **the diagram's endpoint can no longer be turned into the core's.**
Anything the core were one day to grow for an attached endpoint has to be reached through drawing,
which is the argument 081's D1 turned on and the reason the core still gains nothing here.

### `Shape::anchor` is not here

```text
NOT public:  pub(crate) fn anchor(&self, anchor: Anchor) -> Option<Pos>
```

A shape's four side centers exist to be resolved _through_, and `Position::resolve` is how. Nothing
outside the crate asks a figure for an anchor, and the anchors' only consumer today is drawing. The
direct query is
[the specification's first consumer that needs an anchor without drawing](../spec.md#what-this-slice-does-not-decide)
— and note that a caller can already reach the number: holding a `Position` and the diagram,
`resolve` answers it.

`Shape` gains no method beyond the two 081 gave it, and its derives are unchanged. The `Connector`
arm of the drawing method is where a reference reaches a picture, and that method is `pub(crate)`
and stays so.

## `Diagram` — changed in behavior, not in surface

`new`, `add`, `forward`, `backward`, `get`, `remove`, `replace` and `draw` keep the signatures 081
recorded, and `ShapeId` is unchanged. One of them now does more:

- **`draw` resolves what a shape stands at before it writes it.** Nothing about the signature
  changes and nothing new is asked of the caller; what changed is that a connector with an endpoint
  whose reference does not resolve is **not drawn at all**, and every other shape in the diagram
  draws exactly what it drew. One endpoint that does not resolve takes the whole connector out,
  because a connector composes two positions and belongs to neither.
- **`get` reaches further than it did**, as the query a reference resolves through, and returns
  exactly what it returned before.

**No sixth verb, and no seventh.** A connector now follows a shape because the reference says so,
not because the diagram was told to move it. §9 still counts five changes.

## What is still not here

- **No offsets on a reference**, and so no way to hang an endpoint off a side and land it beside the
  side. [#83](https://github.com/andresmoschini/monospace/issues/83).
- **No anchor beyond the four sides**: no corner and no center.
  [#90](https://github.com/andresmoschini/monospace/issues/90).
- **No `Shape::anchor` and no method that attaches an endpoint.** A caller names a `Reference` and
  puts it in the field; there is no `Shape::attached_at(which, position)` beside it, because a
  caller that can match on `Shape`'s public fields already has the four lines it would be.
- **No way to ask which shapes a diagram could not draw.** A reference that does not resolve and a
  diagram that drew nothing look the same from outside. That is the cost
  [ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md) records, and
  [#88](https://github.com/andresmoschini/monospace/issues/88) is where it is answered.
- **No `ids()`, no `len`, no order**, and no way to ask which shape decided a position.
- **No undo and no history**: `remove` hands back nothing, and putting a figure back means building
  it again.
- **No way for a caller to choose which shape gets an identity**, or to edit one. `ShapeId::new`
  spells an identity a caller wants; the diagram still issues them, and a caller that spells one has
  to put it under a name nothing can be added to. §11's own trigger is the first slice reading a
  diagram from a file, and [#86](https://github.com/andresmoschini/monospace/issues/86) is where the
  listing beside it is.
- **No reference in the description format.** `monospace-cli`'s file format is unchanged and gains
  no field: `at` is still a point, and a caller wanting a reference builds the picture in code, the
  way the shipped demonstration does.
- **Nothing in `monospace-core`**, and nothing in `Position` that reaches for it: the four side
  centers are computed from fields the core already publishes (`BoxShape` and `Line` expose `at`,
  `size`, `len` and `orientation` as `pub`).

Two of these are the rules this slice accepts with nothing to verify them, named as such per
principle IV rather than described as tested: displacing a figure that holds a reference is a silent
no-op, and a shape whose own position does not resolve offers no anchor point. The first is B4 and
is tested by drawing; the second is unreachable, because no position but an endpoint's may hold a
reference — that is [#89](https://github.com/andresmoschini/monospace/issues/89).

## Using it

```rust
use monospace_core::{Buffer, Direction, GlyphCatalog, Pos, Size, Stroke, Terminal, render};
use monospace_diagram::{Anchor, Delta, Diagram, Endpoint, Position, Reference, Shape};

let origin = Pos { x: 0, y: 0 };
let size = Size { width: 20, height: 6 };

let mut diagram = Diagram::new();

// The box the arrow hangs from, named by the identity `add` handed back rather than written out:
// only the demonstration, whose identities come from a file it cannot read back, spells one.
let box_id = diagram.add(Shape::Box { at: origin,
                                      size: Size { width: 4, height: 3 },
                                      stroke: Stroke::from("light"),
                                      fill: None });

// The arrow, with one endpoint on that box's right side. Where that lands is the diagram's
// arithmetic and not the caller's: nothing above says the right center is `{3, 1}`.
diagram.add(Shape::Connector {
    from: Endpoint { at: Position::Reference(Reference { id: box_id, anchor: Anchor::Right }),
                     leaving: Direction::Right,
                     terminal: Terminal::Arm },
    to: Endpoint { at: Pos { x: 10, y: 1 }.into(),
                   leaving: Direction::Left,
                   terminal: Terminal::Arm },
    stroke: Stroke::from("light"),
});

// Moving the box takes the arrow's hanging end with it and leaves the other end where it was.
// The endpoint on the side does not move; the absolute one does. Drawing again is how any of it
// is seen, and a shape that does not resolve draws nothing rather than leaving a hole.
if let Some(moved) = diagram.get(&box_id).map(|shape| shape.displaced_by(Delta { dx: 2, dy: 0 })) {
    diagram.replace(&box_id, moved);
}

let mut buffer = Buffer::new(origin, size);
diagram.draw(&mut buffer);
let text = render(&buffer, &GlyphCatalog::light(), origin, size);
```

A caller that wants to know where an endpoint stands without drawing does it the same way the
drawing does:

```rust
let at = endpoint_position.resolve(&diagram);   // Option<Pos>, and None is an ordinary answer
```

and one that wants to know whether the arrow is on the picture at all has no way to ask — see
[#88](https://github.com/andresmoschini/monospace/issues/88) above.
