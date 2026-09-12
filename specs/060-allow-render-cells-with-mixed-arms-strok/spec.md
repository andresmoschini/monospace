<!-- The branch and directory names are truncated by the tooling that creates them; "strok" is
     that truncation, not a word. cspell:ignore strok -->

# Feature Specification: Allow render cells with mixed arms' strokes

**Feature Branch**: `060-allow-render-cells-with-mixed-arms-strok-spec`

**Created**: 2026-09-12

**Status**: Draft

**Input**: [Issue #60](https://github.com/andresmoschini/monospace/issues/60) — when two figures of
different strokes overlap, the figure underneath loses its stroke: the whole junction is drawn in
the stroke of the figure on top. Unicode has characters for many of those junctions, and
`docs/glyph-sets.md` already records them in four tables that mix two strokes — Light with Double,
Light with Heavy, Light Round with Double, Light Round with Heavy. The issue asks for the junction
both figures imply to be drawn where such a character exists, and for the difference to be visible
by running the command-line demonstration on its shipped demo file, with no change to that file.

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Draw the junction two strokes make (Priority: P1)

Someone draws a light line across a double border, or a heavy box over a light one. Today the
crossing is drawn entirely in the stroke of whichever figure was placed on top, and the other
figure's stroke stops dead at that position. They want the character that shows both: a light
vertical crossing a double horizontal drawn as such, not as a double cross.

**Why this priority**: it is the capability the issue asks for. Nothing else in this feature is
visible without it, and it is what the four mixing tables exist to serve.

**Independent Test**: with a catalog holding Light, Heavy and their mixing table, build a cell whose
top and bottom sides carry `light` and whose left and right sides carry `heavy`, and render it. It
must produce `┿`, the character _Mixing Light and Heavy_ publishes for that combination, rather than
the cross of either stroke alone.

**Acceptance Scenarios**:

1. **Given** a catalog holding Light, Heavy and the Light-with-Heavy mixing table, **When** a cell
   whose sides carry `light` on top and bottom and `heavy` on left and right is rendered, **Then**
   it produces `┿`.
2. **Given** the same catalog, **When** the same cell is rendered with `heavy` on top and bottom and
   `light` on left and right, **Then** it produces `╂` — the mixture is read per side, not as an
   unordered pair of strokes.
3. **Given** a catalog holding Light, Double and the Light-with-Double mixing table, **When** each
   of the eighteen combinations _Mixing Light and Double_ records is rendered, **Then** every one
   produces the character that table publishes for it.
4. **Given** any catalog, **When** a cell whose sides all carry one stroke is rendered, **Then** it
   produces exactly the character it produces today.
5. **Given** two figures of different strokes overlapping, **When** they are drawn front to back and
   again back to front, **Then** both orders produce the same buffer, as they do today.

---

### User Story 2 - See the difference in the shipped demonstration (Priority: P1)

Someone who has never read this repository runs the command-line application with no arguments and
sees, in its last figure group, light, double and heavy boxes overlapping each other. Today every
one of those crossings is drawn in one stroke. They want to run the same command, against the same
shipped demo file, and see the mixed junctions instead.

**Why this priority**: the issue names this as the way the feature is judged — "the real difference
should be visible running the CLI with the demo JSON file without making changes in it". It is the
only end-to-end evidence that the capability of Story 1 reaches a user.

**Independent Test**: run the application with no arguments before and after the feature and compare
the two outputs. They must differ only in the eight characters at the crossings of the light/double
and light/heavy pairs, and match the expected block below.

**Acceptance Scenarios**:

1. **Given** the shipped demonstration, unchanged, **When** it is run with no arguments, **Then**
   the figure group that today prints

   ```text
   ┌──┐    ╔══╗    ┌──┐    ┏━━┓    ┌──┐    ┏━━┓
   │ ╔╬═╗  ║ ┌┼─┐  │ ┏╋━┓  ┃ ┌┼─┐  │ ╭┼─╮  ┃ ╔╬═╗
   └─╬┘ ║  ╚═┼╝ │  └─╋┘ ┃  ┗━┼┛ │  └─┼┘ │  ┗━╬┛ ║
     ╚══╝    └──┘    ┗━━┛    └──┘    ╰──╯    ╚══╝
   ```

   prints instead

   ```text
   ┌──┐    ╔══╗    ┌──┐    ┏━━┓    ┌──┐    ┏━━┓
   │ ╔╪═╗  ║ ┌╫─┐  │ ┏┿━┓  ┃ ┌╂─┐  │ ╭┼─╮  ┃ ╔╬═╗
   └─╫┘ ║  ╚═╪╝ │  └─╂┘ ┃  ┗━┿┛ │  └─┼┘ │  ┗━╬┛ ║
     ╚══╝    └──┘    ┗━━┛    └──┘    ╰──╯    ╚══╝
   ```

2. **Given** the shipped demonstration, **When** it is run with no arguments, **Then** every
   character outside those eight positions is byte-identical to what it prints today, trailing
   spaces and line endings included.
3. **Given** the shipped demo file, **When** this feature is delivered, **Then** that file is
   unchanged: no figure is moved, added or removed to make the difference appear.

---

### User Story 3 - Build a catalog from a mixing table (Priority: P2)

Someone assembling their own catalog wants the Light-with-Double junctions but not the Heavy ones,
or wants Light Round to mix the way Light does. They pick the mixing tables they want alongside the
single-stroke tables they already use, and the catalog answers accordingly.

**Why this priority**: the four tables have to be shipped for Stories 1 and 2 to be possible at all,
but a caller choosing among them is a use the demonstration does not exercise, so it is worth
stating and testing separately.

**Independent Test**: build a catalog from Light Round, Heavy and their mixing table, render all
fifty combinations that table records, and check each against the character it publishes. Then build
one without the mixing table and confirm the same fifty degrade.

**Acceptance Scenarios**:

1. **Given** the four mixing tables, **When** each is used to build a catalog, **Then** it holds
   exactly the rows `docs/glyph-sets.md` records for it: eighteen for Light with Double, fifty for
   Light with Heavy, eighteen for Light Round with Double, fifty for Light Round with Heavy.
2. **Given** any selection of the nine tables this project would then ship — five single-stroke and
   four mixing — **When** a catalog is built from them in any order, **Then** every key is answered
   by the table that defines it, because no two of the nine define the same key.
3. **Given** a mixing table, **When** it is reached through the public interface, **Then** it is
   reached the same way Double, Heavy and Light Round already are, with no privilege a table written
   outside this project would not equally have.

---

### User Story 4 - Keep the junctions no table covers exactly as they are (Priority: P2)

Someone's diagram mixes two strokes that no table pairs — a light box over a light-round one, a
heavy box over a double one — or mixes light and double in one of the thirty-two combinations
Unicode has no character for. They want those to keep drawing what they draw today rather than
becoming spaces or changing shape.

**Why this priority**: this feature narrows what degrades; it must not change how degradation works.
Without this story the feature could ship a regression in every diagram it was not aiming at.

**Independent Test**: render a combination one of the incomplete tables does not cover, with and
without that table in the catalog, and confirm both produce the same character — the one the cell's
base stroke draws.

**Acceptance Scenarios**:

1. **Given** a catalog holding Light, Double and their mixing table, **When** a light/double
   combination among the thirty-two that table does not record is rendered, **Then** it degrades to
   the cell's base stroke exactly as it would with no mixing table loaded.
2. **Given** a catalog holding Light and Light Round, **When** a cell mixing those two strokes is
   rendered, **Then** it degrades, because no table pairs them — as the fifth figure pair in the
   demonstration shows, unchanged.
3. **Given** a catalog holding Heavy and Double, **When** a cell mixing those two strokes is
   rendered, **Then** it degrades, because no table pairs them — as the sixth figure pair in the
   demonstration shows, unchanged.
4. **Given** any diagram that mixes no strokes anywhere, **When** it is rendered before and after
   this feature, **Then** the two outputs are byte-identical.

---

### Edge Cases

- **A side with no stroke.** A side that is closed, and a side left for a later figure to decide,
  both mean "no stroke runs here" when the character is chosen. That does not change: a mixture is
  read from the sides that do carry a stroke.
- **A cell holding a literal character.** It renders as that character, with no junction chosen and
  no degradation, exactly as today.
- **A figure drawn over a junction that already mixes two strokes.** The figure on top decides every
  side it claims and imposes its own stroke there; sides it leaves for others keep the stroke they
  were drawn with, which may be a third one. Nothing caps how many strokes one cell's sides may
  name; what caps the result is whether a table has a character for it.
- **A mixture of three or more strokes.** No table this project ships records one, so it degrades.
  That is the same rule as any other uncovered mixture, not a special case.
- **The two drawing orders.** Front to back and back to front must still produce the same buffer.
  Which figure sets the base stroke is unchanged by this feature, and so is the answer when a
  junction degrades.

## Requirements _(mandatory)_

### Functional Requirements

#### The cell

- **FR-001**: A cell MUST be able to carry a different stroke on each of its four sides, per
  [ADR-0037](../../docs/decisions/0037-give-each-arm-its-own-stroke.md).
- **FR-002**: The character chosen for a cell MUST be looked up from the stroke each side actually
  carries, rather than from the cell's base stroke, so that a cell mixing two strokes asks for a
  mixed character.
- **FR-003**: When a figure is drawn over another, a side the upper figure does not claim MUST keep
  the stroke it was drawn with, instead of being redrawn in the upper figure's stroke.
- **FR-004**: The base stroke MUST keep exactly the role
  [ADR-0009](../../docs/decisions/0009-degrade-a-cell-to-its-base-stroke.md) gives it: the stroke
  every side with a stroke moves to when no character matches the exact mixture. Degradation MUST
  NOT change in any other way.
- **FR-005**: Which figure decides a side, and which figure sets the base stroke, MUST NOT change:
  the two stamp modes and the three states of a side are untouched, and drawing front to back MUST
  still produce the same buffer as drawing back to front.

#### The tables

- **FR-006**: The project MUST ship the Light-with-Double mixing table, with exactly the eighteen
  rows `docs/glyph-sets.md` records under _Mixing Light and Double_.
- **FR-007**: The project MUST ship the Light-with-Heavy mixing table, with exactly the fifty rows
  recorded under _Mixing Light and Heavy_.
- **FR-008**: The project MUST ship the Light-Round-with-Double mixing table, with exactly the
  eighteen rows recorded under _Mixing Light Round and Double_.
- **FR-009**: The project MUST ship the Light-Round-with-Heavy mixing table, with exactly the fifty
  rows recorded under _Mixing Light Round and Heavy_.
- **FR-010**: Each of the four MUST be usable to build a catalog the same way Double, Heavy and
  Light Round already are, with no privilege a table written outside this project would not equally
  have, per [ADR-0036](../../docs/decisions/0036-hold-every-table-but-light-outside-the-core.md).
- **FR-011**: No two of the nine tables the project would then ship MUST define the same key, so
  that a catalog built from any selection of them in any order answers every key from the table that
  defines it.
- **FR-012**: The sentence in `docs/glyph-sets.md` saying the mixing sets are reference only, with
  nothing loading them, MUST be corrected once they are shipped.

#### The demonstration

- **FR-013**: The command-line application's shipped demonstration MUST build its catalog from the
  four mixing tables in addition to the five single-stroke tables it already uses.
- **FR-014**: Running the application with no arguments MUST produce the expected block in User
  Story 2, differing from today's output at exactly the eight crossing positions of its light/double
  and light/heavy figure pairs and nowhere else.
- **FR-015**: The shipped demo file MUST NOT be modified. The difference MUST come from the
  rendering and the catalog, not from moving, adding or removing a figure.

#### Scope

- **FR-016**: This feature MUST NOT add a way to choose a catalog from the command line, MUST NOT
  add or change a single-stroke table, and MUST NOT change any figure group of the demonstration
  other than through the characters at its existing crossings.

### Key Entities

- **A side of a cell**: today it says whether a stroke runs that way; it gains the stroke that runs
  there. The three states — claimed, closed, left to others — are unchanged.
- **Base stroke**: unchanged in meaning, narrowed in use. It is now read only when the exact mixture
  has no character.
- **Mixing table**: a group of rules keyed on two stroke names at once, already written in
  `docs/glyph-sets.md` and moving from reference prose to data the project ships. Four of them.
- **Catalog**: unchanged; it gains four more tables it can be built from.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: All fifty combinations _Mixing Light and Heavy_ records, and all fifty _Mixing Light
  Round and Heavy_ records, render the character their table publishes.
- **SC-002**: All eighteen combinations _Mixing Light and Double_ records, and all eighteen _Mixing
  Light Round and Double_ records, render the character their table publishes.
- **SC-003**: Running the command-line application with no arguments produces output that differs
  from today's at exactly eight character positions, all of them crossings between two figures of
  different strokes, and matches the expected block in User Story 2 byte for byte.
- **SC-004**: The shipped demo file is identical before and after the feature, verified by diff.
- **SC-005**: At least one light/double combination the mixing table does not record renders the
  same character with and without that table in the catalog.
- **SC-006**: A diagram that mixes no strokes renders byte-identically before and after the feature
  — demonstrated by the first two figure groups of the demonstration, which are unchanged.
- **SC-007**: Building a catalog from all nine shipped tables in any order answers every key the
  same way, checked across at least two different orders.

## Assumptions

- **The decision to give each side its own stroke is already recorded.** It is
  [ADR-0037](../../docs/decisions/0037-give-each-arm-its-own-stroke.md), which supersedes ADR-0012.
  This spec applies it rather than taking it.
- **`docs/model.md` needs no change.** Under _Rendering_ and _Worked examples_ it already describes
  the mixed key, the two lookups and this exact light-over-double crossing. What changes is that the
  code catches up with it.
- **The shape of the interface is a plan decision.** How a side carries its stroke, and how each of
  the four tables is exposed to a caller, are left to `/speckit-plan`, as features 054 and 056 left
  the same question.
- **No new dependency and no new crate.** `monospace-glyph-sets` already exists and already holds
  the four single-stroke tables the core does not ship.
- **This cannot be a structural commit.** Giving a side its own stroke changes how junctions render
  and touches every place a cell is built, tests included, so it is behavioral work under
  [principle V](../../.specify/memory/constitution.md) — a cost ADR-0012 recorded in advance.
- **No command-line surface is added.** Showing the same crossing with and without a mixing table in
  one run would be a new capability; the visible difference this feature delivers is the shipped
  demonstration's own output changing.
- **The characters in the expected block are verified against `docs/glyph-sets.md`**, row by row,
  and the block it replaces is the current output of `cargo run -p monospace-cli`, observed rather
  than assumed. Whether the application actually prints the expected block is what the feature has
  to make true, and is not claimed here.
