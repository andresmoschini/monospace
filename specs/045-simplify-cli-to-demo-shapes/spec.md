# Feature Specification: Simplify CLI to demo shapes

**Feature Branch**: `045-simplify-cli-to-demo-shapes-spec`

**Created**: 2026-09-11

**Status**: Draft

**Input**: User description: "This is a quick feature, in the future there will be a better storage
model" — on top of [issue #45](https://github.com/andresmoschini/monospace/issues/45): read a JSON
file with an array of serialized shapes and render them in place of the hardcoded shapes, and ship a
JSON file used by default that demonstrates the current capabilities.

## User Scenarios & Testing _(mandatory)_

### User Story 1 - Render a diagram described in a file (Priority: P1)

Someone wants to see a particular arrangement of boxes, lines and arrows without touching Rust. They
write the arrangement in a text file, point the command-line application at it, and the rendered
ASCII diagram is printed.

**Why this priority**: it is the whole point of the feature. Today the only way to change what the
demo draws is to edit `main.rs` and recompile; every capability the core grows is invisible until
someone writes code to exercise it. This story removes that step and is on its own a complete,
demonstrable slice.

**Independent Test**: write a file describing a single box, run the application against it, and
compare the printed output to the same box as the previous hardcoded demo drew it. Delivers the
ability to draw an arbitrary diagram without recompiling.

**Acceptance Scenarios**:

1. **Given** a file describing one box, **When** the application is run against that file, **Then**
   it prints exactly that box and nothing else.
2. **Given** a file describing a box, a line and an arrow at stated positions, **When** the
   application is run against it, **Then** it prints all three composed into one diagram, each in
   the position the file states.
3. **Given** a file whose shapes are reordered but otherwise identical, **When** the application is
   run against it, **Then** the shapes are drawn in the file's order, so the result reflects that
   order wherever two shapes overlap.
4. **Given** the same file run twice, **When** the outputs are compared, **Then** they are
   identical.

---

### User Story 2 - See the current capabilities with no arguments (Priority: P2)

Someone who has just cloned the repository runs the application with no arguments and sees a diagram
that exercises what the core can currently draw — the three shapes, strokes, a filled interior, and
both stamp modes — rather than a bare box.

**Why this priority**: it preserves what the hardcoded demo was for, and the constitution requires
that `cargo run -p monospace-cli` produce output on every commit. Valuable, but only once story 1
can render a file at all.

**Independent Test**: run the application with no arguments and check that the printed diagram shows
each capability named above. Delivers a living demonstration that is updated by editing a data file.

**Acceptance Scenarios**:

1. **Given** no command-line arguments, **When** the application is run, **Then** it prints the
   demonstration diagram and exits successfully.
2. **Given** no command-line arguments, **When** the application is run from a working directory
   other than the repository root, **Then** it prints the same diagram.
3. **Given** the demonstration file is passed explicitly by path, **When** the application is run,
   **Then** the output is identical to running with no arguments.

---

### User Story 3 - Be told plainly when a file cannot be drawn (Priority: P3)

Someone hand-writes a description file and gets something wrong — the path is misspelled, a brace is
missing, a shape names a field that does not exist. They are told what is wrong instead of seeing a
panic or a blank diagram.

**Why this priority**: hand-written input makes mistakes routine, and a message is what makes the
format learnable without documentation. It is last because stories 1 and 2 are demonstrable without
it.

**Independent Test**: run the application against a path that does not exist, and against a file
holding malformed text, and check each prints a message naming the problem and exits with a failure
status.

**Acceptance Scenarios**:

1. **Given** a path that does not exist, **When** the application is run against it, **Then** it
   prints a message naming that path on the error stream, prints nothing on the output stream, and
   exits with a failure status.
2. **Given** a file that is not well-formed, **When** the application is run against it, **Then** it
   prints a message saying where the text stopped making sense, and exits with a failure status.
3. **Given** a file that is well-formed but describes a shape kind that does not exist, **When** the
   application is run against it, **Then** it prints a message naming the unrecognized kind, and
   exits with a failure status.

---

### Edge Cases

- **An empty list of shapes.** The file is valid and describes nothing: the application prints an
  empty canvas of the stated size and exits successfully.
