# Feature Specification: Hold a literal glyph in a cell

**Feature Branch**: `028-hold-a-literal-glyph-in-a-cell`

**Created**: 2026-09-09

**Status**: Draft

**Input**: Issue #28, verbatim: "A cell today can only draw what its arms imply. We want a cell that
can hold one chosen glyph instead — a letter, or a fill character that occludes what's behind it."

## Source material

This feature originates from [spec 0005](../../docs/specs/0005-hold-a-literal-glyph-in-a-cell.md),
the draft that was frozen and abandoned when the spec home moved to Spec Kit — see its own header
note and [ADR-0021](../../docs/decisions/0021-move-the-spec-home-to-spec-kit.md). It is recorded
here rather than in a `research.md` because that file is a Phase 0 output of `/speckit-plan` and
does not exist yet.

That document's _Public surface_ and _Behavior_ sections are deliberately not folded into this spec:
they are the shape of the answer rather than what must be true, and they are deferred to
`/speckit-plan`, which has them as its input.

## User Scenarios & Testing _(mandatory)_

Two stories: the wish, and the demonstration that shows it. The first is testable through the
library before anything on the terminal changes, and both leave the workspace green.

### User Story 1 - A cell can hold a chosen glyph (Priority: P1)

A caller wants one position to draw an `A` and another to draw a shade that covers whatever is
underneath. Today the only thing a cell can say is which strokes reach which of its sides, and the
character is always derived from those through the catalog: no combination of arms spells a letter,
and the nearest thing to a fill is a cell that closes all four of its sides, which draws a space
only because no set defines the empty key — fragile, as spec 0001 already wrote down, and never
anything but a space. This story lets a cell hold one chosen glyph instead: the renderer answers it
with that glyph directly, nothing connects into it, and where it meets another figure the one in
front decides which of the two kinds the position ends up being. It is the first character this
library draws that nobody looked up.

**Why this priority**: it is the whole of the wish, and it has no workaround. It also comes before
anything to do with text, because a word is many chosen glyphs with something above the buffer
deciding where each one goes, and this is one chosen glyph with nothing above it.

**Why holding and occluding are one story rather than two**: they are not separable. The moment a
cell can be one of two things, every stamp has to answer what happens where the two kinds meet —
there is no shipping half of that, and a rule that ships with no test named against it is an
unfinished spec rather than a finished feature. So occlusion arrives with the first cell that can
hold a glyph. The split that is real here is structural against behavioral, and it belongs to the
plan rather than to this list.

**Independent Test**: stamp a chosen glyph into an empty buffer and read it back out of the rendered
text; stamp a figure and a chosen glyph onto one position in both stamp modes and both orders, and
read the resulting cells back out of the buffer; stamp three figures with a chosen glyph in the
middle, once front to back and once back to front, and compare the two buffers; then render a
diagram made only of arm cells and confirm the text is what it was before the slice.

**Acceptance Scenarios**:

1. **Given** an empty buffer, **When** a cell holding the glyph `"A"` is stamped at a position and
   the rectangle is rendered, **Then** that position reads `A`.
2. **Given** a cell holding a glyph that no rule in the catalog mentions, **When** it is rendered,
   **Then** it still reads as that glyph: the catalog is not consulted, and the absence of a rule
   costs nothing.
3. **Given** a position holding a chosen glyph, **When** the cell there is inspected, **Then** it is
   distinguishable from a cell of arms, and it has no arms to read.
4. **Given** a position holding a cell of arms, **When** a cell holding a chosen glyph is stamped
   there by a figure in front, **Then** the position holds the chosen glyph and what the arms would
   have drawn is gone.
5. **Given** a position holding a chosen glyph, **When** a figure behind stamps a cell of arms
   there, **Then** the position still holds the chosen glyph.
6. **Given** a position holding a chosen glyph, **When** a figure in front stamps a cell of arms
   there whose sides it does not decide, **Then** those sides come out refusing a junction rather
   than left open: the chosen glyph is gone, and what it decided about its four sides is not.
7. **Given** any stack of figures at one position, some holding chosen glyphs and some cells of
   arms, **When** it is stamped front to back and again back to front in the mode each order calls
   for, **Then** the two buffers hold the same cell at that position.
