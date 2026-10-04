# Architecture Decision Records

This directory holds the project's architecture decision records (ADRs). Each file records one
decision: what was decided, what else was considered, and why the alternatives were rejected.

The format is MADR 4.0 with two local additions. The reasoning is in
[ADR-0000](0000-use-madr-for-architecture-decisions.md); the template to copy is
[`adr-template.md`](adr-template.md).

## Index

| ADR                                                                                 | Title                                                                                  | Status                               |
| ----------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------- | ------------------------------------ |
| [0000](0000-use-madr-for-architecture-decisions.md)                                 | Use MADR for architecture decision records                                             | accepted                             |
| [0001](0001-virtual-cargo-workspace-under-crates.md)                                | Lay out the project as a virtual Cargo workspace under `crates/`                       | accepted                             |
| [0002](0002-no-minimum-supported-rust-version.md)                                   | Declare no minimum supported Rust version, for now                                     | accepted                             |
| [0003](0003-pin-the-toolchain-exactly.md)                                           | Pin the Rust toolchain to an exact version                                             | accepted                             |
| [0004](0004-node-toolchain-for-the-non-rust-checks.md)                              | Use a Node toolchain for the checks Rust cannot perform                                | accepted                             |
| [0005](0005-install-the-git-hooks-from-claude-code.md)                              | Install the git hooks from the Claude Code session                                     | accepted                             |
| [0006](0006-record-the-claude-session-in-commit-trailers.md)                        | Record the Claude Code session in a commit trailer                                     | superseded by 0007                   |
| [0007](0007-rename-the-session-trailer-to-claude-resume.md)                         | Rename the session trailer to `Claude-Resume`                                          | accepted                             |
| [0008](0008-compose-overlapping-cells-with-three-state-arms.md)                     | Compose overlapping cells with three-state arms and two stamp modes                    | accepted                             |
| [0009](0009-degrade-a-cell-to-its-base-stroke.md)                                   | Degrade a cell to its base stroke when no character matches                            | accepted                             |
| [0010](0010-separate-position-and-size.md)                                          | Separate position and size instead of one rectangle                                    | accepted                             |
| [0011](0011-expose-cell-for-testing-stamping.md)                                    | Expose `cell` so stamping can be checked without rendering                             | accepted                             |
| [0012](0012-one-stroke-per-cell.md)                                                 | Give a cell one stroke, with none per arm                                              | superseded by 0037                   |
| [0013](0013-key-a-rule-by-stroke-per-side.md)                                       | Key a glyph rule by a stroke per side                                                  | accepted                             |
| [0014](0014-collapse-glyph-sets-into-a-catalog.md)                                  | Collapse glyph sets into a single catalog                                              | accepted                             |
| [0015](0015-represent-a-stroke-as-a-string.md)                                      | Represent a stroke as an owned `String`                                                | accepted                             |
| [0016](0016-return-a-string-from-render.md)                                         | Return a `String` from `render`                                                        | accepted                             |
| [0017](0017-ask-the-cell-whether-it-is-decided.md)                                  | Ask the cell whether it is decided, and let `stamp` skip a decided target              | accepted                             |
| [0018](0018-mirror-the-decided-skip-in-above.md)                                    | Mirror the decided-cell skip in `Above`, accepting a second unverifiable branch        | accepted                             |
| [0019](0019-represent-a-glyph-as-a-grapheme-cluster.md)                             | Represent a glyph as a validated grapheme cluster                                      | accepted                             |
| [0020](0020-scope-cargo-xtask-fix-to-deterministic-fixers.md)                       | Scope `cargo xtask fix` to deterministic fixers, and fix `clippy` by hand              | accepted                             |
| [0021](0021-move-the-spec-home-to-spec-kit.md)                                      | Move the spec home to Spec Kit's `specs/` and freeze `docs/specs/`                     | accepted; superseded in part by 0024 |
| [0022](0022-non-interactive-cli-before-the-tui.md)                                  | Build the non-interactive CLI before the interactive TUI                               | accepted                             |
| [0023](0023-direction-and-backlog-in-a-github-project.md)                           | Keep direction and the backlog in a GitHub Project, and dissolve the roadmap           | accepted; superseded in part by 0033 |
| [0024](0024-take-the-feature-number-from-its-issue.md)                              | Take a feature's number from its GitHub issue                                          | accepted                             |
| [0025](0025-every-feature-has-a-parent-issue.md)                                    | Give every feature a parent issue, labeled `capability` or `foundational`              | superseded by 0033                   |
| [0026](0026-represent-a-cell-as-a-sum-of-strokes-and-a-literal.md)                  | Represent a cell as a sum of a stroke cell and a literal glyph                         | accepted                             |
| [0027](0027-control-token-cost-through-session-discipline.md)                       | Control token cost through session discipline, not by trimming artifacts               | accepted                             |
| [0028](0028-give-each-fragment-its-own-cell-rule.md)                                | Give each fragment its own cell rule instead of a cell parameter                       | accepted                             |
| [0029](0029-draw-a-line-end-as-one-arm.md)                                          | Draw a line's end as one arm, and keep an arrow's head a chosen glyph                  | accepted; superseded in part by 0063 |
| [0030](0030-drop-extent-until-a-caller-needs-it.md)                                 | Drop extent from the shape abstraction until a caller needs it                         | accepted                             |
| [0031](0031-a-shape-draws-into-a-surface.md)                                        | A shape draws into a surface, not into a buffer                                        | accepted                             |
| [0032](0032-split-a-spec-into-three-staged-branches.md)                             | Split a spec into three staged branches                                                | superseded by 0051                   |
| [0033](0033-keep-the-flow-state-in-labels-on-one-issue.md)                          | Keep the flow state in labels on one issue                                             | accepted                             |
| [0034](0034-let-xtask-own-the-feature-branch.md)                                    | Let xtask own the feature branch                                                       | accepted                             |
| [0035](0035-keep-the-cli-demo-format-out-of-the-model.md)                           | Keep the CLI's demo file format out of the model                                       | accepted                             |
| [0036](0036-hold-every-table-but-light-outside-the-core.md)                         | Hold every glyph table but Light in a crate outside the core                           | accepted                             |
| [0037](0037-give-each-arm-its-own-stroke.md)                                        | Give each arm its own stroke, ending one stroke per cell                               | accepted                             |
| [0038](0038-hold-the-diagram-model-in-a-crate-above-the-core.md)                    | Hold the diagram model in a crate above the core                                       | accepted                             |
| [0039](0039-a-diagram-shape-is-its-own-entity.md)                                   | A diagram shape is its own entity, held in a closed set of kinds                       | accepted                             |
| [0040](0040-let-each-shape-answer-its-own-anchor-points.md)                         | Let each shape answer its own anchor points, and allow it to answer none               | accepted                             |
| [0041](0041-resolve-a-position-through-a-reference.md)                              | Resolve a position through a reference, and draw nothing that cannot be resolved       | accepted                             |
| [0042](0042-draw-a-diagram-front-to-back-into-a-given-window.md)                    | Draw a diagram front to back, into a window the caller gives                           | accepted                             |
| [0043](0043-let-the-buffer-record-who-decided-each-cell.md)                         | Let the buffer record who decided each cell                                            | accepted                             |
| [0044](0044-let-the-endpoint-order-break-a-tied-route.md)                           | Let the endpoint order break a tied route                                              | absorbed into `shape::connector`     |
| [0045](0045-pin-every-arrow-arrangement-as-a-reviewed-snapshot.md)                  | Pin every arrow arrangement as a reviewed snapshot                                     | accepted                             |
| [0046](0046-rank-a-route-instead-of-bounding-it.md)                                 | Rank a route instead of bounding it                                                    | superseded by 0055                   |
| [0047](0047-let-a-route-cross-no-cell-twice.md)                                     | Let a route cross no cell twice                                                        | superseded by 0055                   |
| [0048](0048-let-the-travel-pick-the-side-of-a-mirrored-route.md)                    | Let the arrow's own travel pick the side of a mirrored route                           | absorbed into `shape::connector`     |
| [0049](0049-derive-a-route-by-searching-the-lines-a-turn-can-sit-on.md)             | Derive a route by searching the lines a turn can sit on                                | absorbed into `shape::connector`     |
| [0050](0050-record-a-decision-at-the-boundary-it-cannot-cross.md)                   | Record a decision at the boundary it cannot cross                                      | accepted                             |
| [0051](0051-stop-at-the-decision-sheet-and-merge-three-stages-into-two.md)          | Stop at the decision sheet, and merge three stages into two                            | accepted                             |
| [0052](0052-show-the-rendering.md)                                                  | Show the rendering                                                                     | accepted                             |
| [0053](0053-report-a-characterization-instead-of-reviewing-it.md)                   | Report a characterization instead of reviewing it                                      | accepted                             |
| [0054](0054-open-the-pull-request-from-xtask-in-two-verbs.md)                       | Open the pull request from xtask, in two verbs                                         | accepted                             |
| [0055](0055-an-arrows-route-is-a-path-and-nothing-bounds-it.md)                     | An arrow's route is a path, and nothing bounds it                                      | accepted                             |
| [0056](0056-give-opencode-its-own-instruction-file.md)                              | Give OpenCode its own instruction file, and leave Claude Code's alone                  | accepted                             |
| [0057](0057-install-the-speckit-opencode-integration-beside-the-claude-one.md)      | Install the spec-kit opencode integration beside the claude one                        | accepted                             |
| [0058](0058-install-the-git-hooks-from-an-opencode-session-too.md)                  | Install the git hooks from an OpenCode session too                                     | accepted                             |
| [0059](0059-normalize-what-the-speckit-cli-writes-or-git-refuses-it.md)             | Normalize what the Speckit CLI writes, or git refuses it                               | accepted                             |
| [0060](0060-stamp-the-opencode-session-into-the-commit.md)                          | Stamp the OpenCode session into the commit, without an environment variable of its own | accepted                             |
| [0062](0062-reduce-what-the-instruction-files-restate.md)                           | Reduce what the instruction files restate, and say what decides it                     | accepted                             |
| [0063](0063-supersede-adr-0029-and-move-its-reasoning-into-the-arrow-module.md)     | Supersede ADR-0029's second half, and move its reasoning into the arrow module         | accepted                             |
| [0064](0064-give-each-generated-picture-the-carrier-that-can-reach-its-subject.md)  | Give each generated picture the carrier that can reach its subject                     | accepted                             |
| [0065](0065-name-a-shape-for-what-it-joins-not-for-the-head-it-may-carry.md)        | Name a shape for what it joins, not for the head it may carry                          | accepted                             |
| [0066](0066-let-a-caller-choose-an-identity-and-say-where-the-numbering-resumes.md) | Let a caller choose an identity, and say where the numbering resumes                   | accepted                             |
| [0067](0067-displace-a-figure-holding-a-reference-by-growing-its-offsets.md)        | Displace a figure holding a reference by growing its offsets                           | accepted                             |
| [0068](0068-freeze-what-hung-from-a-removed-shape-where-it-stood.md)                | Freeze what hung from a removed shape where it stood                                   | accepted                             |

