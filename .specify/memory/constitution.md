<!--
Sync Impact Report — 2026-09-21 | 1.6.0 → 2.0.0

MAJOR: principle VI is redefined in a way earlier practice does not satisfy. A decision now declares
the altitude it belongs to and the commitment it carries, one record covers one subject rather than
one moment, and a record at the two lower commitments is revised rather than superseded. Records
written before this version stay valid and are reclassified when next touched, not in a sweep.

Also: IV gains _Show the rendering_; VIII is new; Testing gains the contract/characterization split
and forbids calling a characterization reviewed; three feature stages become two, with the cut after
the decision sheet; _The model owns the design_ gains what it excludes.

Implements ADR-0050 (altitude and commitment), ADR-0051 (the decision sheet and two stages,
superseding ADR-0032), ADR-0052 (show the rendering) and ADR-0053 (a characterization is reported,
not reviewed, revising ADR-0045). Those records hold the reasoning; this report does not repeat it.

Templates replaced in the same increment: spec, plan, ADR, and decisions (new). Unchanged:
checklist, tasks.

Not yet built, and named here as though it were: `cargo xtask render` with the gate step beside it
(ADR-0052), and `cargo xtask spec`'s two stages (ADR-0051). Until each lands, the rule it belongs to
is enforced by review, and its ADR says so.

Earlier reports: docs/decisions/constitution-history.md. Open follow-ups are `wish` issues, not a
list here.
-->

# Monospace Constitution

## Core Principles

### I. Process over product (NON-NEGOTIABLE)

This repository exists to build proficiency in Spec-Driven Development and in idiomatic Rust
architecture. ASCII diagramming is the vehicle, not the goal.

- When the fastest route to a feature conflicts with the route that teaches better Rust design or
  better spec-driven practice, the second MUST be taken.
- A plan that saves effort by collapsing the process MUST be rejected. A record is never skipped; it
  is sized to the decision it holds, per principle VIII.

Rationale: every other principle costs time, and they are affordable only because the time is the
point — which is also why a record nobody reads costs the time and buys nothing.

### II. Demonstrable increments

- Every commit that lands MUST leave the workspace building, all validations green, and
  `cargo run -p monospace-cli` producing output. A bare `cargo run` is ambiguous once the workspace
  holds more than one binary and MUST NOT be used as the acceptance command.
- Specs MUST be sliced thin. Several small specs are preferred over one large one.
- A commit MUST tick exactly the checkboxes in `tasks.md` that it completed and leave the gate
  green. Since the hook runs the gate and `--no-verify` is forbidden, that is one commit per task
  where a task reaches green alone, and one per group where it does not.
- An increment — a slice that reaches a demonstrable state, usually several commits — MUST end with
  an appended entry in `docs/learning-log.md`: what was learned about Rust design and idiom, what
  was learned about working this way, and optionally a trade-off worth remembering. A lesson earns
  an entry only with evidence: what was tried, and what it turned out to be.

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
- Gate configuration is kept minimal by removal: an entry MUST be shown to break something when
  taken out, or it is removed. In a linter or a spell checker, an entry that changes nothing
  silently permits errors.

Rationale: two definitions of green is the failure this arrangement exists to prevent. Passing
locally and passing in CI have to mean the same thing.

### IV. Claims are measured, not assumed

- Behavior MUST NOT be asserted before it has been observed, and least of all in an ADR, a spec's
  acceptance list or a commit message. Run it first, then write down what happened.
- A new check MUST be verified by making it fail on purpose and then restoring. A green run only
  proves the command ran.
- "Is X faster, smaller, better?" MUST be answered with a measurement: a median over several runs,
  and an explicit statement when the noise is larger than the difference.
- Before the gate is trusted, it MUST be run on a fresh clone. Files written by hand skip the
  transformations Git applies on checkout, so a working copy can be green while the repository is
  broken.
- A requirement accepted with nothing to verify it MUST be named as such where it is recorded,
  rather than described as tested.

#### Show the rendering

