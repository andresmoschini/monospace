<!-- The feature branch is named after the issue title, truncated by `cargo xtask spec`. -->

# Feature Specification: A connector endpoint hangs from a box's side anchor

**Feature Branch**: `082-a-connector-endpoint-hangs-from-a-box-s-deciding` | **Issue**:
[#82](https://github.com/andresmoschini/monospace/issues/82) | **Created**: 2026-09-28 | **Status**:
Draft

**Input**: Issue #82, "A connector endpoint hangs from a box's side anchor" — every position in a
diagram is absolute, so putting a box somewhere else leaves the connector pointing at where it used
to be. This slice gives a box the four centers of its sides, lets a connector's endpoint be a
reference to one of them, and answers the two things that stop working the moment that is possible:
taking a referenced shape out, and displacing a figure that hangs from one. Both are answered with
the general rule the model already states, and each carries the issue that will ask for the answer
we actually want. Part of [#62](https://github.com/andresmoschini/monospace/issues/62), on top of
[#81](https://github.com/andresmoschini/monospace/issues/81).

## What this slice implements

Five sections of [`docs/diagram-model.md`](../../docs/diagram-model.md), none of them amended: the
model already describes this end to end, and a slice implementing a part of it says which part.

- [§4 _Positions_](../../docs/diagram-model.md#4-positions) — a position is an absolute point or a
  reference, and the two ways of failing to resolve are one thing. Left: the offsets (#83), and the
  two temporary answers B3 and B4 give.
- [§5 _Anchor points_](../../docs/diagram-model.md#5-anchor-points) — a box answers all four, a line
  answers them as a flat box, and a connector answers none. Left: the corners and the center (#90).
- [§6 _Attachment_](../../docs/diagram-model.md#6-attachment) — an endpoint's position may be a
  reference, and an endpoint is the only one that may be.
- [§9 _Changing a diagram_](../../docs/diagram-model.md#9-changing-a-diagram) — removal leaves every
  reference to the shape unresolved, which this slice makes observable for the first time.
- [§11 _Open questions_](../../docs/diagram-model.md#11-open-questions) — the question about
  displacing a figure that holds a reference stays open, and **What this slice does not decide**
  says why.

This spec restates none of the model and takes none of the decisions: how a reference is named, how
it is resolved and where the arithmetic lives are the sheet `/speckit-plan` part one writes, beside
[ADR-0040](../../docs/decisions/0040-let-each-shape-answer-its-own-anchor-points.md) and
[ADR-0041](../../docs/decisions/0041-resolve-a-position-through-a-reference.md), which hold the two
that matter.

## Clarifications

### Session 2026-09-28

- Q: The shipped demonstration grows a fifth captioned picture, or does `monospace-cli` stay
  untouched? → A: It grows one. The box it shows is the one the arrow already hangs from, displaced,
  with the connector landing on the new side. The demonstration names that box by the identity the
  diagram gave it when it was added — the third — and writes that value out in its own code rather
  than asking for it at runtime, so a reader of the code can see which shape the connector belongs
  to. `cargo xtask render` cannot fill it either way: a reference is not expressible in the
  description format, which is why the picture is built in the demonstration's own code.
- Q: A line is asked for the four side centers. §5 says it answers them as a flat box and B1 said it
  answered nothing. Which is it? → A: It answers all four, in this slice, and nothing in the model
  is amended to make that true. A horizontal line's top and bottom centers are its middle — one
  point asked for twice — and its left and right centers are its two ends; a vertical line is the
  same seen sideways. B1's "nothing" rule is a connector's now, and only a connector's. What is left
  of §5 is the corners and the center (#90), and #84 keeps the kinds this slice does not cover.
- Q: The spec stands at 243 attributable lines against a 120-line ceiling, and the two answers
  already recorded added twenty-nine of them. Split the slice now, or carry the overage? → A: Carry
  it, with the number and the reason written down when the plan is made. Splitting moves B1's eleven
  lines and leaves 232, so it buys a thinner slice rather than one that fits, and that is not worth
  a second deciding cycle with its own issue, branch, spec, decision sheet and pull request. Settled
  by: `plan.md`'s Complexity Tracking, which owns a ceiling exceeded without a split.
- Q: A shape is taken out and something else was hanging from it. What does the diagram do? → A: The
  general answer, and nothing new: the reference is left exactly as it was, it does not resolve, and
  the shape that held it is not drawn — no error, no report, nothing rewritten. It is not the answer
  we want, and [#142](https://github.com/andresmoschini/monospace/issues/142) is where what we do
  want is asked for: at the moment the referenced shape goes, the positions holding a reference to
  it become the absolute points they were resolving to.
- Q: A figure whose position is a reference is displaced. What does the diagram do? → A: Nothing.
  The displacement has no coordinates to add to, and a reference has no offsets until #83, so the
  figure does not move. It is a silent no-op, which is the one outcome a caller cannot tell from a
  bug. [#143](https://github.com/andresmoschini/monospace/issues/143) carries the real answer and
  depends on #83.

## Behavior

### B1 — A box and a line answer the four centers of their sides

1. **Given** a box, **When** each of the four side centers is asked for, **Then** each is the middle
   of that side, computed from the box's own position and size, and the four are four points rather
   than a rectangle's corners.
2. **Given** a line, **When** the same four are asked for, **Then** it answers them as a flat box:
   it has no thickness, so on a horizontal line the top and bottom centers are its middle, one point
   asked for twice, and the left and right centers are its two ends. A vertical line is the same
   seen sideways.
3. **Given** a connector, **When** any of the four is asked for, **Then** the answer is nothing, and
   nothing is a failure: a figure with no honest answer waits for a slice of its own rather than
   being given a fictional one.

### B2 — An endpoint hangs from a side, and comes with it

1. **Given** a connector whose `from` endpoint is a reference to a box's right side, **When** the
   diagram is drawn, **Then** the endpoint lands on the middle of that side, and the picture is the
   one a connector drawn from that absolute point produces:

   ```text
   ┌──┐
   │  ├──────►
   └──┘
   ```

   Hypothetical — measured, not generated. What today's code draws for a connector whose `from` sits
   at the point this reference resolves to, which is what the referenced diagram has to draw too.

2. **Given** that diagram, **When** the box is replaced by the same box four cells to the right,
   **Then** the endpoint lands on the new side, the free end stays where it was, and the route
   between the two is drawn as it is drawn for those two points:

   ```text
   ┌──┐
   │  ├──────►
   └──┘
   ```

   ```text
       ┌──┐
       │  ├──►
       └──┘
   ```

   Hypothetical — measured, not generated: the same connector drawn from the box's new side, with
   its free end at the point it was at before.

3. **Given** a connector with one endpoint absolute and the other a reference, **When** it is drawn,
   **Then** each endpoint is placed by its own rule and the connector is drawn between the two.
4. **Given** a reference naming a shape the diagram does not hold, or an anchor that shape does not
   answer, **When** the diagram is drawn, **Then** the whole connector is not drawn, every other
   shape draws exactly what it drew, and nothing errors, reports or panics.
5. **Given** a reference to an identity the diagram does not hold yet, **When** a shape is added
   under that identity, **Then** the connector draws, hanging from whatever that shape now is.

### B3 — Taking a referenced shape out leaves the reference alone

1. **Given** a connector hanging from a box and a second box that has nothing to do with it,
   **When** the first box is taken out, **Then** the reference is left exactly as it was, the
   connector draws nothing, and the second box draws what it drew:

   ```text
   ┌──┐
   │  ├──────►
   └──┘
             ┌──┐
             │  │
             └──┘
   ```

   ```text



             ┌──┐
             │  │
             └──┘
   ```

   Hypothetical — measured, not generated. The second window is the first with the referenced box
   taken out: the connector is gone rather than left dangling, because a shape whose position does
   not resolve writes nothing, and the box that had nothing to do with either is where it was.

2. **Given** that same diagram, **When** a box is put back under the removed one's identity,
   **Then** the connector draws again, hanging from the new box's side.
3. **Given** a caller who wants to know which shapes went missing, **When** they look for a way to
   ask, **Then** there is none: a diagram that draws nothing and a diagram whose every reference is
   broken look the same. That is the cost
   [ADR-0041](../../docs/decisions/0041-resolve-a-position-through-a-reference.md) records, and
   [#88](https://github.com/andresmoschini/monospace/issues/88) is where it is answered.

### B4 — Displacing a figure that hangs from another leaves the hanging end where it is

1. **Given** a connector whose endpoint is a reference, **When** the connector is displaced,
   **Then** that endpoint does not move: a displacement adds coordinates to an absolute position,
   and a reference has none to add to yet, so the connector draws exactly what it drew.
2. **Given** a connector with one endpoint absolute and the other a reference, **When** the
   connector is displaced two cells down, **Then** the absolute endpoint moves two cells down, the
   referenced one does not move at all, and the route is drawn between the two:

   ```text
   ┌──┐
   │  ├─────┐
   └──┘     │
            ▼
   ```

   ```text
   ┌──┐
   │  ├─────┐
   └──┘     │
            │
            │
            ▼
   ```

   Hypothetical — measured, not generated. The same connector twice, its free end two rows lower in
   the second: the endpoint on the box's side did not move, and the difference is in the picture
   rather than in a promise.

3. **Given** a box or a line, **When** it is displaced, **Then** its four side centers move with it
   and everything hanging from it moves with them, because what moved is the place the reference
   resolves to.

### B5 — The shipped demonstration shows a box moving with its connection

1. **Given** the shipped demonstration, **When** the application is run with no arguments, **Then**
   it prints one more captioned picture beside the four it prints today, in which the box the arrow
   already hangs from has been displaced and the connector hanging from it has landed on the new
   side. That box is named by the identity the diagram gave it when it was added — the third — and
   the demonstration writes that value out in its own code rather than asking for it at runtime, so
   a reader of the code can see which shape the connector belongs to. Without it a person running
   the application cannot see the one thing this slice is for.
2. **Given** a path, **When** the application is given one, **Then** it prints one picture and
   nothing else, exactly as today. That is what `cargo xtask render` embeds, and it is why the split
   exists.

## Edge cases

- A box one cell wide or one cell tall: its four side centers coincide in pairs, by the general rule
  rather than by an exception.
- An endpoint on a side whose border cell the connector's own arm composes with: the same rule two
  figures sharing a cell always obey, and a reference changes nothing about it.
- A reference to the shape that holds it: it draws nothing, because a connector answers no anchor
  point at all. That is the restriction doing its work — no chain is longer than one link, and no
  cycle can be built.
- A reference to a box that is then replaced by a line: the line answers its four as a flat box, so
  the connector lands on the line's own end or its middle, wherever the new line was put, rather
  than on the box's old side center. Replaced by a connector instead, which answers none of the
  four, it stops drawing — the same answer as a reference to a shape that was never there.
- Two connectors hanging from the same side: both land on the same cell and compose there as two
  figures sharing a cell always do.
- A figure displaced out of the window: what falls inside is drawn and the rest clipped, as any
  figure is.

## What this slice does not decide

- **What displacing a figure that holds a reference means.** B4's no-op is what this slice can do,
  not what §4 states: a displacement reaches the reference's offsets, and there are none until #83.
  Settled by: [#143](https://github.com/andresmoschini/monospace/issues/143), which cannot be
  answered before #83.
- **What taking a referenced shape out should do to what hung from it.** B3 is the model's general
  answer, and this slice makes it observable rather than choosing it. Settled by:
  [#142](https://github.com/andresmoschini/monospace/issues/142).
- **Whether a reference carries an offset at all.** §1 and §4 both describe one as three fields, and
  this slice's is the first two; the third is #83. Whether the model needs a line saying which
  fields have landed is a question for the decision sheet.
- **Whether a caller can ask a shape where one of its anchors is.** The anchors exist to be resolved
  through, and drawing is the only consumer so far. Settled by: the first consumer that needs an
  anchor without drawing the diagram.
- **Whether a caller can name an identity.** B5's demonstration writes the third one out rather than
  asking for it, and naming a value is a way of naming an identity: the diagram generates them today
  and nothing else can choose one, and the model's own trigger for answering that is the first slice
  reading a diagram from a file rather than this one. Settled by: the decision sheet, as the
  domain-level entry it is.
- **Whether the description format grows a field for a reference.** Nothing here needs one: the
  demonstration can build the picture in its own code, the way it carries the displacement's amount.
  Settled by: the first consumer that reads a diagram from a file rather than from code.
- **The two open questions that need a reference wider than an endpoint** — what displacing a figure
  whose own position does not resolve means, and whether an attachment decides the direction a
  connector leaves in. Settled by: [#89](https://github.com/andresmoschini/monospace/issues/89).

## Testing expectations

- **Contract** — a box's four side centers, each asked for and each compared with the absolute point
  the picture depends on; a line's, each asked for and each equal to the same point read as a flat
  box, its top and bottom centers being the one cell; a connector's, each asked for and answered
  nothing.
- **Contract** — a reference, by drawing: a connector hanging from a box's right side draws exactly
  what a connector drawn from that absolute point draws, and the box displaced, the connector
  following it and re-routing to the end that did not move. The two are the pairs B2 shows.
- **Contract** — non-resolution, by drawing, and the three cases ADR-0041 enumerates: a reference to
  an absent shape, a reference to an anchor a kind does not answer, and a connector with one
  endpoint that resolves and one that does not. Each asserts the connector is absent from the output
  and every other shape is unchanged.
- **Contract** — removal and re-adding under the same identity, by drawing: the pair B3 shows, and
  the connector drawing again afterwards.
- **Contract** — the displacement of a figure holding a reference, by drawing: the pair B4 shows, so
  the referenced endpoint standing still and the absolute one moving is a picture rather than a
  claim.
- **Contract** — the demonstration: five captioned pictures on a bare run, and a path still printing
  one picture and nothing else. The test pins the pictures, not the captions' wording.
- **One rule is accepted with nothing to verify it**, named as such per principle IV: a shape whose
  own position does not resolve offers no anchor point, so a reference to it resolves to nothing in
  turn. No position but an endpoint's can hold a reference yet — that is #89 — so the rule is
  unreachable from a test. It is in the model because that document is design intent, and it can be
  asserted the day a position may hold one.

## Success criteria

- **SC-001**: A connector's endpoint can be attached to a box's side rather than to a point, and
  draws the same picture an endpoint at that point draws.
- **SC-002**: Displacing a box takes every connector hanging from it along, and re-draws each route
  to the end that did not move.
- **SC-003**: An endpoint whose reference does not resolve takes the whole connector out of the
  picture, leaves every other shape exactly as it was, and fails no run.
- **SC-004**: Taking the referenced shape out leaves the reference alone, and putting a shape back
  under that identity makes the connector draw again.
- **SC-005**: Displacing a connector that hangs from a box leaves the hanging end exactly where it
  was while an absolute end moves — the difference between the two is in the picture rather than in
  a promise.
- **SC-006**: A bare run of the application shows a box displaced with its connection following it,
  so a person who runs it sees the slice's one claim without reading a test.
- **SC-007**: Given a path, the application still prints one picture and nothing else, which is what
  `cargo xtask render` embeds in a document.
- **SC-008**: `cargo xtask check` is green, including `monospace-diagram` compiling for
  `wasm32-unknown-unknown`.
