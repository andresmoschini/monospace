---
status: accepted
scope: tooling
commitment: working
date: 2026-09-25
decision-makers: Andrés Moschini
---

# Install the spec-kit opencode integration beside the claude one

## Context and Problem Statement

The repository declared one integration. `.specify/integration.json` listed `claude` alone with
`default_integration: claude`, and the ten Speckit skills sat in `.claude/skills/`. An OpenCode
session found them anyway, because it reads that directory too — so the workflow appeared to work
while the repository said nothing about supporting it. The one thing that branches on the declared
integration, `format_speckit_command` in `common.sh`, resolved the Claude separator; it is defined
and never called here, so nothing had broken yet.

Installing is not the obvious command. `specify init --here --force` is what a stale install
suggests, and it would have overwritten the templates this repository replaced and the constitution
it amends.

## Decision Drivers

- `specify integration install` writes only the agent's own files and reports the shared paths it
  declined to touch. Measured: all twelve were left alone.
- Speckit's generated files are already outside the gate's Markdown ownership, by decision, for
  `.claude/skills/speckit-*/`. A second copy needs that exclusion, not a new rule.
- Neither harness should stop working because the other one was installed.

## Decision Outcome

Chosen option: **`specify integration install opencode --force`, with `claude` left as the
default**, because both harnesses are in use. `--force` is what it takes: the CLI marks `opencode`
unsafe to multi-install, and this repository wants exactly that. `default_integration` stays
`claude`, so a Claude Code session behaves as before.

Two things follow, both visible in the diff: `.opencode/commands/` joins `.claude/skills/speckit-*/`
in the two ignore lists under the reason already written there, and `.specify/integrations/.cache/`
is gitignored beside `feature.json`.

### Consequences

- Good, because the repository can say it supports the harness it is worked on from, and because
  `format_speckit_command` resolves a separator matching the session.
- Bad, because `.opencode/commands/` is a second copy of the same ten prompts as `.claude/skills/`,
  and nothing checks the two stay in step. They are versioned together, which is all that keeps them
  equal.
- Bad, because `invoke_separator` is now a second key a future Speckit release may branch on, and
  this repository chose a value for it.

## Reversibility

`specify integration uninstall opencode`, and drop the two ignore entries. No code reads any of it,
so that is one command and one commit.

## Revisions

- 2026-09-25 — recorded with `.opencode/commands/` and the two configuration entries.