8. **Given** a position holding a chosen glyph, **When** it is asked whether it is decided, **Then**
   it answers yes, so a walk from the front can stop at it.
9. **Given** a diagram built only from cells of arms, **When** it is rendered before and after this
   story, **Then** the two outputs are identical byte for byte.

---

### User Story 2 - The terminal shows both halves at once (Priority: P2)

The front end draws one box and the same box twice overlapping, stamped both ways, and its interiors
are empty. A reader looking at it cannot see that a figure in front hides anything, because nothing
in the picture is opaque. This story fills the box interiors, so the single box shows a fill and the
overlapping pair shows occlusion and junctions in the same picture, once per stamp mode.

**Why this priority**: it is what makes the increment demonstrable, and it is the only part a reader
who does not run the tests can see. It is second because it consumes the story above and adds no
rule of its own — it is not independent of it, and pretending otherwise would only hide which of the
two can be shown on its own.

**Independent Test**: run `cargo run -p monospace-cli` and compare the whole output, trailing spaces
included, against what the picture is asserted to be.

**Acceptance Scenarios**:

1. **Given** the front end, **When** it is run, **Then** the single box has a filled interior rather
   than a blank one.
2. **Given** the overlapping pair, **When** it is drawn with the second box in front and again with
   the first box in front, **Then** the two pictures differ from each other, and in each of them at
   least one position shows a fill covering what a border of the other box would have drawn.
3. **Given** the front end's output, **When** it is compared against the assertion in the CLI test,
   **Then** the whole text matches, trailing spaces included.
4. **Given** the front end, **When** the fill is added, **Then** the box is still built once and
   stamped twice, with the fill part of the same construction.

---

### Edge Cases

The middle column is the outcome, not the rule that produces it: composition is owned by _Stamping_
in [`docs/model.md`](../../docs/model.md) and is named rather than restated here.

| Case                                                        | Expected                                                                                                   | Settled by                        |
| ----------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------- | --------------------------------- |
| A stroke reaching the side of a cell that holds a glyph     | It stops there and draws no junction: nothing connects into a chosen glyph                                 | _A cell can be a literal instead_ |
| Two figures both holding a chosen glyph at one position     | The one in front decides which glyph the position holds                                                    | _Stamping_                        |
| A chosen glyph between two figures of arms in a stack       | Both walk orders end at the same cell, which is what the glyph's refusal of every side is for              | _The two orders are equivalent_   |
| A chosen glyph the catalog has no rule for                  | Rendered as itself: no key is built, no lookup happens, and degradation never applies                      | _Rendering_                       |
| A position with no cell at all                              | Still a space, unchanged                                                                                   | _Rendering_                       |
| A glyph wider than one column in a cell                     | Accepted and it shifts the rest of its row: documented, not enforced, and nothing this slice draws is wide | ADR-0019                          |
| A caller wanting a chosen glyph that a stroke connects into | Not expressible, deliberately: that would be a different shape of cell, not a variation of this one        | Out of scope, below               |

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001** (P1): A cell MUST be able to hold one chosen glyph in place of a base stroke and four
  arms. The two kinds MUST be mutually exclusive: no cell is both, and no cell is neither.
- **FR-002** (P1): A cell holding a chosen glyph MUST render as exactly that glyph, without
  consulting the catalog. Whether a rule exists for anything MUST make no difference to it.
- **FR-003** (P1): A cell holding a chosen glyph MUST have no arms to read, and MUST NOT be
  constructible with arms of its own. What it decides about its four sides exists in how it
  composes, not as something a caller can set or inspect.
- **FR-004** (P1): Rendering a cell of arms MUST be unchanged: the same key, the same lookup, the
  same fallback, the same space where there is no answer. A diagram with no chosen glyph in it MUST
  render byte for byte as it did before this feature.
- **FR-005** (P1): A cell holding a chosen glyph MUST occlude what a figure behind it draws at that
  position: after the figure in front has stamped, nothing of what was there shows through.