Where the subject of an option, a question, a decision or a change is something this project
renders, the artifact MUST show the rendering; prose accompanies the picture rather than replacing
it. This holds for a decision sheet, an ADR's options and consequences, a spec's scenarios, a pull
request body, and a question put to the maintainer in a session.

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
the record comes first. What "recorded" means depends on three things the record declares.

**Altitude.** A decision is **domain-level** when something outside the module implementing it can
observe it — another crate, a caller, a second implementation of the same idea. Its home is
`docs/model.md`, or `docs/diagram-model.md` for the layer above, together with an ADR. A decision is
**module-level** when nothing outside can observe it beyond the output it produces. Its home is
rustdoc on that module, under `Design notes`; it takes no ADR, amends no model document, and
changing it is an ordinary `feat` or `fix`.

The test: _if this changes, must anything outside this module change with it?_ Where the answer is
no, and where it is genuinely unclear, the decision is module-level. Promoting later costs one ADR;
demoting later costs undoing the gravity the record already created. A module-level decision that
becomes observable is promoted in the increment that makes it so, not in advance. A recorded
decision whose subject stopped being observable moves its reasoning to that module's rustdoc and sets
its status to `absorbed into <path>`. The file stays where it is: this directory is a history.

**Subject.** A record is sized to its subject, not to the moment it was taken. Where a subject
already has a record, a change to it is a revision of that record and MUST NOT become a second
record beside the first. The test: **if a record cannot be cited without citing another, the two are
one record.**

**Commitment.** Every ADR MUST declare one of three, and it governs how the record is changed.

| | Means | Changed by |
| --- | --- | --- |
| `exploratory` | A hypothesis with a date; nothing depends on it yet. | Revising it in place, with a dated line saying what changed. |
| `working` | It holds, something depends on it, it is still under review. | A new, short ADR naming what it replaces. The options are not re-argued. |
| `load-bearing` | Something outside the module depends on it, or reversing it means migrating consumers or data. | A new ADR; this one becomes `superseded by ADR-NNNN`. |

`load-bearing` is only those two conditions. Not that the record was hard to write, took a long
argument, or that changing it would move a lot of output.

**Understanding changes.** When understanding of the domain changes, the code and the documents
describing it are rewritten to reflect the new understanding. A record of the previous understanding
is not a constraint on the new one: it is the context explaining why the code was as it was.
Rewriting needs no permission; deleting the record does. A record MUST NOT argue that repeated
change is expensive — where that is true the cost is paid by a test, a migration or a public API and
is written there, not where it reads as an instruction to stop thinking.

**What needs a record at all.** It is expensive to undo, or someone will later ask "why is this like
this?". If neither applies it belongs in the learning log. A spec MUST NOT take a decision: one
surfaced while writing it goes on the feature's decision sheet, and the record follows the sheet.

Rationale: an ADR reconstructed weeks later summarizes instead of recording, and its rejected
options are already forgotten. But a record that outranks its subject is the opposite failure and
the one this project met first: five accepted records on one private function, and a picture nobody
could change without reconciling all five.

### VII. The core stays portable

- `crates/monospace-core` holds all domain logic. `crates/monospace-cli` holds none.
- The core's public API MUST NOT assume a CLI, a TUI or a terminal. Later phases compile it to
  WebAssembly, and the gate's `wasm` step enforces that boundary rather than prose.
- `monospace` is reserved for the interactive TUI of a later phase and MUST NOT be taken by another
  binary.
- Public library APIs MUST be documented with rustdoc as they are introduced, not retrofitted.

Rationale: a boundary that exists only in a document erodes; one the compiler refuses to cross does
not.

### VIII. The record is sized to the decision

- An artifact MUST NOT restate what another already says; it cites it by name. A paragraph that
  would change nobody's decision if deleted is deleted.
- Each artifact has a ceiling. Exceeding one is not a violation to justify: it signals that the
  slice is too thick, and the first answer is to split the feature rather than compress the prose.
  Where a ceiling is exceeded without splitting, `plan.md`'s Complexity Tracking records which
  artifact, by how much, and why.

  | `spec.md` | `decisions.md` | `research.md` | `plan.md` | ADR `exploratory`/`working` | ADR `load-bearing` |
  | --- | --- | --- | --- | --- | --- |
  | 120 | 60 | 100 | 80 | 60 | 150 |

