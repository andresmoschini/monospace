# Feature Specification: Draw shapes instead of individual cells

**Feature Branch**: `039-draw-shapes-instead-of-individual-cells`

**Created**: 2026-09-10

**Status**: Draft

**Input**: Issue #39, verbatim: "Currently the only way to draw anything is to write cell by cell,
computing every position and every character by hand at the call site. Even a rectangle means
working out eight separate runs and their corners. I want to describe a figure and have it draw
itself: a box, from a position and a size; a straight line, from a position, a length and an
orientation; an arrow between two endpoints, where each endpoint has a direction it leaves from — so
the route between them is derived, not spelled out by the caller. Shapes should compose — a box is
really just corners and lines placed correctly — and adding a new kind of shape should not mean
touching the existing ones."

## Source material

Two inputs, and they do not overlap.

- **Issue #39**, above, is the wish.
- **The maintainer's brief**, given when this command was invoked, is the constraint set: eighteen
  numbered capabilities, the acceptance pictures reproduced below, a list of questions to leave
  open, and a list of things it forbids this spec from settling because they belong to
  `/speckit-plan`. Every capability in it has a home in _Functional Requirements_; none is cited by
  number here, because a number is only useful when the reader can open the document it points into,
  and the brief is not a repository artifact.

The brief's forbidden list is what _Out of scope_ and _Handoff to the plan_ exist to keep out of
this file: whether a shape is a trait, an enum or a function, who receives the buffer, how a corner
or a head is represented, whether the complete/fragment distinction is a type or a convention, and
whether a route is resolved before or during drawing. None of them is decided here, and the absence
is deliberate rather than an omission.

## What the model owns

[`docs/model.md`](../../docs/model.md) owns the domain's design, and a slice needing a rule it does
not have amends it first — _The model owns the design_ in the constitution, and _Open questions_ in
the model itself. That amendment is done, and it is what this spec is written against rather than a
prerequisite still outstanding.

_Shapes_ is where the vocabulary this spec uses lives: shape, extent, piece and partition, the
complete and fragment distinction, the initial set of three figures, an arrow's starting positions
and route rectangle, and the rule that a degenerate arrangement follows the general rule rather than
an exception of its own. Arrows moved out of _Deliberately unresolved_ in the same amendment, and
the open question asking for the initial set of shapes is answered for these three.

So no requirement below defines any of those terms. Each names the section that does and states what
this feature owes against it. Where a requirement looks like a definition, it is a testable
obligation phrased in the model's words, which is the distinction the constitution draws between
naming a section and restating it.

## Clarifications

### Session 2026-09-10

- Q: May an arrow's route leave the rectangle its two endpoints span, when the leaving directions
  force it to go around? → A: Yes. The route starts one position from each endpoint in that
  endpoint's own leaving direction — outward when the direction points away from the other end — and
  stays inside the rectangle those two starting positions span. Nothing goes further out than a
  starting position, so the route reaches one cell beyond the endpoint rectangle on each side a
  direction points away from, and no further.
- Q: What does an arrow do when both endpoints are at the same position? → A: It is left
  indeterminate on purpose. The general routing rule decides it, as it decides every other
  degenerate combination, and no exception is written for it. What such an arrow draws is not pinned
  by this spec and is free to change when the rule changes; what is required is only that the call
  returns normally.
- Q: Which glyphs draw a line's ends and an arrow's heads? → A: The caller supplies them, as part of
  the line's and the arrow's description. They are chosen glyphs in the sense of feature 028, so no
  glyph set gains a rule and `docs/glyph-sets.md` is unchanged. Deliberately a scope cut: ends and
  heads that follow the glyph set are a later feature, and this answer does not preclude one.
- Q: What is the minimum valid length of a straight line? → A: None is declared. A line of any
  length is accepted and the general rule decides what it draws, on the same reasoning as the
  degenerate arrow: permit it first, and restrict it later if it turns out to cause problems. A
  length of 0 has an empty extent and so draws nothing; a length of 1 draws its one position, and
  which of the two end glyphs lands there is the rule's business and is not pinned here.
- Q: Where does a two-bend route bend when its span is even and the midpoint falls between two
  columns? → A: Not pinned. The general rule decides, and whatever it produces becomes the expected
  picture of the test that covers an even span. An arrow and its reverse are therefore permitted to
  differ by one column, and this spec does not require them to match.