- **FR-006** (P1): Nothing MUST connect into a cell holding a chosen glyph. A neighboring stroke
  stops against it, and a figure that stamps arms over it inherits its refusal on every side the
  incoming figure does not decide for itself.
- **FR-007** (P1): The four combinations of the two kinds of cell MUST compose exactly as the table
  in _Stamping_ of [`docs/model.md`](../../docs/model.md) already says, in both stamp modes. This
  spec names that section and MUST NOT restate it; if implementing shows the model is wrong, the
  model changes first. No composition rule is decided here: the rows where no chosen glyph is
  involved are [spec 0001](../../docs/specs/0001-stamp-cells-and-render-them.md)'s and
  [spec 0003](../../docs/specs/0003-stamp-below-what-is-there.md)'s, implemented and tested already,
  and the three rows that mention a literal have been model prose since commit `addbc01`, the
  amendment written ahead of the source draft. All of them rest on
  [ADR-0008](../../docs/decisions/0008-compose-overlapping-cells-with-three-state-arms.md). What
  this feature adds is a cell that reaches those rows, not a rule for them.
- **FR-008** (P1): The equivalence of the two stamp orders MUST keep holding when a chosen glyph is
  in the stack, as _Properties worth testing_ already requires. A chosen glyph MUST NOT be opaque in
  one order and transparent in the other.
- **FR-009** (P1): A position holding a chosen glyph MUST answer that it is already decided, so a
  walk from the front can stop at it and a stamp from behind is skipped rather than examined.
- **FR-010** (P2): The front end MUST draw a filled box, and the overlapping pair in both stamp
  modes, so that one picture shows occlusion and junctions together. Its whole output, trailing
  spaces included, MUST be asserted end to end.
- **FR-011** (P1): Every public item this feature adds or changes MUST carry rustdoc, and the
  documentation of a cell MUST say which of the two kinds wins where they meet — that is the
  sentence a reader would otherwise reconstruct from the composition code.
- **FR-012** (P1): What a cell holds MUST be the validated glyph feature 006 introduced. This
  feature MUST NOT add a second notion of "the thing a cell draws", and MUST NOT widen or narrow
  that type's invariant.

### Key Entities

- **Cell**: one of two things — a base stroke with four arms, or one chosen glyph. _Vocabulary_ in
  [`docs/model.md`](../../docs/model.md) already defines it that way; this feature is what makes the
  second half real.
- **Glyph**: unchanged, from feature 006. One grapheme cluster, never a control character, validated
  on construction, so a cell holding one has nothing left to check.
- **Buffer**: unchanged in what it is. Each of its positions holds a cell or nothing, and a cell may
  now be either kind.
- **Arm**, **Stroke**, **GlyphKey**, **GlyphCatalog**: untouched. No rule changes, no key is built
  differently, and the catalog gains nothing.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: `cargo run -p monospace-cli` prints a filled box and the overlapping pair in both
  stamp modes, and `crates/monospace-cli/tests/cli.rs` asserts that whole output, trailing spaces
  included.
- **SC-002**: the two pair pictures differ from each other, and each differs from what the front end
  printed before this feature. Countable: the diff of the asserted string is not empty in either
  half.
- **SC-003**: every case in the table of _Stamping_ that involves a chosen glyph has a test per
  stamp mode, named after what it asserts, reading the buffer back through the accessor
  [ADR-0011](../../docs/decisions/0011-expose-cell-for-testing-stamping.md) exposes.
- **SC-004**: one test stamps a stack with a chosen glyph between two figures of arms, front to back
  and back to front, and asserts the shared position is equal in both buffers. It is verified by
  being made to fail on purpose — the glyph stops refusing the sides it loses — and then restored,
  as _Claims are measured, not assumed_ requires.
- **SC-005**: no existing assertion about a cell of arms changes its expected value. Test code may
  be edited to name whatever the plan introduces, but what it asserts stays as it is, and the
  rendered output of arm-only diagrams is identical.
- **SC-006**: `cargo xtask check` passes at every commit, and on the fresh clone CI checks out
  rather than only in the working copy.
- **SC-007**: a reader of the front end's output can point at a position where a fill covers a
  border in one picture and the border survives the fill in the other. Both are visible in the
  asserted text, which is what makes this checkable rather than a matter of taste.

