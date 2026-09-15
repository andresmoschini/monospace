# Phase 0 Research: A diagram holds shapes and draws itself

The architecture is already decided: ADR-0038 puts the model in a crate above the core, ADR-0039
makes a diagram's shape its own entity in a closed set of kinds, and ADR-0042 fixes the drawing
direction and the stamp mode. Nothing here reopens any of them.

What is left is type design inside one new crate — which none of the three ADRs settles, all of it
cheap to undo, and none of it visible outside `monospace-diagram` except as the names in
[contracts/diagram-api.md](contracts/diagram-api.md). That is the split _Where a rationale goes_
asks for, so it is recorded here rather than in an ADR.

## Q1 — How the closed set of kinds is held

**Decision**: one enum with struct variants.

```rust
pub enum Shape {
    Box { at: Pos, size: Size, stroke: Stroke, fill: Option<Glyph> },
    Line { at: Pos, len: u32, orientation: Orientation, stroke: Stroke },
    Arrow { from: Endpoint, to: Endpoint, stroke: Stroke },
}
```

**Rationale**: ADR-0039 chose a closed set so that the compiler propagates a new kind to every place
that reasons per kind. An enum is how Rust says that; nothing else in the language gives exhaustive
matching. Struct variants keep each kind's fields named and public exactly as the core's shapes name
theirs, so the mapping in [data-model.md](data-model.md) is field for field and a reader comparing
the two sees the same words.

The variant can be called `Box`. The core had to call its shape `BoxShape` because `Box` is in the
prelude, but a variant lives in its enum's namespace, so `Shape::Box` costs nothing and the domain
keeps its own word — which is what the maintainer's standing preference for naming the domain rather
than the mechanism asks for.

**Alternatives considered**:

- **Three structs wrapped by an enum** (`enum Shape { Box(BoxShape), .. }`). Same exhaustiveness,
  one more layer to name and to document, and a caller writes `Shape::Box(BoxShape { .. })` instead
  of `Shape::Box { .. }`. It would earn its keep if a kind were passed around on its own, and
  nothing in this slice or in the issues listed under **Out of scope** does that. Reconsider it if a
  later slice needs to hold one kind by itself.
- **A trait and trait objects**. Rejected by ADR-0039 as option B; not reopened.

## Q2 — Which end of the `Vec` is the front of the order

**Decision**: a `Vec<Shape>` whose **last element is the front** of the order. `add` is `push`, and
`draw` iterates `.iter().rev()` so it visits front to back (FR-011).

**Rationale**: `add` puts a shape at the front (FR-006), and this is the only arrangement where that
is `push` — O(1), no shifting, the plainest container Rust has. It also makes the CLI's requirement
fall out with nothing to reverse: FR-018 wants the description's `shapes` array added in the order
it is written with the last entry front-most, which is `for shape in shapes { diagram.add(shape) }`.

The cost is one sentence of documentation, since "the front of the order" and "the front of the
vector" are then opposite ends. That sentence goes on the field itself and on `add`, and the tests
for TE-004 fail if either end is confused.

**Alternatives considered**:

- **Index 0 is the front**, `add` is `insert(0, shape)`. Reads the way the model's words read, and
  makes every add O(n) while giving nothing back. The CLI would also have to walk its array
  backwards to satisfy FR-018.
- **A double-ended queue, pushing at the front**. Matches the words at both ends and costs a
  container that is unusual for a plain list, with no access at the other end to justify it — the
  forward and backward of issue 80 move a shape by one place in the middle, which a queue helps with
  no more than a vector does.

## Q3 — Whether an arrow holds the core's `Endpoint` or the diagram's own

**Decision**: the diagram defines its own `Endpoint { at: Pos, leaving: Direction, head: Glyph }`,
mirroring the core's and converting into it.

**Rationale**: FR-007 puts every position a figure carries in the diagram's shape, and an arrow's
two endpoints carry the only positions it has. Holding `monospace_core::Endpoint` would put those
positions inside a core type, which is the one thing FR-007 is written to prevent — and issue #85,
which lets an endpoint hang off a reference, would then have to replace the type rather than widen a
field the diagram already owns.

It is three fields of duplication today, and it is the same duplication `monospace-cli` already
carries for its own `Endpoint` and for the same reason: the layer that owns the position owns the
type that holds it.

