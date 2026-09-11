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

## 2026-09-08 — Specifying the glyph type, and running the flow for the first time

Ten documentation commits and no code: the first feature to go through `/speckit-specify`,
`/speckit-clarify`, `/speckit-plan`, `/speckit-tasks` and `/speckit-analyze` end to end, authored
from the abandoned spec 0004. No Rust was written, and yet the type system settled a design question
before any of it existed.

### Rust design and idiom

- **`-> &str` is a storage decision wearing an accessor's clothes.** Spec 0004 prescribed a first
  commit holding a `char` inside the type, alongside a public `as_str(&self) -> &str`. Those cannot
  coexist: `char::encode_utf8` writes into a buffer the caller owns, so there is no `&str` with the
  lifetime of `&self` to return. The payload is a `String` from the first increment instead, which
  made the second increment's diff smaller rather than larger — the widening became a change of
  predicate and nothing else. What was tried was reading the agreed signature and the agreed
  representation together, which is cheaper than discovering it in a compiler error a week later.
- **The crate boundary in `unicode-rs` is by purpose, not by "Unicode things".**
  `unicode-segmentation` answers UAX #29 — where the boundaries are — and carries no
  General_Category data at all. So the cluster half of the invariant costs a dependency and the
  control half is `char::is_control()`, already in the standard library and exactly the `Cc`
  category. That is what made a two-increment split possible: the first needs no dependency
  whatsoever. It also settled a clarification, since refusing format characters as well would have
  needed a second crate and would have refused the joiner that holds a multi-code-point emoji
  together.

### Working this way

- **The generated requirements checklist validates completeness, not decomposition.** The first
  draft of the spec had three user stories and ticked all fourteen items the skill generates. One of
  the three was an acceptance criterion wearing a story's clothes, and the tell was in the artifact
  itself: its own "Why this priority" had to explain that it was not a capability. Nothing in the
  checklist asks whether a story, shipped alone, leaves anything demonstrable — so the spec passed
  its own review while carrying two stories that were one change split by technical layer. A Story
  independence block went into this feature's checklist, and it dies with the feature.
- **A prompt is not a home for a fact.** Before running `/speckit-tasks` there were five things to
  tell it. Checked one by one: three were already in `plan.md` and `spec.md`, one was task
  granularity that the command owns, and exactly one — that this feature has no foundational work —
  was written nowhere. That one went into the plan and the command ran bare. The mechanism behind
  the rule is concrete: `/speckit-analyze` cross-checks the spec, the plan and the tasks, so a fact
  that lives only in a prompt is invisible to the step whose job is to find missing facts.
- **The cross-check found what re-reading did not.** Before running it, the coverage was counted by
  hand and declared closed. `/speckit-analyze` then surfaced ten findings, and the one that mattered
  was a requirement whose work no task named: `render`'s rustdoc promises lines "exactly
  `size.width` characters wide", and the second increment is what makes that sentence false. It
  would have shipped as a public doc comment asserting something the code no longer did. Five
  findings were fixed, five were left deliberately, and none was a constitution violation — the
  value was not the volume, it was that the reader was not the author.

### Trade-offs worth remembering

- **The abandoned draft was an input, and treating it as agreed cost two corrections.** Its rules,
  examples and public surface really were reusable, which is what made the rest feel trustworthy:
  its increment shape could not be built, and its claim that the first commit was "a refactor rather
  than a structural commit" was a misclassification with an apology attached. A commit that
  introduces a type refusing `\n` adds behavior, so it is a `feat`, and a `feat` may edit the
  assertions its own signature change forces. Both were found by planning the work rather than by
  reading the document again, which is an argument for planning early rather than for reading
  harder.
- **Verifying a dependency's version at plan time would have been work with a shelf life.** It was
  in the prompt until the maintainer asked why, given that implementation may be a week away. The
  constitution attaches the seven-day rule to the act of adding or pinning, and the spec attaches
  the report to that same commit, so a number verified now is either re-verified later or stale.
  What `research.md` records instead is the part that does not perish: which crate, what it answers,
  and what the standard library already covers.

## 2026-09-08 — Spec 006 implemented: give a glyph a type of its own

Four commits: `Glyph` added and wired into `GlyphCatalog` and `render` in the same story, then the
invariant widened from one `char` to one grapheme cluster behind `unicode-segmentation`, then
FR-009's panic broken and restored on purpose. `cargo run -p monospace-cli` prints the same box it
did before any of it, checked byte for byte after every commit.

### Rust design and idiom

