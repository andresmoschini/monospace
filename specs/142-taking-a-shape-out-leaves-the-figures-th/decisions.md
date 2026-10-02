# Decisions: Taking a shape out leaves the figures that hung from it where they were

**Feature**: #142 | **Written**: 2026-10-01 | **Answered**: — **pending**

**The four answers of 2026-10-02 are void**: they answered the other question. The deciding stage
renamed the issue's desire, declined it for want of a record, and left the question open; the
issue's title is the maintainer's original and correct, and these four are the questions
**implementing** it forces. `git log` carries the superseded sheet.

Four entries, all domain-level, all the maintainer's. **None carries a generated picture** — the
subject that renders cannot be reached by a marker, a description having no field that takes a shape
out
([ADR-0064](../../docs/decisions/0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md))
— and the arrangement is drawn once in [spec.md](spec.md)'s **Behavior**, which each entry names.

## D1 — Is the freeze one slice or two

- **Proposal**: **one**. The measuring work for the rule this replaces — four contract tests, the
  gallery block, the demonstration's seventh picture — sits on an open pull request describing the
  other behavior, and under the freeze **every number it measured changes**: the seventh picture
  from twenty-two cells gone to twelve, the gallery's block from empty to drawn, the routes test
  from a footprint comparison to a whole-buffer one. Measuring the rule you are about to replace is
  measuring it twice.
- **Altitude**: domain, the maintainer's — what merges, and what that pull request is worth.
- **If this is wrong**: that work merges against the old rule, and the freeze slice re-does three
  pictures and one test.
- **The alternative**: two slices, which is what the pull request assumes, at the cost of a whole
  round of the gate for numbers already superseded.

## D2 — Where the freeze is recorded

- **Proposal**: **one new ADR for the freeze, plus a dated `## Revisions` line in ADR-0041** for the
  one consequence it contradicts — "an editor can delete a shape without repairing everything that
  referenced it". ADR-0041's subject is _resolution_ and "what cannot be resolved is not drawn"
  survives untouched; what it got wrong is a consequence about deletion. The model's five sections
  are amended beside it.
- **Altitude**: domain, the maintainer's — the rule is observable from outside the crate.
- **If this is wrong**: a reader of the resolution rule meets two records, or one denying its own
  outcome.
- **The alternative**: **supersede ADR-0041**, which the constitution's test discourages — the
  resolution rule and the removal rule _can_ be cited apart.

## D3 — What a removal freezes

- **Question**: every position holding a reference to the removed identity, or only the ones that
  resolve as it is taken out? And what of one that does not resolve — an identity **never there**,
  and an anchor the shape **did not answer**?
- **Proposal**: **only the positions that resolve now**, because one with no point to freeze at has
  nothing to become. A reference that does not resolve is left as it is: still not drawn, and
  resolving again if the identity comes back — which leaves ADR-0041's option B intact, nothing
  unresolvable being touched and a removal the only thing that creates the case. **Hypothetical:**

  ```text
  the box taken out — the arrow frozen:    a reference never added:
         ┌─┐                                      ┌─┐
    ├────┤ │                                      │ │
         └─┘                                      └─┘
  ```

- **Altitude**: domain, the maintainer's — a caller observes what survives a removal.
- **If this is wrong**: a removal destroys a reference it cannot replace, or freezes one to a point
  never computed.
- **The alternative**: rewrite every reference naming the identity, leaving unresolved ones alone.
  It differs only for an identity never there, which a removal did not create.

## D4 — Where the rewrite lives

- **Proposal**: **inside `remove`, in one pass, before the shape leaves** — the point a reference
  was resolving to is only knowable while the shape is still there. The rewrite reaches figures
  `remove` never named, which is the sentence §9 loses; `remove`'s rustdoc, which says it changes
  nothing else, is what this slice corrects.
- **Altitude**: domain, the maintainer's — a caller observes whether the freeze happens or is asked
  for.
- **If this is wrong**: `remove` keeps its promise and the freeze becomes a sixth change a caller
  must ask for, putting "what hung from it" behind a flag the issue never mentions.
- **The alternative**: a new change beside `remove`, which lets a caller decline the freeze — at the
  cost of a `remove` that leaves the diagram as the issue calls wrong.
