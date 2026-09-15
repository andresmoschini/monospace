---
status: "accepted"
date: 2026-09-14
decision-makers: "Andrés Moschini, with Claude Opus 5"
---

# A diagram shape is its own entity, held in a closed set of kinds

## Context and Problem Statement

[ADR-0038](0038-hold-the-diagram-model-in-a-crate-above-the-core.md) puts the diagram model in a
crate above the core. What that model holds is still open, and issue #62 raises the question itself:
the rendering concepts the core calls shapes and the model-level things a diagram contains may not
be the same concept, and if they are not, forcing the two layers to share one abstraction would be
the mistake.

They are not the same concept, and the difference is not subtle. A core shape is a value with no
identity, no lifecycle and no mutable state, which draws itself and answers nothing else about
itself — [ADR-0030](0030-drop-extent-until-a-caller-needs-it.md) removed the one question it used to
answer, and [ADR-0031](0031-a-shape-draws-into-a-surface.md) gave it a surface with no reader so it
could not even inspect what it draws over. A diagram's shape is the opposite in every one of those:
it is stored rather than constructed at the point of use, it is addressed by an identity that
survives being placed, its position may be a reference to another shape rather than a coordinate,
and it has a place in an order that can change.

So the model needs its own entity. What is left to decide is how the set of kinds is held: closed,
so the compiler knows all of them, or open, so anything can be one.

## Decision Drivers

- The editor of phase 3 is the consumer this is for. It resolves a position to a shape, shows what
  that shape is, and offers what can be done to it — all of which are case analyses over the kinds.
