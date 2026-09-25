---
status: accepted
scope: tooling
commitment: working
date: 2026-09-25
decision-makers: Andrés Moschini
---

# Stamp the OpenCode session into the commit, without an environment variable of its own

## Context and Problem Statement

[ADR-0006](0006-record-the-claude-session-in-commit-trailers.md) put the session in a commit trailer
and [ADR-0007](0007-rename-the-session-trailer-to-claude-resume.md) named the key. The mechanism is
a hook reading an environment variable, and it works because Claude Code exports
`CLAUDE_CODE_SESSION_ID` into the environment of its shell tools. OpenCode exports nothing of the
kind — measured, not assumed: the only `OPENCODE_*` variable in a session's environment is
`OPENCODE_TERMINAL`. Commits from this harness therefore carried no trailer at all.

## Decision Drivers

- The id has to reach `commit-msg` as a child of the shell that ran `git commit`, or it is a guess.
- OpenCode V2 has a place to put it. `ctx.session.hook("context")` carries the session id and runs
  immediately before an agent model request, so the value is the session about to act.
  `ctx.shell.hook("create.before")` then sets environment on the shells the agent spawns.
- Neither client's id resumes the other, so they take separate keys, and neither shape is the
  other's.

## Decision Outcome

Chosen option: **one plugin that captures the id from the session hook and injects it into the
agent's shells**, because that is the only V2 mechanism delivering the guarantee Claude Code gets
from its environment, and a weaker one risks carrying somebody else's id.

`commit-msg` gained `MONOSPACE_SESSION_ID` beside `CLAUDE_CODE_SESSION_ID`, each stamping its own
key and each validated against a pattern of its own, because the two clients disagree on the shape
of an id: a Claude Code id is a UUID, an OpenCode id is `ses_` and then more of the same, and one
pattern shared by both suits neither. An absent or malformed value stamps nothing and lets the
commit proceed, which is what ADR-0006 settled.

### Consequences

- Bad, for the reason ADR-0058 gives and does not repeat: nothing here can execute a plugin under
  `.opencode/`. What it logs reaches the TUI, not the log file, so the loader's own entries are the
  only record that it loaded at all.
- Bad, because the same command now lives in three places — `.claude/settings.json`, two plugins,
  one hook — and nothing checks they agree.

## Reversibility

Delete one file and the `MONOSPACE_SESSION_ID` block from the hook; commits already written keep
their trailers and nothing reads them.

## Revisions

- 2026-09-25 — extends ADR-0006 and ADR-0007 to a second client, replacing neither.
- 2026-09-25 — recorded with two defects beside it, neither of them this mechanism: a plugin that
  had never loaded, and a hook whose one shared pattern matched neither client's id. Both causes and
  both fixes are in ADR-0058's revisions. The width is not pinned, because an exact length is a
  claim about OpenCode's internal id scheme and a change to it would stop the stamping silently.