## User Scenarios & Testing _(mandatory)_

The consumer throughout is **code calling `monospace-core`**, not a person at a terminal. There is
no user interface in this feature, and every scenario below is observed the same way: describe a
figure, draw it into an empty buffer, render that buffer, compare the text to a picture. _Rendering_
in [`docs/model.md`](../../docs/model.md) owns what that text is — a rectangle of exactly the
requested width and height, trailing spaces kept, a final newline — and the pictures below are read
against it rather than against a trimmed approximation of it.

Coordinates are the model's, from _The buffer_: `x` grows right, `y` grows down. The origin of every
picture below is the buffer's own origin.

Three stories, one per figure, in the order the brief prioritizes them: each adds exactly one thing
the one before it did not need, and each is a viable increment on its own.

### User Story 1 - A box draws itself (Priority: P1)

Calling code says "a box at this position, this size, optionally filled" and gets a box. It computes
no positions, writes no cells, and never names a corner character: the eight runs and four corners
that spec 0002 worked out by hand at the call site are now the box's own business. This is the
smallest figure that proves composition, because a box is genuinely made of other shapes — four
corners, up to four border runs, an interior — and the caller sees one shape.

**Why this priority**: it is the whole of the wish at its smallest. It is also the only story that
can be checked against something the repository already draws: the box `monospace-cli` prints today
is the same eight cells, stamped by hand, so the picture it produces is the fixed point this story
lands on.

**Independent Test**: draw a 6×3 box into an empty buffer and compare the rendered text to the
picture below; draw the same box filled and compare again; draw a 2×2 box, where every position is a
corner and no border run exists at all; ask for a box one cell wide and confirm the buffer stays
empty; and draw each of those against a buffer that counts writes per position, confirming none is
written twice.

**Acceptance Scenarios**:

1. **Given** an empty buffer at least 6 wide and 3 high, **When** a box at `(0, 0)` of size 6×3 is
   drawn and the buffer rendered over that rectangle, **Then** the text is exactly:

   ```text
   ┌────┐
   │    │
   └────┘
   ```

2. **Given** the same buffer, **When** the same box is drawn with a fill, **Then** the border is
   character for character the picture above, and the fill occupies exactly the 4×1 interior
   starting at `(1, 1)` — no fill outside it, and no border position replaced by fill.
3. **Given** an empty buffer, **When** a box at `(0, 0)` of size 2×2 is drawn, **Then** the text is
   exactly `┌┐` and `└┘` on two lines: four corners, no border run, no interior.
4. **Given** an empty buffer, **When** a box whose width or height is below 2 is drawn, **Then**
   nothing is drawn: every position of the buffer is still undefined, the rendered text is spaces,
   and the call returns normally.
5. **Given** a buffer that counts writes per position, **When** any box above is drawn, **Then** no
   position has a count above 1.
6. **Given** a filled box and an unfilled box with the same position and size, **When** each is
   asked what its extent is, **Then** the two extents are the same set of positions.

---

### User Story 2 - A straight line draws itself (Priority: P2)

Calling code says "a line from here, this long, horizontal" and gets a line that ends in something
reading as an end rather than as a segment cut short. This is the first figure whose pieces are not
fixed by its kind: a long line has an interior run and a short one does not, so the same shape
decomposes differently depending on its parameters.

**Why this priority**: it is the second thing the wish asks for, and it is what makes the box's
composition general rather than a one-off. It also has no dependency on the box: a line is drawable
and testable before the box exists, and a box built on top of it later changes nothing a line
promises.

**Independent Test**: draw a horizontal line of length 5 and compare the rendered text; confirm the
position one past the end holds no cell at all; draw a vertical line and compare; draw a line of
length 2, and one of length 1, and one of length 0, and confirm each returns normally and writes no
position more than once.

**Acceptance Scenarios**:

1. **Given** an empty buffer at least 6 wide, **When** a horizontal line at `(0, 0)` of length 5 is
   drawn, **Then** the first row is exactly `╾───╼`, the position `(5, 0)` holds no cell, and the
   position `(0, 0)` holds an end rather than a segment — the two are distinguishable in the text.
