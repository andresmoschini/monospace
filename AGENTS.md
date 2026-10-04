# AGENTS

Read in this order, and stop where your question is answered:

1. [`.specify/memory/constitution.md`](.specify/memory/constitution.md) — every rule, the scope and
   the process. Long enough that opening it whole is a decision.
2. This file — how to work here, and which of the usual ways are wrong in this repository.
3. [`CONTRIBUTING.md`](CONTRIBUTING.md) — setup, the commands, which tool owns which file. Read the
   section, not the file.

Where any of them restates a rule, the constitution is the one to follow and the duplication is a
defect.

## What a change is

One GitHub issue, one branch, one pull request. The constitution's
[One question, and where the answer goes](.specify/memory/constitution.md#one-question-and-where-the-answer-goes)
says which shape it takes and where the record goes; nothing else is produced. There is no spec
directory to create, no plan to write before the code, and no checklist to tick.

The pull request body is the record. It carries the decision table and _What proves it_, and those
two sections are the whole difference between a change that can be reviewed and one that has to be
re-derived from the diff.

## The steps

1. **Read the model, then answer the one question.** `docs/model.md` for the core,
   `docs/diagram-model.md` for the layer above. Quote the sentence that would become false. If you
   cannot quote one, the change is not a `decide` and there is nothing to agree first.
2. **Draw the case.** A `<!-- render: -->` marker and `cargo xtask render`, a gallery block, or a
   test that prints. Then say what the drawing showed that the prose did not. A case that cannot be
   drawn is a result — say so in one line.
3. **Draft the body.** `cargo xtask pr body` writes the template with the keyword already appended.
   Fill every section; one with nothing to say says `None.`
4. **Build.** Test first where you can: it is the cheap way to find out whether the rule is
   understood, and a test naming something that is not there does not fail, it does not compile.
5. **Amend the model where building showed it wrong,** in the same commit, quoting the sentence it
   replaces. Not optional and not an escape.
6. **`cargo xtask check`, then `cargo xtask pr open`.**

On a `decide`, step 3 is its own pull request and it merges before any code is written.

## Working with the maintainer

- Do not invent something new because the ecosystem was not known well enough to ask for the usual
  thing. Where it is settled practice, the conventional answer is the answer.
- Talk in whatever language is being written in. What lands in the repository is English.

## Habits the gate cannot check

These are the failure modes the rules above have a habit of failing in. They are not the rules; the
rules are in the constitution.

- **Say it when the code disagrees with a document**, in the first paragraph. Do not reconcile
  silently, and do not change code to match a document without asking which of the two is wrong.
- **Default a new decision to module-level**, into that module's rustdoc under `Design notes`, and
  promote it in the increment that makes something outside able to observe it.
- **Look for the record of the subject before writing a new one.** A change to an existing decision
  revises it; it does not become a second document beside it.
- **Show the rendering in the question, not beside it.** Both options side by side, and anything
  hand-drawn labelled on the spot.
- **Cut the prose a picture already carries.** Keep the sentence that says why, drop the one that
  says what.
- **A snapshot that moved is a question, not a failure.** Report how many cases moved, in which
  families, and three examples with before and after — and ask whether that is the movement wanted.
  A characterization is not self-approving.
- **Never argue that changing something repeatedly is expensive.** Where that is true the cost is
  paid by a test, a migration or a public API, and written there.
- **Do not manufacture a fix to fill a commit.** An increment with nothing in it says so.

## Running a session

Every call re-reads the whole context, so a session costs its length squared rather than its length.

- **One pull request body per session**, and clear the context between them. The body is the
  handoff; the conversation is not.
- **Read the part, not the file.** `docs/learning-log.md`, `docs/model.md`, `docs/diagram-model.md`
  and `docs/glyph-sets.md` are large enough that opening one in full is a decision. Take a line
  range or grep with context. Appending needs no read at all.
- **Delegate a lookup that spans files**, so the files never enter this context.
- **Do not open an artifact nobody asked for.** If you catch yourself writing a plan, a design
  document or a task list, the change does not need one; say what you would have written it in and
  put that in the pull request body instead.

## Commands

```sh
cargo xtask check          # the whole quality gate; the hook and CI run this and nothing else
cargo xtask fix            # every automatic fix the gate knows about
cargo xtask setup          # install the Node tooling the gate needs
cargo xtask render         # rewrite every picture a tracked document carries, from its description
cargo xtask pr body        # write target/pr-body.md; fill it, then `cargo xtask pr open`
cargo xtask pr open        # push the branch and open the pull request with that body
cargo run -p monospace-cli -- path.json   # render one description; bare `cargo run` is ambiguous
cargo test --workspace     # tests only, for a faster loop
cargo insta review         # accept a moved snapshot; report what moved first
```

`CONTRIBUTING.md` covers setup, which tool owns which file, and what to do when a check fails.

## Frozen history

Nothing adds to these and nothing new cites one for a rule. They are kept because they record why
the code came to be this way, which is worth more than the fact that it holds — and a link into one
is still the shortest way to say what happened.

| What                                           | Why                                                                                                              |
| ---------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `specs/NNN-slug/` — 19 features, 24k lines     | The eight artifacts per feature Spe Kit asked for. The flow that produced them is gone.                          |
| `docs/decisions/` — 68 records                 | Architecture decision records. Their reasoning moved into `docs/model.md` and into rustdoc where it still holds. |
| `docs/learning-log.md` — the first 2,182 lines | What was learned while those decisions were taken. Appending to it is still how an increment ends.               |
| `docs/specs/` — features 0001–0005             | The home before `specs/`.                                                                                        |

## For this harness specifically

- **Two plugins live in `.opencode/plugins/`, and nothing in the gate can execute them.** They are
  the only thing that installs the git hooks here and the only thing that puts a session id in a
  commit, so a failure in either is silent. Each logs a line when it arms, and it reaches the TUI
  and not `~/.local/share/opencode/log/opencode.log` — measured, that file has never held a
  `[monospace]` line whether the plugin loaded or died, so do not read its absence there as
  evidence. The log file's own `loading plugin` and `failed to load plugin` entries are the readable
  signal, and the second carries the cause. A reload unloads a plugin without warning: anything that
  checks out these files, a rebase among them, drops both until they load again. A local plugin's
  bare imports resolve from this project's `node_modules`, so whatever a plugin imports has to be a
  dependency here. What it must export is a default object carrying an `id` and a `setup`.
- **`OpenCode-Session` and `Claude-Resume` are separate trailer keys** on purpose: neither client's
  id resumes the other. `commit-msg` reads `MONOSPACE_SESSION_ID` and `CLAUDE_CODE_SESSION_ID`. A
  commit made outside an agent shell correctly carries neither.
