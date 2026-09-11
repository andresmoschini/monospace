# Feature Specification: Glyph tables can come from outside the core

**Feature Branch**: `054-glyph-tables-can-come-from-outside-the-c-spec`

**Created**: 2026-09-11

**Status**: Draft

**Input**: [Issue #54](https://github.com/andresmoschini/monospace/issues/54) — the core ships the
Light table and there is nowhere else a table can come from, so "the glyph sets are extensible" is
something the model says and nothing demonstrates. A second library holds the tables that are not
Light, depends on the core, and is no more privileged than a crate somebody else would write; the
command-line application builds one catalog out of both and renders with it. The table it brings is
ASCII. User description: "Ensure to add demonstration shapes in the CLI with the default JSON file."

## User Scenarios & Testing _(mandatory)_

### User Story 1 - A table written outside the core answers keys like any other (Priority: P1)

Someone who is not this project wants a glyph table the project does not ship. They write it in
their own library, which depends on the diagramming library and sees nothing of it that is not
public. They put their table and the shipped one into a catalog, in an order they choose, and
render. Both answer. Nothing downstream can say which library a rule came from.

**Why this priority**: it is the whole claim. Every other story here is a consequence of it, and
without it the model's statement that a set may ship with a library or be loaded, and that neither
gets special behavior, has nothing behind it.

**Independent Test**: define a table in a crate that depends on the diagramming library through its
public interface only, build a catalog holding that table together with the shipped one, and render
a diagram that uses a key from each. Delivers the extension point and the evidence that it is not
privileged.

**Acceptance Scenarios**:

1. **Given** a table defined outside the diagramming library, **When** a catalog is built from it,
   **Then** rendering a cell whose key that table holds produces that table's character.
2. **Given** a catalog built from the shipped table and an outside table, **When** a diagram uses
   keys from both, **Then** every cell renders from whichever table holds its key, and the result is
   the same as if one table had held all of them.
3. **Given** the same two tables, **When** the catalog is built with them in the other order,
   **Then** every key that only one of them holds renders identically, because order decides nothing
   where nothing is contested.
4. **Given** two tables that both claim one key, **When** a catalog is built from them in a stated
   order, **Then** the one inserted first answers that key, and swapping the order swaps the answer.
5. **Given** a built catalog, **When** anything downstream asks about a rendered character, **Then**
   there is no operation that reports which table it came from.

---

### User Story 2 - A diagram drawn in ASCII alone (Priority: P2)

Someone's terminal, pipeline or destination cannot carry box-drawing characters. They render the
same diagram against a catalog holding the ASCII table and nothing else, and get a diagram made of
`+`, `-` and `|` that means the same thing.

**Why this priority**: it is the reason ASCII is the table worth bringing rather than an arbitrary
one, and it is the first thing the extension point is good for. It depends on story 1 for the
mechanism but is worth having on its own.

**Independent Test**: build a catalog from the ASCII table alone, render a diagram of boxes and
lines against it, and check that every character of the output is one the ASCII table holds, or a
space. Delivers a rendering path for terminals that cannot show box drawing.

**Acceptance Scenarios**:

1. **Given** a catalog holding the ASCII table and no other, **When** a box is rendered, **Then**
   its corners and junctions are `+`, its horizontal runs `-` and its vertical runs `|`.
2. **Given** that same catalog, **When** any diagram is rendered, **Then** no character of the
   output lies outside printable ASCII.
3. **Given** a diagram whose shapes name a stroke the ASCII table does not cover, **When** it is
   rendered against that catalog, **Then** the result is whatever a key no table answers already
   produces, and it is not an error.

---

### User Story 3 - The shipped demonstration shows both at once (Priority: P3)

Somebody clones the repository and runs the command-line application with no arguments. The one
diagram it prints has shapes drawn in the shipped table and shapes drawn in the table that comes
from the other library, side by side on the same canvas, produced by one catalog built from both.

**Why this priority**: it is the evidence a reader can see without running a test, and the
constitution requires the no-argument run to keep producing output. It is last because stories 1 and
2 are complete without it.

**Independent Test**: run the application with no arguments and check the printed diagram contains
both box-drawing characters and ASCII ones. Delivers the demonstration, updated by editing one text
file.

**Acceptance Scenarios**:

1. **Given** no command-line arguments, **When** the application is run, **Then** it prints one
   diagram containing shapes drawn with the shipped table and shapes drawn with the outside one.
2. **Given** the shipped description file, **When** a shape's stroke is changed from one table's
   stroke to the other's and the application is re-run, **Then** that shape's characters change and
   no other shape's do, with no code changed.
3. **Given** the shapes that were already in the shipped description, **When** the application is
   run after this feature, **Then** those shapes render exactly as they did before it.

---

### Edge Cases

- **A cell where a stroke from one table meets a stroke from another.** The key names both stroke
  names, and no single-stroke table holds it. The renderer's existing answer for a key nothing holds
  stands; this is not an error, and nothing in this feature changes it. The rule the model describes
  for this case under _Rendering_ — degrading to the cell's base stroke — is not implemented today
  and is not implemented here.
- **A catalog built from no tables at all.** Every key misses, every cell renders as the miss
  already renders, and this is not an error.
- **A table with no rows.** Contributes nothing and changes nothing about the catalog it goes into.
- **Two tables that claim the same key.** Settled by the order they went in, which is the rule the
  model already states under _Strokes, glyph sets and the catalog_.
- **The same table put into one catalog twice.** The second copy claims nothing, because every key
  it holds is already claimed.
- **A description naming a stroke no table in the catalog covers.** Unchanged from today: the cells
  are stamped and render to whatever a missing key already produces.

## Requirements _(mandatory)_

### Functional Requirements

#### The extension point

- **FR-001**: A glyph table defined in a library that depends on `monospace-core` MUST be usable to
  build a catalog, using only what `monospace-core` makes public.
- **FR-002**: A catalog MUST be buildable from more than one table, in an order its builder states,
  resolving contested keys by the rule `docs/model.md` already gives under _Strokes, glyph sets and
  the catalog_.
- **FR-003**: Nothing in the public interface MUST be able to report which table answered a key,
  preserving what [ADR-0014](../../docs/decisions/0014-collapse-glyph-sets-into-a-catalog.md)
  decided.
- **FR-004**: A key no table in a catalog holds MUST keep exactly the behavior it has today, and
  MUST NOT be an error.
- **FR-005**: Rendering a diagram against a catalog built from one table MUST produce what rendering
  it against that table alone produces today, so this feature changes no existing output.

#### Where tables live

- **FR-006**: `monospace-core` MUST keep the Light table and MUST NOT gain a second one.
- **FR-007**: Every other table the project ships MUST live in a second library that depends on
  `monospace-core`, as decided in
  [ADR-0036](../../docs/decisions/0036-hold-every-table-but-light-outside-the-core.md).
- **FR-008**: `monospace-core` MUST NOT depend on that library, directly or transitively, and MUST
  contain no reference to any table that library holds.
- **FR-009**: That library MUST reach `monospace-core` through its public interface only, so that it
  is exactly as privileged as a library written by somebody outside this project.
- **FR-010**: That library MUST be as portable as `monospace-core`: it MUST build for the
  WebAssembly target, and `cargo xtask check` MUST check that it does, so the boundary principle VII
  asks for is enforced by the gate rather than by prose.

#### The ASCII table

- **FR-011**: The second library MUST hold the ASCII table, and it MUST hold exactly the rows
  `docs/glyph-sets.md` records under _ASCII_ — no row added, none omitted, none altered.
- **FR-012**: A catalog built from the ASCII table alone MUST answer every one of those rows, so a
  diagram drawn entirely in that stroke has no missing cell.
- **FR-013**: This feature MUST bring the ASCII table and no other. Double, Heavy, Light Round and
  the mixing sets stay reference data in `docs/glyph-sets.md`.

#### The demonstration

- **FR-014**: The command-line application MUST build the catalog it renders with out of the Light
  table from `monospace-core` and the ASCII table from the second library, in an order it states,
  rather than using a catalog built from one table.
- **FR-015**: The demonstration description that ships with the application MUST gain shapes drawn
  with the ASCII stroke, on the single canvas it already draws, alongside the shapes it already has,
  so that a no-argument run shows both tables answering from one catalog.
- **FR-016**: Those shapes MUST be added to the shipped description file and nowhere else, keeping
  feature 045's rule that the file is the only place the demonstration's content lives.
- **FR-017**: The demonstration MUST NOT place a cell where an ASCII stroke meets a Light stroke,
  because no table in its catalog holds that key and the cell would render as a hole in a diagram
  whose purpose is to be looked at.
- **FR-018**: The shapes already in the shipped description MUST render exactly as they do today.
- **FR-019**: The application MUST NOT gain a command-line way to choose a table or a style in this
  feature. Which tables go into its catalog is stated in the application, and which stroke a shape
  is drawn with is stated per shape in the description, as it already is.

#### The documents

- **FR-020**: `docs/model.md` describes the sets that ship as built into "the library", singular.
  Because a second library now ships sets, that section MUST be amended to match before the code
  relies on it, as the constitution's _The model owns the design_ requires.
- **FR-021**: `docs/glyph-sets.md` opens by saying nothing loads its tables yet. That sentence MUST
  be corrected to say which of its tables the project now carries as data and which remain reference
  only.

### Key Entities

- **Glyph table**: a group of rules as somebody writes them — one of the tables in
  `docs/glyph-sets.md`. It is data, and it is what a library contributes.
- **Catalog**: what a renderer answers keys from, built from tables in an order. Already owned by
  the model and by ADR-0014; this feature gives it a second source, not a new meaning.
- **The glyph-table library**: the second library. It depends on `monospace-core`, holds the tables
  that are not Light, and has no privilege the core does not give every dependent. It is the
  stand-in for a library somebody else would write, and the evidence that writing one is possible.
- **Demonstration description**: the shipped description file the application renders when given no
  arguments. Unchanged in shape by this feature; it gains shapes drawn with a second stroke.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: A table that lives outside `monospace-core` answers a key in a rendered diagram, and
  searching `monospace-core`'s sources for that table's stroke name or its characters finds nothing.
- **SC-002**: A further table can be added without touching `monospace-core` at all — demonstrated
  by a test that defines its own table, renders with it, and would still pass if `monospace-core`
  were read-only.
- **SC-003**: Running the application with no arguments prints one diagram that contains at least
  one box-drawing character and at least one of `+`, `-` and `|` drawn as a stroke, so both tables
  are visibly answering from one catalog.
- **SC-004**: A diagram rendered against a catalog holding ASCII alone contains no character outside
  printable ASCII — checked over every character of the output, not sampled.
- **SC-005**: The ASCII table answers all fifteen non-empty combinations of its own stroke: the
  count of rows it holds is fifteen, and every one of them is reachable by rendering.
- **SC-006**: When two tables claim one key, the first one in answers it, and reversing the order
  reverses the answer — measured by rendering the same cell against both catalogs and comparing.
- **SC-007**: The number of public operations that report where a rule came from is zero.
- **SC-008**: Every diagram the project rendered before this feature renders byte-identically after
  it, except the shipped demonstration, which gains shapes and nothing else.

## Assumptions

- **The second library's name follows the issue's sketch.** The issue calls it `styles`, which the
  workspace's convention makes `monospace-styles`. This is recorded as an assumption rather than a
  requirement because the word is new: the model's vocabulary has _glyph set_ and no notion of a
  style, and ADR-0014's own driver was that two words for one thing is a vocabulary to keep true in
  two places. Confirming the name, or choosing one built on the model's word, is the maintainer's
  call and does not block the plan.
- **How a table is expressed and handed to a catalog is a plan decision.** This spec requires that a
  table from outside can be contributed and that order decides contested keys; the shape of the
  interface that does it — a type, a constructor, a builder — is not taken here, because a spec must
  not take a decision.
- **Degradation stays unimplemented.** `docs/model.md` describes falling back to a cell's base
  stroke when no rule matches, and nothing implements it. This feature makes mixed-stroke cells
  easier to reach, which is worth noting, but implementing that fallback is its own slice.
- **The demonstration's canvas may grow.** Fitting ASCII shapes beside the existing ones may need a
  wider or taller canvas; that is a value in the shipped file, not a behavior change.
- **No new dependency.** Nothing in this feature needs a crate the workspace does not already have.
- **A scope question is raised and not answered here.** The constitution's _In scope for this phase_
  names two crates, `monospace-core` and `monospace-cli`, and this feature makes three. The new
  crate is on none of the _Out of scope_ list, and it respects principle VII's boundary rather than
  crossing it, so this spec proceeds. Whether the scope section needs an amendment is the
  maintainer's decision, and the plan stage's Constitution Check is where it has to be settled.
