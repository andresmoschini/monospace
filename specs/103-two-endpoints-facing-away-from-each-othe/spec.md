<!-- The feature branch is named after the issue title, truncated by `cargo xtask spec`. -->
<!-- cspell:ignore othe -->

# Feature Specification: An arrow's route is ranked rather than bounded

**Feature Branch**: `103-two-endpoints-facing-away-from-each-othe-spec`

**Created**: 2026-09-18

**Status**: Draft

**Input**: User description: "Two endpoints that face away from each other and happen to be exactly
aligned on the axis they face along currently render as two disconnected heads, with nothing drawn
between them at all — even though they can be arbitrarily far apart. [...] What matters is that two
endpoints facing away from each other, however far apart, should have _some_ line connecting them —
not a gap. [...] as long as the result is one simple rule rather than a pile of special cases for
each geometry."

## What this slice implements

The behavior is settled before this spec rather than by it. Four records were taken first, and _The
route of an arrow_ in [`docs/model.md`](../../docs/model.md) was amended to match them in the same
increment:

- [ADR-0046](../../docs/decisions/0046-rank-a-route-instead-of-bounding-it.md) — the route rectangle
  stops bounding a route; a ranking of fewest bends, then shortest, decides it.
- [ADR-0047](../../docs/decisions/0047-let-a-route-cross-no-cell-twice.md) — a route visits no cell
  twice, which is what declines the loop for two endpoints at one position.
- [ADR-0048](../../docs/decisions/0048-let-the-travel-pick-the-side-of-a-mirrored-route.md) — the
  last tie, extending
  [ADR-0044](../../docs/decisions/0044-let-the-endpoint-order-break-a-tied-route.md).
- [ADR-0049](../../docs/decisions/0049-derive-a-route-by-searching-the-lines-a-turn-can-sit-on.md) —
  the derivation follows the ranking rather than a list of shapes.

This spec adds no rule to those. It says what a person sees change, and what must hold once it has.

Three commits land in order, each green and separately reviewable in the sweep's diff: the sweep's
render window is widened first so that no later diff hides a route behind a clip, then issue 104 is
repaired, then the rule replaces the bound. Issue 104 comes ahead of the rule and in a commit of its
own because it lives in the derivation the rule replaces. Issues 105 and 106 are not part of it: the
first moves the whole connection into a shape of its own and follows this work, and the second
brings the replaced construction back as a rule a shape can choose.

## Clarifications

### Session 2026-09-18

- Q: Between the compact six-bend route and the wider four-bend one that wraps around the outside,
  which picture should the rule draw? → A: The wider four-bend one. The maintainer prefers it on its
  own merits, and the deciding criterion is the algorithm with the fewest exceptions rather than the
  smaller picture, which is the ranking of ADR-0046 unchanged.
- Q: Is "the side the arrow's travel puts to its right" the intended handedness, given that the
  mirror-image rule would serve equally well? → A: Either is valid and the choice is arbitrary;
  ADR-0048's `right` stands as written.
- Q: Should "one rule, and the same rule everywhere" be a user story of its own or part of the story
  it follows from? → A: Part of it. The double escape and the mirrored side are the same ranking as
  the repair of the gap and arrive in the same commit, so they are folded into User Story 1 rather
  than sequenced after it.
- Q: Should the sweep's render window be widened so that every route it draws is visible, or stay as
  it is with the twelve clipped renderings documented? → A: Widened, until nothing clips. A snapshot
  whose job is to make a change visible cannot hide twelve of the changes it exists to catch, and a
  rendering showing two heads with a gap must mean one thing. The widening moves every picture in
  the snapshot, so it lands first and in a commit of its own, before either behavioral change.
- Q: Must the arrangements whose two endpoints sit at one position but leave in different directions
  be pinned by a test? → A: Yes, by one named test for the whole coincident-position family — four
  draw no route and twelve draw one. The sweep grid excludes coincident positions and stays as it
  is, so a named test is the only thing that can cover them.

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Endpoints facing away are joined, by one rule everywhere (Priority: P1)

A person places two endpoints facing away from each other on one line and expects a line between
them. Today there is nothing between them at all, however far apart they are, and the picture looks
like two unrelated heads that happen to share a column.

With one endpoint at `(0, 0)` leaving `up` and the other at `(0, 2)` leaving `down`:

