# Frozen

Nineteen features, 24,101 lines, and nothing more is added here. Each directory is what
[Spec Kit](https://github.com/github/spec-kit) asked for while this repository used it: a `spec.md`,
a `plan.md`, a `research.md`, a `tasks.md`, a `data-model.md`, a `quickstart.md`, a `decisions.md`
and a `checklists/`, per feature.

Two numbers say why the flow changed, both measured over the last feature —
[#142](https://github.com/andresmoschini/monospace/issues/142), taking a shape out freezes what hung
from it, across its ten commits on `main`:

- **3,425 lines** were added under `specs/142-taking-a-shape-out-leaves-the-figures-th/`.
- **1,109 lines** were added under `crates/`. That is 3.1 lines of artifact per line added, and most
  of the 1,109 are tests: `diagram.rs` holds 3,969 of its 4,192 lines inside `mod tests`.

Across the whole history this tree came to 24,101 lines against 14,278 lines of Rust in the tree —
**1.7 lines of process for every line of product**, counted on what is on disk today rather than on
what any one change added.

None of that prose was wrong, and most of it was not read. It is kept because the reasoning in it is
why the code came to be this way, and because the alternatives a rejected record names are the part
that cannot be reconstructed. A decision that outlived its feature is in
[`docs/model.md`](../docs/model.md) or in a module's rustdoc; a record of what was learned is in the
[learning log](../docs/learning-log.md).

**Nothing here states a rule that is current.** Where one disagrees with
[the constitution](../.specify/memory/constitution.md), the constitution holds — including
`_pending_` markers on decision sheets that were answered before the deciding pull request merged,
and stage labels for stages that no longer exist. Read these directories as what a slice looked like
while that flow was in force.

## What replaced it

| Was                            | Is now                                                                         |
| ------------------------------ | ------------------------------------------------------------------------------ |
| `spec.md`, `plan.md`           | The pull request body, in eight sections                                       |
| `decisions.md`                 | The decision table in that body: question, answer, and why not the alternative |
| `research.md`                  | `docs/model.md`, or the rustdoc of the module it was about                     |
| `data-model.md`, `contracts/`  | The types, in rustdoc, where a caller reads them                               |
| `tasks.md`                     | The commits, and the green tree between them                                   |
| `quickstart.md`, `checklists/` | Nothing. Both existed to prove a specification was complete                    |

[`CONTRIBUTING.md`](../CONTRIBUTING.md) has the flow that replaced all of it, and the constitution's
[One question, and where the answer goes](../.specify/memory/constitution.md#one-question-and-where-the-answer-goes)
has the rule behind it.
