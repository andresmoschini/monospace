---
status: accepted
scope: tooling
commitment: working
date: 2026-09-25
decision-makers: Andrés Moschini
---

# Install the git hooks from an OpenCode session too

## Context and Problem Statement

The hooks are installed by a `SessionStart` entry in `.claude/settings.json`, and nothing else
installs them. A commit made from OpenCode, or from a plain terminal in a clone where no Claude Code
session has ever opened, runs no gate and prints nothing: the commit simply succeeds. ADR-0005
records that trade and its cost, and the cost was accepted deliberately — CI is the boundary that
holds, and the hooks are fast feedback in front of it.

That reasoning covers a client with no session-start mechanism. OpenCode has one, and the same work
that added `AGENTS.md` found it: a plugin's body runs at startup, the moment `SessionStart` fires in
the other harness.

## Decision Drivers

- The failure is silent and consequential. Every other gap between the two harnesses costs a
  confused agent; this one lets a commit land that never met the gate.
- A local plugin in `.opencode/plugins/` loads without an `opencode.json`, so this costs one file.
- The plugin runs `git config`. Run carelessly it writes the global config and changes every other
  repository on the machine.

## Decision Outcome

Chosen option: **one plugin running the command `.claude/settings.json` runs**, because both
harnesses need the same effect by an equivalent mechanism, so a divergence would be a bug, not a
choice.

The command is `git -C ${worktree} config --local core.hooksPath .claude/git-hooks`. `--local` and
`-C` are both load-bearing: a bare `git config` outside a checkout writes the global config, so a
session pointed at the wrong directory would change every other repository silently. The failure is
logged and swallowed, because a hook installer that can stop a session is worse than the gap it
closes.

### Consequences

- Good, because an OpenCode session leaves the tree in the state the gate assumes, and because it is
  one file with no configuration, so nothing has to be kept in step beyond the command.
- Bad, because it is JavaScript in a repository that is otherwise Rust and shell, and nothing checks
  it: `.prettierignore` excludes `.opencode/`, so no formatter reads it and nothing parses it. Only
  a session starting exercises it.
- Bad, because it could not be verified from here. The command was run and read back; that the
  plugin loads was not observed, because that needs a session already past its own load.

## Reversibility

Delete one file. The cost is a session's worth of the gap returning. What grows is the duplication:
the command now lives in two harnesses' configuration and nothing checks the two agree.

## Revisions

- 2026-09-25 — recorded with `.opencode/plugins/install-git-hooks.js`.
