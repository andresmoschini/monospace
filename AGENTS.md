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
- **Prettier owns the width, at 100 columns, and it reflows rather than refusing.** Rewording a
  paragraph to shorten it buys nothing — the words return on the next `cargo xtask fix`. Meeting an
  artifact ceiling, 60 lines for a `working` ADR, means deleting content or splitting the decision
  into two records. Rewrapping is not a way under it.
- **`docs/decisions/README.md` is one prettier-aligned table, and a partial edit corrupts it
  silently.** Anchoring on a fragment of a row leaves the rest of that row on the line below, and
  prettier then reflows the damage rather than rejecting it. Replace a whole row, never a fragment.

## For this harness specifically

- **Two plugins live in `.opencode/plugins/`, and nothing in the gate can execute them.** They are
  the only thing that installs the git hooks here and the only thing that puts a session id in a
  commit, so a failure in either is silent, and check that they loaded before assuming either works.
  Each logs a line when it arms, and it reaches the TUI and not
  `~/.local/share/opencode/log/opencode.log` — measured, that file has never held a `[monospace]`
  line whether the plugin loaded or died, so do not read its absence there as evidence. The log
  file's own `loading plugin` and `failed to load plugin` entries are the readable signal, and the
  second carries the cause. A reload unloads a plugin without warning: anything that checks out
  these files, a rebase among them, drops both until they load again, and a session that loses
  `session-trailer.js` loses the variable with it. A missing line is usually not the API moving: the
  loader dies at module resolution first, and a local plugin's bare imports resolve from this
  project's `node_modules`, so whatever a plugin imports has to be a dependency here. What it must
  export is a default object carrying an `id` and a `setup`.
  [ADR-0058](docs/decisions/0058-install-the-git-hooks-from-an-opencode-session-too.md)'s revisions
  carry the history.
- **`OpenCode-Session` and `Claude-Resume` are separate trailer keys** on purpose: neither client's
  id resumes the other. `commit-msg` reads `MONOSPACE_SESSION_ID` and `CLAUDE_CODE_SESSION_ID`. A
  commit made outside an agent shell correctly carries neither.
- **Speckit's own files are written with CRLF on Windows.** `.gitattributes` normalizes to LF and
  `core.safecrlf` is on, so `git add` refuses them: `fatal: CRLF would be replaced by LF`. The cause
  is the CLI, not your edit, and `cargo xtask fix` is what removes it — its `eol` step, which is the
  only one that reaches the tree `editorconfig-checker` skips
  ([ADR-0059](docs/decisions/0059-normalize-what-the-speckit-cli-writes-or-git-refuses-it.md)).
