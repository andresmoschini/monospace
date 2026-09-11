---
status: accepted
date: 2026-09-11
decision-makers: Andrés Moschini
---

# Hold every glyph table but Light in a crate outside the core

## Context and Problem Statement

[The model](../model.md), under _Strokes, glyph sets and the catalog_, says a set may ship with the
library or be loaded, that neither kind gets special behavior, and that once a set is in a catalog
nothing can tell where it came from. [ADR-0014](0014-collapse-glyph-sets-into-a-catalog.md) is what
makes that true of the representation.

Nothing demonstrates it. `monospace-core` holds one table, Light, as a private constant, and there
is no second table anywhere and no way for anyone to supply one. Every table the project adds would
be a change to the core, and somebody outside the project has no way to add one at all. The claim
that glyph sets are extensible is a sentence in a document with no evidence behind it, and
[principle IV](../../.specify/memory/constitution.md#iv-claims-are-measured-not-assumed) says that
is the kind of claim not to leave standing.

The question is where a table that is not Light lives.

## Decision Drivers

- Principle IV: a claim about extensibility is worth what demonstrates it.
- Principle VII: the core's public API is the expensive part to change, so whatever a second table
  needs from it is a commitment.
- ADR-0014 already decided what a catalog is; this is about where the sets that go into it come
  from.
- ASCII is worth having on its own: it is what a terminal that cannot carry box-drawing characters
  needs, so the demonstration is not a toy.

## Considered Options

- **A** — Put every built-in table in `monospace-core`, alongside Light.
- **B** — Keep Light in the core and hold every other table in a second crate that depends on the
  core and has no privilege the core does not give every dependent.
- **C** — Keep one table in the core and add the tables the model's other route describes: a file
  format the core loads at run time.

## Decision Outcome

Chosen option: **B**, because it is the only one of the three whose result is evidence. A second
crate exercises exactly the path somebody outside this project would take — it sees the core's
public API and nothing else — so if a table can be written there, the extensibility the model claims
is demonstrated rather than asserted. The compiler is what enforces it: a crate cannot reach a
private item, so "no privilege" needs no reviewer to check it.

The core keeps Light. Not because Light is special in the domain — it is not, and ADR-0014 is
explicit that a catalog cannot tell its rules apart — but because a core that ships nothing would
make every consumer take a dependency to draw anything, and because keeping one table there is what
proves the two sources compose: a catalog built from a core table and an outside table answers both,
and nothing downstream can say which crate a rule came from.

ASCII is the table that comes with the crate, since a crate with nothing in it demonstrates nothing.

### Consequences

- Good, because the extensibility claim acquires the only evidence that counts: a table that lives
  where a stranger's table would live, working the same way.
- Good, because it forces the core to grow a way to accept a table from outside, which is the
  commitment the model was already promising and had never priced.
- Good, because "render this in ASCII alone" becomes something to run rather than a sentence in
  ADR-0014.
- Bad, because the workspace grows a third crate, and a consumer who wants anything but Light now
  takes two dependencies where it could have taken one.
- Bad, because the split is arbitrary from the domain's point of view: nothing about Light belongs
  in a core and nothing about ASCII belongs outside it, so the boundary has to be explained every
  time someone meets it. This record is that explanation.
- Bad, because the constitution's _In scope for this phase_ names two crates and this makes three.
  It is not on the _Out of scope_ list, which is the list that stops a plan, but the scope section
  is the maintainer's to read: whether it needs an amendment is raised by feature 054's spec rather
  than settled here.
- Neutral, because option C is not foreclosed. A file the core loads at run time is the model's
  other route and stays open; this decision is about where a table written in Rust lives, not about
  whether a table can also be read from a file.

### Confirmation

Mechanical, and in two parts. The new crate compiles against `monospace-core` as an ordinary
dependent, which is what proves it used no private item. And the gate builds the workspace, so a
core that started depending on the crate that depends on it would not compile at all.

The visible evidence is the command-line application: it builds one catalog out of the core's Light
and the other crate's ASCII and renders both on one canvas.

## Pros and Cons of the Options

### A — Every table in the core

- Good, because one dependency draws anything, and there is no boundary to explain.
- Good, because the tables stay next to the rule that consumes them.
- Bad, because it demonstrates nothing: adding a table stays a change to the core, which is the
  exact thing that made the extensibility claim empty.
- Bad, because the core grows with every style anyone wants, and principle VII asks it to stay the
  part that changes least.

### B — A second crate

- Good, because it is the path an outsider takes, so it is evidence rather than an assertion.
- Good, because the core's extension point gets a real first consumer before it is promised to
  anyone.
- Bad, because a third crate and a boundary whose justification is entirely about proof.

### C — Load tables from a file at run time

- Good, because it is the most open of the three: no recompilation, and no crate to publish.
- Good, because the model already describes it, so it answers the claim as written.
- Bad, because it puts a file format in the core's public API, which is the expensive commitment,
  and [ADR-0035](0035-keep-the-cli-demo-format-out-of-the-model.md) has just declined to do that on
  much weaker grounds.
- Bad, because it proves the wrong thing. A loader demonstrates that the core can read a table; it
  does not demonstrate that a library somebody else writes is no more privileged than the core, and
  that is the claim.

## Reversibility

Cheap in one direction and not the other. Folding the crate back into the core is a move of data and
a deletion: nothing about the tables changes, and the core's extension point stays useful because
the core would use it on itself.

Going the other way once the extension point is public is the expensive move, and it is the one
being taken now rather than later — which is the point of taking it while there is one consumer and
nothing outside the repository depends on anything.

## Confidence

High (85%).

What would change it: a consumer who needs every table and finds two dependencies a real cost, which
would argue for re-exporting rather than for undoing the split. What would prove it wrong: the
extension point turning out to need something only the core can do, which would mean the crate is
privileged after all and the demonstration was never one.

## More Information

- [The model](../model.md), _Strokes, glyph sets and the catalog_ — the claim this record is about.
- [ADR-0014](0014-collapse-glyph-sets-into-a-catalog.md) — what a catalog is, and why a rule's
  origin is invisible once it is in one.
- [ADR-0001](0001-virtual-cargo-workspace-under-crates.md) — the workspace this adds a member to.
- [`glyph-sets.md`](../glyph-sets.md), _ASCII_ — the fifteen rows the new crate carries.
