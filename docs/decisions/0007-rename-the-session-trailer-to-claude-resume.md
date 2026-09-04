---
status: accepted
date: 2026-09-04
decision-makers: Andrés Moschini
---

# Rename the session trailer to `Claude-Resume`

## Context and Problem Statement

[ADR-0006](0006-record-the-claude-session-in-commit-trailers.md) added a `Claude-Session` trailer
holding the id from `CLAUDE_CODE_SESSION_ID`, written by the `commit-msg` hook. Two commits later,
Claude Code began appending a trailer under the same name itself, carrying a URL instead:

```text
Claude-Session: https://claude.ai/code/session_<opaque-id>
```

That started after Remote Control was enabled, which is the point that matters. It is not a property
of the environment that can be relied on; it is the visible effect of a setting held outside this
repository, and it can stop as easily as it started.

The two values are not the same identifier in different clothes.
`e221ee91-2501-48ec-85fe-cc0e877f3447` is what `claude --resume` takes locally;
`session_<opaque-id>` is what opens the session in a browser. They are different handles, in
different systems, on the same conversation.

Because the hook uses `--if-exists doNothing`, whichever arrives first wins, and the one written
into the message arrives first. Measured: with the URL present the hook adds nothing and sits inert;
with it absent the hook stamps the local id as designed. So nothing is broken — but one key now
carries two shapes depending on a setting invisible from the repository, and the mechanism this
project controls is the one doing nothing.

## Decision Drivers

- A trailer key should mean one thing. Anything reading `Claude-Session` would have to accept both a
  UUID and a URL, and could not tell which to expect from anything in the repository.
- The durable mechanism is the one checked into the repository. A record that only appears while an
  external setting is on is not a record.
- The two identifiers are both useful and neither replaces the other: one resumes locally without a
  network, one opens from any device without a local transcript.

## Considered Options

- **A** — Leave the collision. One key, two shapes.
- **B** — Drop the hook's stamp and rely on what Claude Code emits.
- **C** — Rename the hook's trailer so both coexist.

## Decision Outcome

Chosen option: **C, rename the hook's trailer to `Claude-Resume`**, because the collision is a
naming problem rather than a redundancy, and the name says what the value is for: it is the argument
to `claude --resume`.

`Claude-Session` is left to Claude Code, which this repository does not control and cannot rename.

### Consequences

- Good, because each key means exactly one thing, and a reader or a script can rely on the shape.
- Good, because the record the repository is responsible for keeps working whether Remote Control is
  on, off, or replaced by something else later.
- Good, because both handles are kept, and they fail in different ways: the URL needs Anthropic's
  hosting, the local id needs the transcript on this machine.
- Bad, because commits now end with three trailer lines. On a published crate that is three lines of
  tooling metadata that most readers will never use.
- Neutral, because the hook keeps `--if-exists doNothing`. With a key nothing else writes, that no
  longer arbitrates between two producers; it only keeps `git commit --amend` idempotent.

### Confirmation

Verified by running the hook against a prepared message file. Without a `Claude-Resume` trailer it
adds one; with it already present it adds nothing. Both are visible in
`git interpret-trailers --parse`, alongside `Co-Authored-By`.

## Pros and Cons of the Options

### A — Leave the collision

- Good, because nothing has to change and nothing is currently broken.
- Bad, because the value under one key depends on a setting no file in the repository records, so
  the history becomes inconsistent for a reason that cannot be reconstructed from the history.

### B — Drop the hook's stamp

- Good, because it removes a mechanism and leaves one line instead of two.
- Good, because the URL fixes the worst flaw named in ADR-0006: it survives a change of machine.
- Bad, because it replaces something this repository owns with something a toggle controls. If the
  setting is turned off, commits silently stop being traceable, and nothing in the repository
  explains why. This is the failure mode the rest of the quality gate is built to avoid.

### C — Rename the hook's trailer

- Good, because it keeps both handles and gives each an unambiguous name.
- Bad, because three trailers is more than most commits need, and the two Claude keys sit next to
  each other looking like duplicates until someone reads this record.

## Reversibility

The mechanism is trivially reversible: one string in the hook.

The stamps already written are not, and the history now contains three shapes — a UUID under
`Claude-Session` in `8545fbe`, a URL under `Claude-Session` from `e8ab716` onwards, and a UUID under
`Claude-Resume` from this change forward. That is the cost of having decided twice, and it is left
in place rather than rewritten: two commits' worth of inconsistency is cheaper than editing history,
and this record explains it.

## Confidence

High (~85%).

Once the two values were found to be different identifiers rather than two formats of one, the
choice was between naming them separately and discarding one. Discarding the local id means trusting
a setting; discarding the URL is not available, since Claude Code writes it.

What would change this: Claude Code emitting the local id as well, which would make the hook
redundant for real rather than only while a setting is on.

## More Information

- Supersedes [ADR-0006](0006-record-the-claude-session-in-commit-trailers.md), whose reasoning for
  using a trailer at all, rather than a free-standing line, still holds and is not repeated here.
