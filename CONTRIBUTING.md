# Contributing

Everything here assumes a clean machine. If a step needs something this document does not mention,
that is a defect in the document.

## What you need first

- **Git.**
- **[rustup](https://rustup.rs).** Not a Rust toolchain — rustup installs the right one by itself.
  `rust-toolchain.toml` pins the exact compiler along with `rustfmt`, `clippy`, `rust-src` and the
  `wasm32-unknown-unknown` target, and rustup reads that file.
- **Node.** The version is in `.nvmrc`. Four of the checks are npm packages with no Rust equivalent;
  [ADR-0004](docs/decisions/0004-node-toolchain-for-the-non-rust-checks.md) explains how a Rust
  project ended up with a second toolchain.
- **[GitHub CLI](https://cli.github.com).** `gh`, authenticated with `gh auth login`.
  `cargo xtask spec` drives it to read an issue, open a branch linked to that issue and move the
  issue's label ([ADR-0034](docs/decisions/0034-let-xtask-own-the-feature-branch.md)).

## Setup

```sh
rustup toolchain install # reads rust-toolchain.toml
cargo xtask setup        # runs npm ci
```

The first downloads a toolchain even if you already have the same version under a different name,
because rustup treats `stable` and `1.98.1` as different installations. The second takes about half
a minute the first time.

There is no third step for the hooks: opening a Claude Code session installs them, through a
`SessionStart` entry in `.claude/settings.json`. Working another way, run
`git config core.hooksPath .claude/git-hooks` yourself — and read the next section first, because
without it nothing checks your commits until CI does.

## Everyday commands

```sh
cargo xtask check          # the whole quality gate, about 3.5 seconds
cargo xtask fix            # apply every automatic fix the gate knows about
cargo xtask spec use 23    # put this clone on the active branch of feature 23
cargo run -p monospace-cli # run the command-line application
cargo test --workspace     # tests only, when you want a faster loop
```

`cargo run` without `-p` does not work: the workspace has more than one binary, so Cargo cannot
pick.

## Starting work

Two flows live here, and the first thing to settle is which one you are in.

- **A feature** is something the tool will be able to do that it cannot do today. It goes through
  Spec Kit: an issue, a directory under `specs/`, and three staged branches.
- **A tooling change** is anything about how the repository is worked on — the gate, the hooks, CI,
  these documents, the workflow itself. It goes through an ADR and a series of commits. No spec
  directory, no stage branches, no Spec Kit command.

The question that decides it: does this change what `monospace` can draw, or does it change how we
work on it? Both still start from an issue and end in a pull request; only the middle differs.

### One issue, and the labels on it

A feature is represented by exactly one issue. There is no parent issue and there are no story
sub-issues ([ADR-0033](docs/decisions/0033-keep-the-flow-state-in-labels-on-one-issue.md)). That
issue's number _is_ the feature's number, shared by the directory and by all three branches
([ADR-0024](docs/decisions/0024-take-the-feature-number-from-its-issue.md)). The numbering skips
wherever an issue was not a feature, and that is expected.

Labels carry the state of the flow, and they are the only place it lives: not a board field, not a
milestone. The board reads the issue.

| Label   | What it means                                               |
| ------- | ----------------------------------------------------------- |
| `wish`  | Someone wants this. Nothing is specified yet.               |
| `spec`  | The spec branch is open, or its pull request is in review.  |
| `plan`  | The spec merged. The plan branch is open.                   |
| `doing` | The plan and the tasks merged. Implementation is under way. |

There is no label for finished work: the implementation pull request closes the issue, and a closed
issue is the end state.

These four are one axis. The kind labels — `capability` for a wish someone had, `foundational` for
what the design demands and nobody asked for, `tooling` for the repository itself — are another, and
the two coexist on the same issue. A `tooling` issue never enters the spec flow, so it never carries
a state label.

An issue does not grow. A title and two or three sentences, never acceptance criteria, requirements
or examples ([ADR-0023](docs/decisions/0023-direction-and-backlog-in-a-github-project.md)). Once the
spec exists the spec is the source of truth, and the issue is a pointer back to where the wish was
first stated. An issue that accumulates requirements is a spec written where no Spec Kit command
will read it.

### The three stages

A spec crosses three stages, potentially with three different people. Each stage is its own branch
and its own pull request against `main`, and the merge of each one is the handoff to the next
([ADR-0032](docs/decisions/0032-split-a-spec-into-three-staged-branches.md)).

| Stage          | Branch          | Label   | Requires in `main`    |
| -------------- | --------------- | ------- | --------------------- |
| Spec           | `NNN-slug-spec` | `spec`  | —                     |
| Plan           | `NNN-slug-plan` | `plan`  | `spec.md`             |
| Implementation | `NNN-slug-impl` | `doing` | `plan.md`, `tasks.md` |

`cargo xtask spec` opens each of them: it creates the branch, links it to the issue, moves the
label, and points Spec Kit at the feature directory
([ADR-0034](docs/decisions/0034-let-xtask-own-the-feature-branch.md)). It never writes the spec
itself — `/speckit-specify` creates the directory and the file, as it always did.

```sh
cargo xtask spec new 23          # opens the spec stage for issue #23
cargo xtask spec stage 23 plan   # after the spec pull request merged
cargo xtask spec stage 23 impl   # after the plan pull request merged
```

`stage` refuses to open a stage whose predecessor has not merged, and says which file it could not
find in `origin/main`. That refusal is the whole point of the handoff: the precondition is a fact
about `main`, not a judgement about a branch.

Inside each stage, the Spec Kit commands that belong to it, one per session:

```text
spec branch   /speckit-specify   then /speckit-clarify if the spec leaves open questions
plan branch   /speckit-plan      then /speckit-tasks
impl branch   /speckit-implement
```

The plan stage runs two of them, which is why the implementation stage requires two files in `main`
rather than one.

Someone who has just cloned, or who is coming back to a feature after working on another, does not
need to know any of the branch names:

```sh
cargo xtask spec use 23
```

It reads the issue's label, checks out the branch of whatever stage is active, and prints what to
run next. It touches no label and creates no remote branch.

Features 001 to 006 predate all of this and keep the numbers they were given.

### Names that carry the number

The directory and all three branches carry the feature's number, so that one string finds every part
of it:

```text
issue     #23
branches  023-read-a-diagram-description-spec
          023-read-a-diagram-description-plan
          023-read-a-diagram-description-impl
directory specs/023-read-a-diagram-description/
```

The slug comes from the issue title, lowercased and hyphenated, at most forty characters. `xtask`
derives it once, when the spec stage is opened, and reads it back from the directory afterwards, so
renaming the issue later does not rename anything.

Nothing yet checks that every directory under `specs/` is named this way and that no two share a
number. That check belongs in `cargo xtask check` and is tracked by issue #26.

### Commits during implementation

Each commit ticks exactly the checkboxes in `tasks.md` that it completed, and leaves
`cargo xtask check` passing. Those two rules together decide how big a commit is, and you do not get
to choose: the hooks run the gate and `--no-verify` is forbidden, so a commit can only exist at a
boundary where the tree is green. A task that is green on its own gets a commit. A group of tasks
that only reaches green together gets one commit for the group, ticking all of their boxes.

### Opening and closing the pull request

The keywords that close an issue go in the pull request's body, never in a commit message. Three
reasons, and the third is the one that decides it:

- The link is visible before the merge. Only the pull request gives you that; a keyword in a commit
  is invisible until it lands.
- A wrong number is edited out of a body. In a commit it is a history rewrite, and a rewrite here
  owes the gate a run on **every** rewritten commit rather than only the tip.
- No single commit is "the" one that closes work that took several.

Which keyword depends on the stage. Only the last pull request of a feature finishes the issue:

- **Spec and plan** pull requests say `Refs #N`. The stage is done; the issue is not.
- **Implementation** pull requests say `Closes #N`.

```sh
gh pr create --base main --title "..." --body "Refs #23"
gh pr view N --json closingIssuesReferences   # confirm GitHub parsed a Closes
gh pr merge --merge --delete-branch
```

A tooling change has one pull request and therefore one keyword: `Closes #N`.

## The quality gate

`cargo xtask check` is the whole gate. Why it is the only definition of "green", and why neither the
hook nor CI may add a check of its own, is
[One definition of green](.specify/memory/constitution.md#iii-one-definition-of-green-non-negotiable)
in the constitution. What follows here is how it behaves.

Every step runs even after one fails, so a single run reports everything wrong rather than making
you fix problems one at a time. A step passes or fails on its exit code alone.

| Step           | What it checks                                                                 |
| -------------- | ------------------------------------------------------------------------------ |
| `fmt`          | Rust formatting, `cargo fmt --check`                                           |
| `prettier`     | Formatting of Markdown, JSON and JSONC, including prose width                  |
| `markdownlint` | Markdown structure: heading levels, duplicate headings, bare URLs, code fences |
| `editorconfig` | Line endings, final newlines and trailing whitespace on every tracked file     |
| `cspell`       | Spelling, in code and prose alike                                              |
| `clippy`       | Lints, including `pedantic`, with warnings denied                              |
| `build`        | The workspace compiles, tests and all                                          |
| `wasm`         | `monospace-core` still compiles for `wasm32-unknown-unknown`                   |
| `test`         | Unit tests, integration tests and doctests                                     |
| `doc`          | `cargo doc` builds, with broken intra-doc links denied                         |

### Which tool owns which file

Two tools disagreeing about the same file is a gate that can never go green, so each concern has one
owner.

- **Rust files** belong to `rustfmt` for layout and `clippy` for everything else. The lints live in
  `[workspace.lints]` in the root `Cargo.toml`, not on a command line, so your editor shows the same
  ones the gate enforces.
- **Markdown** is split. Prettier owns anything it can rewrite, line width included, which is why
  markdownlint's `MD013` is off: prettier guarantees the width by reflowing, while markdownlint
  could only report it and leave you to re-cut paragraphs by hand. markdownlint owns what prettier
  cannot express, like a heading level that skips or a code fence with no language.
- **Everything else tracked** — `LICENSE`, the TOML files, the dotfiles — belongs to
  `editorconfig-checker`. Prettier cannot even infer a parser for those, so without it they would go
  unchecked.
- **`.editorconfig`** is read by prettier and by `editorconfig-checker`, so indentation and line
  endings are configured once and obeyed by both.

### Automatic fixes

`cargo xtask fix` runs `fmt`, `prettier`, `markdownlint` and `editorconfig` in that order, each in
its writing mode instead of its checking mode. Order is not incidental here the way it is for
`check`: these steps rewrite the same files `check` only reads, so a formatter that ran last would
win regardless of which one was "right". Content formatters run first; `editorconfig` runs last
because it owns files none of the others touch — `LICENSE`, the TOML files, the dotfiles — and
otherwise only confirms what the earlier steps already left clean.

`clippy` and `cspell` have no fix step. `cspell` cannot fix a spelling on its own, and
`clippy --fix` is deliberately left out of the automatic command:

```sh
cargo clippy --fix --workspace --all-targets --allow-dirty --allow-staged -- -D warnings
```

`--allow-dirty` and `--allow-staged` are required because clippy otherwise refuses to touch a
working tree that is not clean, which it usually is mid-task. That refusal exists because a fix that
turns out wrong should be a `git diff` away from undone, not mixed irreversibly into work already in
progress — so run it on a tree you can afford to diff and revert, read the diff before committing,
and expect it to leave some warnings behind: `pedantic`, which this project denies, includes lints
that need a human judgment call rather than a mechanical rewrite. `cargo xtask check` afterward is
what confirms which ones remain.

`cargo xtask fix` does not guarantee `cargo xtask check` passes afterward — beyond what `clippy` and
`cspell` never touch, `markdownlint` and `editorconfig` can both report violations they know about
but cannot rewrite.

### When a check fails

- **`cspell` flags a word.** Decide which it is. A misspelling gets fixed. A British spelling gets
  changed, not added — the repository writes American English, and adding those one at a time would
  rebuild the dictionary that was deliberately removed. A genuine term that no dictionary knows goes
  in `project-words.txt`, which carries the rule for adding to it.
- **The gate refuses to start**, saying the Node tooling is missing or out of date. Run
  `cargo xtask setup`. It refuses rather than running the Rust half, because a summary reading "all
  6 checks passed" when ten are configured is worse than an error.

## The hooks

`pre-commit` runs the gate. `commit-msg` checks the message with commitlint. Both live in
`.claude/git-hooks/`.

**They only run if they were installed, and only a Claude Code session installs them.** A commit
made from a plain terminal in a clone where no session has opened runs no hooks at all, and nothing
says so — the commit simply succeeds. That is deliberate, not an oversight:
[ADR-0005](docs/decisions/0005-install-the-git-hooks-from-claude-code.md) records the trade and the
cost. CI runs the same gate on every push and pull request, so the boundary that actually holds is
there; the hooks are fast feedback in front of it. Check yours with `git config core.hooksPath`.

**The pre-commit hook checks your working tree, not what you staged.** With unstaged changes
present, or after `git add -p`, it verifies files that are not the ones being committed, so a commit
can pass and still be broken. Stashing to close that gap risks losing work if the hook is
interrupted, which is the worse failure. If you stage selectively, run `cargo xtask check` on a
clean tree before trusting it.

Never commit with `--no-verify`;
[One definition of green](.specify/memory/constitution.md#iii-one-definition-of-green-non-negotiable)
makes that a rule. What is worth adding here is why it bites: a bypassed gate is worse than no gate,
because the log then claims a green history that was never checked.

## Commits

Messages follow [Conventional Commits](https://www.conventionalcommits.org), enforced by the
`commit-msg` hook.

Which prefix to use follows from
[Structural and behavioral change never share a commit](.specify/memory/constitution.md#v-structural-and-behavioral-change-never-share-a-commit).
The mapping is:

- **`refactor`** — structural. Behavior does not change, the existing tests pass unchanged, and no
  test is added or modified.
- **`feat`, `fix`** — behavioral. Something the program does is different.
- **`build`, `ci`, `docs`, `style`, `chore`, `test`** — neither, which is most of the tooling in
  this repository.

How much goes in one commit, while implementing a spec, is settled by
[Commits during implementation](#commits-during-implementation) rather than by taste.

Write the body for someone who was not there. What the diff does is visible; why it does that is
not.

One thing to know about the body: do not let a colon-terminated word start a line. commitlint reads
`word:` at the beginning of a line as a footer token, splits the message there, and warns that the
footer has no blank line before it. It is only a warning, so it lands unnoticed. Reword with an em
dash, or re-wrap so the word sits mid-line.

### Fixing a commit

Which corrections belong in the commit that got them wrong, and which get a commit of their own, is
[Fixing a commit](.specify/memory/constitution.md#fixing-a-commit) in the constitution. What is
specific to this repository is that the rule holds after pushing, which is not the usual convention.
It works here because a feature branch has an owner, and whoever pulls someone else's branch accepts
that it can be rewritten underneath them. The limits are `main`, which is never rewritten, and
`--force-with-lease`, which aborts instead of clobbering an update you had not seen. The cost is
that GitHub marks inline review comments on a rewritten commit as outdated.

Rewriting means proving the branch green again, on each rewritten commit rather than on the tip
alone, because neither rebase nor cherry-pick fires the hook —
[One definition of green](.specify/memory/constitution.md#iii-one-definition-of-green-non-negotiable).

An accepted ADR is the exception: its conclusion is never edited, whatever the commit history does.
[The decisions README](docs/decisions/README.md) owns that rule.

### The session trailer

A commit made from a Claude Code session can carry two trailers, and they are different handles on
the same conversation rather than the same one twice.

- **`Claude-Resume`** holds the local session id. Reopen the conversation with
  `claude --resume <id>`. The `commit-msg` hook writes it, so it is present whenever the hooks are.
- **`Claude-Session`** holds a URL that opens the session in a browser. Claude Code writes it itself
  when Remote Control is enabled, which is a setting outside this repository — so it is present
  sometimes and absent otherwise.

List them across the history with:

```sh
git log --format='%h %(trailers:key=Claude-Resume,valueonly)'
```

[ADR-0007](docs/decisions/0007-rename-the-session-trailer-to-claude-resume.md) covers why they have
separate keys, and [ADR-0006](docs/decisions/0006-record-the-claude-session-in-commit-trailers.md),
which it supersedes, covers why either is a trailer rather than a plain line — a non-trailer line at
the end of a message silently invalidates `Co-Authored-By` along with it.

It is a convenience, not a record. Transcripts live outside the repository and do not survive a new
machine, so the reasoning that matters still belongs in the commit body or in an ADR. If a commit
body only makes sense with the transcript open, the body is wrong.

## Cross-references

Cite a section of another document by name — in practice, by anchor — and not by number. The rule
and its reasoning are in [Cross-references](.specify/memory/constitution.md#cross-references).

Nothing checks any of it. `markdownlint` validates a link fragment against the headings of the same
file and stops there, so a link to a file that does not exist, or to an anchor in a different file
that does not exist, passes the gate.
[Issue #13](https://github.com/andresmoschini/monospace/issues/13) tracks closing it.

Records under `docs/decisions/` written before this convention keep their numbered citations. They
were true when written, and an accepted record is not edited for style.

## Decisions and notes

Three documents, and
[Where a rationale goes](.specify/memory/constitution.md#where-a-rationale-goes) says which takes
what. The procedure and templates are in each: [`docs/decisions/`](docs/decisions/README.md) for
decision records, [the learning log](docs/learning-log.md) for what was learned, and a feature's own
`research.md` for investigation local to that slice.

Where specs live, and how a slice goes from spec to plan to tasks, is
[Spec Kit is the workflow](.specify/memory/constitution.md#spec-kit-is-the-workflow).