```text
▼

▲
```

There is no rule a person can learn from that. The two heads point away from each other because that
is what leaving `up` and leaving `down` means, and a route that joins them has to go around the
outside — which the route rectangle, one cell thick here, had no room for. The same happens wherever
the rectangle is too thin for any path, and that is not one family but five, from two endpoints in
line facing away to two whose starting positions are not even collinear. Measured over the reviewed
sweep, 340 of the 1856 renderings draw no route at all.

After this slice the same arrow is joined:

```text
┌┐
▼│
 │
▲│
└┘
```

The same person also wants to predict a picture without rendering it, and the bound put two
exceptions in the way of that. Both go with it, in one change and one commit, which is why they
belong to this story rather than to a story after it.

The first is a family that had a shape of its own. Feature 099 found 17 arrangements whose route
needed seven runs rather than five, reached "only when the ordinary escape degenerates", and gave
them a **double escape** — because the run it needed had no room inside the rectangle. With one
endpoint at `(0, 0)` leaving `left` and the other at `(2, 1)` leaving `right`, that is six bends
where four are available around the outside, and it draws the same picture from either end when the
two orders should mirror:

```text
before                            after, from each end

┌►┌─┐                             ┌───┐        ┌►
└─┘◄┘                             └►  │        │  ◄┐
                                     ◄┘        └───┘
```

The two wider pictures are the ones to draw, and not only because fewest bends yields them: they are
what the rule says with no exception written beside it.

The second is that where two routes mirror each other about the line the two starting positions
share, neither is nearer the endpoint the arrow leaves from, so
[ADR-0044](../../docs/decisions/0044-let-the-endpoint-order-break-a-tied-route.md) leaves them
level. That tie is unreachable while the rectangle bounds a route and becomes common the moment it
does not — this story's own first picture is one of them. ADR-0048 settles it: the route passes on
the side the arrow's own travel puts to its right, and coordinates grow rightward and downward, so
leaving `up` puts the larger `x` to the right. Either hand would have served; the model names one so
that a reader can say which side the detour is on.

**Why this priority**: it is the whole of issue 103, and what it repairs is not a choice between two
correct pictures — it is a picture with a gap in it where a person asked for a connection. Neither
of the two exceptions is a gap on its own, and together they are what makes the result "one simple
rule rather than a pile of special cases for each geometry", which is the requirement issue 103
states about the shape of the answer rather than about the picture.

**Independent Test**: describe the arrow above, render it, and check that every cell between the two
heads is on one unbroken route that reaches each head against that head's own leaving direction;
render the double-escape arrangement from both ends and count the bends in each picture; render an
arrangement whose two routes mirror and check which side each is on against the direction its `from`
endpoint leaves in. Over the whole grid this is the sweep of
[ADR-0045](../../docs/decisions/0045-pin-every-arrow-arrangement-as-a-reviewed-snapshot.md), whose
reviewed snapshot moves with the change.

**Acceptance Scenarios**:

1. **Given** one endpoint at `(0, 0)` leaving `up` and one at `(0, 2)` leaving `down`, **When** the
   arrow is rendered, **Then** the two heads are joined by a route that leaves each endpoint in its
   own leaving direction and wraps around the outside, as pictured above.
2. **Given** two endpoints on one line leaving in the _same_ direction — `(0, 0)` and `(4, 0)` both
   leaving `right` — **When** the arrow is rendered, **Then** they are joined rather than left as
   two heads with a gap.

   ```text
   ◄──┐◄┐
      └─┘
   ```

3. **Given** any of the arrangements above with the two endpoints moved further apart, **When** the
   arrow is rendered, **Then** it is still joined, and the picture is the same shape with longer
   runs.
4. **Given** every arrangement of the sweep, **When** each is rendered from both ends, **Then** the
   only renderings that draw two heads with a gap between them are the ones whose route is empty,
   and in each of those the two heads are orthogonally adjacent so nothing looks disconnected.
5. **Given** the double-escape arrangement above, **When** it is rendered from either end, **Then**
   the route has four bends, and the two orders mirror each other.
6. **Given** one endpoint at `(0, 0)` leaving `up` and one at `(1, 2)` leaving `down`, **When** the
   arrow is rendered, **Then** its route takes four bends around the outside:

   ```text
   ┌┐
   │▼
   │
   │ ▲
   └─┘
   ```

