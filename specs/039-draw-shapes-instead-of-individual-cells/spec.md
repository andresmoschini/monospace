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

## What the model owes this feature

This section is a prerequisite, not context. [`docs/model.md`](../../docs/model.md) owns the
domain's design, and a slice that needs a rule it does not have amends it first — _The model owns
the design_ in the constitution, and the model's own header.

Measured by reading it today:

- The model describes **two mechanisms and nothing else**: how cells accumulate in a buffer, and how
  a buffer becomes characters. Its opening paragraph names "placement, the shapes themselves" as
  belonging to "layers that do not exist yet and are not described here".
- _Open questions_ asks **"What is the initial set of shapes and connectors?"** and answers it with
  the rule that anything with a shape of its own — it names an arrowhead — "needs its rule keyed
  like the rest before it can be specified".
- _Deliberately unresolved_ puts **arrows** "out of the model, not merely out of the first slice".

So this feature cannot reach `/speckit-plan` on the strength of this spec alone. It needs the model
to gain a shape-layer section that owns the vocabulary this spec borrows — shape, piece, extent,
partition, complete, fragment — and to move arrows from _Deliberately unresolved_ into it. That
amendment is where the decisions the brief defers will land, and it is the maintainer's to approve;
_Handoff to the plan_ says what it has to answer.

What this spec does in the meantime is use the brief's words as provisional, and say so once, here,
rather than in every requirement that uses one.

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
position one past the end holds no cell at all; draw a vertical line and compare; draw the shortest
line the answer to the open question below allows, and the one below that, and confirm each either
matches its picture or leaves the buffer empty.

**Acceptance Scenarios**:

1. **Given** an empty buffer at least 6 wide, **When** a horizontal line at `(0, 0)` of length 5 is
   drawn, **Then** the first row is exactly `╾───╼`, the position `(5, 0)` holds no cell, and the
   position `(0, 0)` holds an end rather than a segment — the two are distinguishable in the text.
2. **Given** an empty buffer at least 4 high, **When** a vertical line at `(0, 0)` of length 4 is
   drawn, **Then** the four rows are an end, a segment, a segment and an end, in that order, and
   `(0, 4)` holds no cell.
3. **Given** an empty buffer, **When** a line of length 3 is drawn, **Then** it is two ends with
   exactly one segment between them.
4. **Given** an empty buffer, **When** a line shorter than the minimum valid length is drawn,
   **Then** nothing is drawn and the call returns normally. The minimum is [NEEDS CLARIFICATION:
   what is the minimum valid length of a straight line? At 2 it would be two ends with no interior;
   at 1 and 0 it is unclear whether the answer is a single cell, one end, or nothing].
5. **Given** a buffer that counts writes per position, **When** any line above is drawn, **Then** no
   position has a count above 1.

---

### User Story 3 - An arrow between two endpoints draws itself (Priority: P3)

Calling code says "an arrow from here, leaving downward, to there, leaving leftward" and gets an
arrow: a head at each endpoint pointing outward — opposite to the direction that endpoint leaves in
— and a route between them that the caller never described. This is the first figure whose pieces
cannot be read off its description at a glance: the same two positions with different directions are
two different drawings, and the route has to be derived before it can be placed.

**Why this priority**: it is the part of the wish with the most in it, and the part that most needs
the two stories before it to have settled how composition works. It is last because a derived route
is worth attempting only once placing known geometry is proven, and because it is the story whose
behavior is still partly open — the direction families below are not all answered, and the ones that
are not cannot be implemented from this spec as it stands.

**Independent Test**: draw each of the four pictured arrows into an empty buffer and compare the
rendered text; draw the first two, which share endpoint positions and differ only in direction, and
confirm the two texts differ; then, for every direction family in the table under _Edge Cases_,
either compare against a picture or confirm the buffer is untouched.

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

6. **Given** a buffer that counts writes per position, **When** any arrow above is drawn, **Then**
   no position has a count above 1 — including the positions where the route bends, which is where a
   route drawn as two overlapping runs would write twice.
7. **Given** any arrow whose description is degenerate or impossible, **When** it is drawn, **Then**
   nothing is drawn and the call returns normally: no error result and no panic.

---

### Edge Cases

Two tables. The first is the arrow's direction families, which the brief requires covered
exhaustively: each row ends in a picture or in a declared no-op, and a row that is neither is an
unfinished spec. The second is everything else.

A head points opposite to the direction its endpoint leaves in and occupies the endpoint position
itself, so the route runs between the two positions one step inward from each end. "Inward" below
means that step; "the endpoint rectangle" means the smallest rectangle containing both endpoints.

