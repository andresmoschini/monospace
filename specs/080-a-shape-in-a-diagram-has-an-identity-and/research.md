# Phase 0 research: a shape has an identity, and the order can change

Eight questions the design raised. The spec settled under _Assumptions_ that this slice takes no
ADR: every question here is answered inside one crate, or inside `monospace-cli`, and each answer is
undone by changing the code that gives it. That is the split _Where a rationale goes_ describes, so
they are recorded here.

## Q1: How is an identity typed so that it reads as `#1` and cannot be built from `#1`?

**Decision**: `ShapeId`, a newtype over a private `u32`, deriving `Clone`, `Copy`, `Debug`,
`PartialEq` and `Eq`, with a hand-written `Display` that writes `#` and then the number. No public
constructor, no `From<&str>`, no `FromStr`, and no accessor for the number inside.

**Rationale**: FR-003 asks that reading an identity give no way back to one. A private field in a
crate that exposes no constructor is what makes that structural instead of a convention: the only
`ShapeId` values that exist anywhere are the ones `Diagram::add` handed out. `Display` is what
"reads as that text" means in Rust — `{}`, `format!` and `to_string` all come from it, and TE-001
tests it through `to_string`. Holding a `u32` contradicts nothing: _Vocabulary_ says the identity is
a string and FR-001 asks it to read as one, while the spec's own assumption leaves the typing to
this plan. A number is also the whole of the generation in Q3.

**Alternatives considered**:

- A public `String` field. Anyone could then build `#1` and name a shape they were never handed,
  which is FR-003 backwards.
- A private `String` field. The same public surface, but an allocation per shape to hold what four
  bytes hold, and the `#` repeated in every value instead of living in one `Display`.
- Dropping `Copy`. An identity would then be cloned at every call. It is a four-byte value passed as
  an argument, and `Copy` is what lets TE-006 read as `forward(id)` then `backward(id)`.
- Writing `Debug` by hand so it prints `#1` too. The derive is kept: `assert_ne!` needs `Debug`, and
  what it prints is a test message rather than the identity's text, which `Display` owns.

## Q2: Where does an identity live?

**Decision**: `Diagram` holds a `Vec<Placed>`, where `Placed { id: ShapeId, shape: Shape }` is
private to the crate. The public `Shape` enum gains nothing.

**Rationale**: the identity is the diagram's and not the shape's, which is why two equal shapes
added twice get two of them. A caller builds a `Shape` as a literal, so there is no field it could
fill and none it should see. Pairing the two in one private struct puts the invariant — every held
shape has exactly one identity, and no identity exists without a shape — in the type rather than in
the code that maintains it. The name is the model's own: _Identity_ says an identity is what makes a
shape "findable after it has been placed".

**Alternatives considered**:

- `Vec<(ShapeId, Shape)>`. The same layout, with `.0` and `.1` at every use and nothing naming what
  the pair is.
- Two parallel `Vec`s. Staying the same length and in step would be maintained by hand at every
  change, and a reorder is the first thing that would get it wrong.
- An `id` field on the public `Shape`. The caller would have to supply one.

## Q3: How is the next identity generated?

**Decision**: `Diagram` holds a private `next: u32`, starting at 0 and incremented before each use,
so the first shape added is `#1`. Nothing searches what is already there, and there is no state
outside the diagram.

**Rationale**: FR-001 asks for uniqueness within one diagram and nothing more, and a counter gives
it in one addition. It also gives the spec's assumption that identities are never reused, for free,
on the day issue #81 removes a shape: deriving the identity from the length would hand the next
shape the identity of the one just removed. Starting at 0 and incrementing first is what keeps
`#[derive(Default)]` on `Diagram`, which is what `Diagram::new` is built on today.

**Alternatives considered**:

- Deriving the identity from how many shapes are held. One field cheaper, and wrong as soon as
  anything is removed.
- A process-wide counter, or a random or UUID identity. Both buy uniqueness across diagrams, which
  the spec explicitly does not want: an identity from another diagram names nothing there, and an
  edge case tests exactly that.

## Q4: What do `forward` and `backward` look like, given neither may fail?

**Decision**: `pub fn forward(&mut self, id: ShapeId)` and
`pub fn backward(&mut self, id: ShapeId)`, both returning nothing. Each finds the position of `id`
and, when there is one and it is not already at the end it is moving toward, swaps it with its
neighbor.