7. **Given** one endpoint at `(0, 0)` leaving `down` and one at `(0, 2)` leaving `down`, **When**
   the arrow is rendered, **Then** the route passes on the side the travel from the first to the
   second puts to the right:

   ```text
    ▲
   ┌┘
   │▲
   └┘
   ```

8. **Given** any arrangement of the sweep, **When** every route that ties on bends, length and
   distance from the middle is enumerated, **Then** exactly one survives the side the travel names,
   so no arrangement is left with two answers.

---

### User Story 2 - A two-cell free span turns toward the endpoint the arrow leaves from (Priority: P2)

_The route of an arrow_ says the route turns at the cell nearer the endpoint the arrow leaves from
where the middle falls between two cells, so the same arrow described from its other end turns at
the other of the two. Where the coordinate the bends leave free spans exactly two cells it does the
reverse: it turns away from that endpoint, and both orders draw the same picture instead of
mirroring each other.

With one endpoint at `(0, 0)` leaving `right` and the other at `(3, 1)` leaving `left`, the free
span is `x` 1 to 2, so the model names `x = 1` for the first order and `x = 2` for the second. Both
are drawn the other way round:

```text
(0, 0) leaving right first        (3, 1) leaving left first

◄─┐                               ◄┐
  └►                               └─►
```

Measured over the reviewed sweep, 16 of the 1856 renderings, in 8 arrangements taken from both ends.

**Why this priority**: it is issue 104, it predates issue 103, and it is observable on its own — but
it lives in the derivation User Story 1 replaces, so repairing it afterwards would mean repairing
code that no longer exists. It is delivered first, in a commit of its own, so that the two changes
are separately reviewable in the sweep's diff.

**Independent Test**: render the arrangement above from both ends and check each picture against the
middle of the free span, computed from the two starting positions alone. It passes or fails without
any of User Story 1.

**Acceptance Scenarios**:

1. **Given** the arrangement above, **When** it is rendered from each end, **Then** the first turns
   at `x = 1` and the second at `x = 2`, so the two pictures mirror each other.
2. **Given** any arrangement whose free span is wider than two cells, **When** it is rendered,
   **Then** its turn is where it is today.

---

### Edge Cases

- **An endpoint standing on the cell the route would have to arrive at.** The path has nowhere to
  begin, or nowhere to end, so the route is empty and the arrow is its two heads — the rule's own
  answer rather than an exception to it. One endpoint at `(0, 0)` leaving `up` and one at `(0, 1)`
  leaving `up` draw `▼` above `▼`. This is what is left of the family User Story 1 repairs: 36 of
  the 928 arrangements, down from 170, and in every one of them the two heads are orthogonally
  adjacent, so nothing looks disconnected.
- **Both endpoints at one position leaving the same direction.** The two starting positions are one
  cell and a path would have to return to where it began, so the route is empty. ADR-0047 declines
  the loop rather than deferring it: drawing it needs a route cell with three arms, and no fragment
  opens on three sides. Four arrangements, none of them in the sweep grid, and none of them changes
  from what it renders today.
- **Both endpoints at one position.** The two heads fall on one cell and the `to` endpoint's is the
  one seen, exactly as feature 099 pinned it. Nothing here moves that. What the row above leaves
  open is the route, and the rule answers it without a guard: the two starting positions differ
  wherever the two leaving directions do, so a path exists and the twelve such arrangements draw a
  route around the shared head. ADR-0049 compared all sixteen against a search that assumes nothing
  and found the loop the only divergence. They are outside the sweep grid, so FR-012 names the test
  that pins them.
- **A route that leaves the window.** A route that wraps around the outside reaches further than the
  one it replaces, so an arrangement that fitted its window may no longer. It is clipped exactly as
  any shape is clipped today; the window is the caller's, and nothing here makes clipping an error.
  The sweep is one such caller, and its own window is widened so that none of its routes clips —
  twelve of its renderings would otherwise gain a route and still look like two heads. That is a
  property of the check rather than of the rule: a picture that shows two heads with a gap has to
  mean the route is empty, or the snapshot cannot be read.