2. **Given** an empty buffer at least 4 high, **When** a vertical line at `(0, 0)` of length 4 is
   drawn, **Then** the four rows are an end, a segment, a segment and an end, in that order, and
   `(0, 4)` holds no cell.
3. **Given** an empty buffer, **When** a line of length 3 is drawn, **Then** it is two ends with
   exactly one segment between them.
4. **Given** an empty buffer, **When** a line of length 2 is drawn, **Then** it is the two end
   glyphs side by side with no interior between them.
5. **Given** an empty buffer, **When** a line of length 1 is drawn, **Then** the call returns
   normally and exactly one position is written. Which of the two end glyphs it holds is the general
   rule's business and is not asserted: no minimum length is declared, and a length below 2 is
   permitted rather than rejected.
6. **Given** an empty buffer, **When** a line of length 0 is drawn, **Then** its extent is empty,
   nothing is drawn, and the call returns normally.
7. **Given** a buffer that counts writes per position, **When** any line above is drawn — the
   length-1 line included — **Then** no position has a count above 1. No guard produces that at
   length 1: FR-011's partition gives the single position to exactly one piece, so only one thing
   writes it.

---

### User Story 3 - An arrow between two endpoints draws itself (Priority: P3)

Calling code says "an arrow from here, leaving downward, to there, leaving leftward" and gets an
arrow: a head at each endpoint pointing outward — opposite to the direction that endpoint leaves in
— and a route between them that the caller never described. This is the first figure whose pieces
cannot be read off its description at a glance: the same two positions with different directions are
two different drawings, and the route has to be derived before it can be placed.

**Why this priority**: it is the part of the wish with the most in it, and the part that most needs
the two stories before it to have settled how composition works. It is last because a derived route
is worth attempting only once placing known geometry is proven, and because it is the story with the
most cases to get right: every row of the direction families table below is one of its outcomes.

**Independent Test**: draw each of the seven pictured arrows into an empty buffer and compare the
rendered text; draw the first two, and separately the three that share the endpoint positions
`(2, 0)` and `(8, 2)`, and confirm the texts within each group differ; then, for every direction
family in the table under _Edge Cases_, either compare against a picture or confirm the buffer is
untouched.

**Acceptance Scenarios**:

1. **Given** an empty buffer, **When** an arrow from `(0, 0)` leaving `Down` to `(4, 3)` leaving
   `Left` is drawn, **Then** the text is exactly:

   ```text
   ▲
   │
   │
   └───►
   ```

2. **Given** an empty buffer, **When** an arrow from `(0, 0)` leaving `Right` to `(4, 3)` leaving
   `Up` is drawn, **Then** the text is exactly:

   ```text
   ◄───┐
       │
       │
       ▼
   ```

3. **Given** the two arrows above, **When** their rendered texts are compared, **Then** they differ.
   The endpoint positions are identical, so direction is part of the description and not a rendering
   preference.
4. **Given** an empty buffer, **When** an arrow from `(0, 0)` leaving `Right` to `(6, 0)` leaving
   `Left` is drawn, **Then** the text is exactly `◄─────►`: opposite directions with the endpoints
   aligned on the axis is a straight run with no bend.
5. **Given** an empty buffer, **When** an arrow from `(0, 0)` leaving `Right` to `(6, 2)` leaving
   `Left` is drawn, **Then** the text is exactly:

   ```text
   ◄──┐
      │
      └──►
   ```

6. **Given** an empty buffer 9 wide and 3 high, **When** an arrow from `(2, 0)` leaving `Right` to
   `(8, 2)` leaving `Left` is drawn, **Then** the text is exactly:

   ```text
     ◄──┐
        │
        └──►
   ```

   Both directions head toward the other end, so the starting positions `(3, 0)` and `(7, 2)` are
   inside the endpoint rectangle and the route never leaves it.

7. **Given** an empty buffer 10 wide and 3 high, **When** an arrow from `(2, 0)` leaving `Left` to
   `(8, 2)` leaving `Right` is drawn, **Then** the text is exactly:

   ```text
    ┌►
    └───────┐
           ◄┘
   ```

   Both directions head away, so the starting positions are `(1, 0)` and `(9, 2)`, one cell outside
   the endpoint rectangle on each side. The route occupies the columns `x = 1` and `x = 9` and the
   row `y = 1`, and nothing lies beyond a starting position.

