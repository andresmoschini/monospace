---
status: accepted
scope: tooling
commitment: working
date: 2026-09-26
decision-makers: Andrés Moschini
---

# Reduce what the instruction files restate, and say what decides it

## Context and Problem Statement

`TODO(TRIM_DUPLICATION)` has ridden in the constitution's header since 1.0.0: `CONTRIBUTING.md` and
`CLAUDE.md` state several of its rules in their own words. It was a note then; principle VIII and
Governance now make it a rule, and it is still open at 2.0.1. The duplication has since cost more
than redundancy: ADR-0033's table of state labels still names the three stages ADR-0051 replaced,
and `CONTRIBUTING.md` claims twice that only a Claude Code session installs the hooks, which
ADR-0058 made untrue.

`AGENTS.md` joined the pair after the TODO was written, and it is the shape the other two are
measured against: a router, and the facts that are in the code and in no document
([ADR-0056](0056-give-opencode-its-own-instruction-file.md)). What is missing is a test, not a
shape: nothing says what to do with a sentence that is half rule and half local fact.

## Decision Drivers

- A citation works only where the target holds the rule, so the test answers "is this said anywhere
  else" and never "could this live somewhere else" — the second question deletes reasoning that
  exists nowhere else.
- Both files are read on every call and a restated rule is followed at random — the cost
  [ADR-0027](0027-control-token-cost-through-session-discipline.md) measures. A rule of thumb nobody
  can apply paragraph by paragraph is a preference.

## Decision Outcome

Chosen option: **three questions per paragraph, and no relocation**, because the cut has to be
reproducible, and because moving prose into a record is a decision this issue does not take.

- **Does the constitution state this rule?** The statement goes, and a citation by name replaces it,
  in the section that owns the procedure.
- **Does a document this file already cites hold this reasoning?** It goes too; the citation is
  already there.
- **Neither?** It stays, even where a record could have held it.

Two calls the test settled are worth naming, because each looked like a cut and was not.
`CONTRIBUTING.md`'s label table stays: ADR-0033's copy names `spec` and `plan`, so this is the only
correct one, and the drift is [issue #127](https://github.com/andresmoschini/monospace/issues/127).

### Consequences

- Good, because the two files are what the issue asked for: `CONTRIBUTING.md` is how to run the
  tooling, `CLAUDE.md` is session guidance, and a rule is worded once.
- Bad, because a reader after what `deciding` means now opens two files. Addressability is the
  defect an earlier entry measured, and this pays a little of it back deliberately.
- Neutral, because nothing checks the trim: the gate compares the files against neither the
  constitution nor each other. Review holds it, as ADR-0023 says of its own rule.

## Reversibility

Cheap both ways: restoring a paragraph is a revert, and no tool reads the wording.
