<!--
Sync Impact Report — 2026-09-11

Version change: 1.4.0 → 1.5.0

Rationale for MINOR: materially expanded guidance. In scope for this phase admits a third crate, a
library of glyph sets that depends on the core. No principle is added, removed or redefined, and
principle VII is untouched: the new crate respects the boundary that section cites rather than
crossing it, since it holds domain data and assumes no CLI, TUI or terminal. Out of scope for this
phase is unchanged, and nothing on it is widened.

Decisions this amendment implements: ADR-0036 (every glyph table but Light lives outside the core).

Modified sections:
  - In scope for this phase: "Two things" becomes three. monospace-glyph-sets is named, with the
    rule that its presence is about where a table may come from and not about what the phase
    builds, so a crate proposed for any other reason is still a scope change to renegotiate.

Templates and commands reviewed: unchanged. plan-template.md is the only one that refers to this
file and it reads it at runtime.

Sync Impact Report — 2026-09-11

Version change: 1.3.1 → 1.4.0

Rationale for MINOR: materially expanded guidance. Spec Kit is the workflow gains the staged flow,
the single issue and its labels, and the rule that a tooling change is not a feature. No principle
is added, removed or redefined. Demonstrable increments keeps its subject and its rationale; the
rule inside it that said "One task per commit" is refined rather than dropped, because the gate and
the ban on --no-verify already decided where a commit may fall.

Decisions this amendment implements: ADR-0032 (three staged branches), ADR-0033 (the flow's state
in labels on one issue, superseding ADR-0025 and part of ADR-0023) and ADR-0034 (xtask owns the
feature branch).

Modified sections:
  - II. Demonstrable increments: "One task per commit" becomes the rule that a commit ticks the
    checkboxes it completed and leaves the gate green, which is one commit per task only when a
    task reaches green alone.
  - Spec Kit is the workflow: the requirement to invoke /speckit-specify with the feature directory
    given explicitly is gone, because cargo xtask spec writes .specify/feature.json, which is the
    file Spec Kit's own scripts read. The staged flow, the single issue and its labels, and the
    tooling-is-not-a-feature rule are added.

Templates and commands reviewed: unchanged. Of the four templates only plan-template.md refers to
the constitution, and it reads this file at runtime.

Sync Impact Report — 2026-09-09

Version change: 1.3.0 → 1.3.1

Rationale for PATCH: clarification and wording only. No principle is added, removed or redefined,
and no rule changes what it requires.

Decisions this amendment implements: none, and it carries no ADR because it reverses nothing
recorded. What prompted it is the measurement in ADR-0027.

Modified sections:
  - Constraints and Dependencies, Development Workflow, Governance: the seventeen rules already
    named in bold are headings, so each can be cited on its own instead of through the section
    holding it. The three parent anchors are unchanged, so existing links still resolve.
  - I. Process over product, III. One definition of green: one pair of bullets in each stated a
    single rule twice, and each pair is now one sentence.

Templates and commands reviewed: unchanged since 1.0.0. Of the four templates only
plan-template.md refers to the constitution, and it reads this file at runtime.

Prior versions:
  - 1.3.0 (2026-09-08) — a feature's number became the number of the GitHub issue it implements,
    per ADR-0024, and the two pointers naming docs/roadmap.md as the owner of direction were
    redirected to the GitHub Project, per ADR-0023.
  - 1.2.0 (2026-09-08) — took over the last rules docs/brief.md still held: what is in scope, what
    a spec must ask of its tests, and what a learning-log entry covers, with Governance naming
    itself rather than the brief as the owner of scope. It closed TODO(MIGRATION_ADR), recorded as
    ADR-0021, and the brief's half of TODO(TRIM_DUPLICATION); the brief itself was removed in the
    commit after it. Its note that the numbering continues at 006 is superseded as of this version.
  - 1.1.0 (2026-09-08) — Development Workflow gained "Where a rationale goes": docs/decisions/ for
    what outlives a feature, research.md for investigation local to it, Complexity Tracking for
    justifying a departure from this file.
  - 1.0.0 (2026-09-08) — first ratified constitution. Principles I–VII, Constraints and
    Dependencies, Development Workflow and Governance, filled from the practice already recorded in
    docs/brief.md, CONTRIBUTING.md, CLAUDE.md, docs/specs/README.md and docs/decisions/README.md.
    Replaced the unfilled `[PLACEHOLDER]` scaffold from `specify init`.

