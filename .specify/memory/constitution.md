<!--
Sync Impact Report — 2026-10-04 | 2.0.1 → 3.0.0

MAJOR. Principle I stops mandating the volume of the process rather than its discipline, principle
II no longer ties a commit to a checklist, and the eleven-section workflow is replaced by one
question and one place the answer goes. Nothing that the gate enforces changed.

Deliberately unchanged: every heading another document cites by name. Twelve anchors are linked
from `docs/`, the crates, `AGENTS.md` and `CONTRIBUTING.md`, and no check in the gate resolves a
link, so renaming one would break it silently. The four anchors that no longer exist are fixed at
their seven call sites.

Earlier reports: docs/decisions/constitution-history.md. Open follow-ups are `wish` issues, not a
list here.
-->

# Monospace Constitution

The rules below exist to make the next change cheaper to make correctly. One that costs more than
the mistake it prevents is one to delete, and this file is where it gets deleted.

## Core Principles

### I. Process over product (NON-NEGOTIABLE)

This repository exists to build proficiency in Spec-Driven Development and in idiomatic Rust
architecture. ASCII diagramming is the vehicle, not the goal.

- When the fastest route to a feature conflicts with the route that teaches better Rust design, the
  second MUST be taken.
- A record MUST be written when a decision is taken and MUST NOT be written before one is. Nothing
  here obliges anyone to produce an artifact they have nothing to say in.

Rationale: the time is the point, so the rules that cost time have to earn it. They earn it by being
read, not by being thorough — the repository reached 1.7 lines of process prose per line of Rust and
the maintainer reports not reading it, and an unread rule is worse than a missing one because it
carries authority nobody granted it.

### II. Demonstrable increments

- Every commit that lands MUST leave the workspace building, all validations green, and
  `cargo run -p monospace-cli` producing output. A bare `cargo run` is ambiguous once the workspace
  holds more than one binary and MUST NOT be used as the acceptance command.
- Work MUST be sliced thin. Several small changes are preferred over one large one.
- A commit MUST leave `cargo xtask check` green, and `--no-verify` is forbidden. That is one commit
  where a change reaches green alone, and a small group where it does not.
- An increment — a slice that reaches a demonstrable state, usually several commits — MUST end with
  an appended entry in `docs/learning-log.md`: what was learned about Rust design, what was learned
  about working this way, and the evidence — what was tried and what it turned out to be. Three
  lines is the ordinary length; a section is not owed.

Rationale: a slice that cannot be shown to someone cannot be judged, and unjudged work accumulates
silently.

### III. One definition of green (NON-NEGOTIABLE)

- `cargo xtask check` is the only definition of green. The pre-commit hook runs it, CI runs it, and
  a new check MUST be added to that entrypoint rather than to either of them.
- Adding a check is two commits: first fix what it finds, then enable it. If it finds nothing, say
  so — a fix MUST NOT be manufactured to fill the commit.
- `git commit --no-verify` MUST NOT be used. When the hook fails, fix the cause or stop and report.
- `git rebase` and `git cherry-pick` do not fire the hook. After rewriting history, the gate MUST be
  run on each rewritten commit, not only on the tip.
- Gate configuration is kept minimal by removal: an entry MUST be shown to break something when it
  is taken out, or it is removed. A check whose subject no longer exists is removed rather than left
  passing over a frozen directory.
- A step passes or fails on its exit code alone. `rustfmt` reports that `group_imports` needs
  nightly and exits 0, so a green run is evidence the command ran and not that it agreed — read the
  output.

Rationale: two definitions of green is the failure this arrangement exists to prevent. Passing
locally and passing in CI have to mean the same thing.

### IV. Claims are measured, not assumed

- Behavior MUST NOT be asserted before it has been observed, and least of all in a pull request body
  or a commit message. Run it first, then write down what happened.
- A new check MUST be verified by making it fail on purpose and then restoring. A green run only
  proves the command ran.
- "Is X faster, smaller, better?" MUST be answered with a measurement: a median over several runs,
  and an explicit statement when the noise is larger than the difference.
- Before the gate is trusted, it MUST be run on a fresh clone. Files written by hand skip the
  transformations Git applies on checkout, so a working copy can be green while the repository is
  broken.
- A requirement accepted with nothing to verify it MUST be named as such where it is recorded,
  rather than described as tested.