**Alternative considered**: re-export the core's `Endpoint` and use it directly. Smaller now, and
rejected because it puts the crate's most likely next change — an endpoint that hangs off a
reference, issue 85 — inside the crate that must not know about diagrams.

## Q4 — Naming a type `Shape` when the core already has one

**Decision**: the type is `Shape`, and where the crate needs the core's trait for its `draw` method
it imports it anonymously: `use monospace_core::Shape as _;`.

**Rationale**: _Vocabulary_ in `docs/diagram-model.md` names this thing `Shape` and says in as many
words that the core's and the diagram's share a name and are different things. Renaming it here to
dodge an import collision would put a word in the code that the model does not have. The anonymous
import is the idiomatic way to bring a trait's methods into scope without its name, and it is
exactly the case it exists for.

## Q5 — One `Layer` per drawing, or one per shape

**Decision**: one, built once at the top of `draw` and passed to every shape.

**Rationale**: a `Layer` is a buffer and a mode bound together and holds nothing else, so one per
shape would produce identical writes for more work. Building it once also puts the
`StampMode::Below` of FR-012 in a single place, where it is bound by the diagram and nothing
downstream can reach it.

## Q6 — What `draw` takes

**Decision**: `pub fn draw(&self, buffer: &mut Buffer)`.

**Rationale**: FR-010 says the caller's buffer is the window and the diagram creates nothing, and
FR-012 says the caller must not be able to choose a stamp mode. Taking a `&mut dyn Surface` instead
would satisfy the first and break the second, since the obvious `Surface` to hand in is a `Layer`
bound to `Above`. Taking the buffer is also what issue #86 needs later, when the ownership record
lands beside the cells in the buffer rather than in whatever wrapper a caller chose.

`&self` rather than `&mut self` is FR-015: drawing changes nothing about the diagram, and the
signature is what says so first.

## Q7 — How a test compares two buffers

**Decision**: a test-local helper that collects `Vec<Option<Cell>>` over the window with
`Buffer::cell`, plus `monospace_core::render` where the assertion is about the picture a person
would see.

**Rationale**: `Buffer` derives no `PartialEq`, and TE-001, TE-003 and TE-004 all compare two
buffers. Adding `PartialEq` to `Buffer` would be new public surface on the core added for a test in
another crate, which FR-002's spirit and principle VII both argue against. `Cell` already derives
`PartialEq` and `Buffer::cell` is already public, so the helper is a dozen lines inside the test
module and the core stays untouched. Rendering is the second lens: it is what the equivalence in
TE-001 is ultimately about, and it makes a failure readable as two pictures rather than two vectors.

## Q8 — `new` and `Default`

**Decision**: `Diagram::new()` for an empty diagram, with `impl Default for Diagram` beside it
delegating to it.

**Rationale**: clippy's `pedantic` group, which the workspace turns on and the gate runs with
`-D warnings`, has `new_without_default`. Implementing both is the conventional answer and is what
the gate expects; there is nothing to decide beyond writing it.

## Q9 — What the demonstration edit is, exactly

**Decision**: remove every `mode` field from `crates/monospace-cli/assets/demo.json`, and swap the
two pairs whose second shape carries `mode: "below"` — the boxes at `{7,1}`/`{9,2}` and the boxes at
`{18,0}`/`{20,1}` — so the box that decided the shared cells is written last.

**Rationale**: under the diagram, the last entry in the file is front-most and decides first with
`Below`, which _The two orders are equivalent_ in `docs/model.md` makes identical to visiting the
file in order with `Above`. Today the second shape of each of those two pairs is stamped `Below`, so
the first one decides; writing it last is how the same picture is said in a format with no `mode`.
The spec's Assumptions record that this was run and compared before the spec was written; the same
comparison is run again at delivery for SC-003 and TE-007.

## Q10 — Whether `monospace-diagram` re-exports the core types its shapes hold

**Decision**: no. A caller naming `Pos`, `Size`, `Stroke`, `Glyph`, `Orientation` or `Direction`
takes them from `monospace_core`, which `monospace-cli` already depends on for `Buffer` and
`render`.

**Rationale**: re-exporting would give every one of those types two paths and would make the diagram
crate look like a facade over the core, which ADR-0038 deliberately did not build. The one caller
this slice has needs the core anyway.

## Dependencies

None added. `monospace-diagram` depends on `monospace-core` through the workspace, and
`monospace-cli` gains a workspace dependency on `monospace-diagram`. No crate from crates.io enters,
so there is no publication date to verify for this feature.
