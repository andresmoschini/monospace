---
status: accepted
scope: domain
commitment: working
date: 2026-09-27
decision-makers: Andrés Moschini
---

# Name a shape for what it joins, not for the head it may carry

## Context and Problem Statement

The shape that joins two endpoints is a route derived between them plus one terminal at each end,
and a terminal is a glyph or an arm. An arm writes no head, so the shape draws as often as a bare
line or an elbow as it does an arrow, and the name it carried named the one case it cannot
guarantee. That name is also the wire tag and the type three crates export, and the layer above the
core already says connector where `model.md` says arrow.

Nothing consumes the tool yet, which is what makes the name cheap today and expensive once a
consumer exists. Issue #57, which makes an arrowhead a chosen style rather than a spelled-out glyph,
is the change that turns the name into a claim the shape cannot keep.

## Decision Drivers

- Principle VI, on altitude: three crates and a file format observe this word, so its home is
  `model.md` and a record, not a module's rustdoc.
- A name two layers disagree about is two vocabularies for one thing, and only one is the model's.

## Decision Outcome

Chosen option: **Connector**, because it names what the shape does — join two endpoints — and
nothing about how it looks, which its terminals decide. `Link` was the alternative and reads as a
hyperlink rather than a figure; `Edge` names a graph the model does not have; `Segment` and `Route`
are already spoken for inside the core.

The rule is the general one: a shape is named for what it does, and what it is made of is named
separately. A shape that can be drawn without a distinguishing feature is not named for the feature
it may or may not carry.

### Consequences

- Good, because the name stops constraining what the shape can be drawn as, and two layers hold one
  word for it.
- Bad, because the places that record what the shape was called then keep the old word, and a reader
  has to know which vocabulary a given document is speaking.
- Neutral, because how a terminal may be spelled in future — a style rather than a literal, a
  direction derived from an attachment — is a decision about terminals, and this record does not
  make it.

## Reversibility

Free today, since nothing outside this repository reads the name. What makes it costly is the first
description written by anyone else, and that cost is paid by migrating it.

## Confidence

High (90%).

What would change it: a shape that joins two things and is named for neither, which would say the
rule is too general rather than that this name is wrong.
