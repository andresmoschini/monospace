---
status: draft
date: 2026-10-10
---

# A session above the diagram that owns it and undoes its changes

## Why now

[`docs/diagram-model.md`](../docs/diagram-model.md) settles that a diagram is a value and that
holding one is not the same as editing one, and it says so by declining to keep the history: a
removal "hands back nothing. There is no history and nothing to undo". That sentence is a boundary
the model drew around itself, and something has to live on the other side of it, because every
caller that holds a `&mut Diagram` today changes one and none of them can give the change back.

Nothing here is drawn and nothing is read from a file, which is what keeps the slice small: the
subject is a record of what a diagram looked like, and that is testable without a terminal, a screen
or a file. It is this cut before the interactive application because the application is the first
caller that wants to change a diagram at all, and a seam agreed after the first caller is a seam
shaped by whichever caller arrived first.

## Scope

### In

- `docs/session-model.md`, created, owning what holds a diagram while it is being changed and what
  gives a change back. It lands here rather than in the building stage because both existing model
  documents are design intent written before the code that implements them, and this one is a
  decision about two designs — a snapshot per step or an inverse per change — that the code cannot
  be written until it is made.
- `docs/diagram-model.md`, one sentence: the paragraph that says what it does not describe gains a
  pointer to the layer that does, the same move `docs/model.md` makes towards it.
- The two sentences that say every crate but the CLI stays portable, replaced by what the gate's
  `wasm` step actually checks: the table in `CONTRIBUTING.md`, its _What is in scope_, and one line
  of `README.md`.
- `specs/193-a-session-above-the-diagram-that-owns-it.md`, this file.

### Out

- **The crate.** The deciding stage writes no code. `monospace-session` and the `Clone` it needs are
  the building stage's, and this change has decided what they are rather than built them.
- **What is selected and what is on the screen.** A selection is a thing a caller holds and a screen
  is a thing a caller draws, and neither is a record of what a diagram used to be. Both belong to
  the caller above.
- **Reading and writing a diagram.** The description format stays where it is, private to
  `monospace-cli` by its own module doc. It later becomes a layer above the diagram that knows
  nothing of this one, and saying so now is what keeps the two from being built as one crate.
- **Merging a gesture into one step**, which D4 defers and `docs/session-model.md` records as
  deliberately unresolved.
- **A bound on how many steps are kept**, and **whether a session knows where its diagram came
  from**. Both are in that document's _Deliberately unresolved_, with what would settle each.
- **WebAssembly bindings, a web front end, non-terminal GUIs, persistence and collaboration**, which
  `CONTRIBUTING.md` names as renegotiated rather than quietly widened.

## The decision

**D1 — the session owns the diagram rather than borrowing it.**

**Answer:** a field of its own, and no method that hands out a `&mut Diagram`. **Why not** taking
`&mut Diagram` per operation: nothing would then stop a caller from changing the diagram without
going through the session, and the history would silently miss it — the one failure this layer
exists to make impossible. A private field with no mutable accessor makes it the compiler's to
enforce rather than a reviewer's, which is the argument `docs/diagram-model.md` already makes for
its own crate boundary. **What this does not claim:** `monospace-diagram` keeps its whole public
API, and a caller holding its own diagram edits it directly exactly as today. The guarantee is about
the one diagram a session owns. **Answered by** the maintainer, in the session that wrote this.

**D2 — a step holds a whole diagram, and no change is inverted.**

**Answer:** each step carries the diagram as it was, and undo is that diagram becoming the diagram
again. **Why not** the inverse of each change: `Diagram::remove` freezes every reference naming the
figure it takes and states that nothing re-hangs them, so an inverse cannot be derived from the
change — it can only be reconstructed from what a caller captured beforehand, and a command whose
author forgot is wrong in a way nothing reports. **Why not** the difference between two diagrams: it
carries the same ambiguity as a snapshot while being computed on every change, which is a cost on
the write path paid to save memory this domain has no use for. **What it costs:** memory
proportional to the diagram times the number of steps. **Answered by** the maintainer, in the
session that wrote this.

**D3 — a step carries the change beside the snapshot.**

**Answer:** a step is a diagram and the `Change` that replaced it. **Why not** the snapshot alone: a
diagram records what is true, never what was done to it, and the difference is not academic — a
figure dragged one cell and the same figure teleported across the diagram are identical as states
and different as gestures. Anything that later merges a run of small movements into one step has
nothing to read in a pair of diagrams. **What it costs:** one small value per step, and a `Change`
type for the model to name. **Answered by** the maintainer, in the session that wrote this.