8. **Given** an empty buffer 9 wide and 4 high, **When** an arrow from `(2, 0)` leaving `Left` to
   `(8, 2)` leaving `Down` is drawn, **Then** the text is exactly:

   ```text
    ┌►
    │
    │      ▲
    └──────┘
   ```

   The starting positions are `(1, 0)` and `(8, 3)`, one cell outside the endpoint rectangle to the
   left and below. The route occupies the column `x = 1` and the row `y = 3`, and nothing lies
   beyond a starting position.

9. **Given** an empty buffer 5 wide and 2 high, **When** an arrow from `(2, 0)` leaving `Right` to
   `(4, 1)` leaving `Left` is drawn, **Then** the text is exactly:

   ```text
     ◄┐
      └►
   ```

   The tightest arrangement the two-bend family has: the starting positions `(3, 0)` and `(3, 1)`
   share a column, so the two bends are adjacent and there is no run between them.

10. **Given** an empty buffer 5 wide and 3 high, **When** an arrow from `(2, 0)` leaving `Left` to
    `(3, 2)` leaving `Right` is drawn, **Then** the text is exactly:

    ```text
     ┌►
     └──┐
       ◄┘
    ```

    The tightest arrangement of the facing-away family: the route rectangle is one cell wider than
    the endpoint rectangle on each side, and the endpoint rectangle is two columns across.

11. **Given** the three arrows of scenarios 6, 7 and 8, **When** their rendered texts are compared,
    **Then** all three differ. They share the endpoint positions `(2, 0)` and `(8, 2)` and differ
    only in the two directions.
12. **Given** a buffer that counts writes per position, **When** any arrow above is drawn, **Then**
    no position has a count above 1 — including the positions where the route bends, which is where
    a route drawn as two overlapping runs would write twice.
13. **Given** an arrow whose two endpoints are at the same position, **When** it is drawn, **Then**
    the call returns normally: no error result and no panic. What it draws is the general rule's
    business and is not asserted.

---

### Edge Cases

Two tables. The first is the arrow's direction families, which the brief requires covered
exhaustively: each row ends in a picture or in a declared no-op, and a row that is neither is an
unfinished spec. The second is everything else.

A head points opposite to the direction its endpoint leaves in and occupies the endpoint position
itself, so the route runs between the two **starting positions**: one step from each endpoint in
that endpoint's own leaving direction. That step goes into the figure when the direction heads
toward the other end and out of it when the direction heads away. "The endpoint rectangle" below
means the smallest rectangle containing both endpoints, and "the route rectangle" the smallest
rectangle containing both starting positions; the second exceeds the first by exactly one cell on
each side a direction points away from. A route stays inside the route rectangle.

| Direction family                                 | Geometry                                                 | Expected                                                                                                                                                        |
| ------------------------------------------------ | -------------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Opposite, endpoints aligned on the axis          | Starting positions on one row or column                  | A straight run, no bend. Scenario 4 above                                                                                                                       |
| Opposite, endpoints not aligned                  | Both directions head toward the other end                | Two bends. Scenarios 5, 6 and 9 above                                                                                                                           |
| Perpendicular, both heading toward the other end | Both directions head toward the other end                | One bend, where the two axes meet. Scenarios 1 and 2 above                                                                                                      |
| Opposite, endpoints facing away from each other  | Both directions head away                                | Two starting positions outside the endpoint rectangle, three bends, the route rectangle one cell wider on each side. Scenarios 7 and 10 above                   |
| Perpendicular, at least one heading away         | One or both directions head away                         | The route rectangle grows one cell on each side pointed away from, and the route stays inside it. Scenario 8 above                                              |
| Identical at both endpoints                      | One direction heads toward the other end, one heads away | The same rule, with the route rectangle one cell longer on the side the far endpoint points away toward. Not pinned here; SC-003 requires the test that pins it |
| Both endpoints at the same position              | Zero distance                                            | Whatever the general rule yields. Not pinned here, illustrated below, and MUST NOT be special-cased. Scenario 13 above                                          |

The three families that need a route going out and around — opposite directions facing away from
each other, perpendicular directions with an endpoint facing away, and identical directions at both
endpoints — are one rule rather than three, and the _Clarifications_ session above records it: the
route rectangle, not the endpoint rectangle, is the bound. Drawing nothing was rejected as the
answer; a prior implementation that silently drew nothing for the facing-away case was a defect.

