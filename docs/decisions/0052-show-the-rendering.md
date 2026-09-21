---
status: accepted
scope: tooling
commitment: working
date: 2026-09-21
decision-makers: Andrés Moschini
---

# Show the rendering

## Context and Problem Statement

This project's output is monospaced text, so it fits inside any document that would otherwise
describe it — a decision sheet, an ADR, a spec, a pull request body, a question asked in a session.
The repository mostly describes it instead. ADR-0044 shows the two tied routes side by side and then
spends forty lines on them; the picture had already decided.

The pictures that are there were typed by hand: an assertion about behavior made without observing
it, which principle IV forbids, wearing the one disguise that makes it look like evidence.

## Decision Outcome

Chosen option: **where the subject renders, the artifact shows the rendering, and every picture in a
tracked file is either generated or labelled hypothetical** — because prose that a picture replaces
is the cheapest prose to remove, and the ambiguity between "this is what it does" and "this is what
it would do" is the one principle IV exists to prevent.

Generated means: the description lives beside the picture in the file, inside a `<!-- render: … -->`
marker; `cargo xtask render` rewrites the fence from it; and a step of `cargo xtask check` fails
when the two have parted. Hypothetical means hand-drawn, labelled on the spot, and skipped by that
step.

## Reversibility

Cheap. Nothing depends on a marker: a file without one renders in every viewer exactly as it does
today, and dropping the rule leaves the pictures in place. What would grow the cost is the
description format — provisional by ADR-0035, and every tracked description is something a change to
it has to migrate.

## Revisions

- 2026-09-21 — recorded, with `cargo xtask render` and its gate step not yet written.
- 2026-09-21 — both written, and this becomes `working`: the gate depends on it. The pictures
  already in tracked files stay unlabelled until their file is next touched
  ([#112](https://github.com/andresmoschini/monospace/issues/112)), because nothing mechanical can
  see a picture that is outside a marker. Three things the building settled:
  - **A marker carries its description, not a path to one.** The draft showed a path; inline is what
    this record's own wording asks for and what principle VIII's exemption presupposes. The decision
    sheet's template was corrected to it.
  - **The comparative `A=… B=…` form is not built.** Two options are two markers. Side by side under
    one fence would put a caption row and a gutter — hand-written text — inside what the gate
    regenerates whole.
  - **`monospace-cli` renders a file once.** A picture bound for a fence cannot arrive wrapped in
    the captions and the moved shape that are the demonstration's business.

## More Information

- [ADR-0035](0035-keep-the-cli-demo-format-out-of-the-model.md), and
  `specs/079-a-diagram-holds-shapes-and-draws-itself/contracts/description-format.md`, which
  documents the provisional format every marker is written in.