- A generated picture and the description it comes from do not count against a ceiling. They replace
  prose rather than adding to it, and charging for them would push an artifact back toward
  describing what it could have shown.

Rationale: this repository has produced 2.8 lines of prose per line of Rust and the maintainer
reports not reading it. Unread prose governs nothing, yet the agent reads it and treats an
unreviewed sentence as a rule — which makes an unread record worse than a missing one, because it
has authority nobody granted it.

## Constraints and Dependencies

### Language of the artifacts

Code identifiers, comments, doc comments, README, `docs/`, specs, ADRs, commit messages, PR
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

Each spec defines its own testing expectations, and unit tests for core logic are the minimum any
may ask for. A behavior rule with no test named against it is an unfinished spec.

Tests come in two kinds, kept apart in separate directories where they are snapshots:

- A **contract test** pins a decision, and its name or comment says which rule of the model it
  holds. Changing one is changing the decision, so it needs an entry on the decision sheet.
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
crate that does
([ADR-0036](../../docs/decisions/0036-hold-every-table-but-light-outside-the-core.md)); and
`monospace-diagram`, the model holding a diagram after it is drawn — shapes with an identity, an
order and positions that may depend on each other — on the same terms
([ADR-0038](../../docs/decisions/0038-hold-the-diagram-model-in-a-crate-above-the-core.md)).

Each of the last two is here for a reason of its own, recorded in the ADR cited with it, not because
the phase builds four things. A further crate proposed for any other reason widens this section and
is renegotiated rather than assumed.

### Out of scope for this phase

A plan proposing any of these MUST be stopped and renegotiated rather than quietly widened:
WebAssembly bindings, a web front-end, non-terminal GUIs, persistence, collaboration, and export
formats beyond plain text. Where each is expected to arrive instead is the `Phase` field of its
capability issue in the GitHub Project, which holds direction and no rules: nothing on the board
widens this section.

## Development Workflow

### Spec Kit is the workflow

New work goes through `/speckit-specify`, then `/speckit-clarify` when the spec has open questions,
then `/speckit-plan`, `/speckit-tasks` and `/speckit-implement`. Artifacts live under
`specs/NNN-slug/`, where `NNN` is the number of the GitHub issue the feature implements rather than
a position in a local sequence, so the numbering skips. Exactly one issue represents one spec, and
its labels — `wish`, `deciding`, `building` — are the only record of where the work stands. The
branch, the label and the pointer to the feature directory are created by `cargo xtask spec`, and
none of the three may be done by hand. A change to the repository's own tooling is not a feature: it
needs an ADR and commits, and MUST NOT take a spec directory or a stage branch. Features 001 to 006
predate the rule and keep their numbers.

### Two stages, and where the cut falls

A stage boundary falls where a question is answered, not where a command finishes. Each is its own
branch and its own pull request against `main`, and a stage MUST NOT be opened before the previous
one has merged.

- **Deciding** — `/speckit-specify`, `/speckit-clarify`, and part one of `/speckit-plan`. Merges
  with `spec.md`, `research.md`, an answered `decisions.md` and the part of `plan.md` part one fills.
  Closes: is this what is wanted, and is this how it will be resolved?
- **Building** — part two of `/speckit-plan`, `/speckit-tasks`, `/speckit-implement`. Merges with
  the design artifacts, the tasks, the code and the tests. Closes: does it work, and is it what was
  agreed?

### The plan runs in two parts

`/speckit-plan` MUST NOT be run end to end. Part one runs Phase 0 and stops, producing `research.md`,
`decisions.md`, and the part of `plan.md` that precedes a decision — its summary and its Constitution
Check. It MUST NOT produce `data-model.md`, `contracts/` or `quickstart.md`, because writing those is
taking the decisions. The maintainer then reads `decisions.md` in full — the one
artifact read in full, which is what its ceiling buys — and answers it. Part two runs Phase 1
against the answered sheet.

Rationale: Phase 0 closes every open question by research, so a maintainer shown only the finished
plan is reviewing conclusions whose alternatives are already spent.

