# AGENTS.md

OpenCode reads this file. It does not read `CLAUDE.md`, and it does not expand the import on that
file's first line. Nothing here restates a rule: the constitution owns the rules, `CONTRIBUTING.md`
owns the tooling, `CLAUDE.md` owns session guidance. What is left is where each of those lives, and
the handful of facts that are in the code and in no document at all.

The reasoning is [ADR-0056](docs/decisions/0056-give-opencode-its-own-instruction-file.md).

## Read these, in this order

1. `.specify/memory/constitution.md` — every rule, the scope, the testing contract and the process.
   Binding on everything below, and long enough that opening it whole is a decision.
2. `CLAUDE.md` — session habits, and which work is a feature and which is a tooling change. Its
   first line, `@.specify/memory/constitution.md`, is an **import** that Claude Code expands at
   launch, not a mention. Read it as a reference; in a Claude Code session the constitution is
   already in context and re-reading it is a cost, not a check.
3. `CONTRIBUTING.md` — setup, the everyday commands, the two stages, which tool owns which file, and
   what to do when a check fails. Read the section, not the file.
4. `docs/decisions/` — numbered ADRs, `0000-kebab-slug.md`, each declaring a `scope` and a
   `commitment`. Before writing a record, look for the one that already covers your subject.

Speckit's own artifacts are generated, not written by hand, and both harnesses have them:
`.opencode/commands/` for the slash commands here, `.claude/skills/speckit-*/` for Claude Code. They
are the same ten prompts, versioned together, and the gate does not own them.

## Facts that are in the code and in no document

- **The gate refuses to start when the installed Node tree does not match `package-lock.json` by
  content**, not by timestamp. Editing `package.json` or the lockfile means running
  `cargo xtask setup` again, or nothing is checked at all. `xtask/src/main.rs`,
  `node_tooling_state`.
- **`cargo xtask render` only walks `git ls-files '*.md'`.** A new document that has not been
  `git add`ed is invisible to it, and its picture silently never fills. `xtask/src/render.rs`,
  `tracked_markdown`.
- **`cargo insta review` needs `cargo-insta`, which `cargo xtask setup` does not install** — `setup`
  is `npm ci` and nothing else. Install it by hand.
- **`.specify/feature.json` is gitignored and per-clone.** Speckit resolves its feature directory
  from it, so a stale value sends a session at a directory that does not exist.
  `cargo xtask spec use <issue>` rewrites it.
- **`xtask` has no dependencies, deliberately.** It guards the dependency policy, so it must not be
  the first thing to bend it; orchestrating subprocesses and propagating exit codes is `std`'s job.
- **A gate step passes or fails on its exit code alone.** `rustfmt` reports that `group_imports`
  needs nightly and exits 0, and Cargo reports a missing `workspace.resolver` and exits 0. The gate
  notices neither, by deliberate choice, so watch the output rather than trusting a green run.
- **The `wasm` step compiles three crates, not one**: `monospace-core`, `monospace-diagram` and
  `monospace-glyph-sets`. It is what keeps the core free of terminal assumptions, so a `std::io` or
  `std::process` reach in the core fails there even where it works locally. The table in
  `CONTRIBUTING.md` names only `monospace-core`; the code says otherwise.
- **`monospace-cli` has two modes, and `render` depends on the split.** Bare, it prints a
  two-picture demonstration; given a path, it prints one picture and nothing else, because a
  rendering that goes into a Markdown fence cannot arrive wrapped in prose. Do not add output to the
  path form.
- **Contract and characterization snapshots are separated by directory, not by naming** —
  `insta::Settings::set_snapshot_path`. The arrow sweep is 1856 renderings across 8 files.

## For this harness specifically

- **`.opencode/plugins/install-git-hooks.js` installs the git hooks when a session starts**, the
  same way `.claude/settings.json` does for Claude Code. Check it with `git config core.hooksPath`;
  a commit made with the hooks absent runs no gate and says nothing.
- **Speckit's own files are written with CRLF on Windows.** `git add` refuses them, because
  `.gitattributes` normalizes to LF and `core.safecrlf` is on. Normalize after any
  `specify integration install` or `specify update`:
  `sed -i 's/\r$//' .specify/*.json .opencode/commands/*.md`
