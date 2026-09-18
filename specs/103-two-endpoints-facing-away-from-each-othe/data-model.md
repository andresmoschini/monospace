# Data model: An arrow's route is ranked rather than bounded

The domain vocabulary is [`docs/model.md`](../../docs/model.md)'s, under _The route of an arrow_,
and nothing here restates it. What follows is the shape the derivation takes inside `monospace-core`
once the ranking replaces the bound, and which of it is public.

Feature 099's [`data-model.md`](../099-bug-with-arrows/data-model.md) described the same derivation
as a list of shapes with a double escape at the end of it. That description stops being true here.
The concepts it named that the model still names - starting position, route rectangle, middle, run,
path, route - are unchanged; what goes is the catalogue they were arranged into.

## What does not change

No public type, field or signature. `Arrow`, `Endpoint`, `Shape` and `Surface` are exactly what they
are today, and a caller that compiles against the core before this feature compiles against it
after. `derive_path` keeps its signature,

```rust
fn derive_path(a: Pos, da: Direction, b: Pos, db: Direction) -> Option<Vec<Pos>>
```

so `impl Shape for Arrow` is untouched, and `Route` in `shape/route.rs` keeps its meaning and its
`pub(crate)` visibility. Everything below is private to `crates/monospace-core/src/shape/arrow.rs`.

## The concepts that survive

Each is a name the model already uses, and each is already code.

**Starting position.** One step from an endpoint in that endpoint's leaving direction - `s` for the
`from` endpoint, `t` for the `to` endpoint. Fallible: a step that would overflow `i32` has no
starting position, and the arrow is its two heads. `offset`, `arrow.rs:65-72`.

**Route rectangle.** The smallest rectangle containing `s` and `t`. It **stops bounding the path**
and keeps exactly one job: its middle is what the third term of the ranking measures distance from,
and its edges are two of the lines the search may turn on. `RouteRectangle::spanning`,
`arrow.rs:151-158`.

**Middle.** One value per axis, the cell halfway along that axis's span, taken as the one nearer `s`
where the span holds an even number of cells. This is ADR-0044 and it is unchanged code -
`RouteRectangle::middle` and `midpoint`, `arrow.rs:164-180`. What changes is who reads it: a term of
a cost rather than a filter over candidate shapes.

**Path.** A sequence of positions from the `from` endpoint's position to the `to` endpoint's, first
step `da`, last step arriving against `db`, runs alternating between horizontal and vertical,
visiting no position twice and passing through neither endpoint position
([ADR-0047](../../docs/decisions/0047-let-a-route-cross-no-cell-twice.md)). Nothing bounds where it
goes.

**Route.** The path without its two ends, since the endpoint positions carry the heads. `Route`,
`route.rs:35-40`, unchanged.

## The concepts that arrive

Three, and they are ADR-0049's. Whether each is a struct, a function or a local is an implementation
choice; what the plan fixes is that the code says these words, in the model's order.

**Cost.** The ranking as one value, compared lexicographically, with one field per term of _The
route of an arrow_ and in the model's order:

| Field         | The model's term                                              | Record   |
| ------------- | ------------------------------------------------------------- | -------- |
| `bends`       | the fewest bends                                              | ADR-0046 |
| `length`      | then the shortest                                             | ADR-0046 |
| `from_middle` | then the free runs nearest the middle, rounding toward `from` | ADR-0044 |
| `hand`        | then those runs on the side the travel puts to its right      | ADR-0048 |

Derived `Ord` over the fields in that order _is_ the ranking - the tie-break is the derive, not a
comparator someone wrote. Each term is charged at the turn that starts a run, so one pass of the
search yields the ranked-first route with no second pass over candidates. `hand` is charged only on
the axis the `from` endpoint leaves along, which is what makes ADR-0048's handedness the arrow's own
travel rather than the coordinate system's.

**Lattice.** The lines a route may turn on, per axis: the line each starting position pins, the line
beside each of them (where an endpoint's own cell blocks the way, the route passes next to it), the
middle, and one line outside the route rectangle. Nine candidates per axis, at most **seven**
distinct after deduplication - research.md Q2 - so at most `7 x 7 x 4 = 196` states whatever the
arrangement, and the cost of a derivation does not grow with the distance between the endpoints
(FR-008).

The lattice is the one claim in this feature that is not a consequence of the model. "A route turns
only on these lines" is an assertion about the rule, and ADR-0049's confirmation is a comparison
against a search that assumes nothing, not a proof. It is also where the record says this fails: a
term added to the ranking that the lattice does not know about.

**State.** A node of the lattice together with the heading it was reached on -
`(column, row, heading)`. The heading has to be part of the state because the cost of arriving
somewhere depends on whether arriving there is a bend, and because a route may not reverse.

## The concepts that go

`three_waypoint` and `zigzag` - the escape and the double escape - `path_bends`, `direction_between`
and `opposite_orientation`. FR-007 asks for no family of arrangements to keep a construction of its
own, and this is that requirement as a list of deletions.

Five runs becomes the most any route takes. That is a **measured consequence** of ranking the fewest
bends first, down from seven, and FR-007 says explicitly that it must not be enforced: a cap on runs
would be the kind of bound ADR-0046 removed, reintroduced in a place nobody would look for it.

## Invariants

Each is a property of the model, not of this implementation, and each has somewhere in
[`contracts/route-derivation.md`](contracts/route-derivation.md) that a test points at.

1. A path visits no position twice, and passes through neither endpoint position.
2. The route is the path without its two ends, so no cell of the route is a cell a head writes.
3. The route is empty only where no path exists, which after this slice is exactly two arrangements:
   an endpoint standing on the cell the route would have to arrive at, and two endpoints at one
   position leaving the same direction.
4. Exchanging the two endpoints changes the picture only through the third and fourth terms of the
   cost, which is the whole of what the order of the endpoints decides.
