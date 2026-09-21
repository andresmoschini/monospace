---
status: accepted
scope: tooling
commitment: working
date: 2026-09-21
decision-makers: Andrés Moschini
---

# Record a decision at the boundary it cannot cross

## Context and Problem Statement

Fifty records exist, forty-seven of them `accepted`, one for every 186 lines of Rust. Five — 0044
and 0046 to 0049 — hold one subject, how an arrow picks its route, and none can be cited without
citing another. Nothing outside `shape::arrow` can observe that subject: `derive_path` and
`RouteRectangle` are private, `Route` is `pub(crate)`. The rule sits in `docs/model.md` anyway, the
document a spec "MUST NOT restate" and which "changes first" when a slice needs a rule it lacks — so
changing a tie-break inside one private function is an amendment to the domain.

Two rules produced that. Principle VI asked for a record "at the moment it is taken", which sizes a
record to a moment rather than to a subject, and it offered one home, so every decision cost the
same whatever could see it. Nothing consumed the `Reversibility` and `Confidence` the template
already collected, so "cheap, 65%" weighed what "permanent" weighs.

## Decision Drivers

- 26,240 lines of Markdown against 9,302 of Rust, which the maintainer reports not reading. An agent
  reads them and treats a sentence nobody reviewed as a rule.
- Promoting a decision later costs one record; demoting one costs undoing the gravity it created.
  The default belongs on the cheap side.
- "What outside this module can observe it?" has an answer, unlike "is this important?".

## Decision Outcome

Chosen option: **record a decision at the altitude of the boundary it cannot cross, one record per
subject, declaring how far it is committed** — because altitude is the only available test a reader
can apply without knowing the history.

Principle VI is rewritten around three declarations: **altitude** (domain-level into a model
document plus an ADR; module-level into that module's rustdoc under `Design notes`), **subject** (a
record that cannot be cited without citing another is the same record), and **commitment**
(`exploratory`, `working`, `load-bearing`, each with how it is changed). Principle VIII is new: no
artifact restates another, and each has a ceiling whose breach means the slice is too thick. The
constitution goes to 2.0.0 and its earlier reports to
[`constitution-history.md`](constitution-history.md).

### Consequences

- Good, because a rule nothing outside a module can see now costs a `feat` to change, which is what
  the code already said it was.
- Good, because `Reversibility` and `Confidence` are consumed at last: `commitment` is what they
  argue for.
- Bad, because altitude is a judgement whose error shows up late — when a second module wants the
  rule, or when a record already grew gravity.
- Bad, because fifty records carry neither field, and are classified as they are cited rather than
  in a sweep, so the directory stays mixed for a while.

## Reversibility

Cheap while the fields are declarations nothing reads mechanically. What is not cheap is a decision
moved down to rustdoc and later wanted back as a record: its reasoning has to be rebuilt from the
code, because the ADR that would have held it was never written.

## Confidence

Medium (70%). What would change it: finding that module notes are where decisions go to be
forgotten, since nobody greps rustdoc. What would prove it wrong: a promotion that turns out
expensive, which would mean the asymmetry this rests on runs the other way.

## Revisions

- 2026-09-21 — recorded.

## More Information

- [ADR-0027](0027-control-token-cost-through-session-discipline.md), which measured what a file read
  on every call costs.
- Observed 2026-09-21, and why the maintainer note at the head of `CLAUDE.md` moves to
  `CONTRIBUTING.md` instead of being deleted: the session's context carried neither that comment nor
  the constitution's Sync Impact Report. Both were history for a person, kept where only an agent
  looks.