Follow-up TODOs:
  - TODO(TRIM_DUPLICATION): CONTRIBUTING.md and CLAUDE.md still state several of these rules in
    their own words. Reduce them to what only they own — CONTRIBUTING keeps how to run the tooling,
    CLAUDE.md keeps session guidance — and point the rest here. Two wordings of one rule is the
    failure this project already refuses to accept for the quality gate.

Closed since 1.3.0 was written:
  - TODO(ROADMAP_REMOVAL) — docs/roadmap.md is removed and the four references that pointed at it
    are redirected. The Phase field and the capability issues exist.
-->

# Monospace Constitution

## Core Principles

### I. Process over product (NON-NEGOTIABLE)

This repository exists to build proficiency in Spec-Driven Development and in idiomatic Rust
architecture. ASCII diagramming is the vehicle, not the goal.

- When the fastest route to a feature conflicts with the route that teaches better Rust design or
  better spec-driven practice, the second MUST be taken.
- Neither more surface area shipped nor a smaller feature excuses skipping a spec, an ADR or a
  learning-log entry: a plan that saves effort by collapsing the process MUST be rejected.

Rationale: every other principle here costs time. They are affordable only because the time is the
point.

### II. Demonstrable increments

- Every commit that lands MUST leave the workspace building, all validations green, and
  `cargo run -p monospace-cli` producing output. A bare `cargo run` is ambiguous once the workspace
  holds more than one binary and MUST NOT be used as the acceptance command.
- Specs MUST be sliced thin. Several small specs are preferred over one large one.
- A commit MUST tick exactly the checkboxes in `tasks.md` that it completed, and MUST leave the
  gate green. Since the hook runs the gate and `--no-verify` is forbidden, a commit can only
  exist where the tree is green: that is one commit per task where a task reaches green on its
  own, and one per group where it does not.
- An increment — a slice that reaches a demonstrable state, usually several commits — MUST end with
  an appended entry in `docs/learning-log.md`. The entry covers what was learned about Rust design
  and idiom, what was learned about working this way, and optionally a trade-off worth remembering
  later. A lesson earns an entry only if it came with evidence: what was tried, and what it turned
  out to be.

Rationale: a slice that cannot be shown to someone cannot be judged, and unjudged work accumulates
silently.

### III. One definition of green (NON-NEGOTIABLE)

- `cargo xtask check` is the only definition of green. The pre-commit hook runs it, CI runs it, and
  a new check MUST be added to that entrypoint rather than to either of them.
- Adding a check is two commits: first fix what it finds, then enable it. If it finds nothing, say
  so — a fix MUST NOT be manufactured to fill the commit.
- `git commit --no-verify` MUST NOT be used. When the hook fails, fix the cause or stop and report
  it.
- `git rebase` and `git cherry-pick` do not fire the hook. After rewriting history, the gate MUST be
  run on each rewritten commit, not only on the tip.
- Configuration for the gate is kept minimal by removal: an entry MUST be shown to break something
  when taken out, or it is removed. In a linter or a spell checker, an entry that changes nothing
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

Rationale: an unverified claim in a durable document outlives the session that produced it and is
believed by whoever reads it next.

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

Rationale: the prefix is what makes the distinction visible in the log without reading diffs, and a
mixed commit is unreviewable in both of its halves.

### VI. Decisions recorded when taken

- An architecture decision MUST be recorded as an ADR under `docs/decisions/` at the moment it is
  taken. If code is about to depend on an unrecorded decision, the record comes first.
- An accepted ADR MUST NOT be edited to change its conclusion. A reversal is a new ADR, and the old
  one's status becomes `superseded by ADR-NNNN`. Fixing a typo or a broken link is fine.
