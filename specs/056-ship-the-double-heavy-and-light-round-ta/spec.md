# Feature Specification: Ship the Double, Heavy and Light Round tables

**Feature Branch**: `056-ship-the-double-heavy-and-light-round-ta-spec`

**Created**: 2026-09-11

**Status**: Draft

**Input**: [Issue #56](https://github.com/andresmoschini/monospace/issues/56) — `docs/glyph-sets.md`
holds nine tables. Two of them are in code — Light in the core, ASCII in `monospace-glyph-sets` —
and the rest are prose. The issue wants the three remaining single-stroke tables, Double, Heavy and
Light Round, in `monospace-glyph-sets` too, so that there is something to choose between and so that
the choice is a matter of taste rather than of what happens to be implemented. Each of the three
must be complete on its own — all fifteen combinations of a single stroke — because that is what the
model's degradation rule falls back on.

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Draw with the Double table (Priority: P1)

Someone drawing a diagram wants double-line box-drawing characters instead of Light or ASCII. They
build a catalog from the Double table that ships with `monospace-glyph-sets` and render their
diagram against it.

**Why this priority**: it is the first table the issue names, and on its own it already gives users
a second stroke to choose besides Light and ASCII — the reason the issue was raised.

**Independent Test**: build a catalog from the Double table alone, render a diagram exercising every
one of the fifteen non-empty top/right/bottom/left combinations of the double stroke, and check each
produces the character `docs/glyph-sets.md` records for it under _Double_.

**Acceptance Scenarios**:

1. **Given** a catalog built from the Double table alone, **When** a box is rendered, **Then** its
   corners, its horizontal run and its vertical run are the characters _Double_ records for them.
2. **Given** that same catalog, **When** each of the fifteen non-empty combinations of the double
   stroke is rendered, **Then** every one produces a glyph — none is missing.
3. **Given** a diagram whose shapes name a stroke the Double table does not cover, **When** it is
   rendered against a catalog holding only the Double table, **Then** the result is whatever a key
   no table answers already produces today, and it is not an error.

---

### User Story 2 - Draw with the Heavy table (Priority: P2)

Someone wants heavy-line box-drawing characters. They build a catalog from the Heavy table and
render against it, the same way they would with Double or ASCII.

**Why this priority**: the second table the issue names; it depends on nothing from Story 1 and adds
a third stroke to choose from.

**Independent Test**: build a catalog from the Heavy table alone, render a diagram exercising every
one of the fifteen non-empty combinations of the heavy stroke, and check each produces the character
_Heavy_ records for it.

**Acceptance Scenarios**:

1. **Given** a catalog built from the Heavy table alone, **When** a box is rendered, **Then** its
   corners, its horizontal run and its vertical run are the characters _Heavy_ records for them.
2. **Given** that same catalog, **When** each of the fifteen non-empty combinations of the heavy
   stroke is rendered, **Then** every one produces a glyph — none is missing.

---

### User Story 3 - Draw with the Light Round table (Priority: P3)

Someone wants Light's straight runs and junctions but rounded corners. They build a catalog from the
Light Round table and render against it. Its eleven straight and junction rows read exactly as
Light's do; its four corner rows read differently.

**Why this priority**: the last table the issue names. It is last because Stories 1 and 2 already
demonstrate the pattern; this story is also the one that proves a table sharing most of its
characters with another still answers independently and completely.

**Independent Test**: build a catalog from the Light Round table alone, render a diagram exercising
every one of the fifteen non-empty combinations of the light-round stroke, and check each produces
the character _Light Round_ records for it, including the four corners that differ from Light.

**Acceptance Scenarios**:

1. **Given** a catalog built from the Light Round table alone, **When** a box is rendered, **Then**
   its four corners are the rounded characters _Light Round_ records for them, distinct from Light's
   square corners.
2. **Given** that same catalog, **When** each of the fifteen non-empty combinations of the
   light-round stroke is rendered, **Then** every one produces a glyph — none is missing.

---

### Edge Cases

- **A key naming more than one of these strokes** (for example, a cell whose top arm is double and
  whose left arm is heavy). None of the four single-stroke tables in `monospace-glyph-sets` — ASCII,
  Double, Heavy, Light Round — covers such a key; the result is whatever a key no table answers
  already produces today, unchanged by this feature.
- **A catalog built from more than one of the four single-stroke tables at once** (say, Double and
  Heavy together, or all four). None of the four tables' rows share a stroke name with another's, so
  every key renders from whichever table names its stroke and the order the tables went into the
  catalog decides nothing.
- **Light Round's eleven straight and junction rows read as the same characters Light's do.** This
  is the expected duplication `docs/glyph-sets.md` already describes — the two tables' keys carry
  different stroke names even where their characters match, so it is not a collision and both tables
  still answer their own keys independently.
- **A row that would not match its published entry in `docs/glyph-sets.md`.** Must not happen; each
  table is checked row by row against the table the document already publishes for it.

## Requirements _(mandatory)_

### Functional Requirements

#### The three tables

- **FR-001**: `monospace-glyph-sets` MUST hold the Double table, with exactly the fifteen rows
  `docs/glyph-sets.md` records under _Double_ — none added, none omitted, none altered.
- **FR-002**: `monospace-glyph-sets` MUST hold the Heavy table, with exactly the fifteen rows
  `docs/glyph-sets.md` records under _Heavy_.
- **FR-003**: `monospace-glyph-sets` MUST hold the Light Round table, with exactly the fifteen rows
  `docs/glyph-sets.md` records under _Light Round_, including the four rows where it differs from
  Light.
- **FR-004**: A catalog built from the Double table alone MUST answer every one of the fifteen
  non-empty combinations of the double stroke.
- **FR-005**: A catalog built from the Heavy table alone MUST answer every one of the fifteen
  non-empty combinations of the heavy stroke.
- **FR-006**: A catalog built from the Light Round table alone MUST answer every one of the fifteen
  non-empty combinations of the light-round stroke.
- **FR-007**: Each of the three tables MUST be reachable the same way the ASCII table already is:
  usable to build a catalog through `monospace-core`'s public interface, with no privilege the ASCII
  table, or a table written outside this project, does not equally have.
- **FR-008**: A catalog built from more than one of the four single-stroke tables
  `monospace-glyph-sets` now holds (ASCII, Double, Heavy, Light Round) MUST let every key render
  from whichever of those tables names its stroke, since no two of the four define a key with the
  same stroke name.

#### Scope

- **FR-009**: This feature MUST bring the Double, Heavy and Light Round tables and no other. The
  four mixing sets `docs/glyph-sets.md` records — _Mixing Light and Double_, _Mixing Light and
  Heavy_, _Mixing Light Round and Double_, _Mixing Light Round and Heavy_ — MUST remain
  reference-only data, untouched by this feature.
- **FR-010**: The command-line application's shipped demonstration, and the catalog it builds by
  default, MUST NOT change in this feature. The three tables become available to build a catalog
  from; choosing one of them for the shipped demonstration is a separate decision this feature does
  not take.

#### The documents

- **FR-011**: `docs/model.md`'s account, under _Strokes, glyph sets and the catalog_, of which sets
  ship as built-in data MUST be corrected to name Double, Heavy and Light Round alongside Light and
  ASCII, since it currently says only those two ship as data and the rest load from a file.
- **FR-012**: `docs/glyph-sets.md`'s opening statement of which tables are carried as data and which
  remain reference only MUST be corrected to say that Double, Heavy and Light Round are now carried
  as data too.

### Key Entities

- **Double table**, **Heavy table**, **Light Round table**: each a group of rules as
  `docs/glyph-sets.md` already writes them — the three remaining single-stroke tables not yet in
  code. Each ships alongside ASCII in `monospace-glyph-sets`, with no more privilege than ASCII
  already has.
- **Catalog**: unchanged in meaning; this feature gives it three more sources it can be built from.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: The Double table answers all fifteen non-empty combinations of its own stroke,
  measured by rendering each combination against a catalog built from it alone.
- **SC-002**: The Heavy table answers all fifteen non-empty combinations of its own stroke, measured
  the same way.
- **SC-003**: The Light Round table answers all fifteen non-empty combinations of its own stroke,
  measured the same way, including the four corner combinations that read differently from Light's.
- **SC-004**: The count of rows held by each of the three tables is fifteen, and every row's
  character matches the row `docs/glyph-sets.md` publishes for it.
- **SC-005**: A diagram rendered against a catalog holding only the Double, only the Heavy, or only
  the Light Round table contains no character outside that table's own fifteen glyphs and space —
  checked over every character of the output, not sampled.
- **SC-006**: A catalog built from two or more of the four single-stroke tables answers a key from
  each of them correctly, and the result is the same regardless of the order the tables went into
  it, since none of the four contests a key another already holds.
- **SC-007**: Every diagram the project rendered before this feature renders byte-identically after
  it; this feature adds tables and changes no existing output.

## Assumptions

- **The shape of the interface is a plan decision.** How each table is expressed and exposed to a
  caller — a function per table, a shared constructor, something else — is not decided here, only
  that each table can build a catalog and answers completely, mirroring how feature 054 left the
  ASCII table's interface to its plan.
- **The shipped demonstration is unchanged.** The issue asks for the tables to exist to choose
  between, not for the demonstration to show them. Wiring one of the three into the demo, or adding
  a command-line way to choose a table, is a separate decision this feature does not take.
- **The mixing sets stay reference-only.** This feature is the three single-stroke tables the issue
  names and nothing else.
- **No new dependency is needed.** `monospace-glyph-sets` already exists and already depends on
  `monospace-core`.
- **No ADR is needed.** The extension point, the crate boundary, and the rule that an outside table
  gets no privilege were all decided in ADR-0036 and feature 054. This feature applies that decision
  to three more tables rather than taking a new one.
