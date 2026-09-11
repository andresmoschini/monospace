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

## Clarifications

### Session 2026-09-11

- Q: What should the second library be called? → A: `monospace-glyph-sets`, the model's own word,
  rather than the issue's `styles`
- Q: Should the constitution's _In scope for this phase_ be amended in this increment to admit a
  third crate? → A: Yes, amended here to 1.5.0, naming ADR-0036 as the decision it implements
- Q: What should the feature do about a cell where an ASCII figure and a Light one meet? → A:
  Nothing new — one stroke per cell (ADR-0012) means the front figure's base stroke answers the
  whole cell. The demonstration shows it both ways round, ASCII in front and Light in front

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
4. **Given** an ASCII figure crossing a Light one with the ASCII figure in front, **When** the
   application is run, **Then** the shared cell is the ASCII table's character for that junction.
5. **Given** the same crossing with the Light figure in front instead, **When** the application is
   run, **Then** the shared cell is the Light table's character for that junction, so the two
   crossings differ only by what is in front.

---

### Edge Cases

- **A cell two figures share, each drawn from a different table.** Already settled, and settled
  twice over. A cell holds one base stroke and four arms that all draw in it, per
  [ADR-0012](../../docs/decisions/0012-one-stroke-per-cell.md), so a key never names two stroke
  names; and where figures overlap, the one in front owns the base stroke, per _Stamping_ in
  `docs/model.md`. The shared cell therefore renders wholly from the front figure's table. This is
  the behavior the demonstration has to show rather than avoid, which is FR-017.
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
- **FR-007**: Every other table the project ships MUST live in a second library,
  `monospace-glyph-sets`, which depends on `monospace-core`, as decided in
  [ADR-0036](../../docs/decisions/0036-hold-every-table-but-light-outside-the-core.md). The name is
  the model's word for what the library holds, so the project gains no second word for a set.
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
- **FR-017**: The demonstration MUST show a figure drawn from each table crossing the other, twice:
  once with the ASCII figure in front and once with the Light figure in front. The two crossings
  MUST render from the front figure's table in each case — the first as ASCII, the second as Light —
  so the demonstration shows that which table answers a shared cell is decided by what is in front,
  and by nothing about where the table came from.
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
  `docs/glyph-sets.md`. It is data, and it is what a library contributes. A table and a glyph set
  are the same thing: the model's vocabulary names the entity `GlyphSet` and calls its written form
  a table, and this spec uses whichever of the two reads better in the sentence.
- **Catalog**: what a renderer answers keys from, built from tables in an order. Already owned by
  the model and by ADR-0014; this feature gives it a second source, not a new meaning.
- **`monospace-glyph-sets`**: the second library. It depends on `monospace-core`, holds the tables
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
- **SC-009**: The demonstration holds two crossings of a Light figure and an ASCII one, and the
  shared cell of each is read off the printed diagram: the one with ASCII in front is a character
  the ASCII table holds, and the one with Light in front is a character the Light table holds.

## Assumptions

- **How a table is expressed and handed to a catalog is a plan decision.** This spec requires that a
  table from outside can be contributed and that order decides contested keys; the shape of the
  interface that does it — a type, a constructor, a builder — is not taken here, because a spec must
  not take a decision.
- **Degradation is not needed and is not in scope.** `docs/model.md` describes, under _Rendering_,
  falling back to a cell's base stroke when no rule matches. That step is for a representation where
  an arm carries a stroke of its own, which ADR-0012 declined; with one stroke per cell, every `Set`
  arm already draws in the base stroke, so a key that mixes stroke names cannot be built and the
  fallback has nothing to catch. Two tables in one catalog do not change that, which is why FR-017
  can require the crossings rather than forbid them.
- **The demonstration's canvas may grow.** Fitting ASCII shapes beside the existing ones may need a
  wider or taller canvas; that is a value in the shipped file, not a behavior change.
- **No new dependency.** Nothing in this feature needs a crate the workspace does not already have.
- **The constitution already admits the third crate.** Its _In scope for this phase_ was amended to
  1.5.0 in this same increment, naming `monospace-glyph-sets` and ADR-0036 as the decision that
  required it, so the plan stage's Constitution Check has something to point at rather than a
  contradiction to argue around. The amendment admits this crate for this reason and no other: a
  further crate is still a widening to renegotiate.