- What was tried and abandoned MUST be reported when it would otherwise look untried, because the
  cost of the abandoned option is what makes the chosen one legible.

#### Show the rendering

Where the subject of an option, a question, a decision or a change is something this project
renders, the artifact MUST show the rendering; prose accompanies the picture rather than replacing
it. This holds for a decision, a pull request body, and a question put to the maintainer in a
session.

Every picture in a tracked file MUST be unambiguously one of two things:

- **Generated** — produced from a description the file carries, regenerated by `cargo xtask render`,
  and re-checked by `cargo xtask check`.
- **Hypothetical** — hand-drawn, showing what no code produces yet, and labelled on the spot.

Where an alternative is cheap to implement, a spike that generates its picture is preferred to an
argument about what it would look like. Where the subject does not render, no picture is required
and one added anyway is noise.

Rationale: a picture is believed faster than a sentence, so an unlabelled hand-drawn one is the
strongest form of the thing the bullets above forbid — and a generated one is the cheapest artifact
this project can produce, because the output is monospaced text and fits inside every document that
would otherwise describe it.

### V. Structural and behavioral change never share a commit

- Kent Beck's rule holds: make the change easy — this may be hard — then make the easy change.
- A structural commit MUST NOT change behavior. Existing tests pass unchanged, and no test is added
  or modified.
- The Conventional Commits type MUST carry the distinction: `refactor` for structural, `feat` and
  `fix` for behavioral. They MUST NOT be mixed.
- Large refactors MUST go expand/contract: add alongside, migrate, then remove, each step green and
  committed separately.
- If the preparatory refactor turns out to be hard, work MUST stop and the maintainer be told before
  it starts. That is a design signal to discuss, not something to push through.

Rationale: the prefix makes the distinction visible in the log without reading diffs, and a mixed
commit is unreviewable in both halves. This governs how a refactor is committed and is never a
reason to avoid one — see _Understanding changes_ below.

### VI. Decisions recorded at the altitude they belong to

A decision MUST be recorded when it is taken; if code is about to depend on an unrecorded decision,
the record comes first. What "recorded" means depends on one thing: the altitude.

**Altitude.** A decision is **domain-level** when something outside the module implementing it can
observe it — another crate, a caller, a second implementation of the same idea. Its home is
`docs/model.md`, or `docs/diagram-model.md` for the layer above. A decision is **module-level** when
nothing outside can observe it beyond the output it produces. Its home is rustdoc on that module,
under `Design notes`. That is the default home, not the fallback.

The test: _if this changes, must anything outside this module change with it?_ Where the answer is
no, and where it is genuinely unclear, the decision is module-level. Promoting one later costs
moving a paragraph; demoting one later costs undoing the authority a published document already gave
it.

**Subject.** A record is sized to its subject, not to the moment it was taken. Where a subject
already has a record, a change to it is a revision of that record and MUST NOT become a second
record beside the first. The test: **if a record cannot be cited without citing another, the two are
one record.**

**Understanding changes.** When understanding of the domain changes, the code and the documents
describing it are rewritten to reflect the new understanding. A record of the previous understanding
is not a constraint on the new one: it is the context explaining why the code was as it was.
Rewriting needs no permission; deleting the record does not follow from it. A decision MUST NOT be
defended by arguing that changing it repeatedly is expensive — where that is true the cost is paid
by a test, a migration or a public API and is written there, not where it reads as an instruction to
stop thinking.

**What needs a record at all.** It is expensive to undo, or someone will later ask "why is this like
this?". If neither applies it belongs in the learning log.

Rationale: a rationale with three possible homes has none, and this repository had four — a decision
sheet, a research file, an ADR and a model document — which is how five accepted records ended up on
one private function and a picture nobody could change without reconciling all five.

### VII. The core stays portable

- `crates/monospace-core` holds all domain logic. `crates/monospace-cli` holds none.
- The core's public API MUST NOT assume a CLI, a TUI or a terminal. Later phases compile it to
  WebAssembly, and the gate's `wasm` step enforces that boundary rather than prose. It compiles
  three crates — `monospace-core`, `monospace-diagram` and `monospace-glyph-sets` — and that is what
  keeps the core free of terminal assumptions.
- `monospace` is reserved for the interactive TUI of a later phase and MUST NOT be taken by another
  binary.
- Public library APIs MUST be documented with rustdoc as they are introduced, not retrofitted.

Rationale: a boundary that exists only in a document erodes; one the compiler refuses to cross does
not.

