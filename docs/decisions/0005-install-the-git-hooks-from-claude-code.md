---
status: accepted
date: 2026-09-04
decision-makers: Andrés Moschini
---

# Install the git hooks from the Claude Code session

## Context and Problem Statement

The repository has two git hooks: `pre-commit` runs the quality gate, and `commit-msg` checks the
message against Conventional Commits. Git does not clone hooks, so something has to activate them in
each working copy by pointing `core.hooksPath` at a directory the repository tracks.

There are two ways to do that, and they differ in who performs the step rather than in anything Git
does. A person can run one command, documented in `CONTRIBUTING.md`. Or a `SessionStart` hook in
`.claude/settings.json` can run it whenever a Claude Code session opens.

That choice decides who the hooks protect. It is not a question of where the files live: Git gives
no meaning to the directory name, and `.githooks` and `.claude/git-hooks` are equally valid.

## Decision Drivers

- The hooks exist to shorten the feedback loop, not to be the last line of defense. CI runs the same
  entry point on every push and pull request, so nothing reaches the main branch unchecked either
  way.
- Setup steps that a person has to remember are steps that get skipped, and a hook that was never
  installed is indistinguishable from one that passed.
- This project is built through Claude Code sessions. Optimizing the path that is actually used is
  worth more than covering one that is currently hypothetical.

## Considered Options

- **A** — A person runs `git config core.hooksPath .githooks`, documented in `CONTRIBUTING.md`.
- **B** — A `SessionStart` hook in `.claude/settings.json` runs it when a session opens.
- **C** — Both: automatic under Claude Code, and documented for anyone else.

## Decision Outcome

Chosen option: **B, installed by the Claude Code session**, because it removes a manual step from
the workflow the project actually uses, and because CI already provides the guarantee that the hooks
would otherwise be asked to provide.

The hooks move to `.claude/git-hooks/` to match. Nothing technical requires it — the directory name
carries no meaning to Git — but the location should say who installs them, and grouping them with
`.claude/settings.json`, which is what installs them, keeps that honest.

### Consequences

- Good, because a session opens and the hooks are active, with nothing to remember and nothing to
  document as a prerequisite.
- Good, because it matches how the same author's other repositories are set up, so the mechanism is
  already familiar rather than particular to this one.
- Bad, and this is the cost being accepted knowingly: a commit made from a terminal in a clone where
  no Claude Code session has ever opened runs no hooks at all, silently. Nothing announces that the
  gate did not run. The reasoning is that CI catches it on push, so the failure is delayed rather
  than missed — but "delayed" means a broken commit can exist in the local history and be pushed
  before anyone finds out, where option A would have blocked it at creation.
- Bad, because the hooks now depend on a specific tool being installed and used. If work ever
  happens outside Claude Code, this decision quietly stops applying and will need revisiting rather
  than merely re-reading.
- Neutral, because a person can still run `git config core.hooksPath .claude/git-hooks` by hand. The
  path is not hidden; it is simply not required.

### Confirmation

`git config core.hooksPath` reports `.claude/git-hooks` in any working copy where a session has
opened. There is no check that this happened, by construction: the decision is precisely that the
hooks are best-effort and CI is the boundary that holds.

## Pros and Cons of the Options

### A — Installed by hand, documented

- Good, because the hooks protect every commit in the working copy once the step is done, whatever
  tool made it.
- Good, because it depends on nothing beyond Git.
- Bad, because it is a step someone must find and run, and a fresh clone that skips it looks exactly
  like one where everything passed.

### B — Installed by the Claude Code session

- Good, because it cannot be forgotten in the workflow the project uses.
- Bad, because it covers only that workflow, and covers it silently — there is no signal
  distinguishing "the gate passed" from "the gate never ran".

### C — Both

- Good, because it covers both workflows.
- Bad, because it re-applies `core.hooksPath` at every session start, which silently overrides
  someone who unset it on purpose.
- Bad, because two installation paths mean the answer to "are the hooks active?" depends on how the
  repository was opened, which is more confusing than either single answer.

## Reversibility

Fully reversible and cheap. Returning to option A is deleting the `SessionStart` entry and adding
one command back to `CONTRIBUTING.md`; the hook scripts themselves do not change, and neither does
what they run.

What is not reversible is any commit made during the period when the hooks were not active. Those
stay in the history unchecked, and CI reports them only in aggregate on the push that carries them.

## Confidence

Medium (~60%), and lower than the rest of this phase's decisions.

The reasoning is sound where it applies: CI is genuinely the enforcement boundary, and the hooks are
genuinely a convenience on top of it. The doubt is about the silence. Every other decision in this
phase was made to stop a check from passing without running — the entry point refuses to start
without its tooling rather than reporting a partial pass, and a step is never skipped with a
warning. This decision accepts exactly that pattern in one place, on the grounds that CI covers it.

What would change this: a broken commit reaching the history because a session had not been opened,
or work happening outside Claude Code often enough that "the hooks are installed" stops being a safe
assumption. Either would argue for option C, accepting its override problem as the smaller cost.

## More Information

- [ADR-0006](0006-record-the-claude-session-in-commit-trailers.md), which adds a hook that is
  meaningful only inside a Claude Code session and is therefore a natural fit for this arrangement.
- `.claude/settings.json` holds the `SessionStart` entry that performs the installation.
