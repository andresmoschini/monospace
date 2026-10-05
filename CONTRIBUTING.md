# Contributing

Everything here assumes a clean machine. If a step needs something this document does not mention,
that is a defect in the document.

`AGENTS.md` is the other entry point and is read as well: this file is the process and the commands,
that one is what a change is here and which of the usual habits are wrong in this repository. Where
the two disagree, this one is right.

## What you need first

- **Git.**
- **[rustup](https://rustup.rs).** Not a Rust toolchain — rustup installs the right one by itself.
  `rust-toolchain.toml` pins the exact compiler along with `rustfmt`, `clippy`, `rust-src` and the
  `wasm32-unknown-unknown` target, and rustup reads that file.
- **Node.** The version is in `.nvmrc`. Four of the checks are npm packages with no Rust equivalent;
  this is how a Rust project ended up with a second toolchain, and it is the reason
  `cargo xtask setup` exists.
- **GitHub CLI** at [cli.github.com](https://cli.github.com). `gh`, authenticated with
  `gh auth login`. Nothing in `cargo xtask setup` installs it: it is authenticated per person, not
  vendored per repository.

## Setup

```sh
rustup toolchain install # reads rust-toolchain.toml
cargo xtask setup        # runs npm ci
```

The first downloads a toolchain even if you already have the same version under a different name,
because rustup treats `stable` and `1.98.1` as different installations. The second takes about half
a minute the first time.

There is no third step for the hooks: a session installs them, whichever harness it is — a Claude
Code session through the `SessionStart` entry in `.claude/settings.json`, an OpenCode session
through `.opencode/plugins/install-git-hooks.js`. Working another way, run
`git config core.hooksPath .claude/git-hooks` yourself — and read _The hooks_ first, because without
it nothing checks your commits until CI does.

## Everyday commands

```sh
cargo xtask check                   # the whole quality gate, about 3.5 seconds
cargo xtask fix                     # apply every automatic fix the gate knows about
cargo xtask render                  # rewrite the pictures documents carry, from the descriptions beside them
cargo xtask change open 163         # open the branch of a change of one stage
cargo xtask change use 163 building # check out the branch of one stage of issue #163
cargo xtask change status 163       # where issue #163 stands, and what to do next
cargo xtask pr body                 # write the pull request body this branch calls for
cargo run -p monospace-cli          # run the command-line application
cargo test --workspace              # tests only, when you want a faster loop
```

The stage is the suffix the branch carries: `deciding`, `building`, or nothing at all for a change
of one stage. Which one applies is the answer to the question in step 2 of
[`docs/workflow.md`](docs/workflow.md), so it is given rather than guessed —
`cargo xtask change open 163 deciding` is the form that takes it.

## The quality gate

`cargo xtask check` is the whole gate, and it is the **only** definition of green: the `pre-commit`
hook runs it, CI runs it, and a new check is added to that entrypoint rather than to either of them.
Two definitions of green is the failure this arrangement exists to prevent — passing locally and
passing in CI have to mean the same thing.

Every step runs even after one fails, so a single run reports everything wrong rather than making
you fix problems one at a time.

| Step           | What it checks                                                                    |
| -------------- | --------------------------------------------------------------------------------- |
| `fmt`          | Rust formatting, `cargo fmt --check`                                              |
| `prettier`     | Formatting of Markdown, JSON and JSONC, including prose width                     |
| `markdownlint` | Markdown structure: heading levels, duplicate headings, bare URLs, code fences    |
| `editorconfig` | Line endings, final newlines and trailing whitespace on every tracked file        |
| `numbering`    | No two entries under `specs/` claim the same number                               |
| `specs`        | Every spec carries its nine sections, and its frontmatter answers to its `status` |
| `cspell`       | Spelling, in code and prose alike                                                 |
| `clippy`       | Lints, including `pedantic`, with warnings denied                                 |
| `build`        | The workspace compiles, tests and all                                             |
| `wasm`         | Every crate but `monospace-cli` still compiles for `wasm32-unknown-unknown`       |
| `test`         | Unit tests, integration tests and doctests                                        |
| `doc`          | `cargo doc` builds, with broken intra-doc links denied                            |
| `render`       | Every generated picture still matches the description beside it                   |

**A step passes or fails on its exit code alone.** That is a deliberate limit with known holes:
`rustfmt` reports that `group_imports` needs nightly and exits 0, and Cargo reports a missing
`workspace.resolver` the same way. In both cases the tool knows something is wrong, says so, and the
gate does not notice. Closing that would mean matching on the text the tools print, which is brittle
in a different and less obvious way. **Watch the output rather than trusting a green run.**

**Every step has to be shown to break something when it is taken out.** An entry is kept by removal,
not by addition: in a linter or a spell checker, an entry that changes nothing in silence permits
errors. That is why `numbering` is still here with one tree instead of two — the rule it holds, that
no two specs claim one number, is the guarantee that survives; and it is why `specs` is here —
remove it and renaming a heading of the nine goes unreported.

The two easiest to doubt are the two that read `specs/`. `numbering` fails when two files claim one
`NNN`. `specs` fails when a heading is renamed, when `status` is not one of the four values, when
`decided` is missing from an agreed spec, and when an agreed spec still reads `_pending_`. The
listing is `git ls-files ':(glob)specs/*.md'` and the `:(glob)` is load-bearing: a bare `specs/*.md`
is a git pathspec where `*` crosses `/`, and it reported all 1,994 files of the old tree as broken
specs. There is a test for it, and the test was checked by reverting the prefix.

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
  unchecked. It reads every tracked file, with no exclusions: an exclusion for a tree that does not
  exist is a lie that also skips a whole subtree the moment one appears.
- **`.editorconfig`** is read by prettier and by `editorconfig-checker`, so indentation and line
  endings are configured once and obeyed by both. Its one exception is `*.bat` and `*.cmd`, which
  agree with `.gitattributes` rather than adding a rule: the batch interpreter does not run a file
  whose lines end in a bare LF, and a fixer that disagreed with Git here would be the thing breaking
  the file.
- **Line endings** are the one concern with a third reader — `git add`, which refuses a CRLF file
  rather than converting it — and the `eol` fixer is how that is settled. `editorconfig-checker`
  reads the same rule, so it is belt-and-braces rather than the only reach.

### Automatic fixes

`cargo xtask fix` runs `fmt`, `render`, `prettier`, `markdownlint`, `editorconfig` and `eol` in that
order, each in its writing mode instead of its checking mode. Order is not incidental here the way
it is for `check`: these steps rewrite the same files `check` only reads, so a formatter that ran
last would win regardless of which one was "right". `render` writes before the Markdown formatters
so that what it puts in a fence is theirs to normalize; the other content formatters follow;
`editorconfig` runs after them because it owns files none of the others touch; and `eol` runs last,
because every step above it writes and the ending of a line is the last thing a byte should be
decided on.

`eol` is the only step that is not a tool from `package.json`. It rewrites the CRLF of any file Git
would stage — tracked or new, ignored files excepted — and it asks `git ls-files --eol` which files
those are, whether their bytes are text at all, and which ones `.gitattributes` wants in CRLF. That
last question is why it exists rather than a `sed` line: a batch file and a PNG are the two answers
that a rule of the form "CRLF becomes LF" gets wrong, and Git already holds both.

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
  6 checks passed" when thirteen are configured is worse than an error. It compares by **content**,
  not by timestamp, so editing `package.json` or the lockfile means running `setup` again even
  though the files on disk are the same bytes.

## Settle the question by reading the model

Before deciding anything about a change, read what already decides it.

[`docs/model.md`](docs/model.md) owns the domain vocabulary and the design intent.
[`docs/diagram-model.md`](docs/diagram-model.md) owns the layer above it. **If a slice needs a rule
the model does not have, the model changes first**, in the same increment — a spec names the
sections it implements and does not restate them, and a rule that lives only in a spec is a rule
with one reader.

**The model owns the design, and a procedure is not a rule of the model.** What belongs in it is
bounded by one test: _if this changes, must anything outside the module that implements it change
with it?_ Where the answer is no, the decision is module-level and belongs in that module's rustdoc
under `Design notes` — see _Where the reasoning lives_. A rule observable from outside is the
model's, whatever module happens to implement it today.

And **a document collecting per-figure procedures must not be created.** Two figures may join two
points differently, so a shared file invites the second to conform to the first — the same premature
generalization, one floor lower.

The question that decides whether a change takes one branch or two is in `AGENTS.md`: _does this
change force a choice between more than one defensible answer?_ Read the model before answering it,
because the model frequently makes the answer obvious, and a change that only writes code does not
need a spec at all.

### Six passes over what you wrote

There is no second artifact to cross-check against any more, so this is what replaced the
cross-artifact verification, and it is weaker than what it replaced. Six passes over a spec before
it merges:

1. **Duplication.** Two requirements asking for the same thing under different names; keep the
   clearer phrasing and delete the other.
2. **Ambiguity.** **Flag vague adjectives — _fast_, _scalable_, _secure_, _intuitive_, _robust_ —
   that lack measurable criteria**, and flag unresolved placeholders: `TODO`, `TKTK`, `???`,
   `<placeholder>`. An adjective with no criterion behind it is a preference wearing a noun.
3. **Underspecification.** A requirement with a verb but no object, or no outcome to observe.
4. **Alignment.** Anything conflicting with a MUST in _The rules_ below, and a missing mandated
   section.
5. **Coverage gaps.** **A rule of `## Behavior` with nothing in `## What proves it`** — the same
   defect the old pipeline found as _requirements with zero associated tasks_, pointing at the file
   that exists now. Also: something in `## What proves it` that no rule needs.
6. **Inconsistency.** The same concept named two ways, or an entity the spec mentions that it never
   introduces.

**When you ask a question, never use a topic label, a section heading or a requirement id as the
question itself.** `## Behavior` is not a question; _"does a reference's offset grow when its figure
is displaced?"_ is. A label restates where the answer goes, which is the one thing the reader
already knows, and it produces a discussion of the section rather than of the decision. **Write one
plain-language "Why it matters" sentence under it**, so the reader can tell whether their answer
changes anything before they give one.

**At most three clarification markers per session.** A fourth means the change is not understood
yet, and the answer is to read the model, not to ask a fourth question.

### Four kinds of gap, when the code and the spec disagree

Classify every one rather than listing it. The classification says what kind of work it is:

- **`missing`** — the required work is absent from the code entirely.
- **`partial`** — it exists but does not yet satisfy what was agreed.
- **`contradicts`** — the code does something that conflicts with the agreed intent.
- **`unrequested`** — the code does something nobody asked for. **This is not automatically a
  defect**: it may be the better answer, in which case it is a decision to take and record, not a
  bug to fix silently.

## Draw the case

Where the subject of an option, a question, a decision or a change is something this project
renders, **the artifact shows the rendering**; prose accompanies the picture rather than replacing
it. Two paragraphs describing two options are a preference. A drawing is believed faster than a
sentence, and where the alternative is cheap to implement a spike that generates its picture beats
an argument about what it would look like. Where the subject does not render, no picture is required
and one added anyway is noise.

Every picture in a tracked file is one of two things, unambiguously:

- **Generated** — produced from a description the file carries, regenerated by `cargo xtask render`,
  and re-checked by `cargo xtask check`.
- **Hypothetical** — hand-drawn, showing what no code produces yet, and labelled on the spot.
  Nothing mechanical can tell one from any other fence, so labelling it is on you.

A generated one is written as a marker carrying its own description:

````markdown
<!-- render:
{ "canvas": { "origin": { "x": 0, "y": -1 }, "size": { "width": 2, "height": 6 } },
  "next_id": 2,
  "shapes": [ { "kind": "connector", "id": 1,
    "from": { "at": { "kind": "point", "x": 0, "y": 0 }, "leaving": "up",
      "terminal": { "kind": "glyph", "glyph": "▼" } },
    "to": { "at": { "kind": "point", "x": 0, "y": 3 }, "leaving": "down",
      "terminal": { "kind": "glyph", "glyph": "▲" } },
    "stroke": "light" } ] }
-->

```text

```

<!-- /render -->
````

Leave the fence empty and run `cargo xtask render`; it fills it. Three things to know:

- **All four lines are required**, including the closing marker. Without it a picture would end at
  its fence, and the next ordinary fence in the file would be swallowed. A marker missing any of
  them is reported by file and line rather than skipped.
- **A marker shown inside a longer fence is an illustration, not an instance** — which is what the
  four-backtick fence above is doing. The rule is `CommonMark`'s: only a fence at least as long
  closes one.
- **Do not write the marker's own syntax inside an HTML comment.** Anything that scans for the first
  `-->` closes the comment early, and the text after it reads as content — which fills a section
  nobody wrote. Refer to it without spelling it out.

The description is the JSON format `monospace-cli` reads, documented in the module doc of
`crates/monospace-cli/src/description.rs`, which owns it.

## Build

### What one increment is

Every commit that lands leaves the workspace building, all validations green, and
`cargo run -p monospace-cli` producing output. **A bare `cargo run` is not the acceptance command**:
the workspace holds more than one binary, so it is ambiguous.

**Specs are sliced thin.** Several small changes beat one large one, and a spec over 250 lines means
the cut is too coarse — split the change rather than compressing the prose. Prettier reflows at a
hundred columns rather than refusing, so rewording buys nothing: the words come back on the next
`cargo xtask fix`.

**Where the fastest route to a change conflicts with the route that teaches better Rust design or
better spec-driven practice, the second is taken.** A plan that saves effort by collapsing the
process is rejected. That is not a preference for ceremony: it is that the practice is the point of
this repository, and the time is affordable because the time is what is being spent.

**A structural change never shares a commit with a behavioral one.** A structural commit changes no
behavior: existing tests pass unchanged and no test is added or modified. `refactor` carries the
structural one, `feat` and `fix` the behavioral one, and they are not mixed — the prefix is what
makes the distinction visible in the log without reading diffs. Large refactors go expand/contract:
add alongside, migrate, then remove, each step green and committed separately. **If the preparatory
refactor turns out to be hard, stop and say so before it starts.** That is a design signal to
discuss.

**Every claim is measured before it is written.** Behavior is not asserted before it has been
observed, least of all in a document or a commit message: run it first, then write down what
happened. "Is X faster, smaller, better?" is answered with a measurement — a median over several
runs, and an explicit statement when the noise is larger than the difference. **A new check is
verified by making it fail on purpose and then restoring it**; a green run only proves the command
ran. A requirement accepted with nothing to verify it is **named as such** where it is recorded,
rather than described as tested. Before the gate is trusted on a change that touches it, run it on a
fresh clone: files written by hand skip the transformations Git applies on checkout, so a working
copy can be green while the repository is broken.

### Dependencies

**A dependency is not added without asking the maintainer first.** Before adding or pinning any
version, its publication date is verified to be **at least seven days old**, and the version and
that date are reported. Domain logic prefers the standard library; infrastructure concerns prefer
idiomatic, well-established crates over reinvention. Versions are pinned with `=` exactly.

`xtask` has no dependencies and must not become the first thing to bend that: it guards the policy,
and orchestrating subprocesses and propagating exit codes is `std`'s job.

### What is in scope

Four crates, and the boundary is that `monospace-core` holds all the domain logic while the other
three hold none of it, and the core's public API may not assume a CLI, a TUI or a terminal:
`monospace-core`, the library; `monospace-cli`, a minimal non-interactive consumer producing
diagrams from the terminal; `monospace-glyph-sets`, the glyph sets the core does not ship as
built-in data; and `monospace-diagram`, the model holding a diagram after it is drawn.

**A plan proposing any of these is stopped and renegotiated rather than quietly widened:**
WebAssembly bindings, a web front-end, non-terminal GUIs, persistence, collaboration, and export
formats beyond plain text.

### The public surface

**Public library APIs are documented with rustdoc as they are introduced, not retrofitted.** A
spec's `## Public surface` says signatures, not implementation: whether something is a `Vec` or a
`HashMap` inside is not a promise to anyone.

### Testing

Two kinds of test, kept apart in separate directories where they are snapshots:

- A **contract test** pins a decision, and its name or comment says which rule of the model it
  holds. **Changing one is changing the decision**, so it needs a decision in `## The decision`.
- A **characterization test** records what the code does over a range too wide to assert by hand. A
  change to it is the consequence of a decision taken elsewhere, not a decision of its own. The file
  says so at its head, and **it covers its range whole rather than by sample**.

**A characterization is not described as reviewed.** What is required instead, each time one moves,
is a report of **how many cases moved, in which families, and three representative examples with
their before and after**. That report is what is read. A claim of review nobody can keep turns the
safety net into a reason not to look.

**A behavior rule with no test named against it is an unfinished rule**, and belongs in
`## What proves it` unfinished rather than claimed done.

## Where the reasoning lives

Three homes, and the test that picks between them is the one in _Settle the question by reading the
model_.

- **The module's rustdoc, under `Design notes`,** owns every module-level decision. This is the
  default home, not the fallback. It says why the module is the way it is, what was considered and
  rejected, and **what breaks if it changes**. A decision lands here and is promoted out of it in
  the increment that makes something outside able to observe it — never in advance.
- **[`docs/model.md`](docs/model.md) and [`docs/diagram-model.md`](docs/diagram-model.md)** own the
  domain: the vocabulary, the rules, the design intent.
- **The pull request body** owns what was considered and not taken, for the change it belongs to.
  The `## The decision` section is that record.

**A numbered record per decision is deliberately not one of the homes.** _A number that nothing
checks is a number that drifts from the code it points at, and a citation to a file that does not
exist is worse than none_ — it looks like a reason and answers nothing.

**An artifact does not restate what another already says; it cites it by name.** A paragraph that
would change nobody's decision if it were deleted is deleted. That is why `CONTRIBUTING.md` and
`AGENTS.md` do not repeat each other, and why a spec does not restate the model.

## The sweep

A moved snapshot is a question, not a failure. `cargo insta review` needs `cargo-insta`, which
`cargo xtask setup` does not install — install it by hand.

The report that goes in the pull request's `## The sweep` is: **how many cases moved, in which
families, and three examples with before and after.** `Unchanged.` is a real answer and worth
saying: no behavior changed is a fact, not an absence, and 0 of 1856 is the honest report for a
change that touches no product code.

## The hooks

`pre-commit` runs the gate. `commit-msg` checks the message with commitlint and stamps which agent
session produced the commit. Both live in `.claude/git-hooks/`.

**They only run if they were installed, and a session is what installs them.** A commit made from a
plain terminal in a clone where no session has opened runs no hooks at all, and nothing says so —
the commit simply succeeds. That is deliberate: CI runs the same gate on every push and pull
request, so the boundary that actually holds is there, and the hooks are fast feedback in front of
it. Check yours with `git config core.hooksPath`.

**The pre-commit hook checks your working tree, not what you staged.** With unstaged changes
present, or after `git add -p`, it verifies files that are not the ones being committed, so a commit
can pass and still be broken. Stashing to close that gap risks losing work if the hook is
interrupted, which is the worse failure. If you stage selectively, run `cargo xtask check` on a
clean tree before trusting it. **This is also why deletions are staged before the gate is run**:
`render`, `numbering` and `specs` read the index, so a file deleted in the working tree but not
staged is still read as though it were there.

**`git commit --no-verify` is never used.** A bypassed gate is worse than no gate, because the log
then claims a green history that was never checked. When the hook fails, fix the cause or stop and
report.

### Commits

Messages follow [Conventional Commits](https://www.conventionalcommits.org), enforced by the
`commit-msg` hook. Which prefix to use follows from _Build_ above. Everything else — `build`, `ci`,
`docs`, `style`, `chore`, `test` — is neither structural nor behavioral, which is most of the
tooling in this repository.

Write the body for someone who was not there. What the diff does is visible; why it does that is
not.

One thing to know about the body: do not let a colon-terminated word start a line. commitlint reads
`word:` at the beginning of a line as a footer token, splits the message there, and warns that the
footer has no blank line before it. It is only a warning, so it lands unnoticed. Reword with an em
dash, or re-wrap so the word sits mid-line.

**Fixing a commit.** A correction that changed nothing for anyone — a typo, a wrong status line, a
formatting slip — belongs in the commit that got it wrong. When something was actually wrong, in
behavior or in a claim someone could have acted on, the fix is its own commit and says what was
wrong. This holds **after pushing**: `main` is never rewritten, and every force push uses
`--force-with-lease`.

**A rewrite runs the gate on every rewritten commit, not only on the tip.** Neither `git rebase` nor
`git cherry-pick` fires the hook, and a commit that was green before the rewrite may have been green
by accident. The cost is that GitHub marks inline review comments on a rewritten commit as outdated,
and the limit is `main`, which is never rewritten.

### The session trailer

A commit made from an agent session can carry up to three trailers, and each is a different handle
on the same conversation rather than the same one twice.

- **`Claude-Resume`** holds a Claude Code session's local id. Reopen the conversation with
  `claude --resume <id>`. The `commit-msg` hook writes it.
- **`Claude-Session`** holds a URL that opens a Claude Code session in a browser. Claude Code writes
  it itself when Remote Control is enabled, which is a setting outside this repository — so it is
  present sometimes and absent otherwise.
- **`OpenCode-Session`** holds an OpenCode session's id, stamped by the same hook from the value
  `.opencode/plugins/session-trailer.js` injects.

**They are separate keys on purpose because neither client's id resumes the other**, and one key
whose shape depends on the client would make a commit's provenance unreadable after the fact. List
them with:

```sh
git log --format='%h %(trailers:key=Claude-Resume,valueonly)'
```

It is a convenience, not a record. Transcripts live outside the repository and do not survive a new
machine, so the reasoning that matters belongs in the commit body or in the pull request. **If a
commit body only makes sense with the transcript open, the body is wrong.**

## Cross-references

Cite a section of another document by its name rather than by a number that could drift.

**Nothing checks any of it.** `markdownlint` validates a link fragment against the headings of the
same file and stops there, so a link to a file that does not exist, or to an anchor in a different
file that does not exist, passes the gate.
[Issue #13](https://github.com/andresmoschini/monospace/issues/13) tracks closing that. Until it is
closed, **a dangling link is invisible until somebody clicks it**, which makes fixing the ones a
change creates part of writing the change.
