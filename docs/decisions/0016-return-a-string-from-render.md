---
status: accepted
date: 2026-09-07
decision-makers: Andrés Moschini
---

# Return a `String` from `render`

## Context and Problem Statement

Rendering turns a buffer into text, and the text has to reach the caller somehow. Returning it
builds the whole diagram in memory before anyone sees a character of it.

The brief, under _Scope_, warns against the consuming layer's assumptions reaching the core, and a
`String` return is arguably one of them: it is the shape a command-line program wants, and the core
has more consumers coming.

## Decision Drivers

- The core must not carry assumptions that belong to one consumer.
- An abstraction with one implementation and one caller is a guess about the second one.

## Considered Options

- **A** — Return a `String`.
- **B** — Write into a `fmt::Write`, leaving the destination to the caller.

## Decision Outcome

Chosen option: **A, return a `String`**, because the abstraction costs more than it buys with a
single consumer, and the second consumer — the interactive application of phase 3 — is the one that
will say whether it is needed.

### Consequences

- Good, because the signature says what it does and the only consumer there is wants exactly that.
- Bad, because a caller that wants to stream, or to render into a buffer it already owns, cannot,
  and the whole diagram exists in memory whether or not anyone needs it whole.
- Neutral, because the tension with the brief is recorded rather than resolved: this is the kind of
  assumption it warns about, accepted knowingly and with a named trigger.

### Confirmation

None yet. The trigger is the second consumer: when the interactive application renders, either it
wants a `String` too — and the decision is confirmed — or it does not, and this record is what
explains why it was left this way.

## Pros and Cons of the Options

### A — Return a `String`

- Good, because it is the simplest thing that works, and the CLI uses it unchanged.
- Bad, because it decides allocation on the caller's behalf.

### B — Write into a `fmt::Write`

- Good, because the caller chooses the destination, and a large diagram need not exist whole.
- Bad, because it is a generic parameter and a lifetime in the signature of the most-used function
  in the library, bought for a consumer that does not exist yet.

## Reversibility

Cheap, and additive. A function that writes into a sink can be added beside this one later, and this
one can be written in terms of it, without changing what any caller does today.

## Confidence

Medium (65%), lower than the others here because the brief argues against it and the counterargument
is only that the second consumer has not arrived.

What would change it: the interactive application wanting to render into a surface it already owns,
which is a fair guess about what an interactive application does.

## More Information

- Extracted from [spec 0001](../specs/0001-stamp-cells-and-render-them.md), which recorded it as an
  open question and then as a decision, inline, before this record existed.
- The warning this record argues with is now
  [The core stays portable](../../.specify/memory/constitution.md#vii-the-core-stays-portable). It
  was the brief's _Scope_ section when this was written.
