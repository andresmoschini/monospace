# Feature Specification: An arrow draws the same whichever endpoint is named first

**Feature Branch**: `099-bug-with-arrows`

**Created**: 2026-09-16

**Status**: Draft

**Input**: User description: "The _from_ and _to_ endpoints in an arrow does not have meaning yet
(maybe in the future to choose the glyphs). So, result of these two diagrams should be the same [the
same arrow described with its two endpoints exchanged]. But they are not. Also, we should review and
verify other arrow examples to be sure that implementation meets expectations."

## Clarifications

### Session 2026-09-16

- Q: Should "the same picture from either end" be read literally? → A: No. Where the middle of the
  route rectangle falls between two cells the rule leaves two routes level, and both are correct
  pictures of the same arrow. The endpoint order decides which one is drawn, and it decides nothing
  else about a route. This reverses an earlier answer in this session — "FR-005 stands as it is:
  what it asks for is the identical picture, not a named winner" — which was taken before the
  even-span case was on the table.
- Q: Where does that tie-break live, given the model presents the turn as determined? → A: In the
  model, amended by
  [ADR-0044](../../docs/decisions/0044-let-the-endpoint-order-break-a-tied-route.md). The route
  turns at the cell nearer the endpoint the arrow leaves from.
- Q: When both endpoints sit at one position, so the two heads land on one cell, should a rule
  decide which head is visible? → A: Yes, and it is the same rule: the arrow travels towards its
  `to`, so the `to` endpoint's head is drawn on top. This stopped being an exception when the
  tie-break above became the principle, and it is stated in the model rather than here.
- Q: Is widening what bounds a route, so the 32 arrangements that draw nothing although their route
  rectangle has room would draw one, part of this slice? → A: No — a separate slice of its own. This
  spec repairs an implementation against a rule the model already states; that one changes the rule.
- Q: How is the sweep over every arrangement checked, now that two orders may legitimately draw
  different pictures? → A: As a snapshot of all 1856 renderings, reviewed once against the rule and
  pinned as a test, so any later change shows up as a diff.

## User Scenarios & Testing _(mandatory)_

### User Story 1 - The same arrow, described from either end (Priority: P1)

A person describes an arrow by giving its two endpoints. Which one they happen to write first is an
accident of how they were thinking about the diagram, not a statement about the picture. They expect
both descriptions to draw the same arrow.

Today they do not. The bug report's two descriptions differ only in which endpoint is written first.
With one endpoint at `(0, 0)` leaving `right` with head `◄` and the other at `(6, 0)` leaving `left`
with head `►`, they render:

```text
◄─────►
```

```text
◄    ─►─
```

The second is not another route: it is the same route placed in the wrong cells, drawn away from the
arrow instead of along it, and running over one of the heads on its way out. The turning case is
worse. With `(0, 1)` leaving `down` with head `▲` and `(2, 6)` leaving `up` with head `▼`, naming
the `▲` end first draws the route:

```text
▲
│
└─┐
  │
  │
  ▼
```

and naming the `▼` end first draws this:

```text
▲
└─┐


  │
  ▼
  │
```

which breaks the route in two and writes a cell past the head.

**Why this priority**: it is the whole bug report, and it is not a cosmetic difference — over half
of the arrangements measured come out different, and what they draw is broken rather than merely
another correct route.

**Independent Test**: describe one arrow, render it, exchange its two endpoints, render it again,
and check both pictures against _The route of an arrow_ in [`docs/model.md`](../../docs/model.md).
Over the whole grid this is the sweep SC-001 names, which ships as a reviewed snapshot.

**Acceptance Scenarios**:

1. **Given** the bug report's arrow, **When** it is rendered from either end, **Then** the picture
   is `◄─────►` both times.
2. **Given** an arrow whose route turns once, for instance one endpoint at `(0, 0)` leaving `down`
   and the other at `(4, 3)` leaving `left`, **When** the two endpoints are exchanged, **Then** both
   renderings are the corner picture below, identical to each other, because the rule leaves only
   one route.

   ```text
   ▲
   │
   │
   └───►
   ```

3. **Given** the turning arrow above, whose free coordinate spans rows 2 to 5 so that the middle
   falls between two cells, **When** it is rendered from each end, **Then** each picture turns at
   the row nearer the endpoint it was described from: naming the `▲` end first draws the left
   picture and naming the `▼` end first draws the right one, and both are correct.

   ```text
   ▲        ▲
   │        │
   └─┐      │
     │      └─┐
     │        │
     ▼        ▼
   ```