**D4 — one change is one step, and the rule that merges a gesture is deferred.**

**Answer:** a caller asks, and what it asked for is one step. **Why not** writing the gesture rule
now: its input is a `Change` no caller produces yet. **Why not** the caller opening and closing a
gesture explicitly: a session that forgets to is wrong in a way that shows — extra steps rather than
lost ones — so moving the policy out of the layer that will hold it buys an API for a weaker
enforcement. **What reopens it:** the first caller that produces a gesture undo makes useless, which
a drag is. **Answered by** the maintainer, in the session that wrote this.

**D5 — the history is linear.**

**Answer:** a session holds a position among its steps, giving one back moves it, and a change made
behind the end discards the steps ahead. **Why not** a tree: branching earns its keep when a state
somebody discarded is worth returning to, and nothing here says one is. **What reopens it:** the
first time a caller wants a state the history threw away. **Answered by** the maintainer, in the
session that wrote this.

**D6 — the crate is `monospace-session`, and the document is `docs/session-model.md`.**

**Answer:** both named for what the thing is — an instance rather than a value. **Why not**
`monospace-commands`: D4 says a step may be several commands, so the name would be the one thing
about the layer that is provisional. **Why not** `monospace-application`: the interactive program is
the application, and the word belongs to the layer above. **Why not** `monospace-document`:
`docs/model.md` already gave the word to what is kept, and a session is not it. **Why not** a module
of the TUI with no document of its own: every decision here is observable from outside — whether
undo is available, what a step is, whether the diagram can be changed at all — and
`docs/diagram-model.md` §11 puts a rule in a module's rustdoc only "where nothing outside it can
observe the choice". **Answered by** the maintainer, in the session that wrote this.

**D7 — the gate's `wasm` step names the crate, and the prose stops claiming what it excludes.**

**Answer:** the building stage adds `-p monospace-session` to the step, and this change replaces the
two sentences that say every crate but the CLI with what the step checks. **Why not** naming the
exclusions in the step: a list of what is excluded is a second list to keep in step with the first,
and the omission is what keeps a crate added later out. **Why not** deferring the prose to the
building stage with the crate: the claim is already false today, because `xtask` is a crate the step
does not name, so the sentence describes a set that has never existed. **Answered by** the
maintainer, in the session that wrote this.

## Model slice

- `docs/session-model.md`, created. It owns the session: what it holds, that its diagram is changed
  only through it, what a step is, what undo restores and why it is linear, and what it does not
  reach. It lands there because D1, D2, D3, D5 and D6 are the model's kind of rule rather than a
  module's, and a rule that lives only in a new crate's rustdoc is a rule with one reader — which is
  the next change to need it.
- `docs/diagram-model.md`, one sentence. The paragraph beginning _What this document does not
  describe_ names `docs/session-model.md` and keeps the claim it makes about itself, the same way
  `docs/model.md` names this document. Nothing else in it moves.
- `docs/model.md`: none. The buffer, the cell and the glyph sets are untouched by a record of what a
  diagram used to be, and no open question in either document above is answered here — the nearest,
  _How does the core expose mutable state for editing?_, left the list when the diagram model
  answered it.

## Public surface

The deciding stage adds none of this; it is what the building stage publishes.

```rust
// monospace-session
pub struct Session { /* private */ }

impl Session {
    pub fn new(diagram: Diagram) -> Session;
    pub fn diagram(&self) -> &Diagram;
    pub fn change(&mut self, change: Change);
    pub fn undo(&mut self);
    pub fn redo(&mut self);
    pub fn can_undo(&self) -> bool;
    pub fn can_redo(&self) -> bool;
}

pub enum Change { /* the diagram's five, taking what the diagram's take */ }

// monospace-diagram
impl Clone for Diagram {}
```

No method returns `&mut Diagram`, and no method returns a `Diagram` either: `Diagram` is `Clone`, so
a caller that wants to keep one clones it, and a method here would be a second way to say the same
thing. Whether `Change` mirrors the five or names the intents behind them is not settled — see _Open
questions_.

## Behavior

