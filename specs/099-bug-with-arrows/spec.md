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

- Q: When both endpoints sit at one position, so the two heads land on one cell, should a rule
  decide which head is visible, or is that arrangement accepted as order-dependent? → A: Accepted as
  order-dependent, and stated rather than left incidental: the `to` endpoint's head is drawn on top,
  so it is the one seen.
- Q: Is widening what bounds a route, so the 32 arrangements that draw nothing although their route
  rectangle has room would draw one, part of this slice? → A: No — a separate slice of its own. This
  spec repairs an implementation against a rule the model already states; that one changes the rule.
- Q: Where the route rectangle has an even span, so two routes bend equally and neither sits at the
  middle, should `docs/model.md` gain a rule naming which one wins? → A: No. FR-005 stands as it is:
  what it asks for is the identical picture from either end, not a named winner.
- Q: Should the 928-arrangement sweep ship as a test in the workspace, or stay the one-off
  measurement it is today? → A: It ships as a test over the whole grid, alongside the named
  acceptance scenarios, so SC-001 is a standing check rather than a number taken once.

## User Scenarios & Testing _(mandatory)_

### User Story 1 - The same arrow, described from either end (Priority: P1)

A person describes an arrow by giving its two endpoints. Which one they happen to write first is an
accident of how they were thinking about the diagram, not a statement about the picture: nothing in
_The route of an arrow_ in [`docs/model.md`](../../docs/model.md) gives the two endpoints different
roles. They expect to be able to exchange the two and get the same picture back.

Today they do not. The bug report's two descriptions differ only in which endpoint is written first,
and they render:

```text
◄────►
```

```text
◄   ─►──
```

The second is not another route: it is the same route placed in the wrong cells, drawn away from the
arrow instead of along it, and running over one of the heads on its way out.

**Why this priority**: it is the whole bug report, and it is not a cosmetic difference — over half
of the arrangements measured come out wrong, and what they draw is broken rather than merely
different.

**Independent Test**: describe one arrow, render it, exchange its two endpoints, render it again,
and compare the two pictures character for character. Over the whole 928-arrangement grid this is
the sweep SC-001 names, which ships as a test.

**Acceptance Scenarios**:

1. **Given** the first description in the bug report, **When** it is rendered, **Then** the picture
   is `◄────►`.
2. **Given** the second description in the bug report — the same arrow with its two endpoints
   exchanged — **When** it is rendered, **Then** the picture is `◄────►`, identical to the first.
3. **Given** an arrow whose route turns, for instance one endpoint at `(0, 0)` leaving `down` and
   the other at `(4, 3)` leaving `left`, **When** the two endpoints are exchanged, **Then** both
   renderings are the corner picture below, identical to each other.

   ```text
   ▲
   │
   │
   └───►
   ```

4. **Given** any arrow whose route runs toward a smaller coordinate on either axis, **When** it is
   rendered, **Then** every cell the route writes lies on the route, and no cell beyond either end
   of a run is written.
5. **Given** any arrow at all, **When** it is rendered, **Then** neither endpoint position is
   written by anything but its own head, and no position is written twice.

---

### User Story 2 - A route turns where the model says it turns (Priority: P2)

The same person places an arrow that has to turn more than once and expects the turn the model
names: _The route of an arrow_ says the route with the fewest bends, and among those the one that
turns at the middle of the route rectangle on whichever coordinate the bends leave free.

The picture the shipped command-line example pins does not do that. Its arrow leaves `(5, 0)` to the
right and arrives at `(9, 2)` from below, so its route rectangle spans `x` from 6 to 9 and the
middle of that span is 7. The route turns at `x = 6`:

```text
┌──┐ >┐
│░░│  │
└──┘  │  v
      └──┘
────
```

The model's rule names the route that turns at `x = 7`:

```text
┌──┐ >─┐
│░░│   │
└──┘   │ v
       └─┘
────
```

The two bend the same number of times and are the same length; only the model decides between them,
and the picture that is pinned is not the one it decides on.

**Why this priority**: it is the second half of the bug report — verifying the other arrow examples
against expectations — and it is what is left once User Story 1 is done. It changes a picture a
person already sees, but unlike User Story 1 what is drawn today is a route rather than a defect.

**Independent Test**: render the box, line and arrow example the command-line tool pins and check
where its route turns against the middle of its route rectangle, computed from the two endpoint
positions and leaving directions alone.

**Acceptance Scenarios**:

1. **Given** the box, line and arrow example, **When** it is rendered, **Then** its route turns at
   the middle of its route rectangle, so the picture is the second one above.
2. **Given** two routes that bend the same number of times and turn equally far from the middle of
   the route rectangle, **When** the arrow is rendered from either end, **Then** the same one of the
   two is drawn.
3. **Given** every arrow picture pinned by feature 039's acceptance scenarios, **When** they are
   rendered, **Then** each is unchanged.

---

### Edge Cases

- **Both endpoints at one position.** The route is empty and both heads are written to the same
  cell, so one head covers the other and exchanging the two endpoints exchanges which one is
  visible. This is the one arrangement where User Story 1 cannot hold for free, and it is accepted
  as order-dependent rather than ruled out of existence: the `to` endpoint's head is drawn on top,
  so it is the one seen. Both endpoints at `(2, 1)`, one leaving `right` with head `◄` and the other
  leaving `left` with head `►`, draw `►` when the `◄` end is named first and `◄` when it is not.
  Rendering that arrangement shows the opposite today — the `from` head survives — so FR-008 changes
  it. No test pins the cell, so SC-004 is unaffected.