## Assumptions

- **The model needs no amendment.** Verified by reading [`docs/model.md`](../../docs/model.md): _The
  cell_ already carries the subsection _A cell can be a literal instead_, the table in _Stamping_
  already has its three rows for a literal, _Rendering_ already ends its numbered list with the
  sentence that a literal renders as its glyph, and _Properties worth testing_ already asks for the
  equivalence with a literal in the middle. This feature implements those as they stand.
- **No decision is taken here.** _Decisions recorded when taken_ owns that: if planning surfaces
  one, the ADR is written before the plan depends on it. The decisions already in force —
  [ADR-0008](../../docs/decisions/0008-compose-overlapping-cells-with-three-state-arms.md) for
  composition, [ADR-0017](../../docs/decisions/0017-ask-the-cell-whether-it-is-decided.md) for
  asking a cell whether it is decided, and
  [ADR-0019](../../docs/decisions/0019-represent-a-glyph-as-a-grapheme-cluster.md) for what a glyph
  is — are inputs, not choices to revisit.
- **The demonstration's fill character is a demonstration.** The source draft chose `░` because a
  shade shows on a terminal what a space cannot. It is the front end's choice for one picture, not a
  default the library gains: what a figure is filled with, and what it is filled with when nobody
  says, belongs to the slice that gives figures a way to say so.
- **The filled pair replaces the empty one rather than joining it.** The front end shows one thing
  per capability, and the filled pictures show everything the empty ones showed plus occlusion. This
  follows the source draft; it is the maintainer's to overrule, and overruling it changes only
  SC-001's asserted text.
- **Nothing wide reaches the terminal.** The fill this feature draws is one column, so the rectangle
  the renderer promises is not tested against a glyph that would shift a row. ADR-0019 left width
  outside the invariant deliberately and this feature does not revisit it.
- **The feature number is the issue number.** `028` comes from issue #28, per
  [ADR-0024](../../docs/decisions/0024-take-the-feature-number-from-its-issue.md), not from a
  position in a local sequence.

## Out of scope

Every item names where it is handled instead.

- **Text.** A later feature. A word is many chosen glyphs and something above the buffer deciding
  where each one goes; this feature has no opinion about what puts a glyph anywhere.
- **A figure as a thing in code.** As in spec 0002, the front end still stamps cell by cell, and a
  filled box is a few more stamps rather than a new kind of object in the library.
- **A chosen glyph a stroke can connect into.** Deliberately not expressible: a letter is something
  a stroke stops against. Wanting a junction into one is a different shape of cell and needs its own
  decision, not a change here.
- **Degradation.** [ADR-0009](../../docs/decisions/0009-degrade-a-cell-to-its-base-stroke.md), still
  unimplemented, and now with one case fewer to worry about since a chosen glyph never reaches a
  lookup.
- **Column width.** ADR-0019 settled it: a wide grapheme is accepted and shifts its row, documented
  rather than enforced.
- **A default fill for a figure.** The slice that gives figures a fill of their own is where the
  default belongs.
- **Everything spec 0001 left out and this spec does not name** — a catalog from more than one set,
  loading sets from a file, walking a region — stays out, with the destinations that spec gave.

The near miss is that a filled box looks like a new capability in the library. It is not. What the
library gains is a cell that can hold a chosen glyph; "filled" is a word for what the front end does
with a few of them.

## Handoff to the plan

Three things this spec surfaces and does not own:

- **The shape of a cell that is one of two things.** The source draft's _Public surface_ has a
  proposal and the reasoning behind it. It is the plan's input, and whether it needs an ADR of its
  own is the plan's first question.
- **The commit split.** Every construction site in the workspace reads a cell today, so making the
  change easy is likely to touch more code than the change itself. _Structural and behavioral change
  never share a commit_ governs it, and the source draft's closing note proposes an order.
- **The source draft's numbered _Behavior_ rules and its example tables.** They are already worked
  out against the Light table and they map onto FR-005 through FR-009. The plan is where they become
  the acceptance detail, and where every picture in them is confirmed by running rather than by
  derivation.
