---
status: accepted
scope: tooling
commitment: working
date: 2026-09-25
decision-makers: Andrés Moschini
---

# Normalize what the Speckit CLI writes, or git refuses it

## Context and Problem Statement

`.gitattributes` sets one line-ending rule for everything Git considers text — `* text=auto eol=lf`
— and `core.safecrlf` is on, which is the combination that turns a line-ending problem into a
refusal rather than a silent conversion. The Speckit CLI writes CRLF on Windows. So the files it
produces are correct in content and cannot be staged, and `git add` answers
`fatal: CRLF would be replaced by LF`.

Measured on the first `specify integration install`: 251 CRLF lines in one generated command file,
and all three rewritten state files. Nothing about this is OpenCode's; the same install on Claude
Code writes the same bytes, and a `specify update` would too.

## Decision Drivers

- The alternative is to relax the repository. `core.safecrlf` exists to refuse exactly this, and the
  single line-ending rule is deliberate, so neither is the thing to change.
- The affected files are already outside the gate's reach — prettier and markdownlint ignore them,
  and `editorconfig-checker` only sees what is tracked. So nothing mechanical will normalize them.
- It is not recoverable by ignoring it: a file git refuses cannot be committed at all.

## Decision Outcome

Chosen option: **normalize the CLI's output to LF before staging, by hand**, because the two
mechanisms that could do it automatically are both switched off on purpose, and the alternative is
to weaken a rule the whole repository rests on.

The step is one command, and it belongs in `AGENTS.md` under what a session needs to know rather
than in `CONTRIBUTING.md`, which owns the tooling's own commands.

### Consequences

- Good, because the repository keeps one line-ending rule and the refusal that enforces it, and the
  generated files still reach the repository with the endings it expects.
- Bad, because it is a step a person has to remember, and nothing fails until `git add` does, which
  is a confusing place to be told about a decision made in an ADR.
- Bad, because it will recur on every `specify update` as well as on every install, and the error
  names a file rather than the cause.

## Reversibility

Free either way. Normalizing is a one-line command and the result is what the repository wanted
anyway; not normalizing means the files cannot be committed.

What grows is the number of Speckit commands that carry the step, and the chance that someone
"fixes" a `fatal: CRLF` by disabling `core.safecrlf` instead, which would be silent everywhere else.

## Revisions

- 2026-09-25 — recorded after the first `specify integration install` refused its output.
