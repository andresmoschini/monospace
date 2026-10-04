# AGENTS.md

Rust workspace for **Monospace**, an ASCII diagramming library. [CONTRIBUTING.md](CONTRIBUTING.md)
owns the process; this file holds what an agent cannot infer from the code.

**Prefer a rule to a value.** Almost everything below names the file that defines something rather
than repeating what it currently says. A copied count, version or filename goes stale on the next
change and nothing fails when it does — the gate checks code, not prose. When you need a value, read
the file this points at.

## Read less, not more

Every call re-reads the whole context, so a session costs its length squared rather than its length.

- **Take a line range, or grep with context.** `docs/model.md`, `docs/diagram-model.md` and a spec
  are large enough that opening one whole is a decision rather than a reflex. Appending needs no
  read at all.
- **Delegate a lookup that spans files**, so the files never enter this context.

## The gate is the only definition of green

```sh
cargo xtask setup    # installs the Node tooling the gate needs
cargo xtask check    # the whole gate — what CI runs, and what you run before committing
cargo xtask fix      # every fixer, in the one order that works; run before check
```

- **Never add a check outside `cargo xtask check`.** `ci.yml` says so in its first comment: a gate
  people run and a gate CI runs must not disagree, or "green" means two things. Steps are the `GATE`
  array in `xtask/src/main.rs`, fixers are `FIX`. Adding one means editing that array.
- **Every step runs even after one fails**, so one run reports everything wrong rather than making
  you fix problems one at a time. The summary line names how many steps ran, so don't quote that
  number anywhere — including here.
- **A step passes or fails on its exit code alone.** The holes are known and deliberate: `rustfmt`
  reports unusable config and still exits 0, and Cargo reports a missing resolver the same way. A
  green exit is not proof the tool was satisfied.
- `check` and `fix` **refuse to start** unless the installed Node tooling matches
  `package-lock.json` _by content_. Comparing timestamps instead reported a stale tree after every
  branch switch, which is the false alarm that teaches people to ignore the message. Run
  `cargo xtask setup`; don't work around it and don't run only the Rust half.
- `FIX` is ordered, and `GATE` is not — because the fixers rewrite the same files `GATE` only reads,
  so a later step would win regardless of which was right. Read the array and its comments before
  adding or reordering anything.
- `clippy --fix` is deliberately excluded from `fix`: it rewrites code that needs a human to read
  the diff, which does not belong in a command meant to run unattended. Run it by hand:

  ```sh
  cargo clippy --fix --workspace --all-targets --allow-dirty --allow-staged -- -D warnings
  ```

  The `--allow-*` flags are required, because clippy otherwise refuses to touch a working tree that
  is not clean — which mid-task is most of the time. Expect some warnings to survive: the pedantic
  group this project denies includes lints needing human judgment rather than a mechanical rewrite.

### Focused runs

```sh
cargo test --workspace                 # unit + integration + doctests
cargo test -p monospace-core           # one crate
cargo test -p monospace-core <name>    # filter by name substring
cargo test --doc                       # doctests alone
cargo run -p monospace-cli                   # the shipped demonstration
cargo run -p monospace-cli -- path/to.json   # one picture, no caption
```

- `rust-toolchain.toml` pins the compiler **exactly**, along with the components and targets the
  gate needs. Updating it is its own task with its own commit: the point of pinning is that a gate
  failure is attributable to your change, never to a new stable release that taught a formatter a
  new opinion overnight.
- `xtask` has no dependencies on purpose, because a tool guarding the project's dependency policy
  should not be the first thing to bend it. Its module docs say so; keep it that way.
- **Nothing outside `GATE` opens a branch or writes a body**, and `cargo xtask feature` and
  `cargo xtask pr` are typed rather than triggered. `feature`'s rung is a judgement about the model
  and is never defaulted to the rung that needs one; `CONTRIBUTING.md` says which is which.

## Crate boundaries

