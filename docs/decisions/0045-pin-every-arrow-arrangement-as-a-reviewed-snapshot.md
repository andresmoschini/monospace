---
status: "accepted"
date: 2026-09-16
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Pin every arrow arrangement as a reviewed snapshot

## Context and Problem Statement

[Feature 099](../../specs/099-bug-with-arrows/spec.md) repairs an arrow whose picture depends on
which endpoint is written first. The defect is not one arrangement: measured over the grid the spec
defines — two anchors inside a six-by-five field of positions, four leaving directions each, the
coincident pair excluded — 495 of the 928 arrangements draw a different picture when their two
endpoints are exchanged, and 110 of the 1856 renderings draw a route cell over one of the two heads.

Ten of those renderings are pinned today, one assertion each, as feature 039's acceptance scenarios.
That is what let a defect this wide sit unnoticed: each of the ten is correct, and none of them is
one of the broken cases. Pinning ten more by hand would leave the same hole in a smaller shape.

[ADR-0044](0044-let-the-endpoint-order-break-a-tied-route.md) already committed this feature to "a
sweep over the whole grid of arrangements ships as a reviewed snapshot alongside it". What it did
not settle is how a snapshot of 1856 pictures is written down, reviewed and kept — and that needs
deciding before a test depends on it, because the answer brings the repository its first
snapshot-testing dependency and its first artifact that is read by review rather than by assertion.

## Decision Drivers

- [Principle IV](../../.specify/memory/constitution.md#iv-claims-are-measured-not-assumed): the rule
  is prose in the model, and nothing mechanical can tell a correct picture from an incorrect one.
  What a snapshot buys is not correctness but a diff: a later change that moves a picture has to be
  looked at instead of passing silently.
- [Principle II](../../.specify/memory/constitution.md#ii-demonstrable-increments): a snapshot is
  reviewed once, at the moment it is written. A snapshot accepted without reading is worse than no
  snapshot, because it turns an unexamined output into a pinned expectation.
- The [dependencies constraint](../../.specify/memory/constitution.md#constraints-and-dependencies):
  infrastructure concerns prefer idiomatic, well-established crates over reinvention, and a
  dependency is not added without asking. This one was asked and granted.
- The gate reads everything Git tracks. `editorconfig-checker` runs over tracked files with
  `trim_trailing_whitespace` on, and a rendered picture is padded to the window's width, so a
  snapshot written as raw renderings fails the gate on its own trailing blanks.

## Considered Options

- **A** — A hand-rolled fixture: a text file in the repository, a test that regenerates the sweep
  and compares, and an environment variable that rewrites the file.
- **B** — `insta` as a dev-dependency, with `cargo insta review` as the way a diff is accepted.
- **C** — No snapshot: pin a larger but still hand-picked set of arrangements as ordinary
  assertions.

## Decision Outcome

Chosen option: **B**, because a snapshot is only worth having if reviewing its diff is easy enough
that nobody accepts one unread, and that review workflow is the whole of what the crate provides.

The snapshot covers every one of the 1856 renderings the spec's SC-001 names, each labeled with the
arrangement that produced it, and each picture written with its trailing blanks trimmed so the gate
can read the file. Version 1.48.0 is the one taken; it was published on 2026-06-11, more than three
months before this record, which satisfies the seven-day rule the constraints impose.

### Consequences

- Good, because the whole grid is checked instead of a sample, and a change that moves any picture
  shows up as a diff rather than as silence.
- Good, because accepting a diff is a deliberate act with a tool behind it. A hand-rolled fixture's
  "rewrite the file" escape hatch is one environment variable away from being used without looking.
- Good, because the snapshot is generated, so the spec's named acceptance scenarios stay separate
  and keep their pictures readable — which is what feature 099's testing expectations ask for.
- Bad, because it is a dependency the repository did not have, and a second way of writing a test.
  Someone reading the core's tests now finds two idioms, and the rule for which to use is prose.
- Bad, because a reviewed-once artifact decays. The review that matters happened when the file was
  written; a later reviewer sees a diff against a baseline they did not read.
- Bad, because a file of 1856 pictures is not something anyone reads end to end. Its value is
  entirely in the diff, and the first review is the only one that sees the whole.
- Neutral, because nothing outside the tests depends on it. `insta` is a dev-dependency, so it does
  not reach the library, and the gate's `wasm` step does not build it.

### Confirmation

The gate's `test` step runs the snapshot like any other test: a picture that moves fails
`cargo xtask check`. That the file itself is legible to the rest of the gate is confirmed by
`editorconfig-checker` and `cspell`, which read every tracked file and therefore read this one.

What is not mechanical is the first review. That the 1856 pictures were read against _The route of
an arrow_ before being pinned is a claim this record makes and only a person can keep.

## Pros and Cons of the Options

### A — A hand-rolled fixture

- Good, because it adds no dependency, and the whole mechanism is a dozen lines anyone can read.
- Good, because the file format is ours, so the trailing-blank problem is solved by choosing not to
  write trailing blanks.
- Bad, because the accept-a-diff step is a bare environment variable. Nothing about running it
  invites reading what changed, and the one habit this whole arrangement depends on is reading what
  changed.
- Bad, because it reinvents infrastructure, which the constraints say to prefer a crate for.

### B — `insta`

- Good, for the reasons in the outcome.
- Bad, because of the costs in the outcome.

### C — More hand-written assertions

- Good, because it needs no decision, no dependency and no new idiom.
- Bad, because it is what is already in place and is what missed this. Ten pinned pictures, all
  correct, sat beside 495 order-dependent arrangements.
- Bad, because the count that would actually cover the grid cannot be written or read by hand.

## Reversibility

Cheap, and stays cheap. The snapshot is generated from a loop over the public API; removing the
crate means writing that loop's output somewhere else and comparing it by hand. No library code, no
public type and no file outside the core's tests depends on it.

What does not come back is the review. Deleting the snapshot throws away the one reading of the
whole grid that this feature pays for, and the next one costs the same as the first.

## Confidence

High (85%).

What would change it: a gate step that reads the snapshot file badly enough to need a format fight
every time it is regenerated. The trailing-blank case is known and handled; another one would be an
argument for owning the format.

What would prove it wrong: a diff accepted without being read. That is the failure this option is
chosen to make unlikely, and the first time it happens the option has not delivered what it was
picked for.

## More Information

- [ADR-0044](0044-let-the-endpoint-order-break-a-tied-route.md) — where the sweep was promised, and
  the rule the pictures are reviewed against.
- [The model](../model.md), _The route of an arrow_ — the prose the review reads each picture
  against.
- [Feature 099](../../specs/099-bug-with-arrows/spec.md) — SC-001, which this is the standing check
  behind.