### VIII. The record is sized to the decision

- An artifact MUST NOT restate what another already says; it cites it by name. A paragraph that
  would change nobody's decision if deleted is deleted.
- A change carries one record: the pull request body. It has one ceiling, and exceeding it is not a
  violation to justify — it is the signal that the change is too thick, and the first answer is to
  split it rather than to compress the prose.
- A generated picture and the description it comes from do not count against a ceiling. They replace
  prose rather than adding to it, and charging for them would push an artifact back toward
  describing what it could have shown.

Rationale: charging for prose is what produced a `tasks.md` of 760 lines for a change of 450, and
the fix is not a smaller checklist but no checklist.

## Constraints and Dependencies

### Language of the artifacts

Code identifiers, comments, doc comments, README, `docs/`, commit messages, pull request
descriptions, CLI output, error messages and help text are all in English. Conversation with the
maintainer may be in any language; what lands in the repository is English.

### Toolchain

Rust, edition 2024, pinned to an exact version in `rust-toolchain.toml` with its components and the
`wasm32-unknown-unknown` target. No nightly-only features. No MSRV is declared while nothing outside
this repository depends on these crates. Node, at the version in `.nvmrc`, backs the checks Rust
cannot perform.

### Dependencies

A dependency MUST NOT be added without asking the maintainer first. Before adding or pinning any
version, its publication date MUST be verified to be at least seven days old, and the version and
that date reported. Domain logic prefers the standard library; infrastructure concerns prefer
idiomatic, well-established crates over reinvention.

### Testing

Each change defines its own testing expectations, and unit tests for core logic are the minimum any
may ask for. A behavior rule with no test named against it is an unfinished change.

Tests come in two kinds, kept apart in separate directories where they are snapshots:

- A **contract test** pins a decision, and its name or comment says which rule of the model it
  holds. Changing one is changing the decision, so it belongs in the decision table of the pull
  request that changed it.
- A **characterization test** records what the code does over a range too wide to assert by hand. A
  change to it is the consequence of a decision taken elsewhere, not a decision. The file says so at
  its head, and it MUST cover its range whole rather than by sample.

A characterization MUST NOT be described as reviewed. What is required instead, each time one moves,
is a report of **how many cases moved, in which families, and three representative examples with
their before and after**; that report is what is read.

Rationale: the safety net is not that output stays fixed but that a change to it is seen and judged.
A claim of review nobody can keep turns the first into the second.

### In scope for this phase

Four crates, and their boundary is principle VII: `monospace-core`, the library holding the
diagramming logic — input parsing, layout and positioning, ASCII rendering; `monospace-cli`, a
minimal non-interactive consumer producing diagrams from the terminal; `monospace-glyph-sets`, the
glyph sets the core does not ship, depending on the core and privileged no further than any other
crate that does; and `monospace-diagram`, the model holding a diagram after it is drawn — shapes
with an identity, an order and positions that may depend on each other — on the same terms.

Each of the last two is here for a reason of its own, and a further crate proposed for any other
reason widens this section and is renegotiated rather than assumed.

### Out of scope for this phase

A change proposing any of these MUST be stopped and renegotiated rather than quietly widened:
WebAssembly bindings, a web front-end, non-terminal GUIs, persistence, collaboration, and export
formats beyond plain text. Where each is expected to arrive instead is the `Phase` field of its
capability issue in the GitHub Project, which holds direction and no rules: nothing on the board
widens this section.

## Development Workflow

### One question, and where the answer goes

Every change starts from a GitHub issue and ends in a pull request, and the first thing settled is
one question: **is there more than one defensible answer to how this should be done?**

| Answer                                                                        | What happens                                                                                 |
| ----------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------- |
| No — the model says what to do, and the code disagrees with it                | One branch, one pull request, `Closes #N`. No record beyond the code.                        |
| No — the model is silent, or already agrees                                   | One branch, one pull request, `Closes #N`. No record beyond the code.                        |
| Yes — a sentence of the model becomes false, or a caller observes a new shape | Two branches: `NNN-slug-deciding` with `Refs #N`, then `NNN-slug-building` with `Closes #N`. |

The two `No` rows differ in nothing an artifact would record, which is why they share one. They are
the same shape of change and separating them would be ceremony.

The deciding pull request is where the answer is agreed, and it is a pull request body: the decision
table and _What proves it_. No code opens in it. The building pull request writes the code and the
tests, and amends the model where building showed it wrong.