No row is left open. The last one is settled by not being pinned: an arrow whose endpoints coincide
is drawn by the same rule as every other arrow, and whatever that produces is the answer. The rule
is what this feature owes; an exception written for a degenerate arrangement is what it owes not to
write.

**Illustrative, not asserted.** The pictures below show a general arrangement collapsing step by
step into its degenerate one, twice. Only the two general arrangements are acceptance scenarios —
they are scenarios 9 and 10 above. The rest are what the rule is expected to produce, they are
excluded from SC-001, and an implementation that produces something else from the same general rule
has not broken this spec.

Toward each other, collapsing:

```text
(2, 0) Right → (4, 1) Left      (2, 0) Right → (3, 1) Left      (2, 0) Right → (3, 0) Left
  ◄┐                              ◄┐                              ◄►
   └►                              ►

(2, 0) Right → (2, 0) Left
  ►
```

Away from each other, collapsing:

```text
(2, 0) Left → (3, 2) Right      (2, 0) Left → (3, 1) Right      (2, 0) Left → (2, 1) Right
 ┌►                              ┌►                              ┌►
 └──┐                              ◄┘                             ◄┘
   ◄┘

(2, 0) Left → (2, 0) Right
  ◄
```

The rest:

| Case                                                       | Expected                                                                                                            | Settled by                                 |
| ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| A figure drawn partly or wholly outside the buffer         | The positions inside are drawn, the rest do nothing, and it is not an error                                         | _The buffer_ in `docs/model.md`            |
| A stroke from another figure reaching a box's border       | It joins the border, exactly as two boxes join today                                                                | Spec 0002, and _The cell_                  |
| A stroke reaching the interior side of a box's border      | It stops: a border closes the side facing its own interior                                                          | Spec 0002, and _The cell_                  |
| A stroke reaching an arrow's head, or a line's end         | It stops against it, because nothing connects into a chosen glyph                                                   | Feature 028, and _A cell can be a literal_ |
| An unfilled box's interior                                 | Inside the box's extent, and written by nothing: a piece may write no cells at all                                  | _Extent and pieces_                        |
| Two shapes overlapping in one buffer                       | Composed by the stamp, in the caller's order. A shape has no opinion about another shape                            | _Stamping_ in `docs/model.md`              |
| A shape drawn twice, or drawn after another shape          | The same as any two stamps at those positions. "No position written twice" is a promise per drawing, not per buffer | _Shapes_, and _Stamping_                   |
| A box, line or arrow whose fill or ends have no glyph rule | Cannot arise: what a shape draws is either derived through the catalog or a chosen glyph                            | _Rendering_, and feature 028               |
| A shape asked for its extent before being drawn            | Answers from its description alone, having drawn nothing                                                            | _Extent and pieces_                        |

## Requirements _(mandatory)_

### Functional Requirements

Composition:

- **FR-001** (P1): Calling code MUST be able to describe a figure and have it drawn without writing
  a cell and without computing a position. Every position and every character inside the figure MUST
  be the figure's own business.
- **FR-002** (P1): The way a shape is drawn MUST be uniform across shapes. A caller MUST NOT be able
  to tell from the drawing interface whether a shape writes cells itself or is made of other shapes.
- **FR-003** (P1): Shapes MUST compose to arbitrary depth, and no shape MUST depend on knowing how
  deep it sits. A shape placed by another shape MUST be drawable the same way at the top level.
- **FR-004** (P1): Only a shape that writes cells MUST write cells. A shape made of other shapes
  MUST NOT write any position itself; it places pieces, and they write.
- **FR-005** (P1): Drawing MUST happen in a single pass against the existing stamp, with no
  intermediate buffer of its own and no separate layout stage that runs before drawing.
- **FR-006** (P1): A shape MUST be a value: constructed where it is used, drawn, discarded. No
  mutable state and no lifecycle — drawing the same shape twice MUST produce the same writes.
- **FR-007** (P1): A shape made of other shapes MUST compute the concrete geometry of every piece it
  places. A piece MUST NOT have to report anything back for its placement to be decided, with the
  single exception FR-016 names.
- **FR-008** (P1): A shape's decomposition MAY depend on its parameters: the set of pieces MUST NOT
  be fixed per kind of shape. A 2×2 box has no border run and a 6×3 box has four.
