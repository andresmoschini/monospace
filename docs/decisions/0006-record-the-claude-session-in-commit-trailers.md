---
status: accepted
date: 2026-09-04
decision-makers: Andrés Moschini
---

# Record the Claude Code session in a commit trailer

## Context and Problem Statement

The brief states that this project exists to build proficiency in Spec-Driven Development with
Claude. The conversation is not incidental to the work; it is the thing being learned from.

Three layers of record already exist. ADRs hold decisions, commit bodies hold changes and their
reasons, and `docs/learning-log.md` holds what was learned. None of them holds the reasoning as it
happened: the exploration, the measurements that changed a recommendation, the options rejected
before they were worth writing down.

Claude Code keeps that as a session transcript, which `claude --resume <id>` reopens. Nothing
connects a commit to the session that produced it.

## Decision Drivers

- The process is the subject of this project, so a link from a change to the conversation that
  produced it is evidence in exactly the dimension being studied.
- The learning log is written by re-reading sessions; finding the right one should not be a search.
- Commit messages are permanent, and this repository is intended to be published, so anything added
  to them is added forever and for everyone.

## Considered Options

- **A** — No link. The commit body carries whatever matters.
- **B** — A free-standing line, `[Resume: claude --resume <id>]`, appended to the message.
- **C** — A `Claude-Session: <id>` trailer.

## Decision Outcome

Chosen option: **C, a trailer**, because it is the only form that survives alongside the trailers
already in use.

Every commit in this repository ends with a `Co-Authored-By` trailer, and Git parses trailers as a
block: a line that is not a trailer invalidates all of them. This was measured rather than assumed.
With `[Resume: ...]` appended after `Co-Authored-By`, whether as its own paragraph or inside the
block, `git interpret-trailers --parse` reports **no trailers at all**, and the co-author
attribution is silently lost. As a trailer, both are read.

The stamp is applied by the `commit-msg` hook before commitlint runs, so what is validated is what
gets committed. It uses `git interpret-trailers --if-exists doNothing`, which places the line
correctly in the block and makes `git commit --amend` idempotent, keeping the session that first
wrote the commit rather than the one that last touched it.

### Consequences

- Good, because a change can be traced to the conversation that produced it, which is the layer the
  other three records do not capture.
- Good, because trailers are machine-readable:
  `git log --format='%h %(trailers:key=Claude-Session,valueonly)'` lists them, where a bracketed
  line would need a regular expression.
- Good, because it degrades quietly. Outside a Claude Code session the variable is absent, nothing
  is stamped, and the commit proceeds. A commit with no session trailer is fine; one carrying
  somebody else's would not be, and a commit that fails over this would be worse than either.
- Bad, because the link rots and the history does not. Transcripts live in `~/.claude/projects/`,
  outside the repository: they do not survive a new machine, and they can be deleted. The trailer
  outlives what it points at, and on a published crate it is a line every reader sees and nobody but
  the author can follow.
- Bad, because it creates a temptation to under-write commit bodies and ADRs, on the grounds that
  the reasoning is in the session. The test to apply: if a commit body only makes sense with the
  transcript open, the body is wrong and the link hid it rather than helping.
- Neutral, because the id itself discloses nothing. It is an opaque identifier, useless without the
  local transcript store.

### Confirmation

`git interpret-trailers --parse` on any commit made from a session reports both `Co-Authored-By` and
`Claude-Session`, which is what a bracketed line would have broken. Absence of the trailer is not a
failure and is not reported: it means the commit was made outside a session.

## Pros and Cons of the Options

### A — No link

- Good, because commit messages stay about the code, and nothing in them rots.
- Bad, because the reasoning that produced a change is then only recoverable if someone thought to
  write it down at the time, which is exactly the material the learning log is assembled from.

### B — A free-standing `[Resume: ...]` line

- Good, because it is directly actionable: the command can be copied out of the log and run.
- Good, because it is the format already in use in the author's other repositories, so it needs no
  explanation.
- Bad, because it destroys the trailer block, as measured above. That is not a matter of taste.

### C — A `Claude-Session` trailer

- Good, because it coexists with `Co-Authored-By` and is parseable by standard Git tooling.
- Bad, because it is not the command, so `claude --resume` has to be documented; `CONTRIBUTING.md`
  carries it.
- Bad, because it differs from the format used in the author's other repositories.

## Reversibility

The mechanism is trivially reversible: removing one `case` block from the hook stops all future
stamping.

The stamps already written are not. They are part of commit messages, and rewriting history to
remove them costs more than they cost to leave. This is the reason the decision is worth an ADR at
all — the mechanism is small, but its output is permanent.

## Confidence

Medium-high (~75%) on the form, medium (~60%) on doing it at all.

The form is settled by measurement: given that `Co-Authored-By` is in use, a trailer is the only
option that does not break it, and there is nothing left to weigh.

The residual doubt is the second bad consequence. This project's whole discipline is that reasoning
gets written down where it lasts, and a link to a transcript is the one artifact here that points
outside the repository at something impermanent. What would prove it wrong: commit bodies or ADRs
getting thinner, with the session link doing work they should have done.

## More Information

- [ADR-0005](0005-install-the-git-hooks-from-claude-code.md), which puts the hooks under Claude Code
  and makes a session-only stamp a natural fit.
- The hook is `.claude/git-hooks/commit-msg`, and explains its own mechanics inline.
