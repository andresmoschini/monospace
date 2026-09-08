---
status: "draft | agreed | implemented | abandoned"
date: YYYY-MM-DD
---

# NNNN — {Short title, in the imperative: what the slice does}

> **No longer in use.** New features are specified with Spec Kit, whose template is resolved from
> `.specify/`. This file is kept because the five specs in this directory were written against it.
> See [the directory's note](README.md).

## Why now

{One paragraph. What does this unlock? Why this slice before the others? If the honest answer is
"because it was next in the list", say what would break if it came later instead.}

## Scope

### In

{What exists when this is done. One bullet per thing, no verbs like "improve" — either it exists or
it does not.}

### Out

{What is deliberately left out, and where it is handled instead: a later spec, an ADR, the model. An
item with no destination is a question, not a scope decision — move it to Open questions.}

{The important ones are the near misses: the things a reader would assume are included.}

## Model slice

{Which sections of `docs/model.md` this implements, by number. Nothing here should be new: if the
slice needs a rule the model does not have, the model is what changes first.}

## Public surface

{The types and functions that appear in the public API, with signatures. Not the implementation —
whether something is a `Vec` or a `HashMap` inside is not a promise to anyone.}

{Also: what is documented with rustdoc, since the brief asks for it as the API is introduced rather
than later.}

## Behavior

{Numbered rules. One rule per number, each one testable on its own, each one falsifiable. "Renders
correctly" is not a rule; "a position with no cell renders as a space" is.}

## Examples

{Input and expected output, close enough to a test that writing the test is transcription. Cover at
least: the ordinary case, each boundary named in the rules, and the case someone would get wrong.}

{Where output is text, show it in a fenced block with its exact spacing.}

## Acceptance

{The list that gets ticked. Every item is something to run or look at, not something to believe.}

- [ ] `cargo xtask check` passes.
- [ ] `cargo run -p monospace-cli` shows {what}.
- [ ] {a test per behavior rule, named}

## Open questions

{What surfaced while writing this and is deliberately left open, with what would settle it. An empty
section is a fine answer; an absent one means nobody looked.}
