---
status: "accepted"
date: 2026-09-14
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Resolve a position through a reference, and draw nothing that cannot be resolved

## Context and Problem Statement

Issue #62 asks that a shape be able to hold a position that depends on another shape: the identifier
of the referenced shape, the anchor point on it, and a horizontal and vertical offset. The point of
that is movement — moving the referenced shape moves everything hanging from it — which only works
if the dependent position is derived when it is needed rather than stored once as a coordinate.

Deriving it can fail. The referenced shape may not exist, or may have been removed since; it may not
answer the anchor point asked for, which
[ADR-0040](0040-let-each-shape-answer-its-own-anchor-points.md) makes an ordinary answer rather than
an error; and two shapes can reference each other, directly or around a longer loop, so that neither
has a position until the other does.

A model that has failure in it has to say what failure does. That is the decision here: when it is
noticed, and what anyone gets to know about it.

## Decision Drivers

- [The model](../model.md), _Degenerate arrangements_: a shape whose parameters are degenerate or
  describe something impossible never fails — no error, no panic — and what it draws is whatever the
  general rule yields. A diagram that rejected a reference would be introducing the first failure
  into a layer built on a rule that has none.
- A diagram is built and edited in whatever order its user chooses. Pinning a shape to one that has
  not been added yet, and deleting a box that three arrows hang from, are both ordinary moments in
  an editor rather than mistakes to prevent.
- The constitution's removal test, in _One definition of green_: an entry has to be shown to break
  something when taken out. It applies to public API as much as to configuration, and it is the
  argument against shipping a diagnostic query for a consumer that does not exist.
- The maintainer's decision, taken with the alternatives in front of them: what cannot be resolved
  is not drawn, and nothing more is offered.

## Considered Options

- **A** — Reject at mutation time: the diagram refuses a reference it cannot resolve and refuses a
  change that would create a cycle, so an unresolvable state is never stored.
- **B** — Resolve at drawing time: what cannot be resolved is not drawn, silently.
- **C** — B, plus a query that lists what could not be resolved and why.

## Decision Outcome

Chosen option: **B**, because a reference that cannot be resolved yet is a normal state of a diagram
being built, and the only thing the layer owes anybody is not to draw something it cannot place.

A position is therefore either absolute or a reference — a shape's identity, an anchor point on it,
and two offsets — and resolving one means asking the referenced shape for that anchor and adding the
offsets. Resolution is a chain: a reference may resolve to a shape whose own position is a
reference, and the chain ends at an absolute position or at nothing.

Nothing is the answer in three cases, and they are one case as far as the model is concerned:

| The reference names                            | Resolves to |
| ---------------------------------------------- | ----------- |
| a shape the diagram does not hold              | nothing     |
| an anchor point that shape does not answer     | nothing     |
| a shape whose own chain leads back to this one | nothing     |

A shape with no resolved position is not drawn, writes nothing, owns no cell, and answers no anchor
point of its own — so anything hanging from it resolves to nothing in turn. There is no error, no
panic and no report. The rest of the diagram draws normally.

### Consequences

- Good, because the diagram keeps the property the whole model rests on: drawing never fails. The
  layer above shapes inherits _Degenerate arrangements_ instead of becoming the exception to it.
- Good, because an editor can delete a shape without repairing everything that referenced it, and
  can attach to a shape before adding it. Under option A both are errors, and the order in which a
  diagram is assembled would start to matter.
- Good, because the public API stays at what is used. A diagnostic query would be surface added for
  a consumer that does not exist yet, which is the thing the removal test exists to stop.
- Bad, because a diagram that draws nothing and a diagram whose every reference is broken look
  identical, and nobody can ask why. This is the accepted cost, and it is the one the first editor
  will come back for.
- Bad, because a mistyped identity is silent. Today identities are generated rather than written, so
  there is nothing to mistype; the day they become editable, as the issue anticipates, this cost
  grows and option C becomes the obvious answer.
- Bad, because resolution has to be cycle-safe every time it runs, and that is a correctness
  obligation carried by every future kind of reference rather than by one guard at the door.
- Neutral, because option C is additive. Adding the query later changes no behavior: what draws
  keeps drawing, and what is silent gains a voice.

### Confirmation

By test, and the cases are small enough to enumerate: a reference to an absent shape, a reference to
an anchor a kind does not answer, a two-shape cycle, a longer cycle, and a chain several references
deep that does resolve. Each asserts the same two things — the dependent shape is absent from the
output, and everything else is drawn exactly as it would have been.

## Pros and Cons of the Options

### A — Reject at mutation time

- Good, because an unresolvable diagram is unrepresentable, which is normally this project's
  preference.
- Good, because the failure is reported where it was caused, with the caller still holding the
  context that explains it.
- Bad, because it makes the order of construction part of the API: nothing can be attached to a
  shape that does not exist yet, so a diagram has to be assembled dependencies-first.
- Bad, because deleting a shape becomes a cascade — refuse, delete the dependents, or rewrite their
  positions — and every one of those three is a policy the model would have to take a position on.
- Bad, because it introduces the first fallible operation into a layer whose rule is that nothing
  fails.

### B — Resolve at drawing time, silently

- Good, because it matches _Degenerate arrangements_ and keeps every operation infallible.
- Good, because construction and editing are order-independent.
- Bad, because the absence of a figure carries no explanation.

### C — Silent, plus a diagnostic query

- Good, because it keeps everything B has and gives an editor something to show its user.
- Good, because "why is my box not there" is a question somebody will certainly ask.
- Bad, because the shape of the answer would be guessed now, with no editor to guess from, and a
  public query is harder to change than to add.

## Reversibility

Cheap toward C and expensive toward A.

Adding the diagnostic query is additive: nothing that draws today would draw differently, and the
only new thing is a way to ask. Nothing about this decision makes it harder later, which is why it
is the one deliberately postponed.

Moving to A would change every mutation into something that can fail, and every caller written
against infallible operations would have to be revisited. That cost grows with the number of
callers, and the first of them is the command-line application.

## Confidence

High (80%).

What would change it: the first editor. The moment a person is clicking on shapes, "this one did not
draw and here is why" stops being a nicety, and C is where this lands.

What would prove it wrong: a diagram where a broken reference is silently invisible and the user
concludes the tool is broken rather than the diagram. That is the failure this option accepts, and
it will be observed rather than argued.

What would not change it: the argument that invalid states should be unrepresentable. It is the
maintainer's usual preference and it was overruled here on purpose, because "not yet resolvable" is
a legitimate state of a diagram under construction rather than an invalid one.

## More Information

- [ADR-0040](0040-let-each-shape-answer-its-own-anchor-points.md) — why an absent anchor point is an
  ordinary answer, which is one of the three ways a reference fails to resolve.
- [The model](../model.md), _Degenerate arrangements_ — the rule this record extends to the layer
  above shapes.
- [The diagram model](../diagram-model.md), _Positions_ — the prose this outcome is written into.
