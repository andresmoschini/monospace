---
status: accepted
scope: tooling
commitment: working
date: 2026-09-21
decision-makers: Andrés Moschini
---

# Stop at the decision sheet, and merge three stages into two

## Context and Problem Statement

`/speckit-plan` is built to close every open question by itself: its Phase 0 resolves all
`NEEDS CLARIFICATION` and errors on an unresolved one, then writes `data-model.md`, `contracts/` and
`quickstart.md` in the same run. There is no point in it where it stops. `/speckit-clarify` runs
before it, over the spec, with a product taxonomy — user goals, personas, UX flow, entities — which
cannot reach a decision about an algorithm, because none exists yet at that point.

The record of what that costs is in feature 099's `research.md`. Q2, "is `derive_path` repaired or
rewritten?", was put to the maintainer. Q3, "how is the sweep pinned?", was not, and it produced
ADR-0045 and 20,424 lines of snapshot. Nothing distinguishes the two.

The three stages of [ADR-0032](0032-split-a-spec-into-three-staged-branches.md) have a matching
problem from the other side: they are three because Spec Kit has three commands, so a merge boundary
falls where a command ends. At the plan boundary nothing is decided — the decidable content of a
plan is which line of research is taken, and that happens inside the session, not in its pull
request.

## Decision Drivers

- A checkpoint is worth its ceremony only where something is decided. Twelve features at three pull
  requests is thirty-six, and one in three closed no question.
- A review budget that is spent has to be spent on one page, not on 1,200 lines per feature.

## Decision Outcome

Chosen option: **split `/speckit-plan` in two around a one-page decision sheet, and cut the flow at
that sheet rather than at a command boundary** — because that is where the decisions were escaping.

Part one runs Phase 0 and stops, producing `research.md`, `decisions.md` and the part of `plan.md`
that precedes a decision. The maintainer answers the sheet; part two runs Phase 1 against it. The
stages become **deciding** (specify, clarify, plan part one) and **building** (plan part two, tasks,
implement), and the labels become `wish`, `deciding`, `building`. This supersedes ADR-0032, whose
premise — a merged artifact is the handoff — is kept whole; only the count of boundaries changes.

### Consequences

- Good, because the merge boundary now falls where the maintainer decides something.
- Good, because twelve features cost twenty-four pull requests rather than thirty-six.
- Bad, because `cargo xtask spec` does not implement this yet: `new` still opens `NNN-slug-spec` and
  `stage` still takes `plan` and `impl`. Until that lands, the two stages are followed by hand and
  the labels on open issues are the old three.
- Bad, because a decision the sheet failed to foresee now stops the work instead of being taken in
  passing, which costs a session boundary on a feature that was flowing.

## Reversibility

Cheap for a feature not yet started, and the stage names are the only thing a later change would
have to undo. What cannot be undone is the history: features run under this rule leave two pull
requests where three would have been, findable by their `-deciding` and `-building` suffixes.

## Confidence

Medium (65%). What would change it: three features whose deciding pull request changed nothing,
which would mean the cut is in the wrong place again. What would prove it wrong: implementation
sessions that keep meeting decisions the sheet does not hold, which would mean Phase 0 cannot see
far enough to be worth stopping at.

## Revisions

- 2026-09-21 — recorded. The tooling side is not built.

## More Information

- [ADR-0032](0032-split-a-spec-into-three-staged-branches.md), superseded by this record.
- [ADR-0033](0033-keep-the-flow-state-in-labels-on-one-issue.md) and
  [ADR-0034](0034-let-xtask-own-the-feature-branch.md), unchanged: the state is still labels on one
  issue, and `cargo xtask spec` still owns the branch.
