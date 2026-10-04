# Monospace

Everything that governs a change to this repository is in these two files, and they are imported
rather than copied so that the two cannot drift:

@.specify/memory/constitution.md

@AGENTS.md

What is left here is what is true only in a Claude Code session.

## What is different in Claude Code

- **The hooks are installed from here.** `.claude/git-hooks/pre-commit` and `commit-msg` are what
  `core.hooksPath` points at, and only a Claude Code session that has run `update-config` guarantees
  they were written. `AGENTS.md` says the same about the OpenCode plugins, and neither harness can
  see whether the other one is working.
- **The session trailer is `Claude-Resume`**, read from `CLAUDE_CODE_SESSION_ID` by `commit-msg`. It
  is deliberately not the same key as OpenCode's: neither client's id resumes the other, and one key
  for both would be a lie about half the commits.
- **Skills live in `.claude/skills/` and slash commands in `.claude/commands/`.** This repository
  keeps neither — there is nothing to invoke, because the flow is six steps and
  [the constitution](.specify/memory/constitution.md#development-workflow) states all of them. If a
  command is ever added, its body lives in one place under `docs/` and the file here is a pointer,
  so the two harnesses cannot disagree about what it does.

Everything else — how to work a change, what the gate checks, which tool owns which file, how to
commit — is in [AGENTS.md](AGENTS.md) and [CONTRIBUTING.md](CONTRIBUTING.md), and none of it is
repeated here.
