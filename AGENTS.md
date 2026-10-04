# AGENTS.md

OpenCode reads this file at the start of every session. It and `CONTRIBUTING.md` are **the two only
entry points**: a session that reads neither has no rules at all, so any rule worth having is in one
of the two, not in a file one of them points at.

**Read in this order, and stop where your question is answered.**

1. [`CONTRIBUTING.md`](CONTRIBUTING.md) — the process and the commands. Read the section, not the
   file.
2. This file — what a change is here, and what of the usual is wrong in this repository.
3. [`docs/workflow.md`](docs/workflow.md) — the flow written long, for when the question needs more
   than a table. [`specs/README.md`](specs/README.md) is the spec format.

**Prefer naming a file to repeating a value.** A count copied here goes stale in the next change and
nothing fails when it does; a name like "the table in `CONTRIBUTING.md`" or "`xtask/src/spec.rs`"
keeps pointing at the truth. What follows names files rather than restating what they say.

## What a change is

One GitHub issue, one branch, one pull request. **The pull request body is the record of it** — what
changed, what was decided and why not the alternative, what proves it, what was observed. `pr open`
refuses a body with a section left empty, so the record cannot be submitted unfilled.

**Two branches when the change has to agree a decision before writing code.** The question that
decides it: _does this change force a choice between more than one defensible answer?_

| Branch              | Carries           | Keyword  | Precondition                         |
| ------------------- | ----------------- | -------- | ------------------------------------ |
| `NNN-slug`          | the whole change  | `Closes` | —                                    |
| `NNN-slug-deciding` | the spec, no code | `Refs`   | —                                    |
| `NNN-slug-building` | the code          | `Closes` | the deciding one merged, spec agreed |

Two forms of change and three branch names, and no suffix meaning "no decision to take" — its
absence is what says that, so the question is answerable from the name. A change to the gate, the
hooks, CI or these documents is an ordinary one-stage change: no permission needed.

`NNN` is the issue number, and the slug is derived **once**, from the issue's title, and derived
again never: a change of two stages reads it back from `specs/NNN-slug.md`, and a change of one
stage from its own branch name, which is the only place that change writes it. Renaming an issue
does not rename anything.

## The six steps

1. **An issue.** Two or three sentences: a title and what is wanted. Not acceptance criteria — those
   are what the spec is for. What surfaces mid-change and opens future work is another issue, never
   a section of the current spec.
2. **A branch.** `cargo xtask change open <issue> [deciding|building]`, with no stage at all for a
   change of one stage. It derives the slug, links the branch to the issue and checks it out, and it
   refuses a `building` branch until the deciding stage has merged.
3. **Settle the question by reading the model.** `docs/model.md` and `docs/diagram-model.md` are the
   design. If the slice needs a rule the model does not have, the model changes first, in the same
   increment. A spec names the sections it implements or amends and does not restate them.
4. **Decide, if the question said yes.** Write `specs/NNN-slug.md` on `-deciding`. Where the subject
   renders, show both options with a `render` marker — the drawing is the argument. The `Why not` in
   each decision paragraph is what makes it an agreement rather than an opinion.
5. **Build.** On `-building`, against the merged spec. `cargo xtask pr open` refuses to open it if
   the deciding pull request did not merge or the spec still reads `_pending_`.
6. **Draw the case.** Where the change moves a picture, `cargo xtask render` and look at it. A
   picture a document shows is either generated from a description the file carries or labelled
   hypothetical; nothing can tell them apart for you.

## Commands

```sh
cargo xtask check                   # the whole quality gate; the hook and CI run this and nothing else
cargo xtask fix                     # every automatic fix the gate knows about
cargo xtask setup                   # install the Node tooling the gate needs
cargo xtask render                  # rewrite every picture a tracked document carries, from its description
cargo xtask change open 163         # open the branch of a change of one stage
cargo xtask change open 163 deciding # open the deciding stage, for a change of two stages
cargo xtask change use 163 building # check out the branch of one stage of an issue
cargo xtask change status 163       # where the change stands, and what to do next
cargo xtask pr body                 # write target/pr-body.md; fill it, then `cargo xtask pr open`
cargo run -p monospace-cli          # run the application; a bare `cargo run` is ambiguous
cargo test --workspace              # tests only, for a faster loop
cargo insta review                  # accept a moved snapshot; report what moved first
```

