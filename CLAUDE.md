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
- Talk to me in whatever language I write in.

## Habits the gate cannot check

Principles IV, VI and VIII are the rules and they are in context above; these are the failure modes
they have a habit of failing in.

- **Check a record's `commitment` before deferring to it**
  ([principle VI](.specify/memory/constitution.md#vi-decisions-recorded-at-the-altitude-they-belong-to)).
  An `exploratory` or `working` one is a hypothesis with a date, and changing it is expected work.
- **Say it when the code disagrees with a record**, in the first paragraph. Do not reconcile
  silently, and do not change code to match a document without asking which of the two is wrong.
- **Default a new decision to module-level**, into that module's rustdoc, and promote it in the
  increment that makes something outside able to observe it.
- **Look for the record of the subject before writing a new one**
  ([principle VI](.specify/memory/constitution.md#vi-decisions-recorded-at-the-altitude-they-belong-to)).
- **Show the rendering in the question, not beside it**
  ([principle IV](.specify/memory/constitution.md#show-the-rendering)). Both options, side by side,
  and a hand-drawn one labelled on the spot.
- **Cut the prose a picture already carries.** Keep the sentence that says why, drop the one that
  says what.
- **A snapshot that moved is a question, not a failure**
  ([Testing](.specify/memory/constitution.md#testing)). Report what moved, and ask whether that is
  the movement wanted.
- **Never argue that changing something repeatedly is expensive**
  ([principle VI](.specify/memory/constitution.md#vi-decisions-recorded-at-the-altitude-they-belong-to)).

## The flow

A feature and a tooling change are not the same shape, and which one a change is, what each stage
requires of `main`, and where the flow's state lives are in
[Spec Kit is the workflow](.specify/memory/constitution.md#spec-kit-is-the-workflow) and
[Two stages](.specify/memory/constitution.md#two-stages-and-where-the-cut-falls) in the
constitution, in [ADR-0033](docs/decisions/0033-keep-the-flow-state-in-labels-on-one-issue.md) and
in [ADR-0034](docs/decisions/0034-let-xtask-own-the-feature-branch.md). `CONTRIBUTING.md` has the
commands.

Work that has no issue yet is an issue, not a paragraph: what you find mid-feature that opens future
work becomes a `wish` issue of two lines, never a section of the current spec or an early ADR.

## Running a session

Every call re-reads the whole context, so a session costs its length squared rather than its length.
[ADR-0027](docs/decisions/0027-control-token-cost-through-session-discipline.md) holds the
measurement and the options it rejected.

- **One Spec Kit phase per session**, and `/speckit-plan` is two. Clear the context between each.
  The artifacts are the handoff; the conversation is not.
- **Part one of the plan ends at `decisions.md`**
  ([The plan runs in two parts](.specify/memory/constitution.md#the-plan-runs-in-two-parts)). Then
  say which entries need my answer and which you intend to take yourself.
- **Read the part, not the file.** `docs/learning-log.md`, `docs/model.md`, `docs/glyph-sets.md` and
  a feature's `spec.md` are large enough that opening one in full is a decision rather than a
  reflex. Take a line range, or grep with context. Appending needs no read at all.
- **Delegate a lookup that spans files**, so the files never enter this context.

## Commands

```sh
cargo xtask check          # the whole quality gate; the hook and CI run this and nothing else
cargo xtask fix            # every automatic fix the gate knows about
cargo xtask setup          # install the Node tooling the gate needs
cargo xtask render         # rewrite every picture a tracked document carries, from its description
cargo xtask spec new 23    # open the deciding stage; also `spec stage <n> build` and `spec use <n>`
cargo xtask pr body        # write target/pr-body.md; fill it, then `cargo xtask pr open`
cargo run -p monospace-cli -- path.json   # render one description; bare `cargo run` is ambiguous
cargo test --workspace     # tests only, for a faster loop
cargo insta review         # accept a moved snapshot; report what moved first
```

`CONTRIBUTING.md` covers setup, which tool owns which file, and what to do when a check fails.