- **A shape entirely outside the canvas.** It is described at a position no part of which falls
  within the canvas. Nothing of it appears, the rest of the diagram is unaffected, and this is not
  an error — clipping is already how the buffer behaves.
- **A shape partly outside the canvas.** The part inside is drawn; the part outside is not.
- **A degenerate shape**, such as a box smaller than two cells on a side or a line of length zero.
  The existing shape rules already decide these; the file cannot make them behave differently, and
  they are not errors.
- **A stroke name no glyph set defines.** The cells are stamped with that stroke and render to
  whatever the catalog's miss behavior already produces, exactly as when the same stroke is named
  from Rust.
- **More than one command-line argument.** The application prints a usage message and exits with a
  failure status rather than guessing which argument was meant.
- **A file whose canvas size is zero** in either dimension: valid, and prints nothing.

## Requirements _(mandatory)_

### Functional Requirements

- **FR-001**: The application MUST accept an optional single command-line argument naming the path
  of a diagram description file to render.
- **FR-002**: When given a path, the application MUST render the diagram that file describes and
  print it to the output stream.
- **FR-003**: When given no argument, the application MUST render a demonstration description that
  ships with the project, producing the same result as passing that description's path explicitly.
- **FR-004**: The demonstration description MUST be readable and editable as text by someone who has
  not read the source code, and editing it MUST change what the application prints without any code
  change.
- **FR-005**: A description MUST state the canvas it is drawn on — its origin and its size — and the
  ordered list of shapes drawn on it.
- **FR-006**: A description MUST be able to express each shape the core currently offers: a box, a
  line and an arrow.
- **FR-007**: A description MUST be able to express, for each shape, every parameter that shape
  currently exposes to a Rust caller, so that anything drawable from code can also be drawn from a
  file.
- **FR-008**: A description MUST be able to state, per shape, which of the two stamp modes it is
  drawn with, since choosing between them is a current capability the demonstration has to show.
- **FR-009**: Shapes MUST be drawn in the order the description lists them, so that the
  description's order is what decides overlap together with each shape's stamp mode.
- **FR-010**: The demonstration description MUST exercise all three shapes, a filled box interior,
  and both stamp modes. Because it is a single canvas (FR-021), the two stamp modes MUST be shown by
  placing two overlapping arrangements side by side on that one canvas rather than by rendering the
  same arrangement twice under labels, which is how the previous hardcoded demo showed them.
- **FR-011**: The application MUST NOT write any file. It reads a description and prints a diagram;
  saving a diagram is out of scope for this phase.
- **FR-012**: When the named path cannot be read, the application MUST print a message naming the
  path to the error stream and exit with a failure status.
- **FR-013**: When the file's text is not well-formed, the application MUST print a message locating
  the problem to the error stream and exit with a failure status.
- **FR-014**: When the file is well-formed but does not describe a valid diagram — an unknown shape
  kind, a missing required field, a value of the wrong type — the application MUST print a message
  naming the offending part and exit with a failure status.
- **FR-015**: On any failure, the application MUST print nothing to the output stream, so that a
  failed run is never mistaken for a diagram.
- **FR-016**: The application MUST NOT panic on any input file, well-formed or not.
- **FR-017**: Rendering a given description MUST be deterministic: the same file always produces the
  same output.
- **FR-018**: The hardcoded shapes in the command-line application MUST be removed, leaving the
  application with no diagram of its own baked into code.
- **FR-019**: The description format MUST live in the command-line application. `monospace-core`'s
  public API MUST NOT gain any way to read, write or represent a description in this feature, and
  `docs/model.md` MUST NOT be amended by it: the model's open question _What minimal diagram
  description does the core accept?_ stays open, and the record of this feature MUST say so
  explicitly, so that a later reader does not mistake this format for the answer to it.
- **FR-020**: The description format is provisional. No compatibility is promised across versions,
  and a later storage model is expected to replace it. This MUST be stated wherever the format is
  documented, not only in this spec.
- **FR-021**: A description MUST describe exactly one canvas, and the application MUST print exactly
  one diagram and no surrounding text — no labels, no headings, no blank separator sections.
- **FR-022**: The demonstration description MUST travel inside the binary, so that the no-argument
  run of FR-003 produces its diagram from any working directory and from a binary invoked outside a
  checkout of this repository.
