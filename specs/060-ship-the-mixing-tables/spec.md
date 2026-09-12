# Feature Specification: Ship the mixing tables

**Feature Branch**: `060-ship-the-mixing-tables-spec`

**Created**: 2026-09-11

**Status**: Draft

**Input**: [Issue #60](https://github.com/andresmoschini/monospace/issues/60) — `docs/glyph-sets.md`
holds four tables that mix two strokes: Light with Double, Light with Heavy, Light Round with
Double, Light Round with Heavy. They are what lets a light line meet a heavy box and draw the
junction both of them imply, instead of the junction one of them would draw alone. The issue wants
them in play and wants the difference visible — the same overlap drawn with the mixing table and
without it, side by side. The command-line demonstration already places figures where a light,
light-round, double or heavy shape overlaps a shape of a different one of those strokes; several of
those placements were laid out before any mixing table existed and do not land on a row a mixing
table defines, so they need to be corrected as part of delivering this feature rather than left as
they are.

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Draw with the Light and Heavy mixing table (Priority: P1)

Someone draws a diagram where a light line meets a heavy box, or a heavy line meets a light one, and
wants the junction that mixes the two strokes instead of one stroke overriding the other. They build
a catalog that includes the Light and Heavy mixing table alongside Light and Heavy themselves, and
the junction renders as the mixing table draws it.

**Why this priority**: it is the one mixing table that is complete — it covers all fifty
combinations light and heavy can form together — so it is both the most useful table on its own and
the clearest demonstration of what a mixing table adds over degrading to one stroke.

**Independent Test**: build one catalog from Light, Heavy and their mixing table, and a second from
Light and Heavy alone; render the same diagram containing a light/heavy overlap against both. The
two outputs must differ exactly at the overlap, the first matching the character _Mixing Light and
Heavy_ records for that combination and the second showing the stroke the degradation rule already
produces today.

**Acceptance Scenarios**:

1. **Given** a catalog built from Light, Heavy and their mixing table, **When** each of the fifty
   combinations _Mixing Light and Heavy_ records is rendered, **Then** every one produces the
   character the table publishes for it.
2. **Given** that same catalog, **When** a diagram is rendered that does not mix light and heavy
   anywhere, **Then** its output is unchanged from a catalog without the mixing table.
3. **Given** the shipped command-line demonstration, **When** it is rendered after this feature
   ships, **Then** the light/heavy overlap it places shows the junction the mixing table draws
   rather than the junction one stroke alone would draw.

---

### User Story 2 - Draw with the Light and Double mixing table (Priority: P2)

Someone draws a diagram where a light line meets a double box, or a double line meets a light one.
They build a catalog including the Light and Double mixing table, and the eighteen combinations it
covers render the mixed junction; the other thirty-two still degrade to one stroke, exactly as they
do today.

**Why this priority**: the second table the issue names, and the one whose incompleteness matters —
it is the case that proves a mixing table does not have to be complete to be worth shipping, and
that what it does not cover keeps degrading unchanged.

**Independent Test**: build one catalog from Light, Double and their mixing table, and a second from
Light and Double alone; render a diagram touching one of the eighteen covered combinations and one
of the thirty-two uncovered ones against both. The covered combination must differ between the two
outputs and match the table's published character; the uncovered one must render identically in
both.

**Acceptance Scenarios**:

1. **Given** a catalog built from Light, Double and their mixing table, **When** each of the
   eighteen combinations _Mixing Light and Double_ records is rendered, **Then** every one produces
   the character the table publishes for it.
2. **Given** that same catalog, **When** a light/double combination the table does not cover is
   rendered, **Then** it degrades to one stroke exactly as it would without the table.
3. **Given** the shipped command-line demonstration, **When** it is rendered after this feature
   ships, **Then** the light/double overlap it places shows the junction the mixing table draws.

---

### User Story 3 - Draw with the Light Round and Heavy mixing table (Priority: P3)

Someone wants Light Round's rounded corners to mix with Heavy the way Light already does. They build
a catalog including the Light Round and Heavy mixing table, and it behaves exactly as _Mixing Light
and Heavy_ does with `light-round` in place of `light`.

**Why this priority**: `docs/glyph-sets.md` already records this table as `light`'s copy with the
stroke name changed, so it carries no new junction to design — it is last because Stories 1 and 2
already establish the pattern this one only needs to repeat under a different name.

**Independent Test**: build a catalog from Light Round, Heavy and their mixing table; render all
fifty combinations and check each against the table's published character.

**Acceptance Scenarios**:

1. **Given** a catalog built from Light Round, Heavy and their mixing table, **When** each of the
   fifty combinations is rendered, **Then** every one produces the character the table publishes.
2. **Given** the shipped command-line demonstration, **When** it is rendered after this feature
   ships, **Then** the light-round/heavy overlap it places shows the junction the mixing table
   draws.

---

### User Story 4 - Draw with the Light Round and Double mixing table (Priority: P3)

Someone wants Light Round to mix with Double the way Light already does. They build a catalog
including the Light Round and Double mixing table, and the same eighteen combinations _Mixing Light
and Double_ covers render under the `light-round` name, the rest still degrading.

**Why this priority**: the last table the issue names, and the second of the two copies — it shares
its priority with Story 3 rather than trailing it, since neither depends on the other and both are
equally mechanical once Stories 1 and 2 establish the pattern.

**Independent Test**: build a catalog from Light Round, Double and their mixing table; render the
eighteen covered combinations and one uncovered one, and check the covered ones against the table's
published character and the uncovered one against the unchanged degradation rule.

**Acceptance Scenarios**:

1. **Given** a catalog built from Light Round, Double and their mixing table, **When** each of the
   eighteen combinations is rendered, **Then** every one produces the character the table publishes.
2. **Given** the shipped command-line demonstration, **When** it is rendered after this feature
   ships, **Then** the light-round/double overlap it places shows the junction the mixing table
   draws.

---

### Edge Cases

- **A combination one of the four mixing tables does not cover** (thirty-two of fifty for both
  Double pairings). It MUST still degrade to one stroke exactly as it does today; a mixing table
  narrows what degrades, it does not change how degradation works.
- **A key more than one of the nine tables now shipped — five single-stroke, four mixing — could
  claim.** None of the nine defines a key another does: every mixing table's keys carry two stroke
  names, every single-stroke table's keys carry one, and no two mixing tables pair the same two
  strokes. Building a catalog from any combination of them MUST answer every key from whichever
  table defines it, unaffected by the order they were added.
- **A figure already placed in the shipped demonstration that does not land on a row a mixing table
  defines.** Several exist today, laid out before these tables did. Delivering this feature MUST
  correct their placement so each of the four pairs is exercised by a combination its table actually
  covers, rather than leaving a placement that degrades regardless of this feature.
- **A diagram rendered before this feature that touches none of the four mixing tables' keys.** MUST
  render byte-identically after it.

## Requirements _(mandatory)_

### Functional Requirements

#### The four tables

- **FR-001**: `monospace-glyph-sets` MUST hold the Light and Double mixing table, with exactly the
  eighteen rows `docs/glyph-sets.md` records under _Mixing Light and Double_.
- **FR-002**: `monospace-glyph-sets` MUST hold the Light and Heavy mixing table, with exactly the
  fifty rows `docs/glyph-sets.md` records under _Mixing Light and Heavy_.
- **FR-003**: `monospace-glyph-sets` MUST hold the Light Round and Double mixing table, with exactly
  the eighteen rows `docs/glyph-sets.md` records under _Mixing Light Round and Double_.
- **FR-004**: `monospace-glyph-sets` MUST hold the Light Round and Heavy mixing table, with exactly
  the fifty rows `docs/glyph-sets.md` records under _Mixing Light Round and Heavy_.
- **FR-005**: Each of the four tables MUST be reachable the same way Double, Heavy and Light Round
  already are: usable to build a catalog through `monospace-core`'s public interface, with no
  privilege a table written outside this project does not equally have.
- **FR-006**: A catalog built from a mixing table and the two single-stroke tables its rows draw on
  MUST answer every combination the mixing table covers with the character it publishes, and MUST
  continue to degrade every combination it does not cover exactly as a catalog without it would.
- **FR-007**: None of the four mixing tables MUST define a key any other of the nine tables now
  shipped — the five single-stroke tables and the other three mixing tables — also defines. A
  catalog built from any combination of the nine MUST answer every key from whichever table defines
  it, unaffected by the order the tables were added.

#### The shipped demonstration

- **FR-008**: The command-line application's shipped demonstration MUST build its catalog from all
  four mixing tables in addition to the five single-stroke tables it already includes, so that every
  overlap it places between two shapes of differing strokes among light, light-round, double and
  heavy renders the junction the corresponding mixing table draws, wherever that table covers the
  combination the overlap produces.
- **FR-009**: The shipped demonstration's placements MUST be corrected so that each of the four
  mixing pairs is exercised by at least one combination its table covers, since some of today's
  placements were laid out before these tables existed and do not land on a covered row.

#### Scope

- **FR-010**: This feature MUST bring the four mixing tables and the demonstration change above, and
  no other. It MUST NOT add a command-line way to choose a catalog, and MUST NOT touch a
  single-stroke table or any figure in the demonstration unrelated to the four mixing pairs.

### Key Entities

- **Mixing Light and Double table**, **Mixing Light and Heavy table**, **Mixing Light Round and
  Double table**, **Mixing Light Round and Heavy table**: each a group of rules already written in
  `docs/glyph-sets.md`, moving from reference-only prose to data shipped in `monospace-glyph-sets`,
  with no more privilege than the single-stroke tables already there.
- **Catalog**: unchanged in meaning; this feature gives it four more sources it can be built from.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: The Light and Double mixing table answers all eighteen combinations it covers,
  measured by rendering each against a catalog built from it and the two tables it mixes.
- **SC-002**: The Light and Heavy mixing table answers all fifty combinations it covers, measured
  the same way.
- **SC-003**: The Light Round and Double mixing table answers all eighteen combinations it covers,
  measured the same way.
- **SC-004**: The Light Round and Heavy mixing table answers all fifty combinations it covers,
  measured the same way.
- **SC-005**: Every combination one of the four tables does not cover still degrades to one stroke,
  checked for at least one uncovered combination per incomplete table (Light and Double, Light Round
  and Double).
- **SC-006**: The shipped demonstration's rendered output differs from before this feature at
  exactly the positions where two of its shapes overlap on a combination one of the four mixing
  tables covers, and nowhere else.
- **SC-007**: A catalog built from any combination of the nine tables now shipped — five
  single-stroke, four mixing — answers every key correctly regardless of the order the tables were
  added to it.

## Assumptions

- **The shape of the interface is a plan decision.** How each table is expressed and exposed to a
  caller — a function per table, a shared constructor, something else — is not decided here,
  mirroring how features 054 and 056 left the same question to their plans.
- **How the demonstration's placements are corrected is a plan and implementation decision.** This
  spec requires the visible outcome — each of the four pairs exercised by a covered combination —
  not the specific positions or shapes that achieve it.
- **No new command-line capability is added.** Choosing a catalog, or showing the same overlap both
  with and without a mixing table in a single run, is a separate decision this feature does not
  take; the visible difference this feature delivers is the demonstration's own output changing once
  the tables are in play.
- **No new dependency is needed.** `monospace-glyph-sets` already exists and already depends on
  `monospace-core`.
- **No ADR is needed.** The extension point, the crate boundary, and the rule that an outside table
  gets no privilege were decided in ADR-0036 and feature 054; this feature applies that decision to
  four more tables rather than taking a new one.
