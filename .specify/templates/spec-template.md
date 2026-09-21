# Feature Specification: [FEATURE NAME]

**Feature Branch**: `[NNN-slug]` | **Issue**: [#NNN] | **Created**: [DATE] | **Status**: Draft

**Input**: "$ARGUMENTS"

<!--
  Ceiling: 120 lines. Exceeding it is a signal that the slice is too thick — propose the split
  rather than compressing the prose (constitution, principle VIII).

  This spec says what is wanted and how it will be recognised. It takes no decisions: a decision
  surfaced while writing goes on decisions.md, which /speckit-plan part one produces.

  It also does not restate the model. It names the sections of docs/model.md it implements and
  links to them; if a rule is needed that the model does not have, the model changes first
  (constitution, _The model owns the design_).
-->

## What this slice implements

<!-- Which sections of docs/model.md or docs/diagram-model.md this realises, and which part of each.
     Where the slice implements a section only in part, say which part is left. -->

- [_Section name_](../../docs/model.md#anchor) — [what of it, in one line]

## Clarifications

<!-- Filled by /speckit-clarify. One dated session per run, question and answer. -->

### Session [DATE]

- Q: [question] → A: [answer]

## Behaviour

<!-- One numbered group per capability this slice adds, in the order a reader should meet them.
     Each scenario is Given / When / Then, in the vocabulary of the model.

     Where the observable result is a picture, SHOW the picture and drop the sentence describing it:
     it is shorter, it is what will be asserted, and principle IV asks for it. A picture here is
     generated from a description the file carries, or labelled as hypothetical where the behaviour
     does not exist yet — which, in a spec, it usually does not. A hypothetical picture in a spec is
     the requirement; the contract test is what turns it into a generated one. -->

### B1 — [brief title]

1. **Given** [initial state], **When** [action], **Then** [expected outcome]
2. **Given** […], **When** […], **Then** […]

### B2 — [brief title]

1. **Given** […], **When** […], **Then** […]

## Edge cases

<!-- Arrangements at the boundary of the rule, and what the rule yields for each. A degenerate
     arrangement draws whatever the general rule gives it and does not acquire an exception
     (docs/model.md, _Degenerate arrangements_) — so an entry here states the rule's answer, and
     where it does not have one, that is a decision for decisions.md. -->

- [boundary arrangement] → [what the rule yields]

## What this slice does not decide

<!-- Mandatory, and the section most worth writing carefully.

     Name what is deliberately left open, and why it can wait. A question named here is not
     answered by this feature: not in the plan, not in research, not in passing during
     implementation. If one of these has to be answered to finish the slice, it was not a
     non-decision — move it to decisions.md and say so.

     This is what keeps a rule local: a decision that nothing forced is a decision not taken. -->

- [the open question] — [what would force an answer, which is when it gets one]

## Testing expectations

<!-- Constitution, Testing: every behaviour rule names a test, and a rule with none is an
     unfinished spec. Say which of the two kinds each is.

     A contract test pins a decision and names the rule it holds. A characterization test records
     what the code does over a range too wide to read, and its file says so at its head. -->

- **Contract** — [which rule of the model, pinned how]
- **Characterization** — [what range, and what makes it too wide to assert by hand]

## Success criteria

<!-- Observable from outside the change, and checkable. Not business metrics: this is a library, so
     a criterion is something a caller or the gate can see. Three or four is usually enough. -->

- **SC-001**: [observable, checkable outcome]
- **SC-002**: [observable, checkable outcome]
