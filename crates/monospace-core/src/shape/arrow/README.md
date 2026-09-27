# The private project's route, ported

**This is a spike, and it is not merged.** It exists so the answer is written down and can be read
without redoing the work. Nothing here is a decision about how an arrow routes, and nothing here is
part of the model. See _Status_ at the end for why the pull request was closed.

## What the question was

Two ways of deriving an arrow's route are in play, and only one of them is in this repository:

- **What this repository has.** A search over the lines a turn can sit on, minimized by a
  [`Cost`](https://github.com/andresmoschini/projects/2/blob/main/crates/monospace-core/src/shape/arrow.rs)
  of four ordered terms. Recorded in
  [ADR-0049](../../../docs/decisions/0049-derive-a-route-by-searching-the-lines-a-turn-can-sit-on.md).
- **What the private project had.** A construction: chase one cell past each head, then join the two
  cells past those with whichever of six shapes fits. No search, no cost function, about a hundred
  lines of arithmetic.

The question was whether the second one draws anything the first does not, and if so whether it was
worth having as a rule a caller could choose. That is a fair question to ask and it has an answer.

**The construction ported here is the private project's, not this repository's.** That is the one
thing to keep straight, because [#106](https://github.com/andresmoschini/projects/2/issues/106) asks
for a _different_ old route back — the `zigzag` / `three_waypoint` / `path_bends` construction this
repository deleted in
[ADR-0049](../../../docs/decisions/0049-derive-a-route-by-searching-the-lines-a-turn-can-sit-on.md)
and recoverable at `727bd3b`. Two adjacent questions, two different constructions, and this spike
answers neither #106 nor any part of it.

## What is here

| Path                                           | What it is                                                          |
| ---------------------------------------------- | ------------------------------------------------------------------- |
| [`shapes.rs`](./shapes.rs)                     | The port, shape for shape, naming the original function in each doc |
| [`arrow.rs`](./arrow.rs)                       | The comparison, the measurements and the eight snapshot tests       |
| [`snapshots/spike/`](../../../snapshots/spike) | Both routes side by side, one file per anchor                       |

`shapes.rs` is `#[cfg(test)]` and `Arrow` still has three fields, so **nothing outside a test can
reach any of this**. That was deliberate: adding a field to `Arrow` is a structural change, and a
strategy a caller can choose is observable outside this module, which makes it a domain-level
decision with an ADR — not something a spike settles.

## The port is faithful, and that was checked rather than assumed

The only ground truth the original has is its own `bent_lines.tests.rs`, so all twenty-three
pictures it pins are transcribed as a test and the port reproduces every one. Two things about them
are worth recording because they are easy to get wrong:

- The original's `WorldWindow::to_string` renders by the extent of what was written rather than by
  the window it was given, so one of its twenty-three declares a window one row shorter than the
  picture it pins. `Buffer` clips at the window, so the transcription uses the three rows the
  picture needs. The shape is under test, not the window.
- The guards are transcribed including the two that can never fire and the one intermediate
  coordinate that does not reach the picture, so the file can be read against the original line by
  line. What the original _cannot_ express — a path that visits no cell twice and crosses no head —
  is the whole of the difference between the two routes, and it is the reason the port emits
  fragments rather than a path.

The terminals are not ported. The comparison draws the same two heads under both routes, so the
route is the only thing that varies.

## What was found

The comparison covers the same range as the characterization: 928 arrangements, each rendered from
both ends, 1856 renderings.

```text
arrangements rendered from both ends: 1856
  the two routes draw the same picture:            1063
  they differ, and only the ported route drew one: 84
  they differ, and both drew one:                 709
  the ported route wrote over a head, losing it:  85
  the ported route's pieces, by count:  [(1, 1341), (2, 515)]
  the ranked route's pieces, by count:  [(0, 84), (1, 1772)]
```

**The construction is never better.** Asking it the ranking's own question — its own first two terms
are fewest bends and then shortest — over the 1341 arrangements where the ported route is a
connected route at all:

```text
  the ported one turns fewer times:   0
  the ported one turns as often and is shorter:  0
```

Zero of 1341. There is no arrangement the construction wins, which is the answer to the question it
was built to ask.

**It leaves the arrow broken in 515 of 1856.** The original's dispatch maps the six pairs of
_different_ leaving directions onto six shapes, and the four pairs of _equal_ directions match none
of them. The corners at the two heads are drawn either way, so what comes out is two one-cell stubs
with a gap between them. That is a quarter of all arrangements, and it is a property of the
algorithm rather than of the port.

**It destroys the arrowhead in 85.** The original's terminal was a `Cell::Char` and its route a
`CellObject`, and `Cell::merge` answers `(CellObject, _)` with the front one — a corner beats a
character, which is what `Buffer::stamp` does here too. The port inherits the consequence; it does
not invent it.

**And where both routes are connected, 278 still differ.** The comparison's own measure of
"connected" counts cells that touch along an edge, and these touch _around_ a head:

```text
(2,0) Down -> (0,0) Up        ranked: 4 bends over 5 cells
                              ported: 4 bends over 5 cells

      ┌┐                           ┌┌
      ▼│▲                          ▼│▲
       └┘                           ┘┘
```

Same bends, same length, and the ported route passes on the wrong side of the head: the top `┌┌` and
the bottom `┘┘` have nothing joining them. So the construction is worse than the 515 alone says.

## Two hypotheses of ours that the measurements killed

Recorded because both are tempting enough to propose again.

**That the construction is direction-independent, and so could donate a symmetric tie-break.** Its
`bridge_horizontal` sorts by `x` and `bridge_vertical` by `y` before choosing where to turn, so the
staircase lands on the same cell whichever end named it first. Measured, the ranked route draws a
different picture from the other end in 156 arrangements and the ported one in 169. Neither is
direction-independent — `chase` is directional. There is nothing to copy.

**That the lattice carries candidates nobody uses.** `Lattice::axis` offers seven per axis, and the
module's own `Design notes` call the claim that a route turns only on those lines "confirmed against
an unrestricted search rather than proved", with a standing instruction to re-run it against any new
term. A first tally appeared to show two of the seven unused, and that was our own labelling: a line
that is `s + 1` and `t` at once was counted under one name only. Asked in a form that cannot be
mislabelled:

```text
  the from start                      779      beside the from start, inward   577
  the to start                        779      beside the to start, inward     443
  the middle                          650      beside the from start, outside  257
                                               beside the to start, outside    201

turns falling outside the five-line set: 52
```

All seven earn their place, and dropping the two "inward" candidates loses 52 turns. The reduction
is dead, with the number on it.

## What is left that is worth anything

Not a route. Three things, none of which is code:

1. **An independent second opinion for a debt the module already declares.** The `Design notes` say
   the lattice's completeness is confirmed rather than proved and that a term added to `Cost` which
   the lattice knows nothing about "is exactly how that confirmation goes stale. Re-run it against
   any new term." That was an instruction with no way to carry it out. There is now a second
   derivation that is not lattice-bounded, and the verdict is zero — so re-running it is a command
   rather than a hope.
2. **A measurement of how much the search buys.** 278 arrangements where the ranked route is better
   with a route of its own, against 84 where it correctly declines to draw one at all. That is the
   argument for keeping the search, and it was not written down anywhere before.
3. **A one-sentence statement of the rule is not available for the ranking.** The private project's
   construction is describable in one sentence — keep going on your axis while you are still closing
   on the other end, turn when you are not, then join the two ends with whichever of six shapes
   fits. The ranking's four terms are four paragraphs, and it is the more complex of the two models
   on every measure above. The honest reading is that the search earns its complexity, and the
   documentation is the part that is expensive.

## Running it

```sh
cargo test -p monospace-core -- --nocapture shapes_route
```

The measurements print rather than assert, on purpose: a count over a spike is the spike's output
and pinning it would turn a measurement into a contract. The side-by-side pictures are eight insta
snapshots in [`snapshots/spike/`](../../../snapshots/spike), one file per anchor and leaving
direction, each block labelled `same` or `DIFF`.

## Status

The pull request that carried this is **closed unmerged**. The reason is the size: the eight
side-by-side snapshots are 27,840 of the roughly 29,000 lines, they exist so the differences can be
read rather than counted, and that reading has been done. Keeping them in `main` would add a
characterization nobody maintains, in a directory the gate's `editorconfig` and `cspell` steps
reach, for a second implementation that loses every comparison it takes.

What survives is this README and the branch it is on. If a future increment wants the alarm in _What
is left_, point 1, the port is a `refactor` away from being lifted out of `#[cfg(test)]` and the
eight snapshots away from being deleted — and the measurement to re-run is the one line above that
prints zero.
