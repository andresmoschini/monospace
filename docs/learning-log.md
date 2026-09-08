# Learning log

What was learned while building this, kept apart from what was decided. Decisions live in
[`decisions/`](decisions/README.md) as records that are written once and superseded rather than
edited; this file is appended to, newest entry last, one entry per increment.

An increment is a slice of work that reaches a demonstrable state. It is usually several commits,
sometimes many.

The categories come from
[Demonstrable increments](../.specify/memory/constitution.md#ii-demonstrable-increments) in the
constitution: something about Rust design and idiom, something about working this way with Claude,
and the trade-offs worth remembering. A lesson is only worth an entry if it came with evidence —
what was tried, and what it turned out to be.

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

## 2026-09-07 — The model, its records, and the first spec

A buffer-and-render model written down, nine decision records, and a specification for the first
slice of code. Still no domain logic: `monospace-cli` prints the same line it did before.

### Rust design and idiom

- **A weak test grew the public API, and a better test shrank it again.** The spec asked for a test
  asserting that the built-in glyph rules numbered fifteen, which meant `GlyphCatalog` needed a
  `len` — and then `is_empty` too, because clippy's `pedantic` set refuses one without the other.
  Counting catches a rule that went missing but not one that is wrong, so it was replaced by the
  property that every key a light cell can produce is answered. That test needs only the lookup, and
  both accessors left the public surface with it. The API had grown to serve an assertion rather
  than a caller, which is worth noticing early: the same pressure produces accessors nobody calls in
  every codebase.

### Working this way

- **A rule that demands an error has to say which error.** A behavior rule said a zero width or
  height was "rejected at construction" and stopped there. A panic and a `Result` are different
  promises and the difference reaches every call site, so the rule was only half written — and
  nobody had noticed, because it read like a decision. Allowing zero removed the rule, an
  unspecified error path, and a paragraph justifying it: a buffer with no positions already behaves
  correctly under the rules covering stamps and renders outside the window. The general form is that
  a rule requiring something to fail owes an answer about how, and if that answer is hard to give,
  the requirement is usually the thing to drop.

## 2026-09-07 — Spec 0001 implemented: stamp cells and render them

Eleven commits: `Pos`/`Size`, `Stroke`, `Arm`/`Cell`, `Buffer` with `stamp`, `GlyphKey` and
`GlyphCatalog::light`, `render`, and the CLI switched from a placeholder greeting to the spec's own
4×3 box. `cargo run -p monospace-cli` now draws something real.

### Rust design and idiom

- **A field split across two methods forces those methods into one commit.** The plan called for
  `Buffer::new` and `cell` before `stamp`, on the theory that a fresh buffer answering `None`
  everywhere was demonstrable on its own. It compiled and passed its test, and `clippy` still
  failed: `origin` and `size` go unread until `stamp`'s window check exists, so `dead_code` fired on
  fields nothing else touched yet. This is the same shape as the `enum Runner` case from the
  previous increment, in a place the plan did not predict — splitting by _type_ (buffer vs. stamp)
  does not always line up with splitting by _field usage_, and the gate is what catches the mismatch
  before a reader does.
- **Checked conversions sidestep a whole family of pedantic lints, and cost one documented edge
  case.** `contains`'s window check uses `checked_sub` and `u32::try_from(..).ok()`; `render`'s loop
  bounds use `i32::try_from(..).expect(..)`. Neither needed a single `#[allow]` for
  `cast_sign_loss`, `cast_possible_wrap` or `cast_possible_truncation` — the usual `as` casts would
  have needed at least one. The cost is honest rather than hidden: `render` now carries a `# Panics`
  section for a width or height past `i32::MAX`, which `clippy::missing_panics_doc` demanded the
  moment the `expect` existed, and no diagram this crate can address will ever reach it.
- **A test helper is held to the same scrutiny as production code.** A `light() -> Option<Stroke>`
  helper that only ever returned `Some(..)` failed `clippy::unnecessary_wraps` before a single test
  ran against it. Inlining `Some(Stroke::from("light"))` at each call site was smaller than the
  helper it replaced.

### Working this way

- **An example-heavy spec held up against implementation with zero surprises.** Every worked example
  in spec 0001 was checked by hand against `docs/glyph-sets.md` before any code existed, and every
  one matched once the code was written — no rule turned out to admit a second reading, and no
  example needed correcting. `docs/specs/README.md` says the examples are "the section that does the
  real work"; this increment is the first evidence that claim pays off rather than just reads well.
- **The one real gap was worth asking about, and cheap to resolve.** The spec's acceptance list
  asked `monospace-cli` to print "a box" without saying which one. Guessing would have picked
  dimensions nobody agreed to; asking took one question with two options and was settled before any
  code was written. Reusing the spec's own already-verified 4×3 example as the answer meant the
  CLI's integration test asserts data that had already been checked twice, rather than inventing a
  third copy of it.

### Trade-offs worth remembering

- **`Buffer` stores cells in a `HashMap<(i32, i32), Cell>`, unmeasured at any real size.** Fine for
  a four-by-three demo. `docs/model.md` names being able to answer cheaply whether a cell is already
  decided as what makes front-to-back stamping with early stopping worth using; a hash lookup per
  cell does not yet give that for free. Worth revisiting once the `Below` mode spec asks for it, not
  before.

## 2026-09-07 — Hardening what spec 0001 already shipped

Two commits, both found by questioning already-`implemented`, gate-green code rather than by a new
spec: `render` no longer risks an overflow, and `Buffer::stamp` merges a cell by building a new
value instead of mutating one in place.

### Rust design and idiom

- **Mutating a value field by field and replacing it outright cost the same, here.** `stamp`'s
  already-defined branch used to write `target.base` and call a `&mut Arm`-mutating `merge_arm` four
  times. A pure `merge(&Cell, Cell) -> Cell` plus one `*target = merge(target, cell)` needed no
  extra allocation and no extra `HashMap` lookup to replace it — the more idiomatic-looking `Entry`
  API (`and_modify`/`or_insert`) would have needed cloning the base stroke to satisfy the borrow
  checker, since the same `cell` value cannot be borrowed by one closure and moved into the other.
  All 15 existing tests passed unchanged, which is what let the commit land as `refactor` rather
  than `fix`.

### Working this way

- **The gate passing had proven the paths the tests exercised, not the arithmetic itself.** A
  question about `Buffer::stamp` led to one about `render`'s `origin.x + dx`, which turned out to
  overflow `i32` for an origin close to `i32::MAX` — reproduced first with a regression test against
  the unfixed code (`attempt to add with overflow`, under the same `dev` profile `cargo xtask check`
  already runs), only then fixed with `i32::checked_add_unsigned`. Three narrower alternatives were
  weighed first — shrinking `Size` to `u16`, widening the loop to `i64` — and rejected because each
  either left a smaller version of the same overflow in place or introduced a type nothing else in
  the function needed. The fix also removed both `i32::try_from(..).expect(..)` calls and the
  `# Panics` section two entries above praised: that section documented a real panic, correctly, and
  is gone because the panic it documented is gone too.

## 2026-09-07 — A border that could not be crossed, and a spec that was two

No code. One amendment to the model, two specs where there was going to be one, and a decision
record. The increment reached a demonstrable state on paper: a picture nobody had drawn yet turned
out to be wrong, and it was wrong for a reason that no test in the repository could have reported.

### Working this way

- **The example caught the error, and a reader caught it in the example.** Drafting the second spec
  meant specifying a demo of two overlapping boxes, with the expected output derived by hand. The
  derivation said the two borders crossed as `─` and `│` — passing through each other, neither
  joining. Everything about that was internally consistent: the rules were applied correctly, the
  glyph lookups were right, and the quality gate was green because nothing had been written yet. It
  was caught by looking at the picture and saying the lines should meet. Correcting it changed four
  characters in a document, no code, and one paragraph of the model.

- **Choosing `Closed` where `Unset` belonged was invisible by construction.** The box the front end
  drew closed every side it did not use, which says nothing may ever connect here. At render time a
  key carries a stroke or nothing, and `Closed` and `Unset` both give nothing, so a box spelled
  either way draws the same three lines. No test, no lint and no render could report the difference:
  the first thing that could was a second figure touching the border, which is exactly the demo that
  had not been built yet. For state that is invisible at render time, the only instrument is another
  figure.

- **Needing to amend the model was the signal that the spec held two ideas.** The spec that had to
  change `docs/model.md` in order to be writable was carrying both a convention for how a figure
  declares its sides and a second stamp mode. Splitting them cost a rewrite of one document and
  nothing else, because no code existed to move — which is most of the argument for writing specs at
  all. It also made the second one a comparison against a picture already on screen: the same two
  boxes, one argument different, two characters different.

- **"Does this need an ADR?" was asked three times and answered three ways.** The stamp mode as a
  parameter did not need one, because ADR-0008 had already named the operation
  `stamp(x, y, cell, mode)` and the spec was following a decision rather than taking it. A
  positional query on the buffer looked like it needed one, and then dissolved: dropping the method
  removed the decision. The cell-level predicate and the skip did need one, and became ADR-0017.
  What separated them was not size but whether a later spec would have to quote the reasoning — a
  spec becomes history once implemented, and reasoning that outlives it has to live somewhere else.

### Trade-offs worth remembering

- **A requirement was accepted that nothing can verify.** Under `Below`, `stamp` skips a target
  whose four arms are already decided. The resulting buffer is byte for byte what the merge would
  have produced, so no test can fail for it and the branch can be deleted with the gate still green.
  It is recorded in three places — ADR-0017, a paragraph outside spec 0003's numbered rules, and a
  comment the acceptance list requires at the branch — and none of those three is a check. The only
  real one is a benchmark, which needs a workload that does not exist. Worth watching: one such
  branch is a considered cost, and a second arriving for the same reason would mean the practice is
  accumulating rather than paying.

## 2026-09-07 — Spec 0002 implemented: a box that can be crossed

Three commits, none of them touching `monospace-core`: the front end's box now abstains outward
instead of closing, and draws a second, overlapping copy of itself.

### Working this way

- **A spec that already pins the CLI's exact output leaves nothing to guess.** Spec 0001 left the
  demo box unspecified and that turned into the one open question of that increment. Spec 0002 gives
  the exact string in "The whole output," positions and buffer sizes included, and implementing it
  took zero clarifying questions and zero surprises — the rendered text matched the spec's own
  worked example on the first run.
- **The claim that `Closed` and `Unset` render alike was checked by the existing test, not
  assumed.** Switching the box's outward sides from `Closed` to `Unset` is exactly the change the
  previous increment's entry says was invisible at render time. The single-box integration test's
  assertion was left untouched on purpose, and it still passed — the same test that could not have
  caught the original mistake is what confirms this fix didn't introduce a new one.

## 2026-09-07 — Spec 0003 implemented: stamp below what is already there

Six commits: `StampMode` threaded through every call site with `Above` unchanged, then `Below` and
`Cell::is_decided` with their real behavior, then the front end drawing the pair a second time,
labelled.

### Rust design and idiom

- **`Above` and `Below` are mirror images of the same three-line function.** `merge_arm_above`
  checks the incoming arm; `merge_arm_below` checks the target's. Writing them side by side made the
  symmetry ADR-0008 describes in prose — one mode overwrites what the stamp abstains on, the other
  fills what the target left undecided — visible in the code, rather than one function with a mode
  check buried inside it.
- **A mechanical 31-call-site edit was worth scripting, not typing.** Adding a required parameter to
  `stamp` meant touching every existing call site — 15 tests and the CLI — to keep compiling. A
  small bracket-counting script did it in one pass. Its first version produced a double comma
  wherever a call already ended in a trailing one, caught by reading the diff before running
  `cargo fmt` and the gate, not by trusting the script.

### Working this way

- **A third spec in a row held up against implementation with no surprises.** Every value in spec
  0003's worked examples — the two-mode comparison, the decided-cell test, the three-figure
  equivalence — matched what the code produced on the first run, the same as spec 0001 and spec 0002
  before it. The pattern is no longer a one-off: an example-heavy spec keeps paying for itself.
- **The branch the drafting increment anticipated now exists, exactly as unverifiable as
  predicted.** That increment's entry flagged a requirement accepted with nothing to verify it:
  `stamp` skipping a decided target under `Below`. It is now real code, covered only by a comment,
  an acceptance item read rather than run, and ADR-0017 — nothing about implementing it found a way
  to test it. The prediction held.

## 2026-09-07 — Unifying merge, and the branch ADR-0017 saw coming

Three commits: `merge_arm_above` and `merge_arm_below` collapsed into one function, a new ADR, and
the branch that ADR predicted.

### Rust design and idiom

- **Two functions were one function with the arguments swapped, once written the other way around.**
  `merge_arm_above(target, incoming)` and `merge_arm_below(target, incoming)` read as different
  rules until `merge` was rewritten to take `(top, bottom)` instead of `(target, incoming, mode)`.
  With that shape, both collapsed into a single `merge_arm(top, bottom)`, and the `match mode` moved
  to the one call site that already had to choose an order — removing a whole duplicated `Cell`
  literal, not just a duplicated three-line function.

### Working this way

- **All four branches were measured, not just the new one.** Before adding a fourth match arm to
  `stamp`, each of the four was instrumented with a distinct panic and run against the full test
  suite plus the CLI binary. Three were already reached by existing tests; the fourth — the one
  about to be added — was reached by nothing at all, not even indirectly. The new test and its claim
  of coverage came only after that measurement.
- **An accepted ADR's own "what would change this" clause got exercised for real.** ADR-0017 named a
  second unverifiable branch, arriving for the same reason, as the specific thing that would matter.
  When unifying `merge` surfaced exactly that branch's mirror image, that sentence — not a fresh
  argument — was what decided the question needed a new record (ADR-0018) rather than a silent
  addition.

## 2026-09-08 — Migrating to Spec Kit, and dissolving the brief

Nine documentation commits and no code. spec-kit installed, a constitution ratified and amended
twice, the spec home moved to `specs/` with `docs/specs/` closed, and `docs/brief.md` gone — its
rules to the constitution, its provenance and domain questions to the model, its phases to a new
roadmap. No Rust was written, so this entry has nothing to say about Rust.

### Working this way

- **Citations by name survived the move; citations by number did not.** The specs cited the model by
  section name and the ADRs cited the brief by number, and only the second group broke: five records
  pointed at `§3`, `§5`, `§8`, `§9` of a file that stopped existing, while every citation of
  `docs/model.md` still resolved. Measured before converting anything: 139 relative links between
  Markdown files, none of them carrying an anchor. There are 17 anchors now, all verified against
  their target headings, and one of them was wrong on the first attempt because the heading reads
  "III. One definition of green (NON-NEGOTIABLE)" and the slug keeps the parenthetical. The rule the
  repository already had — cite by name, not by number — turns out to be worth an enforcing check
  rather than a convention, which is
  [issue #13](https://github.com/andresmoschini/monospace/issues/13).
- **A rule written down did not prevent the slip it described.** commitlint reads a body line
  starting with `word:` as a footer token and warns. It happened, it was documented in
  CONTRIBUTING.md as a thing to avoid, and then it happened again two commits later — in the commit
  that closed the specs directory. Two amends, same cause. What would have prevented it is a check,
  and the warning that already exists cannot become one: as an error it would reject legitimate
  prose. So this one stays a habit, and the honest conclusion is that writing a rule down is weaker
  than it feels while writing it.
- **A mechanism beat a reminder, and the difference was measurable.** CLAUDE.md held the project's
  rules in its own words because it is the only file loaded into every session, and the constitution
  then restated all of it. Replacing that with `@.specify/memory/constitution.md` removed the
  duplication rather than managing it: `/context` in a fresh session lists the constitution at 4.6k
  tokens. A line saying "read the constitution first" depends on remembering to; an import does not.
  It buys no tokens — the documentation is explicit that imported files load in full — only one
  source of truth.

### Trade-offs worth remembering

- **Estimating what a tool costs was worse than reading the tool.** ADR-0021 recorded the cost of
  adopting Spec Kit's spec template as one thing: no counterpart to the Examples section. Reading
  the template and the tasks skill later showed the estimate was wrong in both directions. Worse
  than recorded: three sections have no home at all — scope with a destination per item, open
  questions parked with what would settle them, and why this slice before the others — and
  `/speckit-tasks` is structurally organized by user story, so the override that record named as the
  escape hatch would have needed a second override to stay usable. Better than recorded: the "no
  implementation details, written for non-technical stakeholders" constraint that looked
  disqualifying for a library's public API is not in the template at all. It lives in a checklist
  the skill generates for itself, in the feature's own directory, which is ours to write. An ADR's
  Consequences section is the part most likely to be an estimate dressed as an observation.
- **The two drafts were abandoned rather than translated, which reverses a decision made two days
  earlier in this same migration.** Rewriting them by hand would have produced the artifact and
  hidden the question; running the flow answers it. That is the second time in this increment that
  the cheaper move was to stop and get evidence, and both times the thing that made stopping
  possible was that nothing had been pushed yet.
