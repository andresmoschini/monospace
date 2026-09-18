# Contract: the route an arrow draws

`monospace-core`'s public surface does not change in this feature. Feature 039's
[`contracts/public-api.md`](../../039-draw-shapes-instead-of-individual-cells/contracts/public-api.md)
still describes `Arrow`, `Endpoint` and `Shape` exactly as they are, and feature 099's
[`contracts/arrow-rendering.md`](../../099-bug-with-arrows/contracts/arrow-rendering.md) still holds
in full - C-1 to C-7 there are statements about a route lying on itself, a head being alone at an
endpoint and no cell being written twice, and none of them is weakened by ranking a route instead of
bounding it.

What changes is which route the rule picks. This file is that behavior written as statements a test
can make, one per rule, so each requirement of the spec has somewhere to point. Each statement is
about the text `render` produces for a buffer an `Arrow` has been drawn into with
`StampMode::Above`.

## The statements

**R-1 - two endpoints facing away are joined.** For two endpoints on one line whose leaving
directions point away from each other, every cell between the two heads lies on one unbroken route
that reaches each head against that head's own leaving direction. True however far apart they are.
Covers FR-002 and User Story 1 scenarios 1 and 3.

_Refuted by_: today's `(0, 0)` leaving `Up` with `(0, 2)` leaving `Down`, which draws two heads and
nothing between them, and by the same arrangement at any separation.

**R-2 - a route is empty only where no path exists.** Across the sweep, the only renderings drawing
two heads with a gap are those whose route is empty, and each of those is an endpoint standing on
the cell the route would have to arrive at, with the two heads orthogonally adjacent. Covers FR-003
and SC-001.

_Refuted by_: 340 of the 1856 renderings today, of which only 72 are the arrangement named.

**R-3 - nothing bounds where a path goes.** No arrangement's route is rejected for leaving the
rectangle its two starting positions span. Covers FR-001's last sentence.

_Refuted by_: the five families the spec names, all of which draw nothing today because the
rectangle was too thin to hold a path.

**R-4 - no family keeps a construction of its own.** `zigzag` is gone from `arrow.rs`, the
double-escape arrangements take four bends rather than six, and their two orders mirror each other.
Covers FR-007 and SC-005.

_Refuted by_: `(0, 0)` leaving `Left` with `(2, 1)` leaving `Right`, which today takes six bends and
draws the same picture from either end.

_How a test asserts it_: by counting bends in the two renderings and checking they mirror. The
absence of `zigzag` is read in review, not asserted - a test that greps its own crate is not a test.

**R-5 - a two-cell free span turns toward the endpoint the arrow leaves from.** Where the coordinate
the bends leave free spans exactly two cells, exchanging the two endpoints moves the turn to the
other of the two rather than leaving the picture unchanged. Covers FR-005 and SC-004.

_Refuted by_: `(0, 0)` leaving `Right` with `(3, 1)` leaving `Left`, which today turns at `x = 2`
for the first order and `x = 1` for the second - both pictures drawn the wrong way round, which is
issue 104.

_This is the one statement that must hold before and after the third commit unchanged_, which is
what research.md Q3 buys by repairing issue 104 first.

**R-6 - a mirrored route passes on the right of the travel.** Where two routes mirror each other
about the line the two starting positions share, the one drawn is the one on the side the arrow's
own travel puts to its right; coordinates grow rightward and downward. Covers FR-006 and SC-003.

_Refuted by_: nothing today, because the tie is unreachable while the rectangle bounds a route. It
becomes common the moment it does not, which is why this statement arrives with the rule rather than
before it.

**R-7 - no arrangement is left with two answers.** For every arrangement of the sweep, enumerating
every route that ties on bends, length and distance from the middle leaves exactly one surviving the
side the travel names. Covers FR-006's second sentence and User Story 1 scenario 8.

_How a test asserts it_: ADR-0048 measured this once over the whole sweep. As a standing test it is
the snapshot that carries it - a derivation with a tie left over is not deterministic, and a
non-deterministic derivation fails the snapshot rather than passing it quietly. This statement is
named here as a property the records establish, not as one a fast test re-proves each run.

**R-8 - the cost of deriving a route does not grow with distance.** The number of states the
derivation searches is equal for two endpoints four, fifty and five hundred cells apart, and never
exceeds 196. Covers FR-008 and SC-007.

_Counted, not timed_ - research.md Q2 has the arithmetic and the reason the spec's "the same number
of steps at either distance" needs the distances to be four or more.

**R-9 - the coincident-position family follows the rule.** Of the sixteen arrangements whose two
endpoints sit at one position, the four leaving in the same direction draw no route and the twelve
leaving in different directions draw one. Covers FR-012 and SC-010.

_How a test asserts it_: one named test for the whole family, because the sweep grid excludes
coincident positions and nothing else can reach them.

**R-10 - the sweep's window holds every route it draws.** Every cell any sweep arrangement writes
lies inside the window the sweep renders into. Covers FR-011 and SC-002.

_How a test asserts it_: mechanically, in the sweep itself, added in the widening commit where it
passes. It is what makes R-2 readable in the snapshot: without it, a picture showing two heads with
a gap could mean either an empty route or a clipped one.

**R-11 - what was pinned stays pinned.** Every picture asserted by feature 039's acceptance
scenarios, and the shipped command-line demonstration, renders unchanged. Covers FR-010 and SC-009.

_The one exception_, named by SC-009: `identical_directions_in_line_gives_an_empty_route`, whose
arrangement is joined by the rule. It is renamed and re-pinned rather than deleted - research.md Q6.

## What has no mechanical statement

SC-008 - that a person reading a description can say where its route runs without rendering it. The
spec names it as the one criterion with nothing behind it, and it is judged by reading _The route of
an arrow_ against the derivation. `Cost`'s four fields, in the model's order and named in the
model's words, are what that reading checks; nothing asserts it.
