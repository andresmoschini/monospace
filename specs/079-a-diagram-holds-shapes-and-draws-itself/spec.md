# Feature Specification: A diagram holds shapes and draws itself

**Feature Branch**: `079-a-diagram-holds-shapes-and-draws-itself-spec`

**Created**: 2026-09-15

**Status**: Draft

**Input**: [Issue #79](https://github.com/andresmoschini/monospace/issues/79) — nothing above the
buffer keeps a figure after it has been drawn: the command-line application builds its own list of
shapes on every run and the buffer is filled once and discarded. This slice creates
`monospace-diagram` with a diagram that holds shapes in an order and draws all of them into a window
the caller gives, front to back, and puts the command-line application on top of it. The shipped
demonstration picks a stamp mode per shape today; a diagram always uses `Below`.

Part of [issue #62](https://github.com/andresmoschini/monospace/issues/62). The decisions are
[ADR-0038](../../docs/decisions/0038-hold-the-diagram-model-in-a-crate-above-the-core.md),
[ADR-0039](../../docs/decisions/0039-a-diagram-shape-is-its-own-entity.md) and
[ADR-0042](../../docs/decisions/0042-draw-a-diagram-front-to-back-into-a-given-window.md), and the
model is [`docs/diagram-model.md`](../../docs/diagram-model.md).

## Which slice of the model this is

This implements [`docs/diagram-model.md`](../../docs/diagram-model.md) under _The diagram_ and
_Drawing_, the `Diagram`, `Shape` and `Order` rows of _Vocabulary_, and the `add` row of _Changing a
diagram_. Everything else in that document is a later slice and is listed under **Out of scope**
below. This spec restates none of it.

## User Scenarios & Testing _(mandatory)_

### User Story 1 - A figure survives being drawn (Priority: P1)

Someone building a picture wants to hold it rather than paint it. They put shapes into a diagram,
one at a time, and the diagram keeps them in an order. Later — once, twice, or after the picture has
already been rendered somewhere — they ask the diagram to draw itself into a window they choose, and
get the same picture every time. The diagram is the source of truth; the buffer is a working surface
rebuilt from it.

**Why this priority**: it is the whole of the capability. Nothing else in issue #62 — an identity, a
reference, an ownership record — has anywhere to live until something holds shapes after they have
been drawn.

**Independent Test**: build a diagram, add two shapes to it, draw it into a window, render the
result, then draw the same diagram again into an equal window. Both drawings produce equal buffers,
and the diagram is unchanged by either.

**Acceptance Scenarios**:

1. **Given** a diagram holding a box and a line, **When** it is drawn into a window and then drawn
   again into an equal window, **Then** the two buffers are equal.
2. **Given** a diagram that has been drawn, **When** it is asked to draw a second time, **Then**
   nothing about the diagram has changed: the same shapes, in the same order.
3. **Given** a diagram holding no shapes, **When** it is drawn into a window, **Then** the buffer is
   left exactly as it was.
4. **Given** a diagram holding a box that falls partly outside the window it is drawn into, **When**
   it is drawn, **Then** what falls inside the window appears and nothing else does, with no error
   and no report.
5. **Given** a diagram holding a box, a line and an arrow, **When** it is drawn, **Then** each is
   drawn exactly as the corresponding core shape draws it with the same parameters.

---

### User Story 2 - The order decides who wins an overlap (Priority: P1)

Someone places two figures that overlap and wants to say which of them decides the cells they share.
They do it by where the shapes sit in the diagram's order: a shape nearer the front decides first.
They do not reach for a stamp mode, and the diagram does not offer one.

**Why this priority**: an order that does not change the picture is a list. This is what makes the
order mean something, and it is what the command-line application's demonstration has been
expressing per shape until now.

**Independent Test**: build two diagrams holding the same two overlapping boxes in opposite orders,
draw each, and compare. The two buffers differ, and each matches the same two core shapes stamped
back to front with `Above` in the corresponding order.

**Acceptance Scenarios**:

1. **Given** two boxes that overlap, **When** a diagram holding them is drawn front to back with
   `Below`, **Then** the buffer equals the one produced by stamping the same two core shapes back to
   front with `Above` — the equivalence _The two orders are equivalent_ states in
   [`docs/model.md`](../../docs/model.md).
2. **Given** the same two boxes, **When** they are put into two diagrams in opposite orders and both
   are drawn, **Then** the two buffers differ, and in each the front-most shape is the one whose
   stroke decides the shared cells.
3. **Given** a diagram holding a horizontal line and a vertical line that cross, **When** it is
   drawn, **Then** the crossing is a junction made by both lines rather than one interrupting the
   other: the order is composition, not occlusion.
4. **Given** a diagram holding a filled box in front of a line, **When** it is drawn, **Then** the
   box's fill hides the line where they overlap, because a literal glyph is the whole of the opacity
   this phase has.
5. **Given** a shape added to a diagram, **When** it is added, **Then** it sits at the front of the
   order, in front of everything already there.

---

### User Story 3 - The command-line application draws through a diagram (Priority: P1)

Someone runs the shipped demonstration and sees exactly the picture they saw before. What changed is
underneath: the application no longer stamps a list of shapes into a buffer, it builds a diagram and
asks the diagram to draw. Its description format loses the per-shape `mode` field, because a diagram
always stamps with `Below` and the order carries what `mode` was carrying.

**Why this priority**: issue #79 asks for the application to be put on top of the diagram, and
[ADR-0038](../../docs/decisions/0038-hold-the-diagram-model-in-a-crate-above-the-core.md) names it
as the visible evidence — "once it builds a `Diagram` and asks it to draw, the model is exercised by
something that runs rather than by tests alone".

**Independent Test**: run the application with no arguments before and after the change and compare
the two outputs. They are identical, byte for byte, although the demonstration file has changed and
the format it is written in no longer has a `mode` field.

**Acceptance Scenarios**:

1. **Given** the demonstration file with its two `mode: "below"` shapes replaced by the
   corresponding order, **When** the application is run with no arguments, **Then** it prints
   exactly what it printed before this feature.
2. **Given** a description file, **When** it is read, **Then** its `shapes` array is added to a
   diagram in the order it is written, so the last shape in the file is the front-most and decides
   first — which is what the last shape in the file did before, when it was painted last.
3. **Given** a description file carrying a `mode` field on any shape, **When** it is read, **Then**
   it is rejected with an error naming the unrecognized field, rather than parsed with the field
   ignored.
4. **Given** a description with two overlapping boxes, **When** it is rendered, **Then** it produces
   the same text as the two corresponding core shapes stamped back to front with `Above`.
5. **Given** the application, **When** it is built, **Then** it still holds no domain logic: its
   description types convert into diagram shapes and it renders, and nothing else.

---

### Edge Cases

- A diagram holding no shapes, drawn into a window: the buffer is untouched, and rendering it gives
  the same text as rendering an untouched buffer of that window.
- A shape entirely outside the window: nothing appears, exactly as a stamp outside a buffer's window
  has always behaved. Nothing measures the drawing to check the window was big enough, so a figure
  that falls outside is silently clipped — that consequence is recorded in ADR-0042 and accepted
  here.
- A window of zero width or zero height: whatever the buffer already does with one, unchanged. This
  feature adds no rule about it.
- A shape added twice, as two equal values: the diagram holds two shapes. Nothing in this slice
  deduplicates, because nothing in this slice can tell two shapes apart.
- The same diagram drawn into two windows with different origins: each drawing is independent, and
  neither is affected by the other.

## Requirements _(mandatory)_

### Functional Requirements

#### The crate

- **FR-001**: The workspace MUST gain a crate named `monospace-diagram`, depending on
  `monospace-core` as an ordinary dependent with no privilege the core does not give every dependent
  (ADR-0038).
- **FR-002**: `monospace-core` MUST NOT gain a dependency on `monospace-diagram`, and MUST NOT gain
  a shape that carries an identity, holds a place in an order, or answers where it is — the three
  things ADR-0039 names as what would abandon its decision.
- **FR-003**: `cargo xtask check`'s `wasm` step MUST check `monospace-diagram` against
  `wasm32-unknown-unknown`, alongside the two crates it checks today.
- **FR-004**: Every public item the crate introduces MUST carry rustdoc as it is introduced, per
  principle VII.

#### The diagram

- **FR-005**: A diagram MUST hold shapes in an order and nothing else: no buffer, no glyph catalog,
  no window and no rendered picture.
- **FR-006**: A caller MUST be able to create an empty diagram and to add a shape to it. Adding puts
  the shape at the front of the order.
- **FR-007**: A diagram's shape MUST be a value of the diagram crate's own type, holding the
  parameters of its figure and an absolute position, and drawing by constructing the corresponding
  core shape (ADR-0039).
- **FR-008**: The set of kinds a diagram's shape may be MUST be closed, and MUST hold exactly three
  in this slice: a box, a line and an arrow — the three the core draws.
- **FR-009**: Each kind MUST draw what the corresponding core shape draws, given the same
  parameters. A parameter the core shape takes and the diagram's kind does not carry is a parameter
  the diagram cannot express, and there MUST be none in this slice.

#### Drawing

- **FR-010**: A diagram MUST draw all of its shapes into a window the caller gives it. The diagram
  MUST NOT compute, measure or size that window (ADR-0042).
- **FR-011**: Drawing MUST visit shapes from the front of the order to the back.
- **FR-012**: Every cell MUST be stamped with `Below`. The caller MUST NOT be able to choose a stamp
  mode, per shape or per drawing.
- **FR-013**: Drawing MUST produce cells and stop there. Turning those cells into text stays the
  caller's, with the glyph catalog the caller holds.
- **FR-014**: What falls outside the window MUST be clipped, silently, exactly as a stamp outside a
  buffer's window is today. Drawing MUST NOT fail and MUST NOT report.
- **FR-015**: Drawing MUST change nothing about the diagram, and MUST be repeatable: the same
  diagram drawn twice into equal windows produces equal buffers.

#### The command-line application

- **FR-016**: The application MUST build a diagram from the description it reads and ask that
  diagram to draw, rather than stamping shapes into a buffer itself.
- **FR-017**: The description format MUST lose its per-shape `mode` field.
- **FR-018**: The description format MUST reject a field it does not recognize, naming it, so that a
  description written against the old format fails rather than drawing something different from what
  it says.
- **FR-019**: The `shapes` array MUST be added to the diagram in the order it is written, so the
  last entry is the front-most.
- **FR-020**: The `canvas` a description carries MUST be the window the diagram is asked to draw
  into.
- **FR-021**: The shipped demonstration MUST be changed so that its rendered output is unchanged:
  the two pairs it expresses with `mode: "below"` today become the corresponding order.
- **FR-022**: The description types MUST stay private to `monospace-cli`, per
  [ADR-0035](../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md), and the
  application MUST keep holding no domain logic.

### Testing expectations

This spec's minimum, beyond the unit tests the constitution asks of any slice:

- **TE-001**: The equivalence of the two orders is tested: a diagram drawn front to back with
  `Below` produces the same buffer as the same core shapes stamped back to front with `Above`. This
  is the confirmation ADR-0042 names for its direction, and it is the property that lets the
  demonstration's output stay unchanged.
- **TE-002**: Clipping is tested with a shape placed partly outside the given window: what falls
  inside is drawn, and nothing else appears.
- **TE-003**: Repeatability is tested: the same diagram drawn twice into equal windows produces
  equal buffers.
- **TE-004**: The order mattering is tested: the same two overlapping shapes in opposite orders
  produce different buffers.
- **TE-005**: Each of the three kinds is tested against the core shape it constructs, so a parameter
  dropped in the conversion fails a test rather than a reading.
- **TE-006**: The command-line application is tested end to end on a description with two
  overlapping boxes, against the text the corresponding core shapes produce.
- **TE-007**: The demonstration's output being unchanged is confirmed by observation at delivery —
  captured before the change, compared after — and recorded in `docs/learning-log.md`. It is not
  pinned by an automated test, for the same reason spec 060 gave: pinning the demonstration's text
  makes every later change to the demonstration a change to a test. **This is a requirement accepted
  with nothing automatic to verify it**, named as such here per principle IV.

### Key Entities

- **Diagram**: shapes in an order, drawable. The source of truth. Holds nothing else.
- **Shape** (the diagram's): one figure in a diagram — what kind of figure it is, its parameters,
  and its position, which is absolute in this slice. Distinct from the core's `Shape`, which is a
  value that draws and answers nothing about itself.
- **Order**: the sequence a diagram holds its shapes in. Its front is drawn first, and a shape
  nearer the front decides a cell before one behind it. Nothing about it is a coordinate.
- **Window**: an origin and a size, given by the caller at the moment of drawing. Not held by the
  diagram, not derived from its shapes.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: A caller can build a picture of three figures, keep it, and draw it twice into two
  windows, without rebuilding the list of figures for the second drawing.
- **SC-002**: Which of two overlapping figures decides their shared cells is expressible by order
  alone: the caller has no stamp mode to reach for, and the picture still changes when the order
  does.
- **SC-003**: Running the command-line application with no arguments produces output identical, byte
  for byte, to what it produced before this feature.
- **SC-004**: A description file written against the previous format is rejected with a message
  naming the field that no longer exists, rather than silently rendering a different picture.
- **SC-005**: `cargo xtask check` is green, including the new crate compiling for
  `wasm32-unknown-unknown`.
- **SC-006**: Every claim above about which figure wins an overlap is backed by a test that fails if
  the drawing order or the stamp mode is changed.

### Accepted on observation

**SC-003** is confirmed by running the application before and after and comparing, once, at
delivery. Nothing automatic pins it, by TE-007.

## Assumptions

- **A diagram draws cells, and the caller renders.** `docs/diagram-model.md` reads two ways on this:
  _The diagram_ says a buffer and a glyph catalog are "given to it when it draws", while _Drawing_
  says the diagram draws into a window "together with whatever else the render needs". This spec
  takes the first: drawing produces cells, and rendering them to text stays the caller's. It is what
  issue #86 needs, since the ownership record lives in the buffer beside its cells while the diagram
  holds the mapping out of it. _Drawing_'s wording is looser than the rule it states, and tightening
  it is a change to the model rather than to this feature.
- **The order is what the demonstration's `mode` was expressing.** Verified before writing this
  spec: swapping the two pairs that use `mode: "below"` and setting every remaining `mode` to
  `"above"` produces output identical to the current demonstration's. That is the back-to-front half
  of the equivalence; the diagram draws the front-to-back half, which the model proves equal.
- **No identity is generated in this slice.** _Changing a diagram_ says adding gives a shape an
  identity; that half of the sentence is issue #80's. Here, adding places a shape in the order and
  nothing more, and nothing can name a shape afterwards.
- **Every position is absolute.** _Positions_ allows a reference, but only on a connector's
  endpoint, and that is issue #85's. Nothing in this slice resolves anything, so the "resolves to
  nothing" rule has no case to apply to.
- **Rejecting an unrecognized field is new behavior, not a restoration.** The format ignores unknown
  fields today. FR-018 changes that so removing `mode` cannot silently change what a file means. It
  replaces the existing test that a bad `mode` value fails to parse.
- **The constitution needs no amendment.** _In scope for this phase_ already names
  `monospace-diagram`, amended when ADR-0038 was taken.

## Out of scope

Each of these is named because the model describes it and this slice does not implement it. A plan
that reaches for one of them is widening the slice.

| Not here                                                   | Where it is                     |
| ---------------------------------------------------------- | ------------------------------- |
| A shape's identity, and moving one forward or backward     | Issue #80                       |
| Removing a shape, and replacing one                        | Issue #81                       |
| Anchor points on a box, and on a line                      | Issues #82 and #84              |
| A reference, an offset, and an endpoint that hangs off one | Issues #83 and #85              |
| A position resolving back to the shape that decided it     | Issue #86                       |
| A diagram saying which shapes it could not draw            | Issue #88                       |
| A reference on any shape's position, and cycles            | Issue #89                       |
| Corner and center anchor points                            | Issue #90                       |
| Groups of shapes                                           | Issue #58, and out of the model |
| A diagram measuring itself, or choosing its own window     | ADR-0042; open in the model     |
| Reading a diagram from a file, or writing one to a file    | ADR-0035; open in the model     |

Two further things stay where they are: `Surface` and `Layer` keep the shape
[ADR-0031](../../docs/decisions/0031-a-shape-draws-into-a-surface.md) gave them, and the composition
rule of [ADR-0008](../../docs/decisions/0008-compose-overlapping-cells-with-three-state-arms.md) is
untouched.