- [Principle VII](../../.specify/memory/constitution.md#vii-the-core-stays-portable): the core's
  public API is a promise, and asking a core shape for an identity or an anchor point would widen it
  for a layer the core does not know about.
- [ADR-0036](0036-hold-every-table-but-light-outside-the-core.md) proved that a glyph table can come
  from outside the project. Whether shapes deserve the same openness is a fair question, and the
  answer here is not the same answer.
- The maintainer's standing preference for the representation that makes invalid states
  unrepresentable, paid for in churn when it costs some.

## Considered Options

- **A** — The diagram stores core shapes, `Box<dyn Shape>` or equivalent, wrapped with an identity,
  a position and a place in the order.
- **B** — The diagram defines its own trait — draw, plus the anchor points of
  [ADR-0040](0040-let-each-shape-answer-its-own-anchor-points.md) — and holds trait objects, so
  anything outside the crate can be a diagram shape.
- **C** — The diagram defines a closed set of the kinds it knows, each of which draws by building
  the core shape it corresponds to.

## Decision Outcome

Chosen option: **C**, because every question the diagram layer exists to answer is a question about
which kind a shape is, and a closed set is the only one of the three where the compiler answers it.

A diagram shape is therefore a value of the diagram crate's own type, holding what the model needs —
an identity, a position that may be a reference, and the parameters of its figure — and drawing
itself by constructing the corresponding core shape and letting that shape do the drawing. The core
learns nothing about diagrams. `Surface` and `Layer` stay exactly as ADR-0031 left them, and the
diagram draws through them like any other caller.

This is the second time the two layers are deliberately kept apart rather than unified, and the
first was ADR-0030 removing extent. The pattern is worth naming: the core answers "what do I draw",
the diagram answers "what is this and where does it belong", and a type that tried to answer both
would be answering the second one for callers that only asked the first.

### Consequences

- Good, because adding a kind is a change the compiler propagates. Every place that reasons per kind
  stops compiling until it has an answer for the new one, which is exactly what an editor's case
  analysis needs and what a trait object cannot give.
- Good, because a shape's identity, its reference-based position and its order are held by the type
  that owns them, instead of riding alongside an opaque drawable that knows nothing about them.
- Good, because serializing a diagram later is a question about a known set of kinds rather than
  about arbitrary code. Nothing here commits to a format — that stays where
  [ADR-0035](0035-keep-the-cli-demo-format-out-of-the-model.md) left it — but the closed set is what
  keeps the question answerable.
- Bad, because nobody outside the crate can define a diagram shape. That is the freedom ADR-0036
  went to some trouble to prove for glyph tables, and this record declines it for shapes. The two
  cases differ in what the consumer does with the result: a table is data that only has to answer a
  key, and a shape is something an editor has to reason about by kind.
- Bad, because every new kind touches a type that everything else matches on, so the cost of a kind
  grows with the number of places that reason about kinds. Issue #58, a shape that is a group of
  shapes, and issue #61, a shape drawn as a path, are the next two to pay it.
- Bad, because the same figure now has two representations — the diagram's and the core's — and a
  parameter added to one has to be carried to the other by hand. The compiler catches an omission
  only where the core's constructor requires the parameter.
- Neutral, because the closed set says nothing about how many kinds there are. It can start with
  exactly the three the core draws and grow one slice at a time.

### Confirmation

By review, and by the shape of the code rather than by a check. The mechanical part is narrower than
"the core gains nothing", and the line is worth drawing where it actually falls: what would abandon
this decision is `monospace-core` growing a shape that carries an identity, that holds a place in an
order, or that answers where it is. Those three are what a diagram's shape is, and each of them is a
question about a stored entity rather than about a figure.

Geometry is not one of them. A function that takes a rectangle and a named point and returns a
position asks no shape anything: the caller already holds the rectangle, and the answer is
arithmetic. The core is a reasonable home for it, and a diagram kind that knows its own rectangle
calling it is this decision working rather than failing. Nor is it
[ADR-0040](0040-let-each-shape-answer-its-own-anchor-points.md)'s rejected option arriving by the
back door: nothing asks a shape for its bounds, and a kind with no rectangle to give is unaffected.
ADR-0030 removed `extent` so that nobody could ask a shape what it occupies, which is a different
sentence from one forbidding arithmetic on a rectangle somebody already has.

## Pros and Cons of the Options

### A — Wrap core shapes

- Good, because it reuses everything that already draws, with no second representation of a figure.
- Good, because it is the smallest change: a struct with an identity, a position and a drawable.
- Bad, because an opaque drawable cannot answer what kind it is, so the editor's questions all come
  back as downcasts or as a kind tag carried alongside — which is the closed set, badly.
- Bad, because a shape whose position is a reference cannot be a core shape at all: a core shape is
  constructed with its coordinates, and the reference is not resolved until drawing time.

### B — An open trait in the diagram crate

- Good, because anyone can add a kind without touching the crate, matching what ADR-0036 achieved
  for tables.
- Good, because it keeps the diagram crate small, since the kinds could live anywhere.
- Bad, because every question an editor asks per kind becomes unanswerable in general, and the
  places that need a case analysis get a default arm that silently does nothing for a kind nobody
  anticipated.
- Bad, because it promises extensibility to nobody: there is no outside consumer, and the trait's
  shape would be guessed before a single implementor exists. ADR-0036 had ASCII as a real first
  consumer, and this has none.

### C — A closed set of kinds

- Good, because the compiler enumerates them, which is what the editor, the eventual format and the
  mutation API all need.
- Good, because each kind can hold exactly what it needs, instead of a common shape that fits all of
  them badly.
- Bad, because extending it is a change to the crate, and outside code cannot.

## Reversibility

Moderate, and it becomes the crate's public API the moment anything depends on it.

Going from a closed set to an open trait is additive in principle — the trait is introduced and the
existing kinds implement it — but every consumer that matched per kind has to be given an answer for
the general case, and those are the places that motivated the closed set. That work is proportional
to how much of the editor exists by then, which is the cost that grows.

Going the other way, from open to closed, is worse: it removes something outside code could rely on.
Taking the closed set first is therefore the order that keeps the cheaper reversal available.

## Confidence

Medium-high (75%).

What would change it: a consumer outside this repository with a shape of its own, which is the
situation ADR-0036 was built for and which does not exist here. One would be enough to argue for the
trait.

What would prove it wrong: issues #58 and #61 — a group of shapes, and a shape drawn as a path. If
either turns out to need an abstraction over kinds rather than a kind of its own, the closed set is
the wrong instrument and the trait was the answer all along. A group is the more dangerous of the
two, because a group contains shapes and would want to speak about them generically.

What would not change it: the asymmetry with glyph tables. It is deliberate, and the reason is
written above.

## More Information

- [ADR-0038](0038-hold-the-diagram-model-in-a-crate-above-the-core.md) — the crate this entity lives
  in.
- [ADR-0030](0030-drop-extent-until-a-caller-needs-it.md) — why a core shape answers nothing about
  itself, which is most of why it cannot be a diagram's shape.
- [ADR-0036](0036-hold-every-table-but-light-outside-the-core.md) — the openness this declines, and
  the case where it was worth taking.
- [The diagram model](../diagram-model.md) — the prose this record's outcome is written into.
