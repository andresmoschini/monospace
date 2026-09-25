# Constitution history

Earlier Sync Impact Reports, moved out of [the constitution](../../.specify/memory/constitution.md)
by [ADR-0050](0050-record-a-decision-at-the-boundary-it-cannot-cross.md).

They are history for a person reading that file, they change no decision anyone takes from it, and
what each of them says its amendment did is already in that amendment's commit message — the
duplication principle VIII refuses, in the file that states it. The current version's report stays
at the top of the constitution; when a new version ships, the one it replaces is appended here,
unedited.

Copied from version 1.6.0 with no word changed; the gate's formatter re-wrapped the lines and
flattened the indented lists, which is the only difference from the header they came out of. The
follow-up TODOs at the end were carried there for several versions; they are `wish` issues now,
which is where work waiting to be done belongs.

---

Sync Impact Report — 2026-09-21

Version change: 1.6.0 → 2.0.0

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

---

Sync Impact Report — 2026-09-14

Version change: 1.5.0 → 1.6.0

Rationale for MINOR: materially expanded guidance. In scope for this phase admits a fourth crate,
the diagram model that holds a diagram after it is drawn. No principle is added, removed or
redefined, and principle VII is untouched: the new crate respects the boundary that section cites
rather than crossing it, since it holds domain logic and assumes no CLI, TUI or terminal, and
`monospace` stays reserved for the interactive application. Out of scope for this phase is
unchanged, and nothing on it is widened.

Decisions this amendment implements: ADR-0038 (the diagram model lives in a crate above the core).

Modified sections:

- In scope for this phase: "Three things" becomes four. monospace-diagram is named, and the
  paragraph that used to explain why the third crate is there now explains both, keeping the rule
  that a crate proposed for any other reason is still a scope change to renegotiate.

Templates and commands reviewed: unchanged. plan-template.md is the only one that refers to this
file and it reads it at runtime.

Sync Impact Report — 2026-09-11

Version change: 1.4.0 → 1.5.0

Rationale for MINOR: materially expanded guidance. In scope for this phase admits a third crate, a
library of glyph sets that depends on the core. No principle is added, removed or redefined, and
principle VII is untouched: the new crate respects the boundary that section cites rather than
crossing it, since it holds domain data and assumes no CLI, TUI or terminal. Out of scope for this
phase is unchanged, and nothing on it is widened.

Decisions this amendment implements: ADR-0036 (every glyph table but Light lives outside the core).

Modified sections:

- In scope for this phase: "Two things" becomes three. monospace-glyph-sets is named, with the rule
  that its presence is about where a table may come from and not about what the phase builds, so a
  crate proposed for any other reason is still a scope change to renegotiate.

Templates and commands reviewed: unchanged. plan-template.md is the only one that refers to this
file and it reads it at runtime.

Sync Impact Report — 2026-09-11

Version change: 1.3.1 → 1.4.0

Rationale for MINOR: materially expanded guidance. Spec Kit is the workflow gains the staged flow,
the single issue and its labels, and the rule that a tooling change is not a feature. No principle
is added, removed or redefined. Demonstrable increments keeps its subject and its rationale; the
rule inside it that said "One task per commit" is refined rather than dropped, because the gate and
the ban on --no-verify already decided where a commit may fall.

Decisions this amendment implements: ADR-0032 (three staged branches), ADR-0033 (the flow's state in
labels on one issue, superseding ADR-0025 and part of ADR-0023) and ADR-0034 (xtask owns the feature
branch).

Modified sections:

- II. Demonstrable increments: "One task per commit" becomes the rule that a commit ticks the
  checkboxes it completed and leaves the gate green, which is one commit per task only when a task
  reaches green alone.
- Spec Kit is the workflow: the requirement to invoke /speckit-specify with the feature directory
  given explicitly is gone, because cargo xtask spec writes .specify/feature.json, which is the file
  Spec Kit's own scripts read. The staged flow, the single issue and its labels, and the
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

- Constraints and Dependencies, Development Workflow, Governance: the seventeen rules already named
  in bold are headings, so each can be cited on its own instead of through the section holding it.
  The three parent anchors are unchanged, so existing links still resolve.
- I. Process over product, III. One definition of green: one pair of bullets in each stated a single
  rule twice, and each pair is now one sentence.

Templates and commands reviewed: unchanged since 1.0.0. Of the four templates only plan-template.md
refers to the constitution, and it reads this file at runtime.

Prior versions:

- 1.3.0 (2026-09-08) — a feature's number became the number of the GitHub issue it implements, per
  ADR-0024, and the two pointers naming docs/roadmap.md as the owner of direction were redirected to
  the GitHub Project, per ADR-0023.
- 1.2.0 (2026-09-08) — took over the last rules docs/brief.md still held: what is in scope, what a
  spec must ask of its tests, and what a learning-log entry covers, with Governance naming itself
  rather than the brief as the owner of scope. It closed TODO(MIGRATION_ADR), recorded as ADR-0021,
  and the brief's half of TODO(TRIM_DUPLICATION); the brief itself was removed in the commit after
  it. Its note that the numbering continues at 006 is superseded as of this version.
- 1.1.0 (2026-09-08) — Development Workflow gained "Where a rationale goes": docs/decisions/ for
  what outlives a feature, research.md for investigation local to it, Complexity Tracking for
  justifying a departure from this file.
- 1.0.0 (2026-09-08) — first ratified constitution. Principles I–VII, Constraints and Dependencies,
  Development Workflow and Governance, filled from the practice already recorded in docs/brief.md,
  CONTRIBUTING.md, CLAUDE.md, docs/specs/README.md and docs/decisions/README.md. Replaced the
  unfilled `[PLACEHOLDER]` scaffold from `specify init`.

Follow-up TODOs:

- TODO(TRIM_DUPLICATION): CONTRIBUTING.md and CLAUDE.md still state several of these rules in their
  own words. Reduce them to what only they own — CONTRIBUTING keeps how to run the tooling,
  CLAUDE.md keeps session guidance — and point the rest here. Two wordings of one rule is the
  failure this project already refuses to accept for the quality gate.

Closed since 1.3.0 was written:

- TODO(ROADMAP_REMOVAL) — docs/roadmap.md is removed and the four references that pointed at it are
  redirected. The Phase field and the capability issues exist.