- **Two endpoints very far apart.** A route between endpoints fifty cells apart is the same shape as
  one between endpoints one cell apart, and costs the same to derive
  ([ADR-0049](../../docs/decisions/0049-derive-a-route-by-searching-the-lines-a-turn-can-sit-on.md)).

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: The route an arrow draws MUST be the one _The route of an arrow_ in
  [`docs/model.md`](../../docs/model.md) ranks first: the fewest bends, then the shortest, then the
  free runs nearest the middle rounding toward the endpoint the arrow leaves from, then those runs
  on the side the arrow's travel puts to its right. Nothing may bound where a path goes.
- **FR-002**: Two endpoints facing away from each other along one line MUST be joined by a route,
  however far apart they are, and so MUST every arrangement for which a path exists at all.
- **FR-003**: The route MUST be empty only where no path exists, and after this slice that MUST be
  exactly two arrangements of the model's own naming: an endpoint standing on the cell the route
  would have to arrive at, and two endpoints at one position leaving the same direction.
- **FR-004**: A route MUST visit no position twice, and MUST pass through neither endpoint position.
- **FR-005**: Where the free span is exactly two cells, the route MUST turn at the cell nearer the
  endpoint the arrow leaves from, so exchanging the two endpoints moves the turn to the other of the
  two rather than leaving the picture unchanged.
- **FR-006**: Where two routes mirror each other about the line the two starting positions share,
  the one drawn MUST be the one on the side the arrow's own travel puts to its right. Together with
  FR-005 this MUST leave no arrangement of the sweep with two answers.
- **FR-007**: No family of arrangements may keep a construction of its own. The double escape MUST
  be gone from the rule and from the code. Five runs is then the most any route takes — a measured
  consequence of ranking the fewest bends first, down from seven, and not a limit the derivation may
  enforce: a cap would be the kind of bound FR-001 forbids.
- **FR-008**: The cost of deriving a route MUST NOT grow with the distance between the two
  endpoints.
- **FR-009**: The reviewed snapshot of every sweep arrangement MUST be regenerated and reviewed
  against _The route of an arrow_, so that the change is visible as a diff rather than asserted.
- **FR-010**: Every arrow picture pinned by an acceptance scenario of feature 039, and the shipped
  command-line demonstration, MUST render unchanged.
- **FR-011**: The sweep MUST render into a window large enough to hold every route the rule draws,
  so that a rendering showing two heads with a gap is one whose route is empty and nothing else. The
  margin that achieves this MUST be measured rather than assumed, and the widening MUST land in its
  own commit ahead of both behavioral changes, so that neither of their diffs is a whole-file
  reshuffle.
- **FR-012**: The sixteen arrangements whose two endpoints sit at one position MUST follow the rule
  rather than a guard of their own, and a test MUST pin which of them draw a route: the four that
  leave in the same direction draw none, and the twelve that leave in different directions draw one.
  They fall outside the sweep grid, so the snapshot does not cover them and a named test is the only
  thing that can.

### Testing expectations

- The sweep of ADR-0045 stays the standing check: 1856 renderings, reviewed once against the rule
  and pinned. Its mechanical assertions — every endpoint renders its own head, and no position is
  written twice — are what enforce FR-004 across the grid, and they MUST keep passing without an
  exemption.
- Each of the two user stories gets named tests of its own, separate from the snapshot, with their
  pictures written a row per source line rather than as one escaped string.
- A test MUST pin each of the two arrangements FR-003 names, so that "no route" is asserted where
  the rule says so rather than wherever it happens.
- A test MUST pin the coincident-position family of FR-012, which the sweep grid excludes, so that
  the twelve arrangements FR-002 covers by rule are covered by something that runs.
- A test MUST show that the derivation's cost does not grow with distance by counting the work
  rather than timing it, so that the check is deterministic and the gate does not acquire a test
  that fails on a slow machine.
- Unit tests for the route derivation are the minimum; a rule above with no test named against it is
  unfinished.

### Key Entities

- **Endpoint**: a position, a leaving direction and a head glyph. Its **starting position** is one
  step from it in its own leaving direction, and the route runs between the two starting positions.
- **Route**: the path between the two starting positions, owned by _The route of an arrow_. This
  slice implements the amended section; it adds no rule to it.