- **FR-009** (P1): Defining a new kind of shape MUST require no change to any existing shape and no
  entry in any central enum, catalogue or factory.

Extent and partition:

- **FR-010** (P1): Every shape MUST be able to report its extent, as _Extent and pieces_ defines
  one, from its description alone and without drawing. A shape MUST NOT write outside it.
- **FR-011** (P1): A shape made of other shapes MUST partition its extent among its pieces, as
  _Extent and pieces_ requires, and MUST give each piece its portion as part of that piece's
  description.
- **FR-012** (P1): A **complete** shape, as _Complete and fragment_ defines one, MUST be
  expressible: this is what a library user sees, so if the abstraction cannot express it the feature
  has not landed. An unfilled box is the case that exercises the model's rule that a complete shape
  may leave positions of its extent unwritten.
- **FR-013** (P1): A **fragment**, as _Complete and fragment_ defines one, MUST be expressible
  alongside it.
- **FR-014** (P1): One shape MUST be able to be complete to its caller and a compositor to its own
  pieces at the same time, which the model allows and which the box is the first to need.
- **FR-015** (P1): What a fragment needs to know about its surroundings MUST reach it the way
  _Complete and fragment_ says: as part of its description. A fragment MUST NOT inspect the buffer
  and MUST NOT inspect its siblings.
- **FR-016** (P1): An arrow's route MUST be what _The route of an arrow_ describes, derived from the
  two endpoint positions and the two directions and from nothing else. Deriving it needs to know
  where it starts; whatever mechanism supplies that is the plan's to choose and MUST NOT be decided
  here.

Degenerate input and case coverage:

- **FR-017** (P1): Degenerate and impossible parameters MUST behave as _Degenerate arrangements_
  says: never failing, and drawing whatever the general rule yields rather than what an exception
  written for them would. Drawing nothing is a valid outcome where the rule yields nothing — a box
  below 2 in either dimension, FR-021.
- **FR-018** (P1): Every guard for a degenerate or impossible configuration MUST be reachable, and
  MUST have a test that exercises it. A guard no test reaches MUST be removed rather than kept.
- **FR-019** (P1): Case coverage MUST be exhaustive by rule rather than by enumeration. There MUST
  be no combination of otherwise valid parameters that panics or returns an error, and every
  combination MUST be an outcome of the shape's general rule. Where this spec pins no picture for a
  combination, the rule's output is the answer and is free to change when the rule does. The
  direction families table is where that is discharged for the arrow, and it MUST have no row
  without a test when this feature is implemented.

The figures, and the workspace:

- **FR-020** (P1): No shape MUST write any position more than once in one drawing. Verifiable rather
  than asserted: the tests draw against a buffer that counts writes per position.
- **FR-021** (P1): A box MUST be described by a position and a size, and MUST draw borders and
  corners, with a fill as an option. Below 2 in either dimension it MUST draw nothing.
- **FR-022** (P2): A straight line MUST be described by a position, a length, an orientation and the
  glyph at each of its two ends, and MUST occupy exactly the requested length, ending in something
  distinguishable from a segment at each end. No minimum length MUST be declared and no length MUST
  be rejected: a length below 2 is an outcome of the rule under FR-017, not a guard.
- **FR-023** (P3): An arrow MUST be described by two endpoints, each a position, the direction the
  arrow leaves it in, and the glyph of the head that sits there. A head MUST sit at each endpoint
  pointing outward, opposite to that endpoint's outgoing direction. The route MUST start one
  position from each endpoint in that endpoint's own leaving direction — outside the endpoint
  rectangle when that direction heads away from the other end — and MUST stay inside the rectangle
  the two starting positions span.
- **FR-024** (P1): `Buffer`, `stamp`, `Cell` and the renderer MUST be unchanged in what they do.
  This feature sits above them; a diagram drawn by stamping cells directly MUST render byte for byte
  as it does today.
- **FR-025** (P1): Everything this feature adds MUST live in `monospace-core` and MUST NOT assume a
  terminal, a command line or a user interface — _The core stays portable_, checked by the gate's
  `wasm` step.
- **FR-026** (P1): Every public item this feature adds MUST carry rustdoc as it is introduced, and
  the documentation of a shape MUST say what its extent is, because that is the sentence a caller
  would otherwise reconstruct from the code.