| Direction family                                | Geometry                                      | Expected                                                            |
| ----------------------------------------------- | --------------------------------------------- | ------------------------------------------------------------------- |
| Opposite, endpoints aligned on the axis         | Inward positions on one row or column         | A straight run, no bend. Scenario 4 above                           |
| Opposite, endpoints not aligned                 | Both inward steps head toward each other      | Two bends. Scenario 5 above                                         |
| Perpendicular                                   | Both inward steps head toward the other end   | One bend, where the two axes meet. Scenarios 1 and 2 above          |
| Perpendicular                                   | One inward step heads away from the other end | Unsettled: the route has to leave the endpoint rectangle. See below |
| Opposite, endpoints facing away from each other | Both inward steps head away                   | Unsettled: the route has to leave the endpoint rectangle. See below |
| Identical at both endpoints                     | Any                                           | Unsettled: one inward step heads away. See below                    |
| Both endpoints at the same position             | Zero distance                                 | Unsettled: two heads want the same position. See below              |

The four unsettled rows are one question rather than four: [NEEDS CLARIFICATION: may an arrow's
route leave the rectangle its endpoints span? Every family whose route is not settled above —
perpendicular directions with one endpoint facing away, opposite directions facing away from each
other, and identical directions at both endpoints — needs a route that goes out and around, and each
is otherwise a valid description. A prior implementation silently drew nothing for the facing-away
case; the maintainer suspects that was a defect rather than a decision, so drawing nothing must not
be adopted as the answer without one]. And separately: [NEEDS CLARIFICATION: what does an arrow do
when both endpoints are at the same position? Both heads claim that one position, there is no route,
and the two directions may still differ].

The rest:

| Case                                                       | Expected                                                                                                            | Settled by                                 |
| ---------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- | ------------------------------------------ |
| A figure drawn partly or wholly outside the buffer         | The positions inside are drawn, the rest do nothing, and it is not an error                                         | _The buffer_ in `docs/model.md`            |
| A stroke from another figure reaching a box's border       | It joins the border, exactly as two boxes join today                                                                | Spec 0002, and _The cell_                  |
| A stroke reaching the interior side of a box's border      | It stops: a border closes the side facing its own interior                                                          | Spec 0002, and _The cell_                  |
| A stroke reaching an arrow's head                          | It stops against it, because nothing connects into a chosen glyph                                                   | Feature 028, and _A cell can be a literal_ |
| An unfilled box's interior                                 | Inside the box's extent, and written by nothing: a piece may write no cells at all                                  | This spec, FR-012                          |
| Two shapes overlapping in one buffer                       | Composed by the stamp, in the caller's order. A shape has no opinion about another shape                            | _Stamping_ in `docs/model.md`              |
| A shape drawn twice, or drawn after another shape          | The same as any two stamps at those positions. "No position written twice" is a promise per drawing, not per buffer | This spec, FR-020                          |
| A box, line or arrow whose fill or ends have no glyph rule | Cannot arise: what a shape draws is either derived through the catalog or a chosen glyph                            | _Rendering_, and feature 028               |
| A shape asked for its extent before being drawn            | Answers from its description alone, having drawn nothing                                                            | This spec, FR-010                          |

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

- **FR-010** (P1): Every shape MUST declare an extent — the set of positions it may write —
  derivable from its description alone, without drawing. A shape MUST NOT write outside its extent.
- **FR-011** (P1): A shape made of other shapes MUST partition its extent among its pieces: every
  position of the extent belongs to exactly one piece, with no overlap and no gap. A piece MUST
  receive its portion as part of its description rather than choosing it.
- **FR-012** (P1): A **complete** shape MUST be expressible: its description defines the finished
  figure, its extent covers everything visible in it — ends, corners, heads, fill — it draws
  finished with no further intervention, and its extent can be read from outside. This is what a
  library user sees. A complete shape MAY leave positions of its extent unwritten: an unfilled box's
  interior is inside its extent, belongs to a piece, and is written by nothing.
- **FR-013** (P1): A **fragment** shape MUST be expressible: its description defines only the
  positions it writes, it adds no end and no decoration of its own accord, and it exists to be
  placed by a shape that has already decided the partition.
- **FR-014** (P1): One shape MUST be able to be complete to its caller and a compositor to its own
  pieces at the same time.
- **FR-015** (P1): Where a fragment needs something about its immediate surroundings to choose what
  to write in a position — whether the position next to it belongs to a sibling of the same figure,
  for instance — that MUST arrive as part of its description. A fragment MUST NOT inspect the buffer
  and MUST NOT inspect its siblings.
- **FR-016** (P1): An arrow's route MUST be derived from the two endpoint positions and the two
  directions, and from nothing else. Deriving it needs to know where it starts, and the head
  occupies the endpoint position; whatever mechanism supplies that is the plan's to choose and MUST
  NOT be decided here.

Degenerate input and case coverage:

- **FR-017** (P1): A shape whose parameters are degenerate or describe an impossible configuration
  MUST draw nothing and MUST NOT fail. Drawing nothing is a valid outcome: no error result, no
  panic, and the call returns normally.