| Crate                  | Owns                                                                          |
| ---------------------- | ----------------------------------------------------------------------------- |
| `monospace-core`       | **All** domain logic: cells, strokes, glyphs, buffer, shapes, routing, render |
| `monospace-diagram`    | The model: a diagram as an ordered set of shapes, drawable                    |
| `monospace-glyph-sets` | Glyph tables the core does not ship as built-in data                          |
| `monospace-cli`        | Description → `Diagram` → draw. **No logic of its own**                       |
| `xtask`                | All repository automation                                                     |

- **Core must stay free of terminal, command-line, filesystem and user-interface concerns** so the
  same code can back the TUI and a WebAssembly build. The gate cross-compiles every crate but the
  CLI for the WebAssembly target on every run, so this is compiler-enforced rather than a
  convention: a dependency that won't build there fails the gate, which is the point. The target and
  the crates it covers are both named in `GATE`.
- Adding a crate: add it to `members` in the root `Cargo.toml` **and** give it
  `[lints] workspace = true`. Without that line it silently opts out of the workspace lints, and the
  editor stops showing you what the gate enforces.
- Non-dev dependencies are pinned with an exact `=`. Follow the existing entries; don't take a
  version from this file.

## Evolving the model

[CONTRIBUTING.md's _The model moves_](CONTRIBUTING.md#the-model-moves) owns this, including the
reasoning. What it asks of a pull request, as three instructions:

- **Amend the model in the same pull request as the code that proves it wrong**, never in a later
  cleanup. Both documents are written to be amended slice by slice, so name the slice rather than
  restating the whole model or inventing your own vocabulary.
- **Do not preserve an earlier definition out of deference to it**, and do not treat an existing
  problem's framing as a constraint on its answer.
- **Do not hold a branch while the understanding is still incomplete.** Write what you currently
  believe and mark what is provisional — that is what makes the later refactor a legible move rather
  than archaeology.

## Snapshots are not where insta puts them

**There is no `.config/insta.yaml`.** Each crate redirects insta in code, from its own `snap`/`pin`
helper, into a directory under that crate's `src/snapshots/` — split into two kinds, because they
answer different questions:

- `characterization/` — sweeps over a range too wide to assert by hand, pinned so a change is
  _reported_ rather than reviewed.
- `gallery/` — the value, the picture, and the table of cells that produced it, for the difference a
  picture cannot show.

Read the helper rather than assuming insta's default location, and don't create a top-level
`snapshots/`.

```sh
cargo insta review    # needs cargo-insta, which this repository does not install
cargo insta accept
# or, with no cargo-insta: INSTA_UPDATE=always cargo test --workspace
```

**A characterization snapshot is not self-approving.** Each one carries a description demanding a
report of how many cases moved, in which families, and examples with before and after. Reading the
diff and saying "looks right" is the exact failure those files exist to prevent.

## Pictures in documents are generated or labelled

Three traps, none of which fails where you would see it. The rule and the marker grammar are in
[CONTRIBUTING.md](CONTRIBUTING.md#a-picture-a-document-generates) and, precisely, in the parser at
`xtask/src/render.rs` — read that rather than reconstructing a marker from memory.

- **Rendered pictures have their trailing blanks trimmed.** Never re-pad a fence out to the window
  width; the gate's whitespace check rejects it.
- **A hand-drawn picture is invisible to the step.** Nothing mechanical can tell one from a
  generated one, so labelling it `Hypothetical — hand-drawn, not generated.` is on you.
- **Never illustrate the grammar in a fence that contains a fence.** Prettier rewrites the result
  into something unbalanced and leaves the damage in an unrelated fence further down. Describe it in
  prose, or use a fence four backticks wider than anything nested inside it.

## Formatting ownership

[CONTRIBUTING.md's _Which tool owns which file_](CONTRIBUTING.md#which-tool-owns-which-file) is the
table. Three things that are easy to get wrong from here:

- **Adding a crate** means adding it to `members` in the root `Cargo.toml` **and** giving it
  `[lints] workspace = true`. Without that line it silently opts out of the workspace lints, and the
  editor stops showing you what the gate enforces.
- **Non-dev dependencies are pinned with an exact `=`.** Follow the existing entries; don't take a
  version from this file.
- **The repository is American English only.** `cspell` runs over every tracked file, and a British
  spelling gets _changed_, not added to `project-words.txt`: the en-GB dictionary was deliberately
  removed, and adding those back one at a time would rebuild it. Add only genuinely unknown domain
  terms.

Workspace lints sit at `warn` locally so a half-written edit doesn't shout, and the gate promotes
them to failures with `-D warnings`. Every public item needs a doc comment. To permit a lint, add it
to `[workspace.lints]` with a comment saying why — never suppress it at the use site.

## Documentation

- **Every module carries a `# Design notes` section**: why it is the way it is, what was considered
  and rejected, what would break if it changed. Reasoning behind code belongs there rather than in a
  comment or a chat transcript.
- The domain model lives in `docs/model.md` (the buffer and render model) and
  `docs/diagram-model.md` (what holds a figure after it has been drawn).
- **The model is expected to change**, and both model documents say so in their own status lines.
  See _Evolving the model_ for what that asks of a pull request carrying one.
- Cross-reference sections **by name or anchor, never by number** — a number drifts from the thing
  it points at, an anchor can be looked up.
- **Claims are meant to be measured, so re-measure them.** The README and CONTRIBUTING state counts
  of tests, renderings and gate steps. When your change moves one, update it in the same commit —
  and prefer, in both files and here, a sentence that stays true to a mechanism over a number that
  will not.
- `cspell` runs over every tracked file, and the repository is **American English only**. A British
  spelling gets _changed_, not added to `project-words.txt`: the en-GB dictionary was deliberately
  removed, and adding those back one at a time would rebuild it. Add only genuinely unknown domain
  terms, and prefer a real dictionary in `cspell.jsonc` when one exists for the domain.

## Branch, commit and pull request flow

The branch suffix is the only thing telling the rest of the tooling what shape of change this is.
`xtask/src/pr.rs` maps suffix → template and closing keyword; read it rather than relying on a
remembered pair.

```sh
cargo xtask pr body        # writes target/pr-body.md for this branch, keyword already appended
                           # a tooling branch carries no issue number: pass one
                           # `--refs` for a change that belongs to an issue it does not finish
#   ...fill every section; one with nothing to say says "None."
cargo xtask pr open        # pushes, then opens it. Refuses a dirty tree, empty sections, and main
```

- **The closing keyword goes in the pull request body, never in a commit message.** Only the body
  shows the link before the merge, a wrong number is an edit rather than a history rewrite, and no
  single commit is "the" one closing work that took several. `pr body` appends the right one for the
  shape rather than leaving you to remember which.
- Conventional Commits, checked by the `commit-msg` hook — but by **no gate step**, so a message
  that never went through an installed hook passes CI. Tracking the gap is
  [#162](https://github.com/andresmoschini/monospace/issues/162).
- **In a commit body, never let a colon-terminated word start a line.** commitlint reads `word:` at
  line start as a footer token, splits the message there and warns that the footer has no blank line
  before it. It is only a warning, so it lands unnoticed. Reword with an em dash, or re-wrap so the
  word sits mid-line.
- Feature branches **may be rewritten after pushing**, which is not the usual convention and works
  here because a feature branch has an owner who accepts that it can change underneath them. `main`
  never is. Use `--force-with-lease`. Neither rebase nor cherry-pick runs the gate, so after
  rewriting, run `cargo xtask check` on **each** rewritten commit rather than only the tip.
- A commit should leave the tree green. The `pre-commit` hook runs the gate, but it checks the
  working tree rather than the staged snapshot, so with unstaged changes present it verifies files
  that are not the ones being committed — see [the hooks](CONTRIBUTING.md#the-hooks).

## The CLI's output shape is load-bearing

`cargo xtask render` embeds `monospace-cli`'s output directly, so **a picture bound for a Markdown
fence cannot arrive wrapped in prose**. That is why the bare run demonstrates and the run with a
path renders once and nothing else: applied to a file someone hands the binary, the demonstration's
assumptions would be imposed on their description. Only the bare run carries captions, and they
describe the shipped `assets/demo.json`, not the file you passed.

Adding or reordering a demonstrated picture means updating `crates/monospace-cli/tests/cli.rs`,
which appends below the first picture precisely so the cell coordinates its later assertions read
keep naming the same cells.
