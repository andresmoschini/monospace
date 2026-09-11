---
status: accepted
date: 2026-09-11
decision-makers: Andrés Moschini
---

# Keep the CLI's demo file format out of the model

## Context and Problem Statement

Feature 045 replaces the diagram hardcoded in `monospace-cli` with a JSON file the binary reads, so
that changing what the demo draws no longer means editing `main.rs`.

[The model](../model.md) has an open question that this looks like the answer to: _What minimal
diagram description does the core accept?_ It names its own trigger — "the first slice that has to
read a diagram from outside the process, which is also the first one that makes the choice visible
in the public API" — and the section's rule is that a feature needing one of those answers amends
the model first and then implements the slice. Feature 045 is literally that slice.

What is ambiguous if nothing is recorded: whether this JSON format is the core's diagram description
or a convenience local to one binary. The two differ in what they cost to change later, and a reader
who finds a JSON format in the repository next to an unanswered question in the model has no way to
tell that the two were ever considered together.

## Decision Drivers

- Principle VII: the core stays portable, and its public API is the expensive part to change.
- Principle VI: a decision taken is a decision recorded, including the decision not to decide.
- The model's own instruction to amend it before implementing a slice that answers an open question.
- The maintainer's framing of feature 045: a quick feature, with a better storage model expected
  later.

## Considered Options

- Keep the format in `monospace-cli`, and leave the model's question open.
- Answer the question: amend the model and put the description in the core's public API.
- Keep the format in the CLI, but record it in the model as a draft of the eventual core format.

## Decision Outcome

Chosen option: **keep the format in `monospace-cli`, and leave the model's question open**, because
the format is a demo convenience that was never designed to be the core's description, and adopting
it as one by proximity would settle a model question on no evidence.

`monospace-core` gains nothing in this feature — no type, no function, no way to read or write a
description — and `docs/model.md` is not amended.

### Consequences

- Good, because the core's public API and its WebAssembly boundary are untouched, so the feature
  cannot make the expensive kind of mistake.
- Good, because the model's question keeps waiting for the evidence it asks for: a slice that
  actually needs the core to accept a description, which the CLI demo does not.
- Bad, because there will be two formats for a while once the real one arrives, and the throwaway
  one will have accumulated files by then.
- Bad, because the format is the first thing anyone will read as "how you describe a diagram in
  Monospace", regardless of what this record says.
- Neutral, because the work is not wasted either way: what the format turns out to need is evidence
  for the eventual answer, just not a commitment to it.

### Confirmation

By review, plus one mechanical signal: if `monospace-core` gains a dependency on a serialization
crate, or its public API grows a description type, this decision has stopped being followed. Feature
045's spec states it as FR-019, and `docs/model.md` staying unamended by that feature is the visible
evidence.

## Pros and Cons of the Options

### Keep the format in the CLI, question open

- Good, because it is confined to one binary and costs nothing to throw away.
- Good, because it keeps the decision for when there is something to decide it with.
- Bad, because "provisional" written in a document does not stop a format from being depended on.

### Answer the question in the core

- Good, because there would be one format rather than two, and the model would be one question
  shorter.
- Bad, because the format would be designed for a demo and then promised to every future consumer,
  which is the reverse of how the model says the question should be settled.
- Bad, because it turns a quick feature into a model amendment plus a public API commitment.

### Record it as a draft in the model

- Good, because the connection between the format and the open question would be visible in the
  model rather than only here.
- Bad, because a draft in a document that owns design intent reads as settled however it is
  labelled, which is the confusion this record exists to prevent.

## Reversibility

Cheap now: the format lives in one binary, has one file written against it, and nothing outside the
repository consumes it. The cost grows with the number of description files anyone writes, and it
becomes real the moment a second consumer reads the format or someone outside the repository does.

Note the asymmetry — this is the cheap direction. Adopting the format into the core later is an
addition; retracting it from the core's public API would not have been.

## Confidence

High (85%).

What would change it: a slice that needs the core itself to read a description, which is exactly the
evidence the model asks for. What would prove it wrong: the CLI format turning out to be adequate,
unchanged, through several features — at which point declining to adopt it will have been caution
that cost two formats for nothing.

## More Information

- [The model](../model.md), _Open questions_ — the question this record declines to answer.
- [Feature 045's spec](../../specs/045-simplify-cli-to-demo-shapes/spec.md), FR-019 and FR-020.
- [ADR-0022](0022-non-interactive-cli-before-the-tui.md) — the CLI exists to produce evidence about
  the core's API, which is the same reason its demo format is not allowed to become that API by
  default.
