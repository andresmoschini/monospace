# Contract: what `monospace-diagram` exposes after this slice

The public surface after feature 148. It is feature 083's
[`diagram-api.md`](../../083-a-reference-carries-a-horizontal-and-a-v/contracts/diagram-api.md) with
two methods added and one rustdoc rewritten; that file stays the record of what 083 shipped, and
this one is what a caller sees now. Everything here carries rustdoc as it is written.

Field-by-field meaning is in [data-model.md](../data-model.md); why the counter is kept the way it
is and why the two new fields are called what they are is in [research.md](../research.md), and the
four answers in [decisions.md](../decisions.md) are named beside each below.

## `Diagram` — two methods

```rust
impl Diagram {
    /// An empty diagram whose next `add` takes the ordinal `next`.
    #[must_use]
    pub fn numbered_from(next: u32) -> Self;

    /// Puts `shape` at the front of the order, under the identity `id`.
    pub fn add_under(&mut self, id: ShapeId, shape: Shape);
}
```

A caller may now choose the identity a shape is added under, and may say where a diagram's numbering
resumes. That is the whole of the addition, and it is what §11's _Can a caller choose an identity?_
asked for — the question this slice closes, not a new question.

- **`numbered_from` seeds the counter, not the shapes.** It returns an empty diagram; the parameter
  is **the ordinal the next `add` takes**, so `numbered_from(27)` means the next addition is handed
  `#27`. Storing the ordinal itself rather than the last issued one is deliberate (Q1): the reverse
  convention is one subtraction away from being wrong in a public method, and there would be nothing
  in the signature to catch it.
- **`new()` is unchanged, and agrees with it.** `add` increments before use, so `new()` and
  `numbered_from(1)` hand back `#1` from their first addition and differ in nothing observable.
  `new` stays the answer for a caller that has no opinion.
- **`add_under` is `add` with the caller's name in place of the diagram's.** It puts the shape at
  the same place in the order — a new `Placed` pushed onto the back, which is the front of the order
  — and it hands back nothing.
- **It cannot fail, and that is a property rather than an accident.** Two shapes may carry one
  identity: both are held, the first is what `get`, `remove`, `replace`, `forward` and `backward`
  find, and the second is reachable by no identity at all until the first is taken out. There is no
  error, no report and no panic, which is why the signature has no `Result` and no `Option` (Q2).
- **It does not touch the counter.** Nothing a caller writes moves the numbering: on a diagram
  seeded at 3, `add_under(ShapeId::new("#7"), shape)` leaves it at 3 and the next `add` is `#4`
  (D2). A file says where its numbering resumes, and the field that says so is trusted rather than
  checked — the specification's _Edge cases_ names the stale-counter case as an accepted cost, and
  this contract carries it as a promise the crate deliberately does not make.
- **`get` and the four other changes answer to the chosen name**, because they all go through one
  search comparing `ShapeId`s by value. An identity this diagram does not hold — including one it
  holds for a _different_ shape — changes nothing, with no error, no report and no panic. That is
  the same answer _Positions_ gives a reference to a shape that is not there, and it was already
  true of an identity from another diagram.
- **Uniqueness is now a claim about what the diagram issues.** The identities `add` hands out are
  unique within the diagram, and always were; an identity supplied through `add_under` is not
  checked, and §3's "unique within that diagram" is amended in this slice's `docs` commit to say
  which of the two it is about (D2). This is the asymmetry a caller choosing an identity buys.

## `ShapeId` — unchanged in shape, wider in meaning

No field, no derive, no signature. `ShapeId::new(text)` already built an identity from its text, so
a caller could always spell one; what it could not do until this slice was **put a shape under it**,
and that is the difference between an identity a diagram holds and one a caller wants.

Two identities differing only in case, or by a trailing space, are two identities. Nothing in the
crate folds them together and nothing normalizes what a caller wrote, which is why a repeated
identity is a cost rather than a normalization to fix.

## Unchanged

