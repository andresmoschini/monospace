# Data model: An arrow draws the same whichever endpoint is named first

The domain vocabulary is [`docs/model.md`](../../docs/model.md)'s, under _The initial set_, _An end
is an arm; a head is a glyph_ and _The route of an arrow_. Nothing here restates it. What follows is
the shape the derivation takes inside `monospace-core`, and which of it is public.

## What does not change

No public type, field or signature. `Arrow`, `Endpoint`, `Shape` and `Surface` are exactly what they
are today, and a caller that compiles against the core before this feature compiles against it
after. Everything below is private to `crates/monospace-core/src/shape/`.

Feature 039's research, Q5, described the derivation as a search with a scoring function. That
description stops being true here; the decision it stood in for now lives in
[ADR-0044](../../docs/decisions/0044-let-the-endpoint-order-break-a-tied-route.md), which is why
research.md Q2 can replace the mechanism without reopening the rule.

## The concepts the derivation names

Each is a name the model already uses. Whether each becomes a type, a function or a local is an
implementation choice; what the plan fixes is that the code says these words.

**Starting position.** One step from an endpoint in that endpoint's leaving direction — `s` for the
`from` endpoint, `t` for the `to` endpoint. Two of them per arrow, and every other concept here is
derived from the pair. Fallible: a step that would overflow `i32` has no starting position, and the
arrow is its two heads.

**Route rectangle.** The smallest rectangle containing `s` and `t`. It bounds the path, and it is
what the middle is the middle of. Its two spans are independent: the horizontal one bounds a column,
the vertical one bounds a row.

**Middle.** One value per axis: the cell halfway along that axis's span of the route rectangle.
Where the span holds an even number of cells the halfway point falls between two, and the middle is
the one nearer `s`. This is the only place the order of the two endpoints reaches the route, and it
is the whole of what ADR-0044 decided.

**Run.** A maximal straight piece of the path: a direction, a length, and one **fixed coordinate** —
its row if horizontal, its column if vertical. Runs alternate between horizontal and vertical by
definition, so a path is its list of runs and nothing else.

A run's fixed coordinate is either **pinned** or **free**. The first run's is pinned by the `from`
endpoint's position and the last run's by the `to` endpoint's. Every other run's is free, and a free
fixed coordinate takes the middle on that run's axis, narrowed to the interval its two neighboring
runs and the route rectangle leave open. "Whichever coordinate the bends leave free", in the model's
words, is this.

**Path.** The sequence of runs from the `from` endpoint's position to the `to` endpoint's: first
direction `da`, last direction `opposite(db)`, alternating, inside the route rectangle. The number
of runs is what _fewest bends_ counts; its parity is fixed by whether `da` and `opposite(db)` share
an axis, so the candidates are 1, 3, 5 or 2, 4, tried in that order.

**Route.** The path without its two ends — the two endpoint positions carry the heads. This is what
`Route` already means in `shape/route.rs`, and it keeps that meaning.

## The invariants a route holds

Each is a rule from the spec, stated as something a test can ask of a drawn arrow. They are the
contract in [`contracts/arrow-rendering.md`](contracts/arrow-rendering.md), listed here because they
are properties of the values above rather than of the rendering.

1. **Every cell written is on the route.** No cell beyond either end of a run, which is D1 in
   research.md Q1.
2. **No route cell is an endpoint position.** A head is the only thing that writes where a head
   goes. The path passes through both endpoint positions as its ends, and the route is the path
   without them; a run that would cross the _other_ endpoint is not a route the rectangle permits.
3. **No cell is written twice.** A bend belongs to exactly one run, which is what `Route::draw`
   already arranges by breaking its positions into runs before placing anything.
4. **Exchanging the two endpoints changes the route in one way only.** A free fixed coordinate whose
   middle falls between two cells moves to the other of the two. Everything else — the run count,
   every run's direction, every pinned coordinate, the length — is the same.

## What is empty rather than absent

An arrangement with no path inside the route rectangle has an empty route, and the arrow is its two
heads. This is the model's answer rather than a special case, it is 170 of the 928 arrangements, and
nothing in this feature changes which arrangements they are. Where the two endpoints share a
position, the two heads share a cell and the one seen is the `to` endpoint's — under
`StampMode::Above`, which is the limit research.md Q5 records.
