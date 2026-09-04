# Architecture Decision Records

This directory holds the project's architecture decision records (ADRs). Each file records one
decision: what was decided, what else was considered, and why the alternatives were rejected.

The format is MADR 4.0 with two local additions. The reasoning is in
[ADR-0000](0000-use-madr-for-architecture-decisions.md); the template to copy is
[`adr-template.md`](adr-template.md).

## Index

| ADR                                                    | Title                                                            | Status   |
| ------------------------------------------------------ | ---------------------------------------------------------------- | -------- |
| [0000](0000-use-madr-for-architecture-decisions.md)    | Use MADR for architecture decision records                       | accepted |
| [0001](0001-virtual-cargo-workspace-under-crates.md)   | Lay out the project as a virtual Cargo workspace under `crates/` | accepted |
| [0002](0002-no-minimum-supported-rust-version.md)      | Declare no minimum supported Rust version, for now               | accepted |
| [0003](0003-pin-the-toolchain-exactly.md)              | Pin the Rust toolchain to an exact version                       | accepted |
| [0004](0004-node-toolchain-for-the-non-rust-checks.md) | Use a Node toolchain for the checks Rust cannot perform          | accepted |
| [0005](0005-install-the-git-hooks-from-claude-code.md) | Install the git hooks from the Claude Code session               | accepted |

## How to add one

1. Copy `adr-template.md` to `NNNN-short-kebab-case-title.md`, where `NNNN` is the next unused
   number, zero-padded to four digits.
2. Fill it in. Use `status: proposed` while it is still under discussion and `status: accepted` once
   it is settled.
3. Add a row to the index above.
4. Commit it with a `docs:` prefix, in the same commit as the change it justifies or in the commit
   immediately before it.

## Rules

**Write the record when the decision is taken.** If you are about to write code that depends on a
decision nobody has recorded, record it first. An ADR reconstructed weeks later is a summary of what
happened rather than a record of the reasoning: by then the rejected options have already been
forgotten, which makes the most valuable section the least accurate one.

**Never edit an accepted record to change its conclusion.** If the decision changes, write a new ADR
and set the old one's status to `superseded by ADR-NNNN`. This directory is a history, not a living
document, and being able to see that a decision was reversed — and what argument reversed it — is
most of what it is worth. Fixing a typo or a broken link is fine.

**Not everything needs an ADR.** The test is whether the decision is expensive to undo, or whether
someone will later ask "why is this like this?". If neither applies, it belongs in
`docs/learning-log.md` instead, which is for what was learned rather than what was decided.