`new`, `add`, `find`, `get`, `remove`, `replace`, `forward`, `backward` and `draw` keep their
signatures and their behavior. `Shape`, `Position`, `Reference`, `Anchor`, `Delta` and every leaf
type are as 083 recorded them, and `Diagram` still has no eighth verb: nothing travels, nothing is
measured, and `Shape::anchor` stays `pub(crate)`.

`get` remains the crate's only reader — no `ids()`, no `len`, no order, no way to ask which shape
decided a position. `add_under` does not become a listing: it adds a shape under a name the caller
already holds, and the specification declines the listing for a consumer that does not exist.

## What is still not here

- **No way to edit an identity after the fact.** §11's open question is half this and half the
  naming above, and only the naming half fires. Renaming is not something a file needs and a caller
  does not yet ask for; settled by the first consumer that edits rather than writes.
- **No way to ask a diagram what identities it holds**, which is the same refusal the specification
  records and the same answer §11 gives its surviving half.
- **No check that a caller-supplied identity is unused.** Two shapes may share one, and the second
  is a shape nobody can name. The repair is one line in `add_under`, and the specification's _What
  this slice does not decide_ names it as a deliberate gap rather than a defect.
- **No report on an identity that resolved to nothing.** That is
  [ADR-0041](../../../docs/decisions/0041-resolve-a-position-through-a-reference.md)'s silent hole
  arriving on a name rather than on a number, and
  [#88](https://github.com/andresmoschini/monospace/issues/88) is where it is answered.
- **No anchor beyond the four sides**
  ([#90](https://github.com/andresmoschini/monospace/issues/90)), **no displacement that reaches a
  reference's offsets** ([#143](https://github.com/andresmoschini/monospace/issues/143)), and **no
  direction derived from an anchor** ([#89](https://github.com/andresmoschini/monospace/issues/89)).
- **No serialization here at all.** The format that can name a shape belongs to `monospace-cli` and
  is recorded in [contracts/description-format.md](description-format.md); this crate depends on no
  serializer and still will not.
- **Nothing in `monospace-core`**, and nothing new that reaches for it. The counter is a `u32` and
  the placement is a `Vec::push` of the same `Placed` an `add` pushes, which is what keeps the
  `wasm` step unchanged.

## Using it

```rust
use monospace_core::{Buffer, GlyphCatalog, Pos, Size, Stroke, render};
use monospace_diagram::{Diagram, Shape, ShapeId};

let origin = Pos { x: 0, y: 0 };
let size = Size { width: 20, height: 6 };

// A diagram whose next addition is handed `#3`, as a file carrying `next_id: 3` asks for.
let mut diagram = Diagram::numbered_from(3);

// A shape placed under a name the caller chose, rather than one the diagram issued. The counter is
// untouched, so the next `add` is still `#3` — which is the point: this is not `add` with a prettier
// name, it is the other way round.
diagram.add_under(ShapeId::new("right"), Shape::Box {
    at: origin,
    size: Size { width: 4, height: 3 },
    stroke: Stroke::from("light"),
    fill: None,
});

// The issued one, and the two ways a change names a shape. Every one of them finds the shape by the
// identity it was given, and finds nothing by any other.
let issued = diagram.add(Shape::Line {
    at: origin,
    len: 4,
    orientation: monospace_core::Orientation::Horizontal,
    stroke: Stroke::from("light"),
});
assert_eq!(issued.to_string(), "#3");

let mut buffer = Buffer::new(origin, size);
diagram.draw(&mut buffer);
let text = render(&buffer, &GlyphCatalog::light(), origin, size);
```

A caller reading a description is the reason both methods exist, and the order of the two is the
order the file is read in: seed the counter from the file's `next_id`, then place each entry under
the name it wrote. The array order is still the drawing order, which is what makes a file's picture
the picture it draws either way round the entries are listed in.

The one thing a caller must decide for itself is **which of the two names it wants**: `add` issues
`#next` and knows nothing is held under it, while `add_under` writes what it is given and checks
nothing. `ShapeId::new` is how either is spelled.
