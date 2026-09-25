---
status: accepted
scope: tooling
commitment: working
date: 2026-09-25
decision-makers: Andrés Moschini
---

# Give OpenCode its own instruction file, and leave Claude Code's alone

## Context and Problem Statement

Session guidance reaches an agent through `CLAUDE.md`, whose first line is
`@.specify/memory/constitution.md` — an import Claude Code expands at launch. OpenCode reads
`AGENTS.md` instead and expands no import, so an OpenCode session had no session guidance at all:
both were files it had to know to open. The same gap already has a name here: the git hooks,
installed only by a Claude Code `SessionStart`, so a commit from another client runs no gate
([ADR-0005](0005-install-the-git-hooks-from-claude-code.md)).

What forces the decision is that the second file now exists, and whether it is a second copy of the
rules or a second owner of them is a governance question: the constitution's
[Governance](../../.specify/memory/constitution.md#governance) names who owns what, and `AGENTS.md`
is not in the list.

## Decision Drivers

- A restated rule is a defect to remove, and a session costs its length squared
  ([ADR-0027](0027-control-token-cost-through-session-discipline.md)). Both cut toward short files;
  neither is answered by making one short enough to serve two clients.
- Pruning `CLAUDE.md` of what `AGENTS.md` also says strips every Claude Code session, which reads
  neither. Principle VI's duplication is of the constitution, not of an adapter for a second client.

## Decision Outcome

Chosen option: **`AGENTS.md` is a router, and `CLAUDE.md` is not pruned**, because the two are
adapters over the same sources for two clients that load one each.

`AGENTS.md` orders the reading and carries only what is verifiable from an executable source, so it
restates no rule. `CLAUDE.md` does not import it: tidier, and the mechanism is already there, but it
would add 4 KB to every Claude Code call to serve a client reading neither half.

### Consequences

- Good, because an OpenCode session learns at launch that binding rules exist and where they live,
  rather than working without them and saying nothing — the failure ADR-0005 already paid for once.
- Bad, because Governance still does not name `AGENTS.md`, so this record alone establishes who owns
  session guidance for a second harness. That amendment belongs in this increment and is not made.
- Bad, because nothing detects the two files disagreeing. The gate reads both for spelling,
  structure and width, and checks neither against the other or against the constitution.

## Reversibility

Deleting it costs nothing: no code reads `AGENTS.md` and no data moves. What is not cheap is the
drift the decision allows, and that grows with the number of rules rather than with the work.

## Revisions

- 2026-09-25 — recorded with `AGENTS.md`, then cut from 212 lines to 59 once the three source files
  were measured against it: most of it restated them, the defect principle VI names. The Governance
  amendment above is still open.
