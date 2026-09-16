# Contract: what an arrow draws

`monospace-core`'s public surface does not change in this feature. Feature 039's
[`contracts/public-api.md`](../../039-draw-shapes-instead-of-individual-cells/contracts/public-api.md)
still describes `Arrow`, `Endpoint` and `Shape` exactly as they are, and no caller has to be
recompiled for a different reason than usual.

What changes is the observable behavior behind that surface. This file is that behavior written as
statements a test can make, one per rule, so each requirement of the spec has somewhere to point.
Each statement is about the text `render` produces for a buffer an `Arrow` has been drawn into with
`StampMode::Above`.

## The statements

**C-1 — a route lies on itself.** Every non-blank cell the arrow writes that is not one of the two
endpoint positions is a cell of the route _The route of an arrow_ names. Covers FR-002.

_Refuted by_: the bug report's arrow from its `(6, 0)` end, which today writes two cells to the
right of its starting position instead of the five to its left.

**C-2 — a head is the only thing at an endpoint.** Both endpoint positions render as the glyph their
own endpoint carries. Covers FR-002's second half.

_Refuted by_: 110 of the 1856 renderings today.

**C-3 — no cell is written twice.** Drawing the arrow into a surface that counts writes leaves every
position written at most once. Covers FR-002 and extends feature 039's
`no_pinned_arrow_writes_any_position_more_than_once` from ten arrangements to the whole grid.

**C-4 — exchanging the endpoints changes at most the free turn.** For an arrangement whose free
coordinate spans an odd number of cells, the two renderings are equal. For one spanning an even
number, they differ only in which of the two middle cells the free run sits on. Covers FR-001 and
FR-003.

_Refuted by_: 495 of the 928 arrangements today, of which the legitimate even-span differences are a
subset.

**C-5 — a free run sits at the middle.** For an arrangement with one free run, the run's fixed
coordinate equals the middle of the route rectangle on that run's axis, taken as the cell nearer the
`from` endpoint's starting position where the span is even. Covers FR-003 and, for the
demonstration, FR-006.

_Refuted by_: User Story 2's `n = 4` and `n = 5`, and by ADR-0044's pair rendered from its `▼` end.

**C-6 — the `to` head wins a shared cell.** Where both endpoints occupy one position, the rendered
glyph is the `to` endpoint's head. Covers FR-004. True today under `StampMode::Above`; pinned rather
than changed, per research.md Q5. `Arrow` does not control the stamp mode, so this statement is
about `Above` and says nothing about `Below`.

**C-7 — what was pinned stays pinned.** Every picture asserted by feature 039's acceptance
scenarios, and the whole output of `cargo run -p monospace-cli`, renders byte for byte as it does
today. Covers FR-005 and FR-006.

## How each is checked

| statement | checked by                                                                          |
| --------- | ----------------------------------------------------------------------------------- |
| C-1       | the snapshot, reviewed once against the model (SC-001)                              |
| C-2       | a sweep assertion over the whole grid, and the snapshot                             |
| C-3       | a sweep assertion over the whole grid, using the existing `CountingSurface`         |
| C-4       | a named test on the bug report's arrow (SC-002) and on ADR-0044's pair (SC-003)     |
| C-5       | a named test per row of User Story 2's table, and one on the demonstration (SC-004) |
| C-6       | one named test                                                                      |
| C-7       | the tests that already exist, unchanged, plus a new one on the demonstration's turn |

C-1 is the one no assertion can make: whether a picture is the route the model names is a judgment,
and the snapshot's job is to make a later change to that judgment visible rather than to make it
mechanically. That is what
[ADR-0045](../../../docs/decisions/0045-pin-every-arrow-arrangement-as-a-reviewed-snapshot.md)
records, and what the constitution's principle IV asks to be said out loud rather than described as
tested.