- **FR-018** (P1): Every guard for a degenerate or impossible configuration MUST be reachable, and
  MUST have a test that exercises it. A guard no test reaches MUST be removed rather than kept.
- **FR-019** (P1): Case coverage MUST be exhaustive and explicit. There MUST be no combination of
  otherwise valid parameters that produces neither a drawing nor a declared no-op — the direction
  families table is where that is discharged for the arrow, and it MUST have no unsettled row left
  when this feature is implemented.

The figures, and the workspace:

- **FR-020** (P1): No shape MUST write any position more than once in one drawing. Verifiable rather
  than asserted: the tests draw against a buffer that counts writes per position.
- **FR-021** (P1): A box MUST be described by a position and a size, and MUST draw borders and
  corners, with a fill as an option. Below 2 in either dimension it MUST draw nothing.
- **FR-022** (P2): A straight line MUST be described by a position, a length and an orientation, and
  MUST occupy exactly the requested length, ending in something distinguishable from a segment at
  each end.
- **FR-023** (P3): An arrow MUST be described by two endpoints, each a position and the direction
  the arrow leaves it in. A head MUST sit at each endpoint pointing outward, opposite to that
  endpoint's outgoing direction, and the route MUST start one position inward from it.
- **FR-024** (P1): `Buffer`, `stamp`, `Cell` and the renderer MUST be unchanged in what they do.
  This feature sits above them; a diagram drawn by stamping cells directly MUST render byte for byte
  as it does today.
- **FR-025** (P1): Everything this feature adds MUST live in `monospace-core` and MUST NOT assume a
  terminal, a command line or a user interface — _The core stays portable_, checked by the gate's
  `wasm` step.
- **FR-026** (P1): Every public item this feature adds MUST carry rustdoc as it is introduced, and
  the documentation of a shape MUST say what its extent is, because that is the sentence a caller
  would otherwise reconstruct from the code.
- **FR-027** (P1): The pictures' characters are placeholders. [NEEDS CLARIFICATION: which glyphs
  draw a line's ends and an arrow's heads? Measured against `docs/glyph-sets.md`: `╾` and `╼` are
  already claimed, in both mixing sets that hold them, by keys meaning heavy on one side and light
  on the other, so they cannot also be keyed as the end of a light line; and `▲ ► ◄ ▼` appear in no
  set at all. So an end and a head are each either a chosen glyph in the sense of feature 028 or a
  new keyed rule, which _Open questions_ in `docs/model.md` requires before an arrowhead can be
  specified].

### Key Entities

- **Shape**: a value describing a figure, which can be drawn into a buffer and asked for its extent.
  Provisional vocabulary until the model amendment owns it — see _What the model owes this feature_.
- **Extent**: the set of positions a shape may write, derivable from its description alone.
- **Piece**: a shape placed by another shape, with its portion of the extent given to it.
- **Endpoint**: a position and the direction an arrow leaves it in. The pair of them, and nothing
  else, determines an arrow's route.
- **Buffer**, **Cell**, **Stroke**, **Arm**, **Glyph**, **GlyphCatalog**: unchanged, all of them.
  This feature adds a layer above the buffer and changes nothing in it.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: every picture in this spec has a test asserting the rendered text equals it exactly,
  trailing spaces and final newline included, and every one of those pictures has been produced by
  running the code rather than derived on paper — _Claims are measured, not assumed_.
- **SC-002**: the arrow of scenario 1 and the arrow of scenario 2 are asserted to differ, in one
  test that builds both from the same two positions.
- **SC-003**: the direction families table has no unsettled row: each row is either a test with an
  exact expected picture or a test asserting the buffer is untouched. Countable — seven rows, seven
  tests, no row without one.
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
- **Where a two-bend route bends is the midpoint of the span, rounded toward the first endpoint.**
  Scenario 5 pins the odd case — inward span `x = 1` to `x = 5`, bend at `x = 3` — and says nothing
  about an even span, where the midpoint falls between two columns. Rounding toward the first
  endpoint is a default chosen for having an answer, is the maintainer's to overrule, and changes
  only the pictures of the even-span tests.
- **A single-arm cell is not an end.** Measured in the Light table of
  [`docs/glyph-sets.md`](../../docs/glyph-sets.md): a cell with only a right arm renders `─`, the
  same as a segment. So a line's ends are not a by-product of its arms, which is why FR-027 has to
  ask what draws them.
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

Four things this spec surfaces and does not own.

- **The model amendment is the first commit of this work, not part of the plan's preamble.** It has
  to name the shape layer's vocabulary, move arrows out of _Deliberately unresolved_, and answer
  _Open questions_' "What is the initial set of shapes and connectors?" for these three figures. The
  questions this spec leaves open are answered there or in the ADRs it needs, and `/speckit-clarify`
  is where they are put to the maintainer.
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