**The record is the pull request body.** It is where a reviewer looks, GitHub versions it, and it
cannot drift from the diff because it is written from the same change. A change that agrees nothing
produces no record, which is the correct outcome rather than a missing one.

### What a change does not have to produce

Named so that leaving them out is not a decision to be made again each time.

- **No spec directory.** `specs/NNN-slug/` and its eight files per feature are gone. The 19 that
  exist are frozen history, and nothing adds to them.
- **No plan, no tasks, no research, no data model, no contracts, no quickstart, no checklist.**
  These were seven ways to write the same decision down before it was taken, and the sum of them was
  larger than the change.
- **No decision sheet as a separate file.** The decision table lives in the pull request body.
- **No architecture decision record.** `docs/decisions/` is frozen history; a decision that outlives
  its change goes to `docs/model.md`, and one nothing outside observes goes to rustdoc.
- **No stage label.** Where the work stands is visible on the branch and the open pull request. The
  `wish` label stays because it is the board's inbox.

### No code before the decision is agreed

A decision is agreed by the maintainer, not assumed. On the two `No` rows there is no decision to
agree, so nothing blocks the code. On the `Yes` row the deciding pull request merges first.

Where building meets a decision the body does not hold, work MUST stop and ask. It MUST NOT be
resolved and recorded afterwards, or the table records what was easy to foresee rather than what was
decided.

### Drawing the case

The cheapest step in the flow and the one that removes the most rework: before settling a decision,
draw the case it is about and say what the drawing says that the prose did not.

Four carriers exist and none of them is new machinery: a `<!-- render: -->` marker in a tracked
document, a gallery block that moves on its own when the picture moves, a marker in a pull request
body, and a test that prints. A case that cannot be drawn yet is a result, not a failure — say so in
one line and draw the nearest thing that does exist.

### What happens when the model is wrong

The model moves in the pull request that found it wrong, in the same commit, with the sentence
quoted and replaced. `docs/model.md` is design intent and owns the domain vocabulary; a change names
the sections it implements and does not restate them. If a slice needs a rule the model does not
have, the model changes first.

### Whose decision it is

When a decision belongs to the maintainer — cost, scope, taste, or a risk they carry — the real
options MUST be presented with their trade-offs, what is reversible and what is not, a
recommendation with a confidence level, and what information would change it. Where it is settled
practice and the maintainer has no stake, choose the conventional answer, say what was chosen and
why, and carry on. Every domain-level decision belongs to the maintainer.

### Fixing a commit

A correction that changed nothing for anyone — a typo, a wrong status line, a formatting slip —
belongs in the commit that got it wrong. When something was actually wrong, in behavior or in a
claim someone could have acted on, the fix is its own commit and says what was wrong. This holds
after pushing: `main` is never rewritten, and every force push uses `--force-with-lease`.

### Cross-references

Cite a section of another document by its name, not by its number.

## Governance

This constitution supersedes prior practice wherever the two conflict. It owns the principles, the
scope and the constraints. [`docs/model.md`](../docs/model.md) owns the domain's design and its open
questions; the GitHub Project owns direction and the backlog; `CONTRIBUTING.md` owns how to run the
tooling; `AGENTS.md` owns session guidance for every harness, and `CLAUDE.md` is an import of it
rather than a second copy. Where any of them restates a rule stated here, this file is the one to
follow and the duplication is a defect to remove.

### Amendment procedure

An amendment is made in the same increment as the decision that requires it, and the commit says so
— never drifted from silently. Every amendment updates the version line and replaces the Sync Impact
Report at the top of this file, moving the previous one to `docs/decisions/constitution-history.md`.
The report says what changed and names where the reasoning lives; it does not repeat it.

### Versioning policy

MAJOR for a backward-incompatible removal or redefinition of a principle, MINOR for a new principle
or materially expanded guidance, PATCH for clarifications and wording.

### Compliance review

The mechanical parts — formatting, lints, spelling, the WebAssembly boundary, tests, docs, rendered
pictures — are enforced by `cargo xtask check` and are not a matter of review. The rest — thin
slices, structural commits kept separate, decisions recorded at the right altitude, claims measured,
a decision agreed before code — is checked by reading, at review time.

**Version**: 3.0.0 | **Ratified**: 2026-09-08 | **Last Amended**: 2026-10-04
