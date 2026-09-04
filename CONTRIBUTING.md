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

## Setup

```sh
rustup toolchain install              # reads rust-toolchain.toml
cargo xtask setup                     # runs npm ci
git config core.hooksPath .githooks   # activates the hooks
```

The first command downloads a toolchain even if you already have the same version under a different
name, because rustup treats `stable` and `1.98.1` as different installations. The second takes about
half a minute the first time.

The third is per clone. Hooks are not part of a checkout, so this is the one piece of setup Git
cannot do for you. Undo it with `git config --unset core.hooksPath`.

## Everyday commands

```sh
cargo xtask check          # the whole quality gate, about 3.5 seconds
cargo run -p monospace-cli # run the command-line application
cargo test --workspace     # tests only, when you want a faster loop
```

`cargo run` without `-p` does not work: the workspace has more than one binary, so Cargo cannot
pick.

## The quality gate

`cargo xtask check` is the only definition of "green". The pre-commit hook runs it, CI runs it, and
neither adds anything of its own — if they could, passing locally and passing in CI would stop
meaning the same thing.

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

### When a check fails

- **`cspell` flags a word.** Decide which it is. A misspelling gets fixed. A British spelling gets
  changed, not added — the repository writes American English, and adding those one at a time would
  rebuild the dictionary that was deliberately removed. A genuine term that no dictionary knows goes
  in `project-words.txt`, which carries the rule for adding to it.
- **The gate refuses to start**, saying the Node tooling is missing or out of date. Run
  `cargo xtask setup`. It refuses rather than running the Rust half, because a summary reading "all
  6 checks passed" when ten are configured is worse than an error.

## The hooks

`pre-commit` runs the gate. `commit-msg` checks the message with commitlint.

**The pre-commit hook checks your working tree, not what you staged.** With unstaged changes
present, or after `git add -p`, it verifies files that are not the ones being committed, so a commit
can pass and still be broken. Stashing to close that gap risks losing work if the hook is
interrupted, which is the worse failure. If you stage selectively, run `cargo xtask check` on a
clean tree before trusting it.

Never commit with `--no-verify`. If a hook fails, fix the cause or say so — a bypassed gate is worse
than no gate, because the log then claims a green history that was never checked.

## Commits

Messages follow [Conventional Commits](https://www.conventionalcommits.org), enforced by the
`commit-msg` hook.

One task per commit, and the type carries a distinction the project cares about:

- **`refactor`** — structural. Behavior does not change, the existing tests pass unchanged, and no
  test is added or modified.
- **`feat`, `fix`** — behavioral. Something the program does is different.

Keeping those apart is Kent Beck's rule: make the change easy, then make the easy change. They never
share a commit, and the prefix is what makes the difference visible in the log without reading
diffs.

`build`, `ci`, `docs`, `style`, `chore` and `test` cover work that is neither, which is most of the
tooling in this repository.

Write the body for someone who was not there. What the diff does is visible; why it does that is
not.

## Decisions and notes

Architecture decisions are records under `docs/decisions/`, written when the decision is taken
rather than reconstructed later. [Its README](docs/decisions/README.md) has the procedure, the
template and the rule that an accepted record is never edited to change its conclusion.

Anything learned along the way that is not a decision goes in `docs/learning-log.md` instead.
