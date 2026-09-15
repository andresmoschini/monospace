# Feature Specification: A shape in a diagram has an identity, and the order can change

**Feature Branch**: `080-a-shape-in-a-diagram-has-an-identity-and-spec`

**Created**: 2026-09-15

**Status**: Draft

**Input**: [Issue #80](https://github.com/andresmoschini/monospace/issues/80) — a diagram that draws
a list still has no way to speak about one of the shapes in it. This slice gives every shape an
identity the diagram generates when it is added, and lets a shape named by its identity move one
place toward the front or the back of the order.

Part of [issue #62](https://github.com/andresmoschini/monospace/issues/62), and it builds on
[issue #79](https://github.com/andresmoschini/monospace/issues/79), the diagram that draws itself.
The model is [`docs/diagram-model.md`](../../docs/diagram-model.md), under _Identity_ and _Changing
a diagram_.

## Which slice of the model this is

This implements [`docs/diagram-model.md`](../../docs/diagram-model.md) under _Identity_, the
`ShapeId` and `Order` rows of _Vocabulary_, and the `forward` and `backward` rows of _Changing a
diagram_ together with that section's rule that none of the changes can fail. Everything else in
that document is a later slice and is listed under **Out of scope** below. This spec restates none
of it.

## User Scenarios & Testing _(mandatory)_

### User Story 1 - A shape can be named after it has been placed (Priority: P1)

Someone builds a diagram one shape at a time and wants to speak about one of them afterwards. Each
time they add a shape, the diagram generates an identity for it and hands that identity back. The
identity is theirs to keep: it names that shape for as long as the diagram holds it, and it is the
only thing a later change can be aimed at.

**Why this priority**: it is the whole of the capability. Nothing else issue #62 asks for — removing
a shape, replacing one, hanging an arrow off one, asking what is at a position — has anything to
name until a shape has an identity.

**Independent Test**: add three shapes to a diagram, keep what each addition hands back, and check
the three are different from one another. No further behavior is needed to show that a shape is now
addressable.

**Acceptance Scenarios**:

1. **Given** an empty diagram, **When** a shape is added, **Then** the addition hands back an
   identity for that shape.
2. **Given** a diagram, **When** three shapes are added to it one after another, **Then** the three
   identities are different from one another.
3. **Given** a diagram, **When** the same shape value is added twice, **Then** the diagram holds two
   shapes with two different identities, because identity is the diagram's and not the shape's.
4. **Given** a diagram, **When** a shape is added, **Then** it still goes to the front of the order,
   exactly as it did before this feature.

---

### User Story 2 - A shape moves one place toward the front or the back (Priority: P1)

Someone looks at a picture where the wrong figure is winning an overlap. They name the figure by its
identity and move it one place toward the front, or one place toward the back, and draw again. The
picture changes because the order changed; nothing else about the diagram did.

**Why this priority**: an identity nothing can be done with is a token. This is what makes the
identity worth generating, and it is the first change to a diagram that is not an addition.

**Independent Test**: build a diagram from two partially overlapping opaque boxes, draw it, move the
back one forward by its identity, draw again, and compare the two buffers. They differ, and the
second equals what the same two boxes added in the opposite order produce.

**Acceptance Scenarios**:

1. **Given** a diagram holding two partially overlapping opaque boxes, **When** the back one is
   moved forward by its identity and the diagram is drawn, **Then** the buffer equals what the same
   two boxes added in the opposite order produce.
2. **Given** the same diagram, **When** the front one is moved backward by its identity and the
   diagram is drawn, **Then** the buffer is the same one moving the other forward produces: the two
   changes are the same swap named from either side.
3. **Given** a shape that is already at the front of the order, **When** it is moved forward,
   **Then** nothing changes, and drawing produces the buffer it produced before.
4. **Given** a shape that is already at the back of the order, **When** it is moved backward,
   **Then** nothing changes, and drawing produces the buffer it produced before.
5. **Given** a diagram, **When** a change names an identity the diagram does not hold, **Then**
   nothing changes, with no error and no report — the same answer _Positions_ gives a reference to a
   shape that is not there.
6. **Given** a shape that has been moved, **When** it is moved back the other way, **Then** the
   diagram draws exactly what it drew before either move: the identity named the same shape both
   times, so it survived the reorder.

---

### User Story 3 - The shipped demonstration shows a reorder (Priority: P2)

Someone runs the command-line application and sees the same figures twice: once as the description
lists them, and once with one figure moved one place toward the front. The pair that changes is two
opaque boxes that partly overlap, so which of them is in front is plain to see in both pictures.
Nothing else about the two pictures differs.

**Why this priority**: it is what makes this increment demonstrable to someone who runs the
application rather than only to someone who reads a test, which is what principle II asks of a
slice. It is second because the capability is complete without it.

**Independent Test**: run the application with no arguments. Two pictures appear; they are identical
except in the top-left pair of overlapping opaque boxes, where the box in front is the other one.

**Acceptance Scenarios**:

1. **Given** the shipped demonstration, **When** the application is run with no arguments, **Then**
   it prints the description as written and then prints it again with one shape moved one place
   toward the front.
2. **Given** any description the application is given, **When** it is rendered, **Then** the first
   picture is exactly what the application printed for that description before this feature.
3. **Given** a description whose shapes do not overlap, **When** it is rendered, **Then** the two
   pictures are identical, because a reorder only shows where figures share cells.
4. **Given** a description holding fewer than two shapes, **When** it is rendered, **Then** the two
   pictures are identical and nothing fails.

---

### Edge Cases

- A diagram holding one shape: moving it forward and moving it backward both change nothing, because
  it is at both ends of the order at once.
- A diagram holding no shapes: any change names an identity it does not hold, so nothing changes.
- An identity kept from one diagram and used on another: the second diagram does not hold that
  shape, so nothing changes. Identities are unique within a diagram and say nothing across two.
- Two shapes that overlap but decide no cell in common — two unfilled boxes with the same stroke,
  whose shared cells every side leaves `Unset` — reorder without the picture changing. The order
  changed; what is drawn did not.
- A shape moved forward as many times as the diagram holds shapes: it arrives at the front and every
  further move changes nothing.
- The demonstration file's first two entries are the pair the demonstration moves. A file whose
  first two entries do not overlap still renders twice, and the two pictures are identical.

## Requirements _(mandatory)_

### Functional Requirements

#### Identity

- **FR-001**: The diagram MUST generate an identity for a shape when that shape is added, and that
  identity MUST be unique within the diagram.
- **FR-002**: Adding a shape MUST hand its identity back to the caller. This is the only way a
  caller learns an identity in this slice.
- **FR-003**: A caller MUST NOT be able to choose an identity, or to change one after the fact. Both
  are open questions in _Identity_ and neither is answered here.
- **FR-004**: An identity MUST survive every change this slice makes to the shape it names: after a
  move forward or backward, the same identity names the same shape.
- **FR-005**: Adding MUST keep putting the shape at the front of the order, unchanged from spec 079.
- **FR-006**: Every public item this feature introduces MUST carry rustdoc as it is introduced, per
  principle VII.

#### Changing the order

- **FR-007**: A caller MUST be able to move a shape, named by its identity, one place toward the
  front of the order.
- **FR-008**: A caller MUST be able to move a shape, named by its identity, one place toward the
  back of the order.
- **FR-009**: Neither change may fail. Naming an identity the diagram does not hold MUST change
  nothing, and MUST NOT produce an error, a report or a panic.
- **FR-010**: Moving a shape that is already at the end it is moving toward MUST change nothing.
- **FR-011**: A move MUST change the order and nothing else: the shapes a diagram holds, their
  parameters and their identities are all untouched.
- **FR-012**: The diagram MUST NOT gain a way to read its shapes, its identities or its order back.
  Drawing stays the only way the order is observed, so a move is observed by drawing again.

#### The command-line application

- **FR-013**: The application MUST render the description it reads, then move one shape one place
  toward the front by its identity, then render the diagram again, printing both pictures.
- **FR-014**: The first picture MUST be exactly what the application printed for that description
  before this feature.
- **FR-015**: The shape the demonstration moves MUST be the first entry of the description's
  `shapes` array — the back-most shape — named by the identity the diagram handed back when that
  entry was added.
- **FR-016**: Where the application makes that choice, the code MUST say in a comment that it is a
  demonstration-only assumption: the application relies on the shipped demonstration's first two
  entries being two partially overlapping opaque boxes, so that the move is visible.
- **FR-017**: The shipped demonstration file MUST NOT need to change. Its first two entries are
  already two partially overlapping opaque boxes.
- **FR-018**: The two pictures MUST be separated in the output so that each is readable as a whole
  picture.
- **FR-019**: The description format MUST NOT gain a field, and the description types MUST stay
  private to `monospace-cli`, per
  [ADR-0035](../../docs/decisions/0035-keep-the-cli-demo-format-out-of-the-model.md). The
  application MUST keep holding no domain logic: it names a shape and asks the diagram to move it.

### Testing expectations

This spec's minimum, beyond the unit tests the constitution asks of any slice:

- **TE-001**: Identities being different is tested: three shapes added to one diagram yield three
  identities that differ from one another.
- **TE-002**: Moving forward is tested by drawing: two partially overlapping opaque boxes, drawn
  before and after the back one moves forward, produce different buffers, and the second equals what
  the same two boxes added in the opposite order produce.
- **TE-003**: Moving backward is tested the same way, and against the same expected buffer, so that
  a move implemented in one direction only fails a test.
- **TE-004**: The ends are tested: moving the front-most forward and the back-most backward each
  leave the drawn buffer unchanged.
- **TE-005**: An identity the diagram does not hold is tested against both changes: the drawn buffer
  is unchanged and nothing fails.
- **TE-006**: An identity surviving a reorder is tested: a shape moved forward and then backward by
  the same identity leaves the diagram drawing what it drew at the start.
- **TE-007**: The application is tested end to end on a description of two partially overlapping
  opaque boxes: it prints two pictures, the first equal to the two boxes in the order written and
  the second equal to the two boxes in the opposite order.
- **TE-008**: The demonstration's two pictures differing only in the top-left pair is confirmed by
  observation at delivery — run, read, and recorded in `docs/learning-log.md`. It is not pinned by
  an automated test, for the reason spec 079 gave under its TE-007: pinning the demonstration's text
  makes every later change to the demonstration a change to a test. **This is a requirement accepted
  with nothing automatic to verify it**, named as such here per principle IV.

### Key Entities

- **ShapeId**: a shape's identity — a string, generated by the diagram when the shape is added, and
  unique within that diagram. It is the diagram's, not the shape's: it survives what happens to the
  shape it names, and two equal shapes added twice get two of them.
- **Order**: the sequence a diagram holds its shapes in, unchanged from spec 079. Its front is drawn
  first. What this slice adds is that a shape can move one place along it in either direction.
- **Diagram**: shapes in an order, drawable, and now changeable by naming one of them.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: A caller who has added a shape can name it afterwards, without having kept the shape
  value itself and without the diagram offering any other way to find it.
- **SC-002**: A caller can change which of two overlapping figures wins their shared cells, on a
  diagram that has already been built, without rebuilding it and without removing anything from it.
- **SC-003**: Every way of naming a shape that is not there — an identity from another diagram, one
  for a diagram holding nothing — leaves the diagram drawing exactly what it drew before, and no run
  fails.
- **SC-004**: Running the command-line application with no arguments shows the same figures twice,
  differing only where the moved figure overlaps its neighbor, so someone who runs it can see what a
  reorder does without reading a test.
- **SC-005**: Every claim above about what a move does to a picture is backed by a test that fails
  if the move is made in the wrong direction, or by one place too many.
- **SC-006**: `cargo xtask check` is green, including `monospace-diagram` compiling for
  `wasm32-unknown-unknown`.

### Accepted on observation

**SC-004** is confirmed by running the application once at delivery and reading the two pictures, by
TE-008. Nothing automatic pins it.

## Assumptions

- **Adding hands the identity back, and the model needs no amendment for it.** _Identity_ says the
  diagram generates an identity when a shape is added, and that an identity "is what makes a shape
  findable after it has been placed". It does not say how the caller learns one, and with no way to
  read the order back there is only one place it can come from: the addition itself. That implements
  the model rather than extending it, so nothing in `docs/diagram-model.md` changes in this feature.
- **Nothing reads the order back, and that is a scope choice rather than a rule.** The diagram
  offers no listing of its shapes or its identities, and no way to ask where one sits, so every test
  here observes a move by drawing — as spec 079's order tests already do. The model forbids no such
  reading, and issue #86 brings one of a different kind; this slice simply adds none.
- **No ADR is taken.** The two decisions this slice could be read as taking — that the addition
  hands the identity back, and that nothing else reads the order — are both cheap to undo: a reading
  operation is added later without changing anything that exists. Neither is expensive to reverse
  and neither is a question someone would later ask "why is this like this?" about, so by principle
  VI both belong in the learning log rather than in `docs/decisions/`.
- **The identity is a string, and its shape is the plan's business.** _Vocabulary_ settles what it
  is — "a string, unique within its diagram" — and _Identity_ settles what the diagram generates:
  the first is `#1`, the second `#2`, and so on. How that is typed so a caller cannot fabricate one
  is a design question for `/speckit-plan`, not a requirement here.
- **Identities are never reused.** Nothing in this slice removes a shape, so the question only
  arises with issue #81. A diagram that has added three shapes has handed out three identities, all
  different.
- **The demonstration prints two pictures for every description, not only for the shipped one.** The
  application special-cases nothing: a file it is given renders twice, and the second picture is
  identical to the first whenever the move changes no cell. How the two are separated in the output
  — a blank line, a caption, or both — is left to the plan and is worth settling at
  `/speckit-clarify` if the maintainer has a preference.
- **The application's output changes, deliberately.** Spec 079's SC-003 pinned it byte for byte
  against the version before that feature; this feature ends that, and FR-014 keeps only the first
  picture equal to what was printed before.
- **The constitution needs no amendment.** _In scope for this phase_ already names
  `monospace-diagram`, and this slice adds no crate.

## Out of scope

Each of these is named because the model describes it and this slice does not implement it. A plan
that reaches for one of them is widening the slice.

| Not here                                                      | Where it is                     |
| ------------------------------------------------------------- | ------------------------------- |
| Removing a shape, and replacing one                           | Issue #81                       |
| A caller choosing an identity, or editing one                 | Open question in the model      |
| Reading a diagram's shapes, identities or order back          | Not asked for by any issue yet  |
| Anchor points on a box, and on a line                         | Issues #82 and #84              |
| A reference, an offset, and an endpoint that hangs off one    | Issues #83 and #85              |
| A position resolving back to the shape that decided it        | Issue #86                       |
| A diagram saying which shapes it could not draw               | Issue #88                       |
| A reference on any shape's position, and cycles               | Issue #89                       |
| Corner and center anchor points                               | Issue #90                       |
| Moving a shape to the very front or the very back in one step | Not in the model                |
| Groups of shapes                                              | Issue #58, and out of the model |

Two further things stay where they are: drawing keeps the shape
[ADR-0042](../../docs/decisions/0042-draw-a-diagram-front-to-back-into-a-given-window.md) gave it —
front to back, every cell stamped with `Below`, into a buffer the caller gives — and a diagram's
shape keeps being its own entity, per
[ADR-0039](../../docs/decisions/0039-a-diagram-shape-is-its-own-entity.md).