- The test for whether something needs an ADR: it is expensive to undo, or someone will later ask
  "why is this like this?". If neither applies it belongs in the learning log.
- A spec MUST NOT take a decision. When writing one surfaces a decision, the ADR is written first
  and the spec follows it.

Rationale: an ADR reconstructed weeks later summarizes what happened instead of recording the
reasoning; by then the rejected options are forgotten, which makes the most valuable section the
least accurate one.

### VII. The core stays portable

- `crates/monospace-core` holds all domain logic. `crates/monospace-cli` holds none.
- The core's public API MUST NOT assume a CLI, a TUI or a terminal. Later phases compile it to
  WebAssembly, and that boundary is enforced by the gate's `wasm` step rather than by prose.
- `monospace` is reserved for the interactive TUI of a later phase and MUST NOT be taken by another
  binary.
- Public library APIs MUST be documented with rustdoc as they are introduced, not retrofitted.

Rationale: a boundary that exists only in a document erodes; one the compiler refuses to cross does
not.

## Constraints and Dependencies

### Language of the artifacts

Code identifiers, comments, doc comments, README, `docs/`, specs,
ADRs, commit messages, PR descriptions, CLI output, error messages and help text are all in English.
Conversation with the maintainer may be in any language; what lands in the repository is English.

### Toolchain

Rust, edition 2024, pinned to an exact version in `rust-toolchain.toml` together with
its components and the `wasm32-unknown-unknown` target. No nightly-only features. No minimum
supported Rust version is declared while nothing outside this repository depends on these crates.
Node, at the version in `.nvmrc`, backs the checks Rust cannot perform.

### Dependencies

A dependency MUST NOT be added without asking the maintainer first. Before adding
or pinning any version, its publication date MUST be verified to be at least seven days old, and the
version and that date reported. Domain logic prefers the standard library; infrastructure concerns
prefer idiomatic, well-established crates over reinvention.

### Testing

Each spec defines its own testing expectations, and unit tests for core logic are the
minimum any of them may ask for. A behavior rule with no test named against it is an unfinished
spec, not a finished feature.

### In scope for this phase

Three things, and their boundary is principle VII: `monospace-core`, the
Rust library holding the diagramming logic — input parsing, layout and positioning, ASCII rendering
— `monospace-cli`, a minimal non-interactive consumer that produces diagrams from the terminal, and
`monospace-glyph-sets`, a library of the glyph sets the core does not ship, depending on the core
and privileged no further than any other crate that depends on it
([ADR-0036](../../docs/decisions/0036-hold-every-table-but-light-outside-the-core.md)).

The third one is here to make a claim checkable — that a glyph set can come from outside the core —
and not because the phase builds three things. A further crate proposed for any other reason is a
widening of this section, to be renegotiated rather than assumed.

### Out of scope for this phase

A plan proposing any of these MUST be stopped and renegotiated rather than quietly widened:
WebAssembly bindings, a web front-end, non-terminal GUIs, persistence,
collaboration, and export formats beyond plain text. Where each of them is expected to arrive
instead is the `Phase` field of its capability issue in the GitHub Project, which holds direction
and no rules. This section is the whole of the rule: nothing on the board widens it.

## Development Workflow

### Spec Kit is the workflow

New work goes through `/speckit-specify`, then `/speckit-clarify` when
the spec has open questions, then `/speckit-plan`, `/speckit-tasks` and `/speckit-implement`.
Artifacts live under `specs/NNN-slug/`, where `NNN` is the number of the GitHub issue the feature
implements rather than a position in a local sequence, so the numbering skips. Exactly one issue
represents one spec, and the labels on it — `wish`, `spec`, `plan`, `doing` — are the only record of
where the work stands. A spec crosses three stages, each on its own branch and its own pull request
against `main`, and a stage MUST NOT be opened before the previous one has merged. The branch, the
label and the pointer to the feature directory are created by `cargo xtask spec`, and none of the
three may be done by hand. A change to the repository's own tooling is not a feature: it needs an
ADR and commits, and MUST NOT take a spec directory or a stage branch. Features 001 to 006 predate
the rule and keep the numbers they were given.