1. A session owns the diagram it was created over.
2. The diagram a session owns is changed only by a `Change` the session was given.
3. No method of a session hands out a mutable reference to the diagram it owns.
4. A change is carried out and the diagram as it was is recorded as a step, whether or not the
   change altered the diagram.
5. Undo restores the diagram the step behind the position holds, whole.
6. Redo restores the diagram the step ahead of the position holds, whole.
7. Undoing with no step behind changes nothing, and redoing with no step ahead changes nothing.
8. A change made with steps ahead of the position discards them.
9. Whether undo and redo are available is answerable without carrying either out.
10. The diagram is drawn by borrowing it, and a drawing of what a session holds is equal to a
    drawing of the diagram it was created over whenever no change has been made.
11. Nothing of a session reaches a file, a screen, or what is selected.

## Examples

**A whole run over one figure.** Hypothetical: no code produces this yet, and none of it has been
observed. It is the acceptance list above drawn out, not the record of a run. The window is
`window(8, 3)` throughout, and the figure is `small_box(0, 0, no fill)`, so every picture below is
what the diagram model already draws — this change contributes the line under each one and nothing
above it.

```text
  session over [small_box(0,0,no fill)]

  no change
  ┌──┐
  │  │
  └──┘
  steps: 0   undo: no   redo: no

  Replace(1, the box displaced four cells right)
      ┌──┐
      │  │
      └──┘
  steps: 1   undo: yes  redo: no

  undo
  ┌──┐
  │  │
  └──┘
  steps: 1   undo: no   redo: yes

  redo
      ┌──┐
      │  │
      └──┘
  steps: 1   undo: yes  redo: no

  Remove(7)  —  a shape the diagram does not hold
      ┌──┐
      │  │
      └──┘
  steps: 2   undo: yes  redo: no

  undo
  undo
  ┌──┐
  │  │
  └──┘
  steps: 2   undo: no   redo: yes
```

Three of those lines carry rules on their own. The picture that does not move between the fourth and
the fifth blocks is rule 4: a change naming nothing is recorded all the same, because the session
does not know the caller meant to do something and a history that skipped a step the caller took is
worse than one longer than it needed to be. The `undo: no` on the seventh block is rule 7 at the end
of the history rather than in the middle of it. The second block is rule 10 — the picture is the
diagram model's, drawn through a borrow, and the session's whole contribution to it is that the
fifth and eighth blocks are the same picture again.

## What proves it

None of these tests exists yet; the deciding stage adds no code. The names are what the building
stage is held to.

| Rule | Test                                                              |
| ---- | ----------------------------------------------------------------- |
| 1    | `a_session_owns_the_diagram_it_was_created_over`                  |
| 2    | `the_diagram_changes_only_by_a_change_the_session_was_given`      |
| 3    | `no_method_of_a_session_hands_out_a_mutable_diagram`              |
| 4    | `a_change_naming_nothing_is_still_a_step`                         |
| 4    | `a_change_that_alters_the_diagram_is_a_step`                      |
| 5    | `undo_restores_the_diagram_the_step_behind_holds`                 |
| 6    | `redo_restores_the_diagram_the_step_ahead_holds`                  |
| 7    | `undo_with_nothing_behind_changes_nothing`                        |
| 7    | `redo_with_nothing_ahead_changes_nothing`                         |
| 8    | `a_change_with_steps_ahead_discards_them`                         |
| 9    | `whether_undo_is_available_is_answerable_without_carrying_it_out` |
| 10   | `drawing_what_a_session_holds_needs_no_copy`                      |
| 11   | nothing holds this, and it is named here rather than claimed done |

Rule 11 is not a rule a test can read: it is a statement about what the crate depends on and what it
knows, and a `Cargo.toml` is where a reader looks for it.

Two things are measured rather than tested. **That the `wasm` step has teeth** for the new crate:
`-p monospace-session` added to the step on purpose, the step run, its failure recorded, the
addition removed — reported in the building pull request. **That the crate compiles for
`wasm32-unknown-unknown`**, which the step does once it is named. The sweep is `Unchanged.` —
nothing renders and no product code changes, so no snapshot moves.

## Open questions

None. What surfaced while writing this became another issue rather than a section here:
[#194](https://github.com/andresmoschini/monospace/issues/194) asks whether a `Change` mirrors the
diagram's five changes or names the intents behind them, which is what decides whether D3's `Change`
is something a gesture rule can read or something a gesture rule has to be written against.
