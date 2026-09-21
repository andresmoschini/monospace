---
status: accepted
scope: tooling
commitment: working
date: 2026-09-21
decision-makers: Andrés Moschini
---

# Stop at the decision sheet, and merge three stages into two

## Context and Problem Statement

`/speckit-plan` is built to close every open question by itself, and has no point where it stops:
Phase 0 errors on an unresolved clarification, then writes the design artifacts in the same run.
`/speckit-clarify` runs before it over the spec, with a product taxonomy that cannot reach a
decision about an algorithm. Feature 099's `research.md` shows the cost: Q2, "is `derive_path`
repaired or rewritten?", was put to the maintainer; Q3, "how is the sweep pinned?", was not, and it
produced ADR-0045 and 20,424 lines of snapshot. Nothing distinguishes the two.

The three stages of [ADR-0032](0032-split-a-spec-into-three-staged-branches.md) have the mirror
problem: they are three because Spec Kit has three commands, so a merge boundary falls where a
command ends rather than where something is decided. Nothing is decided at the plan boundary.

## Decision Drivers

- A checkpoint earns its ceremony only where something is decided. Twelve features at three pull
  requests is thirty-six ceremonies, and one in three closed no question.
- A review budget that is actually spent has to fit on one page, not 1,200 lines per feature.

## Decision Outcome

Chosen option: **split `/speckit-plan` in two around a one-page decision sheet, and cut the flow at
that sheet rather than at a command boundary** — because that is where the decisions were escaping.

Part one runs Phase 0 and stops, producing `research.md`, `decisions.md` and the part of `plan.md`
that precedes a decision; part two runs Phase 1 against the answered sheet. The stages become
**deciding** and **building**. This supersedes ADR-0032, whose premise — a merged artifact is the
handoff — is kept whole.

### Consequences

- Good, because the merge boundary now falls where the maintainer decides something.
- Good, because twelve features cost twenty-four pull requests rather than thirty-six.
- Bad, because `cargo xtask spec` does not implement it yet: it still opens the three branches of
  the old arrangement, so until that lands the two stages are followed by hand.
- Bad, because a decision the sheet missed now stops the work instead of being taken in passing.

## Reversibility

Cheap for a feature not yet started; the stage names are all a later change would have to undo. The
history is permanent: a feature run under this rule leaves two pull requests where three would have
been.

## Confidence

Medium (65%), and three features whose deciding pull request changed nothing would lower it.

## Revisions

- 2026-09-21 — recorded. The tooling side is not built.