- **FR-027** (P2, P3): The character at a line's end and at an arrow's head MUST come from the
  caller, as _The glyph at an end and at a head_ requires, and MUST be a chosen glyph in the sense
  of feature 028. No glyph set gains a rule and `docs/glyph-sets.md` MUST be unchanged by this
  feature. The measurement behind that rule, taken while writing this spec: `╾` and `╼` are already
  claimed, in both mixing sets that hold them, by keys meaning heavy on one side and light on the
  other, so they could not also be keyed as the end of a light line, and `▲ ► ◄ ▼` appear in no set
  at all.
- **FR-028** (P2, P3): The pictures in this spec MUST be read as the output for one particular
  choice of those glyphs — `╾` and `╼` for a horizontal line's ends, `╿` and `╽` for a vertical
  one's, and `▲ ► ◄ ▼` for a head pointing up, right, left and down. A test asserting a picture MUST
  pass that choice in, and MUST NOT depend on a default.

### Key Entities

- **Shape**, **Extent**, **Piece**, **Endpoint**: all four are the model's, defined in _Shapes_ and
  listed in its _Vocabulary_. This feature introduces no entity of its own and redefines none of
  them.
- **Buffer**, **Cell**, **Stroke**, **Arm**, **Glyph**, **GlyphCatalog**: unchanged, all of them.
  This feature builds the layer above the buffer and changes nothing in it.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: every picture in an acceptance scenario has a test asserting the rendered text equals
  it exactly, trailing spaces and final newline included, and every one of those pictures has been
  produced by running the code rather than derived on paper — _Claims are measured, not assumed_.
  The pictures marked illustrative under _Edge Cases_ are excluded: they show what the general rule
  is expected to produce and are asserted by nothing.
- **SC-002**: the arrow of scenario 1 and the arrow of scenario 2 are asserted to differ, in one
  test that builds both from the same two positions; and the arrows of scenarios 6, 7 and 8 are
  asserted to differ from one another, in one test that builds all three from `(2, 0)` and `(8, 2)`.
- **SC-003**: the direction families table has no row without a test. A row this spec pins with a
  picture is a test asserting that exact text; the identical-directions row is a test whose expected
  picture is produced at implementation rather than pinned here; the same-position row is a test
  asserting only that the call returns normally. Countable — seven rows, seven tests, no row without
  one.
- **SC-004**: every degenerate-input guard has a test that reaches it. Verified by removing each
  guard in turn and confirming a test fails, then restoring: a guard nothing reaches is deleted
  rather than documented.
- **SC-005**: no test draws a figure by stamping cells, and no test computes a position inside a
  figure to assert against. What a test knows is a description and a picture.
- **SC-006**: the write counter shows a maximum of 1 per position for every figure in this spec. It
  is verified by being made to fail on purpose — a route drawn as two runs that share their bend —
  and then restored.
- **SC-007**: adding one more kind of shape is demonstrated to touch no existing shape. Countable:
  the diff of the commit that adds it contains no change to any file defining another shape, and no
  change to any list of shapes, because there is none.
- **SC-008**: rendering a diagram made by stamping cells directly is byte for byte what it is today.
- **SC-010**: `docs/glyph-sets.md` is unchanged by this feature. Countable: the diff of every commit
  in it touches that file zero times.
- **SC-009**: `cargo xtask check` passes at every commit, and on a fresh clone rather than only in
  the working copy.

## Assumptions

- **The wish's "cell window" is the model's buffer.** The brief asks for every scenario to be
  observable as the text content of the cell window; that is `render` over a `Buffer`, per
  _Rendering_ in [`docs/model.md`](../../docs/model.md), and no new observation mechanism is
  assumed.
- **"Occupy the same set of cells" is about extent, not about writes.** The brief requires a filled
  and an unfilled box to occupy the same cells. An unfilled box writes nothing in its interior —
  spec 0002 settled that an interior "is not a figure covering something, it is the absence of a
  figure" — so the two boxes agree on extent and differ on what is written inside it. FR-012 records
  the consequence: a piece may write no cells. Were writes meant instead, an unfilled box would have
  to write its interior, and that contradicts spec 0002.
