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
installs them. A commit made from OpenCode runs no gate and prints nothing: it succeeds, and
ADR-0005 records that trade. OpenCode has a session-start mechanism, and a plugin's `setup` is the
nearest equivalent, so one file under `.opencode/plugins/` closes the gap.

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
logged and swallowed: a hook installer that can stop a session is worse than the gap it closes.

### Consequences

- Good, because an OpenCode session leaves the tree in the state the gate assumes.
- Bad, because it is JavaScript in a repository that is otherwise Rust and shell, and nothing checks
  it: `.prettierignore` excludes `.opencode/`, so no formatter reads it and nothing parses it.

## Reversibility

Delete one file. The cost is a session's worth of the gap returning.

## Revisions

- 2026-09-25 — recorded with `.opencode/plugins/install-git-hooks.js`.
- 2026-09-25 — the first version did not run. It was written against the V1 plugin API, and
  OpenCode's own migration guide states that V1 plugin implementations do not run in V2; this
  machine is on v2.0.16. Rewritten to `Plugin.define` with the work in `setup`, and it now logs on
  load, because the `Bad` above turned out to be the whole story rather than a caveat.
- 2026-09-25 — so it did not run either, one step earlier. The cause was
  `import { Plugin } from "@opencode/plugin"`: nothing in this tree provides that package, so each
  start died at module resolution before `setup` was entered. Dropped it — the loader asks for a
  default export carrying an `id` and a `setup`, and `Plugin.define` is the identity function. The
  diagnosis came from `grep '\[monospace\]'` returning nothing, and that step was unsound: the line
  reaches the TUI, not the log file, so its absence there proved nothing at all. The loader's own
  `failed to load plugin` entry carried the cause the whole time, and nobody read it.