4. **Given** any arrow whose route runs toward a smaller coordinate on either axis, **When** it is
   rendered, **Then** every cell it writes lies on the route, and no cell beyond either end of the
   route is written.
5. **Given** any arrow at all, **When** it is rendered, **Then** neither endpoint position is
   written by anything but its own head, and no position is written twice.

---

### User Story 2 - A route turns where the model says it turns (Priority: P2)

The same person places an arrow that has to turn and expects the turn the model names: _The route of
an arrow_ says the route with the fewest bends, and among those the one that turns at the middle of
the route rectangle on whichever coordinate the bends leave free.

Sometimes it does and sometimes it does not, which is worse than always being wrong: there is no
rule a person can learn from the pictures. Measured, with one endpoint at `(0, 0)` leaving `right`
and the other at `(n, 3)` leaving `left`, so that the free coordinate is the column of the vertical
run:

| `n` | route rectangle | the middle | turns at |
| --- | --------------- | ---------- | -------- |
| 4   | `x` 1 to 3      | 2          | 3        |
| 5   | `x` 1 to 4      | between    | 1        |
| 6   | `x` 1 to 5      | 3          | 3        |

At `n = 6` the rule is obeyed. At `n = 4` the route turns at the far edge of the rectangle rather
than at its middle. At `n = 5` it turns at the near edge, which is neither of the two cells the
middle falls between, so FR-003 does not excuse it either.

The shipped command-line demonstration is not one of the broken cases, which is why it looked
correct and why nothing caught this. Its arrow leaves `(13, 3)` to the right and arrives at
`(22, 4)` from below, so its route rectangle spans `x` from 14 to 22 and the middle of that span
is 18. It turns at `x = 18`, and both of its heads point away from the route, as _An end is an arm;
a head is a glyph_ describes them. Nothing about it changes here; what it lacks is a test saying so.

**Why this priority**: it is the second half of the bug report — verifying the other arrow examples
against expectations — and it is what is left once User Story 1 is done. Unlike User Story 1 what is
drawn today is a route rather than a defect: it is the wrong one of several correct routes.

**Independent Test**: render the three arrangements in the table and check where each turns against
the middle of its route rectangle, computed from the two endpoint positions and leaving directions
alone.

**Acceptance Scenarios**:

1. **Given** each arrangement in the table above, **When** it is rendered, **Then** its route turns
   at the middle of its route rectangle.
2. **Given** the shipped demonstration, **When** it is rendered, **Then** its arrow turns at
   `x = 18`, the middle of its route rectangle, and the rest of the picture is unchanged.
3. **Given** every arrow picture pinned by feature 039's acceptance scenarios, **When** they are
   rendered, **Then** each is unchanged.

---

### Edge Cases

- **Both endpoints at one position.** The route is empty and both heads are written to the same
  cell, so one covers the other. The `to` endpoint's head is the one seen, which is _The route of an
  arrow_'s rule rather than an exception to it: the arrow travels towards its `to`, which is also
  what decides a tied turn. Both endpoints at `(2, 1)`, one leaving `right` with head `◄` and the
  other leaving `left` with head `►`, draw `►` when the `◄` end is named first and `◄` when it is
  not. That is what rendering the arrangement shows today, under `StampMode::Above`: `Arrow` draws
  the `from` head and then the `to` head, and `Above` lets the second win. So this slice pins the
  cell instead of changing it. Under `StampMode::Below` the first head written is the one that
  survives and the `from` head is seen — a mode `Arrow` does not control, and a limit this slice
  names rather than removes.
- **An arrangement with no route.** Where no path fits inside the route rectangle the arrow is its
  two heads and nothing else — the model's answer, not an exception to it. Of the 928 arrangements
  measured, 170 draw no route. In 32 of them the route rectangle is more than one cell thick: two
  endpoints one row apart leaving in opposite directions away from each other, for instance `(0, 0)`
  leaving `right` and `(0, 1)` leaving `left`, draw two heads and nothing between them, because
  every path inside the rectangle runs over a head. Widening the bound so those draw a route is out
  of scope here and belongs to a slice of its own: it changes what the model bounds a route by,
  where everything in this spec repairs an implementation against a rule the model already states.
  The maintainer has sketched what such a route might look like — a path that wraps around outside
  the two endpoints, reaching a head from the side opposite its leaving direction — and those
  sketches are input to that slice, not requirements here. Until it exists the empty route is what
  those arrangements render, and SC-005 holds them unchanged.
- **A route that leaves the window.** An arrow whose route runs outside the rendered window is
  clipped exactly as it is today. Nothing here changes what a window shows.
