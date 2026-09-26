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
refusal rather than a silent conversion. The Speckit CLI writes CRLF on Windows, so the files it
produces are correct in content and cannot be staged. Measured on the first
`specify integration install`: 251 CRLF lines in one generated command file, and all three rewritten
state files. Nothing about this is OpenCode's, and a `specify update` writes the same bytes.

## Decision Drivers

- The alternative is to relax the repository. `core.safecrlf` exists to refuse exactly this, and the
  single line-ending rule is deliberate, so neither is the thing to change.
- `editorconfig` already owns the same concern for the files no formatter can parse, and the rule
  has exactly one right answer, so a fixer owns it on ADR-0020's test.
- It is not recoverable by ignoring it: a file git refuses cannot be committed at all.

## Decision Outcome

Chosen option: **a step of `cargo xtask fix` rewrites the endings — beside `editorconfig`, not
inside it, and last** ([issue #125](https://github.com/andresmoschini/monospace/issues/125)). It
cannot go inside `editorconfig-checker`, which skips `.specify/`, nor earlier, since every step
above it writes. Which files it touches is Git's to say; `xtask/src/eol.rs` is where that is argued.

### Consequences

- Good, because the repository keeps one line-ending rule and the refusal that enforces it, and the
  step recurs on every `specify update` without anyone remembering to run it.
- Good, because the message names the cause rather than a file, which is where a reader of a
  `fatal: CRLF` has to start.

## Reversibility

Free either way: a step in `FIX` is one entry, and a repository wanting CRLF throughout would change
`.gitattributes` and let this follow it. What it does not remove is the chance that someone "fixes"
a `fatal: CRLF` with `core.safecrlf=false` in their own clone, which is silent in every other file.

## Revisions

- 2026-09-25 — recorded after the first `specify integration install` refused its output.
- 2026-09-26 — the step is written, revised here rather than recorded beside it: the subject is one
  conflict, and neither record could be cited alone. Three findings, each measured:
  - **The cause was an exclusion, not tracking.** The first draft blamed what Git tracks; an
    untracked CRLF file in the root is reported, and the `Exclude` in `.editorconfig-checker.json`
    is the whole reason `.specify/` is silent.
  - **The two configurations disagreed about batch files.** `.editorconfig` asks for LF in
    everything and `.gitattributes` for CRLF in `*.bat` and `*.cmd`, and the checker believed the
    first; `.editorconfig` carries the exception now, and says there why.
  - **Taken out, it fails as it should.** Without it a CRLF file under `.specify/` survives
    `cargo xtask fix` — which reports `all 5 fixers ran clean` — and `git add` still refuses it.