- **The row type change bought exactly what it was supposed to, and the diff proved it.** The Light
  table's row type carried `&'static str` for the glyph from the first commit, specifically so the
  P2 widening would touch no row. `git diff` on the widening commit confirms it: the whole change is
  the predicate inside `Glyph::new`, the dependency, four new tests, and one doc sentence — no line
  of `LIGHT`, no call site, no public signature. SC-006 asked for exactly this to be counted rather
  than assumed, and counting it found nothing to correct.
- **A borrow with an explicit lifetime replaced a copy without changing a caller.**
  `GlyphCatalog::glyph` moved from `Option<char>` to `Option<&Glyph>`, which meant `render`'s
  `glyph_at` needed a lifetime of its own —
  `fn glyph_at<'a>(.., glyphs: &'a GlyphCatalog, ..) -> &'a str` — to hand back a reference borrowed
  from the catalog rather than from the buffer or the position. `Option::map_or(" ", Glyph::as_str)`
  composed the fallback and the accessor in the same line the old `.unwrap_or(' ')` used, so the
  shape of the call site did not change even though the value flowing through it did.

### Working this way

- **The org's dependency-freshness rule needs a live query, and the wrong source almost answered
  it.** Verifying `unicode-segmentation`'s newest version was seven days old meant asking
  crates.io's API directly — `curl` without a descriptive `User-Agent` gets a bare `403`,
  undocumented in the moment it happens. A summarized fetch of docs.rs first offered "10 August
  2026" for the same version that crates.io's own `created_at` field puts at 2026-06-01 — a docs
  rebuild date standing in for a publish date, and wrong by two months in the direction that would
  have blocked a compliant version for no reason. The number that shipped is the one read from the
  registry's own JSON, not the one a secondary page summarized.
- **A task list's own "produces no commit" note and a legible, checked-off task list turned out to
  want different things, and splitting the difference took a real decision.** T001 and T005 are
  evidence-gathering steps that `tasks.md` explicitly says leave no commit, and the first pass
  through them respected that literally — both stayed unchecked. Asked to reconsider, T005 got a
  small commit carrying only its checkbox and the observed panic text, which cost nothing since
  nothing else had changed. T001 was different: its evidence had already been captured _inside_
  T002's commit, so checking it off separately would have put the mark somewhere other than where
  the evidence lives. The fix was `git rebase --onto` past that one commit, amending it to carry the
  checkbox alongside the baseline it already recorded, then running `cargo xtask check` against each
  of the three commits that had to be replayed on top of it — not just the new tip — per the
  constitution's own rule for what a history rewrite owes. The branch had no upstream, so none of it
  needed `--force-with-lease`.

### Trade-offs worth remembering

- **A fixture shared across two test modules turns one bad data row into a diagnostic, not just a
  failure.** Breaking the first Light table row to observe FR-009's panic failed eleven of the
  crate's thirty-one tests, across both `glyph` and `render` — every test that calls
  `GlyphCatalog::light()` anywhere in its call graph, directly or through `render`. The panic
  message named the one row that was actually wrong, which is what let restoring it be a one-line
  diff instead of a search.

## 2026-09-08 — Direction to a board, and a feature's number to its issue

Seven documentation commits and no code. Four ADRs — the CLI-before-TUI phasing rescued from the
roadmap, the roadmap dissolved into a GitHub Project, a feature's number taken from its issue, and
every feature given a parent issue — the constitution amended to 1.3.0, `docs/roadmap.md` removed
with its four live references redirected, and `CONTRIBUTING.md` given the section on how a feature
starts. No Rust was written, so this entry has nothing to say about Rust.

### Working this way

- **Four assumptions about the vendored scripts were checked before the record was written, and the
  one that was false was the one that mattered.** Three held: the number comes from scanning the
  local `specs/`, the auto-correction for an explicit `--number` sees only the local tree, and
  nothing downstream parses the `NNN-` prefix. The fourth — that resolution in `common.sh` falls
  back to the branch name — was wrong in both directions. `get_feature_paths` resolves
  `SPECIFY_FEATURE_DIRECTORY`, then `feature_directory` in `.specify/feature.json`, then fails; and
  `get_current_branch` returns `$SPECIFY_FEATURE` or the empty string, with zero occurrences of
  `git rev-parse`, `symbolic-ref` or `git branch` anywhere under `.specify/scripts/`. The dependency
  runs the other way: with no feature identifier set, `CURRENT_BRANCH` is filled from the feature
  directory's basename. Had the assumption been true, the branch name would have been part of a
  feature's identity rather than a convention beside it, and ADR-0024 would have had to promise
  something different. One pass of reading bought that.
