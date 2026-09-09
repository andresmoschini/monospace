---
status: accepted
date: 2026-09-04
decision-makers: Andrés Moschini
---

# Build the non-interactive CLI before the interactive TUI

## Context and Problem Statement

The project grows in five phases: `monospace-core`, the library; `monospace-cli`, a minimal
non-interactive consumer of it; `monospace`, the interactive TUI; a WebAssembly library compiled
from the core; and a web application built on that. Phases 1 and 2 are the current focus, phase 3 is
next up, and phases 4 and 5 have not started.

Phases 2 and 3 are both consumers of the same library, and only one of them can be first. The core's
public API is being designed as it is written, with no consumer outside this repository to say
whether it is usable, so the first consumer built is the first evidence anyone has about it.
Evidence from a batch consumer and evidence from an interactive one are not the same kind of thing:
a TUI brings an event loop, a screen buffer, input handling and a redraw strategy of its own, and an
API that is awkward to call can be wrapped inside that machinery without anyone noticing that it was
awkward.

Nothing breaks if the question is left unanswered, since the phases were listed in order from the
start. What is at stake is that the order is the point. Taken deliberately it is an experiment about
API design; taken by default it is a preference for whichever consumer looked more interesting that
week, and it teaches nothing.

**This record is retroactive, and that needs saying.**
[The rules in this directory](README.md#rules) warn that a record reconstructed later summarizes
what happened instead of recording the reasoning, because by then the rejected options have been
forgotten. This record is written on 2026-09-08, and its `date` is the day the decision was taken.
It is legitimate anyway for one narrow reason: it is a transcription, not a reconstruction. The
argument — including the cost that rejects the alternative — has been in the repository as prose
since the initial commit, in `docs/brief.md` under "Future phases", commit `ae12636` of 2026-09-04,
and moved from there unchanged in substance to `docs/roadmap.md` under "Why the CLI comes before the
TUI". Nothing below is recalled from memory. What is new is the format: the roadmap holds direction
and no rules and is about to stop existing, and this reasoning is the part of it with no other home.

## Decision Drivers

- [The core stays portable](../../.specify/memory/constitution.md#vii-the-core-stays-portable): the
  core's public API may assume no CLI, no TUI and no terminal. The gate's `wasm` step enforces the
  half of that a compiler can see. Which consumer comes first decides the other half — whether the
  API's ergonomics are shaped by a consumer that adds almost nothing, or by one that adds an event
  loop.
- [Process over product](../../.specify/memory/constitution.md#i-process-over-product-non-negotiable):
  idiomatic Rust architecture is what this repository exists to practice, and finding out whether an
  API is pleasant to call means calling it from somewhere that cannot compensate for it.
- [Demonstrable increments](../../.specify/memory/constitution.md#ii-demonstrable-increments): every
  commit leaves `cargo run -p monospace-cli` producing output. A non-interactive consumer's output
  is a diagram on stdout, which can be pasted, diffed and asserted on. A TUI's output is a screen,
  demonstrated by driving it by hand.
- Phases 4 and 5 reuse the core as well, and neither has a terminal at all. Between the two
  candidates for first consumer, the one that assumes least is the one closest to what they will
  need.

## Considered Options

- **A** — The non-interactive CLI first: phase 2 consumes the core with as little ceremony as
  possible, and the interactive TUI follows as phase 3.
- **B** — The TUI first: `monospace` becomes the core's first consumer, and a non-interactive entry
  point is added afterwards if it is still wanted.

## Decision Outcome

Chosen option: **A**, because an API that is awkward to call shows it in a consumer that adds
nothing, and hides it in one that adds an event loop.

### Consequences

- Good, because the CLI has nothing to hide behind. Whatever ceremony it takes to get from an input
  to a rendered diagram is visible in one `main.rs` and cannot be attributed to anything else.
- Good, because the first consumer is also the first end-to-end test surface: its result is a
  string, so it can be compared against an expected diagram. A screen cannot.
- Bad, because the API is shaped first against a consumer that builds a diagram and renders it once.
  Nothing in the CLI pulls on editing something already stamped, on selection, or on redrawing after
  a change — which is precisely what phase 3 needs, and why the question "How does the core expose
  mutable state for editing?" in [the model's open questions](../model.md) stays open longer than it
  otherwise would. The evidence this ordering buys is real and narrow, and the narrowness is worth
  naming.
- Bad, because the consumer with the most to say about what the domain actually needs is the last to
  say it. Any reshaping the TUI forces arrives after the CLI has been specified and built against
  the older shape.
- Neutral, because this decision does not keep the core portable — the `wasm` step does that, and
  would do it whichever consumer came first. What the ordering buys is ergonomic evidence, not a
  boundary.

### Confirmation

Enforced by review, and only partly visible in the gate.

What the gate shows: `cargo xtask check`'s `wasm` step runs
`cargo check -p monospace-core --target wasm32-unknown-unknown`, so a terminal assumption reaching
the core fails the build. And the workspace members are `crates/monospace-cli`,
`crates/monospace-core` and `xtask`, with no TUI crate among them.

What nothing checks is the ordering itself. "In scope for this phase" and "Out of scope for this
phase" in the constitution are what keep phase 3 from starting early, and both are read rather than
run.

The claim this record rests on — that a non-interactive consumer surfaces API awkwardness a TUI
would hide — is not measured, and it cannot be: only one of the two orderings will ever be observed.
It is a design judgment, named as one here rather than described as tested.

## Pros and Cons of the Options

### A — The non-interactive CLI first

- Good, because almost nothing sits between the caller and the library, so what is clumsy stays
  clumsy in plain sight.
- Good, because its output is text, which tests can assert on and a reader can paste into a report.
- Bad, because it exercises one direction of the API only: build, then render.
- Bad, because it postpones the feedback of the consumer that will ask the hardest questions.

### B — The TUI first

- Good, because the interactive application is where the project is heading, and building it first
  would surface the domain's hardest requirements — editing, selection, incremental redraw — while
  the API is still cheap to change.
- Good, because it removes an intermediate consumer, and one binary reaches a demonstrable state
  sooner than two do.
- Bad, because usability would then be judged through a layer built precisely to smooth difficulty
  away. An awkward call site becomes a private helper inside the TUI, and from outside the
  awkwardness is invisible.
- Bad, because a TUI's own state — the event loop, the frame, the widget tree — pulls on the core to
  accommodate an interactive interface, which is the one thing the core's public API may not do.
  Resisting that pull is hardest when the only consumer is the one exerting it.
- Bad, because a screen is not something a diff can show, so every increment would be demonstrated
  by driving an application by hand instead of by reading its output.

## Reversibility

The mechanics are free; the evidence is not.

Nothing is built on this ordering, so reversing it today costs no more than deciding to start phase
3 instead. Nothing is deleted and nothing is renumbered.

What cannot be undone is the observation. An ordering runs once. The moment the TUI exists, "would
this API have been comfortable to call from somewhere with no machinery?" can no longer be asked,
because the machinery is there and the API has already been shaped with it in the room. That is
permanent from the day phase 3 starts, and it is why the ordering is worth a record even though
changing course costs nothing.

## Confidence

High (~85%). The confidence is in the ordering. The argument behind it is a judgment rather than a
measurement, and "Confirmation" above says why it will stay one.

What would prove it wrong: a CLI so thin that it exercises nothing. If reaching a rendered diagram
is a single call, the consumer demonstrates that the API is small, not that it is usable, and the
evidence this ordering was meant to buy was never collected. The observation to watch for is phase
3's first slice — if the TUI immediately forces a reshape of an API the CLI pronounced fine, the CLI
was measuring the wrong thing.

What would not change it: the cost of building two consumers instead of one. That was weighed and
accepted. Process over product, and the second consumer is the experiment.

## More Information

- [The constitution](../../.specify/memory/constitution.md), principle "The core stays portable",
  which owns the rule this record explains the phasing behind, and "In scope for this phase", which
  is what keeps phase 3 from starting early.
- `docs/roadmap.md`, sections "Why the CLI comes before the TUI" and "Why phases 4 and 5 shape
  today's design" — the prose this record transcribes. That file is removed in the same increment as
  this record; its text is in commit `69ee15c` and its ancestor in `docs/brief.md`, section "Future
  phases", commit `ae12636`.
- [ADR-0001](0001-virtual-cargo-workspace-under-crates.md), which laid the workspace out so that
  each phase is a crate of its own.
- [The model](../model.md), open question "How does the core expose mutable state for editing?",
  which is the part of phase 3 that this ordering leaves unanswered longest.
