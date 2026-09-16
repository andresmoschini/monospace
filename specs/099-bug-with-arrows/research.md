# Research: An arrow draws the same whichever endpoint is named first

Everything below was taken by running the code on 2026-09-16, at commit `633abb5`, through a
throwaway example that drives the core's public API and prints what it renders. The example is not
kept: what it produced is here, and what it did is [`quickstart.md`](quickstart.md).

Where a finding will outlive this feature it is an ADR and this file links to it, per the
constitution's _Where a rationale goes_.

## Q1 — What is actually broken?

**Decision**: three independent defects, not one. Each is repaired on its own and each is observable
on its own.

**Rationale**: the spec describes the symptom — over half the arrangements draw a different picture
when their endpoints are exchanged — and naming the causes is what turns that into a task list.
Reading `crates/monospace-core/src/shape/arrow.rs` and `.../shape/route.rs` against renderings gives
three:

**D1 — a route is drawn away from itself when it runs toward a smaller coordinate.** `Segment::draw`
walks `run(from, len, orientation)`, which always steps toward the increasing coordinate, and
`Route::draw` hands it `positions[start]` — the first cell of the run in path order. Where the path
travels left or up, the run is drawn from its first cell in the opposite direction: the right cells
are missed and the same number of wrong ones are written, including, often, the cell a head
occupies. This is the whole of the bug report's straight case. Rendering `(0, 0)` leaving `right`
and `(6, 0)` leaving `left` gives `◄─────►`; exchanging the two gives `◄    ──`, which is the same
run written backwards out of its starting cell.

Measured over the grid Q6 defines: **110 of the 1856 renderings draw a route cell over one of the
two heads**. All of them are this.

**D2 — the tie-break among fewest-bend candidates scores the wrong thing.** `derive_path` ranks
candidates by `bends`, then by `closeness`: the sum, over every waypoint, of the Manhattan distance
from that waypoint to the middle of the route rectangle. Summing over waypoints penalizes a
candidate for having more of them, which has nothing to do with the model's rule. In User Story 2's
`n = 4` row — `(0, 0)` leaving `right`, `(4, 3)` leaving `left`, so the starting positions are
`(1, 0)` and `(3, 3)` and the middle column is 2 — the two fewest-bend candidates score:

| waypoints                 | turns at | `closeness` |
| ------------------------- | -------- | ----------- |
| `(1,0) (3,0) (3,3)`       | `x = 3`  | 7           |
| `(1,0) (2,0) (2,3) (3,3)` | `x = 2`  | 8           |

The second is the one the model names and it loses by one, because it has a fourth waypoint. That is
the whole of User Story 2: `n = 4` and `n = 5` turn at an edge of the route rectangle, `n = 6` at
its middle, and which happens is an accident of how many waypoints the right answer needs.

**D3 — the middle always rounds toward the smaller coordinate.**
`mid_x = x_min + (x_max - x_min) / 2` truncates toward `x_min` whichever endpoint the arrow leaves
from. [ADR-0044](../../docs/decisions/0044-let-the-endpoint-order-break-a-tied-route.md) requires
the cell nearer the `from` endpoint's starting position. Measured on the ADR's own arrangement,
`(0, 1)` leaving `down` and `(2, 6)` leaving `up`: the free coordinate spans rows 2 to 5, and both
orders turn at row 3 today. The second one must turn at row 4.

**Alternatives considered**: reading the diff of feature 039 to find where it went wrong. Rejected:
the code is what ships, and three renderings answered the question in less time than the archaeology
would have taken.

## Q2 — Is `derive_path` repaired or rewritten?

**Decision**: rewritten, as a direct construction. The maintainer's call.

**Rationale**: what is there is a depth-first search over a nine-point lattice, collecting every
completed path and ranking them by a score. The rule it implements is a sentence long — fewest
bends, and every turn the bends leave free sits at the middle of the route rectangle — and a search
with a scoring function cannot be read against that sentence. D2 is what an unreadable selection
costs: the score has been wrong since feature 039 and the ten pinned pictures never caught it,
because all ten are cases where the score happens to agree.

The construction, restated from _The route of an arrow_:

1. The **starting positions** are `s = a + da` and `t = b + db`. The path runs from `a` to `b`, its
   first step is `da`, its last step is `opposite(db)`, and the route is that path without its two
   ends.
2. The **route rectangle** is the smallest rectangle containing `s` and `t`.
3. The path is a sequence of **runs** alternating between horizontal and vertical. The first run's
   direction is `da` and the last run's is `opposite(db)`, so the parity of the run count is fixed
   by whether those two share an axis. Run counts are tried in order — 1, 3, 5 or 2, 4 — and the
   first that admits a path inside the rectangle is the one with the fewest bends.
