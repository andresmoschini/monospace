# Contributing

Everything here assumes a clean machine. If a step needs something this document does not mention,
that is a defect in the document.

## What you need first

- **Git.**
- **[rustup](https://rustup.rs).** Not a Rust toolchain — rustup installs the right one by itself.
  `rust-toolchain.toml` pins the exact compiler along with `rustfmt`, `clippy`, `rust-src` and the
  `wasm32-unknown-unknown` target, and rustup reads that file.
- **Node.** The version is in `.nvmrc`. Four of the checks are npm packages with no Rust equivalent,
  which is how a Rust project ends up with a second toolchain.
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

## Everyday commands

```sh
cargo xtask check          # the whole quality gate, about 3.5 seconds
cargo xtask fix            # apply every automatic fix the gate knows about
cargo xtask render         # rewrite the pictures documents carry, from the descriptions beside them
cargo xtask feature        # open the branch a change is worked on; `help` lists its verbs
cargo xtask pr body        # write the pull request body this branch calls for
cargo run -p monospace-cli # run the command-line application
cargo test --workspace     # tests only, when you want a faster loop
```

## The hooks

`pre-commit` runs the gate. `commit-msg` records which agent session wrote the commit and then
checks the message with commitlint. Both live in `.claude/git-hooks/`.

**They only run if they were installed, and a session is what installs them.**
`.claude/settings.json` sets `core.hooksPath` from a `SessionStart` entry when a Claude Code session
opens; `.opencode/plugins/install-git-hooks.js` does the same for an OpenCode session. A commit made
from a plain terminal in a clone where no session has opened runs no hooks at all, and nothing says
so — the commit simply succeeds. To install them by hand:

```sh
git config core.hooksPath .claude/git-hooks
```

That is deliberate, not an oversight. CI runs the same gate on every push and pull request, so the
boundary that actually holds is there; the hooks are fast feedback in front of it. Check yours with
`git config core.hooksPath`.

**The pre-commit hook checks your working tree, not what you staged.** With unstaged changes
present, or after `git add -p`, it verifies files that are not the ones being committed, so a commit
can pass and still be broken. Stashing to close that gap risks losing work if the hook is
interrupted, which is the worse failure. If you stage selectively, run `cargo xtask check` on a
clean tree before trusting it.

If you are about to change several files, or you are done for the day, `cargo xtask fix` applies
every fix the gate knows about first. It does not guarantee `check` passes afterwards: `clippy` and
`cspell` have no automatic fix at all, and `markdownlint` and `editorconfig` can both report
violations they know about but cannot rewrite.

Every step of the gate runs even after one fails, so one run tells you everything that is wrong. A
step passes or fails on its exit code alone, and only the summary line matters:

```text
all 12 checks passed
```

A summary naming a step means that step failed; everything above it is the detail of why, and the
steps below it already ran.

## Starting work

Work starts from an issue and ends in a merged pull request. The issue is a phrase meant to start a
conversation, and it is allowed to stay that way: a title and two or three sentences, never
acceptance criteria, requirements or examples. What an issue may carry instead is evidence — the
smallest arrangement that shows the problem, or the count of cases it reaches — and that is worth
more than anything a specification would have said about it.

Between those two points there is one question, and its answer decides how much gets written down:

> **Does this change force a choice between more than one defensible answer?**

The usual sources of that choice are a sentence in [the model](docs/model.md) or
[the diagram model](docs/diagram-model.md) that the change would make false, and something a caller
outside this crate can observe changing shape. The second is why the question is not only about the
model: a library's public surface can change with no sentence of prose anywhere going stale, and
that is still a decision somebody has to agree to.

Three answers, three rungs:

| Rung       | The answer                                                          | Branches                              | Pull request                 |
| ---------- | ------------------------------------------------------------------- | ------------------------------------- | ---------------------------- |
| **Fix**    | No — the model says what to do, and the code disagrees              | `NNN-slug`                            | the default template         |
| **Slice**  | No — the model is silent or already agrees, and it is still missing | `NNN-slug-building`                   | `building.md`                |
| **Decide** | Yes                                                                 | `NNN-slug-deciding`, then `-building` | `deciding.md`, `building.md` |

A change to the gate, CI or these documents has no model behind it and is a **Fix**: a branch with
no suffix, and the default body.

Two of the three rungs add nothing to the ordinary shape of a pull request. That is the whole
design: a rung is worth having only if it is cheap enough that you never skip it, so the deciding
stage is reserved for the one case where there is genuinely something to decide.

[`docs/workflow.md`](docs/workflow.md) walks one issue through the whole flow, naming at each step
what a person does and what an agent does. This section is the rules; that one is the procedure.

### Draw the case first

Before any of this, make the case drawable and look at it. For this project that is twenty lines:
write the fixture, run it, read the picture.

This step earns its place by being the one that removes the most rework. A decision taken about a
picture nobody drew is a bet, and in a renderer the bet fails in ways only the drawing shows — a
cell carrying one arm renders as the run through it rather than the junction it was meant to be, and
two counts that look like they should subtract turn out to overlap. Each of those was a rewritten
paragraph, not a compiler error.

The drawing reaches the spec, not only the pull request body, because the body is what a reviewer
reads today and the spec is what someone reads in a year. When the case cannot be drawn — no carrier
reaches it yet — say so in the spec instead, and that sentence is worth more than a paragraph
describing what it would probably look like.

### The deciding stage

Only the **Decide** rung opens one. The branch is `NNN-slug-deciding`, it merges a proposal with no
code, and the building branch that follows requires that proposal in `main`.

Two artifacts, and neither is optional:

- **The spec**, at `specs/NNN-slug.md`, copied from
  [`.github/spec-template.md`](.github/spec-template.md). One file, flat, never deleted: the record
  of what a change was decided to be is worth more than the fact that it held, and the building pull
  request completes it rather than replacing it.
- **The pull request body**, from
  [`.github/PULL_REQUEST_TEMPLATE/deciding.md`](.github/PULL_REQUEST_TEMPLATE/deciding.md).

**The spec is more precise than the body, not less.** The body is what a reviewer reads while the
change is being agreed; the spec is what someone reads afterwards, without the conversation.
Everything in it has to stand alone, which means naming things rather than gesturing at them, saying
what happens with the values, carrying the picture, and recording counts measured rather than
estimated.

The body carries **no line count**, and there was one until it was found not to be kept — a number
in a file nothing enforces is one more value to keep true. What bounds it instead: every line either
names a decision with the alternative that was rejected, or names something a caller outside this
crate can observe. Rendered pictures do not count toward that, because a picture costs less
attention than a paragraph and is the cheaper half of an argument.

**Decide what a caller can observe, and nothing a caller cannot.** The test is one question: could
someone outside this crate write a test that fails if this changes? For a library that is more often
yes than for an application, and it is why the stage exists at all — a caller who is not in the
conversation cannot be asked afterwards. So the **public surface belongs here**: types, variants,
signatures and changed behavior, in a section of its own. What stays out is the shape behind it —
private structure, a borrow strategy, which file a test lives in, how the work splits into commits.
The plan the 142 wrote, discarded and rewrote named all four, which is a large part of why it was
wrong twice.

**The model is not amended here.** It moves in the pull request that carries the code proving it
wrong — see [The model moves](#the-model-moves) — and this repository's history already worked that
way without saying so: four deciding pull requests touched the model zero times, three building ones
touched it three times.

The second stage sequences the work; it does not freeze the understanding behind it. A proposal is
agreed from a partial understanding, so building it is a normal place to find that understanding
incomplete — including about the model. That is not a reason to hold the branch: it is the reason
[the model moves](#the-model-moves) applies here as much as in the deciding stage.

The deciding stage used to be the default for every feature. It is not any more, because it was
routinely merged and then reopened by the building stage that followed it — a proposal written at
that size is not read closely enough to be agreed with.

### Names that carry the number

One string finds every part of a feature:

```text
issue     #90
file      specs/090-a-shape-offers-its-corners.md
branches  090-a-shape-offers-its-corners              (fix)
          090-a-shape-offers-its-corners-building     (slice)
          090-a-shape-offers-its-corners-deciding     (decide, first)
          090-a-shape-offers-its-corners-building     (decide, second)
```

The slug comes from the issue title, lowercased and hyphenated, at most forty characters. `NNN` is
the issue number in three digits, and the suffix is what tells `cargo xtask pr` which body to write
— which is why the suffix and the rung cannot be chosen separately. The spec's filename is the same
string, so one name finds the issue, the spec and both branches.

### Opening the branch

`cargo xtask feature` opens it from the issue, which is where the slug comes from. Nothing here is
part of the quality gate: `check` has no notion of `git` branches or GitHub labels.

```sh
cargo xtask feature new 90                # the rung comes from the issue's labels
cargo xtask feature new 90 --rung decide  # or say it, when the labels cannot
cargo xtask feature build 90              # the decide rung's second branch, once the first merged
cargo xtask feature use 90                # a clone onto whichever branch exists
```

`new` reads the issue's title for the slug, opens the branch off `main`, and moves the issue's
label: `wish` becomes `deciding` on a decide rung and `building` on the other two. The rung is
defaulted from the labels — `bug` is a fix, a capability or a wish is a slice — because that is
right often enough to be worth a keystroke, and **it is always printed with the reason**, so a
default taken on faith reads as one. A **decide** rung is never defaulted to: it is the only one
where a wrong answer costs something, and reading the model is what puts a change there.

`build` is the decide rung's second branch, and it refuses until the deciding pull request has
merged — it asks GitHub rather than reading a file, because the merged proposal is now the handoff.

`use` asks which branch exists rather than storing which rung opened it, because `building` is one
label for two rungs and a fix's branch carries no suffix to tell it from a slice's.

Nobody has to remember the three verbs: `cargo xtask feature help` prints them, and what each rung
means, on one screen.

### The spec

`specs/NNN-slug.md`, one file per change, flat. It is copied from
[`.github/spec-template.md`](.github/spec-template.md), it is written by the deciding pull request,
and the building pull request completes it.

**It is never deleted.** A spec that was revised is the record of a decision turning out to be
wrong, and that is readable in a way a commit message is not. What keeps it from going stale is the
status line, which is one of two sentences and says whether the code is still what was decided. When
it is not, the building pull request fills in _What the implementation changed_ and flips the status
to say so — and the gate refuses the file while that section is empty, which is the one thing here
that catches a mistake made _after_ a merge.

There are five sections, and every one of them earns its place: the decisions, the public surface,
what the change does not decide, what proves it, and what building it changed. The one most often
left out is _What proves it_, and it is the one that makes a proposal checkable rather than merely
plausible — a change nobody can observe is a wish with better formatting.

A spec may carry a `<!-- render: -->` marker like any tracked Markdown file, so `cargo xtask render`
keeps its pictures true. The `specs` step of the gate checks the shape of every tracked spec; what
it cannot check is whether the code does what the spec says, which is what
[docs/workflow.md](docs/workflow.md) is for.

### Commits during implementation

A commit leaves the tree green: it runs `cargo xtask check` and only exists at a boundary where that
passes. The `pre-commit` hook enforces it — see [The hooks](#the-hooks), including why it is worth
running by hand as well.

### Opening and closing the pull request

The keywords that close an issue go in the pull request's body, never in a commit message. Three
reasons, and the third is the one that decides it:

- The link is visible before the merge. Only the pull request gives you that; a keyword in a commit
  is invisible until it lands.
- A wrong number is edited out of a body. In a commit it is a history rewrite, and a rewrite owes
  the gate a run on **every** rewritten commit rather than only the tip.
- No single commit is "the" one that closes work that took several.

Which keyword depends on the stage, and `cargo xtask pr` appends it rather than asking you to
remember which:

| Rung                              | Branch              | Body                                | Keyword     |
| --------------------------------- | ------------------- | ----------------------------------- | ----------- |
| **Decide**, deciding              | `NNN-slug-deciding` | `PULL_REQUEST_TEMPLATE/deciding.md` | `Refs #N`   |
| **Decide** or **Slice**, building | `NNN-slug-building` | `PULL_REQUEST_TEMPLATE/building.md` | `Closes #N` |
| **Fix**                           | anything else       | `pull_request_template.md`          | `Closes #N` |

The deciding pull request carries `Refs` rather than `Closes` because it is normally the first of
two: the building pull request that follows is the one that finishes the issue. Nothing enforces
that pairing — a change can be agreed and built in a single pull request, and `Closes` in it then
reads correctly on its own. `cargo xtask pr` appends `Refs` because the deciding branch is the shape
where the mistake is likely, not because the stage is required.

### Opening one

Two commands, because the body has to be filled between them:

```sh
cargo xtask pr body        # writes target/pr-body.md for whatever this branch is
                           # a branch carrying no number: `pr body 34`
                           # `--refs` where the change belongs to an issue it does not finish
#                          ... fill every section; one with nothing to say says "None."
cargo xtask pr open        # pushes the branch, then opens the pull request with that body
```

`body` reads the branch: `-deciding` and `-building` take the two feature bodies, and anything else
is a fix and takes the default. `open` refuses three things before it reaches GitHub — a dirty
working tree, a body whose sections are still empty, and `main` — and derives the title from the
issue (`Decide:` or `Build:`) or, for a fix, from the first commit the branch added. `--title`
overrides it and `--body-file` reads from somewhere else.

### Closing one

```sh
gh pr view N --json closingIssuesReferences   # confirm GitHub parsed a Closes
gh pr merge --merge --delete-branch
```

[`.github/PULL_REQUEST_TEMPLATE/`](.github/PULL_REQUEST_TEMPLATE/README.md) explains why there are
three bodies and why every section stays.

## The quality gate

`cargo xtask check` is the whole gate, and it is the only definition of "green". Nothing may add a
check of its own — not CI, not a script someone runs before pushing. What follows here is how it
behaves.

Every step runs even after one fails, so a single run reports everything wrong rather than making
you fix problems one at a time. A step passes or fails on its exit code alone.

| Step           | What it checks                                                                 |
| -------------- | ------------------------------------------------------------------------------ |
| `fmt`          | Rust formatting, `cargo fmt --check`                                           |
| `prettier`     | Formatting of Markdown, JSON and JSONC, including prose width                  |
| `markdownlint` | Markdown structure: heading levels, duplicate headings, bare URLs, code fences |
| `editorconfig` | Line endings, final newlines and trailing whitespace on every tracked file     |
| `cspell`       | Spelling, in code and prose alike                                              |
| `specs`        | Every tracked spec carries its status line and the five sections               |
| `clippy`       | Lints, including `pedantic`, with warnings denied                              |
| `build`        | The workspace compiles, tests and all                                          |
| `wasm`         | Every crate but `monospace-cli` still compiles for `wasm32-unknown-unknown`    |
| `test`         | Unit tests, integration tests and doctests                                     |
| `doc`          | `cargo doc` builds, with broken intra-doc links denied                         |
| `render`       | Every generated picture still matches the description beside it                |

### A picture a document generates

Every picture in a tracked file is either generated or labelled hypothetical. A generated one is
written as a marker carrying its own description, and the `render` step keeps the two together:

````markdown
<!-- render:
{ "canvas": { "origin": { "x": 0, "y": -1 }, "size": { "width": 2, "height": 6 } },
  "next_id": 2,
  "shapes": [ { "kind": "connector", "id": "#1",
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

Leave the fence empty and run `cargo xtask render`; it fills it. The description is the JSON format
`monospace-cli` reads.

Three things to know:

- **All four lines are required**, including the closing `<!-- /render -->`. Without it a picture
  would end at its fence, and the next ordinary fence in the file would be swallowed. A marker
  missing any of them is reported by file and line rather than skipped.
- **A marker shown inside a longer fence is an illustration, not an instance** — which is what the
  four-backtick fence above is doing. The rule is `CommonMark`'s: only a fence at least as long
  closes one.
- **A hand-drawn picture is invisible to the step.** Nothing mechanical can tell one from any other
  fence, so labelling it `Hypothetical — hand-drawn, not generated.` is on you.

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
  endings are configured once and obeyed by both. Its one exception is `*.bat` and `*.cmd`, which
  agree with `.gitattributes` rather than adding a rule: the batch interpreter does not run a file
  whose lines end in a bare LF, and a fixer that disagreed with Git here would be the thing breaking
  the file.
- **Line endings** are the one concern with a third reader — `git add`, which refuses a CRLF file
  rather than converting it — and the `eol` step is how that is settled. It is not inside
  `editorconfig-checker` because that tool answers from `.editorconfig`, matched by glob, while
  `eol` answers from `.gitattributes`, matched by attribute. The two files can disagree, and when
  they do it is `git add` that breaks rather than a formatter. `*.bat` and `*.cmd` are the live
  case: both configurations agree they are CRLF on purpose, and only Git's answer is asked.

### Automatic fixes

`cargo xtask fix` runs `fmt`, `render`, `prettier`, `markdownlint`, `editorconfig` and `eol` in that
order, each in its writing mode instead of its checking mode. Order is not incidental here the way
it is for `check`: these steps rewrite the same files `check` only reads, so a formatter that ran
last would win regardless of which one was "right". `render` writes before the Markdown formatters
so that what it puts in a fence is theirs to normalize; the other content formatters follow;
`editorconfig` runs after them because it owns files none of the others touch — `LICENSE`, the TOML
files, the dotfiles — and otherwise only confirms what the earlier steps already left clean; and
`eol` runs last, because every step above it writes and the ending of a line is the last thing a
byte should be decided on.

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
  6 checks passed" when ten are configured is worse than an error.

## Commits

Messages follow [Conventional Commits](https://www.conventionalcommits.org). `commitlint` is
installed and configured (`.commitlintrc.json`), but **the gate does not run it** — there is no step
for it in `GATE`, so a malformed message passes CI. Until one exists, the format is a convention
worth following rather than a rule that will catch you.

`build`, `ci`, `docs`, `style`, `chore` and `test` are the prefixes most of the tooling in this
repository uses. None of them is a behavioral change, so none of them needs a structural one
alongside it.

How much goes in one commit, while implementing a feature, is settled by
[Commits during implementation](#commits-during-implementation) rather than by taste.

Write the body for someone who was not there. What the diff does is visible; why it does that is
not.

One thing to know about the body: do not let a colon-terminated word start a line. commitlint reads
`word:` at the beginning of a line as a footer token, splits the message there, and warns that the
footer has no blank line before it. It is only a warning, so it lands unnoticed. Reword with an em
dash, or re-wrap so the word sits mid-line.

### Fixing a commit

Some corrections belong in the commit that got them wrong, and some get a commit of their own. What
is specific to this repository is that the rule holds after pushing, which is not the usual
convention. It works here because a feature branch has an owner, and whoever pulls someone else's
branch accepts that it can be rewritten underneath them. The limits are `main`, which is never
rewritten, and `--force-with-lease`, which aborts instead of clobbering an update you had not seen.
The cost is that GitHub marks inline review comments on a rewritten commit as outdated.

Rewriting means proving the branch green again, on each rewritten commit rather than on the tip
alone, because neither rebase nor cherry-pick runs the gate. Run `cargo xtask check` yourself at
each one; nothing does it for you.

An accepted proposal is not exempt: a pull request superseding one says so and says why, whatever
the commit history does. Nothing here is permanent, including an agreement.

## Cross-references

Cite a section by its name, in practice by anchor, and not by number. A number is a thing that
drifts; a name is a thing a reader can look up.

Nothing checks any of it. `markdownlint` validates a link fragment against the headings of the same
file and stops there, so a link to a file that does not exist, or to an anchor in a different file
that does not exist, passes the gate.
[Issue #13](https://github.com/andresmoschini/monospace/issues/13) tracks closing it.

Prose written before this convention keep their numbered citations. They were true when written, and
an accepted record is not edited for style.

## The model moves

The model is expected to change, and to change soon. Treat a definition as the best statement of a
partial understanding made so far, never as a fixed point to be preserved.

**Write code that reflects your current understanding, even where that understanding is partial.**
This is deliberate. It is the debt Ward Cunningham describes: not writing code well, but writing it
to match what you currently believe, so that when the understanding moves, the code shows what you
were thinking when you wrote it and refactoring into what you think now is a legible move rather
than archaeology. The debt is the record. The alternative — waiting until you understand the whole
problem before writing anything — is not rigor, it is delay that leaves nothing to refactor.

Three things follow, and they are the practical form of the above:

- **When the code shows the model is wrong, change the model in the same pull request.** A model
  document describing something the code no longer does is worse than no model document, because it
  reads as authoritative while being wrong.
- **Do not preserve an earlier definition out of deference to it.** How the domain used to be
  described is not a constraint, and neither is a name. The vocabulary is ours to rename the moment
  the understanding does, and doing it late costs more than doing it early.
- **Neither model document is finished.** Each is amended slice by slice, and each says so in its
  own status line. A model change arriving for the first time in the building stage is ordinary, not
  an escape — see [the building body](.github/PULL_REQUEST_TEMPLATE/building.md).

**For each desired change, make the change easy, then make the easy change** — Kent Beck's rule, and
the general working criterion here. Read it as a sequence rather than a slogan. The unblocking is
often the hard part of the work, and spending a change on _only_ making the next one easy is
legitimate, as long as every commit leaves the tree green.

This is why "small and focused" is a default and not a law. A change that is hard _because the model
is wrong_ should usually carry the model change with it, rather than deferring it to a cleanup that
never arrives. Making the change easy and then making it easy is the whole method; a pull request
too small to hold the unblocking is not focused, it is incomplete.

## Where the reasoning lives

Three places, and each answers a different question.

- **The module's own rustdoc.** Every module carries `# Design notes` saying why it is the way it
  is, what was considered and rejected, and what would break if it changed. That is the place for
  the reasoning behind a piece of code.
- **`docs/model.md` and `docs/diagram-model.md`.** The domain: what a cell is, how shapes compose,
  what the vocabulary means. A change to the model belongs there, and it is expected there: see
  [The model moves](#the-model-moves).
- **The pull request body.** What was considered and not taken, and what is deliberately left open.
  A rationale that only makes sense with a transcript open belongs in the body, because the
  transcript does not survive a new machine.

What is deliberately not here: a separate numbered record per decision. A number that nothing checks
is a number that drifts from the code it points at, and a citation to a file that does not exist is
worse than none — it looks like a reason and answers nothing.