- **Reading the allocator found a trap that no amount of designing would have.** `--number` is a
  preference, not an instruction: when its prefix is already used, the script neither fails nor
  takes the next free number above the one requested — it restarts from the highest existing prefix
  and increments from there, warning on stderr. A mistyped `--number 31` in a repository whose
  highest prefix is `006` yields `007`, and the directory stops naming its issue. That one fact is
  what turned the constitution's new rule from "pass the number" into "invoke `/speckit-specify`
  with the feature directory given explicitly", which is a stronger rule and was not available from
  the design alone. It was also the correction to an amendment already committed, caught by
  comparing the amendment against the row it was meant to implement rather than against itself.
- **A decision that had been right stopped being right, and what ended it was a different
  decision.** The earlier reasoning said not every feature sits under a wish — feature 006 was a
  prerequisite nobody asked for, and inventing a wish for it would have made the board lie to look
  tidy. That was correct while a feature's number was a local fact. Taking the number from the issue
  removed the footing under it in one line, because a feature with no issue has nowhere to be
  numbered from, and an allocator with exceptions is two conventions sharing a directory. ADR-0025
  kept what the earlier reasoning was protecting by turning the absence of a parent into a label on
  one. Nobody was looking for that; writing ADR-0024 is what surfaced it.
- **The gate could not see any part of this increment, and the number is exact.** Ten steps — `fmt`,
  `prettier`, `markdownlint`, `editorconfig`, `cspell`, `clippy`, `build`, `wasm`, `test`, `doc` —
  and not one of them checks a link, a directory number, or anything on a board. Four live
  references to a file that no longer exists had to be found and repaired by hand, and none of them
  would have failed a run. [Issue #13](https://github.com/andresmoschini/monospace/issues/13) is
  still the missing link check; issue #26 is now the missing duplicate-number check.
- **Postponing a task turned it into a card, which is the first thing the new arrangement was
  actually used for.** The duplicate-prefix check was deferred, and instead of a TODO in a file it
  became issue #26 carrying the finding that blocks it: `Step` in `xtask/src/main.rs` is a
  subprocess and nothing else, so a check written in Rust needs the shape of a step to change before
  it can exist. That is one use and it proves something small — a postponement with state rather
  than a line in a document nobody re-reads.

### Trade-offs worth remembering

- **A partial supersession needed a status the template does not have.** ADR-0021 was superseded in
  the one part that settled the numbering and stands in the rest, so `superseded by ADR-0024` would
  have been false about its conclusion and a bare `accepted` would have hidden the reversal. The
  form used is `accepted; superseded in part by ADR-0024`, which the template's four listed values
  do not include, and the cost is a status string no tool can reduce to one state. It is recorded as
  a consequence in ADR-0024 rather than left as a silent local variation.
- **Delegation split along "who decided it", not along "how hard it is".** Sonnet executed what an
  artifact had already settled: eight `gh` calls with the wording fixed in advance, four link
  redirects against a closed list, and a `CONTRIBUTING.md` section whose every claim traces to an
  ADR or to the constitution. The instructive part is the four conventions it was told to leave out
  — where issue-closing keywords live, milestone and branch name formats, and whether closing every
  sub-issue closes its parent — because none of them was decided anywhere at the time. They came
  back absent, which is what naming them bought, and the absence is what made the decision visible
  enough to take: all four were settled in the next exchange and went into `CONTRIBUTING.md` as
  procedure carrying its own reasoning, rather than as a record, on the grounds that where a keyword
  goes is how the tooling is used and not an architecture decision. What could not be delegated was
  choosing between the options and writing the records.
- **Two things this increment claims are still unobserved, and saying so is cheaper than finding out
  later.** `gh` 2.100.0 has no subcommand for project views, so the view grouped by `Phase` — the
  one ADR-0023 says reproduces the roadmap's phase table from the items themselves — does not exist
  yet and has to be made in the web interface. And no feature has been created under the new
  numbering: the collision that ADR-0024 prevents is a reading of the allocator, not an incident,
  and the first real test of the arrangement is the next feature.

## 2026-09-09 — Feature 028 implemented: hold a literal glyph in a cell

Seven commits: an ADR, a structural rename behind a type alias, the sum type with its composition
rules and tests, the front end's fill, two follow-up refactors found by reading the diff rather than
by any check, and two checklist commits recording a guard deleted on purpose and restored.
`cargo run -p monospace-cli` now draws a filled box and shows occlusion next to the junctions it
already showed.

### Rust design and idiom

- **A type alias turned a 33-call-site rename into a commit that touches zero tests.**
  `pub type Cell = StrokeCell;` for exactly one commit let `cell.rs`, `buffer.rs`, `render.rs` and
  `lib.rs` rename the struct while every call site — 32 of them in test modules — kept compiling
  unchanged. `git show --stat` on that commit confirmed no file under a `tests` module or `tests/`
  directory appeared in the diff, which is what let it land as `refactor` rather than needing an
  exception in Complexity Tracking.
- **A cell being decided by definition turned three of five composition rows into branches that
  already existed.** `Cell::is_decided` answering `true` for a literal meant the two `is_decided`
  shortcuts already in `Buffer::stamp` (ADR-0017, ADR-0018) caught every row with a literal on the
  decided side for free — verified in T007 by deleting each shortcut and running the whole suite,
  literal cases included, with nothing failing either time. The only new arithmetic was the fourth
  row, a stroke cell over a literal, closing whichever sides it left `Unset`.
- **A design review after the type already compiled found two things the type system did not
  force.** `render::glyph_at` matched on `Cell::Strokes`/`Cell::Literal` from outside to decide how
  to resolve a position's text; moving that decision onto `StrokeCell::key()` and
  `Cell::glyph_str()` collapsed it back to one chain of `and_then` calls with no match at all — the
  same shape `render.rs` had before the feature existed. Separately, `merge`'s arm-by-arm loop fed a
  literal's absence of arms through `bottom_arm` as a constant `Closed` value, so `merge_arm` — a
  function about two stroke cells falling through to each other — ended up processing a cell with no
  arms at all. Both were numerically correct before the fix; both were caught by a maintainer
  reading the diff and asking why a piece of code was speaking for a type it wasn't, not by any
  check, and both landed as their own `refactor` commits with the 40-plus-1 tests passing unchanged
  and no test added.
- **The renderer's lifetime unification cost one annotation and no allocation.** `glyph_at`'s buffer
  and catalog borrows share one `'a` now, so either can flow out as the return value; `render`
  itself needed no lifetime of its own, which is the fallback R4 in `research.md` named in case the
  annotation alone was not enough.

### Working this way

- **The plan's own Scale/Scope section had the right count before its own checkpoint prose did.**
  Phase 3's checkpoint claimed "`crates/monospace-cli/` is absent from the diff", but `Cell` leaving
  struct shape meant the CLI's one `Cell { .. }` construction site could not compile unchanged
  either — and `plan.md`'s Scale/Scope already said so: "33 construction sites across four files —
  32 of them in test modules, one in the front end." The one-line fix in `main.rs` was decided from
  that existing scope and from the fact that the workspace would not build otherwise, not from
  renegotiating with the maintainer mid-task.
- **A committed feature improving further is not the same event as more of the feature.** Two design
  concerns arrived after the feature's own commit had already landed and the gate was green. Both
  became separate `refactor` commits — never squashed into the `feat` that was already pushed — each
  running the full gate on its own and changing no test, which is what let "the feature is already
  committed" and "the feature got better" both stay true without touching history.
- **A fourth spec in a row held up against implementation with zero surprises.** The CLI's predicted
  output in `quickstart.md`, explicitly marked as derived and never observed, matched
  `cargo run -p monospace-cli`'s real output byte for byte, trailing spaces included, confirmed with
  a `diff` rather than by eye before it went into the test.

### Trade-offs worth remembering

- **An optimization confirmed once does not need re-confirming per new kind of input, but it does
  need re-confirming per new case.** ADR-0017 and ADR-0018 recorded their shortcuts as unverifiable
  by any test, with confidence tied to no second such branch ever arriving for the same reason. A
  literal is a second kind of decided cell, not a second unverifiable branch — the existing
  shortcuts extend to it because they never asked what kind of cell they were skipping, only whether
  it was decided. What did need a fresh check was the specific claim "deleting this changes no
  buffer", which T007 ran again as a measurement, not as an inference from the ADRs' prior
  confidence.

## 2026-09-09 — What the process costs, measured

Two commits and no code: an ADR recording what a session costs and why the obvious economies are the
wrong ones, and the habits that follow from it in `CLAUDE.md`. Issue #29 asked the question, and the
answer contradicted the hypothesis it started from.

### Working this way

- **The first hypothesis was wrong by two orders of magnitude, and only a measurement could say
  so.** The suspicion was that the constitution imported into `CLAUDE.md` was the cost, because it
  is visible, it is ours, and it is in every session. It is about 5k tokens against an average
  context of 264k over roughly 4,250 calls — 1.5% of a call, 1.2% of the total. Removing it would
  have felt like progress, would have cost the rule that makes the other rules apply, and would have
  changed nothing.
- **Cost is quadratic in session length, which is invisible from inside a session.** Every call
  re-reads the whole context, so the total is roughly `base x n + g x n^2 / 2`. Fitted to the
  longest session measured — 658 calls, starting at 55k of context and ending at 755k — that
  reproduces its measured total within 10%. The consequence is counter-intuitive: the same work
  split across four sessions costs about a third, because the `base x n` term is identical either
  way and only the squared term gets divided.
- **The objection to clearing context is measurable, and it is noise.** A cold start costs about 53k
  tokens of cache creation. Three extra starts against a saving three orders of magnitude larger is
  not a trade-off, it is a rounding error. But that is exactly the objection that stops anyone from
  clearing, and nobody could have dismissed it honestly without the number.
- **A restructuring proposed for a good reason died to its own evidence.** Splitting
  `docs/learning-log.md` per increment looked obviously right: it is 49KB, and it was read eleven
  times in a single session. It was read in full every time, though, and reading only its last entry
  costs a few hundred tokens instead of twelve thousand. The split would also have required amending
  the constitution, which names the file by path. A reading habit bought the same saving for
  nothing, and writing this entry used it — `sed -n '536,$p'` for the shape of the previous two
  entries, rather than opening the file.
- **Asked whether the governing documents could be shorter, the useful answer turned out to be a
  different one.** The hypothesis was duplication between the constitution and `CONTRIBUTING.md`,
  and it was wrong: `CONTRIBUTING.md` links rather than repeats in seven of its nine references, and
  only two restatements existed. The real defect was addressability. Seventeen rules had names, in
  bold, inside three headings, and thirty-two links across the repository resolved to those coarse
  anchors — five of them into one section holding eight unrelated rules — while the constitution's
  own Cross-references rule asks for citation by name. Promoting the names to headings took the
  citable anchors from 12 to 29 and the body from 2,240 words to 2,238.

### Trade-offs worth remembering

- **The artifact volume was examined and kept.** A recent feature produced about 70KB of
  specification artifacts against a workspace of some 2,260 lines of Rust. That ratio is principle I
  working as intended rather than a defect, and the decision to keep paying it is now on the record.
  The value of writing it down is not the decision, which changed nothing, but that the next person
  to notice the ratio finds a decision there instead of an oversight.
- **The cheapest lever is the one nothing can enforce.** The gate sees commits, not sessions, so
  every rule this increment produced is a habit. ADR-0027 says so in its Confirmation section rather
  than letting a reader assume a check exists. The honest substitute for enforcement is a
  measurement that can be repeated: the transcripts carry per-call token counts, so a later
  increment can recompute the average context per call and find out whether the habit held.
- **Two words is a result, not a disappointment.** The amendment that made the constitution precise
  changed its length by -2 words, which settles that precision and token cost are separate problems
  rather than one problem with one fix. Had the cleanup been sold as a saving, the number would have
  been an embarrassment; measured first and framed as addressability, it is the confirmation. The
  order matters — measure, then decide what the change is for.

## 2026-09-10 — Reviewing feature 039's research, before agreeing the plan

Three commits and no code: three ADRs, the model amended for them, and the spec amended after them.
The plan for feature 039 had been written and was waiting to be agreed; reading its Phase 0 research
critically instead of accepting it found one design defect, one reversed conclusion, one abstraction
nobody used, and two miscounts in a spec that had already been through `/speckit-clarify`.

### Design and idiom

- **A leaf that takes a `Cell` is a leaf that has no rule, and the rule then gets restated at every
  call site.** The plan's single geometric fragment — position, size and a `Cell`, with three named
  constructors — looked like good factoring: it removes the loop over a rectangle, which is the only
  duplication a leaf has. The evidence that it was wrong was in the plan's own `data-model.md`: the
  box's decomposition table had nine rows and a column giving the four arms of each, which is one
  rule of _The cell_ written out nine times. Six fragments named for their role, each deriving its
  own cell, made the same table two columns narrower and moved the rule to where the model already
  said it lived.
- **The reason to split a type is which decision it owns, not how much code it saves.** The research
  had considered three named types and rejected them on the ground that their `draw` bodies would be
  the same loop. That measured the wrong axis. The loop survives as one private helper either way;
  what only a named type can carry is that a border run closes the side facing its interior and a
  free segment does not.
- **Two enums for four values is right when the four values mean two different things.** `Side` and
  `Direction` are top/right/bottom/left and up/right/down/left. Collapsing them was proposed and
  refused: a side is a place on a cell, a direction is a way to move. Keeping them apart turned out
  to name a layering rule as well — a piece is told sides and positions already computed, and the
  figure above it is the only thing that reasons in directions.
- **A one-armed cell was the answer to a question nobody had asked it.** The model had measured that
  a single-arm cell renders as a segment, concluded that an end must therefore be a chosen glyph,
  and stopped. The measurement was right and the conclusion was not: what an end is for is the join,
  not the character. Two lines meeting at right angles with an arm each render the corner they make,
  which a literal cannot do, because nothing connects into a literal. The visible end was the thing
  being optimized for, and it was the thing least worth having.

### Working this way

- **The review that pays is the one before the plan is agreed.** _No code before the plan is agreed_
  reads like a formality until the plan is read for what it decided rather than for whether it looks
  complete. Four of the five findings were invisible in the spec and visible in the plan, because a
  plan has to name types and a spec does not. All of them would have been code by the time anyone
  noticed.
- **"What reads this?" removed an abstraction that three requirements were built on.** `Extent` had
  a row in the model's _Vocabulary_, a subsection of its own, two functional requirements, an
  acceptance scenario and a line in the trait. Asked what would break if it went, the answer was two
  tests and a restatement of "no position is written twice" — and one of the two tests existed only
  to justify a piece that writes nothing. The removal took a documented model concept with it, which
  is why it needed a record rather than a deletion.
- **A number the spec already recorded was the thing that decided the argument.** "A single-arm cell
  is not an end" was sitting in the spec's _Assumptions_, sourced to the Light table. Re-reading the
  table rather than the assumption showed the measurement held across every single-stroke set and
  that the four half-line characters are in no set at all — which is what made the choice concrete:
  an end costs a picture, and the alternative costs rewriting four keys that already answer.
- **Answers to questions about a prior implementation were worth more than its diagrams.** Four
  Mermaid files gave the module structure; two sentences about what distinguished
  `connectors::Corner` from `corners::TopLeft`, and what `terminals::Line` delegated to, gave the
  layering rule and the join semantics. The diagrams said what the parts were, and only the prose
  said why.

### Trade-offs worth remembering

- **A rule with no answer for one input is not total, however exhaustive its cases look.** The
  research claimed its routing rule landed every pair of directions in a branch. Working the
  alternating-polyline rule by hand against the unpinned families found two arrangements where the
  route rectangle is one cell thick and no alternating path fits at all — which is the same class of
  hole as the `_ => draw nothing` arm that the spec already records as a defect in an earlier
  implementation. The fix was to make the empty route an outcome of the rule and say so, not to add
  a branch.
- **Reversing a conclusion is cheaper than reversing a decision, and the model is where conclusions
  live.** _The glyph at an end and at a head_ was prose in `docs/model.md`, not an ADR, so reversing
  half of it cost one amendment and one new record rather than a supersession chain. That is the
  split working as intended: the model holds design intent that is expected to move, and
  `decisions/` holds what must not be edited.
- **Numbers were retired rather than reused.** FR-010 and FR-011 are gone from the spec and their
  numbers stay empty, because the ADR that withdrew them cites them by number. A renumbered spec
  would have been tidier to read and would have made three references in `decisions/` point at
  requirements that mean something else.

## 2026-09-11 — Splitting the flow into three stages, and giving the branches to xtask

### Rust design and idiom

- **A `const` table of steps is the right shape for a fixed gate and the wrong one for a command
  built at runtime.** `GATE` and `FIX` hold `Step { args: &'static [&'static str] }`, which is
  exactly right for ten invocations known at compile time. The `spec` subcommand builds its
  arguments from an issue number and a slug, so none of them can be `'static`, and trying to reuse
  `Step` would have meant making the whole table generic over a lifetime to serve one caller. Two
  small free functions taking `&[&str]` — one inheriting stdio, one capturing stdout — cost less
  than that and left `GATE` untouched.
- **Shelling out to `gh --jq` is how a program with no JSON dependency reads JSON.** `gh` bundles
  jq, so `gh issue view N --json labels --jq '.labels[].name'` returns one bare label per line and
  the parsing problem disappears. The alternative on offer was a hand-rolled JSON parser inside a
  tool whose own doc comment says it should not be the first to bend the dependency policy.
- **Keeping the pure logic in functions that take plain values is what made any of it testable.**
  Fourteen unit tests cover the slug, the zero-padded number, the branch names, picking the single
  `specs/NNN-*` match and mapping labels to a stage. Everything that touches the network or the
  repository is a thin wrapper around those, and none of it is tested — which is visible rather than
  hidden, because the untested part is the part with no logic in it.

### Working this way

- **A precondition read from a remote is a fact with a timestamp, not a fact.** `spec stage 39 impl`
  created the branch and set `doing`, which looked like the precondition had failed to fire. It had
  not: `origin/main` moved between the session's first fetch, where feature 039 had only `spec.md`,
  and the test's own fetch, where the plan pull request had merged. The tool was right and the
  expectation was stale. Ten minutes went into reading correct code looking for the bug.
- **Making a check fail on purpose needed a repository built for the purpose.** After that merge no
  directory under `specs/` was missing a required file, so there was no natural way to see the
  refusal. A bare clone with `main` moved back to the commit before the merge, and a working clone
  from it, produced the failure in one command — and the failure showed that `git cat-file -e`
  prints its own `fatal:` line before xtask's explanation, which is now silenced. The check would
  have shipped unverified and slightly wrong without the clone.
- **An index that is not derived from the files is not updated by whoever writes the files.** The
  table in `docs/decisions/README.md` lists every ADR with its status. Three ADRs were written and
  two statuses changed; the table knew about none of it until it was edited by hand. Nothing in the
  gate reads that table, which is the same gap issue #26 already names for `specs/`.

### Trade-offs worth remembering

- **Delegating the writing moves the review from the prose to the behavior, it does not remove it.**
  Two subagents produced the three ADRs and the whole subcommand, and the instruction was not to
  read what they wrote. Everything that then needed fixing was found by running things rather than
  by reading them: a test fixture written in Spanish that `cspell` rejected, `/speckit.plan` where
  the installed skill is `speckit-plan`, and git's stderr leaking through a probe. The gate and the
  binary caught all three; a careful read of the diff would have caught two of them and cost more
  than both agents did.
- **A rule that the change introducing it would break is the wrong rule.** The instruction was that
  a tooling change takes a branch without a number, and this work landed on `034-workflow`. What is
  actually load-bearing is that a tooling change takes no `specs/` directory and no stage suffix, so
  that is what CONTRIBUTING.md says. Writing down the literal version would have shipped a document
  violated by its own commit.

## 2026-09-10 — Feature 039 implemented: draw shapes instead of individual cells

### Rust design and idiom

- **A fragment nothing outside its own unit test constructs is dead code, and the compiler means it
  literally.** The plan gave each of the six fragments its own commit, built ahead of the figure
  that would place it. `cargo clippy --all-targets -D warnings` rejected the first one: `Corner`
  compiled and its test passed, but the library's own non-test build never constructed it, which is
  exactly what `dead_code` is for. `--all-targets` runs the test build too, and a fragment used only
  there is still dead in the build that ships. There was no lint to configure around this — the fix
  was to stop pretending a fragment can be shown to work before something draws it, and fold each
  one into the commit of the figure that does. The six fragments and their six cell rules are
  unchanged; only which commit they arrive in moved.
- **The same cell rule serves a corner and a straight run, and `Route` spends that for free.**
  `Corner{ opens: (Side, Side) }` was written for two perpendicular sides and, on inspection, does
  exactly the right thing for two opposite ones too — `Set` on both named sides, `Unset` on the
  rest, which is what a straight-through cell already is. `Route` never asks "is this a bend": it
  compares the orientation of the side facing the predecessor against the one facing the successor,
  and places a `Corner` when they differ, a `Segment` when they don't. The distinction the model
  draws in prose — a bend belongs to one piece, a run to another — fell out of one comparison
  instead of needing a case for each.
- **`checked_add_unsigned` is the idiom for combining an absolute `i32` position with a `u32` extent
  without a cast.** `render.rs` already used it for one offset; this feature reused it for every
  place a position needed to move by a length or a loop index — the box's far corner, a line's `n`th
  cell, a run's or a rectangle's positions — rather than writing `as i32` and inviting
  `clippy::cast_possible_wrap` to say why not.
- **A let-chain collapsed a nested `if let` that `collapsible_if` refused to leave alone.**
  `if width > 2 && height > 2 { if let Some(glyph) = &self.fill { ... } }` became one condition,
  `if width > 2 && height > 2 && let Some(glyph) = &self.fill { ... }`, on the edition's own
  suggestion. Confirmed on the pinned toolchain rather than assumed from the edition number.

### Working this way

- **The general rule, run against the pictures, beat hand-deriving each family.** Research described
  a lattice search — enumerate candidate paths, score by fewest bends, break ties by distance from
  the route rectangle's middle — rather than a switch over the seven direction families. Implemented
  once, as written, it matched all nine pinned pictures on the first test run; the only failure was
  a trailing-space count mistyped by hand while copying a picture out of the spec, caught by running
  the test rather than by proofreading the transcription.
  [ADR-0028](decisions/0028-give-each-fragment-its-own-cell-rule.md)'s reasoning about a vocabulary
  named by role rather than by case held one layer up too: a general search has no family to miss.
- **The dead-code constraint was found by running the gate, not by reading the plan again.**
  `tasks.md` was rewritten twice during implementation, once to fold a scaffold commit into the
  first fragment and once to fold every remaining fragment into its figure — both times because
  `cargo xtask check` failed, not because a re-read of the plan predicted it. A plan reviewed before
  code exists catches a design mistake; it does not catch a commit boundary the compiler will not
  allow, because nothing compiles yet to ask it.
- **The one tie-break the spec built a scenario to test, tested correctly on the first run.**
  Scenario 5 exists because three two-bend candidates tie on bend count and only one is pinned;
  research.md's tie-break — sum of distance from the route rectangle's middle — picked the pinned
  candidate without adjustment. A rule written from a description and checked against the one case
  designed to distinguish it from a simpler rule is a stronger claim than a rule that merely passes
  every test it was fitted to.

### Trade-offs worth remembering

- **Fragment-level commit granularity looked appealing on paper and wasn't buildable.** The tasks
  plan gave `Corner`, `Border` and `Fill` a commit each, ahead of `BoxShape`; all three collapsed
  into `BoxShape`'s own commit once the dead-code constraint was found, and the same happened for
  `Line` and for `Arrow`. The design the six fragments express is unchanged — three figures, six
  single-purpose leaves, one cell rule apiece — but the unit of delivery turned out to be the
  figure, not the fragment, because nothing shorter than a figure gives a fragment a reason to exist
  outside its own test.
- **A general search costs more code than a per-family switch and buys totality instead of
  coverage.** The previous entry recorded a routing rule that had no answer for two arrangements
  because its cases were enumerated by hand and two were missed. The lattice search this feature
  implements has no case list to be incomplete: every candidate the model permits is either found or
  the search correctly reports none exist. The trade is a longer function that searches a nine-point
  lattice instead of a short one that recognizes seven shapes, paid once so that no future family
  can go missing the way two already did.

## 2026-09-11 — Feature 045 implemented: read shapes from a JSON file

### Rust design and idiom

- **An internally tagged enum turns "name the unrecognized kind" into a property of the derive, not
  a branch someone writes.** `ShapeDescription`'s `#[serde(tag = "kind", rename_all = "lowercase")]`
  was the whole of FR-014's "unrecognized kind is a data error naming the value": `serde`'s own
  `unknown variant` message already carries `triangle` and the three names it isn't, because the tag
  is matched before any variant's fields are. No code in this crate checks `kind` against a list.
- **One glyph-validating function served both a required field and an optional one, by wrapping it
  in a one-field struct instead of writing it twice.** `head: Glyph` uses `deserialize_glyph`
  directly; `fill: Option<Glyph>` needed the same one-grapheme check to run only when the field is
  present. Rather than a second function duplicating the `Glyph::new`/`serde::de::Error::custom`
  logic, a local `struct Wrapper(#[serde(deserialize_with = "deserialize_glyph")] Glyph)` let
  `Option::<Wrapper>::deserialize` reuse the exact same function for the "when present" half of an
  optional field.
- **`ShapeDescription::draw(&self, ...)` taking a shared reference meant every mirror type needed
  `Clone` (and `Copy` where cheap) before conversion into a `monospace_core` value, which consumes
  its arguments.** `Pos`, `Size` and the three small enums derive `Copy`; `Endpoint`, carrying a
  `Glyph`, derives `Clone` instead. Deriving `Debug` cascaded the same way once one test used
  `expect_err`, which requires it on the `Ok` type — a reminder that a derive requirement on one
  type is often a derive requirement on everything it contains.
- **`main` returning `ExitCode` instead of unwinding replaced three `.expect()` calls with three
  `return ExitCode::FAILURE` arms.** Each of the two `Result`s the pipeline produces — a read, then
  a parse — gets matched explicitly, printing to stderr and returning before anything reaches
  `print!`; success is the only path that falls through to it. FR-016 ("no panic on any input file")
  became a property of there being no `.expect()` or `.unwrap()` left in `main`, not a claim to take
  on faith.

### Working this way

- **Two fixtures with the same shapes in reversed order, run and diffed, is what actually confirmed
  "draw order changes what's on top" — describing it would not have.** Two temp files, one array
  reversed, run through the built binary and `diff`'d: the outputs differed, and reading them back
  showed which corner glyph each order produced. The reordering assertion in `tests/cli.rs`
  (acceptance scenario 3) is the output `diff` actually printed, not a guess about what stamping
  order ought to do.
- **The Phase 2 checkpoint's "this must land with T012 in one commit" was exactly right, and the
  reason recurred from feature 039: an unreached module compiles clean, so nothing forces the
  grouping except reading the task list before staging.** `description.rs` alone, without
  `mod description;` in `main.rs`, is invisible to `cargo build` — no dead-code warning, no error,
  just a file cargo never looks at. The commit that added it also added the one line that made it
  part of the binary, matching the plan rather than discovering the same constraint the hard way a
  third time.

### Trade-offs worth remembering

- **Mirroring every public field of three core shape types by hand, instead of deriving
  `Deserialize` on the core types themselves, kept `serde` out of `monospace-core` at the cost of
  one duplicate struct per shape.** FR-019 and ADR-0035 already settled this trade before
  implementation; what implementation confirmed is its actual size — four mirror types, three
  conversions, and one field-by-field match in `ShapeDescription::draw` — small enough that the
  duplication reads as the format's own contract (`contracts/description-format.md`) rather than as
  drift waiting to happen.
