---
status: accepted
scope: tooling
commitment: exploratory
date: 2026-09-21
decision-makers: Andrés Moschini
---

# Show the rendering

## Context and Problem Statement

This project's output is monospaced text, so it fits inside any document that would otherwise
describe it — a decision sheet, an ADR, a spec, a pull request body, a question asked in a session.
The repository mostly describes it instead. ADR-0044 shows the two tied routes side by side and then
spends forty lines on them; the picture had already decided.

The pictures that are there were typed by hand. A hand-drawn picture of what code does is an
assertion about behavior made without observing it — what principle IV forbids — wearing the one
disguise that makes it look like evidence.

## Decision Outcome

Chosen option: **where the subject renders, the artifact shows the rendering, and every picture in a
tracked file is either generated or labelled hypothetical** — because prose that a picture replaces
is the cheapest prose to remove, and the ambiguity between "this is what it does" and "this is what
it would do" is the one principle IV exists to prevent.

Generated means: the description lives beside the picture in the file, inside a `<!-- render: … -->`
marker; `cargo xtask render` rewrites the fence from it; and a step of `cargo xtask check` fails
when the two have parted. Hypothetical means hand-drawn, labelled on the spot, and skipped by that
step. Where an alternative is cheap to implement, a spike that generates its picture beats an
argument about what it would look like.

**Neither command exists yet.** Until they do the rule is enforced by review, the pictures already
in `docs/model.md` and in the arrow's records stay unlabelled, and each is labelled or regenerated
the next time its file is touched. That is why this record is `exploratory`: the half that makes it
mechanical is not written, and a rule enforced only by review is a hypothesis about whether it is
followed.

## Reversibility

Cheap, and additive in both directions. Nothing depends on a marker: a file without one renders in
every viewer exactly as it does today, and dropping the rule leaves the pictures in place. What
would grow the cost is the description format — the CLI's input is provisional by
[ADR-0035](0035-keep-the-cli-demo-format-out-of-the-model.md), and every tracked description becomes
something a change to that format has to migrate.

## Revisions

- 2026-09-21 — recorded, with `cargo xtask render` and its gate step not yet written.

## More Information

- [ADR-0035](0035-keep-the-cli-demo-format-out-of-the-model.md), the description format this would
  depend on, and its provisional status.
- `specs/045-simplify-cli-to-demo-shapes/contracts/description-format.md`, which documents it.