## Frozen

**Nothing is added here.** The last record is
[0068](0068-freeze-what-hung-from-a-removed-shape-where-it-stood.md), and there is no procedure for
writing a 0069, because the thing these records recorded now has two homes and both are cheaper than
a new file: [`docs/model.md`](../model.md) for a decision something outside the module can observe,
and a module's rustdoc under `Design notes` for one nothing outside can. That split is
[principle VI](../../.specify/memory/constitution.md#vi-decisions-recorded-at-the-altitude-they-belong-to),
and it was already the rule — what changed is that nothing points here any more, so a decision
landed in rustdoc and stayed there instead of being promoted here for a second reader nobody had.

The reasoning is kept, and a link into it is still the shortest way to say what happened.
`adr-template.md` stays for the same reason. What a record no longer does is state a rule that is
current: where one disagrees with the constitution, the constitution is what holds.

## Rules

**A record is never deleted.** Superseded, revised or `absorbed into <path>`, the file stays,
because being able to see that a decision changed — and what argument changed it — is most of what
this history is worth. Fixing a typo or a link that has gone stale is not a change to a record and
needs no ceremony.

**Records numbered 0000 to 0049 predate principle VI** and carry neither `scope` nor `commitment`.
Each was classified the next time it was cited or touched rather than in a sweep, so the directory
is mixed. `commitment` — which said whether a record was revised in place, superseded by a new one,
or load-bearing — was part of that vocabulary and is gone with the rest of it; where a record
declares one, the field is history and its meaning is in the record's own `Consequences`.

**Records written before the constitution cite `docs/brief.md`, which no longer exists.** The brief
was this project's initial spec, and it held the principles, the scope and the constraints until
[the constitution](../../.specify/memory/constitution.md) took them over. Its other content went to
[the model](../model.md), which owns the domain's design, provenance and open questions, and to the
phases, which have since moved from `docs/roadmap.md` to the
[GitHub Project](https://github.com/users/andresmoschini/projects/2). Where a record's citation
pointed at a rule that still exists, it now points at that rule's home; where it pointed at a
question the record itself answered, the link is gone and only git history has the original. The
prose is untouched: a record saying "the brief requires a stable toolchain" is reporting what it
weighed at the time, and that is not a reference to redirect.

**A link into this directory may dangle, and one does.** Four anchors in the constitution were
removed when the workflow they named went, and
[ADR-0032](0032-split-a-spec-into-three-staged-branches.md) still points at one of them. It was left
pointing rather than rewritten, because a frozen record is not edited for style and a link that no
longer resolves is a truer report of what happened than one quietly aimed at whatever replaced it.
it.