- **An arrow whose two endpoints are adjacent.** The route between them is empty or one cell long,
  so no turn is free and both orders draw the same picture.

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: Describing an arrow from either end describes the same arrow, and both descriptions
  MUST draw a correct picture of it: the same head at each of the same two positions, and a route
  that _The route of an arrow_ in [`docs/model.md`](../../docs/model.md) allows, with the same
  number of bends and the same length. Which endpoint is written first MUST decide nothing beyond
  what FR-003 and FR-004 name.
- **FR-002**: The cells an arrow writes MUST be exactly the cells of the route the model names —
  every one of them and nothing besides. No cell beyond either end of the route may be written, and
  no endpoint position may be written by anything but its own head.
- **FR-003**: Where the middle of the route rectangle falls between two cells, the route MUST turn
  at the one nearer the endpoint the arrow leaves from. Exchanging the two endpoints therefore moves
  the turn to the other of the two, and that is the only difference the exchange may make to a
  route.
- **FR-004**: Where both endpoints occupy one position, the head seen MUST be the `to` endpoint's.
- **FR-005**: Every arrow picture pinned by an acceptance scenario of feature 039 MUST render
  unchanged.
- **FR-006**: The shipped command-line demonstration MUST render unchanged, and a test MUST pin
  where its arrow turns against the middle of its route rectangle rather than against a transcribed
  picture.

### Testing expectations

- The whole grid of arrangements ships as a snapshot: every arrangement rendered from both ends,
  reviewed once against the model, and pinned so a later change shows as a diff. It is the standing
  check behind SC-001, not a measurement taken once.
- The named acceptance scenarios stay separate from the snapshot and keep their pictures readable,
  written a row per source line rather than as one escaped string.
- Unit tests for the route derivation are the minimum; a rule above with no test named against it is
  unfinished.

### Key Entities

- **Endpoint**: a position, a leaving direction and a head glyph, as _The initial set_ and _The
  route of an arrow_ define it. The two endpoints are interchangeable in everything except a tied
  turn and a shared cell.
- **Route**: the path between the two starting positions, owned by _The route of an arrow_. This
  slice adds no rule to it beyond the tie-break ADR-0044 put there; it requires that what is drawn
  is what that section says.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: Every one of the 1856 renderings — 928 endpoint arrangements, two anchor positions
  with four leaving directions each against a six-by-five grid of positions with four leaving
  directions each, every one drawn from both ends — is a correct picture under _The route of an
  arrow_. Before this slice, 502 of the 928 arrangements drew a different picture when their
  endpoints were exchanged; under FR-003 some of those differences are legitimate, so that figure
  bounds the defects rather than counting them.
- **SC-002**: The bug report's arrow renders `◄─────►` from either end.
- **SC-003**: The arrow at `(0, 1)` leaving `down` and `(2, 6)` leaving `up` renders two pictures,
  one per order, differing only in which of rows 3 and 4 holds the turn.
- **SC-004**: Each of the three arrangements in User Story 2's table turns at the middle of its
  route rectangle, and the demonstration's arrow turns at `x = 18`.
- **SC-005**: Every test in the workspace that passes today still passes, and no rendering a test
  pins changes. Feature 039's scenarios 5 to 10 were checked by hand against the middle and each
  turns at a forced coordinate or at the middle, so none of them moves. The cell where two heads
  collide does not move either: it already shows the `to` endpoint's head, which is what FR-004 asks
  for, and the one test that renders that arrangement,
  `both_endpoints_at_the_same_position_returns_normally`, asserts only that the call returns.
- **SC-006**: A person reading a description can say where its route turns without rendering it,
  including which of two cells holds the turn when the middle falls between them.

## Assumptions

- The two endpoints carry no meaning beyond their position, leaving direction and head glyph in
  anything that decides where the route runs. The exceptions are the two ADR-0044 records, and
  neither of them moves a route: one chooses between two routes the rule already allows, the other
  chooses which of two heads on one cell is seen.
- Where a picture pinned by an earlier feature disagrees with _The route of an arrow_, the model
  wins and the picture is corrected. No such picture has been found; SC-005 says what was checked.
- Head glyphs stay the caller's choice, per _An end is an arm; a head is a glyph_. Nothing here
  changes which glyph a head is drawn as, including whether it points away from its route — the
  demonstration's two do, and that is how it was written rather than something the core enforces.
- The numbers quoted above — 928 arrangements with 502 order-dependent and 170 drawing no route, 32
  of those with a route rectangle more than one cell thick, the three turns in User Story 2's table,
  and the demonstration's turn at `x = 18` — were taken by rendering. The 502 and the 170 were
  measured before this revision and have not been re-taken since; everything else in this spec was
  rendered while writing it.
