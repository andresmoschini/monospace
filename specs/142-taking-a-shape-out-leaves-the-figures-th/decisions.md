# Decisions: Taking a shape out leaves the figures that hung from it where they were

**Feature**: #142 | **Written**: 2026-10-01 | **Answered**: 2026-10-02 — all four, the proposal
adopted in each | **Revised**: 2026-10-02 — D4's mechanical consequence, measured and corrected

**The four answers of the first run of this stage are void**, and there is no partial credit in
them. They answered whether to record the question, which this slice no longer asks: the deciding
stage that merged as [#158](https://github.com/andresmoschini/monospace/pull/158) renamed the
issue's desire, declined it for want of a record, and left the question open. The issue's title is
the maintainer's original and correct. `git log` carries the superseded sheet.

Four entries, all domain-level, all the maintainer's, all adopted as proposed. **D4's answer stands;
its mechanical consequence was wrong and is corrected below, dated** — nothing else moved. **None
carries a generated picture**: the subject that renders cannot be reached by a marker, a description
having no field that takes a shape out
([ADR-0064](../../docs/decisions/0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md)),
and the arrangement is drawn once in [spec.md](spec.md)'s **Behavior**.

## D1 — Is the freeze one slice or two

- **Proposal**: one. The measuring work for the rule this replaces sits on an open pull request
  describing the other behavior, and under the freeze every number it measured changes.
- **Altitude**: domain, the maintainer's — what merges, and what that pull request is worth.
- **If this is wrong**: that work merges against the old rule, and the freeze slice re-does three
  pictures and one test.
- **The alternative**: two slices, at the cost of a whole round of the gate for superseded numbers.
- **Answer**: **adopted, and it settles the order too.** This deciding pull request finishes the
  specification; the implementation is a **new** pull request. The building pull request describing
  the opposite behavior is **closed without merging** — its four tests, its gallery block and its
  seventh picture are the three the freeze would have re-measured anyway, and keeping it open beside
  this specification is what made the issue and the documents disagree.

## D2 — Where the freeze is recorded

- **Proposal**: one new ADR for the freeze, plus a dated `## Revisions` line in ADR-0041 for the one
  consequence it contradicts.
- **Altitude**: domain, the maintainer's — the rule is observable from outside the crate.
- **If this is wrong**: a reader of the resolution rule meets two records, or one denying its own
  outcome.
- **The alternative**: supersede ADR-0041, which the constitution's test discourages — the
  resolution rule and the removal rule _can_ be cited apart.
- **Answer**: **adopted, with the maintainer's condition: as small as it can be.** An ADR is
  created, because something has to overrule the behavior of not drawing what hung from a removed
  shape, and **what it records is the rule and the one thing it replaces** — §9's "nothing is
  rewritten" and ADR-0041's "an editor can delete a shape without repairing everything that
  referenced it". **No new vocabulary, no name for the frozen position, and nothing said about
  removals in general.** ADR-0041 itself is revised in place on the same date, since the alternative
  was a second record a reader would have to find beside the first. **Its commitment is `working`,
  deliberately** — that is the cheapest of the three to reverse, since reversing it is a new short
  ADR naming what it replaces rather than a migration. The maintainer's condition and the commitment
  are the same concern answered twice.

## D3 — What a removal freezes

- **Question**: every position holding a reference to the removed identity, or only the ones that
  resolve as it is taken out?
- **Proposal**: only the positions that resolve now. A reference that does not resolve is left as it
  is: still not drawn, and resolving again if the identity comes back. **Hypothetical, no code
  produces either picture:**

  ```text
  the box taken out — the arrow frozen:    a reference never added:
          ┌─┐                                      ┌─┐
     ─────┤ │                                      │ │
          └─┘                                      └─┘
  ```

- **Altitude**: domain, the maintainer's — a caller observes what survives a removal.
- **If this is wrong**: a removal destroys a reference it cannot replace, or freezes one to a point
  never computed.
- **The alternative**: rewrite every reference naming the identity, leaving unresolved ones alone.
- **Answer**: **adopted, and stated in the maintainer's own terms**: a removal freezes **the
  references naming the removed shape and nothing else**. A position pointing at some other shape is
  not touched, and neither is a reference naming an identity that was never there — a case the
  removal did not create, so the rule a caller reads is not the rule the removal changed.

## D4 — Where the rewrite lives

- **Proposal**: inside `remove`, in one pass, before the shape leaves.
- **Altitude**: domain, the maintainer's — a caller observes whether the freeze happens or is asked
  for.
- **If this is wrong**: `remove` keeps its promise and the freeze becomes a sixth change a caller
  must ask for, putting "what hung from it" behind a flag the issue never mentions.
- **The alternative**: a new change beside `remove`, which lets a caller decline the freeze.
- **Answer**: **adopted, and strengthened past the proposal: it must not be possible to remove a
  shape without updating the references to it.** No flag, no separate change, no opt-out, and
  therefore no sixth row in §9's table of five — `remove`'s row and its rustdoc are what change, and
  the rustdoc's "changing nothing else" is what this slice corrects.
- **Revised 2026-10-02, after measurement.** The answer stands; **the mechanical consequence this
  entry named was wrong, and it is corrected here rather than left standing.** It read: _a reference
  can only be resolved while the shape is still held, and `Position::resolve` takes `&Diagram` where
  `remove` holds `&mut self`. The rewrite is therefore **two passes** — collect every resolution,
  then write them._ The first half is right and is what survives: a reference resolves only while
  the figure it names is still held, which is why the rewrite runs before the shape leaves. **The
  second half does not follow from it.** Measured on the building branch,
  `for placed in &mut self.shapes` writing `placed.shape` inside is `error[E0502]` — the mutable
  borrow is live across the call — while `for at in 0..self.shapes.len()` compiles and is **one
  pass**, collecting nothing. What the borrow actually forces is narrower and should have been said
  instead: **a `&mut Shape` cannot be held across a call that wants the diagram.**
- **Where the rewrite lives was the maintainer's call on 2026-10-02, and not a consequence of the
  borrow.** It is **a method on `Shape`** — `Endpoint::frozen_position` and
  `Shape::with_frozen_references`, both crate-private — and `remove` is the five lines that call it,
  because `Diagram` already reaches into every figure it holds and which positions a figure has, and
  what a frozen one is, belongs to the figure. That placement is **module-level and takes no entry
  here**: a caller holds a `Diagram` either way and draws the same picture, so by the constitution's
  own test nothing outside this module observes which of the two it is. All three kinds are matched,
  and the two that cannot hold a reference answer `None` today, so #89 widens two lines of the match
  rather than the method.
