# Architecture Decision Records

This directory holds the project's architecture decision records (ADRs). Each file records one
decision: what was decided, what else was considered, and why the alternatives were rejected.

The format is MADR 4.0 with two local additions. The reasoning is in
[ADR-0000](0000-use-madr-for-architecture-decisions.md); the template to copy is
[`adr-template.md`](adr-template.md).

## Index

| ADR                                                             | Title                                                                           | Status             |
| --------------------------------------------------------------- | ------------------------------------------------------------------------------- | ------------------ |
| [0000](0000-use-madr-for-architecture-decisions.md)             | Use MADR for architecture decision records                                      | accepted           |
| [0001](0001-virtual-cargo-workspace-under-crates.md)            | Lay out the project as a virtual Cargo workspace under `crates/`                | accepted           |
| [0002](0002-no-minimum-supported-rust-version.md)               | Declare no minimum supported Rust version, for now                              | accepted           |
| [0003](0003-pin-the-toolchain-exactly.md)                       | Pin the Rust toolchain to an exact version                                      | accepted           |
| [0004](0004-node-toolchain-for-the-non-rust-checks.md)          | Use a Node toolchain for the checks Rust cannot perform                         | accepted           |
| [0005](0005-install-the-git-hooks-from-claude-code.md)          | Install the git hooks from the Claude Code session                              | accepted           |
| [0006](0006-record-the-claude-session-in-commit-trailers.md)    | Record the Claude Code session in a commit trailer                              | superseded by 0007 |
| [0007](0007-rename-the-session-trailer-to-claude-resume.md)     | Rename the session trailer to `Claude-Resume`                                   | accepted           |
| [0008](0008-compose-overlapping-cells-with-three-state-arms.md) | Compose overlapping cells with three-state arms and two stamp modes             | accepted           |
| [0009](0009-degrade-a-cell-to-its-base-stroke.md)               | Degrade a cell to its base stroke when no character matches                     | accepted           |
| [0010](0010-separate-position-and-size.md)                      | Separate position and size instead of one rectangle                             | accepted           |
| [0011](0011-expose-cell-for-testing-stamping.md)                | Expose `cell` so stamping can be checked without rendering                      | accepted           |
| [0012](0012-one-stroke-per-cell.md)                             | Give a cell one stroke, with none per arm                                       | accepted           |
| [0013](0013-key-a-rule-by-stroke-per-side.md)                   | Key a glyph rule by a stroke per side                                           | accepted           |
| [0014](0014-collapse-glyph-sets-into-a-catalog.md)              | Collapse glyph sets into a single catalog                                       | accepted           |
| [0015](0015-represent-a-stroke-as-a-string.md)                  | Represent a stroke as an owned `String`                                         | accepted           |
| [0016](0016-return-a-string-from-render.md)                     | Return a `String` from `render`                                                 | accepted           |
| [0017](0017-ask-the-cell-whether-it-is-decided.md)              | Ask the cell whether it is decided, and let `stamp` skip a decided target       | accepted           |
| [0018](0018-mirror-the-decided-skip-in-above.md)                | Mirror the decided-cell skip in `Above`, accepting a second unverifiable branch | accepted           |
| [0019](0019-represent-a-glyph-as-a-grapheme-cluster.md)         | Represent a glyph as a validated grapheme cluster                               | accepted           |
| [0020](0020-scope-cargo-xtask-fix-to-deterministic-fixers.md)   | Scope `cargo xtask fix` to deterministic fixers, and fix `clippy` by hand       | accepted           |
| [0021](0021-move-the-spec-home-to-spec-kit.md)                  | Move the spec home to Spec Kit's `specs/` and freeze `docs/specs/`              | accepted           |
| [0022](0022-non-interactive-cli-before-the-tui.md)              | Build the non-interactive CLI before the interactive TUI                        | accepted           |

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
[the learning log](../learning-log.md) instead, which is for what was learned rather than what was
decided.

**Records written before the constitution cite `docs/brief.md`, which no longer exists.** The brief
was this project's initial spec, and it held the principles, the scope and the constraints until
[the constitution](../../.specify/memory/constitution.md) took them over. Its other content went to
[the model](../model.md), which owns the domain's design, provenance and open questions, and to
[the roadmap](../roadmap.md), which owns the phases. Where a record's citation pointed at a rule
that still exists, it now points at that rule's home; where it pointed at a question the record
itself answered, the link is gone and only git history has the original. The prose is untouched: a
record saying "the brief requires a stable toolchain" is reporting what it weighed at the time, and
that is not a reference to redirect.

**A feature's `research.md` is not a decision record.** Spec Kit writes one per feature in almost
this shape — decision, rationale, alternatives considered — but scoped to that feature, with no
status and no way to supersede it. It owns investigation local to the slice: which crate, which
version, what is idiomatic. The moment a finding will outlive the feature, the record is written
here and `research.md` links to it. The full split, including what the Complexity Tracking table in
a plan owns, is in [the constitution](../../.specify/memory/constitution.md) under "Development
Workflow".
