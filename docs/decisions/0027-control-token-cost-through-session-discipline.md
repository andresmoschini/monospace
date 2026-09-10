---
status: accepted
date: 2026-09-09
decision-makers: Andrés Moschini
---

# Control token cost through session discipline, not by trimming artifacts

## Context and Problem Statement

Working this way costs noticeably more per unit of shipped code than working without it, and the
suspicion was that the cause was configuration: the constitution is imported into `CLAUDE.md`, so
every session carries it, and Spec Kit writes several documents per feature. Both are visible, both
are ours to change, and neither had ever been measured.

Measuring them changed the question. Across the sessions this project has produced so far — roughly
4,250 model calls — the average context per call is about 264k tokens, while the first call of a
session costs between 38k and 66k. Around 80% of a typical call is therefore accumulated
conversation, not configuration. The longest session reached 755k tokens of context on its last
call, having started at 55k.

That distribution matters because every call re-reads the entire context. A session's cost is not
proportional to its length but to its length squared: fitting `base x n + g x n^2 / 2` to the
longest session reproduces its measured total within 10%. The consequence is that the levers are not
equally sized, and the two that were suspected are the small ones.

## Decision Drivers

- [Claims are measured, not assumed](../../.specify/memory/constitution.md#iv-claims-are-measured-not-assumed):
  the hypothesis that started this was falsified by the first measurement, and the ADR exists to
  keep that from being re-derived later.
- [Process over product](../../.specify/memory/constitution.md#i-process-over-product-non-negotiable):
  the volume of specification artifacts is the deliberate price of this repository's purpose. It is
  a cost to know, not a cost to eliminate.
- Whatever is chosen has to survive the project growing. A lever that works only while the codebase
  is small is not a decision, it is a coincidence.

## Considered Options

- A — Trim what Spec Kit writes: make `research.md`, `data-model.md`, `quickstart.md` and
  `contracts/` conditional on the slice earning them.
- B — Restructure the documents that are read repeatedly, splitting `docs/learning-log.md` into one
  file per increment.
- C — Shrink what is always in context, starting by dropping the constitution import from
  `CLAUDE.md`.
- D — Change how sessions are run: one Spec Kit phase per session, targeted reads instead of whole
  files, and read-heavy lookups delegated so their output never enters the main context.

## Decision Outcome

Chosen option: **D**, because the measurement puts roughly two thirds of the cost in the quadratic
term that session length controls, and every other option competes for the remaining third.

Modelled against the longest session measured, the options are not close:

| Change                                     | Effect on that session's cost |
| ------------------------------------------ | ----------------------------- |
| D — split it into four sessions            | -65%                          |
| D — halve what each call adds to context   | -43%                          |
| C — drop the constitution from `CLAUDE.md` | -1.2%                         |

The objection to splitting a session is that each new one re-pays its baseline. Measured, a cold
start costs about 53k tokens of cache creation, and the `base x n` term does not change at all,
because the number of calls is the same either way. Splitting the longest session four ways costs
three extra warm-ups against a saving three orders of magnitude larger.

**A is rejected for now, deliberately.** The artifacts of a recent feature came to about 70KB
against a workspace of some 2,260 lines of Rust, and that ratio is the direct consequence of
principle I. It is being paid on purpose. This ADR records that it was examined and kept, so that
the next person to notice the ratio finds a decision rather than an oversight.

**B is rejected on its own evidence.** The learning log was read up to eleven times in a single
session, but always in full, at around 12k tokens a time; reading only its last entry costs a few
hundred. Splitting the file would additionally require amending the constitution, which names it by
path — a governance change to buy something a reading habit already buys.

**C is rejected on the measurement.** The constitution and `CLAUDE.md` together are about 5k tokens,
1.5% of the average context. Removing it would trade a rule that is followed for a saving that
rounds to nothing.

### Consequences

- Good, because the largest lever turns out to cost nothing structurally: no document moves, no
  amendment, no divergence from Spec Kit's templates.
- Good, because the cost of the process is now a number rather than an impression, and option A can
  be reopened later against the same model instead of against a feeling.
- Bad, because this is a habit and nothing enforces it. The gate cannot see a session boundary, and
  no lint can tell a targeted read from a wasteful one.
- Bad, because clearing context between phases discards conversational nuance that has, in practice,
  sometimes carried real information. This shifts load onto the written artifacts: they now have to
  be good enough to be the only handoff. That pressure is a cost, and arguably a benefit disguised
  as one.
- Neutral, because the measurement is a snapshot of one phase of the project. As the codebase grows
  relative to the documents, the balance between the terms will move.

### Confirmation

**Nothing in `cargo xtask check` can verify this**, and it is recorded here as unenforced rather
than described as tested. The gate sees commits, not sessions.

What can be done is to repeat the measurement. The session transcripts carry per-call token counts,
so the average context per call and the fit of the cost model can be recomputed at any time and
compared against the figures above. A later increment that shows the average rising back toward 264k
is the signal that the habit lapsed, or that the model stopped describing the project.

## Pros and Cons of the Options

### A — Trim what Spec Kit writes

- Good, because it attacks a real cost: the per-feature artifacts are re-read by every later phase.
- Good, because a thin slice arguably does not earn four design documents.
- Bad, because it edits vendored Spec Kit skills, creating divergence to re-apply on every update.
- Bad, because it trades against principle I, which is the reason this repository exists.

### B — Split the learning log per increment

- Good, because appending would stop requiring a read, and the file would stop growing without
  bound.
- Bad, because the constitution names the file by path, so this needs an amendment.
- Bad, because it also touches five ADRs, `CONTRIBUTING.md` and `rust-toolchain.toml`, and buys
  almost nothing that reading the tail does not already buy.

### C — Shrink what is always in context

- Good, because it is trivial to do and needs no agreement from anyone.
- Bad, because it is 1.2% of the problem, and the thing removed is the thing that makes the rest of
  the rules apply.

### D — Session discipline

- Good, because it is the only option that addresses the quadratic term.
- Good, because it is free and instantly reversible.
- Bad, because it depends entirely on being remembered.

## Reversibility

Cheap, permanently. The decision lands as guidance in `CLAUDE.md` and as this record; abandoning it
costs a commit. Nothing in the repository's structure depends on it, which is precisely why it was
preferred to options that would have.

What is not reversible is the measurement itself, and that is the durable part of this record. The
figures were taken at one point in the project's life and will drift; the cost model and the
ordering of the levers are what should be re-tested, not re-assumed.

## Confidence

High (85%).

The cost model was fitted to one session and checked against its measured total, so the ordering of
the levers is solid for a project shaped like this one. What would change the decision is the
codebase growing until reading source, rather than accumulating conversation, dominates the growth
term — at which point option A becomes the live question and this ADR should be superseded rather
than stretched. What would prove it wrong is a later measurement showing the average context per
call unchanged after the habit was adopted.

## More Information

- Issue #29, which asked for the investigation.
- `CLAUDE.md`, which carries the working rules this decision produces.
- [ADR-0021](0021-move-the-spec-home-to-spec-kit.md), for why the artifacts are shaped as they are.
