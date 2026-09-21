# Monospace

ASCII diagramming project in Rust. What it is, what is in scope, and every rule that governs a
change to it are in the constitution, imported here so it is in context rather than waiting to be
read:

@.specify/memory/constitution.md

Everything below is what only this file owns. Where the constitution states a rule, it is not
restated here — two wordings of one rule is a rule that gets followed at random.

## Working with me

- Do not invent something new because I did not know the ecosystem well enough to ask for the usual
  thing. When it is settled practice, the conventional answer is the answer.
- Talk to me in whatever language I write in. What lands in the repository is English regardless.
- This system is under construction and its architecture is meant to change. A record tells you what
  was understood when it was written, not what may be thought now.

## Habits the gate cannot check

Principles IV, VI and VIII are the rules; these are what they ask for inside a session, and what is
most easily lost in a long one.

- **Check `commitment` before deferring to a record.** An `exploratory` or `working` ADR is a dated
  hypothesis, and proposing to change it is expected work.
- **Say it when the code disagrees with a record**, in the first paragraph. Do not reconcile
  silently, and do not change code to match a document without asking which of the two is wrong.
- **Default a new decision to module-level**, into that module's rustdoc.
- **Look for the record of the subject before writing a new one.** If you cannot cite the record you
  are about to write without citing another, they are one record and this is a revision.
- **Never ask me about something that renders without showing it.** Generate both options and put
  them side by side; generating them is part of asking. Label a hand-drawn one.
- **Cut the prose a picture already carries.** Keep the sentence that says why, drop the one that
  says what.
- **A snapshot that moved is a question, not a failure.** Report how many cases moved, in which
  families, and three examples with before and after, then ask whether that is the movement wanted.
- **Do not write that changing something repeatedly is expensive.**

## The flow

Two kinds of work, and they are not the same shape. A feature goes through Spec Kit; a change to the
tooling goes through an ADR and commits, with no spec directory and no staged branches.

A feature crosses two stages — deciding, then building — each its own branch and its own pull
request against `main`, with the merge as the handoff. The state is a label on the feature's single
issue, `wish` then `deciding` then `building`, and lives nowhere else
([ADR-0033](docs/decisions/0033-keep-the-flow-state-in-labels-on-one-issue.md)). `cargo xtask spec`
owns the branches and the labels
([ADR-0034](docs/decisions/0034-let-xtask-own-the-feature-branch.md)).

Work that has no issue yet is an issue, not a paragraph: what you find mid-feature that opens future
work becomes a `wish` issue of two lines, never a section of the current spec or an early ADR.

`CONTRIBUTING.md` explains the rest.

## Running a session

Every call re-reads the whole context, so a session costs its length squared rather than its length.
[ADR-0027](docs/decisions/0027-control-token-cost-through-session-discipline.md) holds the
measurement and the options it rejected.

- **One Spec Kit phase per session**, and `/speckit-plan` is two. Clear the context between each.
  The artifacts are the handoff; the conversation is not.
- **Part one of the plan ends at `decisions.md`** — no `data-model.md`, no `contracts/`, no
  `quickstart.md`. Then say which entries need my answer and which you intend to take yourself.
- **Read the part, not the file.** `docs/learning-log.md`, `docs/model.md`, `docs/glyph-sets.md` and
  a feature's `spec.md` are large enough that opening one in full is a decision rather than a
  reflex. Take a line range, or grep with context. Appending needs no read at all.
- **Delegate a lookup that spans files**, so the files never enter this context.

## Commands

```sh
cargo xtask check          # the whole quality gate; the hook and CI run this and nothing else
cargo xtask fix            # every automatic fix the gate knows about
cargo xtask setup          # install the Node tooling the gate needs
cargo xtask spec new 23    # open the deciding stage; also `spec stage <n> build` and `spec use <n>`
cargo xtask pr body        # write target/pr-body.md; fill it, then `cargo xtask pr open`
cargo run -p monospace-cli -- path.json   # render one description; bare `cargo run` is ambiguous
cargo test --workspace     # tests only, for a faster loop
cargo insta review         # accept a moved snapshot; report what moved first
```

One command the rules describe is not written yet: `cargo xtask render`
([ADR-0052](docs/decisions/0052-show-the-rendering.md)). Follow the rule by hand and say so.

`CONTRIBUTING.md` covers setup, which tool owns which file, and what to do when a check fails.