- **An arrangement with no route.** Where no path fits inside the route rectangle the arrow is its
  two heads and nothing else — the model's answer, not an exception to it. Of the 928 arrangements
  measured, 170 draw no route. In 32 of them the route rectangle is more than one cell thick, which
  the cases listed in _The route of an arrow_ do not cover: two endpoints one row apart leaving in
  opposite directions away from each other, for instance `(0, 0)` leaving `right` and `(0, 1)`
  leaving `left`, draw two heads and nothing between them, because every path inside the rectangle
  runs over a head. Widening the bound so those 32 draw a route is out of scope here and belongs to
  a slice of its own: it changes what the model bounds a route by, where everything in this spec
  repairs an implementation against a rule the model already states. Until that slice exists the
  empty route is what those arrangements render, and SC-004 holds them unchanged.
- **A route that leaves the window.** An arrow whose route runs outside the rendered window is
  clipped exactly as it is today. Nothing here changes what a window shows.
- **An arrow whose two endpoints are adjacent.** The route between them is empty or one cell long;
  both orders must still agree.

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: An arrow's picture MUST NOT depend on which of its two endpoints is given first.
  Exchanging them, each keeping its own position, leaving direction and head glyph, MUST produce a
  character-for-character identical picture. The single exception is FR-008's arrangement, where the
  two heads occupy one cell and only one of them can be seen.
- **FR-002**: Every cell an arrow's route writes MUST be a position on that route. No cell beyond
  either end of a run may be written, in particular where a run travels toward a smaller coordinate.
- **FR-003**: An arrow MUST write each position at most once, and MUST NOT write either endpoint
  position other than as that endpoint's own head — the rule _The route of an arrow_ already states,
  and which the defect behind FR-002 breaks today.
- **FR-004**: The route drawn MUST be the one _The route of an arrow_ in
  [`docs/model.md`](../../docs/model.md) names, including where several routes share the fewest
  bends and the middle of the route rectangle decides between them.
- **FR-005**: Where that rule still leaves more than one route, the one drawn MUST be the same from
  either end, so that FR-001 holds without depending on which endpoint was named first. No rule
  naming which of them wins is added to _The route of an arrow_ for this slice: the requirement is
  the identical picture, and whichever route is drawn is pinned by the test that compares the two
  renderings.
- **FR-006**: Every arrow picture pinned by an acceptance scenario of feature 039 MUST render
  unchanged.
- **FR-007**: The box, line and arrow example the command-line tool pins MUST render with its route
  turning at the middle of its route rectangle. This is the one existing picture this slice changes,
  and it changes because it disagrees with FR-004.
- **FR-008**: Where both endpoints occupy one position, the route is empty and both heads are
  written to that cell. The `to` endpoint's head MUST be the visible one. This arrangement is
  therefore order-dependent by decision, and the decision is which head wins rather than whether one
  does. Rendering it today gives the `from` head instead, so this reverses an observed behavior that
  no test pins.

### Key Entities

- **Endpoint**: a position, a leaving direction and a head glyph, as _The initial set_ and _The
  route of an arrow_ define it. The two endpoints of an arrow are interchangeable; this slice is
  about making the drawing say so.
- **Route**: the path between the two starting positions, owned by _The route of an arrow_. This
  slice adds no rule to it; it requires that what is drawn is what that section already says.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: Over the 928 endpoint arrangements measured for this report — two anchor positions
  with four leaving directions each, against a six-by-five grid of positions with four leaving
  directions each — exchanging the two endpoints changes the picture in none of them, other than any
  arrangement whose two endpoints occupy one position, which FR-008 governs instead. It changes it
  in 502 of them today. The sweep ships as a test over the whole grid rather than staying a
  measurement taken once, so this criterion keeps being checked after the slice lands.
- **SC-002**: Both descriptions in the bug report render as `◄────►`.
- **SC-003**: The box, line and arrow example renders with its route turning at `x = 7`.
- **SC-004**: Every test in the workspace that passes today still passes, with exactly one picture
  changed: the one SC-003 names. FR-008 changes a second rendering — the cell where two heads
  collide — but no test pins it, which was checked by reading the one test that renders that
  arrangement, `both_endpoints_at_the_same_position_returns_normally`, and finding it asserts only
  that the call returns.
- **SC-005**: A person reading two descriptions of one arrow can tell, without rendering either,
  that they draw the same picture, because the only difference between them is the order of two
  things the model gives no order to.

## Assumptions

- The two endpoints carry no meaning beyond their position, leaving direction and head glyph, in
  everything that decides where the route runs. This is the bug report's premise and the model's: an
  endpoint that later chose its own glyphs would choose them from itself, not from having been
  written first. FR-008 is the one place the `from` and `to` roles decide anything, and what they
  decide there is which of two heads on one cell is seen — never a position.
- Where a picture pinned by an earlier feature disagrees with _The route of an arrow_, the model
  wins and the picture is corrected. FR-007 is the only place that applies.
- Head glyphs stay the caller's choice, per _An end is an arm; a head is a glyph_. Nothing here
  changes which glyph a head is drawn as.
- The numbers quoted above — 502 of 928 arrangements order-dependent, 170 with no route, 32 of those
  with a route rectangle more than one cell thick, and both pictures of the box, line and arrow
  example — were taken by rendering, not estimated.
