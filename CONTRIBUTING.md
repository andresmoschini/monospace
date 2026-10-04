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
- **GitHub CLI** at [cli.github.com](https://cli.github.com). `gh`, authenticated with
  `gh auth login`. Nothing in `cargo xtask setup` installs it: it is authenticated per person, not
  vendored per repository ([ADR-0034](docs/decisions/0034-let-xtask-own-the-feature-branch.md)).

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
through `.opencode/plugins/install-git-hooks.js`
([ADR-0058](docs/decisions/0058-install-the-git-hooks-from-an-opencode-session-too.md)). Working
another way, run `git config core.hooksPath .claude/git-hooks` yourself — and read the next section
first, because without it nothing checks your commits until CI does.

## Everyday commands

```sh
cargo xtask check          # the whole quality gate, about 3.5 seconds
cargo xtask fix            # apply every automatic fix the gate knows about
cargo xtask render         # rewrite the pictures documents carry, from the descriptions beside them
cargo xtask pr body        # write the pull request body this branch calls for
cargo xtask pr open        # push the branch and open the pull request with that body
cargo run -p monospace-cli # run the command-line application
cargo test --workspace     # tests only, when you want a faster loop
```

## Starting work

One change is one GitHub issue, one branch, one pull request, and nothing else is produced. There is
no spec directory to create, no plan to write before the code, and no checklist to tick. The
constitution's
[One question, and where the answer goes](.specify/memory/constitution.md#one-question-and-where-the-answer-goes)
has the whole of it in a table; this is the procedure around that table.

### One issue, and the labels on it

An issue carries a title and two or three sentences: what someone wants, not what has to be built.
Never acceptance criteria, requirements or examples of behavior — those are what the pull request is
for, and an issue that already holds them has settled the change before anybody read it.

The exception is worth a lot. [#104](https://github.com/andresmoschini/monospace/issues/104) carries
the smallest arrangement that shows the problem and says "16 of the 1856 renderings, in 8
arrangements taken from both ends". That is a measurement, and it beats a paragraph of
specification.

`wish` is the board's inbox and stays. The kinds — `capability` for a wish someone had,
`foundational` for what the design demands and nobody asked for, `tooling` for the repository itself
— are a second axis and coexist with it. `deciding` and `building` are gone: where the work stands
is visible on the branch and the open pull request, and a label restating that is a second place to
forget to update.

### Settle the question by reading the model

This step decides everything after it, and it is reading rather than writing. Open the section of
[`docs/model.md`](docs/model.md) the topic belongs to, or
[`docs/diagram-model.md`](docs/diagram-model.md) for the layer above, and ask: **is there more than
one defensible answer to how this should be done?**

- **No** — the model says what to do and the code disagrees, or the model is silent. One branch, one
  pull request, `Closes #N`. There is no decision to agree, so nothing is agreed before the code.
- **Yes** — a sentence of the model would become false, or a caller would observe a new shape. Two
  branches: `NNN-slug-deciding` with `Refs #N`, merged first and carrying no code, then
  `NNN-slug-building` with `Closes #N`.

Concluding "the model says nothing about this" means the section was not read. A model that
enumerates four anchor points says four, and the change that adds a fifth is a decide whether or not
anybody wants it to be one.

The question is which _shape_ of change this is, not whether the work matters. A change to the gate,
the hooks, CI or these documents is not smaller for being about the repository: one branch and one
pull request like any other, and the same body asks the same questions of it.

### Names that carry the number

One string finds every part of a change:

```text
issue     #23
branch    023-read-a-diagram-description              or  023-read-a-diagram-description-deciding
body      target/pr-body.md
```

The slug comes from the issue title, lowercased and hyphenated, at most forty characters. Nothing
derives it any more, so renaming the issue does not rename anything — write the branch once and it
is yours.

### Draw the case

The cheapest step in the flow and the one that removes the most rework. Say which case you want to
look at; you do not need to know what you expect to see.

| To see                           | Use                                                           |
| -------------------------------- | ------------------------------------------------------------- |
| A picture in a tracked document  | A `<!-- render: -->` marker, then `cargo xtask render`        |
| A picture that exists, to pin it | A gallery block, which moves on its own if the picture moves  |
| A picture in a pull request body | The same marker; a body takes one                             |
| A quick case, for you only       | A test that prints, or a JSON description run through the CLI |

Then answer the only question that matters: **what does the drawing say that you did not say
before?**

A case that cannot be drawn yet is a result, not a failed step. A connector attached to a box's
_corner_ has no path today, because `Anchor` has no such variant and no carrier reaches it — say
that in one line, and draw the nearest thing that does exist. A reference with an offset gets you
_near_ a corner rather than _on_ one, which is what makes the named form worth having.

The 142 decided whether a removal froze or dropped a connector without drawing either case, which is
why its cell counts were wrong three times and the decision had to be reopened twice.

### Draft the body

```sh
cargo xtask pr body        # writes target/pr-body.md with the keyword already appended
                           # ... fill every section; one with nothing to say says "None."
cargo xtask pr open        # pushes the branch, then opens the pull request with that body
```

`body` reads the branch, not the issue's label: `-deciding` is the one that proposes and gets
`Refs`, and anything else closes its issue and gets `Closes`. `--refs` covers a change belonging to
an umbrella issue it does not finish, and a branch carrying no number takes the issue as an argument
— `cargo xtask pr body 34`.

`open` refuses three things before it reaches GitHub — a dirty working tree, a body whose sections
are still empty, and `main` — and derives the title from the issue behind `Decide:` or `Build:`, or
from the first commit the branch added where there is no issue.

[`.github/pull_request_template.md`](.github/pull_request_template.md) is the whole artifact, and
two of its sections are the ones most often left out. **The decision**, because a decision with no
rejected alternative has not been made, it has been typed. **What proves it**, because a proposal
nobody can observe is a wish with better formatting, and finding that out on the deciding branch
costs one conversation instead of a merged pull request.

### Build

Run the gate before every commit. No hook runs it for you, and CI reports after the commit exists:

```sh
cargo xtask check
cargo xtask fix     # when you are about to touch several files
```

You decide where a commit ends. One rule: **the commit leaves the tree green.**

Four things worth doing while building, and one worth breaking once:

1. **Write the test first where you can.** It is the cheap way to find out whether the rule is
   understood. A test naming something that is not there does not fail, it does not compile.
2. **The tests are the proof.** Every rule in _What proves it_ becomes an assertion. A rule with
   nothing against it is named as such in the pull request rather than counted as covered.
3. **When the code shows the model was wrong, amend the model in this pull request.** Not optional,
   and not an escape — quote the sentence and replace it in the same commit.
4. **Measure rather than assume.** If a characterization sweep moved, the count goes in the body.
5. **Break the rule on purpose, once.** Remove the guard that makes the behavior work, run the
   tests, and count what went red. If nothing goes red, the tests are wired to the method rather
   than to the rule, and that is cheaper to find here than in review.

### Merge and close

Yours, not delegated:

```sh
gh pr view N --json closingIssuesReferences   # confirm GitHub parsed the Closes
gh pr merge --merge --delete-branch
```

Read the first line. An empty `closingIssuesReferences` means GitHub did not understand the keyword
and the issue is still open, which is the check most often skipped.

After the merge, and only if the pull request asks: accept the snapshots that moved, with
`INSTA_UPDATE=always cargo test --workspace`. **A characterization snapshot is not self-approving.**
Each carries a description demanding a report of how many cases moved, in which families, and
examples with before and after. Reading the diff and saying "that looks right" is the exact failure
those files exist to prevent.

## The quality gate

`cargo xtask check` is the whole gate. Why it is the only definition of "green", and why neither the
hook nor CI may add a check of its own, is
[One definition of green](.specify/memory/constitution.md#iii-one-definition-of-green-non-negotiable)
in the constitution. What follows here is how it behaves.

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
| `wasm`         | Every crate but `monospace-cli` still compiles for `wasm32-unknown-unknown`    |
| `test`         | Unit tests, integration tests and doctests                                     |
| `doc`          | `cargo doc` builds, with broken intra-doc links denied                         |
| `render`       | Every generated picture still matches the description beside it                |

The `numbering` step went with the artifacts it policed. It rejected two entries claiming one number
under `docs/decisions/` and `specs/`, which mattered while a change created a directory and a
record. Neither happens any more, and a check whose subject is a frozen directory is one more thing
to read in order to learn nothing.

The `eol` fixer went the other way round: it outlived its subject and was measured rather than
assumed. It existed because the Spec Kit CLI wrote CRLF on Windows into `.specify/`, which
`.editorconfig-checker.json` was configured to skip. With the CLI gone, `.specify/` holds one
hand-written file and the exclusion is gone with it — and `editorconfig-checker` then covers what
`eol` was written for. Checked rather than reasoned about: a tracked-shape file written with CRLF
was repaired by `cargo xtask fix` with the `eol` step deleted, and `editorconfig-checker` reported
the same file as an error before that. What `eol` did that nothing else does — ask Git which files
are binary and which want CRLF — matters only for a tracked binary, and this repository has none.

### A picture a document generates

The constitution's [Show the rendering](.specify/memory/constitution.md#show-the-rendering) asks
that every picture in a tracked file be either generated or labelled hypothetical. A generated one
is written as a marker carrying its own description, and the `render` step keeps the two together:

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

Leave the fence empty and run `cargo xtask render`; it fills it. The description is the format
`monospace-cli` reads, documented in
[`description-format.md`](specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md).

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
  unchecked. It also owns the line endings, since `.editorconfig` says `end_of_line = lf` for
  everything except `*.bat` and `*.cmd`.
- **`.editorconfig`** is read by prettier and by `editorconfig-checker`, so indentation and line
  endings are configured once and obeyed by both. Its one exception is `*.bat` and `*.cmd`, which
  agree with `.gitattributes` rather than adding a rule: the batch interpreter does not run a file
  whose lines end in a bare LF, and a fixer that disagreed with Git here would be the thing breaking
  the file.
- **Line endings** are the one concern with a third reader — `git add`, which refuses a CRLF file
  rather than converting it. `editorconfig-checker` is what settles it, and it is a fixer as well as
  a check, so `cargo xtask fix` repairs one before it can reach a commit.

### Automatic fixes

`cargo xtask fix` runs `fmt`, `render`, `prettier`, `markdownlint` and `editorconfig` in that order,
each in its writing mode instead of its checking mode. Order is not incidental here the way it is
for `check`: these steps rewrite the same files `check` only reads, so a formatter that ran last
would win regardless of which one was "right". `render` writes before the Markdown formatters so
that what it puts in a fence is theirs to normalize; the other content formatters follow;
`editorconfig` runs after them because it owns files none of the others touch — `LICENSE`, the TOML
files, the dotfiles — and otherwise only confirms what the earlier steps already left clean.

It also runs last for a second reason: every step above it writes, and the ending of a line is the
last thing a byte should be decided on.

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

## The hooks

`pre-commit` runs the gate. `commit-msg` checks the message with commitlint. Both live in
`.claude/git-hooks/`.

**They only run if they were installed, and a session is what installs them.** A commit made from a
plain terminal in a clone where no session has opened runs no hooks at all, and nothing says so —
the commit simply succeeds. That is deliberate, not an oversight:
[ADR-0005](docs/decisions/0005-install-the-git-hooks-from-claude-code.md) records the trade and the
cost, and [ADR-0058](docs/decisions/0058-install-the-git-hooks-from-an-opencode-session-too.md)
closed the second client's half of the gap. CI runs the same gate on every push and pull request, so
the boundary that actually holds is there; the hooks are fast feedback in front of it. Check yours
with `git config core.hooksPath`.

**The pre-commit hook checks your working tree, not what you staged.** With unstaged changes
present, or after `git add -p`, it verifies files that are not the ones being committed, so a commit
can pass and still be broken. Stashing to close that gap risks losing work if the hook is
interrupted, which is the worse failure. If you stage selectively, run `cargo xtask check` on a
clean tree before trusting it.

What is worth adding here is why `--no-verify` bites: a bypassed gate is worse than no gate, because
the log then claims a green history that was never checked
([One definition of green](.specify/memory/constitution.md#iii-one-definition-of-green-non-negotiable)
makes the bypass a rule).

## Commits

Messages follow [Conventional Commits](https://www.conventionalcommits.org), enforced by the
`commit-msg` hook.

Which prefix to use follows from
[Structural and behavioral change never share a commit](.specify/memory/constitution.md#v-structural-and-behavioral-change-never-share-a-commit).
Everything else — `build`, `ci`, `docs`, `style`, `chore`, `test` — is neither, which is most of the
tooling in this repository.

How much goes in one commit is settled by
[Demonstrable increments](.specify/memory/constitution.md#ii-demonstrable-increments) rather than by
taste: you choose where the change reaches green, and nowhere else.

Write the body for someone who was not there. What the diff does is visible; why it does that is
not.

One thing to know about the body: do not let a colon-terminated word start a line. commitlint reads
`word:` at the beginning of a line as a footer token, splits the message there, and warns that the
footer has no blank line before it. It is only a warning, so it lands unnoticed. Reword with an em
dash, or re-wrap so the word sits mid-line.

### Fixing a commit

Which corrections belong in the commit that got them wrong, and which get a commit of their own, is
[Fixing a commit](.specify/memory/constitution.md#fixing-a-commit) in the constitution. What is
specific to this repository is that the rule holds after pushing, which is not the usual convention.
It works here because a feature branch has an owner, and whoever pulls someone else's branch accepts
that it can be rewritten underneath them. The limits are `main`, which is never rewritten, and
`--force-with-lease`, which aborts instead of clobbering an update you had not seen. The cost is
that GitHub marks inline review comments on a rewritten commit as outdated.

Rewriting means proving the branch green again, on each rewritten commit rather than on the tip
alone, because neither rebase nor cherry-pick fires the hook —
[One definition of green](.specify/memory/constitution.md#iii-one-definition-of-green-non-negotiable).

A frozen record is the exception, and only in one direction: a correction to what a record under
`docs/decisions/` says about itself is edited in place whatever the commit history does. Nothing is
added there, so nothing there is current.

### The session trailer

A commit made from an agent session can carry up to three trailers, and each is a different handle
on the same conversation rather than the same one twice.

- **`Claude-Resume`** holds a Claude Code session's local id. Reopen the conversation with
  `claude --resume <id>`. The `commit-msg` hook writes it, so it is present whenever the hooks are.
- **`Claude-Session`** holds a URL that opens a Claude Code session in a browser. Claude Code writes
  it itself when Remote Control is enabled, which is a setting outside this repository — so it is
  present sometimes and absent otherwise.
- **`OpenCode-Session`** holds an OpenCode session's id, stamped by the same hook from the value
  `.opencode/plugins/session-trailer.js` injects
  ([ADR-0060](docs/decisions/0060-stamp-the-opencode-session-into-the-commit.md)). Neither client's
  id resumes the other, which is why they are separate keys rather than one key whose shape depends
  on the client.

List them across the history with:

```sh
git log --format='%h %(trailers:key=Claude-Resume,valueonly)'
```

[ADR-0007](docs/decisions/0007-rename-the-session-trailer-to-claude-resume.md) covers why the first
two have separate keys, and
[ADR-0006](docs/decisions/0006-record-the-claude-session-in-commit-trailers.md), which it
supersedes, covers why any of them is a trailer rather than a plain line — a non-trailer line at the
end of a message silently invalidates `Co-Authored-By` along with it.

It is a convenience, not a record. Transcripts live outside the repository and do not survive a new
machine, so the reasoning that matters still belongs in the commit body or in an ADR. If a commit
body only makes sense with the transcript open, the body is wrong.

## How the rules reach a session

Two files, and every harness reads the same two.

[`.specify/memory/constitution.md`](.specify/memory/constitution.md) holds the rules and
[AGENTS.md](AGENTS.md) holds how to work here. `CLAUDE.md` is two import lines and nothing else:

```markdown
@.specify/memory/constitution.md

@AGENTS.md
```

Those are imports, not mentions. Claude Code expands them at launch, so both files are in context
from the first message of every session instead of being files the session has to remember to open.
To confirm it loaded, run `/context` and look for them under "Memory files" — if either is missing,
nothing errors, the session simply works without the rules, which is the failure mode worth checking
after touching either file.

OpenCode expands no import, so `AGENTS.md` is read directly and orders the reading itself, with the
constitution first.

**The constitution kept its path on purpose.** It was `.specify/memory/constitution.md` because Spec
Kit put it there, and eighty-nine links across `docs/`, the crates and these files point at it by
path — twelve of them at an anchor inside it. Nothing in the gate resolves a link to another file,
so moving it would break all eighty-nine silently in exchange for a tidier directory name.

Both files are read on **every** call of a session, and a session costs its length squared
([ADR-0027](docs/decisions/0027-control-token-cost-through-session-discipline.md)), so they hold the
most expensive prose in the repository. That is why the constitution's earlier Sync Impact Reports
moved to [`constitution-history.md`](docs/decisions/constitution-history.md): they are history for a
person, and a person can open a file. It is also why the Spec Kit prompt files — 3,811 lines across
two harnesses, read on demand — are gone rather than trimmed.

## Cross-references

The rule and its reasoning are in
[Cross-references](.specify/memory/constitution.md#cross-references) in the constitution: cite a
section by its name, in practice by anchor, and not by number.

Nothing checks any of it. `markdownlint` validates a link fragment against the headings of the same
file and stops there, so a link to a file that does not exist, or to an anchor in a different file
that does not exist, passes the gate.
[Issue #13](https://github.com/andresmoschini/monospace/issues/13) tracks closing it.

Frozen history keeps its numbered citations. An accepted record is not edited for style, and a link
into one of them is still the shortest way to say what happened.

## Frozen history

Four trees are kept and nothing adds to them. They are worth more as a record of why the code came
to be this way than the space they take costs, and the reasoning in most of them has nowhere else to
go now.

| What                                     | Lines  | Replaced by                                                         |
| ---------------------------------------- | ------ | ------------------------------------------------------------------- |
| `specs/NNN-slug/` — 19 features          | 24,101 | The pull request body. Eight artifacts per feature became one file. |
| `docs/decisions/` — 68 records           | 7,323  | `docs/model.md` for what is observable, rustdoc for what is not.    |
| `docs/learning-log.md` — the first 2,182 | 2,182  | Nothing; it is still appended to.                                   |
| `docs/specs/` — features 0001–0005       | 1,145  | `specs/`, which `docs/specs/README.md` already said was frozen.     |

[the learning log](docs/learning-log.md) is the exception worth stating: it is frozen above its
current end and open below it. An increment still ends with an appended entry, because what was
learned is the point of the repository and three lines is not what went wrong.