4. Each run has one **fixed coordinate**: its row if horizontal, its column if vertical. The first
   run's is pinned by `a` and the last run's by `b`. Every run in between is the free one the model
   speaks of, and its fixed coordinate is the middle of the route rectangle on that run's axis,
   narrowed to the interval the neighboring runs and the rectangle leave open.
5. Where that middle falls between two cells, it is the one nearer `s` — ADR-0044.

Checked by hand against every picture the workspace pins today, and against every picture the spec
pins:

| arrangement                                         | free run             | this construction | today   |
| --------------------------------------------------- | -------------------- | ----------------- | ------- |
| 039 scenario 1, `(0,0)` down / `(4,3)` left         | none, two runs       | corner at `(0,3)` | same    |
| 039 scenario 5, `(0,0)` right / `(6,2)` left        | column, `x` 1 to 5   | `x = 3`           | same    |
| 039 scenario 6, `(2,0)` right / `(8,2)` left        | column, `x` 3 to 7   | `x = 5`           | same    |
| 039 scenario 7, `(2,0)` left / `(8,2)` right        | row, `y` 0 to 2      | `y = 1`           | same    |
| 039 scenario 8, `(2,0)` left / `(8,2)` down         | none                 | unchanged         | same    |
| 039 scenario 9, `(2,0)` right / `(4,1)` left        | column, `x` 3 to 3   | `x = 3`           | same    |
| 039 scenario 10, `(2,0)` left / `(3,2)` right       | row, `y` 0 to 2      | `y = 1`           | same    |
| identical directions, `(2,0)` right / `(6,2)` right | none                 | unchanged         | same    |
| the demonstration, `(13,3)` right / `(22,4)` down   | column, `x` 14 to 22 | `x = 18`          | same    |
| US2 `n = 4`                                         | column, `x` 1 to 3   | `x = 2`           | `x = 3` |
| US2 `n = 5`                                         | column, `x` 1 to 4   | `x = 3`           | `x = 1` |
| US2 `n = 6`                                         | column, `x` 1 to 5   | `x = 3`           | same    |
| ADR-0044's pair, from the `▲` end                   | row, `y` 2 to 5      | `y = 3`           | same    |
| ADR-0044's pair, from the `▼` end                   | row, `y` 2 to 5      | `y = 4`           | `y = 3` |

So SC-005 and FR-005 hold by construction rather than by luck: nothing feature 039 pinned moves, and
the only pictures that move are the ones the spec asks to move.

Scenario 7 is the case that fixes the run count's ceiling at five: both endpoints face away from
each other, so the path needs a run to escape each of them, a run to cross, and the two turns
between — five runs, four bends. That five is enough for every arrangement was a claim, not yet a
measurement; the sweep of Q6 is what settled it, and it did not hold. Verified against an
independent brute-force reference (every path on the nine-point lattice, not just the five
constructed shapes above) across all 1856 renderings: 34 of them — 17 arrangements, both orders —
need a seventh run.

Each is the same shape as scenario 7 — both endpoints face away from each other, along the axis `da`
and `exit_dir` share — narrowed until the escape run has no room: the route rectangle is only two
cells wide on that axis, so its middle coincides with one of the two escape runs' own pinned
coordinate rather than sitting strictly between them, and the five-run construction's boundary run
collapses to nothing. `(0, 0)` leaving `up` and `(1, 2)` leaving `down` is the smallest case:
`s = (0, -1)`, `t = (1, 3)`, and the route rectangle's `x` span is `0` to `1` — two cells, no third
column for the escape to use.

The seventh run is a second escape rather than a wider one: each end jogs to the _far_ endpoint's
coordinate on the narrow axis before crossing at the middle on the other axis, then jogs back — `s`,
`(t.x, s.y)`, `(t.x, mid.y)`, `(s.x, mid.y)`, `(s.x, t.y)`, `t`. Six bends, and — checked against
the same brute-force reference — the only shape that fits. This is what
[`data-model.md`](data-model.md) now calls the **double escape**, and it is where the run-count
ceiling actually sits: seven, not five, for this one family, reached only when the ordinary escape
degenerates.

Widening the search does not risk widening the _bound_ — the route rectangle stays the bound
regardless of how many runs fill it — so this is the same rule, not a new one, filling in a gap the
plan's own hand-checked table did not reach.

**Alternatives considered**: repairing the three defects in place — fix `Route`'s run direction,
replace `closeness` with a score over free coordinates only, round the middle toward `s`. Cheaper,
and it would have landed the same pictures. Rejected because the selection would still be a score to
be read against a sentence, and the next rule added to _The route of an arrow_ would be another term
in it.

