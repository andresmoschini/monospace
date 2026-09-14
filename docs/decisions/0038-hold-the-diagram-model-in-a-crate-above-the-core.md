---
status: "accepted"
date: 2026-09-14
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# Hold the diagram model in a crate above the core

## Context and Problem Statement

Everything this repository has built so far is drawn and discarded. A shape is a value constructed
where it is used, drawn into a surface and dropped
([ADR-0030](0030-drop-extent-until-a-caller-needs-it.md),
[ADR-0031](0031-a-shape-draws-into-a-surface.md)); a buffer is filled once and rendered; the
command-line application assembles its own list of shapes on every run. Nothing above the buffer
holds a figure after it has been drawn, which means nothing can be moved, removed, or asked about.
[The model](../model.md) records that gap as an open question — _How does the core expose mutable
state for editing?_ — and says the answer is model prose rather than an implementation detail.

Issue #62 is the first thing that needs the answer. It asks for a diagram that is the source of
truth: shapes in an order, each with an identity, the buffer rebuilt from the diagram rather than
edited in place. That is a second model, sitting on top of the one that exists, and it has to live
somewhere.

The question this record answers is only _where_. What that model is made of is
[ADR-0039](0039-a-diagram-shape-is-its-own-entity.md).

## Decision Drivers

- [Principle VII](../../.specify/memory/constitution.md#vii-the-core-stays-portable): the core's
  public API is the expensive part to change, and every public item is a promise the WebAssembly
  boundary and every later phase inherit. Whatever the diagram needs from the core is a commitment.
- [ADR-0036](0036-hold-every-table-but-light-outside-the-core.md) has already put a crate above the
  core once, for exactly the reason that a dependent sees the public API and nothing else. Its
  result is evidence, and the same instrument works here.
- The constitution's _In scope for this phase_ names three crates and says that a further crate is a
  widening to be renegotiated rather than assumed. This decision is that renegotiation.
- Phase 3 edits a diagram interactively. Whatever holds the model now is what the interactive
  application will depend on, and it will not be the command-line one.

## Considered Options

- **A** — A module inside `monospace-core`.
- **B** — A new crate, `monospace-diagram`, depending on the core with no privilege the core does
  not give every dependent.
- **C** — Inside `monospace-cli`, the only consumer that exists today.

## Decision Outcome

Chosen option: **B**, because the core is the layer that draws and the diagram is the layer that
remembers, and a crate boundary is the only thing that keeps the second from reaching into the
first.

The name is `monospace-diagram`. `monospace` stays reserved for the interactive application of phase
3, as principle VII requires.

The constitution's _In scope for this phase_ is amended in the same increment to name the fourth
crate, which is what its own text asks for: a crate proposed for a reason other than the one it
records is a widening of that section, to be renegotiated rather than assumed. That amendment is
part of executing this decision, not part of taking it.

### Consequences

- Good, because the core keeps drawing and nothing else. A `Diagram` in the core would be a mutable,
  long-lived value inside a crate whose every other type is constructed, used and dropped, and it
  would be the first thing there with a lifecycle.
- Good, because the boundary is enforced by the compiler rather than by review. A crate cannot reach
  a private item, so "the diagram is no more privileged than any other dependent" needs nobody to
  check it — the same mechanical proof ADR-0036 relies on.
- Good, because the command-line application and the phase 3 application depend on the same model
  instead of one of them owning it. Option C would have made the interactive application either
  depend on a command-line crate or reimplement the model.
- Bad, because the workspace grows a fourth crate, and somebody who wants to draw a diagram now
  takes `monospace-core` and `monospace-diagram`, or three crates if they also want a glyph table
  that is not Light.
- Bad, because the split has to be explained every time someone meets it, exactly as ADR-0036's
  does. This record is that explanation, and the two boundaries have different reasons: the glyph
  crate is outside to prove nothing privileges it, the diagram crate is outside to keep a lifecycle
  out of the core.
- Bad, because it costs a constitution amendment, and an amendment for each crate does not scale. If
  a fifth is proposed, the honest question will be whether _In scope for this phase_ should name
  crates at all rather than whether that crate belongs.
- Neutral, because nothing here says the core's public API is finished. If the diagram turns out to
  need something the core does not expose, that is a change to the core made deliberately, with this
  boundary making it visible instead of letting it happen inside one crate.

### Confirmation

Mechanical, in two parts, and neither needs a reviewer.

The new crate compiles against `monospace-core` as an ordinary dependent, which is what proves it
used no private item. And the workspace build refuses a cycle, so a core that started depending on
the crate that depends on it would not compile at all.

The visible evidence is the command-line application: once it builds a `Diagram` and asks it to
draw, the model is exercised by something that runs rather than by tests alone.

## Pros and Cons of the Options

### A — A module inside `monospace-core`

- Good, because one dependency draws and holds a diagram, with no boundary to explain and no
  amendment to make.
- Good, because the model and the shapes it draws through sit next to each other, so a change that
  spans both is one crate.
- Bad, because it puts the first long-lived mutable value into the crate principle VII asks to keep
  smallest and most portable, and every public item it grows is inherited by the WebAssembly build
  and by every later phase.
- Bad, because nothing then stops the model from reaching into the core's private items, and the
  layering would survive only as long as everyone remembered it.

### B — A crate above the core

- Good, because the layering is the compiler's to enforce, and the core stays the part that changes
  least.
- Good, because it reuses an arrangement this repository has already taken and can compare against.
- Bad, because of the fourth crate, the amendment, and two dependencies where there was one.

### C — Inside `monospace-cli`

- Good, because it is free: no crate, no amendment, and the only consumer today is right there.
- Bad, because it puts domain logic in the crate the constitution says holds none. Principle VII is
  explicit: `monospace-cli` holds no domain logic.
- Bad, because the phase 3 application would have to depend on a command-line crate to get the
  model, or copy it.

## Reversibility

Cheap in one direction and not in the other, and it is the same asymmetry ADR-0036 recorded.

Folding the crate back into the core is a move of files and a deletion of a manifest: nothing about
the model changes, because the model would have been written against the core's public API and that
API does not go away.

Going the other way — discovering later that the model needs something only the core can reach — is
the expensive move, and taking the boundary now is what makes that discovery happen while there is
one consumer and nothing outside the repository depends on anything.

What is permanent from the moment it lands is small: the crate name.

## Confidence

High (85%).

What would change it: the model turning out to need repeated changes to the core's public API to do
ordinary things. Two or three slices of this goal are enough to see that; if each one widens the
core, the boundary is in the wrong place and option A was right.

What would prove it wrong: a type that has to exist on both sides. If the diagram and the core end
up needing to share a mutable value, the split is fictional and the crates are one crate with a
manifest between them.

What would not change it: the ceremony of the amendment. That was weighed and accepted, and the
constitution asks for it by name.

## More Information

- [ADR-0036](0036-hold-every-table-but-light-outside-the-core.md) — the first crate above the core,
  and the argument that a dependent with no privilege is what makes a claim evidence.
- [ADR-0039](0039-a-diagram-shape-is-its-own-entity.md) — what the model in this crate is made of.
- [The model](../model.md), _Open questions_ — "How does the core expose mutable state for
  editing?", the question this record begins to answer, and
  [the diagram model](../diagram-model.md), which answers the rest of it.
- [Issue #62](https://github.com/andresmoschini/monospace/issues/62) — the goal this serves.