### The previous spec home is history

`docs/specs/` holds specs 0001–0005 and is frozen: it is not
extended, and its numbering does not continue. Specs 0001–0003 are implemented and stay as the
record of what was asked for and when. Specs 0004 and 0005 were still drafts at the migration and
are re-authored as Spec Kit features; the frozen files stay in place, marked as superseded, for the
same reason an ADR is superseded rather than deleted.

```text
docs/specs/          # 0001-0005, frozen history
specs/NNN-slug/      # where new work lives; NNN is the feature's issue number
docs/decisions/      # ADRs, MADR 4.0, written when the decision is taken
docs/model.md        # the design, sliced by specs rather than restated
docs/learning-log.md # one entry per increment
```

### Where a rationale goes

Three artifacts can hold one, and they are not interchangeable.

- `docs/decisions/` owns every decision that outlives the feature which surfaced it. It is
  append-only history: an accepted record is superseded, never edited to change its conclusion.
- A feature's `research.md` owns investigation local to that feature — which crate, which version,
  what is idiomatic. When a finding turns out to be durable, the ADR is written and `research.md`
  links to it instead of standing in for it. Spec Kit's format for that file — decision, rationale,
  alternatives considered — carries no status, no consequences and no supersession, so it cannot
  serve as the history however well it is written.
- The Complexity Tracking table in a `plan.md` owns one thing: justifying a departure from this
  constitution, in that plan, for that feature.

Rationale: a rationale with three possible homes has none, and principle VI is unenforceable while
"where is this recorded?" has more than one answer.

### The model owns the design

`docs/model.md` is design intent and owns the domain vocabulary. A
spec names the sections of the model it implements and MUST NOT restate them. If a slice needs a
rule the model does not have, the model changes first.

### No code before the plan is agreed

A plan is agreed by the maintainer, not assumed.

### Whose decision it is

When a decision belongs to the maintainer — cost, scope, taste, or a risk
they carry — the real options MUST be presented with their trade-offs, what is reversible and what
is not, a recommendation with a confidence level, and what information would change it. When it is
settled practice and the maintainer has no stake, choose the conventional answer, say what was
chosen and why, and carry on.

### Fixing a commit

A correction that changed nothing for anyone — a typo, a wrong status line, a
formatting slip — belongs in the commit that got it wrong. When something was actually wrong, in
behavior or in a claim someone could have acted on, the fix is its own commit and says what was
wrong. This holds after pushing: `main` is never rewritten, and every force push uses
`--force-with-lease`.

### Cross-references

Cite a section of another document by its name, not by its number.

## Governance

This constitution supersedes prior practice wherever the two conflict. It owns the principles, the
scope and the constraints. [`docs/model.md`](../../docs/model.md) remains the owner of the domain's
design and its open questions; the GitHub Project of direction and of the backlog; `CONTRIBUTING.md`
of how to run the tooling; `CLAUDE.md` of session guidance. Where any of them
restates a rule stated here, this file is the one to follow, and the duplication is a defect to
remove.

### Amendment procedure

An amendment is made in the same increment as the decision that requires
it, and the commit says so — never drifted from silently. An amendment that reverses a recorded
decision also needs the ADR that reverses it. Every amendment updates the version line and the Sync
Impact Report at the top of this file.

### Versioning policy

Semantic versioning of governance: MAJOR for a backward-incompatible removal
or redefinition of a principle, MINOR for a new principle or materially expanded guidance, PATCH for
clarifications and wording.

### Compliance review

Every `/speckit-plan` run MUST fill its Constitution Check against these
principles, and a plan that cannot pass one MUST record the violation in its Complexity Tracking
with the reason, or be changed. The mechanical parts — formatting, lints, spelling, the WebAssembly
boundary, tests, docs — are enforced by `cargo xtask check` and are not a matter of review. The
rest — thin slices, structural commits kept separate, decisions recorded when taken, claims
measured — is checked by reading, at plan time and at review time.

**Version**: 1.5.0 | **Ratified**: 2026-09-08 | **Last Amended**: 2026-09-11