- **A box's arms are already settled and are not revisited.** `Set` along the run, `Closed` on the
  side facing its own interior, `Unset` outward, from spec 0002 and _The cell_. The box shape
  reproduces that; it does not choose it.
- **A two-bend route bends at the midpoint of the span when the span is odd, and where the rule puts
  it when the span is even.** Scenarios 5 and 6 pin the odd case — starting span `x = 1` to `x = 5`,
  bend at `x = 3`, and `x = 3` to `x = 7`, bend at `x = 5`. The even case is deliberately unpinned,
  per _Clarifications_: no rounding rule is chosen here, and an arrow and its reverse are allowed to
  differ by one column.
- **A single-arm cell is not an end.** Measured in the Light table of
  [`docs/glyph-sets.md`](../../docs/glyph-sets.md): a cell with only a right arm renders `─`, the
  same as a segment. That measurement is why _The glyph at an end and at a head_ exists in the
  model; this spec records where it came from rather than restating what it concluded.
- **No decision is taken here.** _Decisions recorded when taken_ owns that. The decisions already in
  force are inputs: ADR-0008 for composition, ADR-0009 for degradation, ADR-0010 for keeping
  position and size apart, ADR-0019 for what a glyph is, ADR-0026 for what a cell is.
- **The feature number is the issue number.** `039` comes from issue #39, per
  [ADR-0024](../../docs/decisions/0024-take-the-feature-number-from-its-issue.md), not from a
  position in a local sequence.

## Out of scope

Every item names where it is handled instead.

- **A user interface of any kind.** The consumer is calling code. Whether `monospace-cli` redraws
  its existing picture through these shapes is a question for the plan — _Handoff to the plan_ — and
  would change no output if the answer is yes.
- **Shape identity, and knowing which shape wrote a position.** Out now, and the abstraction must
  not preclude it: a later feature that wants it has to be able to add it without changing what a
  shape is.
- **Persistence or serialization of a set of shapes.** Same standard: out now, not designed against.
- **An endpoint anchored to another shape** instead of to a fixed position. Same standard. An arrow
  here takes two positions and two directions and nothing else.
- **Text, and anything that places many chosen glyphs.** Feature 028 gave a cell one chosen glyph;
  what puts a word anywhere is still nobody's job.
- **Ends and heads that follow the glyph set.** Deliberately cut here: FR-027 has the caller supply
  them, so a diagram drawn with the ASCII set gets whatever characters its caller passed rather than
  ASCII ones of its own. A later feature may give each set its own ends and heads, and nothing here
  precludes it — but nothing here provides it either, and that is a known limitation rather than an
  oversight.
- **An input format, and layout computed from a description.** Both are _Open questions_ in
  `docs/model.md`. A caller here supplies concrete positions and sizes.
- **Diagonals, rounded corners as a shape's own choice, and a second stroke style per figure.** Out
  of the model, and out here.
- **Editing something already drawn.** The buffer is still write-once; the model's open question
  about mutable state is untouched by this feature.

The near miss is that "a box that draws itself" looks like the diagram layer arriving. It is not:
nothing here reads a description from outside the process, and nothing here decides where a figure
goes. The caller still says where.

## Handoff to the plan

One thing settled since this spec was written, and three it surfaces and does not own.

- **The model amendment is done and is no longer the plan's to schedule.** It was the first commit
  of this work, as this spec asked: _Shapes_ names the vocabulary, arrows left _Deliberately
  unresolved_, and the open question about the initial set is answered for these three figures. The
  plan starts from a model that already owns the words it will use. What the plan inherits is the
  three items below.
- **The complete/fragment distinction is where the design decision lives.** Whether it is two types,
  one type with a parameter, or a convention, is the plan's — and it is the choice most likely to be
  expensive to undo, so the one most likely to need an ADR of its own.
- **How a fragment learns about its surroundings, and how a route learns where it starts.** FR-015
  and FR-016 state what must be true and deliberately not how. Whether those two are one mechanism
  or two is worth answering explicitly rather than by accident.
- **Whether `monospace-cli` redraws its box through the box shape.** Recommended, in a `refactor`
  commit that changes no output: it is the only call site the repository has, so it is the only
  place FR-001 can be shown rather than asserted, and _Structural and behavioral change never share
  a commit_ keeps it separable. Cheap, reversible, and the maintainer's to decline.