### The decision sheet

`specs/NNN-slug/decisions.md` holds every decision the feature has to take and nothing else. Each
entry gives, in one line each: the question, the proposal, the altitude, what reversing it would
cost, and the alternative not taken — shown as a rendering where the subject renders. Every
domain-level entry is the maintainer's to answer; a module-level one may be answered by the agent
where it is settled practice, per _Whose decision it is_.

No more than seven entries, inside the ceiling principle VIII gives it. A feature needing more is too
thick, per principle II, and the sheet not fitting is the earliest available signal of it.

### No decision outside the sheet

Where `/speckit-tasks` or `/speckit-implement` meets a decision the sheet does not hold, work MUST
stop and ask. It MUST NOT be resolved and recorded afterwards, or the sheet records what was easy to
foresee rather than what was decided.

### Where a rationale goes

- `docs/decisions/` owns every domain-level decision that outlives the feature which surfaced it, at
  the commitment principle VI assigns it.
- A module's **rustdoc**, under `Design notes`, owns every module-level decision. This is the
  default home, not the fallback.
- A feature's `research.md` owns investigation local to that feature. When a finding turns out to be
  durable, the ADR is written and `research.md` links to it. Spec Kit's format for that file carries
  no status, no consequences and no supersession, so it cannot serve as the history.
- `plan.md`'s Complexity Tracking owns two things: justifying a departure from this constitution for
  that feature, and recording a ceiling exceeded without a split.

Rationale: a rationale with three possible homes has none.

### The model owns the design

`docs/model.md` is design intent and owns the domain vocabulary. A spec names the sections it
implements and MUST NOT restate them. If a slice needs a rule the model does not have, the model
changes first.

What belongs in it is bounded by principle VI: a rule observable from outside the module that
implements it. The procedure a single figure follows to choose among outputs that all satisfy the
model is not a rule of the model, however precisely it can be stated, and belongs in that figure's
rustdoc. A document collecting such procedures MUST NOT be created: two figures may join two points
differently, so a shared file invites the second to conform to the first — the same premature
generalization, one floor lower.

### The previous spec home is history

`docs/specs/` holds specs 0001–0005, is frozen, and its numbering does not continue. Nothing is
added to it and nothing is deleted from it.

### No code before the plan is agreed

A plan is agreed by the maintainer, not assumed.

### Whose decision it is

When a decision belongs to the maintainer — cost, scope, taste, or a risk they carry — the real
options MUST be presented with their trade-offs, what is reversible and what is not, a
recommendation with a confidence level, and what information would change it. When it is settled
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
scope and the constraints. [`docs/model.md`](../../docs/model.md) owns the domain's design and its
open questions; the GitHub Project owns direction and the backlog; `CONTRIBUTING.md` owns how to run
the tooling; `CLAUDE.md` owns session guidance. Where any of them restates a rule stated here, this
file is the one to follow and the duplication is a defect to remove.

### Amendment procedure

An amendment is made in the same increment as the decision that requires it, and the commit says so
— never drifted from silently. An amendment that reverses a recorded decision also needs the ADR
that reverses it. Every amendment updates the version line and replaces the Sync Impact Report at
the top of this file, moving the previous one to `docs/decisions/constitution-history.md`. The
report says what changed and names the ADRs; it does not repeat their reasoning.

### Versioning policy

MAJOR for a backward-incompatible removal or redefinition of a principle, MINOR for a new principle
or materially expanded guidance, PATCH for clarifications and wording.

### Compliance review

Every `/speckit-plan` run MUST fill its Constitution Check, and a plan that cannot pass a principle
MUST record the violation in Complexity Tracking with the reason, or be changed. The mechanical
parts — formatting, lints, spelling, the WebAssembly boundary, tests, docs, rendered pictures — are
enforced by `cargo xtask check` and are not a matter of review. The rest — thin slices, structural
commits kept separate, decisions recorded at the right altitude, claims measured, the decision sheet
answered before Phase 1 — is checked by reading, at plan time and at review time.

**Version**: 2.0.0 | **Ratified**: 2026-09-08 | **Last Amended**: 2026-09-21
