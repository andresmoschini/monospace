<!-- markdownlint-configure-file { "MD041": false } -->
<!-- A pull request body is not a document: its title is the pull request's, and a top-level
     heading here renders oversized on GitHub. Every other rule still applies. -->

<!--
  The DECIDING stage, and only on the **Decide** rung. Branch `NNN-slug-deciding`. A proposal and the
  agreement on it. No code.

  Open the branch first — `cargo xtask feature new <issue> --rung decide` — which reads the issue's
  title for the slug and moves the issue's label to `deciding`. Then, once the proposal is written
  and agreed: `cargo xtask pr body`, fill it in, `cargo xtask pr open`. The keyword is appended for
  you, and here it is `Refs`.

  This body answers one question: "is this what is wanted, and is this how it will be resolved?"
  It does not restate the proposal — it is right above. Every section stays, and one with nothing
  to say says "None." rather than disappearing: a missing section reads as an oversight, an explicit
  "None." reads as an answer.

  **No line count, deliberately.** Sixty was here and it was not kept, which makes a number in this
  file one more value nothing checks. What bounds the body instead: every line either names a
  decision with the alternative that was rejected, or names something a caller outside this crate
  can observe. A line that would be true of any feature is a line to cut.

  Render blocks do not count toward that, and it is not a courtesy. A picture costs less attention
  than a paragraph, so it is the cheaper half of an argument rather than a way around the limit.
-->

## What the issue asks

<!-- One or two lines. The keyword line is appended by `cargo xtask pr body`, so do not
     write one here — this stage references the issue and does not close it. -->

## Which slice of the model this is

<!-- The sections of docs/model.md or docs/diagram-model.md this implements, linked, and what of
     each. Name them even when this stage amends none of them, because reading the model is what
     decides whether this was a **Decide** rung at all. -->

## The spec

<!-- The file this pull request creates, at `specs/NNN-slug.md`, copied from
     `.github/spec-template.md`. It is the more precise of the two artifacts and it outlives this
     conversation: this body is what a reviewer reads today, the spec is what someone reads in a
     year. Say where it is and which of its sections carry the substance, so a reader knows which
     one to believe when they disagree. -->

## The decisions

<!-- What this stage is actually approving. Per entry: the question, the answer, and why the
     alternative was not taken. Where the subject renders, SHOW the two options; the picture is
     the argument. -->

| #   | Question | Answer | Why not the alternative |
| --- | -------- | ------ | ----------------------- |
| D1  |          |        |                         |

## Public surface

<!-- What a caller outside this crate can observe, and whether any of it breaks. Name the items —
     a type, a variant, a signature, a behavior a test could pin — and say whether an existing
     caller has to change. A silent widening is still a change to something that was already
     public, and this repository is a library as well as an application: a caller who is not in
     this conversation cannot be asked afterwards.

     Two kinds of change are worth separating. **Widening** — a new variant, a new method — breaks
     a caller who matches exhaustively, so say so. **Repointing** the meaning of something that
     stays is worse, because nothing fails to compile: the caller still builds and draws something
     different. That one needs its own line.

     "None." is a real answer, for a slice that is internal or a pure refactor. -->

## What this slice does not decide

<!-- What is deliberately left open, and what would force an answer later. This is the part that
     keeps a rule local, so it is worth a reviewer's attention even when it looks like a list of
     non-events. -->

## Docs touched

<!-- **This stage does not amend the model.** The model moves in the pull request that carries the
     code which proves it wrong, so the two never ship apart — see *The model moves* in
     CONTRIBUTING.md. What belongs here is a record of what this stage decided the model will have
     to say, so the building stage starts knowing which sections it will be editing.

     "None." is a good answer for a slice that decided nothing the model has to carry. -->

## Not in this PR

<!-- Say it explicitly: no code. Writing it here would be taking the decisions this PR exists to
     ask about. -->
