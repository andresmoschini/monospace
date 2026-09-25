# AGENTS.md

Guidance for OpenCode sessions in this repository.

**This file is a router, not a second copy of the rules.** The constitution calls a restated rule a
defect to remove, so nothing here restates a rule it points at. Read the source, then follow it.

## Read first, in this order

1. `.specify/memory/constitution.md` — every rule, the scope, and the testing contract. 422 lines.
2. `CLAUDE.md` — session habits, and which work is a feature vs. a tooling change. Written for
   Claude Code, but it applies here; the one OpenCode-specific gap is noted under Hooks below.
3. `CONTRIBUTING.md` — setup, which tool owns which file, what to do when a check fails. 26 KB; read
   the section, not the file.
4. `docs/decisions/` — 56 numbered ADRs, `0000-kebab-slug.md`, each declaring an `altitude` and a
   `commitment`. Cite the record that covers your subject before writing a new one.

Spec Kit lives in `.claude/skills/speckit-*` and `.specify/templates/`. Artifacts go in
`specs/NNN-slug/`, where `NNN` is the **GitHub issue number**, not a local counter — the numbering
skips, and `docs/specs/` is a frozen archive that is never added to.

## Setup

```sh
rustup toolchain install   # exact 1.98.1 + wasm32 target, read from rust-toolchain.toml
cargo xtask setup          # npm ci; ~30s once
```

Node 22.18.0 from `.nvmrc` backs four checks Rust cannot do. `gh` must be authenticated —
`cargo xtask spec` shells out to it to read issues, create branches and move labels.

`cargo xtask check` refuses to start when the installed Node tree does not match `package-lock.json`
by content. After editing `package.json` or the lockfile, run `setup` again or the gate will not run
at all.

## Commands

```sh
cargo xtask check          # the only definition of green
cargo xtask fix            # fmt, render, prettier, markdownlint, editorconfig
cargo xtask render         # rewrite generated pictures in tracked Markdown
cargo xtask spec use 23    # check out feature 23's active-stage branch
cargo xtask spec new 23    # open the deciding stage
cargo xtask spec stage 23 build
cargo xtask pr body        # writes target/pr-body.md, chosen by branch suffix
cargo xtask pr open
cargo run -p monospace-cli # bare `cargo run` is ambiguous: two binaries in the workspace
cargo test --workspace     # tests only, for a faster loop
```

- `cargo run -p monospace-cli` prints a two-picture demonstration;
  `cargo run -p monospace-cli -- path.json` prints exactly one picture and nothing else. The
  `render` step depends on that split, so do not add output to the file path form.
- Clippy has no fix step on purpose:
  `cargo clippy --fix --workspace --all-targets --allow-dirty --allow-staged -- -D warnings`. Read
  the diff. `pedantic` is denied and some of its lints need judgment, not a rewrite.
- `cargo insta review` needs `cargo-insta`, which `cargo xtask setup` does **not** install (`setup`
  is `npm ci` only). `cargo install cargo-insta` first.

## The gate

`cargo xtask check` runs 11 steps, all of them on every invocation, and a step passes or fails on
its exit code alone. `fmt`, `prettier`, `markdownlint`, `editorconfig`, `cspell`, `clippy`, `build`,
`wasm`, `test`, `doc`, `render`.

- A new check goes into `xtask/src/main.rs` `GATE`, never into the hook or
  `.github/workflows/ci.yml`. Both call `cargo xtask check` and nothing else, and that is what makes
  "green" mean one thing.
- `wasm` compiles only `monospace-core`, `monospace-diagram` and `monospace-glyph-sets` for
  `wasm32-unknown-unknown`. It is the machine that keeps the core free of terminal assumptions, so a
  `std::io` or `std::process` reach in the core fails the build there even if it works locally.
- The `test` step runs doctests too, so there is no separate step for them.
- `render` builds and runs `monospace-cli` itself, so it is the slowest step and belongs last.

### Hooks

`.claude/git-hooks/` holds `pre-commit` (the gate) and `commit-msg` (commitlint). **Only a Claude
Code session installs them**, via a `SessionStart` hook in `.claude/settings.json`. From OpenCode,
run this once per clone or your commits skip the gate silently and nothing says so:

```sh
git config core.hooksPath .claude/git-hooks
```

`--no-verify` is forbidden by the constitution. `rebase` and `cherry-pick` do not fire the hook
either, so the gate has to be re-run on each rewritten commit, not just the tip.

## Pictures in Markdown

Constitution principle IV: a picture in a tracked file is either **generated** or labelled
**hypothetical**. The `render` step enforces the first mechanically.

A generated picture is a four-line marker wrapping a fenced `text` block. Leave the fence empty and
run `cargo xtask render`. Full example in CONTRIBUTING under _A picture a document generates_.

- All four lines are required, closing `<!-- /render -->` included. A malformed marker is reported
  by file and line, never skipped.
- **`render` only walks `git ls-files '*.md'`.** A brand-new untracked document is invisible to it:
  `git add` the file first or `cargo xtask render` will silently do nothing for it.
- Rendered output has each line's trailing blanks trimmed, because `editorconfig-checker` runs with
  `trim_trailing_whitespace` over everything tracked. Do not hand-pad a fence.
- A hand-drawn picture cannot be detected. Labelling it `Hypothetical — hand-drawn, not generated.`
  is on you.
- The JSON inside the marker is the `monospace-cli` description format, documented in
  `specs/079-.../contracts/description-format.md`. It carries no reference to a file — inline only.

## Tests

