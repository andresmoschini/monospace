---
status: accepted
date: 2026-09-07
decision-makers: Andrés Moschini
---

# Ask the cell whether it is decided, and let `stamp` skip a decided target

## Context and Problem Statement

The second stamp mode, `Below`, writes only the sides its target has left `Unset`. A cell whose four
arms are all decided has none of those, so stamping `Below` onto it cannot change it.
[`docs/model.md`](../model.md) names that as the reason to stamp front to back in the first place:
going that way lets a caller stop early on cells that can no longer change, while going back to
front rewrites every cell once per figure. None of it is usable unless something can answer whether
a cell is decided.

Where that question is asked from is not obvious. `stamp` needs the answer about a cell it is
already holding. A caller walking a figure would ask about a position instead. A position is not a
cell: it may hold nothing, or fall outside the window entirely, and a predicate that takes one has
to say something for both.

The second half of the question is whether `stamp` should act on the answer. Skipping the merge for
a decided target produces exactly the buffer the merge would have produced, so nothing observable
separates the two. That makes it a requirement no test can fail for, and one a later reader can
delete as a redundant branch with every check still green.

## Decision Drivers

- The model names answering cheaply whether a cell is already decided as what makes stamping front
  to back worth choosing, under _Stamping_.
- Public surface that no rule requires is a promise with no reader — the driver
  [ADR-0011](0011-expose-cell-for-testing-stamping.md) had to accept a cost against.
- A predicate should report a fact rather than invent one. A position outside the window has no cell
  to report about.
- Whatever is settled about the skip has to survive without a test, because there is no test it
  could have.

## Considered Options

- **A** — No predicate and no skip: `Below` merges every target, decided or not.
- **B** — `Cell::is_decided`, public, and `stamp` skipping a decided target under `Below`.
- **C** — `Buffer::is_decided(at)` as well, or instead: the same question asked of a position.
- **D** — A private helper in the buffer module, with no public predicate.

## Decision Outcome

Chosen option: **B, ask the cell**, because being decided is a property of a cell and of nothing
else, and the only caller that needs the answer today is holding the cell when it needs it.

`Buffer::is_decided(at)` is rejected rather than postponed, and the reason is worth keeping. A
position with no cell is not decided in any useful sense, since it can still be defined. A position
outside the window can never change at all, which is the opposite, and yet there is nothing there to
ask about. The two answers turn out to be indistinguishable anyway: `stamp` already ignores
positions outside the window, so a caller that skips them and a caller that stamps them produce the
same buffer. An answer nobody can check does not belong in a public signature. The method is one
line away on the day a caller can say what it should mean.

`stamp` does not go through a position either. It holds the target cell by the time it needs the
answer, so it asks the cell; going through the buffer would repeat the lookup it has just done.

### Consequences

- Good, because the predicate states something true about a cell regardless of any buffer, window or
  position, so no case analysis rides on it.
- Good, because `stamp` is a caller inside the crate rather than a test. That is what ADR-0011 could
  not get for `cell`, and it is the difference between surface a rule needs and surface only a test
  cashes.
- Good, because `Below` onto a decided cell compares four arms instead of building a cell and
  storing it. No claim is made here that the difference is measurable; there is no workload to
  measure it against.
- Bad, because the skip is a branch no test can fail for. Delete it and every test still passes and
  every buffer is identical.
- Bad, because the predicate is public for the sake of a layer that does not exist yet. One caller
  in the crate justifies its existence, not its visibility.
- Neutral, because that layer will probably want the positional query after all. Adding it is one
  method, and this record is what it has to argue with first.

### Confirmation

The predicate is confirmed by the behavior rules of
[spec 0003](../specs/0003-stamp-below-what-is-there.md): a cell with no `Unset` arm is decided, one
with an `Unset` arm is not, and stamping `Below` onto a decided cell leaves it unchanged.

**Nothing confirms the skip.** No test can fail for it and the quality gate has no opinion about it.
It is held up by a comment at the branch saying why the branch is there, by an acceptance item in
spec 0003 that is read rather than run, and by review. A benchmark is the only thing that would ever
enforce it, and that needs a workload which does not exist — the same one the buffer's storage
question is waiting for.

## Pros and Cons of the Options

### A — No predicate and no skip

- Good, because there is no public surface to justify and no branch to protect.
- Bad, because the sentence the model uses to justify `Below` at all becomes something the library
  cannot express.
- Bad, because the layer above has no way to stop early, which is the only reason to prefer stamping
  front to back.

### B — Ask the cell, and skip

- Good, because the question is asked of the thing that can answer it.
- Good, because the first caller is in the crate, not in the tests.
- Bad, because it accepts a branch that nothing verifies.

### C — A positional query on the buffer

- Good, because a caller walking a region asks in the terms it already has.
- Bad, because it has to answer for a position with no cell and for a position outside the window,
  and the second answer is unobservable, so the signature would carry a decision nobody can check.
- Bad, because `stamp` would repeat a lookup in order to use it.

### D — A private helper

- Good, because the skip works and nothing is promised publicly.
- Bad, because the layer the model wrote that sentence for gets nothing.
- Neutral, because it is what B degrades to if the predicate turns out to be surface nobody wanted.

## Reversibility

Removing the skip is free: no test changes, no buffer changes, nothing outside the branch knows it
existed. Removing the predicate is a breaking change like any other in a library with no external
consumers, which is to say not much of one.

What is not free is the claim the location makes — that being decided belongs to cells rather than
to positions. Every later caller that walks a region will be written against it, and moving the
question to positions afterwards means answering, in public, what it means for a position that holds
nothing.

## Confidence

High (85%) for where the predicate lives. Medium (60%) for the skip being worth a branch nothing can
verify.

What would change the first: a caller that walks regions, finds asking cell by cell awkward, and can
say what a positional query should answer outside the window. That last part is what is missing
today, not the method.

What would change the second: a profile showing the merge costs nothing worth avoiding, or a second
unverifiable branch arriving for the same reason — one is a considered cost, two is a habit.

## More Information

- The sentence this rests on: [`docs/model.md`](../model.md), under _Stamping_.
- [ADR-0008](0008-compose-overlapping-cells-with-three-state-arms.md), which introduced the two
  modes and the three-state arms that make "decided" mean anything.
- [ADR-0011](0011-expose-cell-for-testing-stamping.md), for the driver about public surface and for
  the case that had to accept the cost this one avoids.
- The slice that implements it: [spec 0003](../specs/0003-stamp-below-what-is-there.md).