## Q3 — How is the sweep pinned?

**Decision**: as an `insta` snapshot of all 1856 renderings, reviewed once against the model. Not
local to this feature: recorded as
[ADR-0045](../../docs/decisions/0045-pin-every-arrow-arrangement-as-a-reviewed-snapshot.md), which
holds the options and the costs.

One consequence is this feature's to carry. The gate's `editorconfig-checker` step reads every
tracked file with `trim_trailing_whitespace` on, and `render` pads each line to the window's width,
so a snapshot of raw renderings fails the gate on its own trailing blanks. Trimming each line's
trailing blanks before it goes into the snapshot loses nothing: the padding sits past the last glyph
of a line, so the column of every glyph is preserved.

## Q4 — Which `insta`?

**Decision**: `insta` 1.48.0, as a dev-dependency of `monospace-core` only.

**Rationale**: it is the latest version and it was published on **2026-06-11**, three months before
today, so the seven-day rule in the constitution's dependencies constraint is satisfied with room. A
dev-dependency does not reach the library, so the gate's `wasm` step — which checks
`monospace-core`, `monospace-diagram` and `monospace-glyph-sets` without `--all-targets` — does not
build it, and the portability principle is untouched.

`insta` pulls transitive dependencies of its own. The constitution scopes the seven-day rule to
direct dependencies; at the moment the crate is added, the lock file's new entries are read and
anything published within the last week is reported rather than taken silently.

**Alternatives considered**: a version older than the latest. Rejected — there is no reason to take
one when the latest is three months old.

## Q5 — What do the colliding heads actually do today?

**Decision**: FR-004 already holds where it can, and the spec's edge case says otherwise. The spec
is corrected on this branch; the behavior is pinned by a test and not changed.

**Rationale**: the spec's Edge Cases section says that both endpoints at `(2, 1)`, one leaving
`right` with head `◄` and the other leaving `left` with head `►`, "draw `►` when the `◄` end is
named first", and that "rendering that arrangement shows the opposite today". Rendered, all four
combinations:

| stamp mode | named first | drawn |
| ---------- | ----------- | ----- |
| `Above`    | the `◄` end | `►`   |
| `Above`    | the `►` end | `◄`   |
| `Below`    | the `◄` end | `◄`   |
| `Below`    | the `►` end | `►`   |

Under `StampMode::Above` — which is what every test and the CLI use — the head seen is the `to`
endpoint's, which is exactly FR-004. `Arrow::draw` draws the `from` head, then the `to` head, and
`Above` lets the second win. The spec's claim is reproducible only under `StampMode::Below`, where
the first head written is the one that survives.

The maintainer's call is to leave the behavior alone and pin it: a test renders the arrangement
under `Above` and asserts the `to` head. What remains true is that `Arrow` does not control the
stamp mode, so under `Below` the `from` head is seen. That is a limitation this feature names rather
than removes.

**Alternatives considered**: making the guarantee mode-independent by drawing only the `to` head
where the two positions coincide. Not taken. It is a small change and it would make FR-004 true
under either mode, but it is a behavior change the bug report did not ask for, and the spec's own
edge case calls the cell unpinned.

## Q6 — What exactly is the grid the spec counts?

**Decision**: two anchors, `(0, 0)` and `(2, 1)`, each with four leaving directions, against every
position of a six-by-five field — `x` from 0 to 5, `y` from 0 to 4 — with four leaving directions
each, excluding the arrangements where the second position equals the anchor. Each arrangement is
rendered from both ends.

**Rationale**: the spec names 928 arrangements and 1856 renderings without saying which grid
produces them, and a snapshot needs the grid written down or it cannot be regenerated. The
definition above reproduces both numbers exactly: 2 anchors × 4 directions × 30 positions × 4
directions is 960, less the 32 where the second position is the anchor, is 928; twice that is 1856.
It also reproduces the spec's third number, 170 arrangements drawing no route.

The window is the other half of the definition, since a route can run outside it and be clipped. The
sweep renders into the window with origin `(-2, -2)` and size 10 by 9, which holds the six-by-five
field and the one cell of margin a leaving direction can add on each side.

One number does not reproduce. The spec says 502 arrangements were order-dependent; under this
window it is **495**. The spec says that figure was measured before its last revision and not
retaken, so the difference is most likely a different window clipping seven arrangements
differently. It bounds the defects either way and nothing depends on which figure is right, so it is
reported rather than chased. The snapshot is what replaces both numbers with something that cannot
drift.

**Alternatives considered**: choosing a fresh grid and restating SC-001's numbers. Rejected — the
counts match, which is evidence the reconstruction is the grid the spec meant, and a new grid would
throw that evidence away.