Contract tests pin a decision; characterization tests record what the code does over a range too
wide to assert. They are kept apart **by snapshot directory**
(`insta::Settings::set_snapshot_path`), and each characterization file repeats at its head that it
is accepted on a report, not on a claim of review.

The arrow sweep is 1856 renderings across 8 snapshot files. **A snapshot that moved is a question,
not a failure**: report how many cases moved, in which families, and three examples with before and
after, then ask whether that movement is wanted. Never describe a characterization as reviewed.

## Architecture

Four crates, and the boundary is the point (principle VII):

- `monospace-core` — all domain logic: buffer, cells, geometry, glyphs, render, `shape/`.
- `monospace-diagram` — the drawn model: shapes with identity and order.
- `monospace-glyph-sets` — tables the core does not ship; depends on the core, nothing further.
- `monospace-cli` — no logic of its own. Converts the description format and draws.

`xtask` has **zero dependencies** on purpose: it guards the dependency policy, so it must not be the
first thing to bend it. Orchestrating subprocesses and propagating exit codes is `std`'s job.

Lints live in `[workspace.lints]` in the root `Cargo.toml` (`clippy::pedantic`, `missing_docs`, a
deny-list of rustdoc lints), never on a command line, so the editor shows what the gate enforces.
Workspace lints are `warn` locally and `-D warnings` in the gate; to permit one, add it under that
section with a comment saying why.

A **new dependency requires asking the maintainer first**, and before pinning a version you must
verify its publication date is at least seven days old and report both. Domain logic prefers `std`.

## Flow

A **feature** changes what the tool can do: an issue, `specs/NNN-slug/`, and two staged branches
(`-deciding`, `-building`) as two PRs against `main`. State lives only as the labels `wish` →
`deciding` → `building` on the issue.

A **tooling change** — the gate, hooks, CI, these documents, the workflow — takes an ADR and
commits. No spec directory, no stage branch, no Spec Kit command. The question that decides it: does
this change what `monospace` can draw, or how we work on it?

- `cargo xtask spec stage <n> build` checks `origin/main`, not your working tree: it requires
  `spec.md` and `decisions.md` to be merged **and** no line of the sheet to still contain
  `_pending_`. Answering an entry means removing its `_pending_` marker, then merging the deciding
  PR.
- **Never run `/speckit-plan` end to end.** Part one is Phase 0 and stops at `decisions.md` — no
  `data-model.md`, no `contracts/`, no `quickstart.md`, because writing those is taking the
  decisions. The maintainer reads the sheet in full and answers it; part two runs Phase 1.
- Speckit resolves the feature directory from `SPECIFY_FEATURE_DIRECTORY` or the **gitignored**
  `.specify/feature.json`, which is per-clone and can be stale. Check it, or just run
  `cargo xtask spec use <issue>` first. A stale value points speckit at a directory that does not
  exist.
- Work with no issue yet becomes a two-line `wish` issue, never a section of the current spec.

### Decisions

Default to **module-level**: rustdoc under `Design notes` in that module, no ADR, changed by an
ordinary `feat` or `fix`. Promote to `docs/model.md` + an ADR only when something outside the module
can observe it.

Before writing a record, find the one that covers the subject: if you cannot cite the record you are
about to write without citing another, they are one record and yours is a revision. A record with a
`working` or `exploratory` commitment is revised in place, with a dated line; only `load-bearing` is
superseded by a new ADR. `docs/decisions/` is a history — absorb a record into rustdoc by setting
its status, not by deleting the file.

## Commits

Conventional Commits, enforced by the `commit-msg` hook. **`refactor` never changes behavior** and
touches no test; `feat`/`fix` are the behavioral types. Mixing them is a violation, and large
refactors go expand/contract, each step green and separate.

Do not let a colon-terminated word start a line of the body — commitlint reads `word:` as a footer
token and splits the message there. Reword with an em dash or re-wrap.

A commit must tick exactly the `tasks.md` checkboxes it completed. End each increment — a slice that
reaches a demonstrable state — with an appended entry in `docs/learning-log.md`: what was learned
about Rust design, what about working this way, optionally a trade-off. Appending needs no read of
the file.

## Working inside a session

A session costs its length squared, because the constitution and `CLAUDE.md` are re-read on every
call. One Spec Kit phase per session, clearing context between.

- **Read the part, not the file.** `docs/learning-log.md` (1617 lines), `docs/model.md` (452) and
  `docs/glyph-sets.md` (282) are large enough that opening one whole is a decision. Take a line
  range or grep with context.
- **Delegate a lookup that spans files** so they never enter this context.
- Never ask a question whose subject this project renders. Generate both options and show them side
  by side; generating them is part of asking. Label a hand-drawn one.
- Say it in the first paragraph when the code disagrees with a record. Do not reconcile silently and
  do not change code to match a document without asking which is wrong.
- Cite another document's section by name or anchor, not by number. Nothing checks links — a link to
  a file that does not exist passes the gate.

## Traps the gate will not tell you

- **cspell**: the repository writes American English. A British spelling gets changed, not added. A
  real term no dictionary knows goes in `project-words.txt`, one per line, sorted, case as written.
  `cspell.jsonc` may already carry a dictionary for your domain — check before adding.
- **Cross-file link rot** is invisible to every step; `markdownlint` only validates a fragment
  against the headings of the same file.
- **`rustfmt` and Cargo can print a configuration complaint and still exit 0** — the gate does not
  catch it, by deliberate choice. Watch for stray output rather than trusting a green run.
- `.sdd-v2/` is a gitignored throwaway exercise. Ignore it.
