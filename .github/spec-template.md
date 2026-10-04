# NNN — the issue's own title, unaltered

<!--
  A spec: the record of what a change was decided to be, and the commitment that follows from it.

  One file per feature, flat, at `specs/NNN-slug.md` where `NNN` is the issue number in three
  digits and the slug is its title lowercased and hyphenated. Not a directory per feature: the
  shape that let eight files appear is the shape to stay away from.

  Written by the deciding pull request, which creates this file and opens the deciding branch with
  `cargo xtask feature new <issue> --rung decide`. Completed by the building pull request, and
  never deleted — the reasoning behind a decision is worth more than the fact that it held.

  THIS IS NOT THE PULL REQUEST BODY, and it is not a copy of it. The body is what a reviewer reads
  while the change is being agreed. This is what someone reads afterwards, without the
  conversation, and it is therefore more precise than the body rather than less. Everything said
  here has to stand on its own.

  So: name things. `Anchor`, not "the anchor type"; `Diagram::remove`, not "removal"; `#90`, not
  "the issue". Say what happens, with the values, because a behavior a reader has to infer from
  three sentences is a behavior nobody implemented. Carry the picture, because a paragraph is what
  goes stale quietly — a render marker works here as it does in any tracked Markdown file, so leave
  the fence empty and run `cargo xtask render`. Record counts, measured: "fifteen cells change and
  nine are blank" is checkable and "some cells change" is not.

  The status line below is required, and the gate checks it. It answers the two questions a reader
  of a year-old spec cannot otherwise answer: where this came from, and whether it still describes
  the code. There are exactly two forms. The first says the code is what was decided. The second
  says it is not — and then the gate refuses this file while the last section is empty, because a
  spec that knows it was revised and does not say how is the one failure this document exists to
  prevent.
-->

Status: decided YYYY-MM-DD in #NNN — the code matches it

## The decisions

<!--
  What this change is agreed to do. One entry per decision: the question, the answer, and why the
  alternative was not taken. The third column is what makes this an agreement rather than an
  opinion — a decision with no rejected alternative has not been made, it has been typed.

  Where the subject renders, show it. Two options side by side is an argument; two paragraphs
  describing them is a preference.
-->

| #   | Question | Answer | Why not the alternative |
| --- | -------- | ------ | ----------------------- |
| D1  |          |        |                         |

## Public surface

<!--
  What a caller outside this crate can observe, and whether any of it breaks.

  Name every item: a type, a variant, a signature, a behavior. Say what a caller has to do about
  it, and what happens to a caller who does nothing.

  Two kinds of change are worth keeping apart, because they fail differently. Widening — a new
  variant, a new method, a new accepted input — stops a caller matching exhaustively from
  compiling: loud, and cheap. Changing what something means while it keeps its shape compiles
  perfectly and draws something different: silent, and the expensive one. It gets its own line for
  that reason.

  For a library this section is not optional. A caller who is not in the conversation cannot be
  asked afterwards. "None." is a real answer, for a change that is internal or a pure refactor.
-->

## What this does not decide

<!--
  What is deliberately left open, and what would force an answer later.

  This is not a list of non-events to be polite about. It is what keeps a rule local: a slice that
  answers one question well and refuses three others is a better change than one that answers all
  four badly. Write down the condition that would reopen each one — "a second consumer for the
  corners" is useful; "not now" is not.
-->

## What proves it

<!--
  How anyone can tell this was built correctly, named before it is built.

  Each behavior rule gets a line: the rule, and what will hold it. Name the assertion where one
  exists, or the picture where a picture is the only honest carrier. A rule with nothing against it
  is written here as such, rather than quietly counted as covered.

  This is the section that makes the proposal checkable rather than merely plausible. A change that
  cannot be observed is a wish with better formatting, and finding that out here costs one
  conversation instead of a merged pull request.
-->

## What the implementation changed

<!--
  Left empty by the deciding pull request. The building pull request fills it if — and only if —
  building the decisions showed that one of them was wrong, incomplete, or answerable better.

  Write what changed, why it could not have been foreseen, and whether it came back to the
  maintainer before it was taken. "None." when the decisions held, which is the good answer and the
  common one.

  This is why a spec is never deleted: this section is the record of a decision turning out to be
  wrong, and it is readable in a way a commit message is not.
-->

None.