- **Route rectangle**: the smallest rectangle containing both starting positions. After this slice
  it bounds nothing and is only what the middle in the third term is measured within.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: Of the 1856 renderings of the sweep, the 340 that draw no route today fall to 72 — 36
  of the 928 arrangements — and every one of the 72 is an endpoint standing on the cell the route
  would have to arrive at, with the two heads orthogonally adjacent.
- **SC-002**: 268 renderings gain a route, and after the window is widened all 268 are visible in
  the snapshot; none is clipped.
- **SC-003**: The arrow at `(0, 0)` leaving `up` and `(0, 2)` leaving `down` renders two pictures,
  one per order, that mirror each other about the column the two endpoints share.
- **SC-004**: The arrangement at `(0, 0)` leaving `right` and `(3, 1)` leaving `left` renders two
  pictures, one per order, turning at `x = 1` and at `x = 2` respectively.
- **SC-005**: No rendering of the sweep takes more than five runs, down from seven, and the 34
  renderings that took a six-bend double escape take four bends.
- **SC-006**: Every rendering of the sweep whose _picture_ this slice changes is accounted for
  against the widened window: 306 in all, 16 for User Story 2 and 290 for User Story 1, and the
  remaining 1550 draw what they draw today. The widening commit that precedes them changes the
  snapshot text of all 1856 and the picture of none, which is what makes those two counts readable
  in the diff.
- **SC-007**: Deriving a route for two endpoints fifty cells apart costs what it costs for two
  endpoints one cell apart, counted rather than timed: the work the derivation does is the same
  number of steps at either distance, which is a figure a test can assert without being flaky.
- **SC-008**: A person reading a description can say where its route runs without rendering it,
  including which side a mirrored route passes on and which of two cells holds a turn. This is the
  one criterion here with nothing mechanical behind it — it is the requirement issue 103 states
  about the shape of the answer, and it is judged by reading _The route of an arrow_ against the
  code, not by a test that passes.
- **SC-009**: Every test in the workspace that passes today still passes, though not always against
  the same picture. Only `identical_directions_in_line_gives_an_empty_route` pins a picture this
  slice moves; feature 039's eleven scenarios and the command-line demonstration do not move.
- **SC-010**: Of the sixteen arrangements whose two endpoints sit at one position, four draw no
  route and twelve draw one, asserted rather than left to the rule.

## Assumptions

- The four records listed under _What this slice implements_ are taken, and _The route of an arrow_
  is amended to match them, on this same branch and ahead of this spec. Where this spec and that
  section disagree, the section wins and this spec is the defect.
- Where a picture pinned by an earlier feature disagrees with the amended section, the model wins
  and the picture is corrected. One such picture was found, and SC-009 names it.
- A route that reaches outside the rectangle the two starting positions span is acceptable. Measured
  over the sweep and over 4000 random arrangements up to fifty cells apart, no chosen route ever
  reaches more than one cell outside it (ADR-0046), so "around the outside" stays a picture a person
  can predict rather than an arbitrary detour.
- The numbers above were taken from the spike on branch `103-spike-route-rule`, two commits off
  `main` at `727bd3b` with the gate green on each — the repair of issue 104 and then the rule. The
  spike is a measurement and a review aid, not the implementation: the work goes through the three
  stages like any other feature. The figures 340, 306, 16, 290, 256 and 34 were re-derived while
  writing this spec by comparing the sweep's eight snapshot files across those two commits; 268, 12,
  36, the 4000 random arrangements and the cost claim behind SC-007 are ADR-0046's and ADR-0049's
  measurements, which this spec cites rather than re-takes.
- The spike rendered into today's window, so two figures here are not yet measured and are named as
  such. SC-002's claim that all 268 are visible follows from 256 plus the 12 ADR-0046 found clipped,
  and holds only once FR-011's margin is established; the margin itself is measured at
  implementation, not guessed here. SC-010's split of the sixteen coincident arrangements into four
  and twelve follows the rule and ADR-0049's comparison of all sixteen, and the twelve pictures are
  pinned when the test is written rather than asserted now.
- Head glyphs stay the caller's choice. Nothing here changes which glyph a head is drawn as, and a
  head that now has a route arriving beside it is drawn exactly as it was.
- Issue 105 — holding the connection in a shape of its own — follows this feature and is out of
  scope. So is issue 106, which brings the replaced construction back as a rule a shape can choose;
  what this slice deletes stays recoverable from the history at `727bd3b`.
