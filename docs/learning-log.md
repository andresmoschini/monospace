# Learning log

What was learned while building this, kept apart from what was decided. Decisions live in
[`decisions/`](decisions/README.md) as records that are written once and superseded rather than
edited; this file is appended to, newest entry last, one entry per increment.

An increment is a slice of work that reaches a demonstrable state. It is usually several commits,
sometimes many.

The categories come from section 6 of [the brief](brief.md): something about Rust design and idiom,
something about working this way with Claude, and the trade-offs worth remembering. A lesson is only
worth an entry if it came with evidence — what was tried, and what it turned out to be.

---

## 2026-09-04 — Quality gate and project skeleton

Forty-four commits with no domain logic in them: a virtual workspace, a ten-step gate behind one
entry point, hooks, CI, and seven decision records. `monospace-cli` prints one line that it asks
`monospace-core` for.

### Rust design and idiom

- **An architectural constraint can be delegated to the compiler.** The brief asks that the core
  avoid terminal and CLI assumptions so it can be compiled to WebAssembly later. That was a sentence
  until the gate began running `cargo check -p monospace-core --target wasm32-unknown-unknown`. The
  difference is measurable: a function that is public, documented, `#[must_use]`, with its import
  used and clippy at `-D warnings` reporting nothing, is still rejected if it calls a Windows-only
  API. No other check has any reason to object to that code. A boundary that only exists in prose
  erodes; one the compiler refuses to cross does not.

- **Lints belong in the manifest, not on a command line.** `[workspace.lints.clippy]` with
  `pedantic = { level = "warn", priority = -1 }` is inherited by every member through
  `[lints] workspace = true`, so rust-analyzer shows exactly what the gate enforces while code is
  being written. The `priority = -1` is not decoration: Cargo rejects a lint group alongside
  individual overrides without it, which is the first thing anyone will want to do. Keeping the
  level at `warn` and passing `-D warnings` only in the gate means the editor does not paint
  half-written code red while the gate still forgives nothing.

- **Test the wiring, not a copy of the value.** The CLI's integration test runs the built binary and
  compares its output against `monospace_core::greeting()` rather than against the same string
  written a second time. It therefore fails when the two crates come apart, not when someone edits a
  literal in one place. That it fails at all was checked by breaking the call site on purpose.

- **Cargo's own diagnostics are outside `-D warnings`.** A virtual manifest with edition 2024
  members needs `resolver = "3"` declared, and Cargo warns loudly when it is missing — but that
  warning comes from Cargo, not from `rustc`, so `RUSTFLAGS=-D warnings` never turns it into a
  failure. A gate built on exit codes has blind spots wherever a tool reports a problem and exits
  zero. `rustfmt` has the same shape: it says
  `can't set group_imports, unstable features are only available in nightly channel` and exits 0.

### Working this way

- **Three claims went into records before being tested, and testing changed all three.** That a
  virtual workspace forces `-p` on every command — false, only on `cargo run`, and only once there
  are two binaries. That a missing `resolver` fails silently — false, Cargo is loud about it. That
  nightly `rustfmt` options simply do not apply — true, but worse than stated, since they warn and
  exit 0. Two of the three were committed before being caught. The rule that came out of it is now
  in `CLAUDE.md`: do not write behavior into a record without having run it.

- **A green run proves the command ran.** Every step of the gate was verified by making it fail on
  purpose and then restoring — a broken assertion, a deleted `#[must_use]`, a removed doc comment, a
  bare URL, a misspelling, trailing whitespace in `LICENSE`. Twice this exposed a verification that
  was itself wrong: cspell keeps searching upward for `cspell.json` even when given `--config`, so
  two experiments were quietly using the real configuration and reporting that pieces of it were
  unnecessary.

- **Test the repository, not only the working copy.** Cloning into a fresh directory and running the
  CI sequence there found that four tracked files were checked out with CRLF, including both shell
  hooks — which do not run on Linux at all. It was invisible locally because those files had been
  written rather than checked out, so Git had never applied the conversion. CI would not have found
  it either: on Linux the platform default is LF, so the checkout is correct there and only Windows
  clones break. A green gate and a correct repository are different claims.

- **Writing the record first changed the design, not just the documentation.** Splitting a
  preparatory refactor from the behavioral change requires the refactor to be green on its own. That
  ruled out the obvious `enum Runner { Path, Node }`, because the `Node` variant would be
  constructed by nobody until the next commit and `dead_code` would fail the gate. The alternative —
  a bare program name means `PATH`, a name with a separator means a path from the workspace root —
  is the rule a shell already uses, needs no new concept, and makes the step table say where the
  tool actually is. The constraint produced the better design.

- **A record superseded two commits after it was accepted is the mechanism working.** ADR-0006
  claimed the `Claude-Session` trailer; Claude Code then began writing that same key itself.
  ADR-0007 supersedes it rather than editing it, so the reversal and its cause stay visible.
  Separately, a factual error inside an accepted record was corrected in its own commit, labeled as
  a correction rather than folded into unrelated work — the conclusion did not change, and
  pretending the mistake was never there would have been the worse option.

- **The brief bent on purpose.** Its first principle required a bare `cargo run` to produce output,
  which a second binary in the workspace makes impossible. The choice was to contort the layout with
  `default-members` to preserve the sentence, or to amend the sentence. Amending it, in the same
  increment and with the reason recorded, is not drift; not noticing would have been.

### Trade-offs worth remembering

- **Six approved dependencies arrived as 282.** Every direct dependency was checked for age and
  approved individually. The lockfile holds 282 entries and `node_modules` is 44 MB across 9,425
  files. The policy governs what is asked for, not what comes with it.

- **A permissive entry in a checker is a silent hole.** cspell started with six dictionaries; four
  were already active by default and contributed nothing. The `npm` dictionary did contribute — for
  exactly one word, while bringing thousands of package names that could mask a real misspelling. It
  was dropped and the word written down instead. Every remaining dictionary and every word was
  verified load-bearing by removing it and re-running. Configuration that changes nothing is not
  neutral; in a checker it quietly permits things.

- **The obvious optimization was a quarter of the available one.** Asked whether dropping the
  British dictionary would be faster, the answer was yes, by about 180 ms — measured by interleaving
  the two configurations, because sequential batches drifted by as much as the difference. Looking
  for it turned up `--cache`, worth about 660 ms. The question was answered honestly and the better
  answer was somewhere else.

- **A tool can be worth keeping while costing more than it returns.** The approved version of
  `editorconfig-checker` crashes at startup; the conservative fallback is broken on Windows; the one
  that works needs its main check disabled, because indent-size is incompatible with the alignment
  both rustfmt and prettier produce. What survives is narrow — line endings, final newlines,
  trailing whitespace — but it is the only check that reads `LICENSE`, the TOML files and the
  dotfiles, which prettier cannot even infer a parser for. Narrow and unique beat broad and
  duplicated.

- **One gap was accepted knowingly, and written down as such.** The hooks install themselves when a
  Claude Code session opens, so a commit made from a plain terminal in a fresh clone runs no checks
  and says nothing about it. CI is the boundary that holds. ADR-0005 records this at ~60%
  confidence, lower than anything else here, because every other decision in the increment was made
  to stop a check from passing without running.