- **FR-023**: The demonstration description MUST exist as a single editable text file in the
  repository, and that file MUST be the only source of what the binary carries: the demonstration
  MUST NOT be duplicated anywhere in code.

### Key Entities

- **Diagram description**: what one file holds. A canvas and an ordered list of shape descriptions.
  It is data, not a program: it names positions and sizes, and holds no conditionals, variables or
  references between shapes.
- **Canvas**: the window the diagram is drawn on and rendered from — an origin and a size, the same
  pair the buffer already takes. Everything outside it is clipped away.
- **Shape description**: one entry in the list. Names which of the three shapes it is, carries that
  shape's own parameters, and states the stamp mode it is drawn with.
- **Demonstration description**: the diagram description that ships with the project and is used
  when no path is given. It is the demo, and the only place the demo's content lives.

## Success Criteria _(mandatory)_

### Measurable Outcomes

- **SC-001**: Adding a fourth box to what the application prints takes editing one text file and
  re-running, with zero lines of code changed. The run rebuilds, because the demonstration travels
  inside the binary (FR-022); what the criterion measures is that no code was touched, not that no
  compiler ran.
- **SC-002**: Someone who has never read the source can write a description that renders a box, a
  line and an arrow, working only from the shipped demonstration file and the failure messages.
- **SC-003**: Every parameter the three shapes expose to a Rust caller is reachable from a
  description — counted, and the count matches.
- **SC-004**: Every capability the previous hardcoded demo showed — all three shapes, a filled
  interior, overlap, and both stamp modes — is still visible in the shipped demonstration, now on
  one canvas instead of three labelled renderings.
- **SC-005**: No input file makes the application panic, exit successfully with no diagram, or print
  a partial diagram alongside an error.
- **SC-006**: The command-line application contains no description of a diagram in its code: the
  diagram it draws by default lives entirely in data.

## Assumptions

- **The file format is JSON**, as issue #45 states. It is the conventional structured text format
  for this, and no alternative was weighed.
- **Reading is not persistence.** The constitution puts persistence out of scope for this phase and
  input parsing in scope. This feature only reads; FR-011 states the boundary so the distinction is
  not left to a reader's judgement.
- **No new command-line surface beyond one optional path.** No flags, no subcommands, no options for
  the glyph set or the canvas — those come from the description or stay fixed. Growing the command
  line is a separate concern from reading a file.
- **The glyph set stays the light catalog** the application already uses. Choosing a set per
  description is a capability nothing has asked for yet.
- **The core's shapes do not change.** This feature adds no shape, no parameter and no rendering
  rule; it makes the existing ones reachable from a file.
- **Hand-written files are the normal case**, which is why story 3's messages matter more than they
  would for a machine-generated format.
- **The existing end-to-end test is replaced, not extended.** It asserts the exact text of the
  hardcoded demo, which FR-018 removes. Its replacement asserts the exact text the shipped
  demonstration renders to, which is the same kind of check against a different source.
- **`cargo run -p monospace-cli` with no arguments remains the acceptance command**, per the
  constitution's principle II, which is what makes FR-003 mandatory rather than a convenience.
- **The demonstration's output will not be byte-for-byte what it is today.** FR-021 drops the labels
  and the three separate renderings, so the diagram changes shape even though every capability it
  showed survives (SC-004). The demo is an illustration, not a contract.
- **The core is untouched.** FR-019 confines this feature to `monospace-cli`, so principle VII's
  WebAssembly boundary is not involved: nothing this feature adds has to compile to it.

## Dependencies

- **Feature 039** (draw shapes instead of individual cells), merged, which is what makes a shape a
  value that can be described rather than a sequence of stamps.
- **An ADR recording that this format does not answer the model's open question.** The constitution
  requires the record before code depends on the decision, and the decision here is as much about
  what is _not_ being settled as about what is: `docs/model.md` asks what minimal diagram
  description the core accepts and says the first slice reading a diagram from outside the process
  would settle it. This is that slice, and it declines. Without the record, the next reader has a
  JSON format in the repository and an open question in the model, and no way to tell that the two
  were considered together. The ADR MUST be written before `/speckit-plan`.