**Rationale**: FR-009 rules out an error and a report, and a `bool` is a report. A signature that
returns nothing is what "cannot fail" looks like: the caller has nothing to check because there is
nothing to check. Two named methods rather than one taking a direction, because the model names two
changes and the call site then reads as the change it makes. `Vec::swap` is exactly a one-place move
between neighbors, and such a move is its own inverse, which is what user story 2's scenario 6
asserts.

**Alternatives considered**:

- One method taking a direction argument. A third public type invented for two methods, and
  `Direction` already means something else in the core: where an arrow leaves an endpoint.
- Returning `bool` or `Result` when the identity is not held. Ruled out by FR-009, and the model
  gives the reason: it is the same non-answer a reference to a missing shape gets.

## Q5: Does `add` returning an identity disturb the callers that do not want one?

**Decision**: `pub fn add(&mut self, shape: Shape) -> ShapeId`, with no `#[must_use]`.

**Rationale**: `add` is still the only way to put a shape in, and now the only way to learn an
identity (FR-002). Ignoring the return is a legitimate use — every shape the application adds but
never moves — and `#[must_use]` would turn each of those into `let _ =`. Clippy's
`must_use_candidate`, which the pedantic group turns on, does not fire on a method taking
`&mut self`, so the gate does not ask for the attribute either. Existing call sites compile
unchanged, because a call whose value is discarded is already a statement.

**Alternatives considered**:

- Leaving `add` alone and adding a second way in that hands the identity back. Two doors, and the
  second one reads the order, which this slice does not add.
- `#[must_use]`. Right for a function whose only purpose is its return value; this one's purpose is
  the addition.

## Q6: How does the application learn the identity of the shape it moves?

**Decision**: `Description::into_diagram` returns `(Diagram, Option<ShapeId>)` — the diagram, and
the identity of the first entry of `shapes`, which is the back-most shape. `None` when the
description holds no shapes.

**Rationale**: FR-015 names the shape by its position in the file, so position is all the
application needs: the conversion already visits the entries in order and knows which one it added
first. Handing back one identity rather than all of them adds nothing the application would use, and
`Option` is the answer to a description holding none, which user story 3 asks about.

**Alternatives considered**:

- Returning every identity in description order. The same work, and the application would index the
  first element out of a list built for one.
- Giving the description format a field that names the shape to move. Forbidden by FR-019 and by
  [ADR-0035](../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md).
- Asking the diagram what is at the back of its order. That is a reader of the order, and the spec's
  **Out of scope** table names it.

## Q7: Why does the second picture need its own buffer?

**Decision**: the application builds a fresh `Buffer` for each picture from the canvas's origin and
size, and `Description::buffer` is removed in favor of `Buffer::new` at the call site.

**Rationale**: a diagram draws with `StampMode::Below`
([ADR-0042](../../docs/decisions/0042-draw-a-diagram-front-to-back-into-a-given-window.md)), so a
cell already written wins. Drawing the reordered diagram into the buffer the first picture used
would print the first picture twice — the feature failing in exactly the way it exists to show. Two
buffers is the only arrangement under which the second picture is the second picture.
`Description::buffer` cannot serve both, because `into_diagram` consumes the description; since
building a buffer from the window was all it did, it goes and `Buffer::new(origin, size)` is written
where it is used. That removal changes no behavior, so principle V puts it in its own `refactor`
commit ahead of the feature's.

**Alternatives considered**:

- Clearing the buffer between drawings. `Buffer` offers no such operation, and adding one to the
  core for the convenience of a demonstration is the core growing a feature it has no use for.
- Keeping `Description::buffer` and calling it twice before `into_diagram` consumes the description.
  It would then be a helper for a window the caller already holds from `window()`, called once per
  picture. It earns nothing over `Buffer::new`.

## Q8: What do the captions say?

**Decision**: one line above each picture, and one blank line between the two:

```text
As written:
<picture>

With the back-most shape moved one place toward the front:
<picture>
```

**Rationale**: FR-018 asks for a short line saying what each picture shows and leaves the wording
here. These two name what changed rather than how, and "the back-most shape" is exactly what the
application moves (FR-015), readable by someone who does not know that identities exist. `render`
ends every row with a newline, so each caption is a line of its own by construction and one extra
newline after the first picture is the blank line. TE-007 pins the pictures and not this wording, so
changing it later costs nothing.

**Alternatives considered**:

- Naming the identity in the caption. It reads as internals, and which identity the first entry got
  is a detail of the demonstration rather than something the reader can act on.
- One caption above both pictures. FR-018 asks for one each, and a run whose two pictures are
  identical still needs the second to say what it was showing.