`xtask` has **no dependencies, deliberately**: it guards the dependency policy, so it must not be
the first thing to bend it. Orchestrating subprocesses and propagating exit codes is `std`'s job.

## The rules, and where they live

`CONTRIBUTING.md` holds the rules; they are not repeated here. The ones worth knowing exist:

- **Language.** Everything that lands in the repository — code, comments, `README`, `docs/`, specs,
  commit messages, pull request bodies, CLI output, error messages — is **English**. Conversation
  with the maintainer may be in any language.
- **The core stays portable.** `monospace-core` holds all domain logic; `monospace-cli` holds none,
  and the core's public API may not assume a CLI, a TUI or a terminal. The gate's `wasm` step
  enforces it rather than the prose. `monospace` is reserved for the interactive TUI of a later
  phase and no other binary may take it.
- **Toolchain.** Rust edition 2024, pinned exactly in `rust-toolchain.toml` with its components and
  the `wasm32-unknown-unknown` target; no nightly-only features. Node, at the version in `.nvmrc`,
  backs the checks Rust cannot perform.

## Habits the gate cannot check

- **Say it in the first paragraph when the code and a document disagree.** Do not reconcile
  silently, and do not change code to match a document without asking which of the two is wrong.
- **Do not argue that changing something repeatedly is expensive.** Where that is true the cost is
  paid by a test, a migration or a public API, and written there.

## Facts that are in the code and in no document

- **The `wasm` step compiles three crates, not one**: `monospace-core`, `monospace-diagram` and
  `monospace-glyph-sets`. A `std::io` or `std::process` reach in the core fails there even where it
  works locally.
- **`monospace-cli` has two modes, and `render` depends on the split.** Bare, it prints the
  demonstration; given a path, it prints one picture and nothing else, because a rendering that goes
  into a Markdown fence cannot arrive wrapped in prose. Do not add output to the path form.

### Two plugins, and what a harness needs to know

`.opencode/plugins/` holds the two plugins nothing in the gate can execute. They are the only thing
that installs the git hooks here and the only thing that puts a session id in a commit, so a failure
in either is silent — check that they loaded before assuming either works.

- **They log a line when they arm, and it reaches the TUI, not the log file.** Measured: that file
  has never held a `[monospace]` line whether the plugin loaded or died. Its own `loading plugin`
  and `failed to load plugin` entries are the readable signal, and the second carries the cause.
- **A missing line is usually not the API moving.** The loader dies at module resolution first, and
  a local plugin's bare imports resolve from **this project's `node_modules`** — so whatever a
  plugin imports has to be a dependency here. What it must export is a default object carrying an
  `id` and a `setup`.
- **A reload unloads a plugin without warning.** Anything that checks out these files, a rebase
  among them, drops both until they load again, and a session that loses `session-trailer.js` loses
  the variable with it.
- **The trailer key differs by client and the hook reads both.** `commit-msg` reads
  `MONOSPACE_SESSION_ID` and `CLAUDE_CODE_SESSION_ID`; a commit made outside an agent shell
  correctly carries neither.

## Running a session

Every call re-reads the whole context, so a session costs its length squared. Two habits follow:

- **Read the part, not the file.** `docs/model.md`, `docs/diagram-model.md` and `docs/glyph-sets.md`
  are large enough that opening one whole is a decision. Take a line range, or grep with context.
- **Delegate a lookup that spans files**, so the files never enter this context.

## Known gaps

- **`commitlint` is not in the gate.** It runs from the `commit-msg` hook, so a commit made where
  the hooks are not installed gets no convention check and CI does not catch it either.
  [Issue #162](https://github.com/andresmoschini/monospace/issues/162) tracks putting it in
  `cargo xtask check`, which is the only definition of green.
